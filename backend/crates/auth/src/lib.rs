use anyhow::Context;
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl};
use rand::RngCore;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use sqlx::{Any, AnyPool, Row, Transaction};
use switchboard_config::{AuthConfig, GithubAuthConfig};
use thiserror::Error;
use tracing::{debug, info};
use uuid::Uuid;

const GITHUB_USER_API: &str = "https://api.github.com/user";

#[derive(Clone)]
pub struct Authenticator {
    pool: AnyPool,
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
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GithubProfile {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

impl Authenticator {
    pub fn new(pool: AnyPool, config: AuthConfig) -> Self {
        let session_ttl = Duration::seconds(config.session_ttl_seconds as i64);
        let github = GithubOAuth::from_config(&config.github);

        Self {
            pool,
            session_ttl,
            github,
        }
    }

    pub fn pool(&self) -> AnyPool {
        self.pool.clone()
    }

    pub fn github_enabled(&self) -> bool {
        self.github.is_some()
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

        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let password_hash = self.hash_password(password)?;

        sqlx::query(
            "INSERT INTO users (id, email, display_name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind(email)
        .bind(None::<String>)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO user_identities (id, user_id, provider, provider_uid, secret, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(user_id.to_string())
        .bind("password")
        .bind(email)
        .bind(password_hash)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(User {
            id: user_id,
            email: Some(email.to_owned()),
            display_name: None,
        })
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

        let user_id: String = row.try_get("user_id")?;
        let user_id = Uuid::parse_str(&user_id).map_err(|_| AuthError::InvalidCredentials)?;
        self.fetch_user(user_id).await?;

        self.issue_session(user_id).await
    }

    pub async fn login_with_github_code(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<AuthSession, AuthError> {
        let github = self
            .github
            .as_ref()
            .ok_or(AuthError::GithubOauthDisabled)?;

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
        .await? {
            let user_id: String = row.try_get("user_id")?;
            let user_id = Uuid::parse_str(&user_id).context("invalid user id stored for github identity")?;
            tx.commit().await?;
            return self.issue_session(user_id).await;
        }

        let (user_id, email) = if let Some(email) = profile.email.as_ref() {
            if let Some(row) = sqlx::query("SELECT id FROM users WHERE email = ?")
                .bind(email)
                .fetch_optional(&mut *tx)
                .await? {
                let user_id: String = row.try_get("id")?;
                let user_id = Uuid::parse_str(&user_id).context("invalid stored user id")?;
                (user_id, Some(email.clone()))
            } else {
                let new_id = Uuid::new_v4();
                self.insert_user(&mut tx, new_id, Some(email.clone()), profile.name.clone())
                    .await?;
                (new_id, Some(email.clone()))
            }
        } else {
            let new_id = Uuid::new_v4();
            self.insert_user(&mut tx, new_id, None, profile.name.clone()).await?;
            (new_id, None)
        };

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO user_identities (id, user_id, provider, provider_uid, secret, created_at, updated_at) VALUES (?, ?, ?, ?, NULL, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(user_id.to_string())
        .bind("github")
        .bind(&profile.id)
        .bind(&now)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(user = %user_id, email = ?email, "linked github identity");
        self.issue_session(user_id).await
    }

    async fn insert_user(
        &self,
        tx: &mut Transaction<'_, Any>,
        user_id: Uuid,
        email: Option<String>,
        display_name: Option<String>,
    ) -> Result<(), AuthError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO users (id, email, display_name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind(email)
        .bind(display_name)
        .bind(&now)
        .bind(&now)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn fetch_user(&self, id: Uuid) -> Result<User, AuthError> {
        let row = sqlx::query("SELECT id, email, display_name FROM users WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await?;

        Ok(User {
            id,
            email: row.try_get::<Option<String>, _>("email")?,
            display_name: row.try_get::<Option<String>, _>("display_name")?,
        })
    }

    async fn issue_session(&self, user_id: Uuid) -> Result<AuthSession, AuthError> {
        let token = self.generate_session_token();
        let now = Utc::now();
        let expires_at = now + self.session_ttl;

        sqlx::query(
            "INSERT INTO sessions (id, user_id, token, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(user_id.to_string())
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
