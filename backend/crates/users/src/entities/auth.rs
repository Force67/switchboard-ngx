use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    /// Database primary key
    pub id: i64,
    /// Session token
    pub token: String,
    /// User ID this session belongs to
    pub user_id: i64,
    /// Session creation timestamp
    pub created_at: String,
    /// Session expiration timestamp
    pub expires_at: String,
    /// Last activity timestamp
    pub last_activity_at: String,
    /// User agent string
    pub user_agent: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// Whether this session is active
    pub is_active: bool,
}

/// Authentication provider enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    GitHub,
    Google,
    Email,
    Development,
}

/// Request to create a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    /// User ID
    pub user_id: i64,
    /// User agent (optional)
    pub user_agent: Option<String>,
    /// IP address (optional)
    pub ip_address: Option<String>,
    /// Custom expiration in seconds (optional)
    pub expires_in_seconds: Option<u64>,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Authentication provider
    pub provider: AuthProvider,
    /// Provider-specific auth code
    pub auth_code: String,
    /// Redirect URI for OAuth flow
    pub redirect_uri: Option<String>,
    /// User agent (optional)
    pub user_agent: Option<String>,
    /// IP address (optional)
    pub ip_address: Option<String>,
}

/// Registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// User email
    pub email: String,
    /// Display name
    pub display_name: String,
    /// Password (for email auth)
    pub password: Option<String>,
    /// Authentication provider
    pub provider: AuthProvider,
    /// Provider-specific auth data
    pub provider_data: Option<serde_json::Value>,
    /// User agent (optional)
    pub user_agent: Option<String>,
    /// IP address (optional)
    pub ip_address: Option<String>,
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization URL
    pub auth_url: String,
    /// Token URL
    pub token_url: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes to request
    pub scopes: Vec<String>,
}

impl AuthSession {
    /// Create a new session
    pub fn new(
        user_id: i64,
        user_agent: Option<String>,
        ip_address: Option<String>,
        expires_in_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now
            .checked_add_signed(chrono::Duration::seconds(expires_in_seconds as i64))
            .unwrap_or_else(|| now + chrono::Duration::hours(24));

        Self {
            id: 0, // Will be set by database
            token: Uuid::new_v4().to_string(),
            user_id,
            created_at: now.to_rfc3339(),
            expires_at: expires_at.to_rfc3339(),
            last_activity_at: now.to_rfc3339(),
            user_agent,
            ip_address,
            is_active: true,
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = DateTime::parse_from_rfc3339(&self.expires_at) {
            Utc::now() >= expires.with_timezone(&Utc)
        } else {
            true // Treat invalid dates as expired
        }
    }

    /// Check if session is valid
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity_at = Utc::now().to_rfc3339();
    }

    /// Validate session data
    pub fn validate(&self) -> Result<(), String> {
        if self.token.trim().is_empty() {
            return Err("Session token cannot be empty".to_string());
        }

        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        // Validate timestamp formats
        if let Err(_) = DateTime::parse_from_rfc3339(&self.created_at) {
            return Err("Invalid created_at timestamp format".to_string());
        }

        if let Err(_) = DateTime::parse_from_rfc3339(&self.expires_at) {
            return Err("Invalid expires_at timestamp format".to_string());
        }

        if let Err(_) = DateTime::parse_from_rfc3339(&self.last_activity_at) {
            return Err("Invalid last_activity_at timestamp format".to_string());
        }

        // Validate token format
        if let Err(_) = Uuid::parse_str(&self.token) {
            return Err("Invalid session token format".to_string());
        }

        Ok(())
    }
}

impl From<&str> for AuthProvider {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "github" => AuthProvider::GitHub,
            "google" => AuthProvider::Google,
            "email" => AuthProvider::Email,
            "development" => AuthProvider::Development,
            _ => AuthProvider::Development, // Default to development
        }
    }
}

impl From<AuthProvider> for String {
    fn from(provider: AuthProvider) -> Self {
        match provider {
            AuthProvider::GitHub => "github".to_string(),
            AuthProvider::Google => "google".to_string(),
            AuthProvider::Email => "email".to_string(),
            AuthProvider::Development => "development".to_string(),
        }
    }
}

impl CreateSessionRequest {
    /// Get default expiration in seconds (24 hours)
    pub fn default_expiration_seconds() -> u64 {
        24 * 60 * 60 // 24 hours
    }

    /// Validate the create session request
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        if let Some(ref user_agent) = self.user_agent {
            if user_agent.len() > 500 {
                return Err("User agent too long (max 500 characters)".to_string());
            }
        }

        if let Some(ref ip_address) = self.ip_address {
            // Basic IP validation
            if ip_address.is_empty() || ip_address.len() > 45 {
                return Err("Invalid IP address format".to_string());
            }
        }

        if let Some(expires_in) = self.expires_in_seconds {
            if expires_in == 0 || expires_in > 30 * 24 * 60 * 60 { // Max 30 days
                return Err("Expiration must be between 1 second and 30 days".to_string());
            }
        }

        Ok(())
    }
}

