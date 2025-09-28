use crate::errors::{ApiError, ApiResult};
use crate::strategies_pipeline::{StrategiesPipelineOrchestrator, StrategiesPipelineResult};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};
use riptide_core::{
    strategies::{StrategyConfig, ExtractionStrategy},
    types::CrawlOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Request for strategies-based crawling
#[derive(Debug, Deserialize)]
pub struct StrategiesCrawlRequest {
    /// URL to crawl
    pub url: String,

    /// Optional extraction strategy override
    pub extraction_strategy: Option<String>,

    /// Optional chunking configuration
    pub chunking_config: Option<ChunkingConfigRequest>,

    /// Enable performance metrics collection
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Enable schema validation
    #[serde(default = "default_true")]
    pub validate_schema: bool,

    /// Cache mode
    #[serde(default = "default_cache_mode")]
    pub cache_mode: String,

    /// Custom CSS selectors for CSS_JSON strategy
    pub css_selectors: Option<HashMap<String, String>>,

    /// Custom regex patterns for REGEX strategy
    pub regex_patterns: Option<Vec<RegexPatternRequest>>,

    /// LLM configuration for LLM strategy
    pub llm_config: Option<LlmConfigRequest>,
}

/// Chunking configuration from request
#[derive(Debug, Deserialize)]
pub struct ChunkingConfigRequest {
    pub mode: String, // "sliding", "fixed", "sentence", "topic", "regex"
    pub token_max: Option<usize>,
    pub overlap: Option<usize>,
    pub preserve_sentences: Option<bool>,
    pub deterministic: Option<bool>,

    // Mode-specific parameters
    pub fixed_size: Option<usize>,
    pub fixed_by_tokens: Option<bool>,
    pub max_sentences: Option<usize>,
    pub similarity_threshold: Option<f64>,
    pub regex_pattern: Option<String>,
    pub min_chunk_size: Option<usize>,
}

/// Regex pattern from request
#[derive(Debug, Deserialize)]
pub struct RegexPatternRequest {
    pub name: String,
    pub pattern: String,
    pub field: String,
    pub required: bool,
}

/// LLM configuration from request
#[derive(Debug, Deserialize)]
pub struct LlmConfigRequest {
    pub enabled: bool,
    pub model: Option<String>,
    pub prompt_template: Option<String>,
}

/// Response for strategies crawling
#[derive(Debug, Serialize)]
pub struct StrategiesCrawlResponse {
    /// Success indicator
    pub success: bool,

    /// The processed content with extraction and chunking
    pub result: StrategiesPipelineResult,

    /// Processing statistics
    pub stats: ProcessingStats,
}

/// Processing statistics
#[derive(Debug, Serialize)]
pub struct ProcessingStats {
    pub chunks_created: usize,
    pub total_processing_time_ms: u64,
    pub extraction_strategy_used: String,
    pub chunking_mode_used: String,
    pub cache_hit: bool,
    pub quality_score: f32,
}

/// Query parameters for strategies crawling
#[derive(Debug, Deserialize)]
pub struct StrategiesQueryParams {
    /// Strategy mode: "auto", "trek", "css_json", "regex", "llm"
    #[serde(default = "default_auto")]
    pub strategy: String,

    /// Chunking mode: "sliding", "fixed", "sentence", "topic", "regex"
    #[serde(default = "default_sliding")]
    pub chunking: String,
}

/// Crawl with strategies pipeline
pub async fn strategies_crawl(
    State(state): State<AppState>,
    Query(params): Query<StrategiesQueryParams>,
    Json(request): Json<StrategiesCrawlRequest>,
) -> ApiResult<Json<StrategiesCrawlResponse>> {
    info!(
        url = %request.url,
        strategy = %params.strategy,
        chunking = %params.chunking,
        "Processing strategies crawl request"
    );

    // Validate URL
    let url = request.url.trim();
    if url.is_empty() {
        return Err(ApiError::invalid_request("URL cannot be empty"));
    }

    // Build strategy configuration
    let strategy_config = build_strategy_config(&request, &params)?;

    // Build crawl options
    let crawl_options = CrawlOptions {
        cache_mode: request.cache_mode.clone(),
        ..Default::default()
    };

    // Create strategies pipeline orchestrator
    let orchestrator = if params.strategy == "auto" {
        StrategiesPipelineOrchestrator::with_auto_strategy(state, crawl_options, url)
    } else {
        StrategiesPipelineOrchestrator::new(state, crawl_options, Some(strategy_config))
    };

    // Execute pipeline
    let result = orchestrator.execute_single(url).await?;

    // Build response statistics
    let stats = ProcessingStats {
        chunks_created: 1, // Core only produces one extraction result
        total_processing_time_ms: result.processing_time_ms,
        extraction_strategy_used: format!("{:?}", result.strategy_config.extraction),
        chunking_mode_used: "None".to_string(), // Chunking moved to riptide-html
        cache_hit: result.from_cache,
        quality_score: result.quality_score,
    };

    debug!(
        url = %request.url,
        chunks_created = stats.chunks_created,
        processing_time_ms = stats.total_processing_time_ms,
        strategy_used = %stats.extraction_strategy_used,
        "Strategies crawl completed successfully"
    );

    Ok(Json(StrategiesCrawlResponse {
        success: true,
        result,
        stats,
    }))
}

/// Get available strategies and their capabilities
pub async fn get_strategies_info() -> ApiResult<Json<StrategiesInfo>> {
    let info = StrategiesInfo {
        extraction_strategies: vec![
            StrategyInfo {
                name: "trek".to_string(),
                description: "Default WASM-based extraction (fastest)".to_string(),
                parameters: vec![],
            },
            StrategyInfo {
                name: "css_json".to_string(),
                description: "CSS selector to JSON extraction".to_string(),
                parameters: vec![
                    StrategyParameter {
                        name: "selectors".to_string(),
                        required: false,
                        description: "CSS selectors mapping (field -> selector)".to_string(),
                    },
                ],
            },
            StrategyInfo {
                name: "regex".to_string(),
                description: "Regex pattern extraction".to_string(),
                parameters: vec![
                    StrategyParameter {
                        name: "patterns".to_string(),
                        required: true,
                        description: "List of regex patterns to apply".to_string(),
                    },
                ],
            },
            StrategyInfo {
                name: "llm".to_string(),
                description: "LLM-based extraction (hook-based, disabled by default)".to_string(),
                parameters: vec![
                    StrategyParameter {
                        name: "enabled".to_string(),
                        required: true,
                        description: "Enable LLM extraction".to_string(),
                    },
                    StrategyParameter {
                        name: "model".to_string(),
                        required: false,
                        description: "LLM model to use".to_string(),
                    },
                ],
            },
        ],
        chunking_modes: vec![
            ChunkingModeInfo {
                name: "sliding".to_string(),
                description: "Sliding windows with overlap (default)".to_string(),
                parameters: vec!["token_max", "overlap", "preserve_sentences"],
            },
            ChunkingModeInfo {
                name: "fixed".to_string(),
                description: "Fixed character/token count".to_string(),
                parameters: vec!["size", "by_tokens"],
            },
            ChunkingModeInfo {
                name: "sentence".to_string(),
                description: "Split by sentence boundaries (NLP)".to_string(),
                parameters: vec!["max_sentences"],
            },
            ChunkingModeInfo {
                name: "topic".to_string(),
                description: "Split by semantic topics".to_string(),
                parameters: vec!["similarity_threshold"],
            },
            ChunkingModeInfo {
                name: "regex".to_string(),
                description: "Split by regex pattern".to_string(),
                parameters: vec!["pattern", "min_chunk_size"],
            },
        ],
    };

    Ok(Json(info))
}

/// Information about available strategies
#[derive(Debug, Serialize)]
pub struct StrategiesInfo {
    pub extraction_strategies: Vec<StrategyInfo>,
    pub chunking_modes: Vec<ChunkingModeInfo>,
}

/// Information about a specific extraction strategy
#[derive(Debug, Serialize)]
pub struct StrategyInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<StrategyParameter>,
}

/// Strategy parameter information
#[derive(Debug, Serialize)]
pub struct StrategyParameter {
    pub name: String,
    pub required: bool,
    pub description: String,
}

/// Information about a chunking mode
#[derive(Debug, Serialize)]
pub struct ChunkingModeInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<&'static str>,
}

/// Build strategy configuration from request parameters
fn build_strategy_config(
    request: &StrategiesCrawlRequest,
    params: &StrategiesQueryParams,
) -> ApiResult<StrategyConfig> {
    // Core only supports Trek extraction strategy
    let extraction = ExtractionStrategy::Trek;

    Ok(StrategyConfig {
        extraction,
        enable_metrics: true,
        validate_schema: true,
    })
}

// Chunking configuration functions removed since chunking is now handled by riptide-html

// Default values for serde
fn default_true() -> bool {
    true
}

fn default_cache_mode() -> String {
    "default".to_string()
}

fn default_auto() -> String {
    "auto".to_string()
}

fn default_sliding() -> String {
    "sliding".to_string()
}