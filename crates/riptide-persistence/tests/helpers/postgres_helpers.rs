//! PostgreSQL testcontainer helpers
//!
//! Provides utilities for:
//! - Starting PostgreSQL containers
//! - Schema initialization
//! - Cleanup utilities

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
use testcontainers::{clients::Cli, Container};
use testcontainers_modules::postgres::Postgres as PostgresImage;

/// PostgreSQL test container wrapper
pub struct PostgresTestContainer<'a> {
    #[allow(dead_code)]
    container: Container<'a, PostgresImage>,
    pub connection_string: String,
    pub pool: Pool<Postgres>,
}

impl<'a> PostgresTestContainer<'a> {
    /// Create a new PostgreSQL test container with schema initialization
    pub async fn new(docker: &'a Cli) -> Result<Self, anyhow::Error> {
        // Start PostgreSQL container
        let postgres_image = PostgresImage::default()
            .with_db_name("test_db")
            .with_user("test_user")
            .with_password("test_password");

        let container = docker.run(postgres_image);
        let port = container.get_host_port_ipv4(5432);

        // Build connection string
        let connection_string = format!(
            "postgres://test_user:test_password@127.0.0.1:{}/test_db",
            port
        );

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&connection_string)
            .await?;

        Ok(Self {
            container,
            connection_string,
            pool,
        })
    }

    /// Initialize database schema for session storage tests
    pub async fn init_session_schema(&self) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id VARCHAR(255) PRIMARY KEY,
                tenant_id VARCHAR(255),
                data JSONB NOT NULL,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_tenant ON sessions(tenant_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Initialize database schema for state persistence tests
    #[allow(dead_code)]
    pub async fn init_state_schema(&self) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id VARCHAR(255) PRIMARY KEY,
                checkpoint_type VARCHAR(50) NOT NULL,
                description TEXT,
                data JSONB NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_checkpoints_type ON checkpoints(checkpoint_type);
            CREATE INDEX IF NOT EXISTS idx_checkpoints_created ON checkpoints(created_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up all test data
    #[allow(dead_code)]
    pub async fn cleanup(&self) -> Result<(), anyhow::Error> {
        sqlx::query("TRUNCATE TABLE sessions CASCADE")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("TRUNCATE TABLE checkpoints CASCADE")
            .execute(&self.pool)
            .await
            .ok();
        Ok(())
    }

    /// Get connection pool
    #[allow(dead_code)]
    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Get connection string
    #[allow(dead_code)]
    pub fn get_connection_string(&self) -> &str {
        &self.connection_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_postgres_container_setup() {
        let docker = Cli::default();
        let container = PostgresTestContainer::new(&docker).await;
        assert!(container.is_ok());

        if let Ok(container) = container {
            let result = container.init_session_schema().await;
            assert!(result.is_ok());
        }
    }
}
