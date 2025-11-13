# Distributed Coordination Port Trait Specification

**Status**: Draft
**Version**: 1.0.0
**Date**: 2025-11-12
**Author**: System Architect

## Executive Summary

This document specifies the `DistributedCoordination` port trait that abstracts distributed cache synchronization, pub/sub messaging, and coordination primitives. This trait is critical for Phase 0 hexagonal refactoring of `riptide-persistence`, eliminating direct Redis dependencies from `sync.rs` and enabling testability and backend flexibility.

---

## 1. Problem Statement

### Current Architecture Violations

The `sync.rs` module currently violates hexagonal architecture principles:

- **Direct Redis Coupling**: Uses `redis::aio::MultiplexedConnection` directly
- **Infrastructure Leak**: Redis pub/sub commands scattered throughout business logic
- **Testing Difficulty**: Cannot mock distributed operations for unit tests
- **Backend Lock-in**: Impossible to swap Redis for alternatives (NATS, Kafka, in-memory)

### Impact Analysis

```rust
// Current violation in sync.rs:172-184
pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
    let channel = format!("riptide:sync:{}", "operations");
    let message = serde_json::to_string(&operation)?;

    let mut conn = self.pool.lock().await;  // Direct Redis dependency!
    conn.publish::<_, _, ()>(&channel, &message).await?;
    Ok(())
}
```

**Problems**:
1. Business logic (`notify_operation`) directly calls Redis methods
2. No port abstraction for pub/sub
3. Cannot test without Redis running
4. Cannot switch to NATS, RabbitMQ, or other messaging systems

---

## 2. Port Trait Specification

### 2.1 Trait Definition

