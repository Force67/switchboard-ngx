use std::collections::HashSet;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use chrono::{DateTime, Duration, Utc};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::str::FromStr;
use switchboard_auth::{AuthError, Authenticator, GithubProfile};
use switchboard_config::{AuthConfig, GithubAuthConfig};
use tempfile::TempDir;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

fn default_auth_config() -> AuthConfig {
    AuthConfig {
        session_ttl_seconds: 3_600,
        github: GithubAuthConfig::default(),
    }
}

fn github_auth_config() -> AuthConfig {
    AuthConfig {
        session_ttl_seconds: 3_600,
        github: GithubAuthConfig {
            client_id: Some("test-client-id".into()),
            client_secret: Some("test-client-secret".into()),
        },
    }
}

struct TestContext {
    pool: SqlitePool,
    authenticator: Authenticator,
    _temp_dir: TempDir,
    config: AuthConfig,
}

impl TestContext {
    async fn new(config: AuthConfig) -> TestResult<Self> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("auth.sqlite");
        let db_url = format!("sqlite://{}", db_path.display());

        let mut options = SqliteConnectOptions::from_str(&db_url)?;
        options = options.create_if_missing(true);
        options = options.foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        MIGRATOR.run(&pool).await?;

        let authenticator = Authenticator::new(pool.clone(), config.clone());

        Ok(Self {
            pool,
            authenticator,
            _temp_dir: temp_dir,
            config,
        })
    }

    async fn new_default() -> TestResult<Self> {
        Self::new(default_auth_config()).await
    }

    fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }
}

#[tokio::test]
async fn register_with_password_persists_user_and_password_identity() -> TestResult {
    let ctx = TestContext::new_default().await?;

    let user = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user.id)
        .fetch_one(ctx.pool())
        .await?;
    assert_eq!(user_count, 1, "user row should exist");

    let identity =
        sqlx::query("SELECT provider, provider_uid, secret FROM user_identities WHERE user_id = ?")
            .bind(user.id)
            .fetch_one(ctx.pool())
            .await?;

    let provider: String = identity.get("provider");
    let provider_uid: String = identity.get("provider_uid");
    let secret: String = identity.get("secret");

    assert_eq!(provider, "password");
    assert_eq!(provider_uid, "alice@example.com");
    assert!(
        secret.starts_with("$argon2"),
        "secret must be an argon2 hash"
    );

    Ok(())
}

#[tokio::test]
async fn register_with_password_rejects_duplicate_email() -> TestResult {
    let ctx = TestContext::new_default().await?;
    ctx.authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let err = ctx
        .authenticator()
        .register_with_password("alice@example.com", "another")
        .await
        .expect_err("expected duplicate email to fail");

    assert!(matches!(err, AuthError::UserExists));

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(ctx.pool())
        .await?;
    assert_eq!(user_count, 1, "no additional users should be created");

    Ok(())
}

#[tokio::test]
async fn register_with_password_hashes_secret_using_argon2() -> TestResult {
    let ctx = TestContext::new_default().await?;

    let first = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;
    let first_secret: String =
        sqlx::query_scalar("SELECT secret FROM user_identities WHERE user_id = ?")
            .bind(first.id)
            .fetch_one(ctx.pool())
            .await?;

    let second = ctx
        .authenticator()
        .register_with_password("bob@example.com", "s3cret")
        .await?;
    let second_secret: String =
        sqlx::query_scalar("SELECT secret FROM user_identities WHERE user_id = ?")
            .bind(second.id)
            .fetch_one(ctx.pool())
            .await?;

    assert_ne!(
        first_secret, second_secret,
        "argon2 salts should differ per registration"
    );

    argon2::password_hash::PasswordHash::new(&first_secret)?;
    argon2::password_hash::PasswordHash::new(&second_secret)?;

    Ok(())
}

