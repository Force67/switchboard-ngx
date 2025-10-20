use std::{env, fs, fs::File, path::Path, time::Duration};

use anyhow::{Context, Result};
use sqlx::Row;
use switchboard_backend_runtime::{self, BackendServices};
use switchboard_config::AppConfig;
use tempfile::TempDir;
use tokio::{
    net::TcpListener,
    time::{sleep, timeout},
};

fn sqlite_url(path: &Path) -> String {
    format!("sqlite://{}", path.to_string_lossy())
}

fn build_config(database_url: String, max_connections: u32) -> AppConfig {
    let mut config = AppConfig::default();
    config.database.url = database_url;
    config.database.max_connections = max_connections;
    config.orchestrator.provider_search_path = Vec::new();
    config.orchestrator.openrouter.api_key = Some("unit-test-key".into());
    config
}

async fn initialise(config: &AppConfig) -> Result<BackendServices> {
    BackendServices::initialise(config)
        .await
        .context("failed to initialise backend services")
}

#[tokio::test(flavor = "multi_thread")]
async fn initialise_runs_migrations_and_bootstraps_orchestrator() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("runtime/init.db");
    let config = build_config(sqlite_url(&db_path), 4);

    let services = initialise(&config).await?;
    let table: String = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'users'",
    )
    .fetch_one(&services.db_pool)
    .await?;

    assert_eq!(
        Some(config.orchestrator.default_model.clone()),
        services.orchestrator.active_model()
    );
    assert_eq!("users", table);

    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn initialise_handles_orchestrator_bootstrap_failures() -> Result<()> {
    env::remove_var("OPENROUTER_API_KEY");
    let mut config = build_config("sqlite://:memory:".into(), 2);
    config.orchestrator.openrouter.api_key = None;

    let error = match BackendServices::initialise(&config).await {
        Ok(_) => panic!("expected orchestrator bootstrap to fail without API key"),
        Err(error) => error,
    };
    let message = format!("{error:?}");
    assert!(
        message.contains("failed to bootstrap orchestrator"),
        "expected orchestrator bootstrap failure context, got {message}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn initialise_logs_and_ignores_redis_connection_failure() -> Result<()> {
    let dummy_listener = match TcpListener::bind("127.0.0.1:6379").await {
        Ok(listener) => Some(tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => drop(stream),
                    Err(_) => break,
                }
            }
        })),
        Err(_) => None,
    };

    if dummy_listener.is_none() {
        // The environment already has Redis bound; nothing to assert about the failure path.
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("runtime/redis.db");
    let config = build_config(sqlite_url(&db_path), 1);

    let services = initialise(&config).await?;
    if let Some(handle) = &dummy_listener {
        handle.abort();
    }

    assert!(
        services.redis_conn.is_none(),
        "redis connection errors should be tolerated"
    );
    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn prepare_database_creates_sqlite_directory_if_missing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_dir = temp_dir.path().join("nested");
    let db_path = db_dir.join("prepared.db");
    let config = build_config(sqlite_url(&db_path), 2);

    assert!(!db_dir.exists());

    let services = initialise(&config).await?;
    assert!(db_dir.exists(), "database directory should be created");
    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn prepare_database_enables_sqlite_foreign_keys() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("runtime/foreign_keys.db");
    let config = build_config(sqlite_url(&db_path), 2);

    let services = initialise(&config).await?;

    let enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&services.db_pool)
        .await?;
    assert_eq!(1, enabled, "foreign key enforcement must be enabled");

    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn prepare_database_applies_max_connections_setting() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("runtime/max_conn.db");
    let max_connections = 3;
    let config = build_config(sqlite_url(&db_path), max_connections);

    let services = initialise(&config).await?;
    assert_eq!(
        max_connections,
        services.db_pool.options().get_max_connections()
    );

    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn ensure_sqlite_path_creates_file_when_missing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("ensure/missing.db");
    let config = build_config(sqlite_url(&db_path), 1);

    assert!(!db_path.exists());

    let services = initialise(&config).await?;
    assert!(
        db_path.exists(),
        "sqlite database file should be created when missing"
    );
    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn ensure_sqlite_path_noops_for_memory_database() -> Result<()> {
    let config = build_config("sqlite://:memory:".into(), 1);
    let services = initialise(&config).await?;

    let databases = sqlx::query("PRAGMA database_list")
        .fetch_all(&services.db_pool)
        .await?;
    let main_db = databases
        .into_iter()
        .find(|row| {
            row.try_get::<String, _>("name")
                .map(|name| name == "main")
                .unwrap_or(false)
        })
        .context("expected main in PRAGMA database_list")?;
    let file: String = main_db.try_get("file")?;
    assert!(
        file.is_empty(),
        "in-memory sqlite database should not create filesystem entries"
    );

    drop(services);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn ensure_sqlite_path_ignores_non_sqlite_urls() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let target_dir = temp_dir.path().join("should_not_exist");
    let malformed_url = format!("postgres://{}/ignored.db", target_dir.to_string_lossy());
    let config = build_config(malformed_url, 1);

    assert!(!target_dir.exists());

    let error = match BackendServices::initialise(&config).await {
        Ok(_) => panic!("expected sqlite connection to fail for non-sqlite URL"),
        Err(error) => error,
    };
    assert!(
        !target_dir.exists(),
        "non-sqlite URLs must not create filesystem structures"
    );
    assert!(
        error.to_string().contains("failed to connect to database"),
        "expected database connection failure for malformed sqlite URL"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn run_migrations_propagates_failures() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("readonly.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(&db_path)?;

    let mut perms = fs::metadata(&db_path)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&db_path, perms)?;

    let config = build_config(sqlite_url(&db_path), 1);
    let error = match BackendServices::initialise(&config).await {
        Ok(_) => panic!("expected migrations to fail on read-only database"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("database migrations failed"),
        "migration errors should propagate with context"
    );

    Ok(())
}

#[test]
fn telemetry_init_tracing_sets_global_subscriber() {
    switchboard_backend_runtime::telemetry::init_tracing()
        .expect("first initialisation should succeed");

    let second = switchboard_backend_runtime::telemetry::init_tracing();
    assert!(
        second.is_err(),
        "initialising telemetry twice should fail with global subscriber already set"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[cfg_attr(not(unix), ignore = "requires Unix signal handling")]
async fn shutdown_signal_completes_on_ctrl_c_notification() -> Result<()> {
    let shutdown_task =
        tokio::spawn(async { switchboard_backend_runtime::shutdown_signal().await });

    sleep(Duration::from_millis(50)).await;
    #[cfg(unix)]
    unsafe {
        libc::raise(libc::SIGINT);
    }

    timeout(Duration::from_secs(2), shutdown_task).await??;
    Ok(())
}
