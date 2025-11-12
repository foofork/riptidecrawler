//! Distributed coordination port for multi-instance synchronization
//!
//! This port defines the interface for distributed coordination mechanisms including:
//! - Pub/sub messaging for distributed events
//! - Distributed cache operations
//! - Leader election
//! - Cluster state management
//!
//! # Architecture
//!
//! The DistributedCoordination trait follows the Hexagonal Architecture pattern,
//! providing a backend-agnostic interface for distributed systems coordination.
//! Concrete implementations (e.g., Redis, etcd) are provided as adapters.
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::DistributedCoordination;
//!
//! async fn sync_cache(coord: &dyn DistributedCoordination) -> Result<()> {
//!     // Publish cache invalidation event
//!     coord.publish("cache:invalidate", b"user:123").await?;
//!
//!     // Subscribe to events
//!     let mut subscriber = coord.subscribe(&["cache:*"]).await?;
//!     while let Some(msg) = subscriber.next_message().await? {
//!         println!("Received: {}", msg.channel);
//!     }
//!
//!     Ok(())
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::error::riptide_error::RiptideError;

/// Result type for coordination operations
pub type CoordinationResult<T> = std::result::Result<T, RiptideError>;

/// Distributed coordination interface for multi-instance synchronization
///
/// Provides primitives for distributed systems including:
/// - Pub/sub messaging
/// - Distributed cache operations
/// - Leader election
/// - Cluster membership
///
/// # Thread Safety
///
/// All implementations must be thread-safe (Send + Sync) to support
/// concurrent usage in multi-threaded web servers.
#[async_trait]
pub trait DistributedCoordination: Send + Sync {
    // ========================================================================
    // Pub/Sub Operations
    // ========================================================================

    /// Publish a message to a channel
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name (supports patterns like "cache:*")
    /// * `message` - Message payload
    ///
    /// # Returns
    ///
    /// Number of subscribers that received the message
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// coord.publish("cache:invalidate", b"user:123").await?;
    /// ```
    async fn publish(&self, channel: &str, message: &[u8]) -> CoordinationResult<usize>;

    /// Subscribe to one or more channels
    ///
    /// # Arguments
    ///
    /// * `channels` - List of channel names (supports patterns)
    ///
    /// # Returns
    ///
    /// A subscriber that can receive messages
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut sub = coord.subscribe(&["cache:*", "events:*"]).await?;
    /// while let Some(msg) = sub.next_message().await? {
    ///     println!("Channel: {}, Data: {:?}", msg.channel, msg.payload);
    /// }
    /// ```
    async fn subscribe(&self, channels: &[&str]) -> CoordinationResult<Box<dyn Subscriber>>;

    // ========================================================================
    // Cache Operations
    // ========================================================================

    /// Get a value from the distributed cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// Value as bytes, or None if not found
    async fn cache_get(&self, key: &str) -> CoordinationResult<Option<Vec<u8>>>;

