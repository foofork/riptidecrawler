# Phase 0 Implementation Guide: DistributedCoordination Port

**For**: Development Team (Coder, Tester, Reviewer Agents)
**Date**: 2025-11-12
**Architect**: System Architecture Designer

---

## Quick Start

### What Was Designed?

A **`DistributedCoordination`** port trait that abstracts distributed cache synchronization, eliminating direct Redis dependencies from `sync.rs`.

### Why?

Current `sync.rs` violates hexagonal architecture:
- ❌ Direct Redis imports and method calls
- ❌ Impossible to unit test without infrastructure
- ❌ Backend lock-in (cannot use NATS, Kafka, etc.)

---

## File Structure

```
crates/
├── riptide-types/src/ports/
│   ├── mod.rs                       # Export DistributedCoordination
│   └── coordination.rs              # NEW: Trait definition (PRIMARY WORK)
│
└── riptide-persistence/src/
    ├── adapters/
    │   ├── mod.rs                   # Export adapters
    │   ├── redis_coordination.rs    # NEW: Redis implementation
    │   └── memory_coordination.rs   # NEW: In-memory implementation
    │
    └── sync.rs                      # REFACTOR: Remove Redis, use trait
```

---

## Implementation Checklist

### Phase 1: Trait Definition (First Task)

**File**: `crates/riptide-types/src/ports/coordination.rs`

```rust
// Copy trait definition from specification document
// Located at: /workspaces/riptidecrawler/docs/architecture/distributed-coordination-trait.md
// Section 2.1: Trait Definition

use crate::error::Result as RiptideResult;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::time::Duration;

#[async_trait]
pub trait DistributedCoordination: Send + Sync {
    // Pub/sub messaging
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()>;
    async fn subscribe(&self, pattern: &str) -> RiptideResult<MessageStream>;
    async fn publish_with_ack(&self, channel: &str, message: &[u8], timeout: Duration) -> RiptideResult<usize>;

    // Distributed key operations
    async fn keys(&self, pattern: &str) -> RiptideResult<Vec<String>>;
    async fn delete_many(&self, keys: &[&str]) -> RiptideResult<usize>;
    async fn delete(&self, key: &str) -> RiptideResult<bool> { /* default impl */ }
    async fn flush_all(&self) -> RiptideResult<()>;

    // Cluster coordination
    async fn register_node(&self, node_id: &str, metadata: Option<&[u8]>) -> RiptideResult<()>;
    async fn heartbeat(&self, node_id: &str, ttl: Duration) -> RiptideResult<()>;
    async fn list_nodes(&self) -> RiptideResult<Vec<String>>;
    async fn get_node_metadata(&self, node_id: &str) -> RiptideResult<Option<Vec<u8>>>;

    // Leader election
    async fn try_acquire_leadership(&self, node_id: &str, ttl: Duration) -> RiptideResult<bool>;
    async fn release_leadership(&self, node_id: &str) -> RiptideResult<bool>;
    async fn get_leader(&self) -> RiptideResult<Option<String>>;

    // Health & diagnostics
    async fn health_check(&self) -> RiptideResult<bool> { /* default impl */ }
    async fn stats(&self) -> RiptideResult<CoordinationStats> { /* default impl */ }
}

// Supporting types
pub struct Message {
    pub channel: String,
    pub payload: Vec<u8>,
    pub timestamp: Option<std::time::SystemTime>,
}

pub type MessageStream = Pin<Box<dyn Stream<Item = RiptideResult<Message>> + Send>>;

#[derive(Debug, Clone, Default)]
pub struct CoordinationStats {
    pub active_subscriptions: usize,
    pub publish_rate: f64,
    pub receive_rate: f64,
    pub cluster_size: usize,
    pub current_leader: Option<String>,
    pub metadata: std::collections::HashMap<String, String>,
}
```

**Export from mod.rs**:

```rust
// In crates/riptide-types/src/ports/mod.rs
pub mod coordination;
pub use coordination::{DistributedCoordination, Message, MessageStream, CoordinationStats};
```

**Cargo.toml dependencies** (add if not present):

```toml
[dependencies]
async-trait = "0.1"
futures = "0.3"
```

---

### Phase 2: RedisCoordination Adapter

**File**: `crates/riptide-persistence/src/adapters/redis_coordination.rs`

**Implementation Template**:

```rust
//! Redis implementation of DistributedCoordination
//!
//! See full implementation in:
//! /workspaces/riptidecrawler/docs/architecture/distributed-coordination-trait.md
//! Section 4.1: Redis Adapter

use riptide_types::ports::coordination::{
    DistributedCoordination, Message, MessageStream, CoordinationStats
};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RedisCoordination {
    pool: Arc<Mutex<MultiplexedConnection>>,
    node_id: String,
    key_prefix: String,
}

impl RedisCoordination {
    pub async fn new(
        redis_url: &str,
        node_id: String,
        key_prefix: Option<String>,
    ) -> RiptideResult<Self> {
        // Implementation: Create Redis client and connection
    }

    fn namespaced_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl DistributedCoordination for RedisCoordination {
    // Implement all trait methods
    // Refer to specification document for complete implementation
}
```

**Critical Methods**:

1. **publish**: Use `conn.publish::<_, _, ()>(channel, message)`
2. **subscribe**: Create dedicated connection, use `client.get_async_pubsub()`
3. **keys**: Use `redis::cmd("KEYS").arg(pattern)`
4. **delete_many**: Use `conn.del(&keys)`
5. **try_acquire_leadership**: Use SET NX EX pattern (atomic)

**Testing Strategy**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_redis_publish_subscribe() {
        let coord = RedisCoordination::new("redis://localhost:6379", "node-1".to_string(), None)
            .await
            .unwrap();

        let mut sub = coord.subscribe("test:*").await.unwrap();
        coord.publish("test:channel", b"hello").await.unwrap();

        let msg = sub.next().await.unwrap().unwrap();
        assert_eq!(msg.payload, b"hello");
    }

    // Add more integration tests
}
```

---

### Phase 3: MemoryCoordination Adapter

**File**: `crates/riptide-persistence/src/adapters/memory_coordination.rs`

**Implementation Template**:

```rust
//! In-memory implementation for testing
//!
//! See full implementation in:
//! /workspaces/riptidecrawler/docs/architecture/distributed-coordination-trait.md
//! Section 4.2: In-Memory Adapter

use riptide_types::ports::coordination::{
    DistributedCoordination, Message, MessageStream, CoordinationStats
};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

pub struct MemoryCoordination {
    store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>,
    nodes: Arc<RwLock<HashMap<String, (Vec<u8>, SystemTime)>>>,
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
    // Implement all trait methods with in-memory data structures
}
```

**Testing Strategy**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_publish_subscribe() {
        let coord = MemoryCoordination::new();

        let mut sub = coord.subscribe("test:*").await.unwrap();
        coord.publish("test:channel", b"hello").await.unwrap();

        let msg = sub.next().await.unwrap().unwrap();
        assert_eq!(msg.payload, b"hello");
    }

    #[tokio::test]
    async fn test_leader_election() {
        let coord = MemoryCoordination::new();

        // Node 1 acquires leadership
        assert!(coord.try_acquire_leadership("node-1", Duration::from_secs(10)).await.unwrap());

        // Node 2 cannot acquire while node 1 is leader
        assert!(!coord.try_acquire_leadership("node-2", Duration::from_secs(10)).await.unwrap());

        // Node 1 releases
        coord.release_leadership("node-1").await.unwrap();

        // Node 2 can now acquire
        assert!(coord.try_acquire_leadership("node-2", Duration::from_secs(10)).await.unwrap());
    }

    // Add comprehensive unit tests
}
```

---

### Phase 4: Refactor sync.rs

**File**: `crates/riptide-persistence/src/sync.rs`

**Changes Required**:

1. **Remove Direct Redis Imports**:
```rust
// REMOVE these lines:
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};

// ADD this:
use riptide_types::ports::coordination::DistributedCoordination;
use std::sync::Arc;
```

2. **Update DistributedSync Structure**:
```rust
// BEFORE:
pub struct DistributedSync {
    config: DistributedConfig,
    pool: Arc<Mutex<MultiplexedConnection>>,  // ❌ Remove
    consensus: Arc<ConsensusManager>,
    // ...
}

// AFTER:
pub struct DistributedSync {
    config: DistributedConfig,
    coordination: Arc<dyn DistributedCoordination>,  // ✅ Add
    consensus: Arc<ConsensusManager>,
    // ...
}
```

3. **Update Constructor**:
```rust
// BEFORE:
impl DistributedSync {
    pub async fn new(redis_url: &str, config: DistributedConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        // ...
    }
}

// AFTER:
impl DistributedSync {
    pub fn new(
        config: DistributedConfig,
        coordination: Arc<dyn DistributedCoordination>,
    ) -> Self {
        Self {
            config: config.clone(),
            coordination,
            consensus: Arc::new(ConsensusManager::new(config.clone())),
            // ...
        }
    }
}
```