#[tokio::test]
async fn login_with_password_returns_session_for_valid_credentials() -> TestResult {
    let ctx = TestContext::new_default().await?;
    ctx.authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let session = ctx
        .authenticator()
        .login_with_password("alice@example.com", "s3cret")
        .await?;

    let ttl = Duration::seconds(ctx.config.session_ttl_seconds as i64);
    let remaining = session.expires_at - Utc::now();
    assert!(
        (remaining - ttl).num_seconds().abs() <= 2,
        "session ttl should respect configuration"
    );

    let stored_expires: String =
        sqlx::query_scalar("SELECT expires_at FROM sessions WHERE token = ?")
            .bind(&session.token)
            .fetch_one(ctx.pool())
            .await?;
    let parsed = DateTime::parse_from_rfc3339(&stored_expires)?.with_timezone(&Utc);
    assert_eq!(parsed, session.expires_at);

    Ok(())
}

#[tokio::test]
async fn login_with_password_rejects_incorrect_secret() -> TestResult {
    let ctx = TestContext::new_default().await?;
    ctx.authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let err = ctx
        .authenticator()
        .login_with_password("alice@example.com", "bad-secret")
        .await
        .expect_err("expected invalid password");
    assert!(matches!(err, AuthError::InvalidCredentials));

    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(ctx.pool())
        .await?;
    assert_eq!(session_count, 0, "no sessions should be issued on failure");

    Ok(())
}

#[tokio::test]
async fn login_with_password_rejects_unknown_email() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let err = ctx
        .authenticator()
        .login_with_password("unknown@example.com", "secret")
        .await
        .expect_err("expected unknown email to fail");
    assert!(matches!(err, AuthError::InvalidCredentials));
    Ok(())
}

#[tokio::test]
async fn login_with_github_code_returns_existing_identity_session() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let existing = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO user_identities (user_id, provider, provider_uid, secret, created_at, updated_at)
         VALUES (?, 'github', ?, NULL, ?, ?)",
    )
    .bind(existing.id)
    .bind("github-123")
    .bind(&now)
    .bind(&now)
    .execute(ctx.pool())
    .await?;

    let session = ctx
        .authenticator()
        .login_with_github_profile(GithubProfile {
            id: "github-123".into(),
            email: Some("alice@example.com".into()),
            name: Some("Alice Example".into()),
        })
        .await?;

    assert_eq!(session.user_id, existing.id);

    let session_row: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE user_id = ? AND token = ?")
            .bind(existing.id)
            .bind(&session.token)
            .fetch_one(ctx.pool())
            .await?;
    assert_eq!(session_row, 1, "session should be persisted");

    Ok(())
}

#[tokio::test]
async fn login_with_github_code_links_existing_user_by_email() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let existing = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let session = ctx
        .authenticator()
        .login_with_github_profile(GithubProfile {
            id: "github-456".into(),
            email: Some("alice@example.com".into()),
            name: Some("Alice Example".into()),
        })
        .await?;

    assert_eq!(session.user_id, existing.id);

    let identity_row = sqlx::query(
        "SELECT provider_uid FROM user_identities WHERE user_id = ? AND provider = 'github'",
    )
    .bind(existing.id)
    .fetch_one(ctx.pool())
    .await?;
    let provider_uid: String = identity_row.get("provider_uid");
    assert_eq!(provider_uid, "github-456");

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(ctx.pool())
        .await?;
    assert_eq!(user_count, 1, "user should not be duplicated");

    Ok(())
}

#[tokio::test]
async fn login_with_github_code_creates_user_when_new_profile() -> TestResult {
    let ctx = TestContext::new_default().await?;

    let session = ctx
        .authenticator()
        .login_with_github_profile(GithubProfile {
            id: "github-789".into(),
            email: Some("new@example.com".into()),
            name: Some("New User".into()),
        })
        .await?;

    let user_row = sqlx::query("SELECT email, display_name FROM users WHERE id = ?")
        .bind(session.user_id)
        .fetch_one(ctx.pool())
        .await?;
    let email: String = user_row.get("email");
    let display_name: Option<String> = user_row.try_get("display_name")?;

    assert_eq!(email, "new@example.com");
    assert_eq!(display_name.as_deref(), Some("New User"));

    let identity_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_identities WHERE user_id = ? AND provider = 'github'",
    )
    .bind(session.user_id)
    .fetch_one(ctx.pool())
    .await?;
    assert_eq!(identity_exists, 1);

    Ok(())
}