    /// Set a value in the distributed cache with TTL
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value as bytes
    /// * `ttl` - Time to live (None for no expiration)
    async fn cache_set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<Duration>,
    ) -> CoordinationResult<()>;

    /// Delete a key from the distributed cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key to delete
    ///
    /// # Returns
    ///
    /// True if key existed and was deleted
    async fn cache_delete(&self, key: &str) -> CoordinationResult<bool>;

    /// Delete multiple keys matching a pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - Key pattern (e.g., "cache:user:*")
    ///
    /// # Returns
    ///
    /// Number of keys deleted
    ///
    /// # Warning
    ///
    /// Pattern deletion can be expensive on large datasets.
    /// Use with caution in production.
    async fn cache_delete_pattern(&self, pattern: &str) -> CoordinationResult<usize>;

    /// Check if a key exists in the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// True if key exists
    async fn cache_exists(&self, key: &str) -> CoordinationResult<bool>;

    /// Get remaining TTL for a key
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// Remaining TTL, or None if key doesn't exist or has no expiration
    async fn cache_ttl(&self, key: &str) -> CoordinationResult<Option<Duration>>;

    /// Atomically increment a counter
    ///
    /// # Arguments
    ///
    /// * `key` - Counter key
    /// * `amount` - Amount to increment by
    /// * `ttl` - TTL to set if key doesn't exist
    ///
    /// # Returns
    ///
    /// New counter value after increment
    async fn cache_incr(
        &self,
        key: &str,
        amount: i64,
        ttl: Option<Duration>,
    ) -> CoordinationResult<i64>;

    // ========================================================================
    // Leader Election
    // ========================================================================

    /// Attempt to acquire leadership
    ///
    /// Uses a distributed lock mechanism to elect a leader among
    /// multiple instances.
    ///
    /// # Arguments
    ///
    /// * `election_key` - Unique key for this election
    /// * `node_id` - Unique identifier for this node
    /// * `ttl` - How long to hold leadership before re-election
    ///
    /// # Returns
    ///
    /// True if this node became the leader
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if coord.try_acquire_leadership("sync-leader", "node-1", Duration::from_secs(30)).await? {
    ///     println!("I am the leader!");
    ///     // Perform leader-only tasks
    /// }
    /// ```
    async fn try_acquire_leadership(
        &self,
        election_key: &str,
        node_id: &str,
        ttl: Duration,
    ) -> CoordinationResult<bool>;

    /// Release leadership
    ///
    /// # Arguments
    ///
    /// * `election_key` - Election key
    /// * `node_id` - This node's ID (must match acquiring node)
    ///
    /// # Returns
    ///
    /// True if leadership was released
    async fn release_leadership(
        &self,
        election_key: &str,
        node_id: &str,
    ) -> CoordinationResult<bool>;

    /// Check who the current leader is
    ///
    /// # Arguments
    ///
    /// * `election_key` - Election key
    ///
    /// # Returns
    ///
    /// Node ID of current leader, or None if no leader
    async fn get_leader(&self, election_key: &str) -> CoordinationResult<Option<String>>;

    // ========================================================================
    // Cluster State
    // ========================================================================

    /// Register this node in the cluster
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique node identifier
    /// * `metadata` - Node metadata (e.g., hostname, version)
    /// * `ttl` - How long registration is valid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let metadata = HashMap::from([
    ///     ("hostname".to_string(), "web-1".to_string()),
    ///     ("version".to_string(), "1.0.0".to_string()),
    /// ]);
    /// coord.register_node("node-1", metadata, Duration::from_secs(60)).await?;
    /// ```
    async fn register_node(
        &self,
        node_id: &str,
        metadata: HashMap<String, String>,
        ttl: Duration,
    ) -> CoordinationResult<()>;

    /// Get list of active nodes in the cluster
    ///
    /// # Returns
    ///
    /// Set of active node IDs
    async fn get_active_nodes(&self) -> CoordinationResult<HashSet<String>>;

    /// Get metadata for a specific node
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node identifier
    ///
    /// # Returns
    ///
    /// Node metadata, or None if node not found
    async fn get_node_metadata(
        &self,
        node_id: &str,
    ) -> CoordinationResult<Option<HashMap<String, String>>>;

    /// Send heartbeat to keep node registration alive
    ///
    /// # Arguments
    ///
    /// * `node_id` - Node identifier
    /// * `ttl` - New TTL to set
    ///
    /// # Returns
    ///
    /// True if heartbeat was successful
    async fn send_heartbeat(&self, node_id: &str, ttl: Duration) -> CoordinationResult<bool>;
}

/// Subscriber for receiving pub/sub messages
///
/// Implementations must be thread-safe and can be moved across threads.
#[async_trait]
pub trait Subscriber: Send {
    /// Receive the next message from subscribed channels
    ///
    /// # Returns
    ///
    /// Next message, or None if subscriber is closed
    async fn next_message(&mut self) -> CoordinationResult<Option<SubscriberMessage>>;

    /// Unsubscribe from all channels and close the subscriber
    async fn unsubscribe(&mut self) -> CoordinationResult<()>;
}

/// Message received from a pub/sub subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberMessage {
    /// Channel the message was published to
    pub channel: String,
    /// Message payload
    pub payload: Vec<u8>,
    /// Pattern that matched (if subscribed to a pattern)
    pub pattern: Option<String>,
}

impl SubscriberMessage {
    /// Create a new subscriber message
    pub fn new(channel: String, payload: Vec<u8>, pattern: Option<String>) -> Self {
        Self {
            channel,
            payload,
            pattern,
        }
    }

    /// Get payload as UTF-8 string
    ///
    /// # Returns
    ///
    /// String if payload is valid UTF-8, or None
    pub fn payload_string(&self) -> Option<String> {
        String::from_utf8(self.payload.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscriber_message_creation() {
        let msg = SubscriberMessage::new(
            "test:channel".to_string(),
            b"hello".to_vec(),
            Some("test:*".to_string()),
        );

        assert_eq!(msg.channel, "test:channel");
        assert_eq!(msg.payload, b"hello");
        assert_eq!(msg.pattern, Some("test:*".to_string()));
    }

    #[test]
    fn test_subscriber_message_payload_string() {
        let msg = SubscriberMessage::new("test".to_string(), b"hello".to_vec(), None);
        assert_eq!(msg.payload_string(), Some("hello".to_string()));

        // Invalid UTF-8
        let invalid_msg =
            SubscriberMessage::new("test".to_string(), vec![0xFF, 0xFE], None);
        assert!(invalid_msg.payload_string().is_none());
    }
}
