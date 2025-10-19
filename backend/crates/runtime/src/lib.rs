use std::sync::Arc;

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use switchboard_auth::Authenticator;
use switchboard_config::AppConfig;
use switchboard_database::{initialize_database};
use sqlx::SqlitePool;
use switchboard_orchestrator::Orchestrator;
use tracing::info;


pub mod telemetry {
    use anyhow::Result;
    use tracing::Level;
    use tracing_subscriber::{fmt::SubscriberBuilder, EnvFilter};

    pub fn init_tracing() -> Result<()> {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        let subscriber = SubscriberBuilder::default()
            .with_max_level(Level::INFO)
            .with_env_filter(env_filter)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .map_err(|error| anyhow::anyhow!("failed to set tracing subscriber: {error}"))
    }
}

#[derive(Clone)]
pub struct BackendServices {
    pub db_pool: SqlitePool,
    pub authenticator: Authenticator,
    pub orchestrator: Arc<Orchestrator>,
    pub redis_conn: Option<ConnectionManager>,
}

impl BackendServices {
    pub async fn initialise(config: &AppConfig) -> Result<Self> {
        let db_pool = initialize_database(&config.database).await?;

        let authenticator = Authenticator::new(db_pool.clone(), config.auth.clone());
        let orchestrator = Arc::new(
            Orchestrator::new(config)
                .bootstrap()
                .context("failed to bootstrap orchestrator")?,
        );

        // Initialize Redis connection (optional for development)
        let redis_conn = match redis::Client::open("redis://127.0.0.1:6379") {
            Ok(client) => match ConnectionManager::new(client).await {
                Ok(conn) => {
                    info!("redis connection established");
                    Some(conn)
                }
                Err(e) => {
                    tracing::warn!(
                        "failed to connect to redis, proceeding without redis: {}",
                        e
                    );
                    None
                }
            },
            Err(e) => {
                tracing::warn!(
                    "failed to create redis client, proceeding without redis: {}",
                    e
                );
                None
            }
        };

        info!(model = ?orchestrator.active_model(), "orchestrator ready");
        info!("redis connection established");

        Ok(Self {
            db_pool,
            authenticator,
            orchestrator,
            redis_conn,
        })
    }
}

pub async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(?error, "failed to listen for shutdown signal");
    }
    info!("shutdown signal received");
}
