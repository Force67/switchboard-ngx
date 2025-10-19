//! Error types for the user management system.

use thiserror::Error;

/// User-related errors
#[derive(Debug, Error, Clone)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid user data: {0}")]
    InvalidUserData(String),

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password too weak")]
    PasswordTooWeak,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Password hashing failed")]
    PasswordHashingFailed,

    #[error("Invalid password hash")]
    InvalidPasswordHash,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Account is locked")]
    AccountLocked,

    #[error("Account is suspended")]
    AccountSuspended,

    #[error("Invalid user role")]
    InvalidUserRole,

    #[error("Settings not found")]
    SettingsNotFound,

    #[error("Invalid settings data")]
    InvalidSettingsData,

    #[error("Token creation failed: {0}")]
    TokenCreationFailed(String),

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

/// Authentication-related errors
#[derive(Debug, Error, Clone)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid session token")]
    InvalidSessionToken,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Token creation failed")]
    TokenCreationFailed(String),

    #[error("Token refresh failed")]
    TokenRefreshFailed(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("OAuth provider error: {0}")]
    OAuthError(String),

    #[error("Two-factor authentication required")]
    TwoFactorRequired,

    #[error("Invalid two-factor code")]
    InvalidTwoFactorCode,

    #[error("Account not verified")]
    AccountNotVerified,

    #[error("Too many login attempts")]
    TooManyAttempts,

    #[error("CSRF token invalid")]
    InvalidCsrfToken,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Notification-related errors
#[derive(Debug, Error, Clone)]
pub enum NotificationError {
    #[error("Notification not found")]
    NotificationNotFound,

    #[error("Invalid notification data")]
    InvalidNotificationData,

    #[error("Notification delivery failed")]
    DeliveryFailed,

    #[error("Invalid notification type")]
    InvalidNotificationType,

    #[error("Invalid notification priority")]
    InvalidNotificationPriority,

    #[error("Notification preferences not found")]
    PreferencesNotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Notification service unavailable")]
    ServiceUnavailable,
}

/// Result types for user operations
pub type UserResult<T> = Result<T, UserError>;
pub type AuthResult<T> = Result<T, AuthError>;
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Convert database errors to our error types
impl From<sqlx::Error> for UserError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UserError::UserNotFound,
            sqlx::Error::Database(db_err) => {
                if db_err.message().contains("UNIQUE constraint failed") {
                    if db_err.message().contains("email") {
                        UserError::EmailAlreadyExists
                    } else {
                        UserError::UserAlreadyExists
                    }
                } else {
                    UserError::DatabaseError(db_err.message().to_string())
                }
            }
            _ => UserError::DatabaseError(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AuthError::SessionNotFound,
            _ => AuthError::DatabaseError(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for NotificationError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => NotificationError::NotificationNotFound,
            _ => NotificationError::DatabaseError(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let user_err = UserError::UserNotFound;
        assert_eq!(user_err.to_string(), "User not found");

        let auth_err = AuthError::InvalidCredentials;
        assert_eq!(auth_err.to_string(), "Invalid credentials");

        let notification_err = NotificationError::NotificationNotFound;
        assert_eq!(notification_err.to_string(), "Notification not found");
    }
}