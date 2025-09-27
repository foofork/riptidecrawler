/*!
# RipTide Persistence Layer

Advanced persistence layer for RipTide with Redis/DragonflyDB backend, multi-tenancy support,
and comprehensive state management capabilities.

## Features

- **High-Performance Cache**: Redis/DragonflyDB backend with <5ms access time
- **TTL-based Invalidation**: Automatic cache expiration and cleanup
- **Cache Warming**: Startup optimization for frequently accessed data
- **Distributed Synchronization**: Multi-instance coordination
- **Multi-tenancy**: Complete tenant isolation with resource quotas
- **State Management**: Session persistence and hot configuration reload
- **Checkpoint/Restore**: Full system state preservation

## Example Usage

```rust
use riptide_persistence::{
    PersistentCacheManager,
    StateManager,
    TenantManager,
    PersistenceConfig
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = PersistenceConfig::default();

    // Initialize cache with performance targets
    let mut cache = PersistentCacheManager::new("redis://localhost:6379", config).await?;

    // Set data with TTL
    cache.set("key", &"value", None).await?;

    // Get with performance monitoring
    let value: Option<String> = cache.get("key").await?;

    Ok(())
}
```
*/

pub mod cache;
pub mod state;
pub mod tenant;
pub mod config;
pub mod errors;
pub mod metrics;
pub mod sync;

pub use cache::{
    PersistentCacheManager, CacheEntry, CacheMetadata,
    CacheStats, CacheWarmer, DistributedCache, CompressionInfo
};

pub use state::{
    StateManager, SessionState, ConfigurationManager, CheckpointManager,
    Checkpoint, StateSnapshot, HotReloadWatcher
};

pub use tenant::{
    TenantManager, ResourceUsage, ResourceUsageRecord, TenantSummary,
    BillingTracker, SecurityBoundary, TenantContext, TenantOwner, BillingPlan
};

pub use config::{
    PersistenceConfig, RedisConfig, DistributedConfig,
    PerformanceConfig, SecurityConfig
};

pub use errors::{PersistenceError, PersistenceResult};
pub use metrics::{PersistenceMetrics, PerformanceMetrics};
pub use sync::{DistributedSync, ConsensusManager, LeaderElection};

// Re-export commonly used types
pub use redis::{RedisError, ConnectionInfo};
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

/// Current version of the persistence layer
pub const PERSISTENCE_VERSION: &str = "1.0.0";

/// Performance targets
pub mod targets {
    /// Maximum cache access time in milliseconds
    pub const MAX_CACHE_ACCESS_MS: u64 = 5;

    /// Default TTL for cache entries (24 hours)
    pub const DEFAULT_TTL_SECONDS: u64 = 24 * 60 * 60;

    /// Maximum entry size (20MB)
    pub const MAX_ENTRY_SIZE_BYTES: usize = 20 * 1024 * 1024;

    /// Default compression threshold (1KB)
    pub const COMPRESSION_THRESHOLD_BYTES: usize = 1024;
}