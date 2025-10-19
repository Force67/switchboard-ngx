//! Event types for real-time chat updates.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use switchboard_database::{Chat, ChatMessage, MessageAttachment, ChatMember, ChatInvite};

/// Main chat event type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ChatEvent {
    /// Chat was created
    ChatCreated {
        chat_id: String,
        chat: Chat,
    },

    /// Chat was updated
    ChatUpdated {
        chat_id: String,
        chat: Chat,
        member_ids: Vec<i64>,
    },

    /// Chat was deleted
    ChatDeleted {
        chat_id: String,
        member_ids: Vec<i64>,
    },

    /// Message was created
    MessageCreated {
        chat_id: String,
        message: ChatMessage,
    },

    /// Message was updated
    MessageUpdated {
        chat_id: String,
        message: ChatMessage,
    },

    /// Message was deleted
    MessageDeleted {
        chat_id: String,
        message_id: String,
        user_id: i64,
    },

    /// Attachment was created
    AttachmentCreated {
        chat_id: String,
        message_id: String,
        attachment: MessageAttachment,
    },

    /// Attachment was deleted
    AttachmentDeleted {
        chat_id: String,
        message_id: String,
        attachment_id: String,
        user_id: i64,
    },

    /// Member was added to chat
    MemberAdded {
        chat_id: String,
        member: ChatMember,
    },

    /// Member role was updated
    MemberUpdated {
        chat_id: String,
        member: ChatMember,
    },

    /// Member was removed from chat
    MemberRemoved {
        chat_id: String,
        member_user_id: i64,
        removed_by_user_id: i64,
    },

    /// Invitation was created
    InviteCreated {
        chat_id: String,
        invite: ChatInvite,
    },

    /// Invitation was accepted
    InviteAccepted {
        chat_id: String,
        invite: ChatInvite,
        member: ChatMember,
    },

    /// Invitation was declined
    InviteDeclined {
        chat_id: String,
        invite: ChatInvite,
    },

    /// User is typing
    UserTyping {
        chat_id: String,
        user_id: i64,
    },

    /// User stopped typing
    UserStoppedTyping {
        chat_id: String,
        user_id: i64,
    },

    /// User came online
    UserOnline {
        user_id: i64,
    },

    /// User went offline
    UserOffline {
        user_id: i64,
    },
}

impl ChatEvent {
    /// Get the chat ID associated with this event
    pub fn chat_id(&self) -> Option<&str> {
        match self {
            ChatEvent::ChatCreated { chat_id, .. }
            | ChatEvent::ChatUpdated { chat_id, .. }
            | ChatEvent::ChatDeleted { chat_id, .. }
            | ChatEvent::MessageCreated { chat_id, .. }
            | ChatEvent::MessageUpdated { chat_id, .. }
            | ChatEvent::MessageDeleted { chat_id, .. }
            | ChatEvent::AttachmentCreated { chat_id, .. }
            | ChatEvent::AttachmentDeleted { chat_id, .. }
            | ChatEvent::MemberAdded { chat_id, .. }
            | ChatEvent::MemberUpdated { chat_id, .. }
            | ChatEvent::MemberRemoved { chat_id, .. }
            | ChatEvent::InviteCreated { chat_id, .. }
            | ChatEvent::InviteAccepted { chat_id, .. }
            | ChatEvent::InviteDeclined { chat_id, .. }
            | ChatEvent::UserTyping { chat_id, .. }
            | ChatEvent::UserStoppedTyping { chat_id, .. } => Some(chat_id),
            ChatEvent::UserOnline { .. } | ChatEvent::UserOffline { .. } => None,
        }
    }

