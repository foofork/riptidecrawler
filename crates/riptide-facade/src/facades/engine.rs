//! Engine Selection Facade - Business logic for intelligent engine selection
//!
//! This facade encapsulates all business logic for engine selection, moving it out
//! of API handlers. It provides:
//! - Intelligent engine selection based on content analysis
//! - Engine capability querying
//! - Engine configuration management
//! - Caching of analysis results
//! - Metrics tracking
//! - Event emission for observability
//!
//! Phase 3 Sprint 3.1: Created to support ultra-thin handlers (<30 LOC)

use crate::error::{RiptideError, RiptideResult};
use riptide_reliability::engine_selection::{
    analyze_content, decide_engine_with_flags, ContentAnalysis, Engine, EngineSelectionFlags,
};
use riptide_types::ports::CacheStorage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Engine selection criteria for intelligent selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSelectionCriteria {
    /// HTML content to analyze
    pub html: String,
    /// Source URL for context
    pub url: String,
    /// Feature flags for selection
    pub flags: EngineSelectionFlags,
}

/// Engine configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Selected engine
    pub engine: Engine,
    /// Confidence score (0-100)
    pub confidence: f64,
    /// Selection reasons
    pub reasons: Vec<String>,
    /// Applied feature flags
    pub flags: EngineSelectionFlags,
}

/// Engine capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineCapability {
    /// Engine type
    pub engine: Engine,
    /// Engine name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Supported features
    pub features: Vec<String>,
    /// Performance characteristics
    pub performance: String,
    /// Cost characteristics
    pub cost: String,
}

/// Engine usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineStats {
    /// Total requests analyzed
    pub total_requests: u64,
    /// Per-engine counts
    pub engine_counts: HashMap<String, u64>,
    /// Per-engine percentages
    pub engine_percentages: HashMap<String, f64>,
    /// Probe-first mode enabled
    pub probe_first_enabled: bool,
}

/// Engine Selection Facade providing business logic for engine selection
pub struct EngineFacade {
    /// Cache for analysis results
    cache: Arc<dyn CacheStorage>,
    /// Engine usage statistics
    stats: Arc<tokio::sync::Mutex<EngineStats>>,
    /// Probe-first configuration
    probe_first_enabled: Arc<tokio::sync::RwLock<bool>>,
}

