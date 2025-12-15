//! Integration tests for the chats crate.

#[cfg(test)]
mod tests {
    use switchboard_chats::{
        utils::{MemberAction, PermissionChecker, Validator},
        ChatError, ChatMember, InviteStatus, MemberRole, MessageStatus,
    };
    use switchboard_database::MessageType;

    fn member(chat_id: i64, user_id: i64, role: MemberRole) -> ChatMember {
        ChatMember {
            id: user_id,
            public_id: format!("member-{user_id}"),
            chat_id,
            chat_public_id: format!("chat-{chat_id}"),
            user_id,
            user_public_id: format!("user-{user_id}"),
            role,
            joined_at: "2024-01-01T00:00:00Z".to_string(),
            user_display_name: None,
            user_avatar_url: None,
            user_email: None,
        }
    }

    #[test]
    fn test_validation_and_types() {
        assert!(Validator::email("user@example.com").is_ok());
        assert!(Validator::chat_title("Test Chat").is_ok());
        assert!(Validator::uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());

        // Enum conversions should stay stable
        assert_eq!(MessageType::from("image"), MessageType::Image);
        assert_eq!(MessageStatus::from("read"), MessageStatus::Read);
        assert_eq!(InviteStatus::from("expired"), InviteStatus::Expired);
    }

    #[test]
    fn test_error_variants() {
        let error = ChatError::ChatNotFound;
        assert!(matches!(error, ChatError::ChatNotFound));

        let db_error = ChatError::DatabaseError("boom".into());
        assert!(matches!(db_error, ChatError::DatabaseError(_)));
    }

    #[test]
    fn test_permission_system() {
        let owner = member(1, 1, MemberRole::Owner);
        let admin = member(1, 2, MemberRole::Admin);
        let regular = member(1, 3, MemberRole::Member);

        // Delete
        assert!(PermissionChecker::can_delete_chat(&owner).is_ok());
        assert!(PermissionChecker::can_delete_chat(&admin).is_err());
        assert!(PermissionChecker::can_delete_chat(&regular).is_err());

        // Manage members
        assert!(PermissionChecker::can_manage_members(&owner).is_ok());
        assert!(PermissionChecker::can_manage_members(&admin).is_ok());
        assert!(PermissionChecker::can_manage_members(&regular).is_err());

        // Manage a specific member
        assert!(
            PermissionChecker::can_manage_member(&owner, &regular, MemberAction::Remove).is_ok()
        );
        assert!(
            PermissionChecker::can_manage_member(&admin, &regular, MemberAction::UpdateRole)
                .is_ok()
        );
        assert!(
            PermissionChecker::can_manage_member(&admin, &owner, MemberAction::Remove).is_err()
        );
    }
}
