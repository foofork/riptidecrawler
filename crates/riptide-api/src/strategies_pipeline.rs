use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use reqwest::Response;
use riptide_core::{
    fetch,
    gate::{decide, score, Decision, GateFeatures},
    pdf::{self, utils as pdf_utils},
    types::{CrawlOptions, RenderMode},
    strategies::{
        StrategyManager, StrategyConfig, ExtractionStrategy, ChunkingConfig,
        ProcessedContent, PerformanceMetrics,
    },
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};
use url::Url;

/// Enhanced pipeline result with strategies integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategiesPipelineResult {
    /// The processed content with extraction and chunking
    pub processed_content: ProcessedContent,

    /// Whether the content was served from cache
    pub from_cache: bool,

    /// The decision made by the gate (Raw, ProbesFirst, Headless)
    pub gate_decision: String,

    /// Content quality score from the gate analysis
    pub quality_score: f32,

    /// Total processing time in milliseconds
    pub processing_time_ms: u64,

    /// Cache key used for this URL
    pub cache_key: String,

    /// HTTP status code from the original fetch
    pub http_status: u16,

    /// Strategy configuration used
    pub strategy_config: StrategyConfig,

    /// Performance metrics if enabled
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Pipeline orchestrator with strategies integration
///
/// This orchestrator enhances the standard pipeline with:
/// 1. Multiple extraction strategies (trek, css_json, regex, llm)
/// 2. Configurable chunking modes (regex, sentence, topic, fixed, sliding)
/// 3. Performance tracking and metrics
/// 4. Strategy selection based on content analysis
pub struct StrategiesPipelineOrchestrator {
    state: AppState,
    options: CrawlOptions,
    strategy_config: StrategyConfig,
}

impl StrategiesPipelineOrchestrator {
    /// Create a new strategies pipeline orchestrator
    pub fn new(
        state: AppState,
        options: CrawlOptions,
        strategy_config: Option<StrategyConfig>,
    ) -> Self {
        Self {
            state,
            options,
            strategy_config: strategy_config.unwrap_or_default(),
        }
    }

    /// Create with auto-detected strategy based on URL and options
    pub fn with_auto_strategy(state: AppState, options: CrawlOptions, url: &str) -> Self {
        let strategy_config = Self::auto_detect_strategy(url, &options);
        Self::new(state, options, Some(strategy_config))
    }