impl EngineFacade {
    /// Create new engine facade with caching
    pub fn new(cache: Arc<dyn CacheStorage>) -> Self {
        Self {
            cache,
            stats: Arc::new(tokio::sync::Mutex::new(EngineStats::default())),
            probe_first_enabled: Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    /// Select engine based on content analysis with caching
    ///
    /// This method performs:
    /// 1. Cache lookup for recent analysis
    /// 2. Content analysis if not cached
    /// 3. Confidence calculation
    /// 4. Reason generation
    /// 5. Cache storage
    /// 6. Statistics update
    #[tracing::instrument(skip(self, criteria))]
    pub async fn select_engine(
        &self,
        criteria: EngineSelectionCriteria,
    ) -> RiptideResult<EngineConfig> {
        // Generate cache key
        let cache_key = format!(
            "engine_select:{}:{}:{}:{}",
            Self::hash_content(&criteria.html),
            criteria.flags.use_visible_text_density,
            criteria.flags.detect_placeholders,
            criteria.flags.probe_first_spa
        );

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key).await? {
            debug!("Cache hit for engine selection");
            return serde_json::from_slice(&cached).map_err(|e| {
                RiptideError::Cache(format!("Failed to deserialize cached result: {}", e))
            });
        }

        info!(url = %criteria.url, "Analyzing content for engine selection");

        // Perform content analysis
        let analysis = analyze_content(&criteria.html, &criteria.url);

        // Make engine decision with flags
        let probe_first = *self.probe_first_enabled.read().await;
        let mut flags = criteria.flags;
        flags.probe_first_spa = probe_first || flags.probe_first_spa;

        let engine = decide_engine_with_flags(&criteria.html, &criteria.url, flags, ());

        // Calculate confidence based on analysis
        let confidence = Self::calculate_confidence(&analysis);

        // Generate human-readable reasons
        let reasons = Self::generate_reasons(&analysis, engine);

        let result = EngineConfig {
            engine,
            confidence,
            reasons,
            flags,
        };

        // Cache result (1 hour TTL)
        let serialized = serde_json::to_vec(&result)
            .map_err(|e| RiptideError::Config(format!("Failed to serialize result: {}", e)))?;
        self.cache
            .set(&cache_key, &serialized, Some(Duration::from_secs(3600)))
            .await?;

        // Update statistics
        self.update_stats(engine, confidence).await;

        info!(engine = %engine.name(), confidence = confidence, "Engine selection complete");

        Ok(result)
    }

    /// List all available engines with capabilities
    pub async fn list_engines(&self) -> RiptideResult<Vec<EngineCapability>> {
        Ok(vec![
            EngineCapability {
                engine: Engine::Raw,
                name: "Raw HTTP".to_string(),
                description: "Simple HTTP fetch without JavaScript execution".to_string(),
                features: vec![
                    "No JavaScript execution".to_string(),
                    "Fast response time".to_string(),
                    "Low resource usage".to_string(),
                ],
                performance: "Fastest (< 100ms)".to_string(),
                cost: "Lowest".to_string(),
            },
            EngineCapability {
                engine: Engine::Wasm,
                name: "WASM Extraction".to_string(),
                description: "Local WASM-based extraction with DOM parsing".to_string(),
                features: vec![
                    "DOM parsing".to_string(),
                    "Content extraction".to_string(),
                    "Fast local execution".to_string(),
                ],
                performance: "Fast (< 500ms)".to_string(),
                cost: "Low".to_string(),
            },
            EngineCapability {
                engine: Engine::Headless,
                name: "Headless Browser".to_string(),
                description: "Full headless browser with JavaScript execution".to_string(),
                features: vec![
                    "Full JavaScript execution".to_string(),
                    "SPA support".to_string(),
                    "Anti-scraping bypass".to_string(),
                    "Screenshot capability".to_string(),
                ],
                performance: "Slower (1-5s)".to_string(),
                cost: "High".to_string(),
            },
        ])
    }

    /// Configure engine selection with flags
    pub async fn configure_engine(
        &self,
        probe_first_spa: Option<bool>,
    ) -> RiptideResult<HashMap<String, bool>> {
        let mut config = HashMap::new();

        if let Some(enabled) = probe_first_spa {
            *self.probe_first_enabled.write().await = enabled;
            info!(
                probe_first_enabled = enabled,
                "Probe-first mode configuration updated"
            );
        }

        let current_probe_first = *self.probe_first_enabled.read().await;
        config.insert("probe_first_spa".to_string(), current_probe_first);

        Ok(config)
    }

    /// Get engine capabilities for a specific engine
    pub async fn get_engine_capabilities(&self, engine: Engine) -> RiptideResult<EngineCapability> {
        let engines = self.list_engines().await?;
        engines
            .into_iter()
            .find(|e| e.engine == engine)
            .ok_or_else(|| RiptideError::NotFound(format!("Engine {:?} not found", engine)))
    }

    /// Get engine usage statistics
    pub async fn get_stats(&self) -> RiptideResult<EngineStats> {
        let mut stats = self.stats.lock().await.clone();
        stats.probe_first_enabled = *self.probe_first_enabled.read().await;

        // Calculate percentages
        if stats.total_requests > 0 {
            for (engine, count) in &stats.engine_counts {
                let percentage = (*count as f64 / stats.total_requests as f64) * 100.0;
                stats.engine_percentages.insert(engine.clone(), percentage);
            }
        }

        Ok(stats)
    }

    /// Calculate confidence score based on content analysis
    fn calculate_confidence(analysis: &ContentAnalysis) -> f64 {
        let mut confidence = 50.0; // Base confidence

        // High confidence indicators
        if analysis.has_anti_scraping {
            confidence += 40.0; // Very strong signal for headless
        }

        if analysis.has_react || analysis.has_vue || analysis.has_angular {
            confidence += 30.0; // Strong framework signals
        }

        if analysis.has_spa_markers {
            confidence += 20.0; // SPA indicators
        }

        // Content quality indicators
        if analysis.content_ratio > 0.3 {
            confidence += 15.0; // Good content ratio
        } else if analysis.content_ratio < 0.1 {
            confidence += 25.0; // Very low ratio is strong signal
        }

        // Visible text density refinement
        if analysis.visible_text_density > 0.2 {
            confidence += 10.0; // Good visible content
        }

        // Placeholder detection
        if analysis.has_placeholders {
            confidence += 15.0; // Loading indicators suggest JS needed
        }

        // Main content structure
        if analysis.has_main_content && analysis.content_ratio > 0.2 {
            confidence += 10.0; // Well-structured content
        }

        // Cap at 100
        if confidence > 100.0 {
            100.0
        } else {
            confidence
        }
    }

    /// Generate human-readable reasons for engine selection
    fn generate_reasons(analysis: &ContentAnalysis, engine: Engine) -> Vec<String> {
        let mut reasons = Vec::new();

        // Framework detection reasons
        if analysis.has_react {
            reasons.push("React framework detected (Next.js markers, webpack)".to_string());
        }
        if analysis.has_vue {
            reasons.push("Vue.js framework detected".to_string());
        }
        if analysis.has_angular {
            reasons.push("Angular framework detected".to_string());
        }

        // SPA and anti-scraping reasons
        if analysis.has_spa_markers {
            reasons.push("Single Page Application (SPA) markers found".to_string());
        }
        if analysis.has_anti_scraping {
            reasons.push("Anti-scraping protection detected (Cloudflare, reCAPTCHA)".to_string());
        }

        // Content analysis reasons
        if analysis.content_ratio < 0.1 {
            reasons.push(format!(
                "Low content-to-markup ratio ({:.1}%) suggests client-side rendering",
                analysis.content_ratio * 100.0
            ));
        } else if analysis.content_ratio > 0.3 {
            reasons.push(format!(
                "Good content-to-markup ratio ({:.1}%) indicates server-rendered content",
                analysis.content_ratio * 100.0
            ));
        }

        // Visible text density
        if analysis.visible_text_density < 0.15 {
            reasons.push("Low visible text density (excluding scripts/styles)".to_string());
        }

        // Placeholder detection
        if analysis.has_placeholders {
            reasons
                .push("Skeleton/placeholder UI detected (shimmer, loading indicators)".to_string());
        }

        // Main content structure
        if analysis.has_main_content {
            reasons.push("Well-structured content with article/main tags".to_string());
        }

        // Engine-specific reasoning
        match engine {
            Engine::Headless => {
                if reasons.is_empty() {
                    reasons.push("JavaScript execution required for content rendering".to_string());
                }
            }
            Engine::Wasm => {
                if reasons.is_empty() {
                    reasons.push("Standard HTML extraction with WASM is sufficient".to_string());
                }
            }
            Engine::Raw => {
                reasons.push("Simple HTTP fetch without JavaScript execution".to_string());
            }
            Engine::Auto => {
                reasons.push("Automatic engine selection needed".to_string());
            }
        }

        reasons
    }

    /// Update internal statistics
    async fn update_stats(&self, engine: Engine, confidence: f64) {
        let mut stats = self.stats.lock().await;
        stats.total_requests += 1;

        let engine_name = engine.name().to_string();
        *stats.engine_counts.entry(engine_name).or_insert(0) += 1;

        debug!(
            engine = %engine.name(),
            confidence = confidence,
            total_requests = stats.total_requests,
            "Statistics updated"
        );
    }

    /// Hash HTML content for cache key generation
    fn hash_content(html: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        html.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock cache for testing
    struct MockCache {
        data: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl MockCache {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl CacheStorage for MockCache {
        async fn get(&self, key: &str) -> riptide_types::error::Result<Option<Vec<u8>>> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        async fn set(
            &self,
            key: &str,
            value: &[u8],
            _ttl: Option<Duration>,
        ) -> riptide_types::error::Result<()> {
            self.data
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        async fn delete(&self, key: &str) -> riptide_types::error::Result<()> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> riptide_types::error::Result<bool> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }
    }

    #[tokio::test]
    async fn test_select_engine_headless() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        let criteria = EngineSelectionCriteria {
            html:
                r#"<html><body><div id="root"></div><script src="react.js"></script></body></html>"#
                    .to_string(),
            url: "https://example.com".to_string(),
            flags: EngineSelectionFlags::default(),
        };

        let result = facade.select_engine(criteria).await.unwrap();
        assert_eq!(result.engine, Engine::Headless);
        assert!(result.confidence > 50.0);
        assert!(!result.reasons.is_empty());
    }

    #[tokio::test]
    async fn test_select_engine_wasm() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        let criteria = EngineSelectionCriteria {
            html: r#"<html><body><article><h1>Title</h1><p>Content with good text ratio</p></article></body></html>"#.to_string(),
            url: "https://example.com".to_string(),
            flags: EngineSelectionFlags::default(),
        };

        let result = facade.select_engine(criteria).await.unwrap();
        assert!(matches!(result.engine, Engine::Wasm | Engine::Raw));
    }

    #[tokio::test]
    async fn test_caching() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache.clone());

        let criteria = EngineSelectionCriteria {
            html: "<html><body>Test</body></html>".to_string(),
            url: "https://example.com".to_string(),
            flags: EngineSelectionFlags::default(),
        };

        // First call - should cache
        let result1 = facade.select_engine(criteria.clone()).await.unwrap();

        // Second call - should hit cache
        let result2 = facade.select_engine(criteria).await.unwrap();

        assert_eq!(result1.engine, result2.engine);
        assert_eq!(result1.confidence, result2.confidence);
    }

