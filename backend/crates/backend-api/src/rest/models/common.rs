//! Common types shared across REST endpoints

use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Re-export the canonical ErrorResponse from the error module
pub use crate::error::ErrorResponse;

/// Common pagination parameters for list endpoints
#[derive(Debug, Deserialize, IntoParams, ToSchema, Default)]
pub struct PaginationQuery {
    /// Maximum number of items to return
    pub limit: Option<i64>,
    /// Number of items to skip
    pub offset: Option<i64>,
}
