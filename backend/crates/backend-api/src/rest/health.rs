//! Health check REST endpoints

use std::sync::Arc;

use crate::rest::models::HealthResponse;
use crate::state::GatewayState;
use axum::{routing::get, Json, Router};
use chrono::Utc;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service health status", body = HealthResponse)
    )
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// Create health routes
pub fn create_health_routes() -> Router<Arc<GatewayState>> {
    Router::<Arc<GatewayState>>::new().route("/health", get(health_check))
}
