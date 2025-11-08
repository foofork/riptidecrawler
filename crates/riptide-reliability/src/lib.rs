//! # Riptide Reliability
//!
//! Reliability patterns and fault tolerance mechanisms for the Riptide web scraping framework.
//!
//! This crate provides:
//! - **Circuit Breakers**: Prevent cascading failures by breaking circuits when error rates exceed thresholds
//! - **Gates**: Intelligent routing decisions for extraction strategies (fast vs headless)
//! - **Reliability Patterns**: Retry logic, timeout handling, and graceful degradation
//!
//! ## Circuit Breakers
//!
//! Two circuit breaker implementations are provided:
//!
//! ### 1. Atomic Circuit Breaker (`circuit_breaker`)
//!
//! A lightweight, lock-free circuit breaker using atomic operations for high-performance scenarios:
//!
//! ```rust,ignore
//! use riptide_reliability::circuit_breaker::{CircuitBreaker, Config, RealClock};
//! use std::sync::Arc;
//!
//! let cb = CircuitBreaker::new(
//!     Config {
//!         failure_threshold: 5,
//!         open_cooldown_ms: 30_000,
//!         half_open_max_in_flight: 3,
//!     },
//!     Arc::new(RealClock),
//! );
//!
//! // Try to acquire permission to proceed
//! match cb.try_acquire() {
//!     Ok(permit) => {
//!         // Proceed with operation
//!         // Report success/failure
//!         cb.on_success();
//!     }
//!     Err(msg) => {
//!         // Circuit is open, fail fast
//!     }
//! }
//! ```
//!
//! ### 2. State-Based Circuit Breaker (`circuit_breaker`)
//!
//! A more feature-rich circuit breaker with event bus integration and detailed state tracking:
//!
//! ```rust,ignore
//! use riptide_reliability::circuit_breaker::{CircuitBreakerState, record_extraction_result, ExtractionResult};
//! use std::sync::Arc;
//! use tokio::sync::Mutex;
//!
//! let state = Arc::new(Mutex::new(CircuitBreakerState::default()));
//! let metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));
//!
//! // Record extraction result
//! record_extraction_result(
//!     &metrics,
//!     &state,
//!     &None, // Optional event bus
//!     ExtractionResult {
//!         pool_id: "pool-1".to_string(),
//!         failure_threshold: 50,
//!         timeout_duration: 5000,
//!         success: true,
//!         duration: Duration::from_millis(100),
//!     },
//! ).await;
//! ```
//!
//! ## Gates
//!
//! Intelligent decision-making for extraction strategy selection:
//!
//! ```rust,ignore
//! use riptide_reliability::gate::{GateFeatures, decide, Decision};
//!
//! let features = GateFeatures {
//!     html_bytes: 10000,
//!     visible_text_chars: 5000,
//!     p_count: 15,
//!     article_count: 1,
//!     has_og: true,
//!     script_bytes: 500,
//!     spa_markers: 0,
//!     h1h2_count: 3,
//!     has_jsonld_article: true,
//!     domain_prior: 0.7,
//! };
//!
//! match decide(&features, 0.7, 0.3) {
//!     Decision::Raw => {
//!         // Use fast extraction
//!     }
//!     Decision::ProbesFirst => {
//!         // Try fast first, fallback to headless
//!     }
//!     Decision::Headless => {
//!         // Use headless browser rendering
//!     }
//! }
//! ```
//!
//! ## Reliability Patterns
//!
//! End-to-end reliability orchestration with retry, timeout, and graceful degradation:
//!
//! ```rust,ignore
//! use riptide_reliability::reliability::{ReliableExtractor, ReliabilityConfig, ExtractionMode};
//!
//! let config = ReliabilityConfig::default();
//! let extractor = ReliableExtractor::new(config)?;
//!
//! let result = extractor.extract_with_reliability(
//!     "https://example.com/article",
//!     ExtractionMode::ProbesFirst,
//!     &wasm_extractor,
//!     Some("http://headless-service:3000"),
//! ).await?;
//! ```
//!
//! ## Adaptive Timeouts
//!
//! Intelligent timeout management that learns from historical patterns:
//!
//! ```rust,ignore
//! use riptide_reliability::timeout::{get_global_timeout_manager, TimeoutConfig};
//! use std::time::Instant;
//!
//! // Get global timeout manager
//! let manager = get_global_timeout_manager().await?;
//!
//! // Get adaptive timeout for URL
//! let timeout = manager.get_timeout("https://example.com/page").await;
//!
//! // Record success/failure to learn
//! let start = Instant::now();
//! // ... perform request ...
//! manager.record_success("https://example.com/page", start.elapsed()).await;
//! ```

// Circuit breaker canonical implementation is in riptide-utils (shared infrastructure)
pub mod circuit_breaker_pool;
pub mod engine_selection;
pub mod gate;
pub mod http_client;
#[cfg(feature = "reliability-patterns")]
pub mod reliability;
pub mod timeout;

// Re-export canonical circuit breaker from riptide-utils (shared infrastructure)
// ARCHITECTURE: Circuit breaker lives in riptide-utils to avoid circular dependencies
// while keeping infrastructure separate from domain (riptide-types)
pub use riptide_utils::circuit_breaker::{
    guarded_call, CircuitBreaker, Clock, Config as CircuitConfig, RealClock, State,
};

// Re-export extractor traits from riptide-types
// These traits break circular dependencies via dependency injection
pub use riptide_types::extractors::{HtmlParser, WasmExtractor};

// Backward compatibility aliases for circuit breaker
pub use guarded_call as types_guarded_call;
pub use CircuitBreaker as AtomicCircuitBreaker;
pub use CircuitBreaker as TypesCircuitBreaker;
pub use CircuitConfig as TypesCircuitConfig;
pub use Clock as TypesClock;
pub use RealClock as TypesRealClock;
pub use State as TypesCircuitState;

pub use circuit_breaker_pool::{
    record_extraction_result, CircuitBreakerState, ExtractionResult as CircuitExtractionResult,
};
pub use engine_selection::{
    analyze_content, calculate_content_ratio, decide_engine, decide_engine_with_flags,
    ContentAnalysis, Engine, EngineCacheable, EngineSelectionFlags,
};
pub use gate::{decide, score, should_use_headless, Decision, GateFeatures};
pub use http_client::{FetchOptions, HttpClientService, HttpConfig};
#[cfg(feature = "reliability-patterns")]
pub use reliability::{
    ExtractionMode, ReliabilityConfig, ReliabilityMetrics, ReliabilityMetricsRecorder,
    ReliableExtractor, WasmExtractor as ReliabilityWasmExtractor,
};
pub use timeout::{
    get_global_timeout_manager, AdaptiveTimeoutManager, TimeoutConfig, TimeoutProfile, TimeoutStats,
};
