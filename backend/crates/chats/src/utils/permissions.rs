//! Permission checking utilities.

use switchboard_database::{ChatMember, MemberRole, ChatError};

/// Permission checking utilities
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if a user can access a chat
    pub fn can_access_chat(member: &ChatMember, user_id: i64) -> Result<(), ChatError> {
        if member.user_id != user_id {
            return Err(ChatError::DatabaseError("User is not a member of this chat".to_string()));
        }
        Ok(())
    }

    /// Check if a user can manage members in a chat
    pub fn can_manage_members(member: &ChatMember) -> Result<(), ChatError> {
        match member.role {
            MemberRole::Owner | MemberRole::Admin => Ok(()),
            MemberRole::Member => Err(ChatError::DatabaseError("Only owners and admins can manage members".to_string())),
        }
    }

    /// Check if a user can delete a chat
    pub fn can_delete_chat(member: &ChatMember) -> Result<(), ChatError> {
        match member.role {
            MemberRole::Owner => Ok(()),
            MemberRole::Admin | MemberRole::Member => Err(ChatError::DatabaseError("Only chat owners can delete chats".to_string())),
        }
    }

    /// Check if a user can invite others to a chat
    pub fn can_invite_members(member: &ChatMember) -> Result<(), ChatError> {
        match member.role {
            MemberRole::Owner | MemberRole::Admin => Ok(()),
            MemberRole::Member => Err(ChatError::DatabaseError("Only owners and admins can invite members".to_string())),
        }
    }

    /// Check if a user can update a chat
    pub fn can_update_chat(member: &ChatMember) -> Result<(), ChatError> {
        match member.role {
            MemberRole::Owner | MemberRole::Admin => Ok(()),
            MemberRole::Member => Err(ChatError::DatabaseError("Only owners and admins can update chat settings".to_string())),
        }
    }

    /// Check if a user can perform an action on another member
    pub fn can_manage_member(
        requester: &ChatMember,
        target: &ChatMember,
        action: MemberAction,
    ) -> Result<(), ChatError> {
        // Cannot perform actions on yourself
        if requester.user_id == target.user_id {
            return Err(ChatError::DatabaseError("Cannot perform actions on yourself".to_string()));
        }

        // Check permission based on action
        match action {
            MemberAction::UpdateRole => {
                // Only owners can update roles of admins and owners
                if matches!(target.role, MemberRole::Owner | MemberRole::Admin) {
                    if !matches!(requester.role, MemberRole::Owner) {
                        return Err(ChatError::DatabaseError("Only owners can manage admins and other owners".to_string()));
                    }
                } else {
                    // Admins can update regular members, owners can update anyone
                    if !matches!(requester.role, MemberRole::Owner | MemberRole::Admin) {
                        return Err(ChatError::DatabaseError("Insufficient permissions to update member role".to_string()));
                    }
                }
            }
            MemberAction::Remove => {
                // Cannot remove owners
                if matches!(target.role, MemberRole::Owner) {
                    return Err(ChatError::DatabaseError("Cannot remove chat owner".to_string()));
                }

                // Admins cannot remove other admins
                if matches!(target.role, MemberRole::Admin) && matches!(requester.role, MemberRole::Admin) {
                    return Err(ChatError::DatabaseError("Admins cannot remove other admins".to_string()));
                }

                // Only owners and admins can remove members
                if !matches!(requester.role, MemberRole::Owner | MemberRole::Admin) {
                    return Err(ChatError::DatabaseError("Insufficient permissions to remove member".to_string()));
                }
            }
        }

        Ok(())
    }
}

/// Actions that can be performed on chat members
#[derive(Debug, Clone, PartialEq)]
pub enum MemberAction {
    UpdateRole,
    Remove,
}

#[cfg(test)]
mod tests {
    use super::*;
    use switchboard_database::ChatType;

    #[test]
    fn test_permission_checker_can_access_chat() {
        let member = ChatMember::new(1, 1, MemberRole::Member);

        assert!(PermissionChecker::can_access_chat(&member, 1).is_ok());
        assert!(PermissionChecker::can_access_chat(&member, 2).is_err());
    }

    #[test]
    fn test_permission_checker_can_manage_members() {
        let owner = ChatMember::new(1, 1, MemberRole::Owner);
        let admin = ChatMember::new(1, 2, MemberRole::Admin);
        let member = ChatMember::new(1, 3, MemberRole::Member);

        assert!(PermissionChecker::can_manage_members(&owner).is_ok());
        assert!(PermissionChecker::can_manage_members(&admin).is_ok());
        assert!(PermissionChecker::can_manage_members(&member).is_err());
    }

    #[test]
    fn test_permission_checker_can_delete_chat() {
        let owner = ChatMember::new(1, 1, MemberRole::Owner);
        let admin = ChatMember::new(1, 2, MemberRole::Admin);
        let member = ChatMember::new(1, 3, MemberRole::Member);

        assert!(PermissionChecker::can_delete_chat(&owner).is_ok());
        assert!(PermissionChecker::can_delete_chat(&admin).is_err());
        assert!(PermissionChecker::can_delete_chat(&member).is_err());
    }

    #[test]
    fn test_permission_checker_can_manage_member() {
        let owner = ChatMember::new(1, 1, MemberRole::Owner);
        let admin = ChatMember::new(1, 2, MemberRole::Admin);
        let member = ChatMember::new(1, 3, MemberRole::Member);

        // Owner can manage everyone
        assert!(PermissionChecker::can_manage_member(&owner, &admin, MemberAction::UpdateRole).is_ok());
        assert!(PermissionChecker::can_manage_member(&owner, &member, MemberAction::Remove).is_ok());

        // Admin can manage regular members
        assert!(PermissionChecker::can_manage_member(&admin, &member, MemberAction::UpdateRole).is_ok());
        assert!(PermissionChecker::can_manage_member(&admin, &member, MemberAction::Remove).is_ok());

        // Admin cannot manage owners or other admins
        assert!(PermissionChecker::can_manage_member(&admin, &owner, MemberAction::UpdateRole).is_err());
        assert!(PermissionChecker::can_manage_member(&admin, &owner, MemberAction::Remove).is_err());

        // Member cannot manage anyone
        assert!(PermissionChecker::can_manage_member(&member, &member, MemberAction::UpdateRole).is_err());
        assert!(PermissionChecker::can_manage_member(&member, &member, MemberAction::Remove).is_err());

        // Cannot manage yourself
        assert!(PermissionChecker::can_manage_member(&owner, &owner, MemberAction::Remove).is_err());
    }
}