//! Health service implementation

use crate::error::GatewayResult;
use crate::generated::rest::traits::HealthServiceTrait;
use crate::rest::models::HealthResponse;
use crate::state::GatewayState;
use chrono::Utc;

pub struct HealthServiceImpl<'a> {
    #[allow(dead_code)]
    state: &'a GatewayState,
}

impl<'a> HealthServiceImpl<'a> {
    pub fn new(state: &'a GatewayState) -> Self {
        Self { state }
    }
}

impl HealthServiceTrait for HealthServiceImpl<'_> {
    async fn health_check(&self) -> GatewayResult<HealthResponse> {
        Ok(HealthResponse {
            status: "ok".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }
}
