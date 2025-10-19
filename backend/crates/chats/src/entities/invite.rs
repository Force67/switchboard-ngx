use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a chat invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInvite {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// Chat ID this invitation is for
    pub chat_id: i64,
    /// User ID who created the invitation
    pub inviter_user_id: i64,
    /// User ID who is invited (optional for open invitations)
    pub invited_user_id: Option<i64>,
    /// Email address of invited user (optional)
    pub invited_email: Option<String>,
    /// Invitation status
    pub status: InviteStatus,
    /// Role the invited user will have
    pub role: String,
    /// Personal message from inviter
    pub message: Option<String>,
    /// When the invitation expires
    pub expires_at: String,
    /// Creation timestamp
    pub created_at: String,
    /// When invitation was responded to
    pub responded_at: Option<String>,
}

/// Invitation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InviteStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Cancelled,
}

impl From<&str> for InviteStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "accepted" => InviteStatus::Accepted,
            "declined" => InviteStatus::Declined,
            "expired" => InviteStatus::Expired,
            "cancelled" => InviteStatus::Cancelled,
            _ => InviteStatus::Pending,
        }
    }
}

impl From<InviteStatus> for String {
    fn from(status: InviteStatus) -> Self {
        match status {
            InviteStatus::Pending => "pending".to_string(),
            InviteStatus::Accepted => "accepted".to_string(),
            InviteStatus::Declined => "declined".to_string(),
            InviteStatus::Expired => "expired".to_string(),
            InviteStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

/// Request to create a new invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInviteRequest {
    /// Email address of user to invite
    pub email: String,
    /// Role for the invited user
    pub role: String,
    /// Personal message (optional)
    pub message: Option<String>,
    /// Expiration in hours (default 24 hours)
    pub expires_in_hours: Option<u32>,
}

/// Invite with additional details for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteWithDetails {
    pub id: i64,
    pub public_id: String,
    pub chat_id: i64,
    pub chat_title: String,
    pub inviter_user_id: i64,
    pub inviter_name: Option<String>,
    pub inviter_email: Option<String>,
    pub invited_user_id: Option<i64>,
    pub invited_email: Option<String>,
    pub status: InviteStatus,
    pub role: String,
    pub message: Option<String>,
    pub expires_at: String,
    pub created_at: String,
    pub responded_at: Option<String>,
}

impl ChatInvite {
    /// Create a new invitation instance
    pub fn new(
        chat_id: i64,
        inviter_user_id: i64,
        invited_user_id: Option<i64>,
        invited_email: Option<String>,
        role: String,
        message: Option<String>,
        expires_in_hours: u32,
    ) -> Self {
        let expires_at = Utc::now()
            .checked_add_signed(chrono::Duration::hours(expires_in_hours as i64))
            .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(24))
            .to_rfc3339();

        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            chat_id,
            inviter_user_id,
            invited_user_id,
            invited_email,
            status: InviteStatus::Pending,
            role,
            message,
            expires_at,
            created_at: Utc::now().to_rfc3339(),
            responded_at: None,
        }
    }

    /// Check if the invitation is still valid
    pub fn is_valid(&self) -> bool {
        if self.status != InviteStatus::Pending {
            return false;
        }

        // Check expiration
        if let Ok(expires) = DateTime::parse_from_rfc3339(&self.expires_at) {
            Utc::now() < expires.with_timezone(&Utc)
        } else {
            false
        }
    }

    /// Check if the invitation has expired
    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = DateTime::parse_from_rfc3339(&self.expires_at) {
            Utc::now() >= expires.with_timezone(&Utc)
        } else {
            true // Treat invalid dates as expired
        }
    }

    /// Accept the invitation
    pub fn accept(&mut self) -> Result<(), String> {
        if self.status != InviteStatus::Pending {
            return Err("Invitation is not pending".to_string());
        }

        if self.is_expired() {
            return Err("Invitation has expired".to_string());
        }

        self.status = InviteStatus::Accepted;
        self.responded_at = Some(Utc::now().to_rfc3339());
        Ok(())
    }

    /// Decline the invitation
    pub fn decline(&mut self) -> Result<(), String> {
        if self.status != InviteStatus::Pending {
            return Err("Invitation is not pending".to_string());
        }

        if self.is_expired() {
            return Err("Invitation has expired".to_string());
        }

        self.status = InviteStatus::Declined;
        self.responded_at = Some(Utc::now().to_rfc3339());
        Ok(())
    }

    /// Cancel the invitation
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.status != InviteStatus::Pending {
            return Err("Invitation is not pending".to_string());
        }

        self.status = InviteStatus::Cancelled;
        self.responded_at = Some(Utc::now().to_rfc3339());
        Ok(())
    }

    /// Mark as expired
    pub fn expire(&mut self) -> Result<(), String> {
        if self.status != InviteStatus::Pending {
            return Err("Invitation is not pending".to_string());
        }

        self.status = InviteStatus::Expired;
        self.responded_at = Some(Utc::now().to_rfc3339());
        Ok(())
    }

    /// Check if this is a user-specific invitation
    pub fn is_user_specific(&self) -> bool {
        self.invited_user_id.is_some() || self.invited_email.is_some()
    }

    /// Check if a user can respond to this invitation
    pub fn can_user_respond(&self, user_id: i64, user_email: Option<&str>) -> bool {
        if self.status != InviteStatus::Pending || self.is_expired() {
            return false;
        }

        // Check by user ID
        if let Some(invited_user_id) = self.invited_user_id {
            return invited_user_id == user_id;
        }

        // Check by email
        if let (Some(invited_email), Some(user_email)) = (&self.invited_email, user_email) {
            return invited_email.to_lowercase() == user_email.to_lowercase();
        }

        // If no specific user or email is set, allow anyone to respond (open invitation)
        true
    }

    /// Validate invitation data
    pub fn validate(&self) -> Result<(), String> {
        if self.chat_id <= 0 {
            return Err("Invalid chat ID".to_string());
        }

        if self.inviter_user_id <= 0 {
            return Err("Invalid inviter user ID".to_string());
        }

        if self.role.trim().is_empty() {
            return Err("Role cannot be empty".to_string());
        }

        // Validate timestamps
        if let Err(_) = DateTime::parse_from_rfc3339(&self.created_at) {
            return Err("Invalid created_at timestamp format".to_string());
        }

        if let Err(_) = DateTime::parse_from_rfc3339(&self.expires_at) {
            return Err("Invalid expires_at timestamp format".to_string());
        }

        if let Some(ref responded_at) = self.responded_at {
            if let Err(_) = DateTime::parse_from_rfc3339(responded_at) {
                return Err("Invalid responded_at timestamp format".to_string());
            }
        }

        // Validate email format if present
        if let Some(ref email) = self.invited_email {
            if !email.contains('@') || !email.contains('.') {
                return Err("Invalid email format".to_string());
            }
        }

        Ok(())
    }
}

