//! Development-only REST endpoints
//!
//! These endpoints are only available in debug builds.

#![cfg(debug_assertions)]

use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

use crate::error::{GatewayError, GatewayResult};
use crate::rest::models::SessionResponse;
use crate::state::GatewayState;

/// Create development-only routes
pub fn create_dev_routes() -> Router<Arc<GatewayState>> {
    Router::new().route("/auth/dev/token", get(dev_token))
}

/// Development endpoint to create a test token
#[utoipa::path(
    get,
    path = "/api/v1/auth/dev/token",
    tag = "Auth",
    responses(
        (status = 200, description = "Development session issued", body = SessionResponse),
        (status = 500, description = "Failed to create development session")
    )
)]
pub async fn dev_token(
    State(state): State<Arc<GatewayState>>,
) -> GatewayResult<Json<SessionResponse>> {
    let (session, user) = state
        .authenticator()
        .create_dev_session()
        .await
        .map_err(|e| GatewayError::InternalError(format!("Failed to create dev token: {}", e)))?;

    Ok(Json(SessionResponse::new(session, user)))
}