```rust
//! Distributed coordination abstraction for cache synchronization and messaging
//!
//! This trait provides backend-agnostic distributed primitives including:
//! - Pub/sub messaging for cache invalidation
//! - Distributed key operations (pattern matching, bulk operations)
//! - Cluster coordination (heartbeats, status)
//! - Conflict resolution primitives
//!
//! # Design Goals
//!
//! - **Hexagonal Architecture**: Clean separation from infrastructure
//! - **Testability**: Mockable for unit tests without external dependencies
//! - **Flexibility**: Support Redis, NATS, Kafka, in-memory, or custom backends
//! - **Performance**: Async-first with efficient batch operations
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::DistributedCoordination;
//!
//! async fn sync_cache(coord: &dyn DistributedCoordination) -> anyhow::Result<()> {
//!     // Publish cache invalidation
//!     coord.publish("cache:invalidate", b"user:123").await?;
//!
//!     // Subscribe to updates
//!     let mut sub = coord.subscribe("cache:*").await?;
//!     while let Some(msg) = sub.next().await {
//!         println!("Received: {:?}", msg);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::time::Duration;

/// Backend-agnostic distributed coordination interface
///
/// Implementations must be thread-safe (`Send + Sync`) and support
/// asynchronous operations. All operations should be idempotent where possible.
#[async_trait]
pub trait DistributedCoordination: Send + Sync {
    // ==================== Pub/Sub Messaging ====================

    /// Publish a message to a channel
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name (e.g., "riptide:sync:operations")
    /// * `message` - Binary message payload
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message published successfully
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Guarantees
    ///
    /// - At-most-once delivery semantics
    /// - Non-blocking operation
    /// - Idempotent (safe to retry)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let operation = SyncOperation { ... };
    /// let json = serde_json::to_vec(&operation)?;
    /// coord.publish("riptide:sync:operations", &json).await?;
    /// ```
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()>;

    /// Subscribe to a channel pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - Channel pattern (supports wildcards: "cache:*", "sync:?:ops")
    ///
    /// # Returns
    ///
    /// * `Ok(stream)` - Message stream for incoming messages
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Stream Behavior
    ///
    /// - Messages arrive in order of publication
    /// - Stream ends when subscription is dropped
    /// - Backpressure supported via async iteration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut sub = coord.subscribe("riptide:sync:*").await?;
    /// while let Some(msg) = sub.next().await {
    ///     let op: SyncOperation = serde_json::from_slice(&msg.payload)?;
    ///     process_operation(op).await?;
    /// }
    /// ```
    async fn subscribe(&self, pattern: &str) -> RiptideResult<MessageStream>;

    /// Publish with acknowledgment (for critical messages)
    ///
    /// Waits for at least one subscriber to acknowledge receipt.
    /// Use sparingly as it adds latency.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name
    /// * `message` - Binary message payload
    /// * `timeout` - Maximum wait time for acknowledgment
    ///
    /// # Returns
    ///
    /// * `Ok(ack_count)` - Number of subscribers that acknowledged
    /// * `Err(_)` - Timeout or coordination error
    async fn publish_with_ack(
        &self,
        channel: &str,
        message: &[u8],
        timeout: Duration,
    ) -> RiptideResult<usize>;

    // ==================== Distributed Key Operations ====================

    /// Find keys matching a pattern
    ///
    /// **⚠️ Warning**: This can be expensive on large datasets.
    /// Use with caution in production. Consider implementing SCAN for large sets.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Key pattern with wildcards (e.g., "user:*", "cache:v2:*")
    ///
    /// # Returns
    ///
    /// * `Ok(keys)` - Vector of matching keys
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Performance
    ///
    /// - O(N) where N = total keys in dataset
    /// - Blocks during iteration on single-threaded backends
    /// - Consider pagination for production use
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let expired_keys = coord.keys("cache:expired:*").await?;
    /// coord.delete_many(&expired_keys).await?;
    /// ```
    async fn keys(&self, pattern: &str) -> RiptideResult<Vec<String>>;

    /// Delete multiple keys atomically
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of keys to delete
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of keys actually deleted
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Atomicity
    ///
    /// - Best-effort atomic operation
    /// - Redis: atomic via DEL command
    /// - Other backends: may be eventually consistent
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let stale_keys = vec!["cache:v1:123", "cache:v1:456"];
    /// let deleted = coord.delete_many(&stale_keys).await?;
    /// println!("Deleted {} keys", deleted);
    /// ```
    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize>;

    /// Delete single key
    ///
    /// Convenience method for single-key deletion.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key was deleted
    /// * `Ok(false)` - Key didn't exist
    /// * `Err(_)` - Coordination backend error
    async fn delete(&self, key: &str) -> RiptideResult<bool> {
        let count = self.delete_many(&[key]).await?;
        Ok(count > 0)
    }

    /// Clear all keys (DANGEROUS)
    ///
    /// **⚠️ WARNING**: This operation is irreversible and affects ALL keys
    /// in the coordination backend. Use only in development/testing.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All keys cleared
    /// * `Err(_)` - Coordination backend error or permission denied
    ///
    /// # Safety
    ///
    /// - Should be disabled in production environments
    /// - Consider namespace isolation before calling
    /// - May require elevated permissions
    async fn flush_all(&self) -> RiptideResult<()>;

    // ==================== Cluster Coordination ====================

    /// Register this node in the cluster
    ///
    /// Called during initialization to announce presence to other nodes.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for this node
    /// * `metadata` - Optional node metadata (region, capacity, etc.)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Node registered successfully
    /// * `Err(_)` - Coordination backend error
    async fn register_node(&self, node_id: &str, metadata: Option<&[u8]>) -> RiptideResult<()>;

    /// Send heartbeat for this node
    ///
    /// Periodic heartbeat to maintain cluster membership.
    ///
    /// # Arguments
    ///
    /// * `node_id` - This node's identifier
    /// * `ttl` - How long this heartbeat is valid
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Heartbeat recorded
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Frequency
    ///
    /// - Typical: every 5-10 seconds
    /// - TTL should be 2-3x heartbeat interval
    async fn heartbeat(&self, node_id: &str, ttl: Duration) -> RiptideResult<()>;

    /// List all active nodes in the cluster
    ///
    /// Returns nodes that have sent recent heartbeats.
    ///
    /// # Returns
    ///
    /// * `Ok(nodes)` - Vector of active node IDs
    /// * `Err(_)` - Coordination backend error
    async fn list_nodes(&self) -> RiptideResult<Vec<String>>;

    /// Get metadata for a specific node
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node identifier
    ///
    /// # Returns
    ///
    /// * `Ok(Some(metadata))` - Node exists with metadata
    /// * `Ok(None)` - Node not found or expired
    /// * `Err(_)` - Coordination backend error
    async fn get_node_metadata(&self, node_id: &str) -> RiptideResult<Option<Vec<u8>>>;

    // ==================== Leader Election ====================

    /// Attempt to become leader with expiring lock
    ///
    /// Tries to acquire leadership for the specified duration.
    ///
    /// # Arguments
    ///
    /// * `node_id` - This node's identifier
    /// * `ttl` - How long to hold leadership
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Leadership acquired
    /// * `Ok(false)` - Another node is leader
    /// * `Err(_)` - Coordination backend error
    ///
    /// # Algorithm
    ///
    /// - Redis: SET NX EX pattern
    /// - NATS: key-value watch
    /// - In-memory: atomic compare-and-swap
    async fn try_acquire_leadership(&self, node_id: &str, ttl: Duration) -> RiptideResult<bool>;

    /// Release leadership voluntarily
    ///
    /// Only succeeds if this node is current leader.
    ///
    /// # Arguments
    ///
    /// * `node_id` - This node's identifier
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Leadership released
    /// * `Ok(false)` - Not the leader
    /// * `Err(_)` - Coordination backend error
    async fn release_leadership(&self, node_id: &str) -> RiptideResult<bool>;

    /// Check who is the current leader
    ///
    /// # Returns
    ///
    /// * `Ok(Some(node_id))` - Leader exists
    /// * `Ok(None)` - No current leader
    /// * `Err(_)` - Coordination backend error
    async fn get_leader(&self) -> RiptideResult<Option<String>>;

    // ==================== Health & Diagnostics ====================

    /// Health check for coordination backend
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Backend is healthy and responsive
    /// * `Ok(false)` - Backend is degraded but operational
    /// * `Err(_)` - Backend is unavailable
    async fn health_check(&self) -> RiptideResult<bool> {
        // Default implementation: try a simple operation
        const HEALTH_KEY: &str = "__health_check__";
        self.delete(HEALTH_KEY).await?;
        Ok(true)
    }

    /// Get coordination statistics
    ///
    /// # Returns
    ///
    /// * `Ok(stats)` - Backend statistics
    /// * `Err(_)` - Statistics unavailable
    async fn stats(&self) -> RiptideResult<CoordinationStats> {
        // Default implementation: minimal stats
        Ok(CoordinationStats::default())
    }
}

