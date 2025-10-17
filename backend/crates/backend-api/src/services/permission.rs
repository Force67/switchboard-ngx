use sqlx::SqlitePool;
use crate::routes::models::Permission;
use super::error::ServiceError;

pub const PERMISSION_LEVELS: &[&str] = &["read", "write", "admin"];
pub const RESOURCE_TYPES: &[&str] = &["chat", "folder", "workspace"];

pub async fn check_permission(
    pool: &SqlitePool,
    user_id: i64,
    resource_type: &str,
    resource_id: i64,
    required_permission: &str,
) -> Result<bool, ServiceError> {
    let permission_level = sqlx::query_scalar::<_, String>(
        r#"
        SELECT permission_level
        FROM permissions
        WHERE user_id = ? AND resource_type = ? AND resource_id = ?
        ORDER BY
            CASE permission_level
                WHEN 'admin' THEN 1
                WHEN 'write' THEN 2
                WHEN 'read' THEN 3
                ELSE 4
            END
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(resource_type)
    .bind(resource_id)
    .fetch_optional(pool)
    .await?;

    if let Some(level) = permission_level {
        match required_permission {
            "read" => Ok(true), // All permission levels include read
            "write" => Ok(matches!(level.as_str(), "write" | "admin")),
            "admin" => Ok(level == "admin"),
            _ => Ok(false),
        }
    } else {
        Ok(false)
    }
}

pub async fn grant_permission(
    pool: &SqlitePool,
    user_id: i64,
    resource_type: &str,
    resource_id: i64,
    permission_level: &str,
    _granted_by_user_id: i64,
) -> Result<Permission, ServiceError> {
    let now = chrono::Utc::now().to_rfc3339();

    // Use INSERT OR REPLACE to handle existing permissions
    sqlx::query(
        r#"
        INSERT OR REPLACE INTO permissions (user_id, resource_type, resource_id, permission_level, granted_at)
        VALUES (?, ?, ?, ?, ?)
        "#
    )
    .bind(user_id)
    .bind(resource_type)
    .bind(resource_id)
    .bind(permission_level)
    .bind(&now)
    .execute(pool)
    .await?;

    // Fetch the created/updated permission
    let permission = sqlx::query_as::<_, Permission>(
        r#"
        SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
        FROM permissions
        WHERE user_id = ? AND resource_type = ? AND resource_id = ?
        "#,
    )
    .bind(user_id)
    .bind(resource_type)
    .bind(resource_id)
    .fetch_one(pool)
    .await?;

    Ok(permission)
}

pub async fn revoke_permission(
    pool: &SqlitePool,
    user_id: i64,
    resource_type: &str,
    resource_id: i64,
) -> Result<(), ServiceError> {
    let result = sqlx::query(
        "DELETE FROM permissions WHERE user_id = ? AND resource_type = ? AND resource_id = ?",
    )
    .bind(user_id)
    .bind(resource_type)
    .bind(resource_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::NotFound);
    }

    Ok(())
}

