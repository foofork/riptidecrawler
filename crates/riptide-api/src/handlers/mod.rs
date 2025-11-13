// Module declarations
#[cfg(feature = "persistence")]
pub mod admin;
#[cfg(feature = "browser")]
pub mod browser;
#[cfg(feature = "extraction")]
pub mod chunking;
#[cfg(feature = "spider")]
pub mod crawl;
#[cfg(feature = "search")]
pub mod deepsearch;
pub mod engine_selection;
#[cfg(feature = "extraction")]
pub mod extract;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod health;
#[cfg(feature = "llm")]
pub mod llm;
pub mod memory; // Memory profiling endpoint for production observability
pub mod monitoring;
pub mod pdf;
pub mod pipeline_metrics; // Enhanced pipeline metrics visualization
pub mod pipeline_phases;
#[cfg(feature = "llm")]
pub mod profiles;
pub mod profiling;
#[cfg(feature = "browser")]
pub mod render;
pub mod resources;
#[cfg(feature = "search")]
pub mod search;
pub mod sessions;
pub mod shared; // Shared utilities for handlers (reduces duplication)
#[cfg(feature = "spider")]
pub mod spider;
pub mod stealth;
#[cfg(feature = "extraction")]
pub mod strategies;
pub mod streaming; // NDJSON streaming endpoints for real-time data delivery
pub mod stubs; // HTTP 501 stubs for disabled features
#[cfg(feature = "extraction")]
pub mod tables;
pub mod telemetry;
pub mod trace_backend;
pub mod utils;
#[cfg(feature = "workers")]
pub mod workers; // Phase 10.4: Domain profile management API

// Re-export main handlers for backward compatibility
#[cfg(feature = "spider")]
pub use crawl::crawl;
#[cfg(feature = "search")]
pub use deepsearch::handle_deep_search;
#[cfg(feature = "extraction")]
pub use extract::extract;
pub use health::{health, health_capabilities, health_detailed, init_startup_time, START_TIME};
pub use pipeline_phases::get_pipeline_phases;
#[cfg(feature = "browser")]
pub use render::render;
#[cfg(feature = "search")]
pub use search::search;
#[cfg(feature = "spider")]
pub use streaming::crawl_stream;
// Note: deepsearch_stream not yet implemented in Phase 4.3
// #[cfg(feature = "search")]
// pub use streaming::deepsearch_stream;
pub use utils::{metrics, not_found};
