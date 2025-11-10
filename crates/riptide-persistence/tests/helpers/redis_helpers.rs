//! Redis testcontainer helpers
//!
//! Provides utilities for:
//! - Starting Redis containers
//! - Connection management
//! - Cleanup utilities

use redis::{aio::MultiplexedConnection, Client};
use std::time::Duration;
use testcontainers::{clients::Cli, Container};
use testcontainers_modules::redis::Redis as RedisImage;

/// Redis test container wrapper
pub struct RedisTestContainer<'a> {
    #[allow(dead_code)]
    container: Container<'a, RedisImage>,
    #[allow(dead_code)]
    pub connection_string: String,
    pub client: Client,
}

impl<'a> RedisTestContainer<'a> {
    /// Create a new Redis test container
    pub async fn new(docker: &'a Cli) -> Result<Self, anyhow::Error> {
        // Start Redis container
        let redis_image = RedisImage;
        let container = docker.run(redis_image);
        let port = container.get_host_port_ipv4(6379);

        // Build connection string
        let connection_string = format!("redis://127.0.0.1:{}", port);

        // Create Redis client
        let client = Client::open(connection_string.clone())?;

        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;

        Ok(Self {
            container,
            connection_string,
            client,
        })
    }

    /// Get a multiplexed async connection
    pub async fn get_connection(&self) -> Result<MultiplexedConnection, anyhow::Error> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    /// Clean up test data by pattern
    pub async fn cleanup_pattern(&self, pattern: &str) -> Result<u64, anyhow::Error> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;

        // Get all keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let deleted: u64 = conn.del(&keys).await?;

        Ok(deleted)
    }

    /// Flush all data (use with caution)
    #[allow(dead_code)]
    pub async fn flush_all(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.get_connection().await?;
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await?;
        Ok(())
    }

    /// Get connection string
    #[allow(dead_code)]
    pub fn get_connection_string(&self) -> &str {
        &self.connection_string
    }

    /// Wait for Redis to be ready
    #[allow(dead_code)]
    pub async fn wait_until_ready(&self, timeout: Duration) -> Result<(), anyhow::Error> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!(
                    "Redis container did not become ready in time"
                ));
            }

            match self.get_connection().await {
                Ok(mut conn) => match redis::cmd("PING").query_async::<String>(&mut conn).await {
                    Ok(_) => return Ok(()),
                    Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                },
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_container_setup() {
        let docker = Cli::default();
        let container = RedisTestContainer::new(&docker).await;
        assert!(container.is_ok());

        if let Ok(container) = container {
            let conn = container.get_connection().await;
            assert!(conn.is_ok());
        }
    }

    #[tokio::test]
    async fn test_redis_cleanup() {
        use redis::AsyncCommands;
        let docker = Cli::default();
        let container = RedisTestContainer::new(&docker).await.unwrap();

        // Set some test keys
        let mut conn = container.get_connection().await.unwrap();
        let _: () = conn.set("test:key1", "value1").await.unwrap();

        // Cleanup
        let deleted = container.cleanup_pattern("test:*").await.unwrap();
        assert!(deleted >= 1);
    }
}
