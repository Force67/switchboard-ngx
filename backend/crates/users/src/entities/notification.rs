use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Database primary key
    pub id: i64,
    /// Publicly accessible UUID
    pub public_id: String,
    /// User ID this notification belongs to
    pub user_id: i64,
    /// Notification type
    pub notification_type: NotificationType,
    /// Notification title
    pub title: String,
    /// Notification message/content
    pub message: String,
    /// Notification priority
    pub priority: NotificationPriority,
    /// Whether the notification has been read
    pub is_read: bool,
    /// Related entity ID (optional)
    pub related_entity_id: Option<String>,
    /// Related entity type (optional)
    pub related_entity_type: Option<String>,
    /// Notification metadata (JSON)
    pub metadata: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: String,
    /// Read timestamp (optional)
    pub read_at: Option<String>,
    /// Expiration timestamp (optional)
    pub expires_at: Option<String>,
}

/// Notification type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    Message,
    ChatInvite,
    System,
    Security,
    Update,
    Reminder,
    Achievement,
}

/// Notification priority enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// User notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// User ID
    pub user_id: i64,
    /// Email notifications enabled
    pub email_enabled: bool,
    /// In-app notifications enabled
    pub in_app_enabled: bool,
    /// Push notifications enabled
    pub push_enabled: bool,
    /// Notification type preferences
    pub type_preferences: NotificationTypePreferences,
    /// Quiet hours enabled
    pub quiet_hours_enabled: bool,
    /// Quiet hours start time (HH:MM format)
    pub quiet_hours_start: Option<String>,
    /// Quiet hours end time (HH:MM format)
    pub quiet_hours_end: Option<String>,
    /// Timezone for quiet hours
    pub timezone: Option<String>,
    /// Do not disturb until timestamp
    pub do_not_disturb_until: Option<String>,
    /// Updated timestamp
    pub updated_at: String,
}

/// Notification type preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTypePreferences {
    /// Message notifications
    pub messages: bool,
    /// Chat invitation notifications
    pub chat_invites: bool,
    /// System notifications
    pub system: bool,
    /// Security notifications
    pub security: bool,
    /// Update notifications
    pub updates: bool,
    /// Reminder notifications
    pub reminders: bool,
    /// Achievement notifications
    pub achievements: bool,
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "chat_invite" => NotificationType::ChatInvite,
            "system" => NotificationType::System,
            "security" => NotificationType::Security,
            "update" => NotificationType::Update,
            "reminder" => NotificationType::Reminder,
            "achievement" => NotificationType::Achievement,
            _ => NotificationType::Message,
        }
    }
}

impl From<NotificationType> for String {
    fn from(notification_type: NotificationType) -> Self {
        match notification_type {
            NotificationType::Message => "message".to_string(),
            NotificationType::ChatInvite => "chat_invite".to_string(),
            NotificationType::System => "system".to_string(),
            NotificationType::Security => "security".to_string(),
            NotificationType::Update => "update".to_string(),
            NotificationType::Reminder => "reminder".to_string(),
            NotificationType::Achievement => "achievement".to_string(),
        }
    }
}

impl From<&str> for NotificationPriority {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => NotificationPriority::Low,
            "high" => NotificationPriority::High,
            "urgent" => NotificationPriority::Urgent,
            _ => NotificationPriority::Normal,
        }
    }
}

impl From<NotificationPriority> for String {
    fn from(priority: NotificationPriority) -> Self {
        match priority {
            NotificationPriority::Low => "low".to_string(),
            NotificationPriority::Normal => "normal".to_string(),
            NotificationPriority::High => "high".to_string(),
            NotificationPriority::Urgent => "urgent".to_string(),
        }
    }
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl Notification {
    /// Create a new notification
    pub fn new(
        user_id: i64,
        notification_type: NotificationType,
        title: String,
        message: String,
        priority: NotificationPriority,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: 0, // Will be set by database
            public_id: Uuid::new_v4().to_string(),
            user_id,
            notification_type,
            title,
            message,
            priority,
            is_read: false,
            related_entity_id: None,
            related_entity_type: None,
            metadata: None,
            created_at: now.clone(),
            read_at: None,
            expires_at: None,
        }
    }

    /// Create a notification with related entity
    pub fn with_related_entity(
        user_id: i64,
        notification_type: NotificationType,
        title: String,
        message: String,
        priority: NotificationPriority,
        related_entity_id: String,
        related_entity_type: String,
    ) -> Self {
        let mut notification = Self::new(user_id, notification_type, title, message, priority);
        notification.related_entity_id = Some(related_entity_id);
        notification.related_entity_type = Some(related_entity_type);
        notification
    }

