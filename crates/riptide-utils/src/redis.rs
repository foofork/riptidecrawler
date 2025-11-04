//! Redis connection pool management with health checks

use redis::{aio::MultiplexedConnection, Client, RedisError};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Configuration for Redis connection pool
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            health_check_interval_secs: 30,
        }
    }
}

/// Redis connection pool with health check capabilities
#[derive(Clone)]
pub struct RedisPool {
    manager: MultiplexedConnection,
    config: RedisConfig,
}

impl RedisPool {
    /// Creates a new Redis connection pool
    ///
    /// # Arguments
    ///
    /// * `config` - Redis configuration
    ///
    /// # Errors
    ///
    /// Returns error if connection to Redis fails
    pub async fn new(config: RedisConfig) -> Result<Self, RedisError> {
        info!(
            "Initializing Redis connection pool with URL: {}",
            config.url
        );

        let client = Client::open(config.url.clone())?;
        let manager = client.get_multiplexed_async_connection().await?;

        debug!("Redis connection pool initialized successfully");

        Ok(Self { manager, config })
    }

    /// Gets a connection from the pool
    pub fn get_connection(&self) -> MultiplexedConnection {
        self.manager.clone()
    }

    /// Performs a health check on the Redis connection
    ///
    /// # Errors
    ///
    /// Returns error if health check fails
    pub async fn health_check(&mut self) -> Result<(), RedisError> {
        let timeout_duration = Duration::from_millis(self.config.timeout_ms);

        debug!("Performing Redis health check");

        match timeout(
            timeout_duration,
            redis::cmd("PING").query_async::<String>(&mut self.manager),
        )
        .await
        {
            Ok(Ok(response)) => {
                if response == "PONG" {
                    debug!("Redis health check passed");
                    Ok(())
                } else {
                    warn!(
                        "Redis health check returned unexpected response: {}",
                        response
                    );
                    Err(RedisError::from((
                        redis::ErrorKind::IoError,
                        "Unexpected PING response",
                    )))
                }
            }
            Ok(Err(e)) => {
                error!("Redis health check failed: {}", e);
                Err(e)
            }
            Err(_) => {
                error!(
                    "Redis health check timed out after {}ms",
                    self.config.timeout_ms
                );
                Err(RedisError::from((
                    redis::ErrorKind::IoError,
                    "Health check timeout",
                )))
            }
        }
    }

    /// Gets the current configuration
    pub fn config(&self) -> &RedisConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.health_check_interval_secs, 30);
    }

    #[tokio::test]
    async fn test_redis_config_custom() {
        let config = RedisConfig {
            url: "redis://custom:6380".to_string(),
            timeout_ms: 3000,
            max_retries: 5,
            health_check_interval_secs: 60,
        };

        assert_eq!(config.url, "redis://custom:6380");
        assert_eq!(config.timeout_ms, 3000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.health_check_interval_secs, 60);
    }

    // Note: Integration tests requiring actual Redis instance
    // should be placed in tests/ directory
}
