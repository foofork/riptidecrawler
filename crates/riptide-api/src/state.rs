//! AppState - DEPRECATED
//!
//! This module is deprecated. Use `crate::context::ApplicationContext` instead.
//!
//! This type alias exists for backward compatibility during migration.

/// DEPRECATED: Use context::ApplicationContext instead
///
/// This is a type alias for backward compatibility. All new code should use
/// `ApplicationContext` directly from the `context` module.
#[deprecated(
    since = "0.9.0",
    note = "Use context::ApplicationContext instead. See docs/architecture/ADR-001-appstate-elimination.md"
)]
pub type AppState = crate::context::ApplicationContext;

// Re-export all types from context for backward compatibility
#[deprecated(since = "0.9.0", note = "Use context::ApplicationContext instead")]
pub use crate::context::ApplicationContext;

#[deprecated(since = "0.9.0", note = "Use context::AppConfig instead")]
pub use crate::context::AppConfig;

#[deprecated(since = "0.9.0", note = "Use context::EnhancedPipelineConfig instead")]
pub use crate::context::EnhancedPipelineConfig;

#[deprecated(since = "0.9.0", note = "Use context::EngineSelectionConfig instead")]
pub use crate::context::EngineSelectionConfig;

#[deprecated(since = "0.9.0", note = "Use context::CircuitBreakerConfig instead")]
pub use crate::context::CircuitBreakerConfig;

#[deprecated(since = "0.9.0", note = "Use context::HealthStatus instead")]
pub use crate::context::HealthStatus;

#[deprecated(since = "0.9.0", note = "Use context::DependencyHealth instead")]
pub use crate::context::DependencyHealth;

#[deprecated(since = "0.9.0", note = "Use context::MonitoringSystem instead")]
pub use crate::context::MonitoringSystem;

#[deprecated(since = "0.9.0", note = "Use context::PerformanceReport instead")]
pub use crate::context::PerformanceReport;
