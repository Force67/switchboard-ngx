//! JWT (JSON Web Token) utilities for authentication.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::types::UserError;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub nbf: usize,         // Not before
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
    pub jti: String,        // JWT ID
    pub session_id: String, // Session identifier
    pub user_role: String,  // User role
}

/// JWT token manager
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    token_duration: Duration,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: &str, issuer: String, audience: String) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        Self {
            encoding_key,
            decoding_key,
            issuer,
            audience,
            token_duration: Duration::from_secs(24 * 60 * 60), // 24 hours default
        }
    }

    /// Set custom token duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.token_duration = duration;
        self
    }

    /// Generate a new JWT token
    pub fn generate_token(&self, user_id: &str, session_id: &str, user_role: &str) -> Result<String, UserError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| UserError::TokenCreationFailed("System time error".to_string()))?;

        let exp = now + self.token_duration;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.as_secs() as usize,
            iat: now.as_secs() as usize,
            nbf: now.as_secs() as usize,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            jti: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            user_role: user_role.to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|_| UserError::TokenCreationFailed("Failed to encode token".to_string()))
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, UserError> {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|err| UserError::InvalidToken(format!("Token validation failed: {}", err)))?;

        Ok(token_data.claims)
    }

    /// Refresh an existing token
    pub fn refresh_token(&self, token: &str) -> Result<String, UserError> {
        let claims = self.validate_token(token)?;

        // Check if token is still valid and not expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| UserError::TokenRefreshFailed("System time error".to_string()))?
            .as_secs() as usize;

        if claims.exp < now {
            return Err(UserError::TokenRefreshFailed("Token has expired".to_string()));
        }

        // Generate new token with same session info
        self.generate_token(&claims.sub, &claims.session_id, &claims.user_role)
    }

    /// Extract session ID from token without full validation (for performance)
    pub fn extract_session_id(&self, token: &str) -> Result<String, UserError> {
        // Decode without verification to get session ID quickly
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )
        .map_err(|_| UserError::InvalidToken("Failed to decode token".to_string()))?;

        Ok(token_data.claims.session_id)
    }
}

/// Generate a secure random session ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a CSRF token
pub fn generate_csrf_token() -> String {
    use rand::{distributions::Alphanumeric, Rng};

    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_jwt_manager() -> JwtManager {
        JwtManager::new(
            "test_secret_key_that_is_long_enough_for_hs256",
            "test_issuer".to_string(),
            "test_audience".to_string(),
        )
    }

    #[test]
    fn test_token_generation_and_validation() {
        let jwt_manager = create_test_jwt_manager();
        let user_id = "123";
        let session_id = "session_456";
        let user_role = "user";

        let token = jwt_manager.generate_token(user_id, session_id, user_role).unwrap();
        assert!(!token.is_empty());

        let claims = jwt_manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.session_id, session_id);
        assert_eq!(claims.user_role, user_role);
        assert_eq!(claims.iss, "test_issuer");
        assert_eq!(claims.aud, "test_audience");
    }

    #[test]
    fn test_invalid_token() {
        let jwt_manager = create_test_jwt_manager();
        let invalid_token = "invalid.jwt.token";

        let result = jwt_manager.validate_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_token_refresh() {
        let jwt_manager = create_test_jwt_manager();
        let user_id = "123";
        let session_id = "session_456";
        let user_role = "user";

        let original_token = jwt_manager.generate_token(user_id, session_id, user_role).unwrap();
        let refreshed_token = jwt_manager.refresh_token(&original_token).unwrap();

        assert_ne!(original_token, refreshed_token);

        let original_claims = jwt_manager.validate_token(&original_token).unwrap();
        let refreshed_claims = jwt_manager.validate_token(&refreshed_token).unwrap();

        assert_eq!(original_claims.sub, refreshed_claims.sub);
        assert_eq!(original_claims.session_id, refreshed_claims.session_id);
        assert_eq!(original_claims.user_role, refreshed_claims.user_role);
        assert!(refreshed_claims.exp > original_claims.exp);
    }

    #[test]
    fn test_session_id_extraction() {
        let jwt_manager = create_test_jwt_manager();
        let user_id = "123";
        let session_id = "session_456";
        let user_role = "user";

        let token = jwt_manager.generate_token(user_id, session_id, user_role).unwrap();
        let extracted_session_id = jwt_manager.extract_session_id(&token).unwrap();

        assert_eq!(extracted_session_id, session_id);
    }

    #[test]
    fn test_generate_session_id() {
        let session_id1 = generate_session_id();
        let session_id2 = generate_session_id();

        assert_ne!(session_id1, session_id2);
        assert_eq!(session_id1.len(), 36); // UUID length
    }

    #[test]
    fn test_generate_csrf_token() {
        let csrf_token1 = generate_csrf_token();
        let csrf_token2 = generate_csrf_token();

        assert_ne!(csrf_token1, csrf_token2);
        assert_eq!(csrf_token1.len(), 32);
    }
}