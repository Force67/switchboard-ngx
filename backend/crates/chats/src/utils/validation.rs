//! Validation utilities.

use crate::types::ChatError;

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate email format
    pub fn email(email: &str) -> Result<(), ChatError> {
        if email.trim().is_empty() {
            return Err(ChatError::validation("Email cannot be empty"));
        }

        if !email.contains('@') || !email.contains('.') {
            return Err(ChatError::validation("Invalid email format"));
        }

        if email.len() > 255 {
            return Err(ChatError::validation("Email too long (max 255 characters)"));
        }

        // Basic email validation regex
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).map_err(|e| ChatError::internal(format!("Failed to compile email regex: {}", e)))?;

        if !email_regex.is_match(email) {
            return Err(ChatError::validation("Invalid email format"));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn uuid(uuid_str: &str) -> Result<(), ChatError> {
        if uuid_str.trim().is_empty() {
            return Err(ChatError::validation("UUID cannot be empty"));
        }

        uuid::Uuid::parse_str(uuid_str)
            .map_err(|_| ChatError::validation("Invalid UUID format"))?;

        Ok(())
    }

    /// Validate chat title
    pub fn chat_title(title: &str) -> Result<(), ChatError> {
        if title.trim().is_empty() {
            return Err(ChatError::validation("Chat title cannot be empty"));
        }

        if title.len() > 255 {
            return Err(ChatError::validation("Chat title too long (max 255 characters)"));
        }

        Ok(())
    }

    /// Validate message content
    pub fn message_content(content: &str) -> Result<(), ChatError> {
        if content.trim().is_empty() {
            return Err(ChatError::validation("Message content cannot be empty"));
        }

        if content.len() > 100_000 {
            return Err(ChatError::validation("Message content too long (max 100,000 characters)"));
        }

        Ok(())
    }

    /// Validate file name
    pub fn file_name(file_name: &str) -> Result<(), ChatError> {
        if file_name.trim().is_empty() {
            return Err(ChatError::validation("File name cannot be empty"));
        }

        if file_name.len() > 255 {
            return Err(ChatError::validation("File name too long (max 255 characters)"));
        }

        // Check for invalid characters in file names
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for char in invalid_chars {
            if file_name.contains(char) {
                return Err(ChatError::validation(format!("File name contains invalid character: {}", char)));
            }
        }

        Ok(())
    }

    /// Validate file size
    pub fn file_size(size_bytes: i64, max_size_bytes: i64) -> Result<(), ChatError> {
        if size_bytes <= 0 {
            return Err(ChatError::validation("File size must be positive"));
        }

        if size_bytes > max_size_bytes {
            return Err(ChatError::validation(format!(
                "File size too large (max {} MB)",
                max_size_bytes / (1024 * 1024)
            )));
        }

        Ok(())
    }

    /// Validate MIME type
    pub fn mime_type(mime_type: &str, allowed_types: &[&str]) -> Result<(), ChatError> {
        if mime_type.trim().is_empty() {
            return Err(ChatError::validation("MIME type cannot be empty"));
        }

        if !allowed_types.contains(&mime_type) {
            return Err(ChatError::validation("File type not allowed"));
        }

        Ok(())
    }

    /// Validate pagination parameters
    pub fn pagination(page: Option<u32>, limit: Option<u32>) -> Result<(u32, u32), ChatError> {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20);

        if page == 0 {
            return Err(ChatError::validation("Page number must be greater than 0"));
        }

        if limit == 0 {
            return Err(ChatError::validation("Page limit must be greater than 0"));
        }

        if limit > 100 {
            return Err(ChatError::validation("Page limit cannot exceed 100"));
        }

        Ok((page, limit))
    }

    /// Validate role string
    pub fn role(role: &str) -> Result<(), ChatError> {
        let valid_roles = ["owner", "admin", "member"];

        if !valid_roles.contains(&role.to_lowercase().as_str()) {
            return Err(ChatError::validation("Invalid role"));
        }

        Ok(())
    }

    /// Validate URL format
    pub fn url(url: &str) -> Result<(), ChatError> {
        if url.trim().is_empty() {
            return Err(ChatError::validation("URL cannot be empty"));
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ChatError::validation("URL must start with http:// or https://"));
        }

        // More comprehensive URL validation would require additional dependencies
        // For now, basic check should be sufficient
        Ok(())
    }

    /// Sanitize string input
    pub fn sanitize_string(input: &str, max_length: usize) -> Result<String, ChatError> {
        let sanitized = input.trim();

        if sanitized.is_empty() {
            return Err(ChatError::validation("Input cannot be empty"));
        }

        if sanitized.len() > max_length {
            return Err(ChatError::validation(format!(
                "Input too long (max {} characters)",
                max_length
            )));
        }

        Ok(sanitized.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_email() {
        assert!(Validator::email("test@example.com").is_ok());
        assert!(Validator::email("user.name+tag@domain.co.uk").is_ok());
        assert!(Validator::email("invalid-email").is_err());
        assert!(Validator::email("").is_err());
        assert!(Validator::email("test@").is_err());
        assert!(Validator::email("@example.com").is_err());
    }

    #[test]
    fn test_validator_uuid() {
        assert!(Validator::uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(Validator::uuid("invalid-uuid").is_err());
        assert!(Validator::uuid("").is_err());
    }

    #[test]
    fn test_validator_chat_title() {
        assert!(Validator::chat_title("Valid Title").is_ok());
        assert!(Validator::chat_title("").is_err());
        assert!(Validator::chat_title(" ").is_err());

        let too_long = "a".repeat(256);
        assert!(Validator::chat_title(&too_long).is_err());
    }

    #[test]
    fn test_validator_message_content() {
        assert!(Validator::message_content("Valid message").is_ok());
        assert!(Validator::message_content("").is_err());
        assert!(Validator::message_content(" ").is_err());

        let too_long = "a".repeat(100_001);
        assert!(Validator::message_content(&too_long).is_err());
    }

    #[test]
    fn test_validator_file_name() {
        assert!(Validator::file_name("valid-file.txt").is_ok());
        assert!(Validator::file_name("document.pdf").is_ok());
        assert!(Validator::file_name("").is_err());
        assert!(Validator::file_name("invalid/file.txt").is_err());
        assert!(Validator::file_name("invalid:file.txt").is_err());
    }

    #[test]
    fn test_validator_file_size() {
        assert!(Validator::file_size(1024, 10_485_760).is_ok()); // 1KB, 10MB limit
        assert!(Validator::file_size(0, 10_485_760).is_err());
        assert!(Validator::file_size(-1, 10_485_760).is_err());
        assert!(Validator::file_size(20_971_520, 10_485_760).is_err()); // 20MB, 10MB limit
    }

    #[test]
    fn test_validator_mime_type() {
        let allowed = vec!["image/jpeg", "image/png", "application/pdf"];
        assert!(Validator::mime_type("image/jpeg", &allowed).is_ok());
        assert!(Validator::mime_type("application/pdf", &allowed).is_ok());
        assert!(Validator::mime_type("application/exe", &allowed).is_err());
        assert!(Validator::mime_type("", &allowed).is_err());
    }

    #[test]
    fn test_validator_pagination() {
        assert!(Validator::pagination(None, None).is_ok());
        assert!(Validator::pagination(Some(2), Some(50)).is_ok());
        assert!(Validator::pagination(Some(0), None).is_err());
        assert!(Validator::pagination(None, Some(0)).is_err());
        assert!(Validator::pagination(None, Some(101)).is_err());
    }

    #[test]
    fn test_validator_role() {
        assert!(Validator::role("owner").is_ok());
        assert!(Validator::role("admin").is_ok());
        assert!(Validator::role("member").is_ok());
        assert!(Validator::role("OWNER").is_ok()); // Case insensitive
        assert!(Validator::role("invalid").is_err());
        assert!(Validator::role("").is_err());
    }

    #[test]
    fn test_validator_url() {
        assert!(Validator::url("https://example.com").is_ok());
        assert!(Validator::url("http://localhost:3000").is_ok());
        assert!(Validator::url("ftp://example.com").is_err());
        assert!(Validator::url("example.com").is_err());
        assert!(Validator::url("").is_err());
    }

    #[test]
    fn test_validator_sanitize_string() {
        assert_eq!(Validator::sanitize_string("  hello  ", 10).unwrap(), "hello");
        assert!(Validator::sanitize_string("", 10).is_err());
        assert!(Validator::sanitize_string("  ", 10).is_err());
        assert!(Validator::sanitize_string("hello", 3).is_err());
    }
}