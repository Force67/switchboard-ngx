mod error;
mod state;
mod util;

pub mod routes;

pub use error::ApiError;
pub use state::{AppState, OAuthStateStore};

use axum::{http::header::{AUTHORIZATION, CONTENT_TYPE}, routing::{get, post}, Router};
use tower_http::cors::{Any, CorsLayer};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/api/auth/github/login", get(routes::auth::github_login))
        .route("/api/auth/github/callback", post(routes::auth::github_callback))
        .route("/api/models", get(routes::models::list_models))
        .route("/api/chat", post(routes::chat::chat_completion))
        .with_state(state)
        .layer(cors_layer())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
}