// ==================== Supporting Types ====================

/// Message received from subscription
#[derive(Debug, Clone)]
pub struct Message {
    /// Channel this message was published to
    pub channel: String,
    /// Message payload
    pub payload: Vec<u8>,
    /// Timestamp of publication (if available)
    pub timestamp: Option<std::time::SystemTime>,
}

/// Stream of messages from a subscription
pub type MessageStream = Pin<Box<dyn Stream<Item = RiptideResult<Message>> + Send>>;

/// Coordination backend statistics
#[derive(Debug, Clone, Default)]
pub struct CoordinationStats {
    /// Total active subscriptions
    pub active_subscriptions: usize,
    /// Messages published per second
    pub publish_rate: f64,
    /// Messages received per second
    pub receive_rate: f64,
    /// Number of registered nodes
    pub cluster_size: usize,
    /// Current leader node ID
    pub current_leader: Option<String>,
    /// Backend-specific metadata
    pub metadata: std::collections::HashMap<String, String>,
}
```

---

## 3. Method Contracts & Error Handling

### 3.1 Error Handling Strategy

All methods return `RiptideResult<T>` which is an alias for `Result<T, RiptideError>`.

```rust
// From riptide-types/src/error.rs
pub type Result<T> = std::result::Result<T, RiptideError>;

pub enum RiptideError {
    // Coordination backend errors
    CoordinationError(String),

    // Timeout errors
    TimeoutError(String),

    // Serialization errors
    SerializationError(String),

    // Network errors
    NetworkError(String),

    // ... other variants
}
```

### 3.2 Retry Semantics

| Operation | Idempotent | Retry Safe | Recommended Strategy |
|-----------|-----------|-----------|---------------------|
| `publish` | Yes | Yes | Exponential backoff, 3 retries |
| `subscribe` | Yes | Yes | Retry on connection failure |
| `publish_with_ack` | Yes | No | Manual retry with timeout |
| `keys` | Yes | Yes | Single retry, cached results |
| `delete_many` | Yes | Yes | Retry with confirmation |
| `flush_all` | Yes | **NO** | Manual confirmation only |
| `heartbeat` | Yes | Yes | Skip on failure, next cycle |
| `try_acquire_leadership` | Yes | No | No retry, next election cycle |

### 3.3 Thread Safety

All implementations **MUST** be:
- `Send`: Can be transferred across thread boundaries
- `Sync`: Can be shared between threads
- Arc-safe: Cloneable via `Arc<dyn DistributedCoordination>`

---

## 4. Implementation Examples

### 4.1 Redis Adapter

```rust
//! Redis implementation of DistributedCoordination
//!
//! Location: crates/riptide-persistence/src/adapters/redis_coordination.rs

use riptide_types::ports::coordination::{
    DistributedCoordination, Message, MessageStream, CoordinationStats
};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use async_trait::async_trait;
use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Redis-backed distributed coordination
pub struct RedisCoordination {
    /// Connection pool for commands
    pool: Arc<Mutex<MultiplexedConnection>>,
    /// Node ID for leader election
    node_id: String,
    /// Key prefix for namespacing
    key_prefix: String,
}

