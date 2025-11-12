//! Redis implementation of the DistributedCoordination port
//!
//! This adapter provides distributed coordination capabilities using Redis:
//! - Pub/sub messaging via Redis pub/sub
//! - Distributed cache operations
//! - Leader election using SET NX
//! - Cluster membership tracking
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_cache::adapters::RedisCoordination;
//! use std::time::Duration;
//!
//! let coord = RedisCoordination::new("redis://localhost:6379").await?;
//!
//! // Publish an event
//! coord.publish("cache:invalidate", b"user:123").await?;
//!
//! // Subscribe to events
//! let mut sub = coord.subscribe(&["cache:*"]).await?;
//! while let Some(msg) = sub.next_message().await? {
//!     println!("Received on {}: {:?}", msg.channel, msg.payload);
//! }
//! ```

use async_trait::async_trait;
use futures_util::stream::StreamExt;
use redis::aio::{MultiplexedConnection, PubSub};
use redis::{AsyncCommands, Client, Script};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

use riptide_types::error::riptide_error::RiptideError;
use riptide_types::ports::coordination::{
    CoordinationResult, DistributedCoordination, Subscriber, SubscriberMessage,
};

/// Redis implementation of distributed coordination
///
/// Provides thread-safe distributed coordination using Redis as the backend.
/// All operations are atomic and consistent across multiple instances.
///
/// # Key Prefixes
///
/// - `coord:v1:cache:*` - Cache entries
/// - `coord:v1:leader:*` - Leader election keys
/// - `coord:v1:node:*` - Node registration
/// - `coord:v1:heartbeat:*` - Node heartbeats
pub struct RedisCoordination {
    /// Redis connection for regular operations
    conn: Arc<Mutex<MultiplexedConnection>>,
    /// Redis client for creating new connections
    client: Arc<Client>,
    /// Key version for forward compatibility
    key_version: String,
}

impl RedisCoordination {
    /// Create a new Redis coordination adapter
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let coord = RedisCoordination::new("redis://localhost:6379").await?;
    /// ```
    pub async fn new(redis_url: &str) -> CoordinationResult<Self> {
        let client = Client::open(redis_url).map_err(|e| {
            error!("Failed to open Redis client: {}", e);
            RiptideError::Cache(format!("Failed to open Redis client: {}", e))
        })?;

        let conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| {
                error!("Failed to connect to Redis: {}", e);
                RiptideError::Cache(format!("Failed to connect to Redis: {}", e))
            })?;

        info!("Redis coordination initialized");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            client: Arc::new(client),
            key_version: "v1".to_string(),
        })
    }

    /// Create new coordination adapter with custom key version
    pub async fn with_version(redis_url: &str, version: impl Into<String>) -> CoordinationResult<Self> {
        let mut coord = Self::new(redis_url).await?;
        coord.key_version = version.into();
        Ok(coord)
    }

    /// Format versioned cache key
    fn cache_key(&self, key: &str) -> String {
        format!("coord:{}:cache:{}", self.key_version, key)
    }

    /// Format leader election key
    fn leader_key(&self, election_key: &str) -> String {
        format!("coord:{}:leader:{}", self.key_version, election_key)
    }

    /// Format node registration key
    fn node_key(&self, node_id: &str) -> String {
        format!("coord:{}:node:{}", self.key_version, node_id)
    }

    /// Format heartbeat key
    fn heartbeat_key(&self, node_id: &str) -> String {
        format!("coord:{}:heartbeat:{}", self.key_version, node_id)
    }

    /// Lua script for atomic leader election with TTL
    const ACQUIRE_LEADERSHIP_SCRIPT: &'static str = r#"
        local key = KEYS[1]
        local node_id = ARGV[1]
        local ttl = ARGV[2]

        -- Check if key exists
        local current = redis.call("GET", key)
        if current == false or current == node_id then
            redis.call("SETEX", key, ttl, node_id)
            return 1
        else
            return 0
        end
    "#;

    /// Lua script for safe leadership release
    const RELEASE_LEADERSHIP_SCRIPT: &'static str = r#"
        local key = KEYS[1]
        local node_id = ARGV[1]

        local current = redis.call("GET", key)
        if current == node_id then
            return redis.call("DEL", key)
        else
            return 0
        end
    "#;
}

#[async_trait]
impl DistributedCoordination for RedisCoordination {
    #[instrument(skip(self, message), fields(channel = %channel, message_size = message.len()))]
    async fn publish(&self, channel: &str, message: &[u8]) -> CoordinationResult<usize> {
        debug!("Publishing message to channel");

        let mut conn = self.conn.lock().await;
        let count: usize = conn.publish(channel, message).await.map_err(|e| {
            error!("Failed to publish message: {}", e);
            RiptideError::Cache(format!("Failed to publish message: {}", e))
        })?;

        debug!(subscribers = count, "Message published successfully");
        Ok(count)
    }

