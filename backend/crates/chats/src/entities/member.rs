use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Represents a member of a chat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMember {
    /// Database primary key
    pub id: i64,
    /// Chat ID this member belongs to
    pub chat_id: i64,
    /// User ID of the member
    pub user_id: i64,
    /// Member role in the chat
    pub role: MemberRole,
    /// When the member joined the chat
    pub joined_at: String,
    /// Last active timestamp (optional)
    pub last_active_at: Option<String>,
}

/// Member role enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

impl From<&str> for MemberRole {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => MemberRole::Admin,
            "owner" => MemberRole::Owner,
            _ => MemberRole::Member,
        }
    }
}

impl From<MemberRole> for String {
    fn from(role: MemberRole) -> Self {
        match role {
            MemberRole::Owner => "owner".to_string(),
            MemberRole::Admin => "admin".to_string(),
            MemberRole::Member => "member".to_string(),
        }
    }
}

/// Member with user information included
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithUser {
    pub id: i64,
    pub chat_id: i64,
    pub user_id: i64,
    pub user_public_id: String,
    pub user_display_name: Option<String>,
    pub user_email: Option<String>,
    pub role: MemberRole,
    pub joined_at: String,
    pub last_active_at: Option<String>,
}

impl ChatMember {
    /// Create a new chat member instance
    pub fn new(chat_id: i64, user_id: i64, role: MemberRole) -> Self {
        Self {
            id: 0, // Will be set by database
            chat_id,
            user_id,
            role,
            joined_at: Utc::now().to_rfc3339(),
            last_active_at: None,
        }
    }

    /// Check if the member is an owner
    pub fn is_owner(&self) -> bool {
        matches!(self.role, MemberRole::Owner)
    }

    /// Check if the member is an admin
    pub fn is_admin(&self) -> bool {
        matches!(self.role, MemberRole::Admin)
    }

    /// Check if the member can manage other members
    pub fn can_manage_members(&self) -> bool {
        matches!(self.role, MemberRole::Owner | MemberRole::Admin)
    }

    /// Check if the member can delete the chat
    pub fn can_delete_chat(&self) -> bool {
        matches!(self.role, MemberRole::Owner)
    }

    /// Check if the member can invite others
    pub fn can_invite(&self) -> bool {
        matches!(self.role, MemberRole::Owner | MemberRole::Admin)
    }

    /// Update last active timestamp
    pub fn update_last_active(&mut self) {
        self.last_active_at = Some(Utc::now().to_rfc3339());
    }

    /// Change the member's role
    pub fn change_role(&mut self, new_role: MemberRole) {
        self.role = new_role;
    }

    /// Validate member data
    pub fn validate(&self) -> Result<(), String> {
        if self.chat_id <= 0 {
            return Err("Invalid chat ID".to_string());
        }

        if self.user_id <= 0 {
            return Err("Invalid user ID".to_string());
        }

        // Validate joined_at timestamp format
        if let Err(_) = chrono::DateTime::parse_from_rfc3339(&self.joined_at) {
            return Err("Invalid joined_at timestamp format".to_string());
        }

        // Validate last_active_at timestamp if present
        if let Some(ref last_active) = self.last_active_at {
            if let Err(_) = chrono::DateTime::parse_from_rfc3339(last_active) {
                return Err("Invalid last_active_at timestamp format".to_string());
            }
        }

        Ok(())
    }
}

impl MemberRole {
    /// Get the permission level for this role (higher number = more permissions)
    pub fn permission_level(&self) -> u8 {
        match self {
            MemberRole::Owner => 3,
            MemberRole::Admin => 2,
            MemberRole::Member => 1,
        }
    }

    /// Check if this role has at least the permissions of another role
    pub fn has_at_least_permissions(&self, other: &MemberRole) -> bool {
        self.permission_level() >= other.permission_level()
    }

    /// Get all possible roles
    pub fn all() -> Vec<MemberRole> {
        vec![MemberRole::Owner, MemberRole::Admin, MemberRole::Member]
    }

