//! Parser performance metrics for extraction strategies
//!
//! This module provides Prometheus metrics for tracking parser performance,
//! fallback behavior, and execution paths in the extraction system.

#[cfg(feature = "prometheus")]
use lazy_static::lazy_static;
#[cfg(feature = "prometheus")]
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};

#[cfg(feature = "prometheus")]
lazy_static! {
    /// Parser attempts counter
    /// Tracks total number of parser execution attempts by strategy and path
    pub static ref PARSER_ATTEMPTS: CounterVec = register_counter_vec!(
        "riptide_extraction_parser_attempts_total",
        "Total number of parser execution attempts by strategy",
        &["strategy", "path"]  // strategy=wasm/native/css, path=direct/headless
    )
    .expect("Failed to register PARSER_ATTEMPTS metric");

    /// Parser success/failure counter
    /// Tracks parser execution results by strategy and outcome
    pub static ref PARSER_RESULTS: CounterVec = register_counter_vec!(
        "riptide_extraction_parser_results_total",
        "Parser execution results by strategy and outcome",
        &["strategy", "path", "outcome"]  // outcome=success/fallback/error
    )
    .expect("Failed to register PARSER_RESULTS metric");

    /// Fallback events counter
    /// Tracks number of fallback events between strategies
    pub static ref PARSER_FALLBACKS: CounterVec = register_counter_vec!(
        "riptide_extraction_parser_fallbacks_total",
        "Number of fallback events",
        &["from_strategy", "to_strategy", "path"]
    )
    .expect("Failed to register PARSER_FALLBACKS metric");

    /// Parser execution duration histogram
    /// Measures parser execution time in seconds
    pub static ref PARSER_DURATION: HistogramVec = register_histogram_vec!(
        "riptide_extraction_parser_duration_seconds",
        "Parser execution duration in seconds",
        &["strategy", "path"],
        vec![0.001, 0.005, 0.010, 0.050, 0.100, 0.500, 1.0]  // 1ms to 1s buckets
    )
    .expect("Failed to register PARSER_DURATION metric");

    /// Confidence score histogram
    /// Tracks extraction confidence scores by strategy
    pub static ref PARSER_CONFIDENCE: HistogramVec = register_histogram_vec!(
        "riptide_extraction_confidence_score",
        "Extraction confidence scores",
        &["strategy"],
        vec![0.0, 0.3, 0.6, 0.85, 0.95, 1.0]  // Confidence thresholds
    )
    .expect("Failed to register PARSER_CONFIDENCE metric");
}

/// Parser strategy types
#[derive(Debug, Clone, Copy)]
pub enum ParserStrategy {
    /// WASM-based extraction
    Wasm,
    /// Native CSS selector extraction
    Native,
    /// CSS-only extraction
    Css,
    /// Headless browser extraction
    Headless,
}

impl ParserStrategy {
    /// Convert strategy to string label
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wasm => "wasm",
            Self::Native => "native",
            Self::Css => "css",
            Self::Headless => "headless",
        }
    }
}

/// Execution path types
#[derive(Debug, Clone, Copy)]
pub enum ExecutionPath {
    /// Direct execution without browser
    Direct,
    /// Execution through headless browser
    Headless,
}

impl ExecutionPath {
    /// Convert path to string label
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Headless => "headless",
        }
    }
}

/// Execution outcome types
#[derive(Debug, Clone, Copy)]
pub enum ExecutionOutcome {
    /// Successful extraction
    Success,
    /// Fallback to another strategy
    Fallback,
    /// Error occurred
    Error,
}

impl ExecutionOutcome {
    /// Convert outcome to string label
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Fallback => "fallback",
            Self::Error => "error",
        }
    }
}

/// Parser metrics recorder
///
/// Provides methods to record parser performance metrics
#[derive(Debug, Clone)]
pub struct ParserMetrics;

impl ParserMetrics {
    /// Record a parser attempt
    ///
    /// # Arguments
    /// * `strategy` - The parser strategy used
    /// * `path` - The execution path taken
    #[cfg(feature = "prometheus")]
    pub fn record_attempt(strategy: ParserStrategy, path: ExecutionPath) {
        PARSER_ATTEMPTS
            .with_label_values(&[strategy.as_str(), path.as_str()])
            .inc();
    }

    /// Record parser attempt (no-op without prometheus feature)
    #[cfg(not(feature = "prometheus"))]
    pub fn record_attempt(_strategy: ParserStrategy, _path: ExecutionPath) {}

    /// Record a parser result
    ///
    /// # Arguments
    /// * `strategy` - The parser strategy used
    /// * `path` - The execution path taken
    /// * `outcome` - The execution outcome
    #[cfg(feature = "prometheus")]
    pub fn record_result(strategy: ParserStrategy, path: ExecutionPath, outcome: ExecutionOutcome) {
        PARSER_RESULTS
            .with_label_values(&[strategy.as_str(), path.as_str(), outcome.as_str()])
            .inc();
    }

