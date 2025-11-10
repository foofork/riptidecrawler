//! CrawlHandlerFacade - Business logic extracted from crawl handlers
//!
//! This facade encapsulates the complex orchestration logic from crawl handlers:
//! - Pipeline selection (enhanced vs standard)
//! - Result transformation and formatting
//! - Statistics calculation
//! - Chunking application
//! - Spider mode routing
//!
//! **Purpose**: Reduce handler LOC from 395 to ~60 by moving all business logic here.

use crate::errors::ApiError;
use crate::handlers::chunking::apply_content_chunking;
use crate::models::{
    CrawlResponse, CrawlResult, CrawlStatistics, ErrorInfo, GateDecisionBreakdown,
};
use crate::pipeline::PipelineOrchestrator;
use crate::pipeline_enhanced::EnhancedPipelineOrchestrator;
use crate::state::AppState;
use riptide_facade::facades::chunking::ChunkParameters;
use riptide_types::config::CrawlOptions;
use riptide_types::ExtractedDoc;
use tracing::{debug, info};

/// Facade for crawl handler business logic
///
/// This facade extracts all business logic from the crawl handlers,
/// leaving handlers as thin HTTP adapters that only handle:
/// - Request/response serialization
/// - Trace context extraction
/// - Event emission (optional - could move here)
pub struct CrawlHandlerFacade {
    state: AppState,
}

impl CrawlHandlerFacade {
    /// Create a new crawl handler facade
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Execute batch crawl through appropriate pipeline
    ///
    /// This method handles the complete crawl workflow:
    /// 1. Validate request (done by caller/middleware)
    /// 2. Select pipeline (enhanced vs standard)
    /// 3. Execute batch
    /// 4. Apply chunking if requested
    /// 5. Transform results to API format
    /// 6. Calculate statistics
    ///
    /// # Arguments
    ///
    /// * `urls` - List of URLs to crawl
    /// * `options` - Crawl options (concurrency, caching, etc.)
    ///
    /// # Returns
    ///
    /// Complete crawl response with results and statistics
    pub async fn crawl_batch(
        &self,
        urls: &[String],
        options: CrawlOptions,
    ) -> Result<CrawlResponse, ApiError> {
        debug!(
            concurrency = options.concurrency,
            cache_mode = %options.cache_mode,
            enhanced_pipeline = self.state.config.enhanced_pipeline_config.enable_enhanced_pipeline,
            "Executing batch crawl"
        );

        // Select and execute pipeline
        let (pipeline_results, stats) = self.execute_pipeline(urls, &options).await;

        // Transform results to API format
        let (crawl_results, from_cache_count) = self
            .transform_pipeline_results(pipeline_results, urls, &options)
            .await;

        // Build response with statistics
        let response = self.build_response(urls, crawl_results, from_cache_count, stats);

        Ok(response)
    }