    /// Execute the strategies pipeline for a single URL
    ///
    /// This method orchestrates:
    /// 1. Cache check
    /// 2. Content fetching
    /// 3. Strategy selection and configuration
    /// 4. Content extraction using selected strategy
    /// 5. Content chunking using configured mode
    /// 6. Metadata extraction
    /// 7. Performance metrics collection
    /// 8. Result caching
    pub async fn execute_single(&self, url: &str) -> ApiResult<StrategiesPipelineResult> {
        let start_time = Instant::now();
        let cache_key = self.generate_cache_key(url);

        info!(
            url = %url,
            cache_key = %cache_key,
            extraction_strategy = ?self.strategy_config.extraction,
            chunking_mode = ?self.strategy_config.chunking.mode,
            "Starting strategies pipeline execution"
        );

        // Step 1: Check cache first
        let cached_result = self.check_cache(&cache_key).await;
        if let Ok(Some(cached)) = cached_result {
            info!(url = %url, "Cache hit, returning cached processed content");
            return Ok(StrategiesPipelineResult {
                processed_content: cached,
                from_cache: true,
                gate_decision: "cached".to_string(),
                quality_score: 1.0,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                cache_key,
                http_status: 200,
                strategy_config: self.strategy_config.clone(),
                performance_metrics: None,
            });
        }

        // Step 2: Fetch content
        debug!(url = %url, "Cache miss, fetching content");
        let (response, content_bytes, content_type) = self.fetch_content_with_type(url).await?;
        let http_status = response.status().as_u16();

        // Step 3: Check for PDF content
        if pdf_utils::is_pdf_content(content_type.as_deref(), &content_bytes)
            || matches!(self.options.render_mode, RenderMode::Pdf)
        {
            return self.process_pdf_pipeline(url, &content_bytes, start_time, cache_key, http_status).await;
        }

        // Convert to HTML string
        let html_content = String::from_utf8_lossy(&content_bytes).to_string();

        // Step 4: Gate analysis
        let gate_features = self.analyze_content(&html_content, url).await?;
        let quality_score = score(&gate_features);
        let decision = decide(
            &gate_features,
            self.state.config.gate_hi_threshold,
            self.state.config.gate_lo_threshold,
        );

        let gate_decision_str = match decision {
            Decision::Raw => "raw",
            Decision::ProbesFirst => "probes_first",
            Decision::Headless => "headless",
        }.to_string();

        info!(
            url = %url,
            decision = %gate_decision_str,
            score = %quality_score,
            "Gate analysis complete"
        );

        // Step 5: Create strategy manager and process content
        let mut strategy_manager = StrategyManager::new(self.strategy_config.clone());

        let processed_content = match decision {
            Decision::Headless => {
                // For headless content, first try to render with headless browser
                match self.extract_with_headless(url).await {
                    Ok(rendered_html) => {
                        // Use strategies on rendered content
                        strategy_manager.extract_and_chunk(&rendered_html, url).await
                            .map_err(|e| ApiError::pipeline(format!("Strategy processing failed: {}", e)))?
                    }
                    Err(e) => {
                        warn!(url = %url, error = %e, "Headless rendering failed, using direct strategies");
                        // Fallback to processing original HTML with strategies
                        strategy_manager.extract_and_chunk(&html_content, url).await
                            .map_err(|e| ApiError::pipeline(format!("Strategy processing failed: {}", e)))?
                    }
                }
            }
            _ => {
                // Use strategies directly on HTML content
                strategy_manager.extract_and_chunk(&html_content, url).await
                    .map_err(|e| ApiError::pipeline(format!("Strategy processing failed: {}", e)))?
            }
        };

        // Step 6: Cache the processed result
        if let Err(e) = self.store_in_cache(&cache_key, &processed_content).await {
            warn!(error = %e, "Failed to cache processed result, continuing anyway");
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        let performance_metrics = if self.strategy_config.enable_metrics {
            Some(strategy_manager.get_metrics().clone())
        } else {
            None
        };

        info!(
            url = %url,
            processing_time_ms = %processing_time_ms,
            chunks_created = processed_content.chunks.len(),
            extraction_strategy = ?self.strategy_config.extraction,
            "Strategies pipeline execution complete"
        );

        Ok(StrategiesPipelineResult {
            processed_content,
            from_cache: false,
            gate_decision: gate_decision_str,
            quality_score,
            processing_time_ms,
            cache_key,
            http_status,
            strategy_config: self.strategy_config.clone(),
            performance_metrics,
        })
    }

    /// Process PDF content through strategies pipeline
    async fn process_pdf_pipeline(
        &self,
        url: &str,
        pdf_bytes: &[u8],
        start_time: Instant,
        cache_key: String,
        http_status: u16,
    ) -> ApiResult<StrategiesPipelineResult> {
        info!(
            url = %url,
            file_size = pdf_bytes.len(),
            "Processing PDF through strategies pipeline"
        );

        let processor = pdf::create_pdf_processor();
        let extracted_doc = processor.process_pdf_bytes(pdf_bytes).await
            .map_err(|e| ApiError::pipeline(format!("PDF processing failed: {}", e)))?;

        // Convert PDF extracted doc to strategies format
        let pdf_html = format!(
            "<html><head><title>{}</title></head><body><article>{}</article></body></html>",
            extracted_doc.title.as_deref().unwrap_or("PDF Document"),
            extracted_doc.markdown.replace('\n', "<br>")
        );

        // Process through strategies
        let mut strategy_manager = StrategyManager::new(self.strategy_config.clone());
        let processed_content = strategy_manager.extract_and_chunk(&pdf_html, url).await
            .map_err(|e| ApiError::pipeline(format!("PDF strategy processing failed: {}", e)))?;

        // Cache the result
        if let Err(e) = self.store_in_cache(&cache_key, &processed_content).await {
            warn!(error = %e, "Failed to cache PDF processed result");
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        let performance_metrics = if self.strategy_config.enable_metrics {
            Some(strategy_manager.get_metrics().clone())
        } else {
            None
        };

        Ok(StrategiesPipelineResult {
            processed_content,
            from_cache: false,
            gate_decision: "pdf".to_string(),
            quality_score: 0.95,
            processing_time_ms,
            cache_key,
            http_status,
            strategy_config: self.strategy_config.clone(),
            performance_metrics,
        })
    }

    /// Auto-detect optimal strategy based on URL and options
    fn auto_detect_strategy(url: &str, options: &CrawlOptions) -> StrategyConfig {
        let mut config = StrategyConfig::default();

        // Analyze URL for strategy hints
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                // Website-specific strategy optimization
                config.extraction = match host {
                    host if host.contains("github.com") => {
                        // GitHub: Use CSS selectors for structured content
                        ExtractionStrategy::CssJson {
                            selectors: create_github_selectors(),
                        }
                    }
                    host if host.contains("wikipedia.org") => {
                        // Wikipedia: Trek works well for clean HTML
                        ExtractionStrategy::Trek
                    }
                    host if host.contains("medium.com") || host.contains("dev.to") => {
                        // Blog platforms: CSS selectors for article structure
                        ExtractionStrategy::CssJson {
                            selectors: create_blog_selectors(),
                        }
                    }
                    host if host.contains("reddit.com") || host.contains("news.ycombinator.com") => {
                        // News/discussion sites: Regex for structured data
                        ExtractionStrategy::Regex {
                            patterns: create_news_patterns(),
                        }
                    }
                    _ => {
                        // Default to Trek for unknown sites
                        ExtractionStrategy::Trek
                    }
                }
            }
        }

        // Adjust chunking based on render mode
        config.chunking = match options.render_mode {
            RenderMode::Html => ChunkingConfig {
                mode: riptide_core::strategies::chunking::ChunkingMode::Sliding,
                token_max: 1200,
                overlap: 120,
                preserve_sentences: true,
                deterministic: true,
            },
            RenderMode::Pdf => ChunkingConfig {
                mode: riptide_core::strategies::chunking::ChunkingMode::Fixed { size: 2000, by_tokens: true },
                token_max: 2000,
                overlap: 200,
                preserve_sentences: true,
                deterministic: true,
            },
            RenderMode::Markdown => ChunkingConfig {
                mode: riptide_core::strategies::chunking::ChunkingMode::Topic { similarity_threshold: 0.7 },
                token_max: 1500,
                overlap: 150,
                preserve_sentences: true,
                deterministic: true,
            },
            RenderMode::Static | RenderMode::Dynamic | RenderMode::Adaptive => ChunkingConfig {
                mode: riptide_core::strategies::chunking::ChunkingMode::Sliding,
                token_max: 1200,
                overlap: 120,
                preserve_sentences: true,
                deterministic: true,
            },
        };

        config
    }

