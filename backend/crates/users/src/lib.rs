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

pub mod entities;
pub mod services;
pub mod types;
pub mod utils;

// Re-export database types and repositories
pub use switchboard_database::{
    SqlitePool, UserRepository, SessionRepository, SettingsRepository, NotificationRepository,
    UserResult, UserError, AuthResult, NotificationResult,
    User, AuthSession, Notification, UserSettings, UserPreferences,
    CreateUserRequest, UpdateUserRequest, CreateSessionRequest,
};

// Re-export main types for convenience
pub use entities::{
    User as UserEntity, // Re-export with different name to avoid conflicts
    LoginRequest, RegisterRequest, NotificationPreferences,
};
pub use services::{
    UserService, /* AuthService, NotificationService, SessionService, */
};
pub use types::{
    UserEvent,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        // Basic test to ensure the crate compiles
        assert!(true);
    }
}