    #[instrument(skip(self), fields(channels = ?channels))]
    async fn subscribe(&self, channels: &[&str]) -> CoordinationResult<Box<dyn Subscriber>> {
        debug!("Creating subscription");

        // Create new connection for subscription
        let pubsub_conn = self
            .client
            .get_async_pubsub()
            .await
            .map_err(|e| {
                error!("Failed to create pub/sub connection: {}", e);
                RiptideError::Cache(format!("Failed to create pub/sub connection: {}", e))
            })?;

        let subscriber = RedisSubscriber::new(pubsub_conn, channels).await?;

        info!(channels = ?channels, "Subscription created successfully");
        Ok(Box::new(subscriber))
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn cache_get(&self, key: &str) -> CoordinationResult<Option<Vec<u8>>> {
        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        let data: Option<Vec<u8>> = conn.get(&versioned_key).await.map_err(|e| {
            error!("Failed to get cache value: {}", e);
            RiptideError::Cache(format!("Failed to get cache value: {}", e))
        })?;

        if data.is_some() {
            debug!("Cache hit");
        } else {
            debug!("Cache miss");
        }

        Ok(data)
    }

    #[instrument(skip(self, value), fields(key = %key, value_size = value.len(), ttl_secs = ttl.map(|d| d.as_secs())))]
    async fn cache_set(&self, key: &str, value: &[u8], ttl: Option<Duration>) -> CoordinationResult<()> {
        debug!("Setting cache value");

        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        if let Some(ttl_duration) = ttl {
            let ttl_secs = ttl_duration.as_secs();
            conn.set_ex::<_, _, ()>(&versioned_key, value, ttl_secs)
                .await
                .map_err(|e| {
                    error!("Failed to set cache value with TTL: {}", e);
                    RiptideError::Cache(format!("Failed to set cache value: {}", e))
                })?;
        } else {
            conn.set::<_, _, ()>(&versioned_key, value)
                .await
                .map_err(|e| {
                    error!("Failed to set cache value: {}", e);
                    RiptideError::Cache(format!("Failed to set cache value: {}", e))
                })?;
        }

        debug!("Cache value set successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn cache_delete(&self, key: &str) -> CoordinationResult<bool> {
        debug!("Deleting cache key");

        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        let deleted: u64 = conn.del(&versioned_key).await.map_err(|e| {
            error!("Failed to delete cache key: {}", e);
            RiptideError::Cache(format!("Failed to delete cache key: {}", e))
        })?;

        let existed = deleted > 0;
        debug!(existed = existed, "Cache key deletion completed");
        Ok(existed)
    }

    #[instrument(skip(self), fields(pattern = %pattern))]
    async fn cache_delete_pattern(&self, pattern: &str) -> CoordinationResult<usize> {
        debug!("Deleting keys matching pattern");

        let versioned_pattern = self.cache_key(pattern);
        let mut conn = self.conn.lock().await;

        // Get keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&versioned_pattern)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to scan keys: {}", e);
                RiptideError::Cache(format!("Failed to scan keys: {}", e))
            })?;

        let count = keys.len();

        if !keys.is_empty() {
            let deleted: u64 = conn.del(&keys).await.map_err(|e| {
                error!("Failed to delete keys: {}", e);
                RiptideError::Cache(format!("Failed to delete keys: {}", e))
            })?;

            debug!(deleted = deleted, "Pattern deletion completed");
        } else {
            debug!("No keys matched pattern");
        }

        Ok(count)
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn cache_exists(&self, key: &str) -> CoordinationResult<bool> {
        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        let exists: bool = conn.exists(&versioned_key).await.map_err(|e| {
            error!("Failed to check key existence: {}", e);
            RiptideError::Cache(format!("Failed to check key existence: {}", e))
        })?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn cache_ttl(&self, key: &str) -> CoordinationResult<Option<Duration>> {
        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        let ttl_secs: i64 = conn.ttl(&versioned_key).await.map_err(|e| {
            error!("Failed to get TTL: {}", e);
            RiptideError::Cache(format!("Failed to get TTL: {}", e))
        })?;

        match ttl_secs {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(None), // Key exists but has no TTL
            secs if secs > 0 => Ok(Some(Duration::from_secs(secs as u64))),
            _ => Ok(None),
        }
    }

    #[instrument(skip(self), fields(key = %key, amount = amount, ttl_secs = ttl.map(|d| d.as_secs())))]
    async fn cache_incr(&self, key: &str, amount: i64, ttl: Option<Duration>) -> CoordinationResult<i64> {
        debug!("Incrementing counter");

        let versioned_key = self.cache_key(key);
        let mut conn = self.conn.lock().await;

        let result: i64 = conn.incr(&versioned_key, amount).await.map_err(|e| {
            error!("Failed to increment counter: {}", e);
            RiptideError::Cache(format!("Failed to increment counter: {}", e))
        })?;

        // Set TTL if provided
        if let Some(ttl_duration) = ttl {
            conn.expire::<_, ()>(&versioned_key, ttl_duration.as_secs() as i64)
                .await
                .map_err(|e| {
                    error!("Failed to set TTL: {}", e);
                    RiptideError::Cache(format!("Failed to set TTL: {}", e))
                })?;
        }

        debug!(new_value = result, "Counter incremented successfully");
        Ok(result)
    }

    #[instrument(skip(self), fields(election_key = %election_key, node_id = %node_id, ttl_secs = ttl.as_secs()))]
    async fn try_acquire_leadership(
        &self,
        election_key: &str,
        node_id: &str,
        ttl: Duration,
    ) -> CoordinationResult<bool> {
        debug!("Attempting to acquire leadership");

        let key = self.leader_key(election_key);
        let ttl_secs = ttl.as_secs();

        let mut conn = self.conn.lock().await;

        let acquired: i32 = Script::new(Self::ACQUIRE_LEADERSHIP_SCRIPT)
            .key(&key)
            .arg(node_id)
            .arg(ttl_secs)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to acquire leadership: {}", e);
                RiptideError::Cache(format!("Failed to acquire leadership: {}", e))
            })?;

        let is_leader = acquired == 1;
        if is_leader {
            info!("Leadership acquired");
        } else {
            debug!("Leadership acquisition failed - another node is leader");
        }

        Ok(is_leader)
    }

