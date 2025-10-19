//! Business logic services for the user management system.
//!
//! This module contains all the service layer components that implement
//! the core business logic for user operations. Services coordinate
//! between repositories and handle business rules.

pub mod user_service;
// pub mod auth_service;  // Temporarily commented out due to compilation errors
// pub mod notification_service;  // Temporarily commented out due to compilation errors
// pub mod session_service;  // Temporarily commented out due to compilation errors
mod mock_repositories;

// Re-export all services
pub use user_service::{UserService, UserRepo};
// pub use auth_service::AuthService;  // Temporarily commented out
// pub use notification_service::NotificationService;  // Temporarily commented out
// pub use session_service::SessionService;  // Temporarily commented out