    /// Get roles that can be assigned by this role
    pub fn assignable_roles(&self) -> Vec<MemberRole> {
        match self {
            MemberRole::Owner => vec![MemberRole::Owner, MemberRole::Admin, MemberRole::Member],
            MemberRole::Admin => vec![MemberRole::Admin, MemberRole::Member],
            MemberRole::Member => vec![], // Members cannot assign roles
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_creation() {
        let member = ChatMember::new(1, 1, MemberRole::Owner);

        assert_eq!(member.chat_id, 1);
        assert_eq!(member.user_id, 1);
        assert_eq!(member.role, MemberRole::Owner);
        assert!(member.is_owner());
        assert!(member.can_manage_members());
        assert!(member.can_delete_chat());
        assert!(member.can_invite());
    }

    #[test]
    fn test_member_permissions() {
        let owner = ChatMember::new(1, 1, MemberRole::Owner);
        let admin = ChatMember::new(1, 2, MemberRole::Admin);
        let member = ChatMember::new(1, 3, MemberRole::Member);

        assert!(owner.is_owner());
        assert!(owner.can_delete_chat());
        assert!(owner.can_manage_members());
        assert!(owner.can_invite());

        assert!(!admin.is_owner());
        assert!(admin.is_admin());
        assert!(!admin.can_delete_chat());
        assert!(admin.can_manage_members());
        assert!(admin.can_invite());

        assert!(!member.is_owner());
        assert!(!member.is_admin());
        assert!(!member.can_delete_chat());
        assert!(!member.can_manage_members());
        assert!(!member.can_invite());
    }

    #[test]
    fn test_member_role_conversion() {
        assert_eq!(MemberRole::from("owner"), MemberRole::Owner);
        assert_eq!(MemberRole::from("admin"), MemberRole::Admin);
        assert_eq!(MemberRole::from("member"), MemberRole::Member);
        assert_eq!(MemberRole::from("unknown"), MemberRole::Member);

        assert_eq!(String::from(MemberRole::Owner), "owner");
        assert_eq!(String::from(MemberRole::Admin), "admin");
        assert_eq!(String::from(MemberRole::Member), "member");
    }

    #[test]
    fn test_member_role_permissions() {
        assert_eq!(MemberRole::Owner.permission_level(), 3);
        assert_eq!(MemberRole::Admin.permission_level(), 2);
        assert_eq!(MemberRole::Member.permission_level(), 1);

        assert!(MemberRole::Owner.has_at_least_permissions(&MemberRole::Admin));
        assert!(MemberRole::Owner.has_at_least_permissions(&MemberRole::Member));
        assert!(MemberRole::Admin.has_at_least_permissions(&MemberRole::Member));
        assert!(!MemberRole::Member.has_at_least_permissions(&MemberRole::Admin));
        assert!(!MemberRole::Admin.has_at_least_permissions(&MemberRole::Owner));
    }

    #[test]
    fn test_assignable_roles() {
        let owner_assignable = MemberRole::Owner.assignable_roles();
        assert_eq!(owner_assignable.len(), 3);
        assert!(owner_assignable.contains(&MemberRole::Owner));
        assert!(owner_assignable.contains(&MemberRole::Admin));
        assert!(owner_assignable.contains(&MemberRole::Member));

        let admin_assignable = MemberRole::Admin.assignable_roles();
        assert_eq!(admin_assignable.len(), 2);
        assert!(admin_assignable.contains(&MemberRole::Admin));
        assert!(admin_assignable.contains(&MemberRole::Member));
        assert!(!admin_assignable.contains(&MemberRole::Owner));

        let member_assignable = MemberRole::Member.assignable_roles();
        assert_eq!(member_assignable.len(), 0);
    }

    #[test]
    fn test_role_change() {
        let mut member = ChatMember::new(1, 1, MemberRole::Member);
        assert!(!member.can_manage_members());

        member.change_role(MemberRole::Admin);
        assert!(member.is_admin());
        assert!(member.can_manage_members());
        assert!(!member.can_delete_chat());

        member.change_role(MemberRole::Owner);
        assert!(member.is_owner());
        assert!(member.can_delete_chat());
    }

    #[test]
    fn test_update_last_active() {
        let mut member = ChatMember::new(1, 1, MemberRole::Member);
        assert!(member.last_active_at.is_none());

        let original_joined_at = member.joined_at.clone();

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        member.update_last_active();
        assert!(member.last_active_at.is_some());
        assert_ne!(member.last_active_at.unwrap(), original_joined_at);
        assert_eq!(member.joined_at, original_joined_at);
    }

    #[test]
    fn test_member_validation() {
        let mut member = ChatMember::new(1, 1, MemberRole::Member);
        assert!(member.validate().is_ok());

        member.chat_id = -1;
        assert!(member.validate().is_err());

        member.chat_id = 1;
        member.user_id = -1;
        assert!(member.validate().is_err());

        member.user_id = 1;
        member.joined_at = "invalid-date".to_string();
        assert!(member.validate().is_err());
    }
}