    /// Record parser result (no-op without prometheus feature)
    #[cfg(not(feature = "prometheus"))]
    pub fn record_result(
        _strategy: ParserStrategy,
        _path: ExecutionPath,
        _outcome: ExecutionOutcome,
    ) {
    }

    /// Record a fallback event
    ///
    /// # Arguments
    /// * `from_strategy` - The strategy that failed
    /// * `to_strategy` - The strategy fallen back to
    /// * `path` - The execution path
    #[cfg(feature = "prometheus")]
    pub fn record_fallback(
        from_strategy: ParserStrategy,
        to_strategy: ParserStrategy,
        path: ExecutionPath,
    ) {
        PARSER_FALLBACKS
            .with_label_values(&[from_strategy.as_str(), to_strategy.as_str(), path.as_str()])
            .inc();
    }

    /// Record fallback event (no-op without prometheus feature)
    #[cfg(not(feature = "prometheus"))]
    pub fn record_fallback(
        _from_strategy: ParserStrategy,
        _to_strategy: ParserStrategy,
        _path: ExecutionPath,
    ) {
    }

    /// Record parser execution duration
    ///
    /// # Arguments
    /// * `strategy` - The parser strategy used
    /// * `path` - The execution path taken
    /// * `duration_secs` - The duration in seconds
    #[cfg(feature = "prometheus")]
    pub fn record_duration(strategy: ParserStrategy, path: ExecutionPath, duration_secs: f64) {
        PARSER_DURATION
            .with_label_values(&[strategy.as_str(), path.as_str()])
            .observe(duration_secs);
    }

    /// Record parser duration (no-op without prometheus feature)
    #[cfg(not(feature = "prometheus"))]
    pub fn record_duration(_strategy: ParserStrategy, _path: ExecutionPath, _duration_secs: f64) {}

    /// Record confidence score
    ///
    /// # Arguments
    /// * `strategy` - The parser strategy used
    /// * `confidence` - The confidence score (0.0 to 1.0)
    #[cfg(feature = "prometheus")]
    pub fn record_confidence(strategy: ParserStrategy, confidence: f32) {
        PARSER_CONFIDENCE
            .with_label_values(&[strategy.as_str()])
            .observe(confidence as f64);
    }

    /// Record confidence score (no-op without prometheus feature)
    #[cfg(not(feature = "prometheus"))]
    pub fn record_confidence(_strategy: ParserStrategy, _confidence: f32) {}
}

/// Helper to record a complete extraction operation
///
/// # Arguments
/// * `strategy` - The parser strategy used
/// * `path` - The execution path taken
/// * `duration_secs` - The duration in seconds
/// * `outcome` - The execution outcome
/// * `confidence` - Optional confidence score
pub fn record_extraction(
    strategy: ParserStrategy,
    path: ExecutionPath,
    duration_secs: f64,
    outcome: ExecutionOutcome,
    confidence: Option<f32>,
) {
    ParserMetrics::record_attempt(strategy, path);
    ParserMetrics::record_result(strategy, path, outcome);
    ParserMetrics::record_duration(strategy, path, duration_secs);

    if let Some(conf) = confidence {
        ParserMetrics::record_confidence(strategy, conf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_str_conversion() {
        assert_eq!(ParserStrategy::Wasm.as_str(), "wasm");
        assert_eq!(ParserStrategy::Native.as_str(), "native");
        assert_eq!(ParserStrategy::Css.as_str(), "css");
        assert_eq!(ParserStrategy::Headless.as_str(), "headless");
    }

    #[test]
    fn test_path_str_conversion() {
        assert_eq!(ExecutionPath::Direct.as_str(), "direct");
        assert_eq!(ExecutionPath::Headless.as_str(), "headless");
    }

    #[test]
    fn test_outcome_str_conversion() {
        assert_eq!(ExecutionOutcome::Success.as_str(), "success");
        assert_eq!(ExecutionOutcome::Fallback.as_str(), "fallback");
        assert_eq!(ExecutionOutcome::Error.as_str(), "error");
    }

    #[test]
    fn test_record_metrics_no_panic() {
        // Test that recording metrics doesn't panic (even without prometheus feature)
        ParserMetrics::record_attempt(ParserStrategy::Wasm, ExecutionPath::Direct);
        ParserMetrics::record_result(
            ParserStrategy::Wasm,
            ExecutionPath::Direct,
            ExecutionOutcome::Success,
        );
        ParserMetrics::record_duration(ParserStrategy::Wasm, ExecutionPath::Direct, 0.5);
        ParserMetrics::record_confidence(ParserStrategy::Wasm, 0.95);
        ParserMetrics::record_fallback(
            ParserStrategy::Wasm,
            ParserStrategy::Native,
            ExecutionPath::Direct,
        );
    }

    #[test]
    fn test_record_extraction_helper() {
        // Test complete extraction recording
        record_extraction(
            ParserStrategy::Wasm,
            ExecutionPath::Direct,
            0.5,
            ExecutionOutcome::Success,
            Some(0.95),
        );

        record_extraction(
            ParserStrategy::Native,
            ExecutionPath::Headless,
            1.2,
            ExecutionOutcome::Fallback,
            None,
        );
    }
}