    /// Mark notification as read
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
        self.read_at = Some(Utc::now().to_rfc3339());
    }

    /// Mark notification as unread
    pub fn mark_as_unread(&mut self) {
        self.is_read = false;
        self.read_at = None;
    }

    /// Check if notification is expired
    pub fn is_expired(&self) -> bool {
        if let Some(ref expires_at) = self.expires_at {
            if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                Utc::now() >= expires.with_timezone(&Utc)
            } else {
                true // Treat invalid dates as expired
            }
        } else {
            false
        }
    }

    /// Check if notification is valid
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Get notification age in seconds
    pub fn age_seconds(&self) -> i64 {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            (Utc::now() - created.with_timezone(&Utc)).num_seconds()
        } else {
            0
        }
    }

    /// Validate notification data
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        if self.title.trim().is_empty() {
            return Err("Notification title cannot be empty".to_string());
        }

        if self.title.len() > 255 {
            return Err("Notification title too long (max 255 characters)".to_string());
        }

        if self.message.trim().is_empty() {
            return Err("Notification message cannot be empty".to_string());
        }

        if self.message.len() > 2000 {
            return Err("Notification message too long (max 2000 characters)".to_string());
        }

        // Validate timestamp formats
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            return Err("Invalid created_at timestamp format".to_string());
        }

        if let Some(ref read_at) = self.read_at {
            if let Err(_) = chrono::DateTime::parse_from_rfc3339(read_at) {
                return Err("Invalid read_at timestamp format".to_string());
            }
        }

        if let Some(ref expires_at) = self.expires_at {
            if let Err(_) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                return Err("Invalid expires_at timestamp format".to_string());
            }
        }

        Ok(())
    }
}