impl CreateInviteRequest {
    /// Validate the create request
    pub fn validate(&self) -> Result<(), String> {
        if self.email.trim().is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        // Basic email validation
        if !self.email.contains('@') || !self.email.contains('.') {
            return Err("Invalid email format".to_string());
        }

        if self.email.len() > 255 {
            return Err("Email too long (max 255 characters)".to_string());
        }

        if self.role.trim().is_empty() {
            return Err("Role cannot be empty".to_string());
        }

        // Validate message length if present
        if let Some(ref message) = self.message {
            if message.len() > 1000 {
                return Err("Message too long (max 1000 characters)".to_string());
            }
        }

        // Validate expiration
        if let Some(hours) = self.expires_in_hours {
            if hours == 0 || hours > 8760 { // Max 1 year
                return Err("Expiration must be between 1 and 8760 hours".to_string());
            }
        }

        Ok(())
    }

    /// Get default expiration hours
    pub fn default_expiration_hours() -> u32 {
        24
    }
}

impl InviteStatus {
    /// Get all possible statuses
    pub fn all() -> Vec<InviteStatus> {
        vec![
            InviteStatus::Pending,
            InviteStatus::Accepted,
            InviteStatus::Declined,
            InviteStatus::Expired,
            InviteStatus::Cancelled,
        ]
    }

    /// Check if status is final (cannot be changed)
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            InviteStatus::Accepted | InviteStatus::Declined | InviteStatus::Expired | InviteStatus::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_creation() {
        let invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            Some("Join our chat!".to_string()),
            24,
        );