impl RedisCoordination {
    /// Create new Redis coordination adapter
    pub async fn new(
        redis_url: &str,
        node_id: String,
        key_prefix: Option<String>,
    ) -> RiptideResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| RiptideError::CoordinationError(format!("Redis connect: {}", e)))?;

        let conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Redis pool: {}", e)))?;

        Ok(Self {
            pool: Arc::new(Mutex::new(conn)),
            node_id,
            key_prefix: key_prefix.unwrap_or_else(|| "riptide".to_string()),
        })
    }

    /// Build namespaced key
    fn namespaced_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl DistributedCoordination for RedisCoordination {
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()> {
        let mut conn = self.pool.lock().await;
        conn.publish::<_, _, ()>(channel, message)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Publish failed: {}", e)))
    }

    async fn subscribe(&self, pattern: &str) -> RiptideResult<MessageStream> {
        // Create new connection for subscription (Redis requires dedicated connection)
        let client = redis::Client::open(self.pool.lock().await.get_connection_info())
            .map_err(|e| RiptideError::CoordinationError(format!("Subscribe connect: {}", e)))?;

        let mut pubsub = client
            .get_async_pubsub()
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("PubSub init: {}", e)))?;

        pubsub
            .psubscribe(pattern)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Pattern subscribe: {}", e)))?;

        let stream = pubsub.into_on_message().map(|msg| {
            Ok(Message {
                channel: msg.get_channel_name().to_string(),
                payload: msg.get_payload_bytes().to_vec(),
                timestamp: Some(std::time::SystemTime::now()),
            })
        });

        Ok(Box::pin(stream))
    }

    async fn publish_with_ack(
        &self,
        channel: &str,
        message: &[u8],
        timeout: Duration,
    ) -> RiptideResult<usize> {
        // Redis PUBLISH returns subscriber count
        let mut conn = self.pool.lock().await;

        let subscribers: usize = tokio::time::timeout(
            timeout,
            conn.publish(channel, message)
        )
        .await
        .map_err(|_| RiptideError::TimeoutError("Publish timeout".to_string()))?
        .map_err(|e| RiptideError::CoordinationError(format!("Publish failed: {}", e)))?;

        if subscribers == 0 {
            Err(RiptideError::CoordinationError("No subscribers".to_string()))
        } else {
            Ok(subscribers)
        }
    }

    async fn keys(&self, pattern: &str) -> RiptideResult<Vec<String>> {
        let namespaced = self.namespaced_key(pattern);
        let mut conn = self.pool.lock().await;

        redis::cmd("KEYS")
            .arg(&namespaced)
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("KEYS failed: {}", e)))
    }

    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize> {
        if keys.is_empty() {
            return Ok(0);
        }

        let namespaced_keys: Vec<String> = keys
            .iter()
            .map(|k| self.namespaced_key(k))
            .collect();

        let mut conn = self.pool.lock().await;
        let deleted: usize = conn
            .del(&namespaced_keys)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("DEL failed: {}", e)))?;

        Ok(deleted)
    }

    async fn flush_all(&self) -> RiptideResult<()> {
        let mut conn = self.pool.lock().await;
        redis::cmd("FLUSHDB")
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("FLUSHDB failed: {}", e)))
    }

    async fn register_node(&self, node_id: &str, metadata: Option<&[u8]>) -> RiptideResult<()> {
        let key = format!("{}:nodes:{}", self.key_prefix, node_id);
        let value = metadata.unwrap_or(b"{}");

        let mut conn = self.pool.lock().await;
        conn.set_ex(&key, value, 60) // 60 second TTL
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Node register: {}", e)))
    }

    async fn heartbeat(&self, node_id: &str, ttl: Duration) -> RiptideResult<()> {
        let key = format!("{}:heartbeat:{}", self.key_prefix, node_id);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut conn = self.pool.lock().await;
        conn.set_ex(&key, timestamp, ttl.as_secs())
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Heartbeat: {}", e)))
    }

    async fn list_nodes(&self) -> RiptideResult<Vec<String>> {
        let pattern = format!("{}:heartbeat:*", self.key_prefix);
        let mut conn = self.pool.lock().await;

        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("List nodes: {}", e)))?;

        // Extract node IDs from keys
        let prefix_len = format!("{}:heartbeat:", self.key_prefix).len();
        let nodes = keys.iter().map(|k| k[prefix_len..].to_string()).collect();

        Ok(nodes)
    }

    async fn get_node_metadata(&self, node_id: &str) -> RiptideResult<Option<Vec<u8>>> {
        let key = format!("{}:nodes:{}", self.key_prefix, node_id);
        let mut conn = self.pool.lock().await;

        let data: Option<Vec<u8>> = conn
            .get(&key)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Get metadata: {}", e)))?;

        Ok(data)
    }

    async fn try_acquire_leadership(&self, node_id: &str, ttl: Duration) -> RiptideResult<bool> {
        let key = format!("{}:leader", self.key_prefix);
        let mut conn = self.pool.lock().await;

        // SET NX EX pattern - atomic acquire with expiry
        let acquired: bool = redis::cmd("SET")
            .arg(&key)
            .arg(node_id)
            .arg("NX") // Only set if not exists
            .arg("EX") // Expiry in seconds
            .arg(ttl.as_secs())
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Acquire leader: {}", e)))?;

        Ok(acquired)
    }

    async fn release_leadership(&self, node_id: &str) -> RiptideResult<bool> {
        let key = format!("{}:leader", self.key_prefix);
        let mut conn = self.pool.lock().await;

        // Lua script for atomic check-and-delete
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                redis.call("DEL", KEYS[1])
                return 1
            else
                return 0
            end
        "#;

        let released: bool = redis::Script::new(script)
            .key(&key)
            .arg(node_id)
            .invoke_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Release leader: {}", e)))?;

        Ok(released)
    }

    async fn get_leader(&self) -> RiptideResult<Option<String>> {
        let key = format!("{}:leader", self.key_prefix);
        let mut conn = self.pool.lock().await;

        let leader: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Get leader: {}", e)))?;

        Ok(leader)
    }

    async fn health_check(&self) -> RiptideResult<bool> {
        let mut conn = self.pool.lock().await;
        let pong: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Health check: {}", e)))?;

        Ok(pong == "PONG")
    }

    async fn stats(&self) -> RiptideResult<CoordinationStats> {
        let mut conn = self.pool.lock().await;

        let info: String = redis::cmd("INFO")
            .arg("stats")
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CoordinationError(format!("Get stats: {}", e)))?;

        // Parse Redis INFO output
        let mut stats = CoordinationStats::default();
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                stats.metadata.insert(key.to_string(), value.to_string());
            }
        }

        // Get cluster info
        stats.cluster_size = self.list_nodes().await?.len();
        stats.current_leader = self.get_leader().await?;

        Ok(stats)
    }
}
```

### 4.2 In-Memory Adapter (for testing)

```rust
//! In-memory implementation for testing
//!
//! Location: crates/riptide-persistence/src/adapters/memory_coordination.rs