    /// Fetch content with type detection
    async fn fetch_content_with_type(
        &self,
        url: &str,
    ) -> ApiResult<(Response, Vec<u8>, Option<String>)> {
        let fetch_timeout = Duration::from_secs(15);
        let response = timeout(fetch_timeout, fetch::get(&self.state.http_client, url))
            .await
            .map_err(|_| ApiError::timeout("content_fetch", format!("Timeout fetching {}", url)))?
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .map(|s| s.to_string());

        let content_bytes = timeout(fetch_timeout, response.bytes())
            .await
            .map_err(|_| ApiError::timeout("content_read", format!("Timeout reading content from {}", url)))?
            .map_err(|e| ApiError::fetch(url, format!("Failed to read response body: {}", e)))?
            .to_vec();

        let response = fetch::get(&self.state.http_client, url)
            .await
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        Ok((response, content_bytes, content_type))
    }

    /// Analyze content for gate features
    async fn analyze_content(&self, html: &str, url: &str) -> ApiResult<GateFeatures> {
        let parsed_url = Url::parse(url)
            .map_err(|e| ApiError::invalid_url(url, format!("URL parsing failed: {}", e)))?;

        let html_bytes = html.len();
        let visible_text_chars = html.chars().filter(|c| !c.is_control()).count();

        let p_count = html.matches("<p").count() as u32;
        let article_count = html.matches("<article").count() as u32 + html.matches("<main").count() as u32;
        let h1h2_count = html.matches("<h1").count() as u32 + html.matches("<h2").count() as u32;

        let script_bytes = html
            .split("<script")
            .skip(1)
            .map(|part| {
                part.find("</script>")
                    .map(|end| end + 9)
                    .unwrap_or(part.len())
            })
            .sum::<usize>();

        let has_og = html.contains("property=\"og:") || html.contains("property='og:");
        let has_jsonld_article = html.contains("\"@type\":\"Article\"") || html.contains("'@type':'Article'");

        let mut spa_markers = 0u8;
        if html.contains("__NEXT_DATA__") { spa_markers += 1; }
        if html.contains("data-reactroot") || html.contains("data-react-helmet") { spa_markers += 1; }
        if html.contains("id=\"root\"") && html.matches("<div").count() > 20 { spa_markers += 1; }
        if script_bytes > html_bytes / 2 { spa_markers += 1; }

        let domain_prior = match parsed_url.host_str() {
            Some(host) => {
                if host.contains("wikipedia.") || host.contains("github.") {
                    0.9
                } else if host.contains("medium.") || host.contains("dev.to") {
                    0.8
                } else {
                    0.5
                }
            }
            None => 0.5,
        };

        Ok(GateFeatures {
            html_bytes,
            visible_text_chars,
            p_count,
            article_count,
            h1h2_count,
            script_bytes,
            has_og,
            has_jsonld_article,
            spa_markers,
            domain_prior,
        })
    }

