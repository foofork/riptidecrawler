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
//! ### 1. Atomic Circuit Breaker (`circuit`)
//!
//! A lightweight, lock-free circuit breaker using atomic operations for high-performance scenarios:
//!
//! ```rust,ignore
//! use riptide_reliability::circuit::{CircuitBreaker, Config, RealClock};
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

pub mod circuit;
pub mod circuit_breaker;
pub mod engine_selection;
pub mod gate;
pub mod reliability;

// Re-export commonly used types
pub use circuit::{CircuitBreaker as AtomicCircuitBreaker, Clock, Config as CircuitConfig, State};
pub use circuit_breaker::{
    record_extraction_result, CircuitBreakerState, ExtractionResult as CircuitExtractionResult,
};
pub use engine_selection::{
    analyze_content, calculate_content_ratio, decide_engine, ContentAnalysis, Engine,
};
pub use gate::{decide, score, should_use_headless, Decision, GateFeatures};
pub use reliability::{
    ExtractionMode, ReliabilityConfig, ReliabilityMetrics, ReliableExtractor, WasmExtractor,
};
