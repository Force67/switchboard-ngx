use sqlx::SqlitePool;
use crate::routes::models::{ChatInvite, CreateInviteRequest};
use super::error::ServiceError;

pub async fn create_invite(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
    req: CreateInviteRequest,
) -> Result<(ChatInvite, Vec<i64>), ServiceError> {
    // Check if chat exists and is a group chat, and user is a member
    let result: Option<(i64, String)> = sqlx::query_as(
        r#"
        SELECT c.id, cm.role FROM chats c
        JOIN chat_members cm ON c.id = cm.chat_id
        WHERE c.public_id = ? AND c.chat_type = 'group' AND cm.user_id = ?
        "#,
    )
    .bind(chat_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (chat_db_id, user_role) =
        result.ok_or_else(|| ServiceError::not_found("Chat not found or not a group chat"))?;

    // Check if user has permission to invite (owner or admin)
    if user_role != "owner" && user_role != "admin" {
        return Err(ServiceError::forbidden("Only owners and admins can invite members"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO chat_invites (chat_id, inviter_id, invitee_email, status, created_at, updated_at)
        VALUES (?, ?, ?, 'pending', ?, ?)
        "#
    )
    .bind(chat_db_id)
    .bind(user_id)
    .bind(&req.email)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let invite_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    let invitee_email = req.email.clone();

    let invite = ChatInvite {
        id: invite_id,
        chat_id: chat_db_id,
        inviter_id: user_id,
        invitee_email: req.email,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let member_ids = super::chat::fetch_chat_member_ids_by_id(pool, chat_db_id).await?;

    // Send notification to invited user if they exist in the system
    if let Ok(invited_user_id) = find_user_by_email(pool, &invitee_email).await {
        if let Ok((inviter_name, chat_title)) = get_user_and_chat_info(pool, user_id, chat_db_id).await {
            // This is a fire-and-forget operation, we don't want to fail invite creation if notification fails
            let _ = super::notification::notify_chat_invite(
                pool,
                invited_user_id,
                &chat_title,
                &inviter_name,
            ).await;
        }
    }

    Ok((invite, member_ids))
}

pub async fn list_invites(
    pool: &SqlitePool,
    chat_id: &str,
    user_id: i64,
) -> Result<Vec<ChatInvite>, ServiceError> {
    // Check if user is a member of the chat
    let chat_db_id = super::message::check_chat_membership(pool, chat_id, user_id).await?;

    let invites = sqlx::query_as::<_, ChatInvite>(
        r#"
        SELECT id, chat_id, inviter_id, invitee_email, status, created_at, updated_at
        FROM chat_invites
        WHERE chat_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(chat_db_id)
    .fetch_all(pool)
    .await?;

    Ok(invites)
}

pub async fn accept_invite(
    pool: &SqlitePool,
    invite_id: i64,
    user_id: i64,
    user_email: Option<String>,
) -> Result<(crate::routes::models::ChatMember, Vec<i64>, String), ServiceError> {
    // Get the invite and check if the email matches
    let invite: Option<(i64, String, String)> = sqlx::query_as(
        r#"
        SELECT ci.chat_id, ci.invitee_email, c.public_id
        FROM chat_invites ci
        JOIN chats c ON c.id = ci.chat_id
        WHERE ci.id = ? AND ci.status = 'pending'
        "#,
    )
    .bind(invite_id)
    .fetch_optional(pool)
    .await?;

    let (chat_db_id, invitee_email, chat_public_id) =
        invite.ok_or_else(|| ServiceError::not_found("Invite not found"))?;

    // Check if the user's email matches
    if user_email.as_ref() != Some(&invitee_email) {
        return Err(ServiceError::forbidden("Invite not for this user"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Update invite status
    sqlx::query("UPDATE chat_invites SET status = 'accepted', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(invite_id)
        .execute(pool)
        .await?;

    // Add user to chat members
    sqlx::query(
        r#"
        INSERT INTO chat_members (chat_id, user_id, role, joined_at)
        VALUES (?, ?, 'member', ?)
        "#,
    )
    .bind(chat_db_id)
    .bind(user_id)
    .bind(&now)
    .execute(pool)
    .await?;

    let member = sqlx::query_as::<_, crate::routes::models::ChatMember>(
        r#"
        SELECT id, chat_id, user_id, role, joined_at
        FROM chat_members
        WHERE chat_id = ? AND user_id = ?
        "#,
    )
    .bind(chat_db_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let member_ids = super::chat::fetch_chat_member_ids_by_id(pool, chat_db_id).await?;

    // Send notification to the inviter that the invite was accepted
    // Get the inviter's user_id from the invite
    let inviter_user_id: Option<i64> = sqlx::query_scalar(
        "SELECT inviter_id FROM chat_invites WHERE id = ?"
    )
    .bind(invite_id)
    .fetch_optional(pool)
    .await?;

    if let Some(inviter_id) = inviter_user_id {
        if let Ok((accepted_user_name, chat_title)) = get_user_and_chat_info(pool, user_id, chat_db_id).await {
            // This is a fire-and-forget operation, we don't want to fail invite acceptance if notification fails
            let _ = super::notification::notify_invite_accepted(
                pool,
                inviter_id,
                &accepted_user_name,
                &chat_title,
            ).await;
        }
    }

    Ok((member, member_ids, chat_public_id))
}

pub async fn reject_invite(
    pool: &SqlitePool,
    invite_id: i64,
    user_email: Option<String>,
) -> Result<(), ServiceError> {
    // Get the invite and check if the email matches
    let invitee_email: Option<String> = sqlx::query_scalar(
        "SELECT invitee_email FROM chat_invites WHERE id = ? AND status = 'pending'",
    )
    .bind(invite_id)
    .fetch_optional(pool)
    .await?;

    let invitee_email = invitee_email.ok_or_else(|| ServiceError::not_found("Invite not found"))?;

    // Check if the user's email matches
    if user_email.as_ref() != Some(&invitee_email) {
        return Err(ServiceError::forbidden("Invite not for this user"));
    }

    let now = chrono::Utc::now().to_rfc3339();

    // Update invite status
    sqlx::query("UPDATE chat_invites SET status = 'rejected', updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(invite_id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn find_user_by_email(pool: &SqlitePool, email: &str) -> Result<i64, ServiceError> {
    let user_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM users WHERE email = ?"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    user_id.ok_or_else(|| ServiceError::not_found("User not found"))
}

async fn get_user_and_chat_info(
    pool: &SqlitePool,
    user_id: i64,
    chat_db_id: i64,
) -> Result<(String, String), ServiceError> {
    let result: Option<(Option<String>, String)> = sqlx::query_as(
        r#"
        SELECT u.display_name, c.title
        FROM users u
        JOIN chats c ON c.id = ?
        WHERE u.id = ?
        "#,
    )
    .bind(chat_db_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some((Some(display_name), chat_title)) => Ok((display_name, chat_title)),
        Some((None, chat_title)) => Ok(("Unknown User".to_string(), chat_title)),
        _ => Err(ServiceError::internal("Failed to get user and chat info")),
    }
}

