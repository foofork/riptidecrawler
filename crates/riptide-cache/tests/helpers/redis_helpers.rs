//! Redis testcontainer helpers for cache integration tests

use redis::{aio::MultiplexedConnection, Client};
use std::time::Duration;
use testcontainers::{clients::Cli, Container};
use testcontainers_modules::redis::Redis as RedisImage;

/// Redis test container wrapper for cache tests
pub struct RedisTestContainer<'a> {
    #[allow(dead_code)]
    container: Container<'a, RedisImage>,
    pub connection_string: String,
    pub client: Client,
}

impl<'a> RedisTestContainer<'a> {
    /// Create a new Redis test container
    pub async fn new(docker: &'a Cli) -> Result<Self, anyhow::Error> {
        let redis_image = RedisImage::default();
        let container = docker.run(redis_image);
        let port = container.get_host_port_ipv4(6379);

        let connection_string = format!("redis://127.0.0.1:{}", port);
        let client = Client::open(connection_string.clone())?;

        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await?;

        Ok(Self {
            container,
            connection_string,
            client,
        })
    }

    /// Get a multiplexed async connection
    pub async fn get_connection(
        &self,
    ) -> Result<MultiplexedConnection, anyhow::Error> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }

    /// Clean up test data by pattern
    pub async fn cleanup_pattern(&self, pattern: &str) -> Result<u64, anyhow::Error> {
        use redis::AsyncCommands;
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await?;

        Ok(deleted)
    }

    /// Flush all data
    pub async fn flush_all(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.get_connection().await?;
        redis::cmd("FLUSHALL").query_async(&mut conn).await?;
        Ok(())
    }

    /// Get connection string
    pub fn get_connection_string(&self) -> &str {
        &self.connection_string
    }

    /// Wait for Redis to be ready
    pub async fn wait_until_ready(&self, timeout: Duration) -> Result<(), anyhow::Error> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow::anyhow!("Redis container did not become ready in time"));
            }

            match self.get_connection().await {
                Ok(mut conn) => {
                    match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                        Ok(_) => return Ok(()),
                        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                    }
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }
}
