use sqlx::SqlitePool;
use switchboard_auth::{AuthSession, Authenticator, User};
use crate::routes::auth::SessionResponse;
use super::error::ServiceError;

pub async fn github_login_url(
    authenticator: &Authenticator,
    oauth_state: &crate::state::OAuthStateStore,
    redirect_uri: String,
) -> Result<String, ServiceError> {
    if !authenticator.github_enabled() {
        return Err(ServiceError::config("GitHub OAuth is not configured"));
    }

    let oauth_state = oauth_state.issue().await;
    let authorize_url = authenticator.github_authorization_url(&oauth_state, &redirect_uri)?;

    Ok(authorize_url)
}

pub async fn github_callback(
    authenticator: &Authenticator,
    oauth_state: &crate::state::OAuthStateStore,
    code: String,
    state: String,
    redirect_uri: String,
) -> Result<(AuthSession, User), ServiceError> {
    if !oauth_state.consume(&state).await {
        return Err(ServiceError::bad_request("invalid or expired OAuth state"));
    }

    let session = authenticator.login_with_github_code(&code, &redirect_uri).await?;
    let user = authenticator.user_profile(session.user_id).await?;

    Ok((session, user))
}

pub async fn create_dev_token(pool: &SqlitePool) -> Result<(AuthSession, User), ServiceError> {
    // Create a development user in the database first
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO users (id, public_id, email, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(1i64)
    .bind("dev-user-123")
    .bind("dev@example.com")
    .bind("Dev User")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    // Create a development session by inserting directly into the database
    let session_token = cuid2::create_id();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

    sqlx::query(
        r#"
        INSERT INTO sessions (token, user_id, expires_at, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&session_token)
    .bind(1i64)
    .bind(expires_at.to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    let session = AuthSession {
        token: session_token,
        user_id: 1,
        expires_at,
    };

    let user = User {
        id: 1,
        public_id: "dev-user-123".to_string(),
        email: Some("dev@example.com".to_string()),
        display_name: Some("Dev User".to_string()),
    };

    Ok((session, user))
}

pub fn create_session_response(session: AuthSession, user: User) -> SessionResponse {
    SessionResponse::new(session, user)
}