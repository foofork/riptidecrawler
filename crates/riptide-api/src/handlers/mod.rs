// Module declarations
#[cfg(feature = "persistence")]
pub mod admin;
pub mod browser;
pub mod chunking;
pub mod crawl;
pub mod deepsearch;
pub mod engine_selection;
pub mod extract;
pub mod fetch;
pub mod health;
pub mod llm;
pub mod monitoring;
pub mod pdf;
pub mod pipeline_metrics; // Enhanced pipeline metrics visualization
pub mod pipeline_phases;
pub mod profiles;
pub mod profiling;
pub mod render;
pub mod resources;
pub mod search;
pub mod sessions;
pub mod shared; // Shared utilities for handlers (reduces duplication)
pub mod spider;
pub mod stealth;
pub mod strategies;
pub mod streaming; // NDJSON streaming endpoints for real-time data delivery
pub mod tables;
pub mod telemetry;
pub mod trace_backend;
pub mod utils;
pub mod workers; // Phase 10.4: Domain profile management API

// Re-export main handlers for backward compatibility
pub use crawl::crawl;
pub use deepsearch::deepsearch;
pub use extract::extract;
pub use health::{health, health_detailed, init_startup_time, START_TIME};
pub use pipeline_phases::get_pipeline_phases;
pub use render::render;
pub use search::search;
pub use streaming::{crawl_stream, deepsearch_stream};
pub use utils::{metrics, not_found};
