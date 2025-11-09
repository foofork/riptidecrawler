//! # Riptide Types - Domain Layer
//!
//! Shared type definitions for the Riptide web scraping framework.
//!
//! This crate provides:
//! - **Domain types**: Common data structures used across all Riptide crates
//! - **Port traits**: Backend-agnostic interfaces for dependency inversion
//! - **Error types**: Consistent error handling across the system
//! - **Type aliases**: Common patterns and convenience types
//!
//! # Architecture
//!
//! This crate represents the **Domain Layer** in our hexagonal architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Domain Layer (riptide-types)           │
//! │  - Pure business logic                  │
//! │  - Port trait definitions               │
//! │  - NO infrastructure dependencies       │
//! └─────────────────────────────────────────┘
//!              ↑ uses                ↑ implements
//!              │                     │
//! ┌────────────┴──────────┐   ┌────┴──────────────┐
//! │ Application Layer     │   │ Infrastructure    │
//! │ (riptide-facade)      │   │ (riptide-*)       │
//! │ - Use-case workflows  │   │ - Concrete adapters│
//! └───────────────────────┘   └───────────────────┘
//! ```
//!
//! # Port Interfaces
//!
//! The `ports` module defines backend-agnostic traits:
//! - `Repository<T>`: Generic data persistence
//! - `EventBus`: Domain event publishing
//! - `IdempotencyStore`: Duplicate request prevention
//! - `BrowserDriver`, `PdfProcessor`, `SearchEngine`: Feature capabilities
//! - `Clock`, `Entropy`, `CacheStorage`: Infrastructure abstractions

// Public modules
pub mod component;
pub mod conditional;
pub mod config;
pub mod error;
pub mod extracted;
pub mod extractors;
pub mod http_types;
pub mod pipeline;
pub mod ports; // Port interfaces for hexagonal architecture
pub mod reliability; // Reliability configuration types (circuit breaker, retry)
pub mod secrets;
pub mod traits;
pub mod types;

// Re-export commonly used types at the crate root
pub use component::{ComponentId, ComponentMeta};
pub use conditional::{
    format_http_date, generate_etag, generate_weak_etag, parse_http_date, validate_cache,
    CacheValidation, ConditionalRequest, ConditionalResponse,
};
pub use config::{ChunkingConfig, ExtractionMode, OutputFormat, RenderMode, TopicChunkingConfig};
pub use error::{Result, RiptideError, StrategyError};
pub use extracted::{
    BasicExtractedDoc, ComponentInfo, ContentChunk, ExtractedContent, ExtractedDoc,
    ExtractionQuality, ExtractionStats, HealthStatus, ParserMetadata,
};
pub use http_types::{
    ContentMetadata, CrawledPage, ExtractOptions, ExtractRequest, ExtractResponse,
    ParserMetadataHttp, ResultMode, SearchQuery, SearchResponse, SearchResult, SpiderResultStats,
    SpiderResultUrls,
};
pub use pipeline::{
    CombinedPipelineExecutor, GateDecisionStats, PipelineExecutor, PipelineResult,
    PipelineRetryConfig, PipelineStats, StrategiesPipelineExecutor, StrategiesPipelineResult,
};
pub use reliability::{CircuitBreakerConfig, RetryConfig};
pub use traits::{Browser, Extractor, Scraper};
pub use types::{
    BrowserConfig, ExtractionConfig, ExtractionRequest, ExtractionResult, ScrapedContent,
    ScrapingOptions, Url,
};

// Re-export port traits for dependency injection
pub use ports::{
    BrowserDriver, BrowserSession, CacheStorage, Clock, DeterministicEntropy, DomainEvent, Entropy,
    EventBus, EventHandler, FakeClock, IdempotencyStore, IdempotencyToken, InMemoryCache,
    PdfMetadata, PdfProcessor, Pool, PoolError, PoolHealth, PoolStats, PooledResource, Repository,
    RepositoryFilter, ScriptResult, SearchDocument, SearchEngine, SearchQuery as PortSearchQuery,
    SearchResult as PortSearchResult, Session, SessionFilter, SessionStorage, SubscriptionId,
    SystemClock, SystemEntropy, Transaction, TransactionManager,
};

// Re-export third-party types for convenience
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use uuid::Uuid;
