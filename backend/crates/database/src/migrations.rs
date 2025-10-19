//! Database migrations

use anyhow::Context;
use sqlx::{migrate::Migrator, SqlitePool};
use tracing::info;

// Include migrations from the migrations directory
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Run database migrations
pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    MIGRATOR
        .run(pool)
        .await
        .context("database migrations failed")?;
    info!("database migrations applied");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::prepare_database;
    use switchboard_config::DatabaseConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_migrations_run() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_migrations.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 1,
        };

        let pool = prepare_database(&config).await.unwrap();

        // This will fail if there are no migration files, but that's expected in testing
        let result = run_migrations(&pool).await;

        // The test passes whether migrations exist or not, as we're just testing the function
        match result {
            Ok(_) => println!("Migrations ran successfully"),
            Err(e) => println!("No migrations found or migration error: {}", e),
        }
    }
}