    /// Execute spider crawl mode
    ///
    /// Routes to spider facade for deep crawling with link following.
    ///
    /// # Arguments
    ///
    /// * `seed_urls` - Starting URLs for spider crawl
    /// * `options` - Crawl options (spider depth, strategy, etc.)
    ///
    /// # Returns
    ///
    /// Crawl response formatted from spider results
    #[cfg(feature = "spider")]
    pub async fn crawl_spider_mode(
        &self,
        seed_urls: &[String],
        _options: &CrawlOptions,
    ) -> Result<CrawlResponse, ApiError> {
        use crate::handlers::shared::spider::parse_seed_urls;

        // Get spider facade from state
        let spider_facade = self
            .state
            .spider_facade
            .as_ref()
            .ok_or_else(|| ApiError::ConfigError {
                message: "SpiderFacade not initialized. Spider functionality requires the 'spider' feature.".to_string(),
            })?;

        // Parse URLs
        let parsed_seed_urls = parse_seed_urls(seed_urls)?;

        debug!(
            seed_count = parsed_seed_urls.len(),
            "Executing spider crawl via SpiderFacade"
        );

        // Execute spider crawl
        let spider_result =
            spider_facade
                .crawl(parsed_seed_urls)
                .await
                .map_err(|e| ApiError::InternalError {
                    message: format!("Spider crawl failed: {}", e),
                })?;

        // Transform spider result to crawl response
        let response = self.transform_spider_result(spider_result);

        Ok(response)
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

    /// Select and execute the appropriate pipeline
    async fn execute_pipeline(
        &self,
        urls: &[String],
        options: &CrawlOptions,
    ) -> (
        Vec<Option<crate::pipeline::PipelineResult>>,
        crate::pipeline::PipelineStats,
    ) {
        if self
            .state
            .config
            .enhanced_pipeline_config
            .enable_enhanced_pipeline
        {
            info!("Using enhanced pipeline orchestrator with detailed phase timing");
            let enhanced_pipeline =
                EnhancedPipelineOrchestrator::new(self.state.clone(), options.clone());
            let (results, enhanced_stats) = enhanced_pipeline.execute_batch_enhanced(urls).await;

            // Convert enhanced results to standard format
            let standard_results = self.convert_enhanced_results(results);
            let standard_stats = self.convert_enhanced_stats(enhanced_stats);

            (standard_results, standard_stats)
        } else {
            info!("Using standard pipeline orchestrator");
            let pipeline = PipelineOrchestrator::new(self.state.clone(), options.clone());
            pipeline.execute_batch(urls).await
        }
    }

    /// Convert enhanced pipeline results to standard format
    fn convert_enhanced_results(
        &self,
        enhanced_results: Vec<Option<crate::pipeline_enhanced::EnhancedPipelineResult>>,
    ) -> Vec<Option<crate::pipeline::PipelineResult>> {
        enhanced_results
            .into_iter()
            .map(|opt_result| {
                opt_result.map(|enhanced_result| {
                    let document =
                        enhanced_result
                            .document
                            .unwrap_or_else(|| riptide_types::ExtractedDoc {
                                url: enhanced_result.url.clone(),
                                title: None,
                                text: String::new(),
                                quality_score: None,
                                links: Vec::new(),
                                byline: None,
                                published_iso: None,
                                markdown: None,
                                media: Vec::new(),
                                parser_metadata: None,
                                language: None,
                                reading_time: None,
                                word_count: None,
                                categories: Vec::new(),
                                site_name: None,
                                description: None,
                                html: None,
                            });

                    crate::pipeline::PipelineResult {
                        document,
                        from_cache: enhanced_result.cache_hit,
                        gate_decision: enhanced_result.gate_decision,
                        quality_score: enhanced_result.quality_score,
                        processing_time_ms: enhanced_result.total_duration_ms,
                        cache_key: format!("riptide:v1:enhanced:{}", enhanced_result.url),
                        http_status: 200,
                    }
                })
            })
            .collect()
    }

    /// Convert enhanced stats to standard format
    fn convert_enhanced_stats(
        &self,
        enhanced_stats: crate::pipeline_enhanced::EnhancedBatchStats,
    ) -> crate::pipeline::PipelineStats {
        crate::pipeline::PipelineStats {
            total_processed: enhanced_stats.total_urls,
            cache_hits: enhanced_stats.cache_hits,
            successful_extractions: enhanced_stats.successful,
            failed_extractions: enhanced_stats.failed,
            gate_decisions: enhanced_stats.gate_decisions,
            avg_processing_time_ms: enhanced_stats.avg_processing_time_ms,
            total_processing_time_ms: enhanced_stats.total_duration_ms,
        }
    }

    /// Transform pipeline results to API format
    async fn transform_pipeline_results(
        &self,
        pipeline_results: Vec<Option<crate::pipeline::PipelineResult>>,
        urls: &[String],
        options: &CrawlOptions,
    ) -> (Vec<CrawlResult>, usize) {
        let mut crawl_results = Vec::with_capacity(urls.len());
        let mut from_cache_count = 0;

        for (index, pipeline_result) in pipeline_results.into_iter().enumerate() {
            let url = &urls[index];

            match pipeline_result {
                Some(result) => {
                    if result.from_cache {
                        from_cache_count += 1;
                    }

                    // Apply chunking if requested
                    let document = self.apply_chunking(result.document, options).await;

                    crawl_results.push(CrawlResult {
                        url: url.clone(),
                        status: result.http_status,
                        from_cache: result.from_cache,
                        gate_decision: result.gate_decision,
                        quality_score: result.quality_score,
                        processing_time_ms: result.processing_time_ms,
                        document: Some(document),
                        error: None,
                        cache_key: result.cache_key,
                    });
                }
                None => {
                    crawl_results.push(CrawlResult {
                        url: url.clone(),
                        status: 0,
                        from_cache: false,
                        gate_decision: "failed".to_string(),
                        quality_score: 0.0,
                        processing_time_ms: 0,
                        document: None,
                        error: Some(ErrorInfo {
                            error_type: "pipeline_error".to_string(),
                            message: "Failed to process URL".to_string(),
                            retryable: true,
                        }),
                        cache_key: "".to_string(),
                    });
                }
            }
        }

        (crawl_results, from_cache_count)
    }

    /// Apply content chunking if configured
    async fn apply_chunking(&self, document: ExtractedDoc, options: &CrawlOptions) -> ExtractedDoc {
        if let Some(ref _chunking_config) = options.chunking_config {
            apply_content_chunking(
                document.clone(),
                "adaptive".to_string(),
                Some(ChunkParameters {
                    chunk_size: 1000,
                    overlap_size: 200,
                    min_chunk_size: 100,
                    preserve_sentences: true,
                    window_size: None,
                }),
            )
            .await
            .unwrap_or(document)
        } else {
            document
        }
    }

    /// Build complete crawl response
    fn build_response(
        &self,
        urls: &[String],
        crawl_results: Vec<CrawlResult>,
        from_cache_count: usize,
        stats: crate::pipeline::PipelineStats,
    ) -> CrawlResponse {
        // Calculate cache hit rate
        let cache_hit_rate = if !urls.is_empty() {
            from_cache_count as f64 / urls.len() as f64
        } else {
            0.0
        };

        // Build statistics
        let statistics = CrawlStatistics {
            total_processing_time_ms: stats.total_processing_time_ms,
            avg_processing_time_ms: stats.avg_processing_time_ms,
            gate_decisions: GateDecisionBreakdown {
                raw: stats.gate_decisions.raw,
                probes_first: stats.gate_decisions.probes_first,
                headless: stats.gate_decisions.headless,
                cached: from_cache_count,
            },
            cache_hit_rate,
        };

        CrawlResponse {
            total_urls: urls.len(),
            successful: stats.successful_extractions,
            failed: stats.failed_extractions,
            from_cache: from_cache_count,
            results: crawl_results,
            statistics,
        }
    }

    /// Transform spider result to crawl response format
    fn transform_spider_result(
        &self,
        spider_result: riptide_facade::facades::CrawlSummary,
    ) -> CrawlResponse {
        let mut crawl_results = Vec::new();

        // Create results for discovered URLs
        for (index, discovered_url) in spider_result.discovered_urls.iter().enumerate() {
            let is_successful = index < spider_result.pages_crawled as usize;
            crawl_results.push(CrawlResult {
                url: discovered_url.clone(),
                status: if is_successful { 200 } else { 0 },
                from_cache: false,
                gate_decision: "spider_crawl".to_string(),
                quality_score: if is_successful { 0.8 } else { 0.0 },
                processing_time_ms: (spider_result.duration_secs * 1000.0) as u64
                    / spider_result.pages_crawled.max(1),
                document: None,
                error: if !is_successful {
                    Some(ErrorInfo {
                        error_type: "spider_crawl_failed".to_string(),
                        message: "Page failed during spider crawl".to_string(),
                        retryable: true,
                    })
                } else {
                    None
                },
                cache_key: format!("spider:v1:{}", index),
            });
        }

        let statistics = CrawlStatistics {
            total_processing_time_ms: (spider_result.duration_secs * 1000.0) as u64,
            avg_processing_time_ms: if !spider_result.discovered_urls.is_empty() {
                (spider_result.duration_secs * 1000.0) / spider_result.discovered_urls.len() as f64
            } else {
                0.0
            },
            gate_decisions: GateDecisionBreakdown {
                raw: spider_result.pages_crawled as usize,
                probes_first: 0,
                headless: 0,
                cached: 0,
            },
            cache_hit_rate: 0.0,
        };

        CrawlResponse {
            total_urls: spider_result.discovered_urls.len(),
            successful: spider_result.pages_crawled as usize,
            failed: spider_result.pages_failed as usize,
            from_cache: 0,
            results: crawl_results,
            statistics,
        }
    }
}
