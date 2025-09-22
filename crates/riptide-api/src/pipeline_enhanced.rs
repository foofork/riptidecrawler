use crate::metrics::{PhaseTimer, PhaseType, RipTideMetrics};
use crate::state::AppState;
use anyhow::Result;
use riptide_core::types::{CrawlOptions, ExtractedDoc};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Enhanced pipeline orchestrator with comprehensive phase timing and metrics
pub struct EnhancedPipelineOrchestrator {
    /// Application state
    state: AppState,

    /// Crawl options
    options: CrawlOptions,

    /// Metrics collector
    metrics: Arc<RipTideMetrics>,
}

impl EnhancedPipelineOrchestrator {
    /// Create new enhanced pipeline orchestrator
    pub fn new(state: AppState, options: CrawlOptions) -> Self {
        let metrics = state.metrics.clone();

        Self {
            state,
            options,
            metrics,
        }
    }

    /// Execute enhanced pipeline with detailed phase timing
    pub async fn execute_enhanced(&self, url: &str) -> Result<EnhancedPipelineResult> {
        let overall_start = Instant::now();

        info!(
            url = %url,
            cache_mode = %self.options.cache_mode,
            "Starting enhanced pipeline execution"
        );

        let mut result = EnhancedPipelineResult {
            url: url.to_string(),
            success: false,
            total_duration_ms: 0,
            phase_timings: PhaseTiming::default(),
            document: None,
            error: None,
            cache_hit: false,
            gate_decision: "unknown".to_string(),
            quality_score: 0.0,
        };

        // Phase 1: Fetch
        let fetch_result = self.execute_fetch_phase(url).await;
        result.phase_timings.fetch_ms = fetch_result.duration_ms;

        let (content, http_status) = match fetch_result.result {
            Ok(data) => data,
            Err(e) => {
                result.error = Some(format!("Fetch phase failed: {}", e));
                self.metrics.record_error(crate::metrics::ErrorType::Http);
                return Ok(result);
            }
        };

        // Phase 2: Gate Analysis
        let gate_result = self.execute_gate_phase(url, &content).await;
        result.phase_timings.gate_ms = gate_result.duration_ms;
        result.gate_decision = gate_result.decision.clone();
        result.quality_score = gate_result.quality_score;

        // Record gate decision in metrics
        self.metrics.record_gate_decision(&gate_result.decision);

        // Phase 3: WASM Extraction
        let wasm_result = self.execute_wasm_phase(url, &content, &gate_result.decision).await;
        result.phase_timings.wasm_ms = wasm_result.duration_ms;

        let document = match wasm_result.result {
            Ok(doc) => doc,
            Err(e) => {
                result.error = Some(format!("WASM phase failed: {}", e));
                self.metrics.record_error(crate::metrics::ErrorType::Wasm);
                return Ok(result);
            }
        };

        // Phase 4: Render (if headless required)
        if gate_result.decision == "headless" {
            let render_result = self.execute_render_phase(url).await;
            result.phase_timings.render_ms = Some(render_result.duration_ms);

            match render_result.result {
                Ok(enhanced_doc) => {
                    result.document = Some(enhanced_doc);
                }
                Err(e) => {
                    warn!("Render phase failed, using WASM result: {}", e);
                    result.document = Some(document);
                }
            }
        } else {
            result.document = Some(document);
        }

        result.success = true;
        result.total_duration_ms = overall_start.elapsed().as_millis() as u64;

        info!(
            url = %url,
            total_duration_ms = result.total_duration_ms,
            fetch_ms = result.phase_timings.fetch_ms,
            gate_ms = result.phase_timings.gate_ms,
            wasm_ms = result.phase_timings.wasm_ms,
            render_ms = ?result.phase_timings.render_ms,
            gate_decision = %result.gate_decision,
            quality_score = result.quality_score,
            "Enhanced pipeline execution completed"
        );

        Ok(result)
    }

    /// Execute fetch phase with timing
    async fn execute_fetch_phase(&self, url: &str) -> PhaseResult<(String, u16)> {
        let timer = PhaseTimer::start(PhaseType::Fetch, url.to_string());
        let start = Instant::now();

        let result = self.fetch_content(url).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Execute gate analysis phase with timing
    async fn execute_gate_phase(&self, url: &str, content: &str) -> GateResult {
        let timer = PhaseTimer::start(PhaseType::Gate, url.to_string());
        let start = Instant::now();

        let (decision, quality_score) = self.analyze_content_gate(url, content).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, true); // Gate analysis always succeeds

        debug!(
            url = %url,
            gate_decision = %decision,
            quality_score = quality_score,
            duration_ms = duration_ms,
            "Gate analysis completed"
        );

        GateResult {
            decision,
            quality_score,
            duration_ms,
        }
    }

