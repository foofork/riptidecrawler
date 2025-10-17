/// Smart Engine Selection with Fallback Chain
///
/// This module implements intelligent extraction engine selection with automatic
/// fallback chains: raw → wasm → headless
///
/// Features:
/// - Content analysis heuristics for optimal engine selection
/// - Exponential backoff retry logic
/// - Performance metrics tracking
/// - Comprehensive logging of decision-making process
/// - Memory coordination for distributed agents
use crate::commands::extract::{ExtractArgs, ExtractResponse};
use crate::output;
use anyhow::Result;
use serde::Serialize;
use std::time::{Duration, Instant};

// Fallback chain configuration
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;
const MIN_CONTENT_LENGTH: usize = 100;
const MIN_TEXT_RATIO: f64 = 0.05;
const MIN_CONFIDENCE: f64 = 0.5;

/// Extraction engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EngineType {
    Raw,
    Wasm,
    Headless,
}

impl EngineType {
    pub fn name(&self) -> &'static str {
        match self {
            EngineType::Raw => "raw",
            EngineType::Wasm => "wasm",
            EngineType::Headless => "headless",
        }
    }
}

/// Extraction result quality metrics
#[derive(Debug, Clone, Serialize)]
pub struct ExtractionQuality {
    pub content_length: usize,
    pub text_ratio: f64,
    pub has_structure: bool,
    pub confidence_score: f64,
    pub extraction_time_ms: u64,
}

