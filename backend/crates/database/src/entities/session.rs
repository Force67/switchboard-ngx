//! Session entity definitions

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub token: String,
    pub provider: AuthProvider,
    pub expires_at: String,
    pub created_at: String,
    pub last_accessed_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: i64,
    pub token: String,
    pub provider: AuthProvider,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthProvider {
    Email,
    GitHub,
    Google,
    Development,
}

impl AuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthProvider::Email => "email",
            AuthProvider::GitHub => "github",
            AuthProvider::Google => "google",
            AuthProvider::Development => "development",
        }
    }
}

impl From<&str> for AuthProvider {
    fn from(s: &str) -> Self {
        match s {
            "github" => AuthProvider::GitHub,
            "google" => AuthProvider::Google,
            "development" => AuthProvider::Development,
            _ => AuthProvider::Email,
        }
    }
}

impl ToString for AuthProvider {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}