    /// Extract with headless browser and return HTML
    async fn extract_with_headless(&self, url: &str) -> ApiResult<String> {
        match &self.state.config.headless_url {
            Some(headless_url) => {
                let render_request = serde_json::json!({
                    "url": url,
                    "wait_for": self.options.dynamic_wait_for,
                    "scroll_steps": self.options.scroll_steps
                });

                let response = self
                    .state
                    .http_client
                    .post(format!("{}/render", headless_url))
                    .json(&render_request)
                    .send()
                    .await
                    .map_err(|e| ApiError::dependency("headless_service", e.to_string()))?;

                if !response.status().is_success() {
                    return Err(ApiError::dependency(
                        "headless_service",
                        format!("Render request failed: {}", response.status()),
                    ));
                }

                response
                    .text()
                    .await
                    .map_err(|e| ApiError::dependency("headless_service", e.to_string()))
            }
            None => Err(ApiError::dependency(
                "headless_service",
                "Headless service not configured".to_string(),
            )),
        }
    }

    /// Check cache for processed content
    async fn check_cache(&self, cache_key: &str) -> ApiResult<Option<ProcessedContent>> {
        if self.options.cache_mode == "bypass" {
            return Ok(None);
        }

        let mut cache = self.state.cache.lock().await;
        cache
            .get::<ProcessedContent>(cache_key)
            .await
            .map_err(|e| ApiError::cache(format!("Cache read failed: {}", e)))
            .map(|entry| entry.map(|e| e.data))
    }

    /// Store processed content in cache
    async fn store_in_cache(&self, cache_key: &str, content: &ProcessedContent) -> ApiResult<()> {
        if self.options.cache_mode == "bypass" {
            return Ok(());
        }

        let mut cache = self.state.cache.lock().await;
        cache
            .set_simple(cache_key, content, self.state.config.cache_ttl)
            .await
            .map_err(|e| ApiError::cache(format!("Cache write failed: {}", e)))
    }

    /// Generate cache key with strategy config
    fn generate_cache_key(&self, url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        self.options.cache_mode.hash(&mut hasher);
        format!("{:?}", self.strategy_config.extraction).hash(&mut hasher);
        format!("{:?}", self.strategy_config.chunking.mode).hash(&mut hasher);

        format!(
            "riptide:strategies:v1:{}:{:x}",
            self.options.cache_mode,
            hasher.finish()
        )
    }
}

/// Create GitHub-specific CSS selectors
fn create_github_selectors() -> std::collections::HashMap<String, String> {
    let mut selectors = std::collections::HashMap::new();
    selectors.insert("title".to_string(), "h1.entry-title, .js-issue-title, .repository-content h1".to_string());
    selectors.insert("content".to_string(), ".entry-content, .markdown-body, .comment-body".to_string());
    selectors.insert("author".to_string(), ".author, .commit-author, .discussion-item-header a".to_string());
    selectors.insert("date".to_string(), "time, .commit-date, relative-time".to_string());
    selectors
}

/// Create blog-specific CSS selectors
fn create_blog_selectors() -> std::collections::HashMap<String, String> {
    let mut selectors = std::collections::HashMap::new();
    selectors.insert("title".to_string(), "h1, .entry-title, .post-title, [data-testid='storyTitle']".to_string());
    selectors.insert("content".to_string(), ".entry-content, .post-content, .story-content, article".to_string());
    selectors.insert("author".to_string(), ".author, .byline, .writer, [data-testid='authorName']".to_string());
    selectors.insert("date".to_string(), "time, .date, .published, [data-testid='storyPublishDate']".to_string());
    selectors
}

/// Create news site regex patterns
fn create_news_patterns() -> Vec<riptide_core::strategies::RegexPattern> {
    vec![
        riptide_core::strategies::RegexPattern {
            name: "title".to_string(),
            pattern: r"<title>([^<]+)</title>".to_string(),
            field: "title".to_string(),
            required: true,
        },
        riptide_core::strategies::RegexPattern {
            name: "points".to_string(),
            pattern: r"(\d+)\s+points?".to_string(),
            field: "score".to_string(),
            required: false,
        },
        riptide_core::strategies::RegexPattern {
            name: "comments".to_string(),
            pattern: r"(\d+)\s+comments?".to_string(),
            field: "comment_count".to_string(),
            required: false,
        },
    ]
}

impl Clone for StrategiesPipelineOrchestrator {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            options: self.options.clone(),
            strategy_config: self.strategy_config.clone(),
        }
    }
}