    /// Execute WASM extraction phase with timing
    async fn execute_wasm_phase(&self, url: &str, content: &str, gate_decision: &str) -> PhaseResult<ExtractedDoc> {
        let timer = PhaseTimer::start(PhaseType::Wasm, url.to_string());
        let start = Instant::now();

        let result = self.extract_with_wasm(url, content, gate_decision).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Execute render phase with timing (for headless content)
    async fn execute_render_phase(&self, url: &str) -> PhaseResult<ExtractedDoc> {
        let timer = PhaseTimer::start(PhaseType::Render, url.to_string());
        let start = Instant::now();

        let result = self.render_with_headless(url).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        // Record timing in metrics
        timer.end(&self.metrics, result.is_ok());

        PhaseResult {
            result,
            duration_ms,
        }
    }

    /// Fetch content from URL
    async fn fetch_content(&self, url: &str) -> Result<(String, u16)> {
        // Simplified implementation - integrate with actual fetch logic
        let response = self.state.http_client.get(url).send().await?;
        let status = response.status().as_u16();
        let content = response.text().await?;
        Ok((content, status))
    }

    /// Analyze content for gate decision
    async fn analyze_content_gate(&self, _url: &str, content: &str) -> (String, f32) {
        // Simplified gate analysis - integrate with actual gate logic
        let content_length = content.len();
        let quality_score = if content_length > 10000 {
            0.8
        } else if content_length > 1000 {
            0.5
        } else {
            0.2
        };

        let decision = if quality_score >= self.state.config.gate_hi_threshold {
            "raw"
        } else if quality_score >= self.state.config.gate_lo_threshold {
            "probes_first"
        } else {
            "headless"
        };

        (decision.to_string(), quality_score)
    }

    /// Extract content using WASM
    async fn extract_with_wasm(&self, url: &str, content: &str, _gate_decision: &str) -> Result<ExtractedDoc> {
        // Simplified implementation - integrate with actual WASM extraction
        let doc = ExtractedDoc {
            url: url.to_string(),
            title: "Extracted Title".to_string(),
            content: content.chars().take(1000).collect(),
            links: vec![],
            images: vec![],
            metadata: std::collections::HashMap::new(),
            word_count: content.split_whitespace().count(),
            language: Some("en".to_string()),
            reading_time: Some(5),
        };

        Ok(doc)
    }

    /// Render content using headless browser
    async fn render_with_headless(&self, url: &str) -> Result<ExtractedDoc> {
        // Simplified implementation - integrate with actual headless rendering
        if self.state.config.headless_url.is_none() {
            return Err(anyhow::anyhow!("Headless service not configured"));
        }

        // For now, return a placeholder
        let doc = ExtractedDoc {
            url: url.to_string(),
            title: "Headless Rendered Title".to_string(),
            content: "Enhanced content from headless rendering".to_string(),
            links: vec![],
            images: vec![],
            metadata: std::collections::HashMap::new(),
            word_count: 100,
            language: Some("en".to_string()),
            reading_time: Some(2),
        };

        Ok(doc)
    }
}

/// Result from enhanced pipeline execution
#[derive(Debug, Clone)]
pub struct EnhancedPipelineResult {
    pub url: String,
    pub success: bool,
    pub total_duration_ms: u64,
    pub phase_timings: PhaseTiming,
    pub document: Option<ExtractedDoc>,
    pub error: Option<String>,
    pub cache_hit: bool,
    pub gate_decision: String,
    pub quality_score: f32,
}

/// Detailed phase timing information
#[derive(Debug, Clone, Default)]
pub struct PhaseTiming {
    pub fetch_ms: u64,
    pub gate_ms: u64,
    pub wasm_ms: u64,
    pub render_ms: Option<u64>,
}

/// Generic phase result with timing
#[derive(Debug)]
struct PhaseResult<T> {
    result: Result<T>,
    duration_ms: u64,
}

/// Gate analysis result
#[derive(Debug)]
struct GateResult {
    decision: String,
    quality_score: f32,
    duration_ms: u64,
}