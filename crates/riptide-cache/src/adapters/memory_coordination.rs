//! In-memory implementation of distributed coordination
//!
//! ⚠️ **WARNING: Single-Process Only** ⚠️
//!
//! This adapter provides an in-memory implementation of the `DistributedCoordination` trait
//! suitable for:
//! - Development and testing
//! - Single-instance deployments
//! - Integration tests
//!
//! # Limitations
//!
//! - **Not distributed**: All coordination is local to a single process
//! - **No persistence**: Data is lost when the process stops
//! - **No cross-process pub/sub**: Messages only reach subscribers in the same process
//! - **Fake leader election**: Always succeeds since there's only one "node"
//!
//! For production distributed deployments, use `RedisCoordination` or another
//! distributed coordination adapter.
//!
//! # Example
//!
//! ```rust
//! use riptide_cache::adapters::MemoryCoordination;
//! use riptide_types::ports::DistributedCoordination;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let coord = MemoryCoordination::new();
//!
//! // Cache operations
//! coord.cache_set("key", b"value", Some(Duration::from_secs(60))).await?;
//! let value = coord.cache_get("key").await?;
//!
//! // Pub/sub (within same process only)
//! coord.publish("events", b"message").await?;
//!
//! // Leader election (always succeeds)
//! let is_leader = coord.try_acquire_leadership(
//!     "election",
//!     "node-1",
//!     Duration::from_secs(30)
//! ).await?;
//! assert!(is_leader); // Always true in single-process mode
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

use riptide_types::error::riptide_error::RiptideError;
use riptide_types::ports::{
    CoordinationResult, DistributedCoordination, Subscriber, SubscriberMessage,
};

/// Type alias for cache entry: (value, optional expiration)
type CacheEntry = (Vec<u8>, Option<Instant>);

/// Node metadata for cluster tracking
#[derive(Debug, Clone)]
struct NodeMetadata {
    /// Arbitrary metadata
    metadata: HashMap<String, String>,
    /// When registration expires
    expires_at: Instant,
}

/// In-memory distributed coordination adapter
///
/// Provides local-only implementation of distributed coordination primitives.
/// Suitable for development, testing, and single-instance deployments.
///
/// # Thread Safety
///
/// All operations are thread-safe using `DashMap` and `RwLock`.
pub struct MemoryCoordination {
    /// In-memory cache with expiration tracking
    cache: Arc<DashMap<String, CacheEntry>>,
    /// Pub/sub broadcast channels by channel name
    channels: Arc<DashMap<String, broadcast::Sender<Vec<u8>>>>,
    /// Current leader for each election key
    leaders: Arc<DashMap<String, (String, Instant)>>,
    /// Active nodes in the "cluster"
    nodes: Arc<DashMap<String, NodeMetadata>>,
}

impl MemoryCoordination {
    /// Create a new in-memory coordination instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use riptide_cache::adapters::MemoryCoordination;
    ///
    /// let coord = MemoryCoordination::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            leaders: Arc::new(DashMap::new()),
            nodes: Arc::new(DashMap::new()),
        }
    }

    /// Get or create a broadcast channel
    fn get_or_create_channel(&self, channel: &str) -> broadcast::Sender<Vec<u8>> {
        self.channels
            .entry(channel.to_string())
            .or_insert_with(|| {
                // Channel capacity: 100 messages
                broadcast::channel(100).0
            })
            .clone()
    }

    /// Check if a key matches a pattern
    ///
    /// Supports simple glob patterns with '*' wildcard.
    fn matches_pattern(pattern: &str, key: &str) -> bool {
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let (prefix, suffix) = (parts[0], parts[1]);
                key.starts_with(prefix) && key.ends_with(suffix)
            } else {
                key.starts_with(pattern.trim_end_matches('*'))
            }
        } else {
            pattern == key
        }
    }

    /// Remove expired entries from cache
    fn cleanup_expired_cache(&self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, expires_at)| {
            expires_at.is_none_or(|exp| exp > now)
        });
    }

    /// Remove expired nodes
    fn cleanup_expired_nodes(&self) {
        let now = Instant::now();
        self.nodes.retain(|_, node| node.expires_at > now);
    }
}

impl Default for MemoryCoordination {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DistributedCoordination for MemoryCoordination {
    // ========================================================================
    // Pub/Sub Operations
    // ========================================================================

    async fn publish(&self, channel: &str, message: &[u8]) -> CoordinationResult<usize> {
        let sender = self.get_or_create_channel(channel);

        // Send returns the number of active receivers
        match sender.send(message.to_vec()) {
            Ok(count) => Ok(count),
            Err(_) => Ok(0), // No subscribers
        }
    }

