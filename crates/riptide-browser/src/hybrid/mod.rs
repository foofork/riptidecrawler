//! Hybrid browser fallback and A/B testing infrastructure

pub mod fallback;
pub use fallback::{BrowserResponse, EngineKind, FallbackMetrics, HybridBrowserFallback};
