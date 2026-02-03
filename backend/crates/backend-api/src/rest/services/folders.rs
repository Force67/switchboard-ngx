//! Folders service implementation

use crate::error::{GatewayError, GatewayResult};
use crate::generated::rest::traits::FoldersServiceTrait;
use crate::rest::models::{
    CreateFolderRequest, Folder, FolderResponse, FoldersResponse, UpdateFolderRequest,
};
use crate::state::GatewayState;
use chrono::Utc;

/// Implementation of FoldersServiceTrait
pub struct FoldersServiceImpl<'a> {
    state: &'a GatewayState,
}

impl<'a> FoldersServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }

    async fn resolve_parent_id(&self, user_id: i64, parent_public_id: &str) -> GatewayResult<i64> {
        let parent = self.fetch_folder(user_id, parent_public_id).await?;
        Ok(parent.id)
    }

    async fn fetch_folder(&self, user_id: i64, folder_public_id: &str) -> GatewayResult<Folder> {
        sqlx::query_as::<_, Folder>(
            "SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
             FROM folders
             WHERE public_id = ? AND user_id = ?",
        )
        .bind(folder_public_id)
        .bind(user_id)
        .fetch_optional(&self.state.pool)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?
        .ok_or_else(|| GatewayError::NotFound("Folder not found".to_string()))
    }
}

impl FoldersServiceTrait for FoldersServiceImpl<'_> {
    async fn list_folders(&self, user_id: i64) -> GatewayResult<FoldersResponse> {
        let folders = sqlx::query_as::<_, Folder>(
            "SELECT id, public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at
             FROM folders
             WHERE user_id = ?
             ORDER BY created_at ASC",
        )
        .bind(user_id)
        .fetch_all(&self.state.pool)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

        Ok(FoldersResponse { folders })
    }

    async fn create_folder(
        &self,
        user_id: i64,
        req: CreateFolderRequest,
    ) -> GatewayResult<FolderResponse> {
        let public_id = cuid2::cuid();
        let now = Utc::now().to_rfc3339();

        let parent_id = if let Some(parent_public_id) = req.parent_id.as_deref() {
            Some(self.resolve_parent_id(user_id, parent_public_id).await?)
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO folders (public_id, user_id, name, color, parent_id, collapsed, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 0, ?, ?)",
        )
        .bind(&public_id)
        .bind(user_id)
        .bind(&req.name)
        .bind(&req.color)
        .bind(parent_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.state.pool)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

        let folder = self.fetch_folder(user_id, &public_id).await?;
        Ok(FolderResponse { folder })
    }

    async fn get_folder(&self, user_id: i64, folder_id: String) -> GatewayResult<FolderResponse> {
        let folder = self.fetch_folder(user_id, &folder_id).await?;
        Ok(FolderResponse { folder })
    }

    async fn update_folder(
        &self,
        user_id: i64,
        folder_id: String,
        req: UpdateFolderRequest,
    ) -> GatewayResult<FolderResponse> {
        let current = self.fetch_folder(user_id, &folder_id).await?;
        let parent_id = if let Some(parent_public_id) = req.parent_id.as_deref() {
            let resolved = self.resolve_parent_id(user_id, parent_public_id).await?;
            if resolved == current.id {
                return Err(GatewayError::InvalidRequest(
                    "Folder cannot be its own parent".to_string(),
                ));
            }
            Some(resolved)
        } else {
            None
        };

        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE folders
             SET name = COALESCE(?, name),
                 color = COALESCE(?, color),
                 collapsed = COALESCE(?, collapsed),
                 parent_id = COALESCE(?, parent_id),
                 updated_at = ?
             WHERE public_id = ? AND user_id = ?",
        )
        .bind(req.name.as_ref())
        .bind(req.color.as_ref())
        .bind(req.collapsed)
        .bind(parent_id)
        .bind(&now)
        .bind(&folder_id)
        .bind(user_id)
        .execute(&self.state.pool)
        .await
        .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

        let folder = self.fetch_folder(user_id, &folder_id).await?;
        Ok(FolderResponse { folder })
    }

    async fn delete_folder(&self, user_id: i64, folder_id: String) -> GatewayResult<()> {
        let folder = self.fetch_folder(user_id, &folder_id).await?;

        sqlx::query("UPDATE chats SET folder_id = NULL WHERE folder_id = ? AND created_by = ?")
            .bind(&folder.public_id)
            .bind(user_id.to_string())
            .execute(&self.state.pool)
            .await
            .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

        let result = sqlx::query("DELETE FROM folders WHERE public_id = ? AND user_id = ?")
            .bind(&folder_id)
            .bind(user_id)
            .execute(&self.state.pool)
            .await
            .map_err(|e| GatewayError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(GatewayError::NotFound("Folder not found".to_string()));
        }

        Ok(())
    }
}