impl LoginRequest {
    /// Validate the login request
    pub fn validate(&self) -> Result<(), String> {
        if self.auth_code.trim().is_empty() {
            return Err("Auth code cannot be empty".to_string());
        }

        if self.auth_code.len() > 1000 {
            return Err("Auth code too long (max 1000 characters)".to_string());
        }

        if let Some(ref redirect_uri) = self.redirect_uri {
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                return Err("Redirect URI must be a valid HTTP/HTTPS URL".to_string());
            }
        }

        Ok(())
    }
}

impl RegisterRequest {
    /// Validate the registration request
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

        // For email auth, validate password
        if matches!(self.provider, AuthProvider::Email) {
            if let Some(ref password) = self.password {
                if password.len() < 8 {
                    return Err("Password must be at least 8 characters long".to_string());
                }

                if password.len() > 128 {
                    return Err("Password too long (max 128 characters)".to_string());
                }
            } else {
                return Err("Password is required for email authentication".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_session_creation() {
        let session = AuthSession::new(
            1,
            Some("Mozilla/5.0".to_string()),
            Some("127.0.0.1".to_string()),
            3600, // 1 hour
        );

        assert_eq!(session.user_id, 1);
        assert_eq!(session.user_agent, Some("Mozilla/5.0".to_string()));
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
        assert!(session.is_valid());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_auth_session_expiration() {
        let mut session = AuthSession::new(1, None, None, 0); // Expires immediately

        // Wait a moment to ensure it's expired
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(session.is_expired());
        assert!(!session.is_valid());
    }

    #[test]
    fn test_auth_provider_conversion() {
        assert_eq!(AuthProvider::from("github"), AuthProvider::GitHub);
        assert_eq!(AuthProvider::from("google"), AuthProvider::Google);
        assert_eq!(AuthProvider::from("email"), AuthProvider::Email);
        assert_eq!(AuthProvider::from("development"), AuthProvider::Development);
        assert_eq!(AuthProvider::from("unknown"), AuthProvider::Development);

        assert_eq!(String::from(AuthProvider::GitHub), "github");
        assert_eq!(String::from(AuthProvider::Google), "google");
        assert_eq!(String::from(AuthProvider::Email), "email");
        assert_eq!(String::from(AuthProvider::Development), "development");
    }

    #[test]
    fn test_update_activity() {
        let mut session = AuthSession::new(1, None, None, 3600);
        let original_activity = session.last_activity_at.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        session.update_activity();
        assert_ne!(session.last_activity_at, original_activity);
    }

    #[test]
    fn test_create_session_request_validation() {
        let valid_request = CreateSessionRequest {
            user_id: 1,
            user_agent: Some("Mozilla/5.0".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            expires_in_seconds: Some(3600),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_user_id = CreateSessionRequest {
            user_id: 0,
            user_agent: None,
            ip_address: None,
            expires_in_seconds: None,
        };
        assert!(invalid_user_id.validate().is_err());

        let invalid_expiration = CreateSessionRequest {
            user_id: 1,
            user_agent: None,
            ip_address: None,
            expires_in_seconds: Some(0),
        };
        assert!(invalid_expiration.validate().is_err());
    }

    #[test]
    fn test_login_request_validation() {
        let valid_request = LoginRequest {
            provider: AuthProvider::GitHub,
            auth_code: "valid_auth_code".to_string(),
            redirect_uri: Some("https://example.com/callback".to_string()),
            user_agent: None,
            ip_address: None,
        };
        assert!(valid_request.validate().is_ok());

        let invalid_auth_code = LoginRequest {
            provider: AuthProvider::GitHub,
            auth_code: "".to_string(),
            redirect_uri: None,
            user_agent: None,
            ip_address: None,
        };
        assert!(invalid_auth_code.validate().is_err());

        let invalid_redirect_uri = LoginRequest {
            provider: AuthProvider::GitHub,
            auth_code: "valid_auth_code".to_string(),
            redirect_uri: Some("invalid-uri".to_string()),
            user_agent: None,
            ip_address: None,
        };
        assert!(invalid_redirect_uri.validate().is_err());
    }

    #[test]
    fn test_register_request_validation() {
        let valid_email_request = RegisterRequest {
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            password: Some("securepassword123".to_string()),
            provider: AuthProvider::Email,
            provider_data: None,
            user_agent: None,
            ip_address: None,
        };
        assert!(valid_email_request.validate().is_ok());

        let valid_oauth_request = RegisterRequest {
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            password: None,
            provider: AuthProvider::GitHub,
            provider_data: Some(serde_json::json!({"github_id": 123})),
            user_agent: None,
            ip_address: None,
        };
        assert!(valid_oauth_request.validate().is_ok());

        let email_request_no_password = RegisterRequest {
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            password: None,
            provider: AuthProvider::Email,
            provider_data: None,
            user_agent: None,
            ip_address: None,
        };
        assert!(email_request_no_password.validate().is_err());

        let weak_password_request = RegisterRequest {
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            password: Some("weak".to_string()),
            provider: AuthProvider::Email,
            provider_data: None,
            user_agent: None,
            ip_address: None,
        };
        assert!(weak_password_request.validate().is_err());
    }
}