#[tokio::test]
async fn login_with_github_code_requires_github_configuration() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let err = ctx
        .authenticator()
        .login_with_github_code("dummy", "https://example.com")
        .await
        .expect_err("github oauth should be disabled");
    assert!(matches!(err, AuthError::GithubOauthDisabled));
    Ok(())
}

#[tokio::test]
async fn login_with_github_profile_handles_missing_email() -> TestResult {
    let ctx = TestContext::new_default().await?;

    let session = ctx
        .authenticator()
        .login_with_github_profile(GithubProfile {
            id: "github-999".into(),
            email: None,
            name: Some("No Email".into()),
        })
        .await?;

    let row = sqlx::query("SELECT email, display_name FROM users WHERE id = ?")
        .bind(session.user_id)
        .fetch_one(ctx.pool())
        .await?;
    let email: Option<String> = row.try_get("email")?;
    let display_name: Option<String> = row.try_get("display_name")?;

    assert!(email.is_none(), "email should remain NULL");
    assert_eq!(display_name.as_deref(), Some("No Email"));

    Ok(())
}

#[tokio::test]
async fn authenticate_token_returns_user_and_session_for_active_token() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let user = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;
    let session = ctx
        .authenticator()
        .login_with_password("alice@example.com", "s3cret")
        .await?;

    let (resolved_user, resolved_session) = ctx
        .authenticator()
        .authenticate_token(&session.token)
        .await?;

    assert_eq!(resolved_user.id, user.id);
    assert_eq!(resolved_session.token, session.token);
    Ok(())
}

#[tokio::test]
async fn authenticate_token_deletes_expired_sessions() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let user = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let token = "expired-token";
    let created_at = (Utc::now() - Duration::hours(2)).to_rfc3339();
    let expires_at = (Utc::now() - Duration::hours(1)).to_rfc3339();

    sqlx::query(
        "INSERT INTO sessions (user_id, token, created_at, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(user.id)
    .bind(token)
    .bind(&created_at)
    .bind(&expires_at)
    .execute(ctx.pool())
    .await?;

    let err = ctx
        .authenticator()
        .authenticate_token(token)
        .await
        .expect_err("expired token should be rejected");
    assert!(matches!(err, AuthError::SessionExpired));

    let remaining: Option<i64> = sqlx::query_scalar("SELECT 1 FROM sessions WHERE token = ?")
        .bind(token)
        .fetch_optional(ctx.pool())
        .await?;
    assert!(
        remaining.is_none(),
        "expired session should be removed from the database"
    );

    Ok(())
}

#[tokio::test]
async fn authenticate_token_rejects_unknown_token() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let err = ctx
        .authenticator()
        .authenticate_token("missing-token")
        .await
        .expect_err("unknown token should not authenticate");
    assert!(matches!(err, AuthError::SessionNotFound));
    Ok(())
}

#[tokio::test]
async fn user_profile_fetches_optional_fields_correctly() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let user = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let fetched = ctx.authenticator().user_profile(user.id).await?;
    assert_eq!(fetched.email.as_deref(), Some("alice@example.com"));
    assert!(
        fetched.display_name.is_none(),
        "display name should be None"
    );

    sqlx::query("UPDATE users SET display_name = ? WHERE id = ?")
        .bind("Alice Example")
        .bind(user.id)
        .execute(ctx.pool())
        .await?;

    let updated = ctx.authenticator().user_profile(user.id).await?;
    assert_eq!(updated.display_name.as_deref(), Some("Alice Example"));
    Ok(())
}

