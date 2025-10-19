//! Error types for the database layer

use thiserror::Error;

/// General database error
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database query error: {0}")]
    QueryError(String),

    #[error("Database migration error: {0}")]
    MigrationError(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Duplicate entity: {0}")]
    Duplicate(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// User-specific database errors
#[derive(Debug, Error)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Username already exists")]
    UsernameAlreadyExists,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Account is locked")]
    AccountLocked,

    #[error("Account is suspended")]
    AccountSuspended,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Chat-specific database errors
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("Chat not found")]
    ChatNotFound,

    #[error("Message not found")]
    MessageNotFound,

    #[error("Member not found")]
    MemberNotFound,

    #[error("Invite not found")]
    InviteNotFound,

    #[error("Attachment not found")]
    AttachmentNotFound,

    #[error("Member already exists")]
    MemberAlreadyExists,

    #[error("Invite already used")]
    InviteAlreadyUsed,

    #[error("Invite expired")]
    InviteExpired,

    #[error("Access denied")]
    AccessDenied,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Chat is archived")]
    ChatArchived,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Notification-specific database errors
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Notification not found")]
    NotificationNotFound,

    #[error("Invalid notification type")]
    InvalidNotificationType,

    #[error("Invalid notification priority")]
    InvalidPriority,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Auth-specific database errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Database error: {0}")]
    DatabaseError(String),
}