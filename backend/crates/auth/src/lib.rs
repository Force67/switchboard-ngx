use anyhow::Context;
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use cuid2::CuidConstructor;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use once_cell::sync::Lazy;
use rand::RngCore;
use reqwest::{header::ACCEPT, Url};
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool, Transaction};
use switchboard_config::{AuthConfig, GithubAuthConfig};
use thiserror::Error;
use tracing::{debug, info};

const GITHUB_USER_API: &str = "https://api.github.com/user";

static CUID: Lazy<CuidConstructor> = Lazy::new(CuidConstructor::new);

const MAX_DISPLAY_NAME_LEN: usize = 64;
const MAX_USERNAME_LEN: usize = 64;
const MAX_BIO_LEN: usize = 512;
const MAX_AVATAR_URL_LEN: usize = 2048;

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
    #[error("invalid profile: {0}")]
    InvalidProfile(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: i64,
    pub public_id: String,
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct UpdateUserProfile {
    pub username: Option<Option<String>>,
    pub display_name: Option<Option<String>>,
    pub bio: Option<Option<String>>,
    pub avatar_url: Option<Option<String>>,
}

impl UpdateUserProfile {
    pub fn is_empty(&self) -> bool {
        self.username.is_none()
            && self.display_name.is_none()
            && self.bio.is_none()
            && self.avatar_url.is_none()
    }
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
    pub login: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
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
            .insert_user(&mut tx, Some(email.to_owned()), None, None, None, None)
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

        let user_id = if let Some(row) = sqlx::query(
            "SELECT user_id FROM user_identities WHERE provider = 'github' AND provider_uid = ?",
        )
        .bind(&profile.id)
        .fetch_optional(&mut *tx)
        .await?
        {
            let user_id: i64 = row.try_get("user_id")?;
            tx.commit().await?;
            user_id
        } else {
            let user_id = if let Some(email) = profile.email.as_ref() {
                if let Some(row) = sqlx::query("SELECT id FROM users WHERE email = ?")
                    .bind(email)
                    .fetch_optional(&mut *tx)
                    .await?
                {
                    row.try_get("id")?
                } else {
                    let user = self
                        .insert_user(
                            &mut tx,
                            Some(email.clone()),
                            profile.name.clone(),
                            profile.login.clone(),
                            profile.avatar_url.clone(),
                            profile.bio.clone(),
                        )
                        .await?;
                    user.id
                }
            } else {
                let user = self
                    .insert_user(
                        &mut tx,
                        None,
                        profile.name.clone(),
                        profile.login.clone(),
                        profile.avatar_url.clone(),
                        profile.bio.clone(),
                    )
                    .await?;
                user.id
            };

            let now = Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO user_identities (user_id, provider, provider_uid, secret, created_at, updated_at) VALUES (?, ?, ?, NULL, ?, ?)",
            )
            .bind(user_id)
            .bind("github")
            .bind(&profile.id)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            user_id
        };

        let user = self.apply_github_profile(user_id, &profile).await?;
        info!(user = %user.public_id, email = ?user.email, "linked github identity");
        self.issue_session(user_id).await
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

    pub async fn update_user_profile(
        &self,
        user_id: i64,
        changes: UpdateUserProfile,
    ) -> Result<User, AuthError> {
        let username = sanitize_username(changes.username)?;
        let display_name = sanitize_display_name(changes.display_name)?;
        let bio = sanitize_bio(changes.bio)?;
        let avatar_url = sanitize_avatar_url(changes.avatar_url)?;

        if username.is_none() && display_name.is_none() && bio.is_none() && avatar_url.is_none() {
            return self.fetch_user(user_id).await;
        }

        let now = Utc::now().to_rfc3339();
        let mut query = QueryBuilder::<Sqlite>::new("UPDATE users SET ");
        let mut is_first_assignment = true;

        let mut push_assignment = |column: &str, value: Option<String>| {
            if !is_first_assignment {
                query.push(", ");
            }
            query.push(column);
            query.push(" = ");
            match value {
                Some(value) => {
                    query.push_bind(value);
                }
                None => {
                    query.push_bind::<Option<String>>(None);
                }
            }
            is_first_assignment = false;
        };

        if let Some(value) = username {
            push_assignment("username", value);
        }

        if let Some(value) = display_name {
            push_assignment("display_name", value);
        }

        if let Some(value) = bio {
            push_assignment("bio", value);
        }

        if let Some(value) = avatar_url {
            push_assignment("avatar_url", value);
        }

        if !is_first_assignment {
            query.push(", ");
        }
        query.push("updated_at = ");
        query.push_bind(&now);

        query.push(" WHERE id = ");
        query.push_bind(user_id);

        query.build().execute(&self.pool).await?;

        self.fetch_user(user_id).await
    }

    async fn insert_user(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        email: Option<String>,
        display_name: Option<String>,
        username: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<User, AuthError> {
        let now = Utc::now().to_rfc3339();
        let public_id = new_public_id();

        sqlx::query(
            "INSERT INTO users (public_id, email, display_name, username, avatar_url, bio, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&public_id)
        .bind(email.as_ref().map(|value| value.as_str()))
        .bind(display_name.as_ref().map(|value| value.as_str()))
        .bind(username.as_ref().map(|value| value.as_str()))
        .bind(avatar_url.as_ref().map(|value| value.as_str()))
        .bind(bio.as_ref().map(|value| value.as_str()))
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
            username,
            display_name,
            bio,
            avatar_url,
        })
    }

    async fn fetch_user(&self, id: i64) -> Result<User, AuthError> {
        let row = sqlx::query(
            "SELECT id, public_id, email, display_name, username, bio, avatar_url,\n                CASE WHEN email IS NULL THEN 0 ELSE 1 END AS email_present,\n                CASE WHEN display_name IS NULL THEN 0 ELSE 1 END AS display_name_present,\n                CASE WHEN username IS NULL THEN 0 ELSE 1 END AS username_present,\n                CASE WHEN bio IS NULL THEN 0 ELSE 1 END AS bio_present,\n                CASE WHEN avatar_url IS NULL THEN 0 ELSE 1 END AS avatar_url_present\n             FROM users WHERE id = ?",
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

        let username_present: i64 = row.try_get("username_present")?;
        let username = if username_present != 0 {
            Some(row.try_get::<String, _>("username")?)
        } else {
            None
        };

        let bio_present: i64 = row.try_get("bio_present")?;
        let bio = if bio_present != 0 {
            Some(row.try_get::<String, _>("bio")?)
        } else {
            None
        };

        let avatar_url_present: i64 = row.try_get("avatar_url_present")?;
        let avatar_url = if avatar_url_present != 0 {
            Some(row.try_get::<String, _>("avatar_url")?)
        } else {
            None
        };

        Ok(User {
            id,
            public_id: row.try_get("public_id")?,
            email,
            username,
            display_name,
            bio,
            avatar_url,
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

    async fn apply_github_profile(
        &self,
        user_id: i64,
        profile: &GithubProfile,
    ) -> Result<User, AuthError> {
        if let Some(email) = profile.email.as_ref() {
            sqlx::query("UPDATE users SET email = ? WHERE id = ? AND email IS NULL")
                .bind(email)
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        }

        let current = self.fetch_user(user_id).await?;
        let mut update = UpdateUserProfile::default();

        if let Some(name) = profile.name.as_ref() {
            if !name.trim().is_empty() {
                update.display_name = Some(Some(name.clone()));
            }
        }

        if let Some(login) = profile.login.as_ref() {
            if !login.trim().is_empty() {
                update.username = Some(Some(login.clone()));
                if update.display_name.is_none() && current.display_name.is_none() {
                    update.display_name = Some(Some(login.clone()));
                }
            }
        }

        if let Some(avatar) = profile.avatar_url.as_ref() {
            if !avatar.trim().is_empty() {
                update.avatar_url = Some(Some(avatar.clone()));
            }
        }

        if let Some(bio) = profile.bio.as_ref() {
            if !bio.trim().is_empty() {
                update.bio = Some(Some(bio.clone()));
            }
        }

        if update.is_empty() {
            Ok(current)
        } else {
            self.update_user_profile(user_id, update).await
        }
    }
}

fn new_public_id() -> String {
    CUID.create_id()
}

fn sanitize_display_name(
    value: Option<Option<String>>,
) -> Result<Option<Option<String>>, AuthError> {
    sanitize_optional_text(value, MAX_DISPLAY_NAME_LEN, "display_name")
}

fn sanitize_username(value: Option<Option<String>>) -> Result<Option<Option<String>>, AuthError> {
    match value {
        Some(Some(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(Some(None))
            } else if trimmed.len() > MAX_USERNAME_LEN {
                Err(AuthError::InvalidProfile(format!(
                    "username must be at most {MAX_USERNAME_LEN} characters"
                )))
            } else if !trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            {
                Err(AuthError::InvalidProfile(
                    "username may only contain letters, numbers, '.', '-' or '_'".to_string(),
                ))
            } else {
                Ok(Some(Some(trimmed.to_string())))
            }
        }
        Some(None) => Ok(Some(None)),
        None => Ok(None),
    }
}

fn sanitize_bio(value: Option<Option<String>>) -> Result<Option<Option<String>>, AuthError> {
    sanitize_optional_text(value, MAX_BIO_LEN, "bio")
}

fn sanitize_optional_text(
    value: Option<Option<String>>,
    max_len: usize,
    field: &str,
) -> Result<Option<Option<String>>, AuthError> {
    match value {
        Some(Some(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(Some(None))
            } else if trimmed.len() > max_len {
                Err(AuthError::InvalidProfile(format!(
                    "{field} must be at most {max_len} characters"
                )))
            } else {
                Ok(Some(Some(trimmed.to_string())))
            }
        }
        Some(None) => Ok(Some(None)),
        None => Ok(None),
    }
}

fn sanitize_avatar_url(value: Option<Option<String>>) -> Result<Option<Option<String>>, AuthError> {
    match value {
        Some(Some(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(None));
            }
            if trimmed.len() > MAX_AVATAR_URL_LEN {
                return Err(AuthError::InvalidProfile(format!(
                    "avatar_url must be at most {MAX_AVATAR_URL_LEN} characters"
                )));
            }

            let parsed = Url::parse(trimmed).map_err(|_| {
                AuthError::InvalidProfile("avatar_url must be a valid URL".to_string())
            })?;

            match parsed.scheme() {
                "http" | "https" => Ok(Some(Some(trimmed.to_string()))),
                _ => Err(AuthError::InvalidProfile(
                    "avatar_url must use http or https".to_string(),
                )),
            }
        }
        Some(None) => Ok(Some(None)),
        None => Ok(None),
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
            login: Some(user.login),
            avatar_url: user.avatar_url,
            bio: user.bio,
        })
    }
}

#[derive(Deserialize)]
struct GithubUserResponse {
    id: i64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
}
