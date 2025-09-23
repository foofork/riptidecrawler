use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use reqwest::Response;
use riptide_core::{
    fetch,
    gate::{decide, score, Decision, GateFeatures},
    pdf::{self, utils as pdf_utils},
    types::{CrawlOptions, ExtractedDoc, RenderMode},
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use url::Url;

/// Pipeline execution result containing the extracted document and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// The extracted document content
    pub document: ExtractedDoc,

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
}

/// Pipeline execution statistics for monitoring and optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Total URLs processed
    pub total_processed: usize,

    /// Number of cache hits
    pub cache_hits: usize,

    /// Number of successful extractions
    pub successful_extractions: usize,

    /// Number of failed extractions
    pub failed_extractions: usize,

    /// Gate decision breakdown
    pub gate_decisions: GateDecisionStats,

    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,

    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateDecisionStats {
    pub raw: usize,
    pub probes_first: usize,
    pub headless: usize,
}

/// Core pipeline orchestrator for the fetch -> gate -> extract workflow.
///
/// This orchestrator handles the complete crawling pipeline:
/// 1. **Cache Check**: Look for cached content first
/// 2. **Fetch**: Retrieve HTML content from the target URL
/// 3. **Gate Analysis**: Analyze content to determine extraction strategy
/// 4. **Extract**: Use appropriate extraction method (fast or headless)
/// 5. **Cache Store**: Save results for future requests
///
/// The pipeline is designed for high throughput and includes comprehensive
/// error handling, timeout management, and performance monitoring.
pub struct PipelineOrchestrator {
    state: AppState,
    options: CrawlOptions,
}

impl PipelineOrchestrator {
    /// Create a new pipeline orchestrator with the given state and options.
    pub fn new(state: AppState, options: CrawlOptions) -> Self {
        Self { state, options }
    }

    /// Execute the complete pipeline for a single URL.
    ///
    /// This method orchestrates the entire crawling workflow with proper
    /// error handling, caching, and performance monitoring.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to crawl
    ///
    /// # Returns
    ///
    /// A `PipelineResult` containing the extracted content and metadata.
    ///
    /// # Errors
    ///
    /// Returns `ApiError` for various failure scenarios:
    /// - Invalid URLs
    /// - Network failures
    /// - Extraction failures
    /// - Cache errors
    /// - Timeouts
    pub async fn execute_single(&self, url: &str) -> ApiResult<PipelineResult> {
        let start_time = Instant::now();
        let cache_key = self.generate_cache_key(url);

        info!(url = %url, cache_key = %cache_key, "Starting pipeline execution");

        // Step 1: Check cache first
        let cached_result = self.check_cache(&cache_key).await;
        if let Ok(Some(cached)) = cached_result {
            info!(url = %url, "Cache hit, returning cached result");
            return Ok(PipelineResult {
                document: cached,
                from_cache: true,
                gate_decision: "cached".to_string(),
                quality_score: 1.0, // Cached content is assumed to be good
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                cache_key,
                http_status: 200, // Assume success for cached content
            });
        }

        // Step 2: Fetch content
        debug!(url = %url, "Cache miss, fetching content");
        let (response, content_bytes, content_type) = self.fetch_content_with_type(url).await?;
        let http_status = response.status().as_u16();

        // Step 3: Check if this is PDF content
        if pdf_utils::is_pdf_content(content_type.as_deref(), &content_bytes)
            || matches!(self.options.render_mode, RenderMode::Pdf)
        {
            info!(url = %url, "Detected PDF content, processing with PDF pipeline");
            let document = self.process_pdf_content(&content_bytes, url).await?;

            // Cache the PDF result
            if let Err(e) = self.store_in_cache(&cache_key, &document).await {
                warn!(error = %e, "Failed to cache PDF result, continuing anyway");
            }

            let processing_time_ms = start_time.elapsed().as_millis() as u64;

            return Ok(PipelineResult {
                document,
                from_cache: false,
                gate_decision: "pdf".to_string(),
                quality_score: 0.95, // PDFs typically have high quality
                processing_time_ms,
                cache_key,
                http_status,
            });
        }

        // Convert bytes back to string for HTML processing
        let html_content = String::from_utf8_lossy(&content_bytes).to_string();

        // Step 4: Gate analysis for HTML content
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
        }
        .to_string();

        info!(
            url = %url,
            decision = %gate_decision_str,
            score = %quality_score,
            "Gate analysis complete"
        );

