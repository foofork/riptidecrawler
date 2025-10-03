// Module declarations
pub mod chunking;
pub mod crawl;
pub mod deepsearch;
pub mod health;
pub mod llm;
pub mod monitoring;
pub mod pdf;
pub mod pipeline_phases;
pub mod render;
pub mod sessions;
pub mod spider;
pub mod stealth;
pub mod strategies;
pub mod tables;
pub mod telemetry;
pub mod utils;
pub mod workers;

// Re-export main handlers for backward compatibility
pub use crawl::crawl;
pub use deepsearch::deepsearch;
pub use health::{health, init_startup_time, START_TIME};
pub use pipeline_phases::get_pipeline_phases;
pub use render::render;
pub use telemetry::{get_telemetry_status, get_trace_tree, list_traces};
pub use utils::{metrics, not_found};