use riptide_types::ports::coordination::{
    DistributedCoordination, Message, MessageStream, CoordinationStats
};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use async_trait::async_trait;
use futures::stream::StreamExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, broadcast};

/// In-memory coordination for testing
pub struct MemoryCoordination {
    /// Key-value store
    store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// Pub/sub channels
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
    /// Node registry
    nodes: Arc<RwLock<HashMap<String, (Vec<u8>, SystemTime)>>>,
    /// Leader state
    leader: Arc<RwLock<Option<(String, SystemTime)>>>,
}

impl MemoryCoordination {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            leader: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl DistributedCoordination for MemoryCoordination {
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()> {
        let channels = self.channels.read().await;

        if let Some(tx) = channels.get(channel) {
            let msg = Message {
                channel: channel.to_string(),
                payload: message.to_vec(),
                timestamp: Some(SystemTime::now()),
            };

            // Ignore send errors (no subscribers)
            let _ = tx.send(msg);
        }

        Ok(())
    }

    async fn subscribe(&self, pattern: &str) -> RiptideResult<MessageStream> {
        let mut channels = self.channels.write().await;

        // Create channel if not exists
        let tx = channels
            .entry(pattern.to_string())
            .or_insert_with(|| broadcast::channel(100).0);

        let rx = tx.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .map(|result| {
                result.map_err(|e| RiptideError::CoordinationError(format!("Stream error: {}", e)))
            });

        Ok(Box::pin(stream))
    }

    async fn publish_with_ack(
        &self,
        channel: &str,
        message: &[u8],
        _timeout: Duration,
    ) -> RiptideResult<usize> {
        let channels = self.channels.read().await;

        if let Some(tx) = channels.get(channel) {
            Ok(tx.receiver_count())
        } else {
            Ok(0)
        }
    }

