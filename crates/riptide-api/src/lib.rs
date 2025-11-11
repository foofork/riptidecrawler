pub mod adapters; // Sprint 4.3: Transport adapters for streaming
pub mod composition; // Sprint 1.3: Dependency Injection composition root
pub mod config;
pub mod context; // ApplicationContext type alias - clean replacement for AppState god object
pub mod dto;
pub mod errors;
pub mod facades; // Sprint 3.2: Handler business logic facades
pub mod handlers;
pub mod health;
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
pub mod jemalloc_stats;
pub mod metrics;
pub mod metrics_integration; // Sprint 4.5: Merged metrics for /metrics endpoint
pub mod metrics_transport; // Sprint 4.5: Transport-level metrics (HTTP, WebSocket, SSE)
pub mod middleware;
pub mod models;
pub mod pipeline;
pub mod strategies_pipeline;
// pub mod pipeline_dual;  // Temporarily disabled - requires ai_processor module from riptide-intelligence
pub mod pipeline_enhanced;
pub mod reliability_integration;
pub mod resource_manager;
pub mod routes;
pub mod rpc_client;
pub mod rpc_session_context;
pub mod sessions;
pub mod state; // DEPRECATED: Use context::ApplicationContext instead
pub mod streaming;
pub mod telemetry_config;
pub mod tests;
pub mod utils;
pub mod validation;

// Re-export DI composition root types for convenience
pub use composition::{ApplicationContext as DiContext, ApplicationContextBuilder, DiConfig};

// Re-export handler context (ApplicationContext is the primary export)
// Note: This is different from composition::ApplicationContext (DI context)
// This is context::ApplicationContext (type alias for AppState - the HTTP handler state)
pub use context::ApplicationContext;

// Also re-export AppState for backward compatibility
#[allow(deprecated)]
pub use state::AppState;
