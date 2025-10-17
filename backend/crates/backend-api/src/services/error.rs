#[derive(Debug)]
pub enum ServiceError {
    NotFound,
    Forbidden,
    BadRequest(String),
    Database(sqlx::Error),
    Auth(switchboard_auth::AuthError),
    Config(String),
    Internal(String),
}

impl ServiceError {
    pub fn not_found(_msg: impl Into<String>) -> Self {
        Self::NotFound
    }

    pub fn forbidden(_msg: impl Into<String>) -> Self {
        Self::Forbidden
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
}

impl From<crate::ApiError> for ServiceError {
    fn from(err: crate::ApiError) -> Self {
        match err.status {
            axum::http::StatusCode::NOT_FOUND => Self::NotFound,
            axum::http::StatusCode::FORBIDDEN => Self::Forbidden,
            axum::http::StatusCode::BAD_REQUEST => Self::BadRequest(err.message.clone()),
            _ => Self::Internal(err.message.clone()),
        }
    }
}

impl From<ServiceError> for crate::ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound => crate::ApiError::not_found("Resource not found"),
            ServiceError::Forbidden => crate::ApiError::forbidden("Access denied"),
            ServiceError::BadRequest(msg) => crate::ApiError::bad_request(&msg),
            ServiceError::Database(db_err) => {
                tracing::error!("Database error: {}", db_err);
                crate::ApiError::internal_server_error("Database operation failed")
            }
            ServiceError::Auth(auth_err) => {
                tracing::error!("Authentication error: {}", auth_err);
                crate::ApiError::from(auth_err)
            }
            ServiceError::Config(msg) => {
                tracing::error!("Configuration error: {}", msg);
                crate::ApiError::new(axum::http::StatusCode::SERVICE_UNAVAILABLE, &msg)
            }
            ServiceError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                crate::ApiError::internal_server_error(&msg)
            }
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl From<switchboard_auth::AuthError> for ServiceError {
    fn from(err: switchboard_auth::AuthError) -> Self {
        Self::Auth(err)
    }
}