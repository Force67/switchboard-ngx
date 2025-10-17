use sqlx::SqlitePool;
use crate::routes::models::{ChatMember, UpdateMemberRoleRequest};
use super::error::ServiceError;

pub async fn list_members(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
) -> Result<Vec<ChatMember>, ServiceError> {
    // Check if user is a member of the chat
    let chat_db_id = super::message::check_chat_membership(pool, chat_id, user_id).await?;

    let members = sqlx::query_as::<_, ChatMember>(
        r#"
        SELECT id, chat_id, user_id, role, joined_at
        FROM chat_members
        WHERE chat_id = ?
        ORDER BY joined_at ASC
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(pool)
    .await?;

    Ok(members)
}

pub async fn update_member_role(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
    target_user_id: i64,
    req: UpdateMemberRoleRequest,
) -> Result<(ChatMember, Vec<i64>), ServiceError> {
    // Check if user is an owner/admin of the chat
    let user_role: Option<String> = sqlx::query_scalar(
        r#"
        SELECT cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let user_role = user_role.ok_or_else(|| ServiceError::forbidden("Not a member of this chat"))?;

    if user_role != "owner" && user_role != "admin" {
        return Err(ServiceError::forbidden("Insufficient permissions"));
    }

    // Validate role
    if req.role != "member" && req.role != "admin" && req.role != "owner" {
        return Err(ServiceError::bad_request("Invalid role"));
    }

    // Prevent demoting the last owner
    if req.role != "owner" {
        let owner_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM chat_members cm
            JOIN chats c ON c.id = cm.chat_id
            WHERE c.public_id = ? AND cm.role = 'owner'
            "#,
        )
        .bind(chat_id)
        .fetch_one(pool)
        .await?;

        if owner_count <= 1 {
            let target_role: Option<String> = sqlx::query_scalar(
                "SELECT role FROM chat_members cm JOIN chats c ON c.id = cm.chat_id WHERE c.public_id = ? AND cm.user_id = ?"
            )
            .bind(chat_id)
            .bind(target_user_id)
            .fetch_optional(pool)
            .await?;

            if target_role.as_deref() == Some("owner") {
                return Err(ServiceError::bad_request("Cannot remove the last owner"));
            }
        }
    }

    // Update the role
    sqlx::query(
        r#"
        UPDATE chat_members
        SET role = ?
        WHERE chat_id = (SELECT id FROM chats WHERE public_id = ?) AND user_id = ?
        "#,
    )
    .bind(&req.role)
    .bind(chat_id)
    .bind(target_user_id)
    .execute(pool)
    .await?;

    // Return the updated member
    let member = sqlx::query_as::<_, ChatMember>(
        r#"
        SELECT cm.id, cm.chat_id, cm.user_id, cm.role, cm.joined_at
        FROM chat_members cm
        JOIN chats c ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(target_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::not_found("Member not found"))?;

    let member_ids = super::chat::fetch_chat_member_ids_by_id(pool, member.chat_id).await?;

    Ok((member, member_ids))
}

pub async fn remove_member(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
    target_user_id: i64,
) -> Result<Vec<i64>, ServiceError> {
    // Check if user is an owner/admin of the chat and capture chat id
    let chat_info: Option<(i64, String)> = sqlx::query_as(
        r#"
        SELECT c.id, cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (chat_db_id, user_role) =
        chat_info.ok_or_else(|| ServiceError::forbidden("Not a member of this chat"))?;

    if user_role != "owner" && user_role != "admin" {
        return Err(ServiceError::forbidden("Insufficient permissions"));
    }

    // Prevent removing the last owner
    let owner_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chat_members WHERE chat_id = ? AND role = 'owner'",
    )
    .bind(chat_db_id)
    .fetch_one(pool)
    .await?;

    let target_role: Option<String> =
        sqlx::query_scalar("SELECT role FROM chat_members WHERE chat_id = ? AND user_id = ?")
            .bind(chat_db_id)
            .bind(target_user_id)
            .fetch_optional(pool)
            .await?;

    if target_role.as_deref() == Some("owner") && owner_count <= 1 {
        return Err(ServiceError::bad_request("Cannot remove the last owner"));
    }

    let member_ids = super::chat::fetch_chat_member_ids_by_id(pool, chat_db_id).await?;

    // Remove the member
    sqlx::query("DELETE FROM chat_members WHERE chat_id = ? AND user_id = ?")
        .bind(chat_db_id)
        .bind(target_user_id)
        .execute(pool)
        .await?;

    Ok(member_ids)
}