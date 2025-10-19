//! Validation utilities.

use switchboard_database::ChatError;

/// Validation utilities
pub struct Validator;

impl Validator {
    /// Validate email format
    pub fn email(email: &str) -> Result<(), ChatError> {
        if email.trim().is_empty() {
            return Err(ChatError::DatabaseError("Email cannot be empty".to_string()));
        }

        if !email.contains('@') || !email.contains('.') {
            return Err(ChatError::DatabaseError("Invalid email format".to_string()));
        }

        if email.len() > 255 {
            return Err(ChatError::DatabaseError("Email too long (max 255 characters)".to_string()));
        }

        // Basic email validation regex
        let email_regex = regex::Regex::new(
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        ).map_err(|e| ChatError::DatabaseError(format!("Failed to compile email regex: {}", e)))?;

        if !email_regex.is_match(email) {
            return Err(ChatError::DatabaseError("Invalid email format".to_string()));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn uuid(uuid_str: &str) -> Result<(), ChatError> {
        if uuid_str.trim().is_empty() {
            return Err(ChatError::DatabaseError("UUID cannot be empty".to_string()));
        }

        uuid::Uuid::parse_str(uuid_str)
            .map_err(|_| ChatError::DatabaseError("Invalid UUID format".to_string()))?;

        Ok(())
    }

    /// Validate chat title
    pub fn chat_title(title: &str) -> Result<(), ChatError> {
        if title.trim().is_empty() {
            return Err(ChatError::DatabaseError("Chat title cannot be empty".to_string()));
        }

        if title.len() > 255 {
            return Err(ChatError::DatabaseError("Chat title too long (max 255 characters)".to_string()));
        }

        Ok(())
    }

    /// Validate message content
    pub fn message_content(content: &str) -> Result<(), ChatError> {
        if content.trim().is_empty() {
            return Err(ChatError::DatabaseError("Message content cannot be empty".to_string()));
        }

        if content.len() > 100_000 {
            return Err(ChatError::DatabaseError("Message content too long (max 100,000 characters)".to_string()));
        }

        Ok(())
    }

    /// Validate file name
    pub fn file_name(file_name: &str) -> Result<(), ChatError> {
        if file_name.trim().is_empty() {
            return Err(ChatError::DatabaseError("File name cannot be empty".to_string()));
        }

        if file_name.len() > 255 {
            return Err(ChatError::DatabaseError("File name too long (max 255 characters)".to_string()));
        }

        // Check for invalid characters in file names
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        for char in invalid_chars {
            if file_name.contains(char) {
                return Err(ChatError::DatabaseError(format!("File name contains invalid character: {}", char)));
            }
        }

        Ok(())
    }

    /// Validate file size
    pub fn file_size(size_bytes: i64, max_size_bytes: i64) -> Result<(), ChatError> {
        if size_bytes <= 0 {
            return Err(ChatError::DatabaseError("File size must be positive".to_string()));
        }

        if size_bytes > max_size_bytes {
            return Err(ChatError::DatabaseError(format!(
                "File size too large (max {} MB)",
                max_size_bytes / (1024 * 1024)
            )));
        }

        Ok(())
    }

    /// Validate MIME type
    pub fn mime_type(mime_type: &str, allowed_types: &[&str]) -> Result<(), ChatError> {
        if mime_type.trim().is_empty() {
            return Err(ChatError::DatabaseError("MIME type cannot be empty".to_string()));
        }

        if !allowed_types.contains(&mime_type) {
            return Err(ChatError::DatabaseError("File type not allowed".to_string()));
        }

        Ok(())
    }

    /// Validate pagination parameters
    pub fn pagination(page: Option<u32>, limit: Option<u32>) -> Result<(u32, u32), ChatError> {
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(20);

        if page == 0 {
            return Err(ChatError::DatabaseError("Page number must be greater than 0".to_string()));
        }

        if limit == 0 {
            return Err(ChatError::DatabaseError("Page limit must be greater than 0".to_string()));
        }

        if limit > 100 {
            return Err(ChatError::DatabaseError("Page limit cannot exceed 100".to_string()));
        }

        Ok((page, limit))
    }

    /// Validate role string
    pub fn role(role: &str) -> Result<(), ChatError> {
        let valid_roles = ["owner", "admin", "member"];

        if !valid_roles.contains(&role.to_lowercase().as_str()) {
            return Err(ChatError::DatabaseError("Invalid role".to_string()));
        }

        Ok(())
    }

    /// Validate URL format
    pub fn url(url: &str) -> Result<(), ChatError> {
        if url.trim().is_empty() {
            return Err(ChatError::DatabaseError("URL cannot be empty".to_string()));
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ChatError::DatabaseError("URL must start with http:// or https://".to_string()));
        }

        // More comprehensive URL validation would require additional dependencies
        // For now, basic check should be sufficient
        Ok(())
    }

    /// Sanitize string input
    pub fn sanitize_string(input: &str, max_length: usize) -> Result<String, ChatError> {
        let sanitized = input.trim();

        if sanitized.is_empty() {
            return Err(ChatError::DatabaseError("Input cannot be empty".to_string()));
        }

        if sanitized.len() > max_length {
            return Err(ChatError::DatabaseError(format!(
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