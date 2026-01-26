//! Folder-related request and response types

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Folder entity
#[derive(Debug, Serialize, FromRow, ToSchema, Clone)]
pub struct Folder {
    pub id: i64,
    pub public_id: String,
    pub user_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<i64>,
    pub collapsed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Response containing multiple folders
#[derive(Debug, Serialize, ToSchema)]
pub struct FoldersResponse {
    pub folders: Vec<Folder>,
}

/// Response containing a single folder
#[derive(Debug, Serialize, ToSchema)]
pub struct FolderResponse {
    pub folder: Folder,
}

/// Request body for creating a folder
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFolderRequest {
    pub name: String,
    pub color: Option<String>,
    /// Parent folder public_id
    pub parent_id: Option<String>,
}

/// Request body for updating a folder
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFolderRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub collapsed: Option<bool>,
    /// Parent folder public_id
    pub parent_id: Option<String>,
}
