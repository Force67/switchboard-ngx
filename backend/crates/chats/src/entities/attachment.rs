use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a file attachment for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// Message ID this attachment belongs to
    pub message_id: i64,
    /// Original filename
    pub file_name: String,
    /// MIME type
    pub file_type: String,
    /// File size in bytes
    pub file_size_bytes: i64,
    /// URL to access the file
    pub file_url: String,
    /// Attachment type
    pub attachment_type: AttachmentType,
    /// Creation timestamp
    pub created_at: String,
}

/// Attachment type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    Image,
    Document,
    Audio,
    Video,
    File,
}

impl From<&str> for AttachmentType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            s if s.starts_with("image/") => AttachmentType::Image,
            s if s.starts_with("audio/") => AttachmentType::Audio,
            s if s.starts_with("video/") => AttachmentType::Video,
            s if s.contains("pdf") || s.contains("doc") || s.contains("text") => AttachmentType::Document,
            _ => AttachmentType::File,
        }
    }
}

impl From<AttachmentType> for String {
    fn from(attachment_type: AttachmentType) -> Self {
        match attachment_type {
            AttachmentType::Image => "image".to_string(),
            AttachmentType::Document => "document".to_string(),
            AttachmentType::Audio => "audio".to_string(),
            AttachmentType::Video => "video".to_string(),
            AttachmentType::File => "file".to_string(),
        }
    }
}

/// Request to create a new attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachmentRequest {
    /// Original filename
    pub file_name: String,
    /// MIME type
    pub file_type: String,
    /// File size in bytes
    pub file_size_bytes: i64,
    /// URL to access the file
    pub file_url: String,
}

impl MessageAttachment {
    /// Create a new attachment instance
    pub fn new(
        message_id: i64,
        file_name: String,
        file_type: String,
        file_size_bytes: i64,
        file_url: String,
    ) -> Self {
        let attachment_type: AttachmentType = file_type.as_str().into();
        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            message_id,
            file_name,
            file_type,
            file_size_bytes,
            file_url,
            attachment_type,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    /// Check if this is an image attachment
    pub fn is_image(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Image)
    }

    /// Check if this is a document attachment
    pub fn is_document(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Document)
    }

    /// Check if this is an audio attachment
    pub fn is_audio(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Audio)
    }

    /// Check if this is a video attachment
    pub fn is_video(&self) -> bool {
        matches!(self.attachment_type, AttachmentType::Video)
    }

    /// Get human-readable file size
    pub fn formatted_size(&self) -> String {
        let size = self.file_size_bytes as f64;

        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as i64, UNITS[unit_index])
        } else {
            format!("{} {}", size, UNITS[unit_index])
        }
    }

    /// Validate attachment data
    pub fn validate(&self) -> Result<(), String> {
        if self.file_name.trim().is_empty() {
            return Err("File name cannot be empty".to_string());
        }

        if self.file_name.len() > 255 {
            return Err("File name too long (max 255 characters)".to_string());
        }

        if self.file_type.trim().is_empty() {
            return Err("File type cannot be empty".to_string());
        }

        if self.file_url.trim().is_empty() {
            return Err("File URL cannot be empty".to_string());
        }

        if self.file_size_bytes <= 0 {
            return Err("File size must be positive".to_string());
        }

        // Check file size limits (100MB max for now)
        const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024; // 100MB
        if self.file_size_bytes > MAX_FILE_SIZE {
            return Err("File size too large (max 100MB)".to_string());
        }

        Ok(())
    }

    /// Check if the file type is allowed
    pub fn is_allowed_type(&self) -> bool {
        // Basic MIME type validation
        const ALLOWED_TYPES: &[&str] = &[
            // Images
            "image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp",
            // Documents
            "application/pdf", "text/plain", "application/msword",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            // Audio
            "audio/mpeg", "audio/wav", "audio/ogg",
            // Video
            "video/mp4", "video/webm", "video/ogg",
            // Archives (limited)
            "application/zip", "application/x-tar",
        ];

        ALLOWED_TYPES.contains(&self.file_type.as_str())
    }
}

impl CreateAttachmentRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), String> {
        if self.file_name.trim().is_empty() {
            return Err("File name cannot be empty".to_string());
        }

        if self.file_name.len() > 255 {
            return Err("File name too long (max 255 characters)".to_string());
        }

        if self.file_type.trim().is_empty() {
            return Err("File type cannot be empty".to_string());
        }