    async fn keys(&self, pattern: &str) -> RiptideResult<Vec<String>> {
        let store = self.store.read().await;

        // Simple glob matching
        let keys: Vec<String> = store
            .keys()
            .filter(|k| pattern_matches(k, pattern))
            .cloned()
            .collect();

        Ok(keys)
    }

    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize> {
        let mut store = self.store.write().await;
        let mut count = 0;

        for key in keys {
            if store.remove(*key).is_some() {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn flush_all(&self) -> RiptideResult<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }

    async fn register_node(&self, node_id: &str, metadata: Option<&[u8]>) -> RiptideResult<()> {
        let mut nodes = self.nodes.write().await;
        nodes.insert(
            node_id.to_string(),
            (metadata.unwrap_or(b"{}").to_vec(), SystemTime::now()),
        );
        Ok(())
    }

    async fn heartbeat(&self, node_id: &str, _ttl: Duration) -> RiptideResult<()> {
        let mut nodes = self.nodes.write().await;
        if let Some((metadata, _)) = nodes.get(node_id) {
            let metadata = metadata.clone();
            nodes.insert(node_id.to_string(), (metadata, SystemTime::now()));
        }
        Ok(())
    }

    async fn list_nodes(&self) -> RiptideResult<Vec<String>> {
        let nodes = self.nodes.read().await;
        let now = SystemTime::now();

        let active: Vec<String> = nodes
            .iter()
            .filter(|(_, (_, last_seen))| {
                now.duration_since(*last_seen).unwrap_or_default() < Duration::from_secs(60)
            })
            .map(|(id, _)| id.clone())
            .collect();

        Ok(active)
    }

    async fn get_node_metadata(&self, node_id: &str) -> RiptideResult<Option<Vec<u8>>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.get(node_id).map(|(metadata, _)| metadata.clone()))
    }