    async fn subscribe(&self, channels: &[&str]) -> CoordinationResult<Box<dyn Subscriber>> {
        let mut receivers = Vec::new();

        for channel in channels {
            let sender = self.get_or_create_channel(channel);
            receivers.push((channel.to_string(), sender.subscribe()));
        }

        Ok(Box::new(MemorySubscriber {
            receivers,
        }))
    }

    // ========================================================================
    // Cache Operations
    // ========================================================================

    async fn cache_get(&self, key: &str) -> CoordinationResult<Option<Vec<u8>>> {
        // Clean expired entries
        self.cleanup_expired_cache();

        if let Some(entry) = self.cache.get(key) {
            let (value, expires_at) = entry.value();

            // Check if expired
            if let Some(exp) = expires_at {
                if *exp <= Instant::now() {
                    drop(entry);
                    self.cache.remove(key);
                    return Ok(None);
                }
            }

            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    async fn cache_set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<Duration>,
    ) -> CoordinationResult<()> {
        let expires_at = ttl.map(|d| Instant::now() + d);
        self.cache.insert(key.to_string(), (value.to_vec(), expires_at));
        Ok(())
    }

    async fn cache_delete(&self, key: &str) -> CoordinationResult<bool> {
        Ok(self.cache.remove(key).is_some())
    }

    async fn cache_delete_pattern(&self, pattern: &str) -> CoordinationResult<usize> {
        let mut deleted = 0;
        let keys_to_delete: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| Self::matches_pattern(pattern, entry.key()))
            .map(|entry| entry.key().clone())
            .collect();

        for key in keys_to_delete {
            if self.cache.remove(&key).is_some() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    async fn cache_exists(&self, key: &str) -> CoordinationResult<bool> {
        self.cleanup_expired_cache();

        if let Some(entry) = self.cache.get(key) {
            let (_, expires_at) = entry.value();
            if let Some(exp) = expires_at {
                if *exp <= Instant::now() {
                    drop(entry);
                    self.cache.remove(key);
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn cache_ttl(&self, key: &str) -> CoordinationResult<Option<Duration>> {
        if let Some(entry) = self.cache.get(key) {
            let (_, expires_at) = entry.value();
            if let Some(exp) = expires_at {
                let now = Instant::now();
                if *exp > now {
                    Ok(Some(*exp - now))
                } else {
                    drop(entry);
                    self.cache.remove(key);
                    Ok(None)
                }
            } else {
                Ok(None) // No expiration
            }
        } else {
            Ok(None)
        }
    }

    async fn cache_incr(
        &self,
        key: &str,
        amount: i64,
        ttl: Option<Duration>,
    ) -> CoordinationResult<i64> {
        let expires_at = ttl.map(|d| Instant::now() + d);

        let new_value = self.cache
            .entry(key.to_string())
            .and_modify(|(value, exp)| {
                // Try to parse existing value as i64
                if let Ok(s) = String::from_utf8(value.clone()) {
                    if let Ok(current) = s.parse::<i64>() {
                        let new = current + amount;
                        *value = new.to_string().into_bytes();
                        if expires_at.is_some() {
                            *exp = expires_at;
                        }
                        return;
                    }
                }
                // If parsing fails, treat as 0 + amount
                *value = amount.to_string().into_bytes();
                if expires_at.is_some() {
                    *exp = expires_at;
                }
            })
            .or_insert_with(|| (amount.to_string().into_bytes(), expires_at));

        // Parse the result
        let value_bytes = new_value.value().0.clone();
        let value_str = String::from_utf8(value_bytes)
            .map_err(|e| RiptideError::Cache(format!("Invalid counter value: {}", e)))?;
        value_str
            .parse::<i64>()
            .map_err(|e| RiptideError::Cache(format!("Failed to parse counter: {}", e)))
    }

    // ========================================================================
    // Leader Election
    // ========================================================================

    async fn try_acquire_leadership(
        &self,
        election_key: &str,
        node_id: &str,
        ttl: Duration,
    ) -> CoordinationResult<bool> {
        let expires_at = Instant::now() + ttl;

        // Try to insert or update if expired
        let mut acquired = false;
        self.leaders
            .entry(election_key.to_string())
            .and_modify(|(current_leader, exp)| {
                // If expired or same node, take leadership
                if *exp <= Instant::now() || current_leader == node_id {
                    *current_leader = node_id.to_string();
                    *exp = expires_at;
                    acquired = true;
                }
            })
            .or_insert_with(|| {
                acquired = true;
                (node_id.to_string(), expires_at)
            });

        Ok(acquired)
    }

    async fn release_leadership(
        &self,
        election_key: &str,
        node_id: &str,
    ) -> CoordinationResult<bool> {
        if let Some((_, (current_leader, _))) = self.leaders.remove(election_key) {
            Ok(current_leader == node_id)
        } else {
            Ok(false)
        }
    }

    async fn get_leader(&self, election_key: &str) -> CoordinationResult<Option<String>> {
        if let Some(entry) = self.leaders.get(election_key) {
            let (leader, expires_at) = entry.value();
            if *expires_at > Instant::now() {
                Ok(Some(leader.clone()))
            } else {
                drop(entry);
                self.leaders.remove(election_key);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    // ========================================================================
    // Cluster State
    // ========================================================================

    async fn register_node(
        &self,
        node_id: &str,
        metadata: HashMap<String, String>,
        ttl: Duration,
    ) -> CoordinationResult<()> {
        let expires_at = Instant::now() + ttl;
        self.nodes.insert(
            node_id.to_string(),
            NodeMetadata {
                metadata,
                expires_at,
            },
        );
        Ok(())
    }

    async fn get_active_nodes(&self) -> CoordinationResult<HashSet<String>> {
        self.cleanup_expired_nodes();
        Ok(self.nodes.iter().map(|entry| entry.key().clone()).collect())
    }

    async fn get_node_metadata(
        &self,
        node_id: &str,
    ) -> CoordinationResult<Option<HashMap<String, String>>> {
        if let Some(entry) = self.nodes.get(node_id) {
            let node = entry.value();
            if node.expires_at > Instant::now() {
                Ok(Some(node.metadata.clone()))
            } else {
                drop(entry);
                self.nodes.remove(node_id);
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn send_heartbeat(&self, node_id: &str, ttl: Duration) -> CoordinationResult<bool> {
        if let Some(mut entry) = self.nodes.get_mut(node_id) {
            entry.expires_at = Instant::now() + ttl;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// In-memory subscriber implementation
struct MemorySubscriber {
    /// Receivers for each subscribed channel
    receivers: Vec<(String, broadcast::Receiver<Vec<u8>>)>,
}

#[async_trait]
impl Subscriber for MemorySubscriber {
    async fn next_message(&mut self) -> CoordinationResult<Option<SubscriberMessage>> {
        if self.receivers.is_empty() {
            return Ok(None);
        }

        // Use select to wait for messages from any channel
        // For simplicity, we'll poll each receiver in turn
        for (channel, receiver) in &mut self.receivers {
            match receiver.try_recv() {
                Ok(payload) => {
                    return Ok(Some(SubscriberMessage::new(
                        channel.clone(),
                        payload,
                        Some(channel.clone()),
                    )));
                }
                Err(broadcast::error::TryRecvError::Empty) => continue,
                Err(broadcast::error::TryRecvError::Closed) => {
                    // Channel closed
                    continue;
                }
                Err(broadcast::error::TryRecvError::Lagged(_)) => {
                    // Messages were dropped due to slow consumer
                    continue;
                }
            }
        }

        // If no messages available, wait for the first one
        if let Some((channel, receiver)) = self.receivers.first_mut() {
            match receiver.recv().await {
                Ok(payload) => Ok(Some(SubscriberMessage::new(
                    channel.clone(),
                    payload,
                    Some(channel.clone()),
                ))),
                Err(_) => Ok(None), // All senders dropped
            }
        } else {
            Ok(None)
        }
    }

    async fn unsubscribe(&mut self) -> CoordinationResult<()> {
        self.receivers.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_operations() {
        let coord = MemoryCoordination::new();

        // Set and get
        coord.cache_set("key1", b"value1", None).await.unwrap();
        let value = coord.cache_get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Exists
        assert!(coord.cache_exists("key1").await.unwrap());
        assert!(!coord.cache_exists("nonexistent").await.unwrap());

        // Delete
        assert!(coord.cache_delete("key1").await.unwrap());
        assert!(!coord.cache_exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let coord = MemoryCoordination::new();

        // Set with short TTL
        coord
            .cache_set("key1", b"value1", Some(Duration::from_millis(100)))
            .await
            .unwrap();

        // Should exist initially
        assert!(coord.cache_exists("key1").await.unwrap());

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Should be gone
        assert!(!coord.cache_exists("key1").await.unwrap());
        assert_eq!(coord.cache_get("key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_cache_increment() {
        let coord = MemoryCoordination::new();

        // Increment new key
        let val = coord.cache_incr("counter", 1, None).await.unwrap();
        assert_eq!(val, 1);

        // Increment again
        let val = coord.cache_incr("counter", 5, None).await.unwrap();
        assert_eq!(val, 6);

        // Decrement
        let val = coord.cache_incr("counter", -3, None).await.unwrap();
        assert_eq!(val, 3);
    }

    #[tokio::test]
    async fn test_cache_pattern_delete() {
        let coord = MemoryCoordination::new();

        // Set multiple keys
        coord.cache_set("user:1", b"alice", None).await.unwrap();
        coord.cache_set("user:2", b"bob", None).await.unwrap();
        coord.cache_set("post:1", b"hello", None).await.unwrap();

        // Delete pattern
        let deleted = coord.cache_delete_pattern("user:*").await.unwrap();
        assert_eq!(deleted, 2);

        // Verify
        assert!(!coord.cache_exists("user:1").await.unwrap());
        assert!(!coord.cache_exists("user:2").await.unwrap());
        assert!(coord.cache_exists("post:1").await.unwrap());
    }

    #[tokio::test]
    async fn test_pubsub() {
        let coord = Arc::new(MemoryCoordination::new());

        // Subscribe
        let coord_clone = coord.clone();
        let handle = tokio::spawn(async move {
            let mut sub = coord_clone.subscribe(&["test:channel"]).await.unwrap();
            sub.next_message().await.unwrap()
        });

        // Give subscriber time to set up
        sleep(Duration::from_millis(50)).await;

        // Publish
        let count = coord.publish("test:channel", b"hello").await.unwrap();
        assert_eq!(count, 1);

        // Receive
        let msg = handle.await.unwrap().unwrap();
        assert_eq!(msg.channel, "test:channel");
        assert_eq!(msg.payload, b"hello");
    }

    #[tokio::test]
    async fn test_leader_election() {
        let coord = MemoryCoordination::new();

        // Acquire leadership
        let acquired = coord
            .try_acquire_leadership("election1", "node1", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired);

        // Check leader
        let leader = coord.get_leader("election1").await.unwrap();
        assert_eq!(leader, Some("node1".to_string()));

        // Another node tries to acquire
        let acquired = coord
            .try_acquire_leadership("election1", "node2", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(!acquired); // Should fail

        // Release
        let released = coord
            .release_leadership("election1", "node1")
            .await
            .unwrap();
        assert!(released);

        // Now node2 can acquire
        let acquired = coord
            .try_acquire_leadership("election1", "node2", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired);
    }

    #[tokio::test]
    async fn test_leader_election_expiration() {
        let coord = MemoryCoordination::new();

        // Acquire with short TTL
        coord
            .try_acquire_leadership("election1", "node1", Duration::from_millis(100))
            .await
            .unwrap();

        // Wait for expiration
        sleep(Duration::from_millis(150)).await;

        // Another node should be able to acquire
        let acquired = coord
            .try_acquire_leadership("election1", "node2", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(acquired);
    }

    #[tokio::test]
    async fn test_node_registration() {
        let coord = MemoryCoordination::new();

        // Register nodes
        let mut meta1 = HashMap::new();
        meta1.insert("host".to_string(), "server1".to_string());
        coord
            .register_node("node1", meta1, Duration::from_secs(10))
            .await
            .unwrap();

        let mut meta2 = HashMap::new();
        meta2.insert("host".to_string(), "server2".to_string());
        coord
            .register_node("node2", meta2, Duration::from_secs(10))
            .await
            .unwrap();

        // Get active nodes
        let nodes = coord.get_active_nodes().await.unwrap();
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains("node1"));
        assert!(nodes.contains("node2"));

        // Get metadata
        let meta = coord.get_node_metadata("node1").await.unwrap();
        assert_eq!(meta.unwrap().get("host").unwrap(), "server1");
    }

    #[tokio::test]
    async fn test_node_heartbeat() {
        let coord = MemoryCoordination::new();

        // Register with short TTL
        coord
            .register_node(
                "node1",
                HashMap::new(),
                Duration::from_millis(100),
            )
            .await
            .unwrap();

        // Send heartbeat to extend TTL
        sleep(Duration::from_millis(50)).await;
        let success = coord
            .send_heartbeat("node1", Duration::from_secs(10))
            .await
            .unwrap();
        assert!(success);

        // Wait past original TTL
        sleep(Duration::from_millis(100)).await;

        // Should still be active due to heartbeat
        let nodes = coord.get_active_nodes().await.unwrap();
        assert!(nodes.contains("node1"));
    }

    #[test]
    fn test_pattern_matching() {
        assert!(MemoryCoordination::matches_pattern("cache:*", "cache:user:123"));
        assert!(MemoryCoordination::matches_pattern("cache:*", "cache:post"));
        assert!(!MemoryCoordination::matches_pattern("cache:*", "user:123"));

        assert!(MemoryCoordination::matches_pattern("*:123", "user:123"));
        assert!(MemoryCoordination::matches_pattern("*:123", "post:123"));
        assert!(!MemoryCoordination::matches_pattern("*:123", "user:456"));

        assert!(MemoryCoordination::matches_pattern("exact", "exact"));
        assert!(!MemoryCoordination::matches_pattern("exact", "not-exact"));
    }
}
