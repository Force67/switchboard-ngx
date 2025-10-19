//! Input validation utilities.

use regex::Regex;
use crate::types::UserError;

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), UserError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .map_err(|_| UserError::ValidationFailed("Invalid email regex".to_string()))?;

    if !email_regex.is_match(email) {
        return Err(UserError::ValidationFailed("Invalid email format".to_string()));
    }

    if email.len() > 255 {
        return Err(UserError::ValidationFailed("Email too long".to_string()));
    }

    Ok(())
}

/// Validate password strength requirements
pub fn validate_password(password: &str) -> Result<(), UserError> {
    if password.len() < 8 {
        return Err(UserError::ValidationFailed("Password must be at least 8 characters long".to_string()));
    }

    if password.len() > 128 {
        return Err(UserError::ValidationFailed("Password must be less than 128 characters long".to_string()));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_lowercase {
        return Err(UserError::ValidationFailed("Password must contain at least one lowercase letter".to_string()));
    }

    if !has_uppercase {
        return Err(UserError::ValidationFailed("Password must contain at least one uppercase letter".to_string()));
    }

    if !has_digit {
        return Err(UserError::ValidationFailed("Password must contain at least one digit".to_string()));
    }

    Ok(())
}

/// Validate username
pub fn validate_username(username: &str) -> Result<(), UserError> {
    if username.len() < 3 {
        return Err(UserError::ValidationFailed("Username must be at least 3 characters long".to_string()));
    }

    if username.len() > 30 {
        return Err(UserError::ValidationFailed("Username must be less than 30 characters long".to_string()));
    }

    let username_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
        .map_err(|_| UserError::ValidationFailed("Invalid username regex".to_string()))?;

    if !username_regex.is_match(username) {
        return Err(UserError::ValidationFailed("Username can only contain letters, numbers, underscores, and hyphens".to_string()));
    }

    Ok(())
}

/// Validate display name
pub fn validate_display_name(display_name: &str) -> Result<(), UserError> {
    if display_name.trim().is_empty() {
        return Err(UserError::ValidationFailed("Display name cannot be empty".to_string()));
    }

    if display_name.len() > 50 {
        return Err(UserError::ValidationFailed("Display name must be less than 50 characters long".to_string()));
    }

    // Allow most characters but prevent obvious problematic ones
    let disallowed_chars = ['\n', '\r', '\t', '\0'];
    if display_name.chars().any(|c| disallowed_chars.contains(&c)) {
        return Err(UserError::ValidationFailed("Display name contains invalid characters".to_string()));
    }

    Ok(())
}

/// Validate URL format
pub fn validate_url(url: &str) -> Result<(), UserError> {
    if url.is_empty() {
        return Ok(()); // Empty URLs are allowed (optional fields)
    }

    if url.len() > 2048 {
        return Err(UserError::ValidationFailed("URL too long".to_string()));
    }

    let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$")
        .map_err(|_| UserError::ValidationFailed("Invalid URL regex".to_string()))?;

    if !url_regex.is_match(url) {
        return Err(UserError::ValidationFailed("Invalid URL format".to_string()));
    }

    Ok(())
}

/// Sanitize and validate user input
pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .replace('\0', "") // Remove null bytes
        .chars()
        .filter(|c| c.is_ascii() || *c == ' ') // Keep ASCII and spaces for simplicity
        .collect::<String>()
        .chars()
        .take(1000) // Limit length
        .collect()
}

/// Check if a string contains potentially malicious content
pub fn is_safe_content(content: &str) -> bool {
    let suspicious_patterns = [
        r"<script",
        r"javascript:",
        r"onload=",
        r"onerror=",
        r"onclick=",
        r"<iframe",
        r"eval(",
        r"document\.",
        r"window\.",
    ];

    let content_lower = content.to_lowercase();

    !suspicious_patterns.iter().any(|pattern| {
        Regex::new(pattern).map(|regex| regex.is_match(&content_lower)).unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());

        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("test@").is_err());
        assert!(validate_email("a".repeat(256).as_str()).is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("Password123").is_ok());
        assert!(validate_password("StrongPassword456!").is_ok());

        assert!(validate_password("weak").is_err());
        assert!(validate_password("nouppercase123").is_err());
        assert!(validate_password("NOLOWERCASE123").is_err());
        assert!(validate_password("NoDigits!").is_err());
        assert!(validate_password("Short1").is_err());
        assert!(validate_password("a".repeat(129).as_str()).is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("validuser").is_ok());
        assert!(validate_username("user_123").is_ok());
        assert!(validate_username("test-user").is_ok());

        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("user@name").is_err()); // Invalid character
        assert!(validate_username("a".repeat(31).as_str()).is_err()); // Too long
    }

    #[test]
    fn test_display_name_validation() {
        assert!(validate_display_name("John Doe").is_ok());
        assert!(validate_display_name("用户名").is_ok()); // Unicode allowed

        assert!(validate_display_name("").is_err()); // Empty
        assert!(validate_display_name("   ").is_err()); // Whitespace only
        assert!(validate_display_name("Name\nWith\nNewlines").is_err()); // Invalid chars
        assert!(validate_display_name("a".repeat(51).as_str()).is_err()); // Too long
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:3000").is_ok());
        assert!(validate_url("").is_ok()); // Empty allowed

        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("a".repeat(2049).as_str()).is_err()); // Too long
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("  hello world  "), "hello world");
        assert_eq!(sanitize_input("hello\0world"), "helloworld");
        assert_eq!(sanitize_input("a".repeat(1500).as_str()), "a".repeat(1000).as_str());
    }

    #[test]
    fn test_is_safe_content() {
        assert!(is_safe_content("This is safe content"));
        assert!(is_safe_content("Regular text with links: https://example.com"));

        assert!(!is_safe_content("<script>alert('xss')</script>"));
        assert!(!is_safe_content("javascript:alert('xss')"));
        assert!(!is_safe_content("<div onclick=\"alert('xss')\">Click me</div>"));
    }
}