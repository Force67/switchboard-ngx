use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// User email address
    pub email: Option<String>,
    /// Display name for the user
    pub display_name: Option<String>,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// User status
    pub status: UserStatus,
    /// User role
    pub role: UserRole,
    /// When the user was created
    pub created_at: String,
    /// When the user was last updated
    pub updated_at: String,
    /// Last login timestamp
    pub last_login_at: Option<String>,
    /// Email verification status
    pub email_verified: bool,
    /// Account is active
    pub is_active: bool,
}

/// User status enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

/// User role enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Admin,
    Moderator,
    System,
}

impl From<&str> for UserStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "inactive" => UserStatus::Inactive,
            "suspended" => UserStatus::Suspended,
            "deleted" => UserStatus::Deleted,
            _ => UserStatus::Active,
        }
    }
}

impl From<UserStatus> for String {
    fn from(status: UserStatus) -> Self {
        match status {
            UserStatus::Active => "active".to_string(),
            UserStatus::Inactive => "inactive".to_string(),
            UserStatus::Suspended => "suspended".to_string(),
            UserStatus::Deleted => "deleted".to_string(),
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "moderator" => UserRole::Moderator,
            "system" => UserRole::System,
            _ => UserRole::User,
        }
    }
}

impl From<UserRole> for String {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::User => "user".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Moderator => "moderator".to_string(),
            UserRole::System => "system".to_string(),
        }
    }
}

/// Request to create a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// User email address
    pub email: String,
    /// Display name
    pub display_name: String,
    /// Avatar URL (optional)
    pub avatar_url: Option<String>,
    /// Initial role (defaults to User)
    pub role: Option<UserRole>,
}

/// Request to update a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    /// New email address (optional)
    pub email: Option<String>,
    /// New display name (optional)
    pub display_name: Option<String>,
    /// New avatar URL (optional)
    pub avatar_url: Option<String>,
    /// New status (optional)
    pub status: Option<UserStatus>,
    /// New role (optional)
    pub role: Option<UserRole>,
}

impl User {
    /// Create a new user instance
    pub fn new(
        email: String,
        display_name: String,
        avatar_url: Option<String>,
        role: UserRole,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            email: Some(email),
            display_name: Some(display_name),
            avatar_url,
            status: UserStatus::Active,
            role,
            created_at: now.clone(),
            updated_at: now,
            last_login_at: None,
            email_verified: false,
            is_active: true,
        }
    }

    /// Get user's display name or fallback
    pub fn display_name_or_fallback(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| self.email.clone().unwrap_or_else(|| "Unknown User".to_string()))
    }

    /// Check if user is an admin
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin)
    }

    /// Check if user is a moderator
    pub fn is_moderator(&self) -> bool {
        matches!(self.role, UserRole::Moderator | UserRole::Admin)
    }

    /// Check if user is active
    pub fn is_active_user(&self) -> bool {
        self.is_active && matches!(self.status, UserStatus::Active)
    }

    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login_at = Some(Utc::now().to_rfc3339());
        self.touch();
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Validate user data
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref email) = self.email {
            if email.trim().is_empty() {
                return Err("Email cannot be empty".to_string());
            }

            if !email.contains('@') || !email.contains('.') {
                return Err("Invalid email format".to_string());
            }

            if email.len() > 255 {
                return Err("Email too long (max 255 characters)".to_string());
            }
        }

        if let Some(ref display_name) = self.display_name {
            if display_name.trim().is_empty() {
                return Err("Display name cannot be empty".to_string());
            }

            if display_name.len() > 100 {
                return Err("Display name too long (max 100 characters)".to_string());
            }
        }

        if let Some(ref avatar_url) = self.avatar_url {
            if avatar_url.trim().is_empty() {
                return Err("Avatar URL cannot be empty".to_string());
            }

            if !avatar_url.starts_with("http://") && !avatar_url.starts_with("https://") {
                return Err("Avatar URL must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        Ok(())
    }
}

impl CreateUserRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), String> {
        if self.email.trim().is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        if !self.email.contains('@') || !self.email.contains('.') {
            return Err("Invalid email format".to_string());
        }

        if self.email.len() > 255 {
            return Err("Email too long (max 255 characters)".to_string());
        }

        if self.display_name.trim().is_empty() {
            return Err("Display name cannot be empty".to_string());
        }

        if self.display_name.len() > 100 {
            return Err("Display name too long (max 100 characters)".to_string());
        }

