//! Domain entities for the user management system.
//!
//! This module contains all the core domain entities that represent the
//! user system's data models. These are pure domain objects without
//! API-specific concerns.

pub mod user;
pub mod auth;
pub mod notification;
pub mod settings;

// Re-export all entity types
pub use user::{User, CreateUserRequest, UpdateUserRequest, UserStatus};
pub use auth::{AuthSession, CreateSessionRequest, LoginRequest, RegisterRequest, AuthProvider};
pub use notification::{Notification, NotificationPreferences, NotificationType, NotificationPriority};
pub use settings::{UserSettings, UserPreferences};