/// Engine attempt result for fallback chain
#[derive(Debug, Serialize)]
pub struct EngineAttempt {
    pub engine: EngineType,
    pub success: bool,
    pub quality: Option<ExtractionQuality>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Content analysis results for engine selection
#[derive(Debug, Serialize)]
pub struct ContentAnalysis {
    pub has_react: bool,
    pub has_vue: bool,
    pub has_angular: bool,
    pub has_spa_markers: bool,
    pub has_anti_scraping: bool,
    pub content_ratio: f64,
    pub has_main_content: bool,
    pub recommended_engine: EngineType,
}

/// Analyze content characteristics to determine optimal engine
pub fn analyze_content_for_engine(html: &str, url: &str) -> ContentAnalysis {
    output::print_info("🔍 Analyzing content for optimal engine selection...");

    // Check for heavy JavaScript frameworks
    let has_react = html.contains("__NEXT_DATA__")
        || html.contains("react")
        || html.contains("_reactRoot")
        || html.contains("__webpack_require__");

    let has_vue = html.contains("v-app") || html.contains("vue") || html.contains("createApp");

    let has_angular = html.contains("ng-app")
        || html.contains("ng-version")
        || html.contains("platformBrowserDynamic");

    // Check for SPA (Single Page Application) markers
    let has_spa_markers = html.contains("<!-- rendered by")
        || html.contains("__webpack")
        || html.contains("window.__INITIAL_STATE__")
        || html.contains("data-react-helmet");

    // Check for anti-scraping measures
    let has_anti_scraping = html.contains("Cloudflare")
        || html.contains("cf-browser-verification")
        || html.contains("grecaptcha")
        || html.contains("hCaptcha")
        || html.contains("PerimeterX");

    // Calculate content-to-markup ratio
    let content_ratio = calculate_content_ratio(html);

    // Analyze content structure
    let has_main_content = html.contains("<article")
        || html.contains("class=\"content\"")
        || html.contains("id=\"content\"")
        || html.contains("<main");

    // Determine recommended engine based on analysis
    let recommended_engine = if has_anti_scraping {
        output::print_info("⚠️  Anti-scraping detected - recommending Headless engine");
        EngineType::Headless
    } else if has_react || has_vue || has_angular || has_spa_markers {
        output::print_info("⚛️  JavaScript framework detected - recommending Headless engine");
        EngineType::Headless
    } else if content_ratio < 0.1 {
        output::print_info("📉 Low content ratio detected - recommending Headless engine");
        EngineType::Headless
    } else if html.contains("wasm") || url.contains(".wasm") {
        output::print_info("🦀 WASM content detected - recommending WASM engine");
        EngineType::Wasm
    } else if has_main_content && content_ratio > 0.2 {
        output::print_info("📄 Standard HTML with good content - recommending WASM engine");
        EngineType::Wasm
    } else {
        output::print_info("🚀 Using WASM engine for extraction");
        EngineType::Wasm
    };

    // Log detailed analysis
    output::print_info(&format!(
        "📊 Content Analysis Results:\n  \
         - React/Next.js: {}\n  \
         - Vue: {}\n  \
         - Angular: {}\n  \
         - SPA Markers: {}\n  \
         - Anti-Scraping: {}\n  \
         - Content Ratio: {:.2}%\n  \
         - Main Content: {}\n  \
         - Recommended Engine: {}",
        has_react,
        has_vue,
        has_angular,
        has_spa_markers,
        has_anti_scraping,
        content_ratio * 100.0,
        has_main_content,
        recommended_engine.name()
    ));

    ContentAnalysis {
        has_react,
        has_vue,
        has_angular,
        has_spa_markers,
        has_anti_scraping,
        content_ratio,
        has_main_content,
        recommended_engine,
    }
}

/// Calculate content-to-markup ratio (heuristic for client-side rendering)
fn calculate_content_ratio(html: &str) -> f64 {
    let total_len = html.len() as f64;
    if total_len == 0.0 {
        return 0.0;
    }

    // Count text content (rough estimate)
    let text_content: String = html
        .split('<')
        .filter_map(|s| s.split('>').nth(1))
        .collect();

    let content_len = text_content.trim().len() as f64;
    content_len / total_len
}

/// Validate extraction result quality
pub fn is_extraction_sufficient(result: &ExtractResponse) -> bool {
    let content_length = result.content.len();
    let confidence = result.confidence.unwrap_or(0.0);

    // Criteria for sufficient extraction
    let has_min_content = content_length >= MIN_CONTENT_LENGTH;
    let has_good_confidence = confidence >= MIN_CONFIDENCE;

    // Calculate text ratio
    let text_ratio = if content_length > 0 {
        result.content.split_whitespace().count() as f64 / content_length as f64
    } else {
        0.0
    };
    let has_good_text_ratio = text_ratio >= MIN_TEXT_RATIO;

    let is_sufficient = has_min_content && has_good_confidence && has_good_text_ratio;

    output::print_info(&format!(
        "✅ Extraction Quality Check:\n  \
         - Content Length: {} chars (min: {})\n  \
         - Confidence: {:.2}% (min: {:.0}%)\n  \
         - Text Ratio: {:.2}% (min: {:.0}%)\n  \
         - Sufficient: {}",
        content_length,
        MIN_CONTENT_LENGTH,
        confidence * 100.0,
        MIN_CONFIDENCE * 100.0,
        text_ratio * 100.0,
        MIN_TEXT_RATIO * 100.0,
        is_sufficient
    ));

    is_sufficient
}

/// Analyze extraction quality metrics
pub fn analyze_extraction_quality(result: &ExtractResponse) -> ExtractionQuality {
    let content_length = result.content.len();
    let text_ratio = if content_length > 0 {
        result.content.split_whitespace().count() as f64 / content_length as f64
    } else {
        0.0
    };

    let has_structure = result.metadata.is_some();
    let confidence_score = result.confidence.unwrap_or(0.0);
    let extraction_time_ms = result.extraction_time_ms.unwrap_or(0);

    ExtractionQuality {
        content_length,
        text_ratio,
        has_structure,
        confidence_score,
        extraction_time_ms,
    }
}

/// Format attempt summary for error reporting
pub fn format_attempt_summary(attempts: &[EngineAttempt]) -> String {
    let mut summary = String::new();

    for (i, attempt) in attempts.iter().enumerate() {
        summary.push_str(&format!(
            "\n  {}. {} - {} ({}ms)",
            i + 1,
            attempt.engine.name(),
            if attempt.success {
                "✅ Success"
            } else {
                "❌ Failed"
            },
            attempt.duration_ms
        ));

        if let Some(ref error) = attempt.error {
            summary.push_str(&format!("\n     Error: {}", error));
        }

        if let Some(ref quality) = attempt.quality {
            summary.push_str(&format!(
                "\n     Quality: {} chars, {:.1}% text, {:.1}% confidence",
                quality.content_length,
                quality.text_ratio * 100.0,
                quality.confidence_score * 100.0
            ));
        }
    }

    summary
}

/// Store extraction decision in memory for coordination
pub async fn store_extraction_decision(url: &str, decision: &str) -> Result<()> {
    use std::process::Command;

    let memory_data = serde_json::json!({
        "url": url,
        "decision": decision,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let _ = Command::new("npx")
        .args(&[
            "claude-flow@alpha",
            "hooks",
            "memory-store",
            "--key",
            &format!("swarm/engine-selection/{}", url.replace('/', "_")),
            "--value",
            &memory_data.to_string(),
        ])
        .output();

    Ok(())
}

/// Store extraction metrics in memory
pub async fn store_extraction_metrics(
    final_engine: &str,
    attempts: &[EngineAttempt],
    total_duration: Duration,
    url: Option<&str>,
) -> Result<()> {
    use std::process::Command;

    let metrics = serde_json::json!({
        "final_engine": final_engine,
        "total_duration_ms": total_duration.as_millis(),
        "attempts": attempts.len(),
        "attempt_details": attempts.iter().map(|a| {
            serde_json::json!({
                "engine": a.engine.name(),
                "success": a.success,
                "duration_ms": a.duration_ms,
            })
        }).collect::<Vec<_>>(),
        "url": url,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let _ = Command::new("npx")
        .args(&[
            "claude-flow@alpha",
            "hooks",
            "memory-store",
            "--key",
            "swarm/engine-selection/metrics",
            "--value",
            &metrics.to_string(),
        ])
        .output();

    // Also notify other agents about the metrics
    let _ = Command::new("npx")
        .args(&[
            "claude-flow@alpha",
            "hooks",
            "notify",
            "--message",
            &format!(
                "Extraction completed with {} engine in {}ms",
                final_engine,
                total_duration.as_millis()
            ),
        ])
        .output();

    Ok(())
}

/// Retry with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: u32,
    initial_backoff_ms: u64,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..max_retries {
        if attempt > 0 {
            let backoff = Duration::from_millis(initial_backoff_ms * 2u64.pow(attempt - 1));
            output::print_info(&format!(
                "🔄 Retry {}/{} after {:?}...",
                attempt + 1,
                max_retries,
                backoff
            ));
            tokio::time::sleep(backoff).await;
        }

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                output::print_warning(&format!("Attempt {} failed: {}", attempt + 1, e));
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Operation failed after all retries")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_ratio_calculation() {
        let html = "<html><body>Hello World</body></html>";
        let ratio = calculate_content_ratio(html);
        assert!(ratio > 0.0 && ratio < 1.0);
    }

    #[test]
    fn test_spa_detection() {
        let html = r#"<html><head><script>window.__INITIAL_STATE__={}</script></head></html>"#;
        let analysis = analyze_content_for_engine(html, "https://example.com");
        assert!(analysis.has_spa_markers);
        assert_eq!(analysis.recommended_engine, EngineType::Headless);
    }

    #[test]
    fn test_react_detection() {
        let html = r#"<html><head><script>window.__NEXT_DATA__={}</script></head></html>"#;
        let analysis = analyze_content_for_engine(html, "https://example.com");
        assert!(analysis.has_react);
        assert_eq!(analysis.recommended_engine, EngineType::Headless);
    }

    #[test]
    fn test_standard_html_detection() {
        let html = r#"<html><body><article>Hello World with good content ratio for extraction</article></body></html>"#;
        let analysis = analyze_content_for_engine(html, "https://example.com");
        assert!(analysis.has_main_content);
        assert_eq!(analysis.recommended_engine, EngineType::Wasm);
    }

    #[test]
    fn test_extraction_quality_validation() {
        let good_result = ExtractResponse {
            content: "This is a good extraction with sufficient content length and quality"
                .to_string(),
            confidence: Some(0.8),
            method_used: Some("wasm".to_string()),
            extraction_time_ms: Some(100),
            metadata: None,
        };

        assert!(is_extraction_sufficient(&good_result));

        let bad_result = ExtractResponse {
            content: "Short".to_string(),
            confidence: Some(0.2),
            method_used: Some("raw".to_string()),
            extraction_time_ms: Some(50),
            metadata: None,
        };

        assert!(!is_extraction_sufficient(&bad_result));
    }

    #[test]
    fn test_quality_analysis() {
        let result = ExtractResponse {
            content: "Test content with reasonable length".to_string(),
            confidence: Some(0.85),
            method_used: Some("wasm".to_string()),
            extraction_time_ms: Some(150),
            metadata: Some(serde_json::json!({"title": "Test"})),
        };

        let quality = analyze_extraction_quality(&result);
        assert!(quality.content_length > 0);
        assert!(quality.text_ratio > 0.0);
        assert!(quality.has_structure);
        assert_eq!(quality.confidence_score, 0.85);
        assert_eq!(quality.extraction_time_ms, 150);
    }
}