        if self.file_url.trim().is_empty() {
            return Err("File URL cannot be empty".to_string());
        }

        if self.file_size_bytes <= 0 {
            return Err("File size must be positive".to_string());
        }

        // Check file size limits
        const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024; // 100MB
        if self.file_size_bytes > MAX_FILE_SIZE {
            return Err("File size too large (max 100MB)".to_string());
        }

        // Basic MIME type validation
        const ALLOWED_TYPES: &[&str] = &[
            "image/jpeg", "image/jpg", "image/png", "image/gif", "image/webp",
            "application/pdf", "text/plain",
            "application/msword",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "audio/mpeg", "audio/wav", "audio/ogg",
            "video/mp4", "video/webm", "video/ogg",
            "application/zip", "application/x-tar",
        ];

        if !ALLOWED_TYPES.contains(&self.file_type.as_str()) {
            return Err("File type not allowed".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_creation() {
        let attachment = MessageAttachment::new(
            1,
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1024,
            "https://example.com/test.jpg".to_string(),
        );

        assert_eq!(attachment.message_id, 1);
        assert_eq!(attachment.file_name, "test.jpg");
        assert_eq!(attachment.file_type, "image/jpeg");
        assert_eq!(attachment.file_size_bytes, 1024);
        assert!(attachment.is_image());
        assert!(!attachment.is_document());
        assert!(attachment.is_allowed_type());
    }

    #[test]
    fn test_attachment_type_conversion() {
        assert_eq!(AttachmentType::from("image/jpeg"), AttachmentType::Image);
        assert_eq!(AttachmentType::from("audio/mpeg"), AttachmentType::Audio);
        assert_eq!(AttachmentType::from("video/mp4"), AttachmentType::Video);
        assert_eq!(AttachmentType::from("application/pdf"), AttachmentType::Document);
        assert_eq!(AttachmentType::from("unknown/type"), AttachmentType::File);

        assert_eq!(String::from(AttachmentType::Image), "image");
        assert_eq!(String::from(AttachmentType::Document), "document");
        assert_eq!(String::from(AttachmentType::Audio), "audio");
        assert_eq!(String::from(AttachmentType::Video), "video");
        assert_eq!(String::from(AttachmentType::File), "file");
    }

    #[test]
    fn test_formatted_size() {
        let attachment = MessageAttachment::new(
            1,
            "test.txt".to_string(),
            "text/plain".to_string(),
            1024,
            "https://example.com/test.txt".to_string(),
        );

        assert_eq!(attachment.formatted_size(), "1 KB");

        let large_attachment = MessageAttachment::new(
            1,
            "large.jpg".to_string(),
            "image/jpeg".to_string(),
            2_500_000,
            "https://example.com/large.jpg".to_string(),
        );

        let formatted = large_attachment.formatted_size();
        assert!(formatted.starts_with("2.3") && formatted.ends_with(" MB"));
    }

    #[test]
    fn test_attachment_validation() {
        let mut attachment = MessageAttachment::new(
            1,
            "valid.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
            "https://example.com/valid.pdf".to_string(),
        );

        assert!(attachment.validate().is_ok());

        attachment.file_name = "".to_string();
        assert!(attachment.validate().is_err());

        attachment.file_name = "valid.pdf".to_string();
        attachment.file_size_bytes = -1;
        assert!(attachment.validate().is_err());
    }

    #[test]
    fn test_create_attachment_request_validation() {
        let valid_request = CreateAttachmentRequest {
            file_name: "test.jpg".to_string(),
            file_type: "image/jpeg".to_string(),
            file_size_bytes: 1024,
            file_url: "https://example.com/test.jpg".to_string(),
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = CreateAttachmentRequest {
            file_name: "".to_string(),
            file_type: "image/jpeg".to_string(),
            file_size_bytes: 1024,
            file_url: "https://example.com/test.jpg".to_string(),
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_file_type_validation() {
        let allowed_attachment = MessageAttachment::new(
            1,
            "document.pdf".to_string(),
            "application/pdf".to_string(),
            1024,
            "https://example.com/document.pdf".to_string(),
        );

        assert!(allowed_attachment.is_allowed_type());

        let blocked_attachment = MessageAttachment::new(
            1,
            "malware.exe".to_string(),
            "application/x-executable".to_string(),
            1024,
            "https://example.com/malware.exe".to_string(),
        );

        assert!(!blocked_attachment.is_allowed_type());
    }
}