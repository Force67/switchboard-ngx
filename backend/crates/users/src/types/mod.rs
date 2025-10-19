//! Shared types and interfaces for the user management system.
//!
//! This module contains common types, error definitions, and interfaces
//! that are used across multiple modules in the crate.

pub mod events;

// Re-export common types
pub use events::*;

// Common type aliases
pub type UserId = i64;
pub type SessionId = String;
pub type NotificationId = String;

// Common enums
#[derive(Debug, Clone, PartialEq)]
pub enum AuthStatus {
    Success,
    InvalidCredentials,
    AccountLocked,
    AccountSuspended,
    TwoFactorRequired,
    ProviderError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserEventType {
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserLoggedIn,
    UserLoggedOut,
    SettingsUpdated,
    PasswordChanged,
    TwoFactorEnabled,
    TwoFactorDisabled,
}