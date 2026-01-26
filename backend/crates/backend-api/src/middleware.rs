//! Middleware for authentication and other cross-cutting concerns

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::error::{GatewayError, GatewayResult};
use crate::state::GatewayState;

const DEFAULT_ALLOWED_ORIGINS: &[&str] = &["http://localhost:3000", "http://localhost:5173"];
const ALLOWED_ORIGINS_ENV: &str = "SWITCHBOARD_ALLOWED_ORIGINS";
const DEV_AUTH_FALLBACK_ENV: &str = "SWITCHBOARD_DEV_AUTH_FALLBACK";
const PUBLIC_ENDPOINTS: &[&str] = &[
    "/api/v1/health",
    "/api/v1/auth/github/login",
    "/api/v1/auth/github/callback",
];

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(state): State<Arc<GatewayState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, GatewayError> {
    let path = request.uri().path().to_string();

    if request.method() == Method::OPTIONS || is_public_endpoint(&path) {
        return Ok(next.run(request).await);
    }

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

    // Only accept token via query parameters for WebSocket handshakes.
    //
    // Using query tokens on normal HTTP routes increases accidental leakage risk (logs,
    // referers, analytics), and browsers can set Authorization headers for XHR/fetch.
    let query_token = if path.starts_with("/ws/") || path == "/ws" {
        request.uri().query().and_then(|query| {
            urlencoding::decode(query).ok().and_then(|decoded| {
                decoded.split('&').find_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    match (parts.next(), parts.next()) {
                        (Some("token"), Some(value)) => Some(value.to_string()),
                        _ => None,
                    }
                })
            })
        })
    } else {
        None
    };

    let token = auth_header.or(query_token.as_deref());

    // For local development, optionally mint a dev token when none is provided
    if token.is_none() && dev_auth_fallback_enabled() {
        return issue_dev_token_and_continue(request, state, next).await;
    }

    // For development endpoints, allow access without token
    if is_dev_endpoint(&path) {
        let Ok((user_id, token)) = get_dev_user(&state).await else {
            return Err(GatewayError::AuthenticationFailed(
                "Failed to create dev user".to_string(),
            ));
        };

        request.extensions_mut().insert(user_id);
        request.extensions_mut().insert(token);
        return Ok(next.run(request).await);
    }

    let token = token.ok_or_else(|| {
        GatewayError::AuthenticationFailed("Missing authentication token".to_string())
    })?;

    let auth_result = state.authenticator().authenticate_token(token).await;

    let (user, session) = match auth_result {
        Ok(result) => result,
        Err(e) if dev_auth_fallback_enabled() => {
            return issue_dev_token_and_continue(request, state, next).await;
        }
        Err(e) => {
            return Err(GatewayError::AuthenticationFailed(format!(
                "Invalid token: {}",
                e
            )))
        }
    };

    request.extensions_mut().insert(user.id);
    request.extensions_mut().insert(session.token.clone());

    Ok(next.run(request).await)
}

/// Check if the endpoint is a development endpoint that doesn't require authentication
fn is_dev_endpoint(path: &str) -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }

    path.contains("/dev/")
        || path.starts_with("/swagger-ui")
        || path == "/api-docs/openapi.json"
        || path == "/api/v1/auth/dev/token"
}

fn is_public_endpoint(path: &str) -> bool {
    if PUBLIC_ENDPOINTS.contains(&path) {
        return true;
    }

    cfg!(debug_assertions) && path == "/api/v1/auth/dev/token"
}

fn dev_auth_fallback_enabled() -> bool {
    match std::env::var(DEV_AUTH_FALLBACK_ENV) {
        Ok(val) => {
            if cfg!(debug_assertions) {
                val != "false" && val != "0"
            } else {
                // In production builds, require an explicit true/1 to avoid accidental enablement.
                val == "true" || val == "1"
            }
        }
        Err(_) => cfg!(debug_assertions),
    }
}