        if let Some(ref avatar_url) = self.avatar_url {
            if avatar_url.trim().is_empty() {
                return Err("Avatar URL cannot be empty".to_string());
            }

            if !avatar_url.starts_with("http://") && !avatar_url.starts_with("https://") {
                return Err("Avatar URL must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        Ok(())
    }
}

impl UpdateUserRequest {
    /// Validate the update request
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref email) = self.email {
            if email.trim().is_empty() {
                return Err("Email cannot be empty".to_string());
            }

            if !email.contains('@') || !email.contains('.') {
                return Err("Invalid email format".to_string());
            }

            if email.len() > 255 {
                return Err("Email too long (max 255 characters)".to_string());
            }
        }

        if let Some(ref display_name) = self.display_name {
            if display_name.trim().is_empty() {
                return Err("Display name cannot be empty".to_string());
            }

            if display_name.len() > 100 {
                return Err("Display name too long (max 100 characters)".to_string());
            }
        }

        if let Some(ref avatar_url) = self.avatar_url {
            if avatar_url.trim().is_empty() {
                return Err("Avatar URL cannot be empty".to_string());
            }

            if !avatar_url.starts_with("http://") && !avatar_url.starts_with("https://") {
                return Err("Avatar URL must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            Some("https://example.com/avatar.jpg".to_string()),
            UserRole::User,
        );

        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert_eq!(user.role, UserRole::User);
        assert_eq!(user.status, UserStatus::Active);
        assert!(user.is_active_user());
        assert!(!user.is_admin());
        assert!(!user.is_moderator());
    }

    #[test]
    fn test_user_role_conversion() {
        assert_eq!(UserRole::from("admin"), UserRole::Admin);
        assert_eq!(UserRole::from("moderator"), UserRole::Moderator);
        assert_eq!(UserRole::from("system"), UserRole::System);
        assert_eq!(UserRole::from("user"), UserRole::User);
        assert_eq!(UserRole::from("unknown"), UserRole::User);

        assert_eq!(String::from(UserRole::User), "user");
        assert_eq!(String::from(UserRole::Admin), "admin");
        assert_eq!(String::from(UserRole::Moderator), "moderator");
        assert_eq!(String::from(UserRole::System), "system");
    }

    #[test]
    fn test_user_status_conversion() {
        assert_eq!(UserStatus::from("active"), UserStatus::Active);
        assert_eq!(UserStatus::from("inactive"), UserStatus::Inactive);
        assert_eq!(UserStatus::from("suspended"), UserStatus::Suspended);
        assert_eq!(UserStatus::from("deleted"), UserStatus::Deleted);
        assert_eq!(UserStatus::from("unknown"), UserStatus::Active);

        assert_eq!(String::from(UserStatus::Active), "active");
        assert_eq!(String::from(UserStatus::Inactive), "inactive");
        assert_eq!(String::from(UserStatus::Suspended), "suspended");
        assert_eq!(String::from(UserStatus::Deleted), "deleted");
    }

    #[test]
    fn test_user_validation() {
        let mut user = User::new(
            "valid@example.com".to_string(),
            "Valid User".to_string(),
            Some("https://example.com/avatar.jpg".to_string()),
            UserRole::User,
        );

        assert!(user.validate().is_ok());

        // Test invalid email
        user.email = Some("invalid-email".to_string());
        assert!(user.validate().is_err());

        // Reset and test invalid display name
        user.email = Some("valid@example.com".to_string());
        user.display_name = Some("".to_string());
        assert!(user.validate().is_err());

        // Reset and test invalid avatar URL
        user.display_name = Some("Valid User".to_string());
        user.avatar_url = Some("invalid-url".to_string());
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_create_user_request_validation() {
        let valid_request = CreateUserRequest {
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            role: Some(UserRole::User),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = CreateUserRequest {
            email: "invalid-email".to_string(),
            display_name: "Test User".to_string(),
            avatar_url: None,
            role: None,
        };
        assert!(invalid_email.validate().is_err());

        let invalid_display_name = CreateUserRequest {
            email: "test@example.com".to_string(),
            display_name: "".to_string(),
            avatar_url: None,
            role: None,
        };
        assert!(invalid_display_name.validate().is_err());
    }

    #[test]
    fn test_display_name_or_fallback() {
        let user_with_display_name = User::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            None,
            UserRole::User,
        );
        assert_eq!(user_with_display_name.display_name_or_fallback(), "Test User");

        let user_without_display_name = User::new(
            "test@example.com".to_string(),
            "".to_string(),
            None,
            UserRole::User,
        );
        user_without_display_name.display_name = None;
        assert_eq!(user_without_display_name.display_name_or_fallback(), "test@example.com");

        let user_nothing = User::new(
            "test@example.com".to_string(),
            "".to_string(),
            None,
            UserRole::User,
        );
        user_nothing.display_name = None;
        user_nothing.email = None;
        assert_eq!(user_nothing.display_name_or_fallback(), "Unknown User");
    }

    #[test]
    fn test_update_last_login() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            None,
            UserRole::User,
        );

        assert!(user.last_login_at.is_none());

        let original_updated_at = user.updated_at.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        user.update_last_login();

        assert!(user.last_login_at.is_some());
        assert_ne!(user.updated_at, original_updated_at);
    }
}