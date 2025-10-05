use anyhow::Context;
use sqlx::any::{install_default_drivers, AnyPoolOptions};
use std::path::Path;
use tokio::fs;
use switchboard_auth::Authenticator;
use switchboard_config::load as load_config;
use switchboard_orchestrator::Orchestrator;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod migrations {
    pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_env_filter(env_filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("failed to set tracing subscriber")?;

    info!("starting Switchboard backend");

    let config = load_config().context("failed to load configuration")?;

    // Ensure SQLx Any can talk to SQLite/Postgres before we connect.
    install_default_drivers();

    if let Some(sqlite_path) = config.database.url.strip_prefix("sqlite://") {
        if sqlite_path != ":memory:" {
            let path = Path::new(sqlite_path);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .await
                        .with_context(|| {
                            format!("failed to create sqlite directory {}", parent.display())
                        })?;
                }
            }

            if !path.exists() {
                fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(path)
                    .await
                    .with_context(|| format!("failed to create sqlite database file {}", path.display()))?;
            }
        }
    }

    let db_pool = AnyPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .with_context(|| format!("failed to connect to database {}", config.database.url))?;

    if config.database.url.starts_with("sqlite://") {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&db_pool)
            .await
            .context("failed to enable foreign keys for sqlite")?;
    }

    migrations::MIGRATOR
        .run(&db_pool)
        .await
        .context("database migrations failed")?;

    let authenticator = Authenticator::new(db_pool.clone(), config.auth.clone());
    info!(github_oauth = authenticator.github_enabled(), "authentication subsystem ready");

    let orchestrator = Orchestrator::new(&config)
        .bootstrap()
        .context("failed to bootstrap orchestrator")?;

    info!(model = ?orchestrator.active_model(), "backend initialised");
    info!("awaiting shutdown signal");

    signal::ctrl_c()
        .await
        .context("failed to listen for shutdown signal")?;

    info!("shutdown signal received");
    Ok(())
}