async fn issue_dev_token_and_continue(
    mut request: Request,
    state: Arc<GatewayState>,
    next: Next,
) -> Result<Response, GatewayError> {
    let Ok((user_id, token)) = get_dev_user(&state).await else {
        return Err(GatewayError::AuthenticationFailed(
            "Failed to create dev user".to_string(),
        ));
    };

    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(token);
    Ok(next.run(request).await)
}

/// Get or create a development user for development endpoints
async fn get_dev_user(state: &GatewayState) -> GatewayResult<(i64, String)> {
    let (session, user) = state
        .authenticator()
        .create_dev_session()
        .await
        .map_err(|e| GatewayError::InternalError(format!("Failed to create dev token: {}", e)))?;

    Ok((user.id, session.token))
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
        if let Ok((user, session)) = state.authenticator().authenticate_token(token).await {
            request.extensions_mut().insert(user.id);
            request.extensions_mut().insert(session.token);
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
pub fn create_trace_middleware(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
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
    // Avoid logging query strings since we accept auth tokens via query params
    // (notably for WebSocket authentication).
    let path = request.uri().path().to_string();

    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();

    tracing::info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    Ok(response)
}

/// Rate limiting middleware (placeholder implementation)
pub async fn rate_limit_middleware(request: Request, next: Next) -> Result<Response, GatewayError> {
    // TODO: Implement proper rate limiting using something like redis or in-memory storage
    // For now, just pass through
    Ok(next.run(request).await)
}

fn resolve_allowed_origins() -> Vec<HeaderValue> {
    let configured_origins = std::env::var(ALLOWED_ORIGINS_ENV)
        .ok()
        .and_then(|value| {
            let origins: Vec<String> = value
                .split(',')
                .map(|origin| origin.trim().to_string())
                .filter(|origin| !origin.is_empty())
                .collect();

            if origins.is_empty() {
                None
            } else {
                Some(origins)
            }
        })
        .unwrap_or_else(|| {
            DEFAULT_ALLOWED_ORIGINS
                .iter()
                .map(|origin| origin.to_string())
                .collect()
        });

    let parsed_origins: Vec<HeaderValue> = configured_origins
        .into_iter()
        .filter_map(|origin| HeaderValue::from_str(&origin).ok())
        .collect();

    if parsed_origins.is_empty() {
        DEFAULT_ALLOWED_ORIGINS
            .iter()
            .filter_map(|origin| HeaderValue::from_str(origin).ok())
            .collect()
    } else {
        parsed_origins
    }
}

/// CORS middleware for cross-origin requests
pub fn create_cors_middleware() -> tower_http::cors::CorsLayer {
    let allowed_origins = resolve_allowed_origins();

    tower_http::cors::CorsLayer::new()
        .allow_origin(allowed_origins)
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

/// Dev-only CORS that allows any `localhost` port to access the API from a browser.
///
/// This is intentionally only compiled for debug builds.
#[cfg(debug_assertions)]
pub fn create_dev_cors_middleware() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _parts| {
            let Ok(origin_str) = origin.to_str() else {
                return false;
            };

            // Allow typical localhost variants with any port.
            is_localhost_origin(origin_str)
        }))
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any())
        .allow_credentials(true)
}

#[cfg(debug_assertions)]
fn is_localhost_origin(origin: &str) -> bool {
    fn host_matches(origin: &str, host: &str) -> bool {
        let Some(rest) = origin.strip_prefix("http://") else {
            return false;
        };
        let Some(after_host) = rest.strip_prefix(host) else {
            return false;
        };
        after_host.is_empty() || after_host.starts_with(':')
    }

    host_matches(origin, "localhost")
        || host_matches(origin, "127.0.0.1")
        || host_matches(origin, "[::1]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dev_endpoint() {
        assert!(is_dev_endpoint("/api/v1/auth/dev/token"));
        assert!(is_dev_endpoint("/swagger-ui"));
        assert!(is_dev_endpoint("/api-docs/openapi.json"));
        assert!(!is_dev_endpoint("/api/v1/auth/me"));
        assert!(!is_dev_endpoint("/api/v1/chats"));
    }
}
