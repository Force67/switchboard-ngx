use anyhow::Context;
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use rand::RngCore;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool, Transaction};
use switchboard_config::{AuthConfig, GithubAuthConfig};
use thiserror::Error;
use tracing::{debug, info};
use cuid2::CuidConstructor;

const GITHUB_USER_API: &str = "https://api.github.com/user";

static CUID: Lazy<CuidConstructor> = Lazy::new(CuidConstructor::new);

#[derive(Clone)]
pub struct Authenticator {
    pool: SqlitePool,
    session_ttl: Duration,
    github: Option<GithubOAuth>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("user already exists")]
    UserExists,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("github oauth is not configured")]
    GithubOauthDisabled,
    #[error("github oauth error: {0}")]
    GithubOauth(#[from] anyhow::Error),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("password hashing failed: {0}")]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error("session not found")]
    SessionNotFound,
    #[error("session expired")]
    SessionExpired,
    #[error("invalid session token")]
    InvalidSession,
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: i64,
    pub public_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub token: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GithubProfile {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

impl Authenticator {
    pub fn new(pool: SqlitePool, config: AuthConfig) -> Self {
        let session_ttl = Duration::seconds(config.session_ttl_seconds as i64);
        let github = GithubOAuth::from_config(&config.github);

        Self {
            pool,
            session_ttl,
            github,
        }
    }

    pub fn pool(&self) -> SqlitePool {
        self.pool.clone()
    }

    pub fn github_enabled(&self) -> bool {
        self.github.is_some()
    }

    pub fn github_authorization_url(
        &self,
        state: &str,
        redirect_uri: &str,
    ) -> Result<String, AuthError> {
        let github = self.github.as_ref().ok_or(AuthError::GithubOauthDisabled)?;
        github
            .authorize_url(state, redirect_uri)
            .map_err(AuthError::GithubOauth)
    }

    pub async fn register_with_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, AuthError> {
        let mut tx = self.pool.begin().await?;

        let existing = sqlx::query("SELECT id FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_some() {
            return Err(AuthError::UserExists);
        }

        let now = Utc::now();
        let password_hash = self.hash_password(password)?;

        let user = self
            .insert_user(&mut tx, Some(email.to_owned()), None)
            .await?;

        sqlx::query(
            "INSERT INTO user_identities (user_id, provider, provider_uid, secret, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user.id)
        .bind("password")
        .bind(email)
        .bind(password_hash)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user)
    }

    pub async fn login_with_password(
        &self,
        email: &str,
        password: &str,
    ) -> Result<AuthSession, AuthError> {
        let identity = sqlx::query(
            "SELECT user_id, secret FROM user_identities WHERE provider = 'password' AND provider_uid = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = identity else {
            return Err(AuthError::InvalidCredentials);
        };

        let secret: String = row.try_get("secret")?;
        let stored_hash = PasswordHash::new(&secret)?;
        Argon2::default()
            .verify_password(password.as_bytes(), &stored_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        let user_id: i64 = row.try_get("user_id")?;
        self.fetch_user(user_id).await?;

        self.issue_session(user_id).await
    }

    pub async fn login_with_github_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<AuthSession, AuthError> {
        let github = self.github.as_ref().ok_or(AuthError::GithubOauthDisabled)?;

        let profile = github
            .exchange_code(code, redirect_uri)
            .await
            .map_err(AuthError::GithubOauth)?;

        self.login_with_github_profile(profile).await
    }

    pub async fn login_with_github_profile(
        &self,
        profile: GithubProfile,
    ) -> Result<AuthSession, AuthError> {
        let mut tx = self.pool.begin().await?;

        if let Some(row) = sqlx::query(
            "SELECT user_id FROM user_identities WHERE provider = 'github' AND provider_uid = ?",
        )
        .bind(&profile.id)
        .fetch_optional(&mut *tx)
        .await?
        {
            let user_id: i64 = row.try_get("user_id")?;
            tx.commit().await?;
            return self.issue_session(user_id).await;
        }

        let (user, email) = if let Some(email) = profile.email.as_ref() {
            if let Some(row) = sqlx::query("SELECT id FROM users WHERE email = ?")
                .bind(email)
                .fetch_optional(&mut *tx)
                .await?
            {
                let user_id: i64 = row.try_get("id")?;
                let user = self.fetch_user(user_id).await?;
                (user, Some(email.clone()))
            } else {
                let user = self
                    .insert_user(&mut tx, Some(email.clone()), profile.name.clone())
                    .await?;
                (user, Some(email.clone()))
            }
        } else {
            let user = self
                .insert_user(&mut tx, None, profile.name.clone())
                .await?;
            (user, None)
        };

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO user_identities (user_id, provider, provider_uid, secret, created_at, updated_at) VALUES (?, ?, ?, NULL, ?, ?)",
        )
        .bind(user.id)
        .bind("github")
        .bind(&profile.id)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(user = %user.public_id, email = ?email, "linked github identity");
        self.issue_session(user.id).await
    }

    pub async fn authenticate_token(&self, token: &str) -> Result<(User, AuthSession), AuthError> {
        let row = sqlx::query("SELECT user_id, expires_at FROM sessions WHERE token = ?")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else {
            return Err(AuthError::SessionNotFound);
        };

        let user_id: i64 = row.try_get("user_id")?;
        let expires_at: String = row.try_get("expires_at")?;

        let expires_at = DateTime::parse_from_rfc3339(&expires_at)
            .map_err(|_| AuthError::InvalidSession)?
            .with_timezone(&Utc);

        if expires_at <= Utc::now() {
            sqlx::query("DELETE FROM sessions WHERE token = ?")
                .bind(token)
                .execute(&self.pool)
                .await?;
            return Err(AuthError::SessionExpired);
        }

        let user = self.fetch_user(user_id).await?;
        let session = AuthSession {
            token: token.to_owned(),
            user_id,
            expires_at,
        };

        Ok((user, session))
    }

    pub async fn user_profile(&self, user_id: i64) -> Result<User, AuthError> {
        self.fetch_user(user_id).await
    }

    async fn insert_user(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        email: Option<String>,
        display_name: Option<String>,
    ) -> Result<User, AuthError> {
        let now = Utc::now().to_rfc3339();
        let public_id = new_public_id();

        sqlx::query(
            "INSERT INTO users (public_id, email, display_name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&public_id)
        .bind(email.as_ref().map(|value| value.as_str()))
        .bind(display_name.as_ref().map(|value| value.as_str()))
        .bind(&now)
        .bind(&now)
        .execute(&mut **tx)
        .await?;

        let row = sqlx::query("SELECT id FROM users WHERE public_id = ?")
            .bind(&public_id)
            .fetch_one(&mut **tx)
            .await?;

        Ok(User {
            id: row.try_get("id")?,
            public_id,
            email,
            display_name,
        })
    }

    async fn fetch_user(&self, id: i64) -> Result<User, AuthError> {
        let row = sqlx::query(
            "SELECT id, public_id, email, display_name, \n                CASE WHEN email IS NULL THEN 0 ELSE 1 END AS email_present,\n                CASE WHEN display_name IS NULL THEN 0 ELSE 1 END AS display_name_present\n             FROM users WHERE id = ?",
        )
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let email_present: i64 = row.try_get("email_present")?;
        let email = if email_present != 0 {
            Some(row.try_get::<String, _>("email")?)
        } else {
            None
        };

        let display_name_present: i64 = row.try_get("display_name_present")?;
        let display_name = if display_name_present != 0 {
            Some(row.try_get::<String, _>("display_name")?)
        } else {
            None
        };

        Ok(User {
            id,
            public_id: row.try_get("public_id")?,
            email,
            display_name,
        })
    }

    async fn issue_session(&self, user_id: i64) -> Result<AuthSession, AuthError> {
        let token = self.generate_session_token();
        let now = Utc::now();
        let expires_at = now + self.session_ttl;

        sqlx::query(
            "INSERT INTO sessions (user_id, token, created_at, expires_at) VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(&token)
        .bind(now.to_rfc3339())
        .bind(expires_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(AuthSession {
            token,
            user_id,
            expires_at,
        })
    }

    fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default().hash_password(password.as_bytes(), &salt)?;
        Ok(hash.to_string())
    }

fn generate_session_token(&self) -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }
}

fn new_public_id() -> String {
    CUID.create_id()
}

#[derive(Clone)]
struct GithubOAuth {
    client: BasicClient,
    http: reqwest::Client,
}

impl GithubOAuth {
    fn from_config(config: &GithubAuthConfig) -> Option<Self> {
        let client_id = config.client_id.clone()?;
        let client_secret = config.client_secret.clone()?;
        Some(Self::new(client_id, client_secret))
    }

    fn new(client_id: String, client_secret: String) -> Self {
        let client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                .expect("invalid github auth url"),
            Some(
                TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .expect("invalid github token url"),
            ),
        )
        .set_auth_type(oauth2::AuthType::RequestBody);

        let http = reqwest::Client::builder()
            .user_agent("switchboard-backend")
            .build()
            .expect("failed to build github http client");

        Self { client, http }
    }

    fn authorize_url(&self, state: &str, redirect_uri: &str) -> anyhow::Result<String> {
        let redirect = RedirectUrl::new(redirect_uri.to_owned())
            .context("invalid redirect uri for github oauth")?;

        let (url, _) = self
            .client
            .clone()
            .set_redirect_uri(redirect)
            .authorize_url(|| CsrfToken::new(state.to_owned()))
            .add_scope(Scope::new("read:user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
            .url();

        Ok(url.to_string())
    }

    async fn exchange_code(&self, code: &str, redirect_uri: &str) -> anyhow::Result<GithubProfile> {
        let redirect = RedirectUrl::new(redirect_uri.to_owned())
            .context("invalid redirect uri for github oauth")?;

        let token_response = self
            .client
            .clone()
            .set_redirect_uri(redirect)
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .request_async(async_http_client)
            .await
            .context("failed to exchange github oauth code")?;

        let access_token = token_response.access_token().secret();

        let user: GithubUserResponse = self
            .http
            .get(GITHUB_USER_API)
            .bearer_auth(access_token)
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .context("failed to call github user api")?
            .error_for_status()
            .context("github user api returned error")?
            .json()
            .await
            .context("failed to decode github user response")?;

        debug!(login = %user.login, id = user.id, "fetched github user profile");

        Ok(GithubProfile {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        })
    }
}

#[derive(Deserialize)]
struct GithubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
}
