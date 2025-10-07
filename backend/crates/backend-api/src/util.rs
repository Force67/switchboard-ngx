use axum::http::{header::AUTHORIZATION, HeaderMap};

use crate::ApiError;

pub fn require_bearer(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized("missing authorization header"))?;

    let mut parts = value.split_whitespace();
    let scheme = parts.next().unwrap_or("");
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(ApiError::unauthorized("invalid authorization scheme"));
    }

    let token = parts.next().unwrap_or("");
    if token.is_empty() {
        return Err(ApiError::unauthorized("missing bearer token"));
    }

    Ok(token.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn require_bearer_extracts_token_case_insensitive() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("bearer TOKEN123"));

        let token = require_bearer(&headers).expect("token should be extracted");
        assert_eq!(token, "TOKEN123");
    }

    #[test]
    fn require_bearer_rejects_missing_token() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer"));

        let error = require_bearer(&headers).expect_err("should reject missing token");
        assert_eq!(error.status, axum::http::StatusCode::UNAUTHORIZED);
        assert!(error.message.contains("missing bearer token"));
    }
}
