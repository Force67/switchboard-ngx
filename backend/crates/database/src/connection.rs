//! Database connection management

use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use tokio::fs;
use tracing::info;
use switchboard_config::DatabaseConfig;

/// Prepare and establish a database connection
pub async fn prepare_database(config: &DatabaseConfig) -> Result<SqlitePool> {
    ensure_sqlite_path(&config.url).await?;

    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections as u32)
        .connect(&config.url)
        .await
        .with_context(|| format!("failed to connect to database {}", config.url))?;

    // Enable foreign keys for SQLite
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .context("failed to enable foreign keys for sqlite")?;

    // Enable WAL mode for better performance
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await
        .context("failed to enable WAL mode for sqlite")?;

    // Set busy timeout to prevent database locked errors
    sqlx::query("PRAGMA busy_timeout = 5000")
        .execute(&pool)
        .await
        .context("failed to set busy timeout for sqlite")?;

    info!(url = %config.url, "database connection established");
    Ok(pool)
}

/// Ensure the SQLite database file and directory exist
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

/// Database connection wrapper for easier management
#[derive(Clone)]
pub struct DatabaseConnection {
    pub pool: SqlitePool,
}

impl DatabaseConnection {
    /// Create a new database connection from configuration
    pub async fn from_config(config: &DatabaseConfig) -> Result<Self> {
        let pool = prepare_database(config).await?;
        Ok(Self { pool })
    }

    /// Create a new database connection from an existing pool
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection pool
    pub async fn close(self) {
        self.pool.close().await;
    }

    /// Test the database connection
    pub async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .context("failed to test database connection")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_connection_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db_url = format!("sqlite:{}", db_path.display());

        let config = DatabaseConfig {
            url: db_url,
            max_connections: 1,
        };

        let conn = DatabaseConnection::from_config(&config).await.unwrap();
        conn.test_connection().await.unwrap();
    }

    #[tokio::test]
    async fn test_in_memory_database() {
        let config = DatabaseConfig {
            url: "sqlite://:memory:".to_string(),
            max_connections: 1,
        };

        let conn = DatabaseConnection::from_config(&config).await.unwrap();
        conn.test_connection().await.unwrap();
    }
}