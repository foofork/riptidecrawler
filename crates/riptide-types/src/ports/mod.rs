//! Port interfaces for dependency inversion (Hexagonal Architecture)
//!
//! This module provides backend-agnostic trait definitions that enable
//! dependency inversion and facilitate testing. Concrete implementations
//! are provided in their respective infrastructure crates.
//!
//! # Architecture Pattern
//!
//! These ports follow the **Hexagonal Architecture** (Ports & Adapters) pattern:
//!
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines ports (traits)
//! Infrastructure Layer (riptide-*)
//!     ↓ implements adapters
//! Composition Root (riptide-api)
//!     ↓ wires dependencies
//! ```
//!
//! # Available Ports
//!
//! ## Data Persistence
//! - **repository**: Generic repository pattern for domain entities
//! - **events**: Event bus for domain event publishing
//! - **idempotency**: Idempotency store for duplicate prevention
//!
//! ## Features
//! - **features**: Browser automation, PDF processing, search engine
//!
//! ## Infrastructure
//! - **infrastructure**: Clock, entropy, and cache abstractions
//! - **cache**: Cache storage (from Phase 0)
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{Repository, EventBus, CacheStorage};
//!
//! async fn example(
//!     repo: &dyn Repository<User>,
//!     events: &dyn EventBus,
//!     cache: &dyn CacheStorage,
//! ) -> Result<()> {
//!     // Use ports without knowing concrete implementations
//!     let user = repo.find_by_id("user-123").await?;
//!     events.publish(user_event).await?;
//!     cache.set("key", data, ttl).await?;
//!     Ok(())
//! }
//! ```

// Phase 0 ports
pub mod cache;
pub mod memory_cache;

// Phase 1 ports
pub mod events;
pub mod features;
pub mod idempotency;
pub mod infrastructure;
pub mod repository;
pub mod session;

// Sprint 1.5 ports
pub mod health;
pub mod http;
pub mod metrics;

// Sprint 4.7 ports
pub mod pool;

// Sprint 4.3 ports
pub mod streaming;

// Re-export all ports for convenience
pub use cache::{CacheStats, CacheStorage};
pub use events::{DomainEvent, EventBus, EventHandler, SubscriptionId};
pub use features::{
    BrowserDriver, BrowserSession, PdfMetadata, PdfProcessor, ScriptResult, SearchDocument,
    SearchEngine, SearchQuery, SearchResult,
};
pub use health::{HealthCheck, HealthRegistry, HealthStatus};
pub use http::{HttpClient, HttpRequest, HttpResponse};
pub use idempotency::{IdempotencyStore, IdempotencyToken};
pub use infrastructure::{
    Clock, DeterministicEntropy, Entropy, FakeClock, SystemClock, SystemEntropy,
};
pub use memory_cache::InMemoryCache;
pub use metrics::{BusinessMetrics, MetricsCollector, MetricsRegistry};
pub use pool::{Pool, PoolError, PoolHealth, PoolStats, PooledResource};
pub use repository::{Repository, RepositoryFilter, Transaction, TransactionManager};
pub use session::{Session, SessionFilter, SessionStorage};
pub use streaming::{
    DeepSearchMetadata, DeepSearchResultData, ProcessedResult, StreamCompletionSummary,
    StreamConfig, StreamErrorData, StreamEvent, StreamLifecycle, StreamMetadata, StreamMetrics,
    StreamProcessor, StreamProgress, StreamResult, StreamResultData, StreamState, StreamSummary,
    StreamingTransport,
};
