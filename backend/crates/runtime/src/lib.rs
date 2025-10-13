use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use switchboard_auth::Authenticator;
use switchboard_config::{AppConfig, DatabaseConfig};
use switchboard_orchestrator::Orchestrator;
use tokio::fs;
use tracing::{error, info};

mod migrations {
    pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");
}

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
        let db_pool = prepare_database(&config.database).await?;
        run_migrations(&db_pool).await?;

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

async fn prepare_database(config: &DatabaseConfig) -> Result<SqlitePool> {
    ensure_sqlite_path(&config.url).await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections as u32)
        .connect(&config.url)
        .await
        .with_context(|| format!("failed to connect to database {}", config.url))?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .context("failed to enable foreign keys for sqlite")?;

    info!(url = %config.url, "database connection established");
    Ok(pool)
}

async fn ensure_sqlite_path(url: &str) -> Result<()> {
    let Some(sqlite_path) = url.strip_prefix("sqlite://") else {
        return Ok(());
    };

    if sqlite_path == ":memory:" {
        return Ok(());
    }

    let path = Path::new(sqlite_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("failed to create sqlite directory {}", parent.display())
            })?;
        }
    }

    if fs::metadata(path).await.is_err() {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .await
            .with_context(|| format!("failed to create sqlite database file {}", path.display()))?;
    }

    Ok(())
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    migrations::MIGRATOR
        .run(pool)
        .await
        .context("database migrations failed")?;
    info!("database migrations applied");
    Ok(())
}

pub async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(?error, "failed to listen for shutdown signal");
    }
    info!("shutdown signal received");
}