#[tokio::test]
async fn issue_session_applies_configured_ttl_and_persists_record() -> TestResult {
    let ctx = TestContext::new_default().await?;
    ctx.authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;
    let session = ctx
        .authenticator()
        .login_with_password("alice@example.com", "s3cret")
        .await?;

    let ttl = Duration::seconds(ctx.config.session_ttl_seconds as i64);
    let diff = session.expires_at - Utc::now();
    assert!(
        (diff - ttl).num_seconds().abs() <= 2,
        "issued session should honour TTL"
    );

    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM sessions WHERE token = ?")
        .bind(&session.token)
        .fetch_optional(ctx.pool())
        .await?;
    assert!(exists.is_some(), "session should be stored");

    Ok(())
}

#[tokio::test]
async fn generate_session_token_produces_unique_urlsafe_tokens() -> TestResult {
    let ctx = TestContext::new_default().await?;
    ctx.authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;

    let mut tokens = HashSet::new();
    for _ in 0..5 {
        let session = ctx
            .authenticator()
            .login_with_password("alice@example.com", "s3cret")
            .await?;
        assert!(
            URL_SAFE_NO_PAD.decode(session.token.as_bytes()).is_ok(),
            "token should be URL safe base64"
        );
        assert!(
            tokens.insert(session.token.clone()),
            "tokens should be unique per session"
        );
    }
    Ok(())
}

#[tokio::test]
async fn hash_password_uses_random_salt_per_call() -> TestResult {
    let ctx = TestContext::new_default().await?;
    let first = ctx
        .authenticator()
        .register_with_password("alice@example.com", "s3cret")
        .await?;
    let second = ctx
        .authenticator()
        .register_with_password("bob@example.com", "s3cret")
        .await?;

    let first_secret: String =
        sqlx::query_scalar("SELECT secret FROM user_identities WHERE user_id = ?")
            .bind(first.id)
            .fetch_one(ctx.pool())
            .await?;
    let second_secret: String =
        sqlx::query_scalar("SELECT secret FROM user_identities WHERE user_id = ?")
            .bind(second.id)
            .fetch_one(ctx.pool())
            .await?;

    assert_ne!(
        first_secret, second_secret,
        "argon2 salts must randomise identical passwords"
    );
    Ok(())
}

#[tokio::test]
async fn github_authorization_url_includes_required_scopes_and_state() -> TestResult {
    let ctx = TestContext::new(github_auth_config()).await?;
    assert!(
        ctx.authenticator().github_enabled(),
        "github auth should be enabled"
    );

    let state = "state-token-123";
    let url = ctx
        .authenticator()
        .github_authorization_url(state, "https://example.com/callback")?;

    let parsed = reqwest::Url::parse(&url)?;
    let query = parsed.query_pairs().collect::<Vec<_>>();

    assert!(query.iter().any(|(k, v)| k == "state" && v == state));
    assert!(query
        .iter()
        .any(|(k, v)| k == "scope" && v.contains("read:user")));
    assert!(query
        .iter()
        .any(|(k, v)| k == "scope" && v.contains("user:email")));

    Ok(())
}

#[tokio::test]
async fn github_exchange_code_propagates_http_failures() -> TestResult {
    let ctx = TestContext::new(github_auth_config()).await?;
    let err = ctx
        .authenticator()
        .login_with_github_code("invalid-code", "not-a-valid-url")
        .await
        .expect_err("invalid redirect URI should fail before network request");
    assert!(matches!(err, AuthError::GithubOauth(_)));
    Ok(())
}

#[tokio::test]
async fn github_oauth_from_config_requires_client_credentials() -> TestResult {
    let ctx_disabled = TestContext::new_default().await?;
    assert!(
        !ctx_disabled.authenticator().github_enabled(),
        "GitHub OAuth should be disabled without credentials"
    );

    let ctx_enabled = TestContext::new(github_auth_config()).await?;
    assert!(
        ctx_enabled.authenticator().github_enabled(),
        "GitHub OAuth should be enabled with credentials"
    );
    Ok(())
}