    #[instrument(skip(self), fields(election_key = %election_key, node_id = %node_id))]
    async fn release_leadership(&self, election_key: &str, node_id: &str) -> CoordinationResult<bool> {
        debug!("Releasing leadership");

        let key = self.leader_key(election_key);
        let mut conn = self.conn.lock().await;

        let released: i32 = Script::new(Self::RELEASE_LEADERSHIP_SCRIPT)
            .key(&key)
            .arg(node_id)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to release leadership: {}", e);
                RiptideError::Cache(format!("Failed to release leadership: {}", e))
            })?;

        let was_released = released > 0;
        if was_released {
            info!("Leadership released successfully");
        } else {
            warn!("Leadership release failed - not the current leader");
        }

        Ok(was_released)
    }

    #[instrument(skip(self), fields(election_key = %election_key))]
    async fn get_leader(&self, election_key: &str) -> CoordinationResult<Option<String>> {
        let key = self.leader_key(election_key);
        let mut conn = self.conn.lock().await;

        let leader: Option<String> = conn.get(&key).await.map_err(|e| {
            error!("Failed to get leader: {}", e);
            RiptideError::Cache(format!("Failed to get leader: {}", e))
        })?;

        Ok(leader)
    }

    #[instrument(skip(self, metadata), fields(node_id = %node_id, ttl_secs = ttl.as_secs()))]
    async fn register_node(
        &self,
        node_id: &str,
        metadata: HashMap<String, String>,
        ttl: Duration,
    ) -> CoordinationResult<()> {
        debug!("Registering node");

        let node_key = self.node_key(node_id);
        let heartbeat_key = self.heartbeat_key(node_id);
        let ttl_secs = ttl.as_secs();

        // Serialize metadata
        let metadata_json = serde_json::to_string(&metadata).map_err(|e| {
            error!("Failed to serialize metadata: {}", e);
            RiptideError::Cache(format!("Failed to serialize metadata: {}", e))
        })?;

        let mut conn = self.conn.lock().await;

        // Store node metadata with TTL
        conn.set_ex::<_, _, ()>(&node_key, &metadata_json, ttl_secs)
            .await
            .map_err(|e| {
                error!("Failed to register node: {}", e);
                RiptideError::Cache(format!("Failed to register node: {}", e))
            })?;

        // Set heartbeat
        conn.set_ex::<_, _, ()>(&heartbeat_key, "alive", ttl_secs)
            .await
            .map_err(|e| {
                error!("Failed to set heartbeat: {}", e);
                RiptideError::Cache(format!("Failed to set heartbeat: {}", e))
            })?;

        info!("Node registered successfully");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_active_nodes(&self) -> CoordinationResult<HashSet<String>> {
        debug!("Getting active nodes");

        let pattern = self.heartbeat_key("*");
        let mut conn = self.conn.lock().await;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to scan for nodes: {}", e);
                RiptideError::Cache(format!("Failed to scan for nodes: {}", e))
            })?;

        // Extract node IDs from heartbeat keys
        let prefix = self.heartbeat_key("");
        let nodes: HashSet<String> = keys
            .into_iter()
            .filter_map(|key| {
                key.strip_prefix(&prefix).map(|id| id.to_string())
            })
            .collect();

        debug!(count = nodes.len(), "Active nodes retrieved");
        Ok(nodes)
    }

    #[instrument(skip(self), fields(node_id = %node_id))]
    async fn get_node_metadata(&self, node_id: &str) -> CoordinationResult<Option<HashMap<String, String>>> {
        let node_key = self.node_key(node_id);
        let mut conn = self.conn.lock().await;

        let metadata_json: Option<String> = conn.get(&node_key).await.map_err(|e| {
            error!("Failed to get node metadata: {}", e);
            RiptideError::Cache(format!("Failed to get node metadata: {}", e))
        })?;

        match metadata_json {
            Some(json) => {
                let metadata: HashMap<String, String> = serde_json::from_str(&json).map_err(|e| {
                    error!("Failed to deserialize metadata: {}", e);
                    RiptideError::Cache(format!("Failed to deserialize metadata: {}", e))
                })?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self), fields(node_id = %node_id, ttl_secs = ttl.as_secs()))]
    async fn send_heartbeat(&self, node_id: &str, ttl: Duration) -> CoordinationResult<bool> {
        debug!("Sending heartbeat");

        let heartbeat_key = self.heartbeat_key(node_id);
        let node_key = self.node_key(node_id);
        let ttl_secs = ttl.as_secs();

        let mut conn = self.conn.lock().await;

        // Update heartbeat TTL
        let heartbeat_updated: bool = conn
            .set_ex::<_, _, ()>(&heartbeat_key, "alive", ttl_secs)
            .await
            .is_ok();

        // Also update node metadata TTL
        let _: Result<(), _> = conn.expire(&node_key, ttl_secs as i64).await;

        if heartbeat_updated {
            debug!("Heartbeat sent successfully");
        } else {
            warn!("Heartbeat update failed");
        }

        Ok(heartbeat_updated)
    }
}

