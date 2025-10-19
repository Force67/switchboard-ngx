//! Event types for the user management system.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::entities::user::{UserRole, UserStatus};
use crate::entities::notification::NotificationType;

/// User-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UserEvent {
    /// User account was created
    UserCreated {
        user_id: i64,
        public_id: String,
        username: String,
        email: String,
        role: UserRole,
        timestamp: DateTime<Utc>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    },

    /// User profile was updated
    UserUpdated {
        user_id: i64,
        public_id: String,
        changed_fields: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// User account was deleted
    UserDeleted {
        user_id: i64,
        public_id: String,
        username: String,
        deleted_by: i64,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// User logged in
    UserLoggedIn {
        user_id: i64,
        public_id: String,
        session_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// User logged out
    UserLoggedOut {
        user_id: i64,
        public_id: String,
        session_id: String,
        timestamp: DateTime<Utc>,
    },

    /// User password was changed
    PasswordChanged {
        user_id: i64,
        public_id: String,
        changed_by: i64, // Can be self or admin
        timestamp: DateTime<Utc>,
    },

    /// User role was changed
    RoleChanged {
        user_id: i64,
        public_id: String,
        old_role: UserRole,
        new_role: UserRole,
        changed_by: i64,
        timestamp: DateTime<Utc>,
    },

    /// User status was changed
    StatusChanged {
        user_id: i64,
        public_id: String,
        old_status: UserStatus,
        new_status: UserStatus,
        changed_by: i64,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// User account was verified
    AccountVerified {
        user_id: i64,
        public_id: String,
        verification_method: String,
        timestamp: DateTime<Utc>,
    },

    /// Two-factor authentication was enabled
    TwoFactorEnabled {
        user_id: i64,
        public_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Two-factor authentication was disabled
    TwoFactorDisabled {
        user_id: i64,
        public_id: String,
        disabled_by: i64,
        timestamp: DateTime<Utc>,
    },

    /// Failed login attempt
    LoginFailed {
        email: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        reason: String,
        timestamp: DateTime<Utc>,
    },

    /// Suspicious activity detected
    SuspiciousActivity {
        user_id: Option<i64>,
        public_id: Option<String>,
        activity_type: String,
        details: serde_json::Value,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// Password reset requested
    PasswordResetRequested {
        user_id: i64,
        public_id: String,
        email: String,
        reset_token: String,
        expires_at: DateTime<Utc>,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// Password was reset
    PasswordResetCompleted {
        user_id: i64,
        public_id: String,
        timestamp: DateTime<Utc>,
    },
}

/// Authentication-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthEvent {
    /// New session created
    SessionCreated {
        user_id: i64,
        session_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        expires_at: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },

    /// Session was validated
    SessionValidated {
        user_id: i64,
        session_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Session expired
    SessionExpired {
        user_id: i64,
        session_id: String,
        timestamp: DateTime<Utc>,
    },

    /// Session was revoked
    SessionRevoked {
        user_id: i64,
        session_id: String,
        revoked_by: i64, // Can be self or admin
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// All sessions for user were revoked
    AllSessionsRevoked {
        user_id: i64,
        revoked_by: i64,
        reason: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// OAuth authentication initiated
    OAuthStarted {
        provider: String,
        state: String,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },

    /// OAuth authentication completed
    OAuthCompleted {
        user_id: i64,
        provider: String,
        email: String,
        timestamp: DateTime<Utc>,
    },

    /// OAuth authentication failed
    OAuthFailed {
        provider: String,
        error: String,
        ip_address: Option<String>,
        timestamp: DateTime<Utc>,
    },
}

/// Notification-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotificationEvent {
    /// Notification was created
    NotificationCreated {
        notification_id: i64,
        user_id: i64,
        notification_type: NotificationType,
        title: String,
        timestamp: DateTime<Utc>,
    },

    /// Notification was read
    NotificationRead {
        notification_id: i64,
        user_id: i64,
        timestamp: DateTime<Utc>,
    },

    /// Notification was deleted
    NotificationDeleted {
        notification_id: i64,
        user_id: i64,
        deleted_by: i64,
        timestamp: DateTime<Utc>,
    },

    /// Notification preferences updated
    PreferencesUpdated {
        user_id: i64,
        updated_fields: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Notification delivery failed
    DeliveryFailed {
        notification_id: i64,
        user_id: i64,
        error: String,
        retry_count: u32,
        timestamp: DateTime<Utc>,
    },

    /// Batch notification created
    BatchNotificationCreated {
        user_ids: Vec<i64>,
        notification_type: NotificationType,
        title: String,
        batch_id: String,
        timestamp: DateTime<Utc>,
    },
}

/// Settings-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SettingsEvent {
    /// User settings were updated
    SettingsUpdated {
        user_id: i64,
        changed_fields: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    /// Settings were reset to defaults
    SettingsReset {
        user_id: i64,
        reset_by: i64,
        timestamp: DateTime<Utc>,
    },

    /// Settings were exported
    SettingsExported {
        user_id: i64,
        export_format: String,
        exported_by: i64,
        timestamp: DateTime<Utc>,
    },

    /// Settings were imported
    SettingsImported {
        user_id: i64,
        import_format: String,
        imported_fields: Vec<String>,
        imported_by: i64,
        timestamp: DateTime<Utc>,
    },
}

/// System-wide events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemEvent {
    /// User statistics updated
    UserStatsUpdated {
        total_users: i64,
        active_users: i64,
        new_users_today: i64,
        timestamp: DateTime<Utc>,
    },

    /// Security audit event
    SecurityAudit {
        event_type: String,
        user_id: Option<i64>,
        details: serde_json::Value,
        severity: String,
        timestamp: DateTime<Utc>,
    },

    /// Maintenance started
    MaintenanceStarted {
        reason: String,
        estimated_duration: Option<u32>, // minutes
        timestamp: DateTime<Utc>,
    },

    /// Maintenance completed
    MaintenanceCompleted {
        reason: String,
        actual_duration: u32, // minutes
        timestamp: DateTime<Utc>,
    },
}

/// Combined event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "category")]
pub enum Event {
    User(UserEvent),
    Auth(AuthEvent),
    Notification(NotificationEvent),
    Settings(SettingsEvent),
    System(SystemEvent),
}

impl Event {
    /// Get the event timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Event::User(event) => match event {
                UserEvent::UserCreated { timestamp, .. }
                | UserEvent::UserUpdated { timestamp, .. }
                | UserEvent::UserDeleted { timestamp, .. }
                | UserEvent::UserLoggedIn { timestamp, .. }
                | UserEvent::UserLoggedOut { timestamp, .. }
                | UserEvent::PasswordChanged { timestamp, .. }
                | UserEvent::RoleChanged { timestamp, .. }
                | UserEvent::StatusChanged { timestamp, .. }
                | UserEvent::AccountVerified { timestamp, .. }
                | UserEvent::TwoFactorEnabled { timestamp, .. }
                | UserEvent::TwoFactorDisabled { timestamp, .. }
                | UserEvent::LoginFailed { timestamp, .. }
                | UserEvent::SuspiciousActivity { timestamp, .. }
                | UserEvent::PasswordResetRequested { timestamp, .. }
                | UserEvent::PasswordResetCompleted { timestamp, .. } => *timestamp,
            },
            Event::Auth(event) => match event {
                AuthEvent::SessionCreated { timestamp, .. }
                | AuthEvent::SessionValidated { timestamp, .. }
                | AuthEvent::SessionExpired { timestamp, .. }
                | AuthEvent::SessionRevoked { timestamp, .. }
                | AuthEvent::AllSessionsRevoked { timestamp, .. }
                | AuthEvent::OAuthStarted { timestamp, .. }
                | AuthEvent::OAuthCompleted { timestamp, .. }
                | AuthEvent::OAuthFailed { timestamp, .. } => *timestamp,
            },
            Event::Notification(event) => match event {
                NotificationEvent::NotificationCreated { timestamp, .. }
                | NotificationEvent::NotificationRead { timestamp, .. }
                | NotificationEvent::NotificationDeleted { timestamp, .. }
                | NotificationEvent::PreferencesUpdated { timestamp, .. }
                | NotificationEvent::DeliveryFailed { timestamp, .. }
                | NotificationEvent::BatchNotificationCreated { timestamp, .. } => *timestamp,
            },
            Event::Settings(event) => match event {
                SettingsEvent::SettingsUpdated { timestamp, .. }
                | SettingsEvent::SettingsReset { timestamp, .. }
                | SettingsEvent::SettingsExported { timestamp, .. }
                | SettingsEvent::SettingsImported { timestamp, .. } => *timestamp,
            },
            Event::System(event) => match event {
                SystemEvent::UserStatsUpdated { timestamp, .. }
                | SystemEvent::SecurityAudit { timestamp, .. }
                | SystemEvent::MaintenanceStarted { timestamp, .. }
                | SystemEvent::MaintenanceCompleted { timestamp, .. } => *timestamp,
            },
        }
    }

    /// Get the event category name
    pub fn category_name(&self) -> &'static str {
        match self {
            Event::User(_) => "user",
            Event::Auth(_) => "auth",
            Event::Notification(_) => "notification",
            Event::Settings(_) => "settings",
            Event::System(_) => "system",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize_deserialize_events() {
        let event = Event::User(UserEvent::UserCreated {
            user_id: 1,
            public_id: "abc123".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::User,
            timestamp: Utc::now(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        });

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match (event, deserialized) {
            (Event::User(original), Event::User(deserialized)) => {
                match (original, deserialized) {
                    (
                        UserEvent::UserCreated { user_id: orig_id, .. },
                        UserEvent::UserCreated { user_id: de_id, .. },
                    ) => {
                        assert_eq!(orig_id, de_id);
                    }
                    _ => panic!("Event types don't match"),
                }
            }
            _ => panic!("Event categories don't match"),
        }
    }

    #[test]
    fn test_event_timestamp() {
        let timestamp = Utc::now();
        let event = Event::Auth(AuthEvent::SessionCreated {
            user_id: 1,
            session_id: "session123".to_string(),
            ip_address: None,
            user_agent: None,
            expires_at: timestamp,
            timestamp,
        });

        assert_eq!(event.timestamp(), timestamp);
        assert_eq!(event.category_name(), "auth");
    }
}