        assert_eq!(invite.chat_id, 1);
        assert_eq!(invite.inviter_user_id, 1);
        assert_eq!(invite.invited_user_id, Some(2));
        assert_eq!(invite.invited_email, Some("user@example.com".to_string()));
        assert_eq!(invite.status, InviteStatus::Pending);
        assert!(invite.is_valid());
        assert!(!invite.is_expired());
        assert!(invite.is_user_specific());
    }

    #[test]
    fn test_invite_status_conversion() {
        assert_eq!(InviteStatus::from("pending"), InviteStatus::Pending);
        assert_eq!(InviteStatus::from("accepted"), InviteStatus::Accepted);
        assert_eq!(InviteStatus::from("declined"), InviteStatus::Declined);
        assert_eq!(InviteStatus::from("expired"), InviteStatus::Expired);
        assert_eq!(InviteStatus::from("cancelled"), InviteStatus::Cancelled);
        assert_eq!(InviteStatus::from("unknown"), InviteStatus::Pending);

        assert_eq!(String::from(InviteStatus::Pending), "pending");
        assert_eq!(String::from(InviteStatus::Accepted), "accepted");
        assert_eq!(String::from(InviteStatus::Declined), "declined");
        assert_eq!(String::from(InviteStatus::Expired), "expired");
        assert_eq!(String::from(InviteStatus::Cancelled), "cancelled");
    }

    #[test]
    fn test_invite_acceptance() {
        let mut invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            24,
        );

        assert!(invite.accept().is_ok());
        assert_eq!(invite.status, InviteStatus::Accepted);
        assert!(invite.responded_at.is_some());

        // Can't accept again
        assert!(invite.accept().is_err());
    }

    #[test]
    fn test_invite_decline() {
        let mut invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            24,
        );

        assert!(invite.decline().is_ok());
        assert_eq!(invite.status, InviteStatus::Declined);
        assert!(invite.responded_at.is_some());

        // Can't decline again
        assert!(invite.decline().is_err());
    }

    #[test]
    fn test_invite_cancel() {
        let mut invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            24,
        );

        assert!(invite.cancel().is_ok());
        assert_eq!(invite.status, InviteStatus::Cancelled);
        assert!(invite.responded_at.is_some());
    }

    #[test]
    fn test_expired_invite() {
        let mut invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            0, // Expires immediately
        );

        // Wait a moment to ensure it's expired
        std::thread::sleep(std::time::Duration::from_millis(10));

        assert!(!invite.is_valid());
        assert!(invite.is_expired());
        assert!(invite.accept().is_err());
        assert!(invite.decline().is_err());

        assert!(invite.expire().is_ok());
        assert_eq!(invite.status, InviteStatus::Expired);
    }

    #[test]
    fn test_can_user_respond() {
        let invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            24,
        );

        // User can respond by ID
        assert!(invite.can_user_respond(2, Some("different@email.com")));
        assert!(!invite.can_user_respond(3, Some("user@example.com")));

        // User can respond by email
        assert!(invite.can_user_respond(2, Some("user@example.com")));
        assert!(invite.can_user_respond(2, Some("USER@EXAMPLE.COM"))); // Case insensitive
        assert!(!invite.can_user_respond(999, Some("different@email.com")));
    }

    #[test]
    fn test_create_invite_request_validation() {
        let valid_request = CreateInviteRequest {
            email: "user@example.com".to_string(),
            role: "member".to_string(),
            message: Some("Join us!".to_string()),
            expires_in_hours: Some(24),
        };
        assert!(valid_request.validate().is_ok());

        let invalid_email = CreateInviteRequest {
            email: "invalid-email".to_string(),
            role: "member".to_string(),
            message: None,
            expires_in_hours: None,
        };
        assert!(invalid_email.validate().is_err());

        let invalid_expiration = CreateInviteRequest {
            email: "user@example.com".to_string(),
            role: "member".to_string(),
            message: None,
            expires_in_hours: Some(0),
        };
        assert!(invalid_expiration.validate().is_err());
    }

    #[test]
    fn test_invite_status_is_final() {
        assert!(!InviteStatus::Pending.is_final());
        assert!(InviteStatus::Accepted.is_final());
        assert!(InviteStatus::Declined.is_final());
        assert!(InviteStatus::Expired.is_final());
        assert!(InviteStatus::Cancelled.is_final());
    }

    #[test]
    fn test_invite_validation() {
        let mut invite = ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            None,
            24,
        );

        assert!(invite.validate().is_ok());

        invite.chat_id = -1;
        assert!(invite.validate().is_err());

        invite.chat_id = 1;
        invite.invited_email = Some("invalid-email".to_string());
        assert!(invite.validate().is_err());
    }
}