//! # Switchboard Users Crate
//!
//! This crate provides user management, authentication, and notification functionality
//! for the Switchboard application. It contains domain entities, services, repositories,
//! and types for managing users, sessions, authentication, and notifications.
//!
//! ## Architecture
//!
//! - **Entities**: Domain models (User, AuthSession, Notification, etc.)
//! - **Services**: Business logic layer
//! - **Repositories**: Data access layer
//! - **Types**: Shared types and interfaces
//! - **Utils**: Internal utilities
//!
//! ## Usage
//!
//! ```rust
//! use switchboard_users::{UserService, CreateUserRequest};
//!
//! let service = UserService::new(pool);
//! let user = service.create_user(request).await?;
//! ```

pub mod services;
pub mod types {
    pub mod requests;
    pub mod responses;
    pub mod events;
}
pub mod utils;

// Re-export database types and repositories
pub use switchboard_database::{
    UserRepository, SessionRepository, SettingsRepository, NotificationRepository,
    UserResult, UserError, AuthResult, NotificationResult,
    User, AuthSession, Notification, UserSettings, UserPreferences,
    CreateUserRequest, UpdateUserRequest, CreateSessionRequest,
    UserRole, UserStatus,
};

// Re-export sqlx for pool access
pub use sqlx::sqlite::SqlitePool;

// Re-export main types for convenience
// Note: LoginRequest, RegisterRequest, NotificationPreferences need to be moved elsewhere
pub use services::{
    UserService, /* AuthService, NotificationService, SessionService, */
};
pub use types::events::UserEvent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}