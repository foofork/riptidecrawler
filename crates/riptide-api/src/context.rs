/// ApplicationContext - The central application context replacing AppState god object
///
/// This module provides the core application context that handlers use to access
/// infrastructure components. It's a clean abstraction that follows hexagonal
/// architecture principles.
///
/// # Migration Note
/// This is currently a type alias to AppState to enable gradual migration.
/// Once all references are updated, AppState will be eliminated entirely.
///
/// # Note on Naming
/// This is separate from `composition::ApplicationContext` which is used for DI.
/// This type alias is specifically for HTTP handler contexts (State<ApplicationContext>).
// Re-export AppState and related types so handlers can find them through context module
pub use crate::state::{
    AppConfig, AppState, CircuitBreakerConfig, DependencyHealth, EnhancedPipelineConfig,
    EngineSelectionConfig, HealthStatus, MonitoringSystem, PerformanceReport,
};

/// Application context containing all shared infrastructure components
///
/// This is the main entry point for handlers to access:
/// - HTTP clients
/// - Cache managers
/// - Extractors
/// - Resource managers
/// - Metrics collectors
/// - Health checkers
/// - And all other infrastructure
///
/// # Architecture
/// ApplicationContext follows hexagonal architecture:
/// - Handlers depend on ApplicationContext (port)
/// - Infrastructure provides ApplicationContext (adapter)
/// - No business logic in ApplicationContext itself
///
/// # Usage
/// ```rust
/// use axum::extract::State;
/// use riptide_api::context::ApplicationContext;
///
/// async fn my_handler(State(ctx): State<ApplicationContext>) -> Result<Json<Response>> {
///     // Access infrastructure through context
///     let result = ctx.resource_manager.acquire_slot().await?;
///     Ok(Json(result))
/// }
/// ```
pub type ApplicationContext = AppState;