    #[tokio::test]
    async fn test_list_engines() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        let engines = facade.list_engines().await.unwrap();
        assert_eq!(engines.len(), 3);
        assert!(engines.iter().any(|e| e.engine == Engine::Raw));
        assert!(engines.iter().any(|e| e.engine == Engine::Wasm));
        assert!(engines.iter().any(|e| e.engine == Engine::Headless));
    }

    #[tokio::test]
    async fn test_configure_engine() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        let config = facade.configure_engine(Some(true)).await.unwrap();
        assert_eq!(config.get("probe_first_spa"), Some(&true));

        let config = facade.configure_engine(Some(false)).await.unwrap();
        assert_eq!(config.get("probe_first_spa"), Some(&false));
    }

    #[tokio::test]
    async fn test_get_engine_capabilities() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        let capability = facade.get_engine_capabilities(Engine::Wasm).await.unwrap();
        assert_eq!(capability.engine, Engine::Wasm);
        assert_eq!(capability.name, "WASM Extraction");
        assert!(!capability.features.is_empty());
    }

    #[tokio::test]
    async fn test_get_stats() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        // Initial stats should be empty
        let stats = facade.get_stats().await.unwrap();
        assert_eq!(stats.total_requests, 0);

        // Make a request
        let criteria = EngineSelectionCriteria {
            html: "<html><body>Test</body></html>".to_string(),
            url: "https://example.com".to_string(),
            flags: EngineSelectionFlags::default(),
        };
        facade.select_engine(criteria).await.unwrap();

        // Stats should be updated
        let stats = facade.get_stats().await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert!(!stats.engine_counts.is_empty());
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        let analysis = ContentAnalysis {
            has_react: true,
            has_vue: false,
            has_angular: false,
            has_spa_markers: true,
            has_anti_scraping: true,
            content_ratio: 0.1,
            has_main_content: false,
            visible_text_density: 0.15,
            has_placeholders: true,
            recommended_engine: Engine::Headless,
        };

        let confidence = EngineFacade::calculate_confidence(&analysis);
        assert!(confidence > 80.0); // High confidence for strong signals
    }

    #[tokio::test]
    async fn test_reason_generation() {
        let analysis = ContentAnalysis {
            has_react: true,
            has_vue: false,
            has_angular: false,
            has_spa_markers: true,
            has_anti_scraping: false,
            content_ratio: 0.25,
            has_main_content: true,
            visible_text_density: 0.2,
            has_placeholders: false,
            recommended_engine: Engine::Headless,
        };

        let reasons = EngineFacade::generate_reasons(&analysis, Engine::Headless);
        assert!(reasons.iter().any(|r| r.contains("React")));
        assert!(reasons.iter().any(|r| r.contains("SPA")));
    }

    #[tokio::test]
    async fn test_probe_first_mode() {
        let cache = Arc::new(MockCache::new());
        let facade = EngineFacade::new(cache);

        // Enable probe-first mode
        facade.configure_engine(Some(true)).await.unwrap();

        let criteria = EngineSelectionCriteria {
            html: "<html><body><div id='app'></div></body></html>".to_string(),
            url: "https://example.com".to_string(),
            flags: EngineSelectionFlags::default(),
        };

        let result = facade.select_engine(criteria).await.unwrap();
        assert!(result.flags.probe_first_spa);
    }
}
