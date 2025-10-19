/// Simplified reliability module for Phase 2 - Day 1-2
///
/// This module provides basic reliability patterns. Full implementation will be
/// added in later phases when circular dependencies are resolved.
use anyhow::Result;
use riptide_types::ExtractedDoc;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Reliability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// Enable graceful degradation fallback
    pub enable_graceful_degradation: bool,
    /// Headless service timeout (hard cap)
    pub headless_timeout: Duration,
    /// Quality threshold for accepting fast extraction results
    pub fast_extraction_quality_threshold: f32,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            enable_graceful_degradation: true,
            headless_timeout: Duration::from_secs(3), // 3s hard cap
            fast_extraction_quality_threshold: 0.6,
        }
    }
}

/// Extraction mode selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionMode {
    Fast,
    Headless,
    ProbesFirst,
}

/// Reliability metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub http_circuit_breaker_state: String,
    pub http_failure_count: u64,
    pub headless_circuit_breaker_state: String,
    pub headless_failure_count: u64,
    pub graceful_degradation_enabled: bool,
}

/// WASM extractor trait for dependency injection
pub trait WasmExtractor: Send + Sync {
    fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc>;
}

/// Enhanced extraction orchestrator with reliability patterns
#[derive(Debug)]
pub struct ReliableExtractor {
    config: ReliabilityConfig,
}

impl ReliableExtractor {
    pub fn new(config: ReliabilityConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Get reliability metrics for monitoring
    pub async fn get_reliability_metrics(&self) -> ReliabilityMetrics {
        ReliabilityMetrics {
            http_circuit_breaker_state: "Closed".to_string(),
            http_failure_count: 0,
            headless_circuit_breaker_state: "Closed".to_string(),
            headless_failure_count: 0,
            graceful_degradation_enabled: self.config.enable_graceful_degradation,
        }
    }
}

/// Minimal PerformanceMetrics for non-events build
#[cfg(not(feature = "events"))]
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub avg_processing_time_ms: f64,
    pub circuit_breaker_trips: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_config_default() {
        let config = ReliabilityConfig::default();
        assert!(config.enable_graceful_degradation);
        assert_eq!(config.headless_timeout, Duration::from_secs(3));
        assert_eq!(config.fast_extraction_quality_threshold, 0.6);
    }

    #[tokio::test]
    async fn test_reliable_extractor_creation() {
        let config = ReliabilityConfig::default();
        let extractor = ReliableExtractor::new(config);
        assert!(extractor.is_ok());
    }

    #[tokio::test]
    async fn test_reliability_metrics() {
        let extractor = ReliableExtractor::new(ReliabilityConfig::default()).unwrap();
        let metrics = extractor.get_reliability_metrics().await;

        assert_eq!(metrics.http_circuit_breaker_state, "Closed");
        assert_eq!(metrics.http_failure_count, 0);
        assert!(metrics.graceful_degradation_enabled);
    }
}