        // Step 5: Extract content based on gate decision
        let document = self.extract_content(&html_content, url, decision).await?;

        // Step 6: Cache the result
        if let Err(e) = self.store_in_cache(&cache_key, &document).await {
            warn!(error = %e, "Failed to cache result, continuing anyway");
        }

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            url = %url,
            processing_time_ms = %processing_time_ms,
            "Pipeline execution complete"
        );

        Ok(PipelineResult {
            document,
            from_cache: false,
            gate_decision: gate_decision_str,
            quality_score,
            processing_time_ms,
            cache_key,
            http_status,
        })
    }

    /// Execute the pipeline for multiple URLs concurrently.
    ///
    /// This method processes multiple URLs in parallel while respecting
    /// concurrency limits and providing aggregate statistics.
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to process
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Vector of results (Some(result) for success, None for failure)
    /// - Overall pipeline statistics
    ///
    /// # Performance
    ///
    /// Uses semaphore-based concurrency control to prevent overwhelming
    /// target servers while maximizing throughput.
    pub async fn execute_batch(
        &self,
        urls: &[String],
    ) -> (Vec<Option<PipelineResult>>, PipelineStats) {
        let start_time = Instant::now();
        let mut stats = PipelineStats {
            total_processed: urls.len(),
            cache_hits: 0,
            successful_extractions: 0,
            failed_extractions: 0,
            gate_decisions: GateDecisionStats::default(),
            avg_processing_time_ms: 0.0,
            total_processing_time_ms: 0,
        };

        info!(url_count = urls.len(), "Starting batch pipeline execution");

        // Use semaphore to limit concurrency
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.options.concurrency));

        // Process URLs concurrently
        let tasks: Vec<_> = urls
            .iter()
            .enumerate()
            .map(|(index, url)| {
                let semaphore = semaphore.clone();
                let pipeline = self.clone();
                let url = url.clone();

                tokio::spawn(async move {
                    let _permit = match semaphore.acquire().await {
                        Ok(permit) => permit,
                        Err(e) => {
                            error!(url = %url, index = index, error = %e, "Failed to acquire semaphore permit");
                            return None;
                        }
                    };

                    match pipeline.execute_single(&url).await {
                        Ok(result) => {
                            debug!(url = %url, index = index, "URL processed successfully");
                            Some(result)
                        }
                        Err(e) => {
                            error!(url = %url, index = index, error = %e, "URL processing failed");
                            None
                        }
                    }
                })
            })
            .collect();

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;

        // Collect results and compute statistics
        let mut pipeline_results = Vec::with_capacity(urls.len());
        let mut total_time = 0u64;

        for result in results {
            match result {
                Ok(Some(pipeline_result)) => {
                    stats.successful_extractions += 1;
                    total_time += pipeline_result.processing_time_ms;

                    if pipeline_result.from_cache {
                        stats.cache_hits += 1;
                    }

                    match pipeline_result.gate_decision.as_str() {
                        "raw" => stats.gate_decisions.raw += 1,
                        "probes_first" => stats.gate_decisions.probes_first += 1,
                        "headless" => stats.gate_decisions.headless += 1,
                        _ => {} // cached or unknown
                    }

                    pipeline_results.push(Some(pipeline_result));
                }
                Ok(None) | Err(_) => {
                    stats.failed_extractions += 1;
                    pipeline_results.push(None);
                }
            }
        }

        stats.total_processing_time_ms = start_time.elapsed().as_millis() as u64;
        stats.avg_processing_time_ms = if stats.successful_extractions > 0 {
            total_time as f64 / stats.successful_extractions as f64
        } else {
            0.0
        };

        info!(
            total_urls = urls.len(),
            successful = stats.successful_extractions,
            failed = stats.failed_extractions,
            cache_hits = stats.cache_hits,
            total_time_ms = stats.total_processing_time_ms,
            "Batch pipeline execution complete"
        );

        (pipeline_results, stats)
    }

    /// Fetch content with content type detection for PDF handling.
    async fn fetch_content_with_type(
        &self,
        url: &str,
    ) -> ApiResult<(Response, Vec<u8>, Option<String>)> {
        let fetch_timeout = Duration::from_secs(15);
        let response = timeout(fetch_timeout, fetch::get(&self.state.http_client, url))
            .await
            .map_err(|_| ApiError::timeout("content_fetch", format!("Timeout fetching {}", url)))?
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        // Extract content type before consuming response
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .map(|s| s.to_string());

        let content_bytes = timeout(fetch_timeout, response.bytes())
            .await
            .map_err(|_| {
                ApiError::timeout(
                    "content_read",
                    format!("Timeout reading content from {}", url),
                )
            })?
            .map_err(|e| ApiError::fetch(url, format!("Failed to read response body: {}", e)))?
            .to_vec();

        // Recreate response for status code (since we consumed it for bytes)
        let response = fetch::get(&self.state.http_client, url)
            .await
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        Ok((response, content_bytes, content_type))
    }

    /// Process PDF content using the PDF pipeline.
    async fn process_pdf_content(&self, pdf_bytes: &[u8], url: &str) -> ApiResult<ExtractedDoc> {
        let _pdf_config = self.options.pdf_config.clone().unwrap_or_default();

        info!(
            url = %url,
            file_size = pdf_bytes.len(),
            "Processing PDF content"
        );

        match pdf::process_pdf(pdf_bytes).await {
            Ok(document) => {
                info!(
                    url = %url,
                    text_length = document.text.len(),
                    title = ?document.title,
                    "PDF processing completed successfully"
                );
                Ok(document)
            }
            Err(e) => {
                error!(
                    url = %url,
                    error = %e,
                    "PDF processing failed"
                );

                // Return a fallback document with error information
                Ok(ExtractedDoc {
                    url: url.to_string(),
                    title: Some("PDF Processing Failed".to_string()),
                    byline: None,
                    published_iso: None,
                    markdown: format!("**PDF Processing Error**: {}", e),
                    text: format!("PDF processing failed: {}", e),
                    links: Vec::new(),
                    media: Vec::new(),
                    language: None,
                    reading_time: Some(1),
                    quality_score: Some(20), // Low quality due to error
                    word_count: Some(5),
                    categories: vec!["pdf".to_string(), "error".to_string()],
                    site_name: None,
                    description: Some("Failed to process PDF document".to_string()),
                })
            }
        }
    }

    /// Fetch HTML content from a URL with timeout and error handling.
    async fn fetch_content(&self, url: &str) -> ApiResult<(Response, String)> {
        let fetch_timeout = Duration::from_secs(15);

        let response = timeout(fetch_timeout, fetch::get(&self.state.http_client, url))
            .await
            .map_err(|_| ApiError::timeout("content_fetch", format!("Timeout fetching {}", url)))?
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        let content = timeout(fetch_timeout, response.text())
            .await
            .map_err(|_| {
                ApiError::timeout(
                    "content_read",
                    format!("Timeout reading content from {}", url),
                )
            })?
            .map_err(|e| ApiError::fetch(url, format!("Failed to read response body: {}", e)))?;

        // Recreate response for status code (since we consumed it for text)
        let response = fetch::get(&self.state.http_client, url)
            .await
            .map_err(|e| ApiError::fetch(url, e.to_string()))?;

        Ok((response, content))
    }

    /// Analyze HTML content to extract gate features for decision making.
    async fn analyze_content(&self, html: &str, url: &str) -> ApiResult<GateFeatures> {
        // Parse URL for domain analysis
        let parsed_url = Url::parse(url)
            .map_err(|e| ApiError::invalid_url(url, format!("URL parsing failed: {}", e)))?;

        // Extract basic features
        let html_bytes = html.len();
        let visible_text_chars = html.chars().filter(|c| !c.is_control()).count();

        // Count HTML elements (simplified approach)
        let p_count = html.matches("<p").count() as u32;
        let article_count =
            html.matches("<article").count() as u32 + html.matches("<main").count() as u32;
        let h1h2_count = html.matches("<h1").count() as u32 + html.matches("<h2").count() as u32;

        // Estimate script content size
        let script_bytes = html
            .split("<script")
            .skip(1)
            .map(|part| {
                part.find("</script>")
                    .map(|end| end + 9) // Include closing tag
                    .unwrap_or(part.len())
            })
            .sum::<usize>();

        // Check for metadata
        let has_og = html.contains("property=\"og:") || html.contains("property='og:");
        let has_jsonld_article =
            html.contains("\"@type\":\"Article\"") || html.contains("'@type':'Article'");

        // Detect SPA markers
        let mut spa_markers = 0u8;
        if html.contains("__NEXT_DATA__") {
            spa_markers += 1;
        }
        if html.contains("data-reactroot") || html.contains("data-react-helmet") {
            spa_markers += 1;
        }
        if html.contains("id=\"root\"") && html.matches("<div").count() > 20 {
            spa_markers += 1;
        }
        if script_bytes > html_bytes / 2 {
            spa_markers += 1;
        } // More than 50% scripts

        // Domain prior (simplified - could be enhanced with historical data)
        let domain_prior = match parsed_url.host_str() {
            Some(host) => {
                if host.contains("wikipedia.") || host.contains("github.") {
                    0.9 // High-quality content domains
                } else if host.contains("medium.") || host.contains("dev.to") {
                    0.8 // Blog platforms
                } else {
                    0.5 // Neutral prior
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

    /// Extract content using the appropriate method based on gate decision.
    async fn extract_content(
        &self,
        html: &str,
        url: &str,
        decision: Decision,
    ) -> ApiResult<ExtractedDoc> {
        match decision {
            Decision::Raw => {
                // Use fast WASM extraction
                self.state
                    .extractor
                    .extract(html.as_bytes(), url, "article")
                    .map_err(|e| ApiError::extraction(format!("WASM extraction failed: {}", e)))
            }
            Decision::ProbesFirst => {
                // Try fast extraction first, fallback to headless if needed
                match self
                    .state
                    .extractor
                    .extract(html.as_bytes(), url, "article")
                {
                    Ok(doc) => {
                        // Validate extraction quality
                        if doc.text.len() > 100 && doc.title.is_some() {
                            Ok(doc)
                        } else {
                            debug!(url = %url, "Fast extraction produced low-quality result, falling back to headless");
                            self.extract_with_headless(url).await
                        }
                    }
                    Err(_) => {
                        debug!(url = %url, "Fast extraction failed, falling back to headless");
                        self.extract_with_headless(url).await
                    }
                }
            }
            Decision::Headless => {
                // Use headless browser extraction
                self.extract_with_headless(url).await
            }
        }
    }

    /// Extract content using headless browser rendering.
    async fn extract_with_headless(&self, url: &str) -> ApiResult<ExtractedDoc> {
        // If headless service is not configured, fall back to fast extraction
        match &self.state.config.headless_url {
            Some(headless_url) => self.render_and_extract(url, headless_url).await,
            None => {
                warn!(url = %url, "Headless extraction requested but no headless service configured, using fast extraction");
                // Try fast extraction as fallback
                self.state
                    .extractor
                    .extract(&[], url, "article") // Empty HTML, will need to fetch first
                    .map_err(|e| ApiError::extraction(format!("Fallback extraction failed: {}", e)))
            }
        }
    }

    /// Render page with headless browser and extract content.
    async fn render_and_extract(&self, url: &str, headless_url: &str) -> ApiResult<ExtractedDoc> {
        // Construct headless service request
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

        let rendered_html = response
            .text()
            .await
            .map_err(|e| ApiError::dependency("headless_service", e.to_string()))?;

        // Extract from rendered HTML
        self.state
            .extractor
            .extract(rendered_html.as_bytes(), url, "article")
            .map_err(|e| ApiError::extraction(format!("Headless extraction failed: {}", e)))
    }

    /// Check cache for existing content.
    async fn check_cache(&self, cache_key: &str) -> ApiResult<Option<ExtractedDoc>> {
        if self.options.cache_mode == "bypass" {
            return Ok(None);
        }

        let mut cache = self.state.cache.lock().await;
        cache
            .get::<ExtractedDoc>(cache_key)
            .await
            .map_err(|e| ApiError::cache(format!("Cache read failed: {}", e)))
            .map(|entry| entry.map(|e| e.data))
    }

    /// Store content in cache.
    async fn store_in_cache(&self, cache_key: &str, document: &ExtractedDoc) -> ApiResult<()> {
        if self.options.cache_mode == "bypass" {
            return Ok(());
        }

        let mut cache = self.state.cache.lock().await;
        cache
            .set_simple(cache_key, document, self.state.config.cache_ttl)
            .await
            .map_err(|e| ApiError::cache(format!("Cache write failed: {}", e)))
    }

    /// Generate a cache key for a URL.
    fn generate_cache_key(&self, url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        self.options.cache_mode.hash(&mut hasher);
        format!(
            "riptide:v1:{}:{:x}",
            self.options.cache_mode,
            hasher.finish()
        )
    }
}

// Implement Clone for PipelineOrchestrator to support concurrent execution
impl Clone for PipelineOrchestrator {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            options: self.options.clone(),
        }
    }
}