4. **Refactor notify_operation (Line 172-184)**:
```rust
// BEFORE:
pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
    let channel = format!("riptide:sync:{}", "operations");
    let message = serde_json::to_string(&operation)?;

    let mut conn = self.pool.lock().await;  // ❌ Direct Redis
    conn.publish::<_, _, ()>(&channel, &message).await?;
    Ok(())
}

// AFTER:
pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
    let channel = format!("riptide:sync:operations");
    let message = serde_json::to_vec(&operation)?;

    self.coordination.publish(&channel, &message).await?;  // ✅ Port method
    Ok(())
}
```

5. **Refactor apply_invalidate_operation (Line 330-351)**:
```rust
// BEFORE:
async fn apply_invalidate_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
    let mut conn = self.pool.lock().await;

    let keys: Vec<String> = redis::cmd("KEYS")  // ❌ Direct Redis
        .arg(&operation.key)
        .query_async(&mut *conn)
        .await
        .unwrap_or_default();

    if !keys.is_empty() {
        let deleted: u64 = conn.del(&keys).await?;
        debug!(pattern = %operation.key, deleted = deleted, "Pattern invalidation applied");
    }
    Ok(())
}

// AFTER:
async fn apply_invalidate_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
    let keys = self.coordination.keys(&operation.key).await?;  // ✅ Port method

    if !keys.is_empty() {
        let key_refs: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        let deleted = self.coordination.delete_many(&key_refs).await?;  // ✅ Port method
        debug!(pattern = %operation.key, deleted = deleted, "Pattern invalidation applied");
    }
    Ok(())
}
```

6. **Refactor apply_clear_operation (Line 353-360)**:
```rust
// BEFORE:
async fn apply_clear_operation(&self, _operation: &SyncOperation) -> PersistenceResult<()> {
    let mut conn = self.pool.lock().await;
    let _: () = redis::cmd("FLUSHDB").query_async(&mut *conn).await?;  // ❌ Direct Redis
    info!("Cache clear operation applied");
    Ok(())
}

// AFTER:
async fn apply_clear_operation(&self, _operation: &SyncOperation) -> PersistenceResult<()> {
    self.coordination.flush_all().await?;  // ✅ Port method
    info!("Cache clear operation applied");
    Ok(())
}
```

---

## Dependency Injection Pattern

### Application Initialization (main.rs or lib.rs)

```rust
use riptide_persistence::adapters::{RedisCoordination, MemoryCoordination};
use riptide_persistence::sync::DistributedSync;
use riptide_persistence::config::DistributedConfig;
use std::sync::Arc;

/// Production initialization with Redis
async fn initialize_production_sync() -> anyhow::Result<DistributedSync> {
    let redis_url = std::env::var("REDIS_URL")?;
    let node_id = std::env::var("NODE_ID")?;

    // Create Redis adapter
    let coordination = Arc::new(
        RedisCoordination::new(&redis_url, node_id.clone(), Some("riptide".to_string())).await?
    ) as Arc<dyn DistributedCoordination>;

    // Create configuration
    let config = DistributedConfig {
        node_id,
        cluster_nodes: vec![],
        heartbeat_interval_ms: 5000,
        leader_election_timeout_ms: 10000,
    };

    // Inject coordination into sync manager
    Ok(DistributedSync::new(config, coordination))
}

/// Testing initialization with in-memory coordination
async fn initialize_test_sync() -> DistributedSync {
    // Create in-memory adapter
    let coordination = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;

    let config = DistributedConfig {
        node_id: "test-node".to_string(),
        cluster_nodes: vec![],
        heartbeat_interval_ms: 1000,
        leader_election_timeout_ms: 2000,
    };

    DistributedSync::new(config, coordination)
}
```

---

## Testing Requirements

### Unit Tests (with MemoryCoordination)

