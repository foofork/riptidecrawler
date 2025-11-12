# EngineSelection Port Trait Design

**Port Trait**: `EngineSelection`
**Facade**: EngineFacade
**Action**: Create abstraction
**Location**: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/engine.rs`
**Risk Level**: Low
**Estimated Time**: 3-4 hours

---

## Rationale

The `EngineFacade` provides intelligent engine selection (browser vs. scraper) based on content analysis. This is unique business logic that needs a proper port trait abstraction.

---

## Current EngineFacade Interface

**File**: `/workspaces/riptidecrawler/crates/riptide-facade/src/facades/engine.rs`

```rust
pub struct EngineFacade {
    cache: Arc<dyn CacheStorage>,
    stats: Arc<tokio::sync::Mutex<EngineStats>>,
    probe_first_enabled: Arc<tokio::sync::RwLock<bool>>,
}

impl EngineFacade {
    pub async fn select_engine(&self, criteria: EngineSelectionCriteria) -> RiptideResult<EngineConfig>;
    pub async fn get_capabilities(&self) -> Vec<EngineCapability>;
    pub async fn enable_probe_first(&self, enabled: bool) -> RiptideResult<()>;
    pub async fn get_stats(&self) -> RiptideResult<EngineStats>;
}
```

**Key Operations**:
1. Intelligent engine selection based on content analysis
2. Engine capability querying
3. Probe-first mode configuration
4. Statistics tracking

---

## Port Trait Design

### File: `/workspaces/riptidecrawler/crates/riptide-types/src/ports/engine.rs`

```rust
//! Engine Selection Port - Hexagonal abstraction for intelligent engine selection
//!
//! This port trait provides a backend-agnostic interface for selecting the best
//! engine (browser vs. scraper) based on content analysis and user preferences.
//!
//! # Architecture
//!
//! ```text
//! Domain Layer (riptide-types)
//!     ↓ defines EngineSelection trait
//! Application Layer (riptide-facade)
//!     ↓ adapts EngineFacade with business logic
//! Composition Root (riptide-api)
//!     ↓ wires Arc<dyn EngineSelection>
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{EngineSelection, EngineSelectionRequest};
//!
//! async fn choose_engine(selector: &dyn EngineSelection, html: &str, url: &str) -> Engine {
//!     let request = EngineSelectionRequest::new(html, url);
//!     let choice = selector.select_engine(request).await?;
//!     println!("Selected: {:?} (confidence: {}%)", choice.engine, choice.confidence * 100.0);
//!     choice.engine
//! }
//! ```

use async_trait::async_trait;
use crate::error::Result as RiptideResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Engine selection port trait
///
/// Defines the interface for intelligent engine selection.
/// Implementations analyze content and provide engine recommendations.
#[async_trait]
pub trait EngineSelection: Send + Sync {
    /// Select the best engine for given content
    ///
    /// Analyzes HTML content and URL to determine whether browser rendering
    /// or simple scraping is more appropriate.
    ///
    /// # Arguments
    ///
    /// * `request` - Engine selection request with content and options
    ///
    /// # Returns
    ///
    /// * `Ok(EngineChoice)` - Selected engine with confidence score and reasoning
    /// * `Err(_)` - Analysis error or invalid request
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = EngineSelectionRequest::new(html, url)
    ///     .with_probe_first(true)
    ///     .with_quality_threshold(0.7);
    ///
    /// let choice = selector.select_engine(request).await?;
    /// if choice.confidence > 0.8 {
    ///     println!("High confidence: use {:?}", choice.engine);
    /// }
    /// ```
    async fn select_engine(&self, request: EngineSelectionRequest) -> RiptideResult<EngineChoice>;

    /// Get available engines and their capabilities
    ///
    /// # Returns
    ///
    /// List of available engines with their features and characteristics
    async fn available_engines(&self) -> Vec<EngineInfo>;

    /// Validate engine compatibility with content
    ///
    /// Checks if a specific engine is compatible with the given URL/content.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to check
    /// * `engine` - The engine type to validate
    ///
    /// # Returns
    ///
    /// `true` if the engine can handle the URL/content
    async fn validate_compatibility(&self, url: &str, engine: EngineType) -> bool;

    /// Get selection statistics
    ///
    /// # Returns
    ///
    /// Usage statistics for engine selections
    async fn selection_stats(&self) -> EngineSelectionStats;

    /// Configure selection behavior
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration options for engine selection
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration applied successfully
    /// * `Err(_)` - Invalid configuration
    async fn configure(&self, config: EngineSelectionConfig) -> RiptideResult<()>;
}

