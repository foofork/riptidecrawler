use anyhow::Result;
use riptide_fetch::ReliableHttpClient;
/// Enhanced reliability patterns for RipTide Phase-2 Lite
///
/// This module implements timeout and reliability patterns for robust web scraping:
/// 1. Fetch reliability - 3s connect, 15-20s total timeout, 1 retry for idempotent requests
/// 2. Headless resilience - DOMContentLoaded + 1s idle, 3s hard cap, circuit breaker
/// 3. Graceful degradation - Fallback to fast path when headless fails
// P2-F1 Day 3: Import from external crates (no longer from riptide-core)
// Use trait abstraction instead of concrete type to avoid circular dependency
use riptide_types::extractors::HtmlParser;
use riptide_types::ExtractedDoc;
use riptide_types::{CircuitBreakerConfig, RetryConfig};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Reliability configuration for the entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    /// HTTP client retry configuration
    pub http_retry: RetryConfig,
    /// Circuit breaker configuration for headless service
    pub headless_circuit_breaker: CircuitBreakerConfig,
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
            http_retry: RetryConfig {
                max_attempts: 2, // 1 retry as per requirements
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(2),
                backoff_multiplier: 1.5,
                jitter: true,
            },
            headless_circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 3, // More lenient for headless service
                open_cooldown_ms: 60_000,
                half_open_max_in_flight: 2,
            },
            enable_graceful_degradation: true,
            headless_timeout: Duration::from_secs(3), // 3s hard cap
            fast_extraction_quality_threshold: 0.6,
        }
    }
}