    async fn try_acquire_leadership(&self, node_id: &str, ttl: Duration) -> RiptideResult<bool> {
        let mut leader = self.leader.write().await;
        let now = SystemTime::now();

        // Check if leadership expired
        if let Some((_, expires)) = &*leader {
            if now >= *expires {
                *leader = None;
            }
        }

        // Try to acquire
        if leader.is_none() {
            *leader = Some((node_id.to_string(), now + ttl));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn release_leadership(&self, node_id: &str) -> RiptideResult<bool> {
        let mut leader = self.leader.write().await;

        if let Some((current, _)) = &*leader {
            if current == node_id {
                *leader = None;
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn get_leader(&self) -> RiptideResult<Option<String>> {
        let leader = self.leader.read().await;
        let now = SystemTime::now();

        if let Some((id, expires)) = &*leader {
            if now < *expires {
                return Ok(Some(id.clone()));
            }
        }

        Ok(None)
    }

    async fn health_check(&self) -> RiptideResult<bool> {
        Ok(true) // Always healthy in-memory
    }

    async fn stats(&self) -> RiptideResult<CoordinationStats> {
        let nodes = self.nodes.read().await;
        let channels = self.channels.read().await;
        let leader = self.leader.read().await;

        Ok(CoordinationStats {
            active_subscriptions: channels.len(),
            publish_rate: 0.0,
            receive_rate: 0.0,
            cluster_size: nodes.len(),
            current_leader: leader.as_ref().map(|(id, _)| id.clone()),
            metadata: HashMap::new(),
        })
    }
}

/// Simple glob pattern matching
fn pattern_matches(key: &str, pattern: &str) -> bool {
    // Simplified: * matches anything
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        key.starts_with(parts[0]) && key.ends_with(parts.last().unwrap())
    } else {
        key == pattern
    }
}
```

---

## 5. Dependency Injection Pattern

### 5.1 Constructor Injection

All components that need distributed coordination should accept it via constructor:

```rust
pub struct DistributedSync {
    /// Node configuration
    config: DistributedConfig,
    /// Coordination abstraction (NO direct Redis!)
    coordination: Arc<dyn DistributedCoordination>,
    /// Consensus manager
    consensus: Arc<ConsensusManager>,
    /// Leader election manager
    leader_election: Arc<LeaderElection>,
    /// Sync state
    state: Arc<RwLock<SyncState>>,
    /// Node ID
    node_id: String,
}

impl DistributedSync {
    /// Create with injected coordination
    pub fn new(
        config: DistributedConfig,
        coordination: Arc<dyn DistributedCoordination>,
    ) -> Self {
        Self {
            config: config.clone(),
            coordination,
            consensus: Arc::new(ConsensusManager::new(config.clone())),
            leader_election: Arc::new(LeaderElection::new(config.clone())),
            state: Arc::new(RwLock::new(SyncState::default())),
            node_id: config.node_id.clone(),
        }
    }
}
```

### 5.2 Builder Pattern (Alternative)

For complex initialization:

```rust
pub struct DistributedSyncBuilder {
    config: Option<DistributedConfig>,
    coordination: Option<Arc<dyn DistributedCoordination>>,
    consensus: Option<Arc<ConsensusManager>>,
}

impl DistributedSyncBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            coordination: None,
            consensus: None,
        }
    }

    pub fn with_config(mut self, config: DistributedConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_coordination(
        mut self,
        coordination: Arc<dyn DistributedCoordination>,
    ) -> Self {
        self.coordination = Some(coordination);
        self
    }

    pub fn build(self) -> Result<DistributedSync> {
        let config = self.config.ok_or("Config required")?;
        let coordination = self.coordination.ok_or("Coordination required")?;

        Ok(DistributedSync {
            config: config.clone(),
            coordination,
            consensus: Arc::new(ConsensusManager::new(config.clone())),
            leader_election: Arc::new(LeaderElection::new(config.clone())),
            state: Arc::new(RwLock::new(SyncState::default())),
            node_id: config.node_id.clone(),
        })
    }
}
```

### 5.3 Application Composition Root

```rust
// In main.rs or application initialization

use riptide_persistence::adapters::{RedisCoordination, MemoryCoordination};
use riptide_persistence::sync::DistributedSync;
use std::sync::Arc;

async fn initialize_production() -> anyhow::Result<DistributedSync> {
    let redis_url = std::env::var("REDIS_URL")?;
    let node_id = std::env::var("NODE_ID")?;

    // Create Redis adapter
    let coordination = Arc::new(
        RedisCoordination::new(&redis_url, node_id.clone(), Some("riptide".to_string())).await?
    ) as Arc<dyn DistributedCoordination>;

    // Create sync manager with injected coordination
    let config = DistributedConfig {
        node_id,
        cluster_nodes: vec![],
        heartbeat_interval_ms: 5000,
        leader_election_timeout_ms: 10000,
    };

    Ok(DistributedSync::new(config, coordination))
}

async fn initialize_testing() -> anyhow::Result<DistributedSync> {
    // Create in-memory adapter for tests
    let coordination = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;

    let config = DistributedConfig {
        node_id: "test-node".to_string(),
        cluster_nodes: vec![],
        heartbeat_interval_ms: 1000,
        leader_election_timeout_ms: 2000,
    };

    Ok(DistributedSync::new(config, coordination))
}
```

---

## 6. Migration Plan

### Phase 0: Trait Definition ✅ (This Document)
- [x] Define `DistributedCoordination` trait
- [x] Document method contracts
- [x] Specify error handling
- [x] Create usage examples

### Phase 1: Adapter Implementation
- [ ] Implement `RedisCoordination` adapter
- [ ] Implement `MemoryCoordination` adapter
- [ ] Unit tests for both adapters
- [ ] Integration tests with Redis

### Phase 2: DistributedSync Refactoring
- [ ] Add `coordination: Arc<dyn DistributedCoordination>` field
- [ ] Replace direct Redis calls with trait methods
- [ ] Update constructor for dependency injection
- [ ] Refactor `notify_operation` to use `publish`
- [ ] Refactor pattern operations to use `keys` and `delete_many`

### Phase 3: Testing & Validation
- [ ] Unit tests with `MemoryCoordination`
- [ ] Integration tests with `RedisCoordination`
- [ ] Performance benchmarks
- [ ] Load testing

### Phase 4: Documentation & Cleanup
- [ ] Update architecture docs
- [ ] Add usage examples
- [ ] Document migration guide
- [ ] Remove legacy Redis dependencies

---

## 7. Testing Strategy

### 7.1 Unit Tests (with MemoryCoordination)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::MemoryCoordination;

    #[tokio::test]
    async fn test_distributed_sync_with_memory_backend() {
        let coord = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;

        let config = DistributedConfig {
            node_id: "test-node".to_string(),
            cluster_nodes: vec![],
            heartbeat_interval_ms: 1000,
            leader_election_timeout_ms: 2000,
        };

        let sync = DistributedSync::new(config, coord);

        // Test pub/sub
        let operation = SyncOperation {
            id: "op-1".to_string(),
            operation_type: SyncOperationType::Set,
            key: "test-key".to_string(),
            value: Some(b"test-value".to_vec()),
            ttl: Some(60),
            timestamp: Utc::now(),
            origin_node: "test-node".to_string(),
            priority: 1,
        };

        sync.notify_operation(operation).await.unwrap();
    }

    #[tokio::test]
    async fn test_leader_election() {
        let coord = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;

        // Node 1 attempts leadership
        let acquired1 = coord
            .try_acquire_leadership("node-1", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired1);

        // Node 2 fails to acquire
        let acquired2 = coord
            .try_acquire_leadership("node-2", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(!acquired2);

        // Node 1 releases
        coord.release_leadership("node-1").await.unwrap();

        // Node 2 can now acquire
        let acquired3 = coord
            .try_acquire_leadership("node-2", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired3);
    }
}
```

### 7.2 Integration Tests (with RedisCoordination)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::adapters::RedisCoordination;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_redis_pub_sub() {
        let redis_url = "redis://localhost:6379";
        let coord = Arc::new(
            RedisCoordination::new(redis_url, "node-1".to_string(), None)
                .await
                .unwrap()
        ) as Arc<dyn DistributedCoordination>;

        // Subscribe
        let mut sub = coord.subscribe("test:*").await.unwrap();

        // Publish
        coord.publish("test:channel", b"hello").await.unwrap();

        // Receive
        let msg = sub.next().await.unwrap().unwrap();
        assert_eq!(msg.payload, b"hello");
    }
}
```

---

## 8. Future Enhancements

### 8.1 NATS Adapter

```rust
pub struct NatsCoordination {
    client: nats::aio::Connection,
}

// Implement DistributedCoordination for NATS JetStream
```

### 8.2 Kafka Adapter

```rust
pub struct KafkaCoordination {
    producer: rdkafka::producer::FutureProducer,
    consumer: rdkafka::consumer::StreamConsumer,
}

// Implement DistributedCoordination for Kafka
```

### 8.3 Distributed Tracing

Add OpenTelemetry instrumentation:

```rust
#[tracing::instrument(skip(self))]
async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()> {
    let span = tracing::info_span!("distributed_coordination.publish", channel = %channel);
    // ...
}
```

---

## 9. Architecture Decision Record

### ADR-001: Distributed Coordination Port Trait

**Status**: Proposed
**Date**: 2025-11-12
**Deciders**: System Architect, Development Team

#### Context

The `riptide-persistence` crate's `sync.rs` module directly depends on Redis for distributed coordination, violating hexagonal architecture principles and making testing difficult.

#### Decision

We will introduce a `DistributedCoordination` port trait that abstracts pub/sub messaging, distributed key operations, and cluster coordination primitives.

#### Rationale

1. **Hexagonal Architecture**: Clean separation between business logic and infrastructure
2. **Testability**: Enable unit tests with in-memory implementations
3. **Flexibility**: Support multiple backends (Redis, NATS, Kafka, in-memory)
4. **Maintainability**: Reduce coupling to specific technologies

#### Consequences

**Positive**:
- Testable distributed operations without external dependencies
- Can switch backends without changing business logic
- Cleaner, more maintainable codebase
- Better separation of concerns

**Negative**:
- Additional abstraction layer adds initial complexity
- Need to implement multiple adapters
- Slight performance overhead from trait dispatch (negligible)

**Neutral**:
- Requires refactoring existing code
- Team needs to learn dependency injection patterns

#### Alternatives Considered

1. **Keep Direct Redis**: Rejected - violates architecture principles
2. **Use Generic Parameters**: Rejected - less flexible than trait objects
3. **Monomorphization**: Rejected - code size bloat and less dynamic

---

## 10. Success Criteria

This design will be considered successful when:

1. ✅ **Zero Direct Redis Dependencies**: `sync.rs` has no `redis::` imports
2. ✅ **100% Unit Test Coverage**: All coordination operations testable with `MemoryCoordination`
3. ✅ **Performance Parity**: <5% performance regression vs. direct Redis
4. ✅ **Type Safety**: Compile-time guarantees via trait bounds
5. ✅ **Documentation**: Complete examples and migration guide
6. ✅ **Backend Flexibility**: Can swap Redis for alternatives in <50 LOC
7. ✅ **Thread Safety**: All implementations are `Send + Sync`

---

## Appendix A: Related Patterns

### Existing Port Traits

1. **CacheStorage** (`cache.rs`) - Key-value cache operations
2. **SessionStorage** (`session.rs`) - Session persistence
3. **Repository** (`repository.rs`) - Domain entity persistence

### Consistency Checks

- All port traits use `async_trait`
- All return `RiptideResult<T>`
- All are `Send + Sync`
- All have default method implementations where applicable

---

## Appendix B: File Locations

```
crates/
├── riptide-types/
│   └── src/
│       └── ports/
│           ├── mod.rs                      # Export DistributedCoordination
│           ├── coordination.rs             # NEW: Trait definition
│           ├── cache.rs                    # Existing: CacheStorage
│           └── session.rs                  # Existing: SessionStorage
│
└── riptide-persistence/
    └── src/
        ├── adapters/
        │   ├── mod.rs                      # Export adapters
        │   ├── redis_coordination.rs       # NEW: Redis adapter
        │   └── memory_coordination.rs      # NEW: In-memory adapter
        │
        ├── sync.rs                         # REFACTOR: Use trait
        └── lib.rs                          # Update exports
```

---

## Appendix C: Glossary

| Term | Definition |
|------|------------|
| **Port** | Abstract interface defining operations (hexagonal architecture) |
| **Adapter** | Concrete implementation of a port for specific infrastructure |
| **Coordination** | Distributed primitives (pub/sub, locking, leader election) |
| **Pub/Sub** | Publish-subscribe messaging pattern |
| **Leader Election** | Distributed algorithm for selecting a single coordinator |
| **Idempotent** | Operation that can be repeated without changing the result |
| **TTL** | Time-to-live; duration before expiration |

---

**END OF SPECIFICATION**