/// Engine selection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSelectionRequest {
    /// HTML content to analyze
    pub html: String,

    /// Source URL for context
    pub url: String,

    /// Feature flags for selection
    pub flags: EngineSelectionFlags,

    /// Minimum acceptable confidence (0.0-1.0)
    #[serde(default = "default_confidence")]
    pub min_confidence: f64,

    /// Cached analysis results (optional)
    #[serde(skip)]
    pub cached_analysis: Option<ContentAnalysis>,
}

impl EngineSelectionRequest {
    /// Create new engine selection request
    pub fn new(html: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            html: html.into(),
            url: url.into(),
            flags: EngineSelectionFlags::default(),
            min_confidence: 0.5,
            cached_analysis: None,
        }
    }

    /// Enable probe-first mode
    pub fn with_probe_first(mut self, enabled: bool) -> Self {
        self.flags.probe_first_spa = enabled;
        self
    }

    /// Set minimum confidence threshold
    pub fn with_quality_threshold(mut self, threshold: f64) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Enable visible text density analysis
    pub fn with_text_density_check(mut self, enabled: bool) -> Self {
        self.flags.use_visible_text_density = enabled;
        self
    }

    /// Enable placeholder detection
    pub fn with_placeholder_detection(mut self, enabled: bool) -> Self {
        self.flags.detect_placeholders = enabled;
        self
    }
}

/// Engine selection flags/options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EngineSelectionFlags {
    /// Use visible text density in analysis
    #[serde(default)]
    pub use_visible_text_density: bool,

    /// Detect placeholder content
    #[serde(default)]
    pub detect_placeholders: bool,

    /// Probe SPA behavior first
    #[serde(default)]
    pub probe_first_spa: bool,
}

impl Default for EngineSelectionFlags {
    fn default() -> Self {
        Self {
            use_visible_text_density: true,
            detect_placeholders: true,
            probe_first_spa: false,
        }
    }
}

/// Engine selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineChoice {
    /// Selected engine type
    pub engine: EngineType,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Human-readable reasons for selection
    pub reasons: Vec<String>,

    /// Content analysis that informed the decision
    pub analysis: ContentAnalysis,

    /// Applied feature flags
    pub flags: EngineSelectionFlags,
}

/// Engine type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineType {
    /// Browser-based rendering (for SPAs, dynamic content)
    Browser,

    /// HTTP scraper (for static content)
    Scraper,

    /// Hybrid approach (try scraper first, fallback to browser)
    Hybrid,
}

/// Engine information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    /// Engine type
    pub engine_type: EngineType,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: String,

    /// Supported features
    pub features: Vec<String>,

    /// Performance characteristics
    pub performance: PerformanceProfile,

    /// Cost characteristics (relative)
    pub cost_profile: CostProfile,
}

/// Performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Typical latency in milliseconds
    pub typical_latency_ms: u64,

    /// CPU usage (low, medium, high)
    pub cpu_usage: ResourceUsage,

    /// Memory usage (low, medium, high)
    pub memory_usage: ResourceUsage,
}

/// Cost profile (relative)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProfile {
    /// Computational cost (low, medium, high)
    pub computational: ResourceUsage,

    /// Network cost (low, medium, high)
    pub network: ResourceUsage,
}

/// Resource usage level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceUsage {
    Low,
    Medium,
    High,
}

/// Content analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Visible text density (0.0-1.0)
    pub text_density: f64,

    /// Contains JavaScript indicators
    pub has_javascript: bool,

    /// Contains SPA framework indicators
    pub has_spa_framework: bool,

    /// Contains dynamic content indicators
    pub has_dynamic_content: bool,

    /// Contains placeholder content
    pub has_placeholders: bool,

    /// HTML size in bytes
    pub html_size: usize,

    /// Script tag count
    pub script_count: usize,

    /// Interactive element count
    pub interactive_elements: usize,
}

/// Engine selection statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineSelectionStats {
    /// Total selections made
    pub total_selections: u64,

    /// Per-engine selection counts
    pub engine_counts: HashMap<EngineType, u64>,

    /// Per-engine selection percentages
    pub engine_percentages: HashMap<EngineType, f64>,

    /// Average confidence scores
    pub avg_confidence: HashMap<EngineType, f64>,
}

/// Configuration for engine selection behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSelectionConfig {
    /// Enable/disable probe-first mode globally
    pub probe_first_enabled: bool,

    /// Default confidence threshold
    pub default_confidence_threshold: f64,

    /// Cache analysis results
    pub enable_caching: bool,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for EngineSelectionConfig {
    fn default() -> Self {
        Self {
            probe_first_enabled: false,
            default_confidence_threshold: 0.5,
            enable_caching: true,
            cache_ttl_seconds: 3600,
        }
    }
}

