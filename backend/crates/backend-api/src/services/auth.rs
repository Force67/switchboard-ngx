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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_utils::{create_test_db, create_test_user};
    use crate::services::test_utils::fixtures::*;
    use switchboard_auth::{Authenticator, AuthSession};

    // Note: GitHub OAuth tests require complex mocking which is beyond the scope of current test setup.
    // These would require either a more sophisticated mocking framework or integration tests.

    #[tokio::test]
    async fn test_create_dev_token_success() {
        let (pool, _temp_dir) = create_test_db().await;

        let result = create_dev_token(&pool).await;

        assert!(result.is_ok());
        let (session, user) = result.unwrap();
        assert_eq!(session.user_id, 1);
        assert_eq!(user.id, 1);
        assert_eq!(user.public_id, "dev-user-123");
        assert_eq!(user.email.unwrap(), "dev@example.com");
    }

    #[tokio::test]
    async fn test_create_session_response() {
        let session = AuthSession {
            token: "test-token".to_string(),
            user_id: 1,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        };
        let user = test_user();

        let response = create_session_response(session.clone(), user.clone());

        assert_eq!(response.token, session.token);
        assert_eq!(response.user.id, user.public_id);
        // Note: SessionResponse doesn't include public_id based on the current implementation
        assert_eq!(response.user.email, user.email);
        assert_eq!(response.user.display_name, user.display_name);
    }
}