impl ReliabilityConfig {
    /// Create ReliabilityConfig from environment variables
    pub fn from_env() -> Self {
        let max_retries = std::env::var("RELIABILITY_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let timeout_secs = std::env::var("RELIABILITY_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        let enable_graceful_degradation = std::env::var("RELIABILITY_GRACEFUL_DEGRADATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true);

        let quality_threshold = std::env::var("RELIABILITY_QUALITY_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.6);

        Self {
            http_retry: RetryConfig {
                max_attempts: max_retries,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(2),
                backoff_multiplier: 1.5,
                jitter: true,
            },
            headless_circuit_breaker: CircuitBreakerConfig {
                failure_threshold: 3,
                open_cooldown_ms: 60_000,
                half_open_max_in_flight: 2,
            },
            enable_graceful_degradation,
            headless_timeout: Duration::from_secs(timeout_secs),
            fast_extraction_quality_threshold: quality_threshold,
        }
    }
}

/// Enhanced extraction orchestrator with reliability patterns
#[derive(Debug)]
pub struct ReliableExtractor {
    http_client: ReliableHttpClient,
    headless_client: ReliableHttpClient,
    config: ReliabilityConfig,
    metrics: Option<std::sync::Arc<dyn ReliabilityMetricsRecorder>>,
}

impl ReliableExtractor {
    pub fn new(config: ReliabilityConfig) -> Result<Self> {
        // Create HTTP client for general requests
        let http_client = ReliableHttpClient::new(
            config.http_retry.clone(),
            CircuitBreakerConfig::default(), // Use default for general HTTP
        )?;

        // Create separate client for headless service with its own circuit breaker
        let headless_client = ReliableHttpClient::new(
            RetryConfig {
                max_attempts: 1, // No retries for headless - fast fail
                ..config.http_retry.clone()
            },
            config.headless_circuit_breaker.clone(),
        )?;

        Ok(Self {
            http_client,
            headless_client,
            config,
            metrics: None,
        })
    }

    /// Create ReliableExtractor with metrics support
    pub fn new_with_metrics(
        config: ReliabilityConfig,
        metrics: std::sync::Arc<dyn ReliabilityMetricsRecorder>,
    ) -> Result<Self> {
        let mut extractor = Self::new(config)?;
        extractor.metrics = Some(metrics);
        Ok(extractor)
    }

    /// Perform content extraction with full reliability patterns
    ///
    /// # Arguments
    /// * `url` - URL to extract content from
    /// * `extraction_mode` - Extraction strategy (Fast, Headless, ProbesFirst)
    /// * `wasm_extractor` - WASM-based extractor for untrusted HTML
    /// * `html_parser` - Native HTML parser for trusted headless HTML
    /// * `headless_url` - Optional headless service URL
    pub async fn extract_with_reliability(
        &self,
        url: &str,
        extraction_mode: ExtractionMode,
        wasm_extractor: &dyn WasmExtractor,
        html_parser: &dyn HtmlParser,
        headless_url: Option<&str>,
    ) -> Result<ExtractedDoc> {
        let start_time = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        info!(
            request_id = %request_id,
            url = %url,
            mode = ?extraction_mode,
            "Starting reliable extraction"
        );

        match extraction_mode {
            ExtractionMode::Fast => {
                self.extract_fast(url, wasm_extractor, html_parser, &request_id)
                    .await
            }
            ExtractionMode::Headless => {
                self.extract_headless(url, headless_url, html_parser, wasm_extractor, &request_id)
                    .await
            }
            ExtractionMode::ProbesFirst => {
                self.extract_with_probes(
                    url,
                    headless_url,
                    wasm_extractor,
                    html_parser,
                    &request_id,
                )
                .await
            }
        }
        .map_err(|e| {
            let duration = start_time.elapsed();
            error!(
                request_id = %request_id,
                duration_ms = duration.as_millis(),
                error = %e,
                "Reliable extraction failed"
            );
            e
        })
    }

    /// Fast extraction path (untrusted HTML - WASM primary, native fallback)
    async fn extract_fast(
        &self,
        url: &str,
        wasm_extractor: &dyn WasmExtractor,
        html_parser: &dyn HtmlParser,
        request_id: &str,
    ) -> Result<ExtractedDoc> {
        debug!(request_id = %request_id, "Using fast extraction path (WASM primary)");

        // Fetch HTML with retry logic
        let response = self.http_client.get_with_retry(url).await?;
        let raw_html = response.text().await?;

        let extraction_start = std::time::Instant::now();

        // PRIMARY: Try WASM extractor (secure for untrusted HTML)
        info!(
            path = "fast",
            parser = "wasm",
            request_id = %request_id,
            url = %url,
            "Primary parser selected for fast path"
        );

        let mut doc = wasm_extractor
            .extract(raw_html.as_bytes(), url, "article")
            .or_else(|wasm_err| {
                let wasm_duration = extraction_start.elapsed();
                warn!(
                    path = "fast",
                    primary_parser = "wasm",
                    fallback_parser = "native",
                    request_id = %request_id,
                    error = %wasm_err,
                    wasm_duration_ms = wasm_duration.as_millis(),
                    "Fallback triggered - WASM extractor failed, trying native parser"
                );

                // FALLBACK: Try native parser (via trait - breaks circular dependency)
                let fallback_start = std::time::Instant::now();
                let native_result = html_parser
                    .parse_html(&raw_html, url)
                    .map_err(|native_err| {
                        anyhow::anyhow!(
                            "Both parsers failed in fast path. WASM: {}, Native: {}",
                            wasm_err,
                            native_err
                        )
                    });

                if let Ok(ref doc) = native_result {
                    let fallback_duration = fallback_start.elapsed();
                    info!(
                        path = "fast",
                        parser = "native",
                        fallback_occurred = true,
                        request_id = %request_id,
                        fallback_duration_ms = fallback_duration.as_millis(),
                        content_length = doc.text.len(),
                        "Native parser fallback succeeded"
                    );
                }

                native_result
            })?;

        let extraction_duration = extraction_start.elapsed();

        // Add parser metadata for observability
        doc.parser_metadata = Some(riptide_types::ParserMetadata {
            parser_used: if doc.parser_metadata.is_some()
                && doc.parser_metadata.as_ref().unwrap().fallback_occurred
            {
                "native".to_string()
            } else {
                "wasm".to_string()
            },
            confidence_score: doc.quality_score.map(|q| q as f64 / 100.0).unwrap_or(0.8),
            fallback_occurred: doc.parser_metadata.is_some()
                && doc.parser_metadata.as_ref().unwrap().fallback_occurred,
            parse_time_ms: extraction_duration.as_millis() as u64,
            extraction_path: Some("fast".to_string()),
            primary_error: None,
        });

        info!(
            request_id = %request_id,
            content_length = doc.text.len(),
            parser_used = doc.parser_metadata.as_ref().map(|m| m.parser_used.as_str()).unwrap_or("unknown"),
            extraction_time_ms = extraction_duration.as_millis(),
            "Fast extraction completed"
        );

        Ok(doc)
    }

    /// Headless extraction with circuit breaker protection (trusted HTML - native primary, WASM fallback)
    async fn extract_headless(
        &self,
        url: &str,
        headless_url: Option<&str>,
        html_parser: &dyn HtmlParser,
        wasm_extractor: &dyn WasmExtractor,
        request_id: &str,
    ) -> Result<ExtractedDoc> {
        debug!(request_id = %request_id, "Using headless extraction path (native primary)");

        let headless_service_url =
            headless_url.ok_or_else(|| anyhow::anyhow!("Headless service URL not configured"))?;

        // Check circuit breaker state
        let cb_state = self.headless_client.get_circuit_breaker_state().await;
        debug!(request_id = %request_id, circuit_state = ?cb_state, "Circuit breaker state");

        // Prepare headless request
        let render_request = serde_json::json!({
            "url": url,
            "wait_for": null,
            "scroll_steps": 0
        });

        // Call headless service with timeout and circuit breaker
        let start_time = Instant::now();
        let response = tokio::time::timeout(
            self.config.headless_timeout,
            self.headless_client
                .post_with_retry(&format!("{}/render", headless_service_url), &render_request),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Headless service timeout"))?;

        let response = response.map_err(|e| {
            warn!(
                request_id = %request_id,
                error = %e,
                "Headless service request failed"
            );
            e
        })?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Headless service returned status: {}",
                response.status()
            ));
        }

        let rendered_html = response.text().await?;
        let duration = start_time.elapsed();

        debug!(
            request_id = %request_id,
            duration_ms = duration.as_millis(),
            html_size = rendered_html.len(),
            "Headless rendering completed"
        );

        let extraction_start = Instant::now();

        // PRIMARY: Try native parser (fast for trusted HTML - via trait)
        info!(
            path = "headless",
            parser = "native",
            request_id = %request_id,
            url = %url,
            "Primary parser selected for headless path"
        );

        let mut doc = html_parser
            .parse_html(&rendered_html, url)
            .or_else(|native_err| {
                let native_duration = extraction_start.elapsed();
                warn!(
                    path = "headless",
                    primary_parser = "native",
                    fallback_parser = "wasm",
                    request_id = %request_id,
                    error = %native_err,
                    native_duration_ms = native_duration.as_millis(),
                    "Fallback triggered - Native parser failed, trying WASM extractor"
                );

                // FALLBACK: Try WASM extractor (non-circular)
                let fallback_start = Instant::now();
                let wasm_result = wasm_extractor
                    .extract(rendered_html.as_bytes(), url, "article")
                    .map_err(|wasm_err| {
                        anyhow::anyhow!(
                            "Both parsers failed in headless path. Native: {}, WASM: {}",
                            native_err,
                            wasm_err
                        )
                    });

                if let Ok(ref doc) = wasm_result {
                    let fallback_duration = fallback_start.elapsed();
                    info!(
                        path = "headless",
                        parser = "wasm",
                        fallback_occurred = true,
                        request_id = %request_id,
                        fallback_duration_ms = fallback_duration.as_millis(),
                        content_length = doc.text.len(),
                        "WASM extractor fallback succeeded"
                    );
                }

                wasm_result
            })?;

        let extraction_duration = extraction_start.elapsed();

        // Add parser metadata for observability
        doc.parser_metadata = Some(riptide_types::ParserMetadata {
            parser_used: if doc.parser_metadata.is_some()
                && doc.parser_metadata.as_ref().unwrap().fallback_occurred
            {
                "wasm".to_string()
            } else {
                "native".to_string()
            },
            confidence_score: doc.quality_score.map(|q| q as f64 / 100.0).unwrap_or(0.85),
            fallback_occurred: doc.parser_metadata.is_some()
                && doc.parser_metadata.as_ref().unwrap().fallback_occurred,
            parse_time_ms: extraction_duration.as_millis() as u64,
            extraction_path: Some("headless".to_string()),
            primary_error: None,
        });

        info!(
            request_id = %request_id,
            content_length = doc.text.len(),
            quality_score = doc.quality_score,
            parser_used = doc.parser_metadata.as_ref().map(|m| m.parser_used.as_str()).unwrap_or("unknown"),
            extraction_time_ms = extraction_duration.as_millis(),
            "Headless extraction completed"
        );

        Ok(doc)
    }

    /// Probes-first extraction with graceful degradation
    async fn extract_with_probes(
        &self,
        url: &str,
        headless_url: Option<&str>,
        wasm_extractor: &dyn WasmExtractor,
        html_parser: &dyn HtmlParser,
        request_id: &str,
    ) -> Result<ExtractedDoc> {
        debug!(request_id = %request_id, "Using probes-first extraction");

        // First, try fast extraction
        match self
            .extract_fast(url, wasm_extractor, html_parser, request_id)
            .await
        {
            Ok(doc) => {
                // Evaluate quality of fast extraction
                let quality_score = self.evaluate_extraction_quality(&doc);
                debug!(
                    request_id = %request_id,
                    quality_score = quality_score,
                    threshold = self.config.fast_extraction_quality_threshold,
                    "Fast extraction quality evaluation"
                );

                if quality_score >= self.config.fast_extraction_quality_threshold {
                    info!(
                        request_id = %request_id,
                        quality_score = quality_score,
                        "Fast extraction quality acceptable, returning result"
                    );
                    return Ok(doc);
                }

                debug!(
                    request_id = %request_id,
                    "Fast extraction quality below threshold, trying headless"
                );

                // INJECTION POINT 3: Record fallback metrics
                if let Some(ref metrics) = self.metrics {
                    metrics.record_extraction_fallback(
                        "raw",
                        "headless",
                        "quality_threshold_not_met",
                    );
                }
            }
            Err(e) => {
                warn!(
                    request_id = %request_id,
                    error = %e,
                    "Fast extraction failed, trying headless"
                );

                // INJECTION POINT 3: Record fallback metrics
                if let Some(ref metrics) = self.metrics {
                    metrics.record_extraction_fallback("raw", "headless", "fast_extraction_failed");
                }
            }
        }

        // If fast extraction failed or quality is poor, try headless
        if self.config.enable_graceful_degradation {
            match self
                .extract_headless(url, headless_url, html_parser, wasm_extractor, request_id)
                .await
            {
                Ok(doc) => {
                    info!(
                        request_id = %request_id,
                        "Headless extraction succeeded after fast extraction issues"
                    );
                    Ok(doc)
                }
                Err(e) => {
                    warn!(
                        request_id = %request_id,
                        error = %e,
                        "Headless extraction also failed, attempting final fallback"
                    );

                    // Final fallback: try fast extraction one more time with minimal content
                    if let Ok(doc) = self
                        .extract_fast(url, wasm_extractor, html_parser, request_id)
                        .await
                    {
                        warn!(
                            request_id = %request_id,
                            "Returning low-quality fast extraction as final fallback"
                        );
                        return Ok(doc);
                    }

                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!(
                "Fast extraction quality poor and graceful degradation disabled"
            ))
        }
    }

    /// Evaluate extraction quality for decision making
    fn evaluate_extraction_quality(&self, doc: &ExtractedDoc) -> f32 {
        let mut score: f32 = 0.0;

        // Title presence (20%)
        if doc.title.as_ref().is_some_and(|t| !t.trim().is_empty()) {
            score += 0.2;
        }

        // Content length (40%)
        let text_length = doc.text.len();
        if text_length > 1000 {
            score += 0.4;
        } else if text_length > 200 {
            score += 0.2;
        }

        // Markdown structure (20%)
        let markdown_indicators = if let Some(ref markdown) = doc.markdown {
            markdown.matches('#').count()
                + markdown.matches('*').count()
                + markdown.matches('[').count()
        } else {
            0
        };
        if markdown_indicators > 5 {
            score += 0.2;
        } else if markdown_indicators > 2 {
            score += 0.1;
        }

        // Metadata presence (20%)
        let metadata_score = [
            doc.byline.is_some(),
            doc.published_iso.is_some(),
            doc.description.is_some(),
            !doc.links.is_empty(),
        ]
        .iter()
        .filter(|&&x| x)
        .count() as f32
            * 0.05;

        score += metadata_score;

        score.min(1.0_f32)
    }

    /// Get circuit breaker metrics for monitoring
    pub async fn get_reliability_metrics(&self) -> ReliabilityMetrics {
        let http_cb_state = self.http_client.get_circuit_breaker_state().await;
        let http_failures = self.http_client.get_circuit_breaker_failure_count();
        let headless_cb_state = self.headless_client.get_circuit_breaker_state().await;
        let headless_failures = self.headless_client.get_circuit_breaker_failure_count();

        ReliabilityMetrics {
            http_circuit_breaker_state: format!("{:?}", http_cb_state),
            http_failure_count: http_failures as u64,
            headless_circuit_breaker_state: format!("{:?}", headless_cb_state),
            headless_failure_count: headless_failures as u64,
            graceful_degradation_enabled: self.config.enable_graceful_degradation,
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

/// Trait for recording reliability metrics (abstraction for metrics integration)
pub trait ReliabilityMetricsRecorder: Send + Sync + std::fmt::Debug {
    /// Record extraction fallback event
    fn record_extraction_fallback(&self, from_mode: &str, to_mode: &str, reason: &str);
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::collections::HashMap;

    #[allow(dead_code)]
    struct MockWasmExtractor;

    impl WasmExtractor for MockWasmExtractor {
        fn extract(&self, html: &[u8], url: &str, _mode: &str) -> Result<ExtractedDoc> {
            let html_str = String::from_utf8_lossy(html);
            Ok(ExtractedDoc {
                url: url.to_string(),
                title: Some("Test Title".to_string()),
                byline: None,
                published_iso: None,
                markdown: Some("# Test Content".to_string()),
                text: if html_str.len() > 100 {
                    "Long content with good quality"
                } else {
                    "Short"
                }
                .to_string(),
                links: vec![],
                media: vec![],
                language: Some("en".to_string()),
                reading_time: Some(2),
                quality_score: Some(85),
                word_count: Some(200),
                categories: vec![],
                site_name: None,
                description: Some("Test description".to_string()),
                html: None,
                parser_metadata: None,
            })
        }
    }

    #[tokio::test]
    async fn test_extraction_quality_evaluation() {
        let extractor = ReliableExtractor::new(ReliabilityConfig::default()).unwrap();

        let high_quality_doc = ExtractedDoc {
            url: "https://test.com".to_string(),
            title: Some("Great Article Title".to_string()),
            text: "This is a long and comprehensive article with lots of valuable content that should score well on quality metrics.".repeat(20),
            markdown: Some("# Title
## Subtitle
*emphasis* and [links](url)".to_string()),
            byline: Some("Author Name".to_string()),
            description: Some("Article description".to_string()),
            ..Default::default()
        };

        let quality_score = extractor.evaluate_extraction_quality(&high_quality_doc);
        assert!(
            quality_score > 0.8,
            "High quality document should score > 0.8, got {}",
            quality_score
        );

        let low_quality_doc = ExtractedDoc {
            url: "https://test.com".to_string(),
            title: None,
            text: "Short".to_string(),
            markdown: Some("Short".to_string()),
            ..Default::default()
        };

        let quality_score = extractor.evaluate_extraction_quality(&low_quality_doc);
        assert!(
            quality_score < 0.3,
            "Low quality document should score < 0.3, got {}",
            quality_score
        );
    }
}