**Target**: 100% coverage of `sync.rs` business logic

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::MemoryCoordination;

    fn create_test_sync() -> DistributedSync {
        let coord = Arc::new(MemoryCoordination::new()) as Arc<dyn DistributedCoordination>;
        let config = DistributedConfig {
            node_id: "test-node".to_string(),
            cluster_nodes: vec![],
            heartbeat_interval_ms: 1000,
            leader_election_timeout_ms: 2000,
        };
        DistributedSync::new(config, coord)
    }

    #[tokio::test]
    async fn test_notify_operation() {
        let sync = create_test_sync();

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
    async fn test_pattern_invalidation() {
        let sync = create_test_sync();

        let operation = SyncOperation {
            id: "op-2".to_string(),
            operation_type: SyncOperationType::InvalidatePattern,
            key: "cache:*".to_string(),
            value: None,
            ttl: None,
            timestamp: Utc::now(),
            origin_node: "test-node".to_string(),
            priority: 1,
        };

        sync.apply_invalidate_operation(&operation).await.unwrap();
    }

    // Add more tests for all operations
}
```

### Integration Tests (with RedisCoordination)

**Target**: Verify real Redis interactions

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::adapters::RedisCoordination;

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_distributed_sync_with_redis() {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let coord = Arc::new(
            RedisCoordination::new(&redis_url, "node-1".to_string(), None)
                .await
                .unwrap()
        ) as Arc<dyn DistributedCoordination>;

        let config = DistributedConfig {
            node_id: "node-1".to_string(),
            cluster_nodes: vec![],
            heartbeat_interval_ms: 5000,
            leader_election_timeout_ms: 10000,
        };

        let sync = DistributedSync::new(config, coord);

        // Test operations against real Redis
        let operation = SyncOperation { /* ... */ };
        sync.notify_operation(operation).await.unwrap();
    }
}
```

---

## Common Pitfalls & Solutions

### Pitfall 1: Forgetting async_trait

❌ **Wrong**:
```rust
pub trait DistributedCoordination {
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()>;
}
```

✅ **Correct**:
```rust
#[async_trait]
pub trait DistributedCoordination: Send + Sync {
    async fn publish(&self, channel: &str, message: &[u8]) -> RiptideResult<()>;
}
```

### Pitfall 2: Missing Send + Sync Bounds

❌ **Wrong**:
```rust
pub trait DistributedCoordination {
    // Missing Send + Sync
}
```

✅ **Correct**:
```rust
#[async_trait]
pub trait DistributedCoordination: Send + Sync {
    // Can be shared across threads
}
```

### Pitfall 3: Incorrect Arc Usage

❌ **Wrong**:
```rust
let coord = RedisCoordination::new(...).await?;
let sync = DistributedSync::new(config, coord); // Type mismatch
```

✅ **Correct**:
```rust
let coord = Arc::new(RedisCoordination::new(...).await?) as Arc<dyn DistributedCoordination>;
let sync = DistributedSync::new(config, coord);
```

### Pitfall 4: Not Handling Stream Properly

❌ **Wrong**:
```rust
let sub = coord.subscribe("test").await?;
let msg = sub.await; // Stream is not a future
```

✅ **Correct**:
```rust
use futures::StreamExt;

let mut sub = coord.subscribe("test").await?;
while let Some(result) = sub.next().await {
    let msg = result?;
    // Process message
}
```

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| `publish` | <1ms | Latency |
| `subscribe` creation | <10ms | Setup time |
| `keys` (100 keys) | <5ms | Query time |
| `delete_many` (100 keys) | <5ms | Delete time |
| Trait dispatch overhead | <1% | vs. direct Redis |

---

## Quality Gates

Before merging, ensure:

- [ ] All trait methods documented with examples
- [ ] Both adapters fully implemented
- [ ] Unit tests pass with MemoryCoordination
- [ ] Integration tests pass with RedisCoordination (if Redis available)
- [ ] Zero clippy warnings: `cargo clippy -p riptide-persistence -- -D warnings`
- [ ] Zero direct Redis imports in `sync.rs`
- [ ] Performance regression <5% vs. baseline
- [ ] Code formatted: `cargo fmt -p riptide-persistence`

---

## Next Steps After Implementation

1. **Phase 2 Refactoring**:
   - Update all `sync.rs` method calls
   - Update `ConsensusManager` and `LeaderElection` if needed
   - Remove legacy Redis code

2. **Phase 3 Testing**:
   - Achieve 100% unit test coverage
   - Run integration tests against Redis
   - Performance benchmarking

3. **Phase 4 Documentation**:
   - Update README with usage examples
   - Add migration guide for existing code
   - Document configuration options

---

## Questions?

**Architecture Spec**: `/workspaces/riptidecrawler/docs/architecture/distributed-coordination-trait.md`
**ADR**: `/workspaces/riptidecrawler/docs/architecture/adr/001-distributed-coordination-port.md`
**Memory Store**: Check `.swarm/memory.db` for design decisions

**Contact**: System Architect via coordination hooks

---

**END OF IMPLEMENTATION GUIDE**