    /// Get the user IDs that should receive this event
    pub fn target_users(&self) -> Vec<i64> {
        match self {
            ChatEvent::ChatCreated { chat, .. } => {
                // Parse created_by from string to i64, default to 0 if parsing fails
                vec![chat.created_by.parse().unwrap_or(0)]
            },
            ChatEvent::ChatUpdated { member_ids, .. } => member_ids.clone(),
            ChatEvent::ChatDeleted { member_ids, .. } => member_ids.clone(),
            ChatEvent::MessageCreated { message, .. } => vec![message.sender_id],
            ChatEvent::MessageUpdated { message, .. } => vec![message.sender_id],
            ChatEvent::MessageDeleted { user_id, .. } => vec![*user_id],
            ChatEvent::AttachmentCreated { attachment, .. } => vec![], // TODO: Add user_id to attachment
            ChatEvent::AttachmentDeleted { user_id, .. } => vec![*user_id],
            ChatEvent::MemberAdded { member, .. } => vec![member.user_id],
            ChatEvent::MemberUpdated { member, .. } => vec![member.user_id],
            ChatEvent::MemberRemoved { member_user_id, .. } => vec![*member_user_id],
            ChatEvent::InviteCreated { .. } => vec![], // Specific to invited user
            ChatEvent::InviteAccepted { member, .. } => vec![member.user_id],
            ChatEvent::InviteDeclined { .. } => vec![], // Specific to inviter
            ChatEvent::UserTyping { user_id, .. } => vec![*user_id],
            ChatEvent::UserStoppedTyping { user_id, .. } => vec![*user_id],
            ChatEvent::UserOnline { user_id, .. } => vec![*user_id],
            ChatEvent::UserOffline { user_id, .. } => vec![*user_id],
        }
    }

    /// Check if this is a real-time event (should be broadcast immediately)
    pub fn is_realtime(&self) -> bool {
        matches!(
            self,
            ChatEvent::MessageCreated { .. }
                | ChatEvent::MessageUpdated { .. }
                | ChatEvent::MessageDeleted { .. }
                | ChatEvent::AttachmentCreated { .. }
                | ChatEvent::AttachmentDeleted { .. }
                | ChatEvent::MemberAdded { .. }
                | ChatEvent::MemberUpdated { .. }
                | ChatEvent::MemberRemoved { .. }
                | ChatEvent::UserTyping { .. }
                | ChatEvent::UserStoppedTyping { .. }
                | ChatEvent::UserOnline { .. }
                | ChatEvent::UserOffline { .. }
        )
    }

    /// Get event type name for logging/metrics
    pub fn event_type_name(&self) -> &'static str {
        match self {
            ChatEvent::ChatCreated { .. } => "chat_created",
            ChatEvent::ChatUpdated { .. } => "chat_updated",
            ChatEvent::ChatDeleted { .. } => "chat_deleted",
            ChatEvent::MessageCreated { .. } => "message_created",
            ChatEvent::MessageUpdated { .. } => "message_updated",
            ChatEvent::MessageDeleted { .. } => "message_deleted",
            ChatEvent::AttachmentCreated { .. } => "attachment_created",
            ChatEvent::AttachmentDeleted { .. } => "attachment_deleted",
            ChatEvent::MemberAdded { .. } => "member_added",
            ChatEvent::MemberUpdated { .. } => "member_updated",
            ChatEvent::MemberRemoved { .. } => "member_removed",
            ChatEvent::InviteCreated { .. } => "invite_created",
            ChatEvent::InviteAccepted { .. } => "invite_accepted",
            ChatEvent::InviteDeclined { .. } => "invite_declined",
            ChatEvent::UserTyping { .. } => "user_typing",
            ChatEvent::UserStoppedTyping { .. } => "user_stopped_typing",
            ChatEvent::UserOnline { .. } => "user_online",
            ChatEvent::UserOffline { .. } => "user_offline",
        }
    }
}

/// Event metadata for tracking and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event ID
    pub event_id: String,
    /// Timestamp when event was created
    pub timestamp: String,
    /// User ID who triggered the event
    pub user_id: Option<i64>,
    /// Event type
    pub event_type: String,
    /// Additional context
    pub context: std::collections::HashMap<String, String>,
}

impl EventMetadata {
    /// Create new event metadata
    pub fn new(event_type: impl Into<String>, user_id: Option<i64>) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            user_id,
            event_type: event_type.into(),
            context: std::collections::HashMap::new(),
        }

    }

    /// Add context information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}