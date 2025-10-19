//! User entity definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity representing a user in the system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub public_id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
    pub created_at: String,
    pub updated_at: String,
    pub last_login_at: Option<String>,
    pub email_verified: bool,
    pub is_active: bool,
}

/// Request for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// Request for updating an existing user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub role: Option<UserRole>,
}

/// User status enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Suspended => "suspended",
            UserStatus::Deleted => "deleted",
        }
    }
}

impl From<&str> for UserStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            "suspended" => UserStatus::Suspended,
            "deleted" => UserStatus::Deleted,
            _ => UserStatus::Active,
        }
    }
}

impl ToString for UserStatus {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

/// User role enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Admin,
    Moderator,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Admin => "admin",
            UserRole::Moderator => "moderator",
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        match s {
            "admin" => UserRole::Admin,
            "moderator" => UserRole::Moderator,
            _ => UserRole::User,
        }
    }
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}