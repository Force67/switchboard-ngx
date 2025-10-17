use sqlx::SqlitePool;
use crate::routes::models::{Folder, CreateFolderRequest, UpdateFolderRequest};
use super::error::ServiceError;

pub async fn list_folders(pool: &SqlitePool, user_id: i64) -> Result<Vec<Folder>, ServiceError> {
    let folders = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        WHERE user_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(folders)
}

pub async fn create_folder(
    pool: &SqlitePool,
    user_id: i64,
    req: CreateFolderRequest,
) -> Result<Folder, ServiceError> {
    let public_id = cuid2::create_id();
    let now = chrono::Utc::now().to_rfc3339();

    // Resolve parent folder ID from public_id if provided
    let parent_db_id = if let Some(parent_public_id) = &req.parent_id {
        Some(
            sqlx::query_scalar::<_, i64>("SELECT id FROM folders WHERE public_id = ? AND user_id = ?")
                .bind(parent_public_id)
                .bind(user_id)
                .fetch_optional(pool)
                .await?
                .ok_or_else(|| ServiceError::bad_request("Parent folder not found"))?,
        )
    } else {
        None
    };

    sqlx::query(
        r#"
        INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&public_id)
    .bind(user_id)
    .bind(&req.name)
    .bind(&req.color)
    .bind(parent_db_id)
    .bind(false)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let folder_id = sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    Ok(Folder {
        id: folder_id,
        public_id,
        user_id,
        name: req.name.clone(),
        color: req.color.clone(),
        parent_id: parent_db_id,
        collapsed: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

pub async fn get_folder(pool: &SqlitePool, user_id: i64, folder_public_id: &str) -> Result<Folder, ServiceError> {
    let folder = sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
        FROM folders
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(folder_public_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ServiceError::not_found("Folder not found"))?;

    Ok(folder)
}

pub async fn update_folder(
    pool: &SqlitePool,
    user_id: i64,
    folder_public_id: &str,
    req: UpdateFolderRequest,
) -> Result<Folder, ServiceError> {
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE folders
        SET name = COALESCE(?, name),
            color = COALESCE(?, color),
            collapsed = COALESCE(?, collapsed),
            updated_at = ?
        WHERE public_id = ? AND user_id = ?
        "#,
    )
    .bind(&req.name)
    .bind(&req.color)
    .bind(req.collapsed)
    .bind(&now)
    .bind(folder_public_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    get_folder(pool, user_id, folder_public_id).await
}

pub async fn delete_folder(
    pool: &SqlitePool,
    user_id: i64,
    folder_public_id: &str,
) -> Result<(), ServiceError> {
    let result = sqlx::query("DELETE FROM folders WHERE public_id = ? AND user_id = ?")
        .bind(folder_public_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ServiceError::not_found("Folder not found"));
    }

    Ok(())
}

// TODO: Fix folder service tests - currently failing due to database schema conflicts