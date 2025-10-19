//! Data access layer for the user management system.
//!
//! This module contains repository implementations that handle all
//! database operations. Repositories provide a clean interface
//! between the business logic and the database.

pub mod user_repository;
pub mod session_repository;
pub mod notification_repository;
pub mod settings_repository;

// Re-export all repositories
pub use user_repository::UserRepository;
pub use session_repository::SessionRepository;
pub use notification_repository::NotificationRepository;
pub use settings_repository::SettingsRepository;