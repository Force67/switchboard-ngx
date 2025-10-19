//! Integration tests for the chats crate.

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_crate_basic_functionality() {
        // Basic test to ensure the crate compiles and loads correctly
        // This will be expanded as we implement the actual functionality

        // Test entity creation
        let chat = switchboard_chats::Chat::new(
            "Test Chat".to_string(),
            switchboard_chats::ChatType::Direct,
            Some(1),
            None,
        );

        assert_eq!(chat.title, "Test Chat");
        assert_eq!(chat.chat_type, switchboard_chats::ChatType::Direct);

        // Test validation
        assert!(chat.validate().is_ok());

        // Test message creation
        let message = switchboard_chats::ChatMessage::new(
            1,
            1,
            "Hello, world!".to_string(),
            switchboard_chats::entities::MessageRole::User,
            None,
        );

        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.role, switchboard_chats::entities::MessageRole::User);

        // Test attachment creation
        let attachment = switchboard_chats::MessageAttachment::new(
            1,
            "test.jpg".to_string(),
            "image/jpeg".to_string(),
            1024,
            "https://example.com/test.jpg".to_string(),
        );

        assert_eq!(attachment.file_name, "test.jpg");
        assert!(attachment.is_image());

        // Test member creation
        let member = switchboard_chats::ChatMember::new(
            1,
            1,
            switchboard_chats::entities::MemberRole::Owner,
        );

        assert!(member.is_owner());
        assert!(member.can_delete_chat());

        // Test invite creation
        let invite = switchboard_chats::ChatInvite::new(
            1,
            1,
            Some(2),
            Some("user@example.com".to_string()),
            "member".to_string(),
            Some("Join our chat!".to_string()),
            24,
        );

        assert!(invite.is_valid());
        assert!(invite.is_user_specific());
    }

    #[tokio::test]
    async fn test_error_handling() {
        use switchboard_chats::{ChatError, utils::Validator};

        // Test validation errors
        assert!(Validator::email("invalid-email").is_err());
        assert!(Validator::chat_title("").is_err());
        assert!(Validator::uuid("invalid-uuid").is_err());

        // Test error creation
        let error = ChatError::chat_not_found("test-id");
        assert!(matches!(error, ChatError::ChatNotFound { .. }));

        let validation_error = ChatError::validation("Test validation error");
        assert!(matches!(validation_error, ChatError::Validation { .. }));
    }

    #[tokio::test]
    async fn test_permission_system() {
        use switchboard_chats::utils::{PermissionChecker, MemberAction};

        let owner = switchboard_chats::ChatMember::new(
            1,
            1,
            switchboard_chats::entities::MemberRole::Owner,
        );

        let member = switchboard_chats::ChatMember::new(
            1,
            2,
            switchboard_chats::entities::MemberRole::Member,
        );

        // Test permission checks
        assert!(PermissionChecker::can_delete_chat(&owner).is_ok());
        assert!(PermissionChecker::can_delete_chat(&member).is_err());

        assert!(PermissionChecker::can_manage_members(&owner).is_ok());
        assert!(PermissionChecker::can_manage_members(&member).is_err());

        // Test member management permissions
        assert!(PermissionChecker::can_manage_member(&owner, &member, MemberAction::Remove).is_ok());
        assert!(PermissionChecker::can_manage_member(&member, &owner, MemberAction::Remove).is_err());
    }
}