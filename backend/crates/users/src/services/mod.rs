//! Business logic services for the user management system.
//!
//! This module contains all the service layer components that implement
//! the core business logic for user operations. Services coordinate
//! between repositories and handle business rules.

pub mod auth_service;
mod mock_repositories;
pub mod notification_service;
pub mod session_service;
pub mod user_service;

// Re-export all services
pub use auth_service::AuthService;
pub use notification_service::NotificationService;
pub use session_service::SessionService;
pub use user_service::{UserRepo, UserService};
