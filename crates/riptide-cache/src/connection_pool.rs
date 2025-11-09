//! Redis connection pool wrapper for external crates
//!
//! This module provides a simplified interface for external crates (like riptide-persistence)
//! to use Redis without directly depending on the redis crate. It bridges the CacheStorage
//! trait with low-level Redis operations needed for advanced features.

use redis::aio::MultiplexedConnection;
use redis::Client;
use riptide_types::error::{Result as RiptideResult, RiptideError};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error};

/// Redis connection pool that can be shared across crates
///
/// This wrapper allows external crates to use Redis functionality
/// without adding direct redis dependencies. It's designed for
/// advanced use cases that need connection pooling or pipelining.
pub struct RedisConnectionPool {
    client: Client,
    connections: Arc<Mutex<Vec<MultiplexedConnection>>>,
    max_connections: usize,
}

impl RedisConnectionPool {
    /// Create a new Redis connection pool
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `pool_size` - Maximum number of connections in the pool
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let pool = RedisConnectionPool::new("redis://localhost:6379", 10).await?;
    /// ```
    pub async fn new(redis_url: &str, pool_size: usize) -> anyhow::Result<Self> {
        let client = Client::open(redis_url)?;

        // Pre-create initial connections
        let mut connections = Vec::with_capacity(pool_size);
        for _ in 0..std::cmp::min(2, pool_size) {
            let conn = client.get_multiplexed_tokio_connection().await?;
            connections.push(conn);
        }

        Ok(Self {
            client,
            connections: Arc::new(Mutex::new(connections)),
            max_connections: pool_size,
        })
    }

    /// Get a connection from the pool
    ///
    /// If no connections are available and the pool isn't full,
    /// creates a new connection. Otherwise, waits for one to become available.
    pub async fn get_connection(&self) -> RiptideResult<MultiplexedConnection> {
        // Retry loop to wait for available connections
        const MAX_RETRIES: usize = 100;
        const RETRY_DELAY_MS: u64 = 10;

        for attempt in 0..MAX_RETRIES {
            let mut pool = self.connections.lock().await;

            if let Some(conn) = pool.pop() {
                debug!("Reusing pooled Redis connection (attempt {})", attempt + 1);
                return Ok(conn);
            } else if pool.len() < self.max_connections {
                debug!("Creating new Redis connection (pool not full)");
                drop(pool); // Release lock before async operation
                let conn = self
                    .client
                    .get_multiplexed_tokio_connection()
                    .await
                    .map_err(|e| {
                        RiptideError::Cache(format!("Failed to create Redis connection: {}", e))
                    })?;
                return Ok(conn);
            } else {
                // Pool is full and no connections available, wait and retry
                debug!(
                    "Pool full, waiting for connection (attempt {}/{})",
                    attempt + 1,
                    MAX_RETRIES
                );
                drop(pool); // Release lock before sleeping
                tokio::time::sleep(tokio::time::Duration::from_millis(RETRY_DELAY_MS)).await;
            }
        }

        // If we exhausted retries, return error
        Err(RiptideError::Cache(format!(
            "Failed to acquire connection after {} attempts",
            MAX_RETRIES
        )))
    }

    /// Return a connection to the pool
    ///
    /// Connections should be returned when no longer needed to allow reuse.
    pub async fn return_connection(&self, conn: MultiplexedConnection) {
        let mut pool = self.connections.lock().await;
        if pool.len() < self.max_connections {
            pool.push(conn);
            debug!("Returned connection to pool");
        } else {
            // Pool is full, drop the connection
            debug!("Pool full, dropping connection");
        }
    }

    /// Execute a Redis command directly
    ///
    /// For advanced use cases that need direct Redis access.
    /// Prefer using CacheStorage trait methods when possible.
    pub async fn execute<T, F, Fut>(&self, f: F) -> RiptideResult<T>
    where
        F: FnOnce(MultiplexedConnection) -> Fut,
        Fut: std::future::Future<Output = Result<(MultiplexedConnection, T), redis::RedisError>>,
    {
        let conn = self.get_connection().await?;

        match f(conn).await {
            Ok((conn, result)) => {
                self.return_connection(conn).await;
                Ok(result)
            }
            Err(e) => {
                error!("Redis command failed: {}", e);
                Err(RiptideError::Cache(format!("Redis error: {}", e)))
            }
        }
    }

    /// Get the underlying Redis client
    ///
    /// Use with caution - prefer connection pool methods.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Health check for the connection pool
    pub async fn health_check(&self) -> RiptideResult<bool> {
        let mut conn = self.get_connection().await?;

        match redis::cmd("PING").query_async::<String>(&mut conn).await {
            Ok(response) => {
                self.return_connection(conn).await;
                Ok(response == "PONG")
            }
            Err(e) => {
                error!("Health check failed: {}", e);
                Err(RiptideError::Cache(format!("Health check failed: {}", e)))
            }
        }
    }

    /// Get pool statistics
    pub async fn pool_stats(&self) -> PoolStats {
        let pool = self.connections.lock().await;
        PoolStats {
            available_connections: pool.len(),
            max_connections: self.max_connections,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Number of available connections
    pub available_connections: usize,
    /// Maximum pool size
    pub max_connections: usize,
}

impl Clone for RedisConnectionPool {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            connections: self.connections.clone(),
            max_connections: self.max_connections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_connection_pool() {
        let pool = RedisConnectionPool::new("redis://localhost:6379", 5)
            .await
            .unwrap();

        let stats = pool.pool_stats().await;
        assert!(stats.available_connections > 0);
        assert_eq!(stats.max_connections, 5);
    }

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_health_check() {
        let pool = RedisConnectionPool::new("redis://localhost:6379", 5)
            .await
            .unwrap();

        assert!(pool.health_check().await.unwrap());
    }
}