// Default value functions for serde
fn default_confidence() -> f64 {
    0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_selection_request_builder() {
        let request = EngineSelectionRequest::new("<html></html>", "https://example.com")
            .with_probe_first(true)
            .with_quality_threshold(0.8);

        assert_eq!(request.url, "https://example.com");
        assert!(request.flags.probe_first_spa);
        assert_eq!(request.min_confidence, 0.8);
    }

    #[test]
    fn test_engine_type_equality() {
        assert_eq!(EngineType::Browser, EngineType::Browser);
        assert_ne!(EngineType::Browser, EngineType::Scraper);
    }

    #[test]
    fn test_selection_stats_default() {
        let stats = EngineSelectionStats::default();
        assert_eq!(stats.total_selections, 0);
        assert!(stats.engine_counts.is_empty());
    }

    #[test]
    fn test_resource_usage() {
        let usage = ResourceUsage::Low;
        assert_eq!(usage, ResourceUsage::Low);
    }
}
```

---

## Integration with ApplicationContext

### Update context.rs

**File**: `/workspaces/riptidecrawler/crates/riptide-api/src/context.rs`

```diff
pub struct ApplicationContext {
    // ... other fields ...

-   /// Engine facade for intelligent engine selection
-   pub engine_facade: Arc<riptide_facade::facades::EngineFacade>,
+   /// Engine selector for intelligent engine selection
+   pub engine_selector: Arc<dyn EngineSelection>,

    // ... other fields ...
}
```

---

## Benefits of Port Trait

1. ✅ **Testability** - Easy to create mock implementations
2. ✅ **Swappability** - Can swap selection strategies
3. ✅ **Hexagonal compliance** - Proper domain abstraction
4. ✅ **Type safety** - Strongly typed requests and results
5. ✅ **Observability** - Clear statistics and analysis data

---

## Mock Implementation for Testing

```rust
// File: crates/riptide-types/src/ports/engine/mock.rs

#[cfg(test)]
pub struct MockEngineSelector {
    default_engine: EngineType,
}

#[cfg(test)]
impl MockEngineSelector {
    pub fn new(default_engine: EngineType) -> Self {
        Self { default_engine }
    }

    pub fn always_browser() -> Self {
        Self::new(EngineType::Browser)
    }

    pub fn always_scraper() -> Self {
        Self::new(EngineType::Scraper)
    }
}

#[cfg(test)]
#[async_trait]
impl EngineSelection for MockEngineSelector {
    async fn select_engine(&self, _request: EngineSelectionRequest) -> RiptideResult<EngineChoice> {
        Ok(EngineChoice {
            engine: self.default_engine,
            confidence: 1.0,
            reasons: vec!["Mock always selects same engine".to_string()],
            analysis: ContentAnalysis {
                text_density: 0.5,
                has_javascript: false,
                has_spa_framework: false,
                has_dynamic_content: false,
                has_placeholders: false,
                html_size: 1000,
                script_count: 0,
                interactive_elements: 0,
            },
            flags: EngineSelectionFlags::default(),
        })
    }

    async fn available_engines(&self) -> Vec<EngineInfo> {
        vec![
            EngineInfo {
                engine_type: EngineType::Browser,
                name: "Browser".to_string(),
                description: "Browser-based rendering".to_string(),
                features: vec!["JavaScript".to_string(), "SPA".to_string()],
                performance: PerformanceProfile {
                    typical_latency_ms: 2000,
                    cpu_usage: ResourceUsage::High,
                    memory_usage: ResourceUsage::High,
                },
                cost_profile: CostProfile {
                    computational: ResourceUsage::High,
                    network: ResourceUsage::Medium,
                },
            },
            EngineInfo {
                engine_type: EngineType::Scraper,
                name: "Scraper".to_string(),
                description: "HTTP-based scraping".to_string(),
                features: vec!["Fast".to_string(), "Lightweight".to_string()],
                performance: PerformanceProfile {
                    typical_latency_ms: 200,
                    cpu_usage: ResourceUsage::Low,
                    memory_usage: ResourceUsage::Low,
                },
                cost_profile: CostProfile {
                    computational: ResourceUsage::Low,
                    network: ResourceUsage::Low,
                },
            },
        ]
    }

    async fn validate_compatibility(&self, _url: &str, _engine: EngineType) -> bool {
        true
    }

    async fn selection_stats(&self) -> EngineSelectionStats {
        EngineSelectionStats::default()
    }

    async fn configure(&self, _config: EngineSelectionConfig) -> RiptideResult<()> {
        Ok(())
    }
}
```

---

## Next Steps

1. Create the port trait file
2. Implement adapter (see `08-engine-facade-adapter.md`)
3. Update ApplicationContext
4. Update all call sites
5. Run tests

---

**Status**: ✅ Design Complete - Ready for Implementation
**Dependencies**: None
**Blockers**: None
