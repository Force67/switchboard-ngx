//! Middleware for authentication and other cross-cutting concerns

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_http::trace::{TraceLayer, DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::Level;
use std::sync::Arc;

use crate::state::GatewayState;
use crate::error::{GatewayError, GatewayResult};

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(state): State<Arc<GatewayState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, GatewayError> {
    // Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    // Check for token in query parameters (for WebSocket connections)
    let query_token = request
        .uri()
        .query()
        .and_then(|query| {
            urlencoding::decode(query).ok()
                .and_then(|decoded| {
                    decoded.split('&')
                        .find_map(|pair| {
                            let mut parts = pair.splitn(2, '=');
                            match (parts.next(), parts.next()) {
                                (Some("token"), Some(value)) => Some(value.to_string()),
                                _ => None,
                            }
                        })
                })
        });

    let token = auth_header.or(query_token.as_deref());

    // For development endpoints, allow access without token
    if is_dev_endpoint(request.uri().path()) {
        let Ok(user_id) = get_dev_user_id(&state).await else {
            return Err(GatewayError::AuthenticationFailed("Failed to create dev user".to_string()));
        };

        // Add user ID to request extensions
        request.extensions_mut().insert(user_id);
        return Ok(next.run(request).await);
    }

    let token = token.ok_or_else(|| {
        GatewayError::AuthenticationFailed("Missing authentication token".to_string())
    })?;

    // Validate token
    let session = state
        .session_service()
        .validate_session(token)
        .await
        .map_err(|e| GatewayError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

    // Add user ID to request extensions
    request.extensions_mut().insert(session.user_id);

    Ok(next.run(request).await)
}

/// Check if the endpoint is a development endpoint that doesn't require authentication
fn is_dev_endpoint(path: &str) -> bool {
    path.contains("/dev/") || path.starts_with("/swagger-ui") || path == "/api-docs/openapi.json"
}

/// Get or create a development user for development endpoints
async fn get_dev_user_id(state: &GatewayState) -> GatewayResult<i64> {
    // Try to create a dev token, which will also create a dev user if needed
    let (session, _user) = state
        .session_service()
        .create_dev_token()
        .await
        .map_err(|e| GatewayError::InternalError(format!("Failed to create dev token: {}", e)))?;

    Ok(session.user_id)
}

/// Optional authentication middleware that allows unauthenticated access
/// but adds user ID to request extensions if token is present
pub async fn optional_auth_middleware(
    State(state): State<Arc<GatewayState>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Try to extract and validate token, but don't fail if it's missing
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    if let Some(token) = auth_header {
        if let Ok(session) = state.session_service().validate_session(token).await {
            request.extensions_mut().insert(session.user_id);
        }
    }

    next.run(request).await
}

/// Extract user ID from request extensions
pub fn extract_user_id(request: &Request) -> GatewayResult<i64> {
    request
        .extensions()
        .get::<i64>()
        .copied()
        .ok_or_else(|| GatewayError::AuthenticationFailed("User not authenticated".to_string()))
}

/// Create tracing middleware
pub fn create_trace_middleware() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// Logging middleware for request/response logging
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    Ok(response)
}

/// Rate limiting middleware (placeholder implementation)
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, GatewayError> {
    // TODO: Implement proper rate limiting using something like redis or in-memory storage
    // For now, just pass through
    Ok(next.run(request).await)
}

/// CORS middleware for cross-origin requests
pub fn create_cors_middleware() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(), // Vite dev server
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dev_endpoint() {
        assert!(is_dev_endpoint("/api/auth/dev/token"));
        assert!(is_dev_endpoint("/swagger-ui"));
        assert!(is_dev_endpoint("/api-docs/openapi.json"));
        assert!(!is_dev_endpoint("/api/auth/me"));
        assert!(!is_dev_endpoint("/api/chats"));
    }
}