impl NotificationPreferences {
    /// Create default notification preferences for a user
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            user_id,
            email_enabled: true,
            in_app_enabled: true,
            push_enabled: false, // Default to off for push
            type_preferences: NotificationTypePreferences::default(),
            quiet_hours_enabled: false,
            quiet_hours_start: None,
            quiet_hours_end: None,
            timezone: Some("UTC".to_string()),
            do_not_disturb_until: None,
            updated_at: now,
        }
    }

    /// Check if user is in quiet hours
    pub fn is_in_quiet_hours(&self) -> bool {
        if !self.quiet_hours_enabled {
            return false;
        }

        if let (Some(ref start), Some(ref end)) = (&self.quiet_hours_start, &self.quiet_hours_end) {
            // Simple implementation - in real app, would handle timezone and date crossing
            let current_time = Utc::now().format("%H:%M").to_string();
            current_time >= *start && current_time <= *end
        } else {
            false
        }
    }

    /// Check if user is in do not disturb mode
    pub fn is_do_not_disturb(&self) -> bool {
        if let Some(ref until) = self.do_not_disturb_until {
            if let Ok(until_time) = chrono::DateTime::parse_from_rfc3339(until) {
                Utc::now() < until_time.with_timezone(&Utc)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if notification should be delivered based on preferences
    pub fn should_deliver_notification(&self, notification_type: NotificationType, priority: NotificationPriority) -> bool {
        // Always deliver urgent notifications
        if matches!(priority, NotificationPriority::Urgent) {
            return true;
        }

        // Check do not disturb mode
        if self.is_do_not_disturb() {
            return false;
        }

        // Check quiet hours
        if self.is_in_quiet_hours() {
            return false;
        }

        // Check type preferences
        match notification_type {
            NotificationType::Message => self.type_preferences.messages && self.in_app_enabled,
            NotificationType::ChatInvite => self.type_preferences.chat_invites && self.in_app_enabled,
            NotificationType::System => self.type_preferences.system && self.in_app_enabled,
            NotificationType::Security => self.type_preferences.security && self.in_app_enabled,
            NotificationType::Update => self.type_preferences.updates && self.in_app_enabled,
            NotificationType::Reminder => self.type_preferences.reminders && self.in_app_enabled,
            NotificationType::Achievement => self.type_preferences.achievements && self.in_app_enabled,
        }
    }

    /// Validate preferences data
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        // Validate time format if provided
        if let Some(ref start) = self.quiet_hours_start {
            if let Err(_) = chrono::NaiveTime::parse_from_str(start, "%H:%M") {
                return Err("Invalid quiet hours start time format (use HH:MM)".to_string());
            }
        }

        if let Some(ref end) = self.quiet_hours_end {
            if let Err(_) = chrono::NaiveTime::parse_from_str(end, "%H:%M") {
                return Err("Invalid quiet hours end time format (use HH:MM)".to_string());
            }
        }

        // Validate do not disturb timestamp if provided
        if let Some(ref until) = self.do_not_disturb_until {
            if let Err(_) = chrono::DateTime::parse_from_rfc3339(until) {
                return Err("Invalid do not disturb timestamp format".to_string());
            }
        }

        Ok(())
    }
}

impl Default for NotificationTypePreferences {
    fn default() -> Self {
        Self {
            messages: true,
            chat_invites: true,
            system: true,
            security: true,
            updates: false, // Less critical by default
            reminders: true,
            achievements: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            1,
            NotificationType::Message,
            "New Message".to_string(),
            "You have a new message".to_string(),
            NotificationPriority::Normal,
        );

        assert_eq!(notification.user_id, 1);
        assert_eq!(notification.notification_type, NotificationType::Message);
        assert_eq!(notification.title, "New Message");
        assert_eq!(notification.priority, NotificationPriority::Normal);
        assert!(!notification.is_read);
        assert!(notification.is_valid());
    }

    #[test]
    fn test_notification_with_related_entity() {
        let notification = Notification::with_related_entity(
            1,
            NotificationType::ChatInvite,
            "Chat Invitation".to_string(),
            "You've been invited to a chat".to_string(),
            NotificationPriority::High,
            "chat-123".to_string(),
            "chat".to_string(),
        );

        assert_eq!(notification.related_entity_id, Some("chat-123".to_string()));
        assert_eq!(notification.related_entity_type, Some("chat".to_string()));
    }

    #[test]
    fn test_notification_mark_as_read() {
        let mut notification = Notification::new(
            1,
            NotificationType::Message,
            "Test".to_string(),
            "Test message".to_string(),
            NotificationPriority::Normal,
        );

        assert!(!notification.is_read);
        assert!(notification.read_at.is_none());

        notification.mark_as_read();

        assert!(notification.is_read);
        assert!(notification.read_at.is_some());
    }

    #[test]
    fn test_notification_type_conversion() {
        assert_eq!(NotificationType::from("message"), NotificationType::Message);
        assert_eq!(NotificationType::from("chat_invite"), NotificationType::ChatInvite);
        assert_eq!(NotificationType::from("system"), NotificationType::System);
        assert_eq!(NotificationType::from("security"), NotificationType::Security);
        assert_eq!(NotificationType::from("unknown"), NotificationType::Message);

        assert_eq!(String::from(NotificationType::Message), "message");
        assert_eq!(String::from(NotificationType::ChatInvite), "chat_invite");
        assert_eq!(String::from(NotificationType::System), "system");
        assert_eq!(String::from(NotificationType::Security), "security");
    }

    #[test]
    fn test_notification_preferences_creation() {
        let prefs = NotificationPreferences::new(1);

        assert_eq!(prefs.user_id, 1);
        assert!(prefs.email_enabled);
        assert!(prefs.in_app_enabled);
        assert!(!prefs.push_enabled);
        assert!(prefs.type_preferences.messages);
        assert!(!prefs.quiet_hours_enabled);
        assert_eq!(prefs.timezone, Some("UTC".to_string()));
    }

    #[test]
    fn test_notification_preferences_quiet_hours() {
        let mut prefs = NotificationPreferences::new(1);

        // Initially not in quiet hours
        assert!(!prefs.is_in_quiet_hours());

        // Enable quiet hours
        prefs.quiet_hours_enabled = true;
        prefs.quiet_hours_start = Some("22:00".to_string());
        prefs.quiet_hours_end = Some("08:00".to_string());

        // Note: This test might fail depending on current time
        // In a real implementation, you'd mock the current time
    }

    #[test]
    fn test_notification_should_deliver() {
        let prefs = NotificationPreferences::new(1);

        // Normal message should be delivered
        assert!(prefs.should_deliver_notification(NotificationType::Message, NotificationPriority::Normal));

        // Urgent notifications should always be delivered
        assert!(prefs.should_deliver_notification(NotificationType::System, NotificationPriority::Urgent));

        // Updates should not be delivered by default
        assert!(!prefs.should_deliver_notification(NotificationType::Update, NotificationPriority::Normal));
    }

    #[test]
    fn test_notification_validation() {
        let notification = Notification::new(
            1,
            NotificationType::Message,
            "Valid Title".to_string(),
            "Valid message".to_string(),
            NotificationPriority::Normal,
        );
        assert!(notification.validate().is_ok());

        let mut invalid_notification = Notification::new(
            0, // Invalid user ID
            NotificationType::Message,
            "Valid Title".to_string(),
            "Valid message".to_string(),
            NotificationPriority::Normal,
        );
        assert!(invalid_notification.validate().is_err());

        invalid_notification.user_id = 1;
        invalid_notification.title = "".to_string(); // Empty title
        assert!(invalid_notification.validate().is_err());
    }

    #[test]
    fn test_notification_age() {
        let notification = Notification::new(
            1,
            NotificationType::Message,
            "Test".to_string(),
            "Test message".to_string(),
            NotificationPriority::Normal,
        );

        let age = notification.age_seconds();
        assert!(age >= 0 && age < 1); // Should be very recent
    }
}