pub async fn get_user_permissions(
    pool: &SqlitePool,
    user_id: i64,
    resource_type: Option<&str>,
    resource_id: Option<i64>,
) -> Result<Vec<Permission>, ServiceError> {
    let permissions = match (resource_type, resource_id) {
        (Some(rt), Some(rid)) => sqlx::query_as::<_, Permission>(
            r#"
                SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                FROM permissions
                WHERE user_id = ? AND resource_type = ? AND resource_id = ?
                ORDER BY granted_at DESC
                "#,
        )
        .bind(user_id)
        .bind(rt)
        .bind(rid)
        .fetch_all(pool)
        .await?,
        (Some(rt), None) => sqlx::query_as::<_, Permission>(
            r#"
                SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                FROM permissions
                WHERE user_id = ? AND resource_type = ?
                ORDER BY granted_at DESC
                "#,
        )
        .bind(user_id)
        .bind(rt)
        .fetch_all(pool)
        .await?,
        (None, Some(rid)) => sqlx::query_as::<_, Permission>(
            r#"
                SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                FROM permissions
                WHERE user_id = ? AND resource_id = ?
                ORDER BY granted_at DESC
                "#,
        )
        .bind(user_id)
        .bind(rid)
        .fetch_all(pool)
        .await?,
        (None, None) => sqlx::query_as::<_, Permission>(
            r#"
                SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
                FROM permissions
                WHERE user_id = ?
                ORDER BY granted_at DESC
                "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?,
    };

    Ok(permissions)
}

pub async fn get_resource_permissions(
    pool: &SqlitePool,
    resource_type: &str,
    resource_id: i64,
) -> Result<Vec<Permission>, ServiceError> {
    let permissions = sqlx::query_as::<_, Permission>(
        r#"
        SELECT id, user_id, resource_type, resource_id, permission_level, granted_at
        FROM permissions
        WHERE resource_type = ? AND resource_id = ?
        ORDER BY granted_at DESC
        "#,
    )
    .bind(resource_type)
    .bind(resource_id)
    .fetch_all(pool)
    .await?;

    Ok(permissions)
}

pub async fn check_chat_permission(
    pool: &SqlitePool,
    user_id: i64,
    chat_id: i64,
    required_permission: &str,
) -> Result<bool, ServiceError> {
    let role = sqlx::query_scalar::<_, String>(
        "SELECT role FROM chat_members WHERE chat_id = ? AND user_id = ?",
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(role) = role {
        match required_permission {
            "read" => Ok(true),  // All chat members can read
            "write" => Ok(true), // All chat members can write messages
            "admin" => Ok(matches!(role.as_str(), "admin" | "owner")),
            "owner" => Ok(role == "owner"),
            _ => Ok(false),
        }
    } else {
        // Check permissions table for explicit permission
        check_permission(pool, user_id, "chat", chat_id, required_permission).await
    }
}

pub async fn check_folder_permission(
    pool: &SqlitePool,
    user_id: i64,
    folder_id: i64,
    required_permission: &str,
) -> Result<bool, ServiceError> {
    let folder_user_id =
        sqlx::query_scalar::<_, i64>("SELECT user_id FROM folders WHERE id = ?")
            .bind(folder_id)
            .fetch_optional(pool)
            .await?;

    if let Some(owner_id) = folder_user_id {
        if owner_id == user_id {
            return Ok(true); // Owner has all permissions
        }
    }

    // Check permissions table for explicit permission
    check_permission(pool, user_id, "folder", folder_id, required_permission).await
}

// Helper function to resolve public IDs to database IDs
pub async fn resolve_resource_id(
    pool: &SqlitePool,
    resource_type: &str,
    public_id: &str,
) -> Result<Option<i64>, ServiceError> {
    let resource_id = match resource_type {
        "chat" => sqlx::query_scalar::<_, i64>("SELECT id FROM chats WHERE public_id = ?")
            .bind(public_id)
            .fetch_optional(pool)
            .await?,
        "folder" => sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ?")
            .bind(public_id)
            .fetch_optional(pool)
            .await?,
        _ => return Err(ServiceError::BadRequest("Invalid resource type".to_string())),
    };

    Ok(resource_id)
}

// Helper function to resolve user public ID to database ID
pub async fn resolve_user_id(
    pool: &SqlitePool,
    public_id: &str,
) -> Result<Option<i64>, ServiceError> {
    let user_id = sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE public_id = ?")
        .bind(public_id)
        .fetch_optional(pool)
        .await?;

    Ok(user_id)
}

// Service functions that handle business logic for API routes

/// Get user permissions with authorization check
/// Users can only see their own permissions unless they're admin
pub async fn get_user_permissions_with_auth(
    pool: &SqlitePool,
    requester_user_id: i64,
    target_user_public_id: &str,
) -> Result<Vec<Permission>, ServiceError> {
    let target_user_id = resolve_user_id(pool, target_user_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("User not found".to_string()))?;

    // Users can only see their own permissions unless they're admin
    if requester_user_id != target_user_id {
        // TODO: Implement admin check when admin system is available
        return Err(ServiceError::BadRequest("Cannot view other users' permissions".to_string()));
    }

    get_user_permissions(pool, target_user_id, None, None).await
}

/// Get resource permissions with admin authorization check
pub async fn get_resource_permissions_with_auth(
    pool: &SqlitePool,
    requester_user_id: i64,
    resource_type: &str,
    resource_public_id: &str,
) -> Result<Vec<Permission>, ServiceError> {
    let resource_id = resolve_resource_id(pool, resource_type, resource_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("Resource not found".to_string()))?;

    // Check if user has admin permission for this resource
    if !check_permission(pool, requester_user_id, resource_type, resource_id, "admin").await? {
        return Err(ServiceError::BadRequest("Admin permission required".to_string()));
    }

    get_resource_permissions(pool, resource_type, resource_id).await
}

/// Grant permission to user with admin authorization check
pub async fn grant_permission_with_auth(
    pool: &SqlitePool,
    requester_user_id: i64,
    resource_type: &str,
    resource_public_id: &str,
    target_user_public_id: &str,
    permission_level: &str,
) -> Result<Permission, ServiceError> {
    let resource_id = resolve_resource_id(pool, resource_type, resource_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("Resource not found".to_string()))?;

    // Check if user has admin permission for this resource
    if !check_permission(pool, requester_user_id, resource_type, resource_id, "admin").await? {
        return Err(ServiceError::BadRequest("Admin permission required".to_string()));
    }

    let target_user_id = resolve_user_id(pool, target_user_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("Target user not found".to_string()))?;

    grant_permission(pool, target_user_id, resource_type, resource_id, permission_level, requester_user_id).await
}

/// Revoke permission from user with admin authorization check
pub async fn revoke_permission_with_auth(
    pool: &SqlitePool,
    requester_user_id: i64,
    resource_type: &str,
    resource_public_id: &str,
    target_user_public_id: &str,
) -> Result<(), ServiceError> {
    let resource_id = resolve_resource_id(pool, resource_type, resource_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("Resource not found".to_string()))?;

    // Check if user has admin permission for this resource
    if !check_permission(pool, requester_user_id, resource_type, resource_id, "admin").await? {
        return Err(ServiceError::BadRequest("Admin permission required".to_string()));
    }

    let target_user_id = resolve_user_id(pool, target_user_public_id)
        .await?
        .ok_or_else(|| ServiceError::BadRequest("Target user not found".to_string()))?;

    revoke_permission(pool, target_user_id, resource_type, resource_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::test_utils::{create_test_db, create_test_user, create_test_chat};
    use crate::services::test_utils::fixtures::*;

    #[tokio::test]
    async fn test_check_permission_no_permission() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let result = check_permission(&pool, user.id, "chat", 1, "read").await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // No permission granted
    }

    #[tokio::test]
    async fn test_grant_permission_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let result = grant_permission(&pool, user.id, "chat", 1, "read", user.id).await;
        assert!(result.is_ok());
        let permission = result.unwrap();
        assert_eq!(permission.user_id, user.id);
        assert_eq!(permission.resource_type, "chat");
        assert_eq!(permission.resource_id, 1);
        assert_eq!(permission.permission_level, "read");
    }

    #[tokio::test]
    async fn test_check_permission_after_grant() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Grant read permission
        grant_permission(&pool, user.id, "chat", 1, "read", user.id).await.unwrap();

        // Check various permission levels
        assert!(check_permission(&pool, user.id, "chat", 1, "read").await.unwrap());
        assert!(!check_permission(&pool, user.id, "chat", 1, "write").await.unwrap());
        assert!(!check_permission(&pool, user.id, "chat", 1, "admin").await.unwrap());

        // Grant admin permission (should replace read)
        grant_permission(&pool, user.id, "chat", 1, "admin", user.id).await.unwrap();

        // Check permissions again
        assert!(check_permission(&pool, user.id, "chat", 1, "read").await.unwrap());
        assert!(check_permission(&pool, user.id, "chat", 1, "write").await.unwrap());
        assert!(check_permission(&pool, user.id, "chat", 1, "admin").await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_permission_success() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Grant permission first
        grant_permission(&pool, user.id, "chat", 1, "read", user.id).await.unwrap();

        // Revoke permission
        let result = revoke_permission(&pool, user.id, "chat", 1).await;
        assert!(result.is_ok());

        // Check permission is revoked
        assert!(!check_permission(&pool, user.id, "chat", 1, "read").await.unwrap());
    }

    #[tokio::test]
    async fn test_revoke_permission_not_found() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        let result = revoke_permission(&pool, user.id, "chat", 1).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::NotFound));
    }

    #[tokio::test]
    async fn test_get_user_permissions() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Grant some permissions
        grant_permission(&pool, user.id, "chat", 1, "read", user.id).await.unwrap();
        grant_permission(&pool, user.id, "folder", 1, "admin", user.id).await.unwrap();

        // Get all permissions
        let result = get_user_permissions(&pool, user.id, None, None).await;
        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 2);

        // Get chat permissions only
        let result = get_user_permissions(&pool, user.id, Some("chat"), None).await;
        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].resource_type, "chat");

        // Get specific resource permissions
        let result = get_user_permissions(&pool, user.id, Some("chat"), Some(1)).await;
        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0].resource_id, 1);
    }

    #[tokio::test]
    async fn test_get_resource_permissions() {
        let (pool, _temp_dir) = create_test_db().await;
        let user1 = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();
        let user2 = create_test_user(&pool, TEST_USER_ID_2, "test-user-456", Some("test2@example.com"), Some("Test User 2")).await.unwrap();

        // Grant permissions to both users for the same resource
        grant_permission(&pool, user1.id, "chat", 1, "read", user1.id).await.unwrap();
        grant_permission(&pool, user2.id, "chat", 1, "admin", user2.id).await.unwrap();

        let result = get_resource_permissions(&pool, "chat", 1).await;
        assert!(result.is_ok());
        let permissions = result.unwrap();
        assert_eq!(permissions.len(), 2);

        // Verify both users have permissions
        let user_ids: Vec<i64> = permissions.iter().map(|p| p.user_id).collect();
        assert!(user_ids.contains(&user1.id));
        assert!(user_ids.contains(&user2.id));
    }

    #[tokio::test]
    async fn test_check_chat_permission_via_membership() {
        let (pool, _temp_dir) = create_test_db().await;
        let user1 = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();
        let user2 = create_test_user(&pool, TEST_USER_ID_2, "test-user-456", Some("test2@example.com"), Some("Test User 2")).await.unwrap();
        let chat_id = create_test_chat(&pool, user1.id, TEST_CHAT_TITLE, TEST_CHAT_TYPE).await.unwrap();

        // Add second user as chat member
        sqlx::query("INSERT INTO chat_members (chat_id, user_id, role, joined_at) VALUES (?, ?, 'member', ?)")
            .bind(chat_id)
            .bind(user2.id)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&pool)
            .await
            .unwrap();

        // Check chat permissions via membership for user2
        assert!(check_chat_permission(&pool, user2.id, chat_id, "read").await.unwrap());
        assert!(check_chat_permission(&pool, user2.id, chat_id, "write").await.unwrap());
        assert!(!check_chat_permission(&pool, user2.id, chat_id, "admin").await.unwrap());
        assert!(!check_chat_permission(&pool, user2.id, chat_id, "owner").await.unwrap());
    }

    #[tokio::test]
    async fn test_check_folder_permission_as_owner() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Create a folder owned by the user
        let folder_id = sqlx::query(
            "INSERT INTO folders (public_id, user_id, name, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind("test-folder-id")
        .bind(user.id)
        .bind("Test Folder")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        // Owner should have all permissions
        assert!(check_folder_permission(&pool, user.id, folder_id, "read").await.unwrap());
        assert!(check_folder_permission(&pool, user.id, folder_id, "write").await.unwrap());
        assert!(check_folder_permission(&pool, user.id, folder_id, "admin").await.unwrap());
    }

    #[tokio::test]
    async fn test_resolve_resource_id() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();
        let chat_id = create_test_chat(&pool, user.id, TEST_CHAT_TITLE, TEST_CHAT_TYPE).await.unwrap();

        // Get the public_id of the chat
        let public_id = sqlx::query_scalar::<_, String>("SELECT public_id FROM chats WHERE id = ?")
            .bind(chat_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        // Resolve public ID back to database ID
        let result = resolve_resource_id(&pool, "chat", &public_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(chat_id));

        // Test invalid resource type
        let result = resolve_resource_id(&pool, "invalid", &public_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resolve_user_id() {
        let (pool, _temp_dir) = create_test_db().await;
        let user = create_test_user(&pool, TEST_USER_ID, TEST_USER_PUBLIC_ID, Some(TEST_USER_EMAIL), Some(TEST_USER_DISPLAY_NAME)).await.unwrap();

        // Resolve public ID to database ID
        let result = resolve_user_id(&pool, TEST_USER_PUBLIC_ID).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TEST_USER_ID));

        // Test non-existent user
        let result = resolve_user_id(&pool, "non-existent").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}