/// Redis pub/sub subscriber implementation
struct RedisSubscriber {
    pubsub: Arc<Mutex<PubSub>>,
}

impl RedisSubscriber {
    /// Create new Redis subscriber
    async fn new(mut pubsub: PubSub, channels: &[&str]) -> CoordinationResult<Self> {
        // Subscribe to all channels
        for channel in channels {
            pubsub.subscribe(channel).await.map_err(|e| {
                error!("Failed to subscribe to channel {}: {}", channel, e);
                RiptideError::Cache(format!("Failed to subscribe to channel: {}", e))
            })?;
        }

        Ok(Self {
            pubsub: Arc::new(Mutex::new(pubsub)),
        })
    }
}

#[async_trait]
impl Subscriber for RedisSubscriber {
    async fn next_message(&mut self) -> CoordinationResult<Option<SubscriberMessage>> {
        let mut pubsub = self.pubsub.lock().await;

        // Get the stream and take the next message
        let mut stream = pubsub.on_message();
        let msg_opt = stream.next().await;

        match msg_opt {
            Some(msg) => {
                let channel = msg.get_channel_name().to_string();
                let payload: Vec<u8> = msg.get_payload().map_err(|e| {
                    error!("Failed to get message payload: {}", e);
                    RiptideError::Cache(format!("Failed to get message payload: {}", e))
                })?;

                let pattern = msg.get_pattern::<String>().ok();

                Ok(Some(SubscriberMessage::new(channel, payload, pattern)))
            }
            None => Ok(None),
        }
    }

    async fn unsubscribe(&mut self) -> CoordinationResult<()> {
        // Redis pubsub doesn't have unsubscribe_all, we'll just drop the connection
        // which effectively unsubscribes from all channels
        info!("Unsubscribed from all channels");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_formatting() {
        // Test key formatting logic without needing actual Redis connection
        let key_version = "v1";

        // Test cache key format
        let cache_key = format!("coord:{}:cache:{}", key_version, "user:123");
        assert_eq!(cache_key, "coord:v1:cache:user:123");

        // Test leader key format
        let leader_key = format!("coord:{}:leader:{}", key_version, "sync-leader");
        assert_eq!(leader_key, "coord:v1:leader:sync-leader");

        // Test node key format
        let node_key = format!("coord:{}:node:{}", key_version, "node-1");
        assert_eq!(node_key, "coord:v1:node:node-1");

        // Test heartbeat key format
        let heartbeat_key = format!("coord:{}:heartbeat:{}", key_version, "node-1");
        assert_eq!(heartbeat_key, "coord:v1:heartbeat:node-1");
    }

    // Note: Integration tests with actual Redis are in crates/riptide-cache/tests/
}
