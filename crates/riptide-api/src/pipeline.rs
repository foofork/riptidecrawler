use crate::errors::{ApiError, ApiResult};
use crate::state::AppState;
use reqwest::Response;
use riptide_events::{BaseEvent, EventSeverity};
use riptide_fetch as fetch;
use riptide_pdf::{self as pdf, utils as pdf_utils};
use riptide_reliability::gate::{decide, score, Decision, GateFeatures};
use riptide_types::config::CrawlOptions;
use riptide_types::{ExtractedDoc, RenderMode};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use url::Url;

/// Convert from riptide_types ExtractedDoc to riptide_core ExtractedDoc
fn convert_html_doc(doc: riptide_types::ExtractedDoc) -> ExtractedDoc {
    ExtractedDoc {
        url: doc.url,
        title: doc.title,
        text: doc.text,
        quality_score: doc.quality_score,
        links: doc.links,
        byline: doc.byline,
        published_iso: doc.published_iso,
        markdown: doc.markdown,
        media: doc.media,
        parser_metadata: doc.parser_metadata,
        language: doc.language,
        reading_time: doc.reading_time,
        word_count: doc.word_count,
        categories: doc.categories,
        site_name: doc.site_name,
        description: doc.description,
        html: doc.html,
    }
}

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

        // Emit pipeline start event
        let mut start_event = BaseEvent::new(
            "pipeline.execution.started",
            "pipeline_orchestrator",
            EventSeverity::Info,
        );
        start_event.add_metadata("url", url);
        start_event.add_metadata("cache_key", &cache_key);
        if let Err(e) = self.state.event_bus.emit(start_event).await {
            warn!(error = %e, "Failed to emit pipeline start event");
        }

        // Step 1: Check cache first
        let cached_result = self.check_cache(&cache_key).await;
        if let Ok(Some(cached)) = cached_result {
            info!(url = %url, "Cache hit, returning cached result");

            // Emit cache hit event
            let mut cache_event = BaseEvent::new(
                "pipeline.cache.hit",
                "pipeline_orchestrator",
                EventSeverity::Info,
            );
            cache_event.add_metadata("url", url);
            cache_event.add_metadata("cache_key", &cache_key);
            if let Err(e) = self.state.event_bus.emit(cache_event).await {
                warn!(error = %e, "Failed to emit cache hit event");
            }

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
        let fetch_start = Instant::now();
        let (response, content_bytes, content_type) = self.fetch_content_with_type(url).await?;
        let http_status = response.status().as_u16();
        let fetch_duration = fetch_start.elapsed().as_secs_f64();
        self.state
            .metrics
            .record_phase_timing(crate::metrics::PhaseType::Fetch, fetch_duration);

        // Step 3: Check if this is PDF content
        if pdf_utils::is_pdf_content(content_type.as_deref(), &content_bytes)
            || matches!(self.options.render_mode, RenderMode::Pdf)
        {
            info!(url = %url, "Detected PDF content, processing with PDF pipeline");

            // Emit PDF processing event
            let mut pdf_event = BaseEvent::new(
                "pipeline.pdf.processing",
                "pipeline_orchestrator",
                EventSeverity::Info,
            );
            pdf_event.add_metadata("url", url);
            pdf_event.add_metadata("content_size", &content_bytes.len().to_string());
            if let Err(e) = self.state.event_bus.emit(pdf_event).await {
                warn!(error = %e, "Failed to emit PDF processing event");
            }

            let pdf_start = Instant::now();
            let document = self.process_pdf_content(&content_bytes, url).await?;
            let pdf_duration = pdf_start.elapsed().as_secs_f64();

            // Record PDF processing metrics
            self.state.metrics.record_pdf_processing_success(
                pdf_duration,
                document.word_count.unwrap_or(0) / 250, // Estimate pages from word count
                (content_bytes.len() as f64) / (1024.0 * 1024.0), // Memory in MB
            );

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
        let gate_start = Instant::now();
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

        let gate_duration = gate_start.elapsed();
        self.state
            .metrics
            .record_phase_timing(crate::metrics::PhaseType::Gate, gate_duration.as_secs_f64());
        self.state.metrics.record_gate_decision(&gate_decision_str);

        // INJECTION POINT 1: Enhanced gate metrics (non-blocking)
        tokio::spawn({
            let metrics = self.state.metrics.clone();
            let decision_str = gate_decision_str.clone();
            let score = quality_score;
            let features = gate_features.clone();
            let duration_ms = gate_duration.as_millis() as f64;
            async move {
                // Calculate feature ratios
                let text_ratio = if features.html_bytes > 0 {
                    features.visible_text_chars as f32 / features.html_bytes as f32
                } else {
                    0.0
                };
                let script_density = if features.html_bytes > 0 {
                    features.script_bytes as f32 / features.html_bytes as f32
                } else {
                    0.0
                };

                metrics.record_gate_decision_enhanced(
                    &decision_str,
                    score,
                    text_ratio,
                    script_density,
                    features.spa_markers,
                    duration_ms,
                );
            }
        });

        info!(
            url = %url,
            decision = %gate_decision_str,
            score = %quality_score,
            "Gate analysis complete"
        );

        // Emit gate decision event
        let mut gate_event = BaseEvent::new(
            "pipeline.gate.decision",
            "pipeline_orchestrator",
            EventSeverity::Info,
        );
        gate_event.add_metadata("url", url);
        gate_event.add_metadata("decision", &gate_decision_str);
        gate_event.add_metadata("quality_score", &quality_score.to_string());
        if let Err(e) = self.state.event_bus.emit(gate_event).await {
            warn!(error = %e, "Failed to emit gate decision event");
        }

        // Step 5: Extract content based on gate decision or skip extraction
        let extract_start = Instant::now();
        let document = if self.options.skip_extraction.unwrap_or(false) {
            // Skip extraction and return raw HTML only
            info!(url = %url, "Skipping extraction, returning raw HTML");
            ExtractedDoc {
                url: url.to_string(),
                html: Some(html_content.clone()),
                title: None,
                text: String::new(),
                quality_score: None,
                links: Vec::new(),
                byline: None,
                published_iso: None,
                markdown: None,
                parser_metadata: None,
                media: Vec::new(),
                language: None,
                reading_time: None,
                word_count: None,
                categories: Vec::new(),
                site_name: None,
                description: None,
            }
        } else {
            self.extract_content(&html_content, url, decision).await?
        };
        let extract_duration = extract_start.elapsed();

        // Record WASM extraction phase timing
        self.state.metrics.record_phase_timing(
            crate::metrics::PhaseType::Wasm,
            extract_duration.as_secs_f64(),
        );

        // INJECTION POINT 2: Enhanced extraction metrics (non-blocking)
        tokio::spawn({
            let metrics = self.state.metrics.clone();
            let mode = match decision {
                Decision::Raw => "raw",
                Decision::ProbesFirst => "probes",
                Decision::Headless => "headless",
            };
            let extraction_duration_ms = extract_duration.as_millis() as u64;
            let doc_clone = document.clone();
            async move {
                let quality_score = doc_clone.quality_score.unwrap_or(0) as f32;
                let content_length = doc_clone.text.len();
                let links_count = doc_clone.links.len();
                let images_count = doc_clone.media.len();
                let has_author = doc_clone.byline.is_some();
                let has_date = doc_clone.published_iso.is_some();

                metrics.record_extraction_result(
                    mode,
                    extraction_duration_ms,
                    true,
                    quality_score,
                    content_length,
                    links_count,
                    images_count,
                    has_author,
                    has_date,
                );
            }
        });

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

        // Emit pipeline completion event
        let mut completion_event = BaseEvent::new(
            "pipeline.execution.completed",
            "pipeline_orchestrator",
            EventSeverity::Info,
        );
        completion_event.add_metadata("url", url);
        completion_event.add_metadata("gate_decision", &gate_decision_str);
        completion_event.add_metadata("quality_score", &quality_score.to_string());
        completion_event.add_metadata("processing_time_ms", &processing_time_ms.to_string());
        completion_event.add_metadata("http_status", &http_status.to_string());
        if let Err(e) = self.state.event_bus.emit(completion_event).await {
            warn!(error = %e, "Failed to emit pipeline completion event");
        }

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
        info!(
            url = %url,
            file_size = pdf_bytes.len(),
            "Processing PDF content"
        );

        // Acquire PDF resources with RAII guard for automatic cleanup
        let pdf_guard_result = self
            .state
            .resource_manager
            .acquire_pdf_resources()
            .await
            .map_err(|e| ApiError::extraction(format!("Failed to acquire PDF resources: {}", e)))?;

        let _pdf_guard = match pdf_guard_result {
            crate::resource_manager::ResourceResult::Success(guard) => guard,
            crate::resource_manager::ResourceResult::Timeout => {
                return Err(ApiError::timeout(
                    "pdf_resource",
                    "PDF resource acquisition timeout".to_string(),
                ));
            }
            crate::resource_manager::ResourceResult::ResourceExhausted => {
                return Err(ApiError::extraction(
                    "PDF resources exhausted, try again later".to_string(),
                ));
            }
            crate::resource_manager::ResourceResult::MemoryPressure => {
                return Err(ApiError::extraction(
                    "System under memory pressure, PDF processing unavailable".to_string(),
                ));
            }
            crate::resource_manager::ResourceResult::RateLimited { retry_after } => {
                return Err(ApiError::extraction(format!(
                    "Rate limited, retry after {:?}",
                    retry_after
                )));
            }
            crate::resource_manager::ResourceResult::Error(msg) => {
                return Err(ApiError::extraction(format!("PDF resource error: {}", msg)));
            }
        };

        let processor = pdf::create_pdf_processor();
        let pdf_bytes_vec = pdf_bytes.to_vec();
        let url_str = url.to_string();

        processor
            .process_pdf_bytes(&pdf_bytes_vec)
            .await
            .map_err(|e| ApiError::extraction(format!("PDF processing error: {}", e)))
            .inspect(|document| {
                info!(
                        url = %url_str,
                    text_length = document.text.len(),
                    title = ?document.title,
                    "PDF processing completed successfully"
                );
            })
            .or_else(|e| {
                error!(
                    url = %url,
                    error = %e,
                    "PDF processing failed"
                );

                // Track PDF processing failure
                self.state.metrics.record_pdf_processing_failure(false);

                // Return a fallback document with error information
                Ok(ExtractedDoc {
                    url: url.to_string(),
                    title: Some("PDF Processing Failed".to_string()),
                    byline: None,
                    published_iso: None,
                    markdown: Some(format!("**PDF Processing Error**: {}", e)),
                    text: format!("PDF processing failed: {}", e),
                    links: Vec::new(),
                    media: Vec::new(),
                    language: None,
                    parser_metadata: None,
                    reading_time: Some(1),
                    quality_score: Some(20), // Low quality due to error
                    word_count: Some(5),
                    categories: vec!["pdf".to_string(), "error".to_string()],
                    site_name: None,
                    description: Some("Failed to process PDF document".to_string()),
                    html: None,
                })
            })
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
    ///
    /// This method now uses ReliableExtractor for enhanced retry logic and
    /// graceful degradation, wrapping extraction calls with the reliability module.
    async fn extract_content(
        &self,
        html: &str,
        url: &str,
        decision: Decision,
    ) -> ApiResult<ExtractedDoc> {
        use crate::reliability_integration::WasmExtractorAdapter;
        use riptide_reliability::ExtractionMode;

        // Create adapter for WasmExtractor to work with ReliableExtractor (with metrics)
        let extractor_adapter = WasmExtractorAdapter::with_metrics(
            self.state.extractor.clone(),
            self.state.metrics.clone(),
        );

        // Map gate Decision to ExtractionMode
        let extraction_mode = match decision {
            Decision::Raw => ExtractionMode::Fast,
            Decision::ProbesFirst => ExtractionMode::ProbesFirst,
            Decision::Headless => ExtractionMode::Headless,
        };

        // Use ReliableExtractor with retry and circuit breaker patterns
        let result = self
            .state
            .reliable_extractor
            .extract_with_reliability(
                url,
                extraction_mode,
                &extractor_adapter,
                self.state.config.headless_url.as_deref(),
            )
            .await;

        match result {
            Ok(doc) => {
                // Emit reliability success event
                let mut event = riptide_events::BaseEvent::new(
                    "pipeline.extraction.reliable_success",
                    "pipeline_orchestrator",
                    riptide_events::EventSeverity::Info,
                );
                event.add_metadata("url", url);
                event.add_metadata("decision", &format!("{:?}", decision));
                event.add_metadata("content_length", &doc.text.len().to_string());
                if let Err(e) = self.state.event_bus.emit(event).await {
                    warn!(error = %e, "Failed to emit reliability success event");
                }

                Ok(doc)
            }
            Err(e) => {
                // Log the failure and attempt fallback to direct WASM extraction
                warn!(
                    url = %url,
                    error = %e,
                    "ReliableExtractor failed, attempting direct WASM fallback"
                );

                // Emit reliability failure event
                let mut event = riptide_events::BaseEvent::new(
                    "pipeline.extraction.reliable_failure",
                    "pipeline_orchestrator",
                    riptide_events::EventSeverity::Warn,
                );
                event.add_metadata("url", url);
                event.add_metadata("error", &e.to_string());
                if let Err(e) = self.state.event_bus.emit(event).await {
                    warn!(error = %e, "Failed to emit reliability failure event");
                }

                // Track extraction errors
                self.state
                    .metrics
                    .record_error(crate::metrics::ErrorType::Wasm);

                // Fallback: Try direct WASM extraction as last resort
                self.fallback_to_wasm_extraction(html, url).await
            }
        }
    }

    /// Fallback to direct WASM extraction when ReliableExtractor fails
    async fn fallback_to_wasm_extraction(&self, html: &str, url: &str) -> ApiResult<ExtractedDoc> {
        info!(url = %url, "Attempting direct WASM fallback extraction");

        let extractor = self.state.extractor.clone();
        let html_bytes = html.as_bytes().to_vec();
        let url_str = url.to_string();

        extractor
            .extract(&html_bytes, &url_str, "article")
            .map(convert_html_doc)
            .map_err(|e| {
                error!(url = %url_str, error = %e, "All extraction methods failed");
                self.state
                    .metrics
                    .record_error(crate::metrics::ErrorType::Wasm);
                ApiError::extraction(format!("All extraction attempts failed: {}", e))
            })
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
            .map_err(|e| {
                self.state
                    .metrics
                    .record_error(crate::metrics::ErrorType::Redis);
                ApiError::cache(format!("Cache read failed: {}", e))
            })
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
            .map_err(|e| {
                self.state
                    .metrics
                    .record_error(crate::metrics::ErrorType::Redis);
                ApiError::cache(format!("Cache write failed: {}", e))
            })
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
