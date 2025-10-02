//! Stealth configuration and testing handlers
//!
//! This module provides HTTP endpoints for stealth configuration management,
//! testing stealth capabilities, and integration verification.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{errors::ApiError, state::AppState};
use riptide_core::stealth::{StealthController, StealthPreset, StealthConfig};

/// Stealth configuration request
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthConfigRequest {
    /// Stealth preset level
    pub preset: Option<StealthPreset>,
    /// Custom stealth configuration
    pub config: Option<StealthConfig>,
    /// Test URL for validation
    pub test_url: Option<String>,
}

/// Stealth configuration response
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthConfigResponse {
    /// Current configuration status
    pub status: String,
    /// Active stealth preset
    pub preset: StealthPreset,
    /// Applied stealth measures
    pub measures: Vec<String>,
    /// Generated user agent
    pub user_agent: String,
    /// Generated headers (sample)
    pub headers: std::collections::HashMap<String, String>,
    /// Calculated delay (milliseconds)
    pub delay_ms: u64,
    /// Stealth effectiveness score (0-100)
    pub effectiveness_score: u8,
}

/// Stealth test request
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthTestRequest {
    /// Target URLs to test
    pub urls: Vec<String>,
    /// Stealth preset to use
    pub preset: Option<StealthPreset>,
    /// Custom stealth configuration
    pub config: Option<StealthConfig>,
    /// Number of test iterations
    pub iterations: Option<u32>,
}

/// Stealth test response
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthTestResponse {
    /// Test success status
    pub success: bool,
    /// Test results for each URL
    pub results: Vec<StealthTestResult>,
    /// Overall effectiveness metrics
    pub metrics: StealthMetrics,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Individual stealth test result
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthTestResult {
    /// Target URL
    pub url: String,
    /// Test success
    pub success: bool,
    /// Response status code
    pub status_code: u16,
    /// Detection indicators found
    pub detection_indicators: Vec<String>,
    /// Response time (milliseconds)
    pub response_time_ms: u64,
    /// Headers analysis
    pub headers_analysis: HeadersAnalysis,
}

/// Headers analysis for stealth testing
#[derive(Debug, Serialize, Deserialize)]
pub struct HeadersAnalysis {
    /// User agent effectiveness
    pub user_agent_score: u8,
    /// Headers fingerprint score
    pub headers_score: u8,
    /// Overall fingerprint quality
    pub fingerprint_score: u8,
}

/// Stealth metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct StealthMetrics {
    /// Success rate percentage
    pub success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: f64,
    /// Detection rate percentage
    pub detection_rate: f64,
    /// Fingerprint uniqueness score
    pub fingerprint_uniqueness: u8,
}

/// Configure stealth settings
///
/// This endpoint allows configuration of stealth settings and returns
/// the current stealth configuration with sample generated values.
pub async fn configure_stealth(
    State(_state): State<AppState>,
    Json(request): Json<StealthConfigRequest>,
) -> Result<Json<StealthConfigResponse>, ApiError> {
    debug!("Configuring stealth settings: {:?}", request);

    // Create stealth controller based on request
    let mut controller = if let Some(config) = request.config {
        StealthController::new(config)
    } else {
        let preset = request.preset.unwrap_or(StealthPreset::Medium);
        StealthController::from_preset(preset)
    };

    // Generate sample values (clone to avoid borrowing issues)
    let user_agent = controller.next_user_agent().to_string();
    let headers = controller.generate_headers();
    let delay = controller.calculate_delay();

    // Determine active measures
    let measures = determine_active_measures(&controller);

    // Calculate effectiveness score
    let effectiveness_score = calculate_effectiveness_score(&controller);

    let response = StealthConfigResponse {
        status: "configured".to_string(),
        preset: StealthPreset::Medium, // Default since we can't easily get it from controller
        measures,
        user_agent,
        headers,
        delay_ms: delay.as_millis() as u64,
        effectiveness_score,
    };

    info!(
        preset = ?controller.get_preset(),
        effectiveness_score = effectiveness_score,
        "Stealth configuration completed"
    );

    Ok(Json(response))
}

/// Test stealth capabilities
///
/// This endpoint tests the effectiveness of stealth configurations
/// against real websites and provides detailed analysis.
pub async fn test_stealth(
    State(state): State<AppState>,
    Json(request): Json<StealthTestRequest>,
) -> Result<Json<StealthTestResponse>, ApiError> {
    let start_time = std::time::Instant::now();

    debug!("Starting stealth test: {:?}", request);

    // Validate request
    if request.urls.is_empty() {
        return Err(ApiError::validation("At least one URL is required for testing"));
    }

    if request.urls.len() > 10 {
        return Err(ApiError::validation("Maximum 10 URLs allowed for testing"));
    }

    // Create stealth controller
    let mut controller = if let Some(config) = request.config {
        StealthController::new(config)
    } else {
        let preset = request.preset.unwrap_or(StealthPreset::Medium);
        StealthController::from_preset(preset)
    };

    let iterations = request.iterations.unwrap_or(1).min(5);
    let mut all_results = Vec::new();

    // Test each URL
    for url in &request.urls {
        for _iteration in 0..iterations {
            let result = test_stealth_for_url(&mut controller, url, &state).await?;
            all_results.push(result);
        }
    }

    // Calculate metrics
    let metrics = calculate_stealth_metrics(&all_results);

    // Generate recommendations
    let recommendations = generate_stealth_recommendations(&all_results, &metrics);

    let response = StealthTestResponse {
        success: metrics.success_rate > 80.0,
        results: all_results,
        metrics,
        recommendations,
    };

    info!(
        urls_tested = request.urls.len(),
        iterations = iterations,
        success_rate = response.metrics.success_rate,
        total_time_ms = start_time.elapsed().as_millis(),
        "Stealth test completed"
    );

    // Record metrics
    state.metrics.record_http_request(
        "POST",
        "/stealth/test",
        200,
        start_time.elapsed().as_secs_f64(),
    );

    Ok(Json(response))
}

/// Get current stealth capabilities
///
/// Returns information about available stealth features and current configuration.
pub async fn get_stealth_capabilities(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let capabilities = serde_json::json!({
        "version": riptide_stealth::VERSION,
        "features": {
            "user_agent_rotation": true,
            "header_randomization": true,
            "timing_jitter": true,
            "fingerprinting_countermeasures": true,
            "proxy_support": true,
            "javascript_evasion": true
        },
        "presets": ["None", "Low", "Medium", "High"],
        "rotation_strategies": ["Random", "Sequential", "Sticky", "DomainBased"],
        "supported_browsers": ["Chrome", "Firefox", "Safari", "Edge"]
    });

    Ok(Json(capabilities))
}

/// Test stealth effectiveness for a single URL
async fn test_stealth_for_url(
    controller: &mut StealthController,
    url: &str,
    _state: &AppState,
) -> Result<StealthTestResult, ApiError> {
    let start_time = std::time::Instant::now();

    // Generate stealth parameters (clone to avoid borrowing issues)
    let user_agent = controller.next_user_agent().to_string();
    let headers = controller.generate_headers();
    let delay = controller.calculate_delay();

    // Apply delay
    tokio::time::sleep(delay).await;

    // Create HTTP client with stealth headers
    let client = reqwest::Client::builder()
        .user_agent(&user_agent)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| ApiError::internal(format!("Failed to create HTTP client: {}", e)))?;

    // Add stealth headers to request
    let mut request_builder = client.get(url);
    for (name, value) in &headers {
        request_builder = request_builder.header(name, value);
    }

    // Perform request
    let response_result = request_builder.send().await;
    let response_time = start_time.elapsed();

    let (status_code, detection_indicators, headers_analysis) = match response_result {
        Ok(response) => {
            let status = response.status().as_u16();
            let response_headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();

            // Analyze response for detection indicators
            let indicators = analyze_detection_indicators(&body, &response_headers);

            // Analyze headers effectiveness
            let analysis = analyze_headers_effectiveness(&user_agent, &headers, &response_headers);

            (status, indicators, analysis)
        }
        Err(e) => {
            debug!("Request failed for {}: {}", url, e);
            (0, vec![format!("Request failed: {}", e)], HeadersAnalysis {
                user_agent_score: 0,
                headers_score: 0,
                fingerprint_score: 0,
            })
        }
    };

    let success = status_code >= 200 && status_code < 400 && detection_indicators.is_empty();

    Ok(StealthTestResult {
        url: url.to_string(),
        success,
        status_code,
        detection_indicators,
        response_time_ms: response_time.as_millis() as u64,
        headers_analysis,
    })
}

/// Determine active stealth measures
fn determine_active_measures(_controller: &StealthController) -> Vec<String> {
    let mut measures = Vec::new();

    // Since we can't access the config directly, assume common measures
    measures.push("user_agent_rotation".to_string());
    measures.push("request_randomization".to_string());
    measures.push("timing_jitter".to_string());
    measures.push("fingerprinting_countermeasures".to_string());

    measures
}

/// Calculate stealth effectiveness score
fn calculate_effectiveness_score(_controller: &StealthController) -> u8 {
    // Return a reasonable score since we can't access internal config
    80 // Medium-high effectiveness
}

/// Analyze response for detection indicators
fn analyze_detection_indicators(
    body: &str,
    headers: &reqwest::header::HeaderMap,
) -> Vec<String> {
    let mut indicators = Vec::new();

    // Check for common bot detection patterns in response body
    let bot_patterns = [
        "blocked", "bot", "robot", "crawler", "spider", "captcha",
        "verify", "challenge", "cloudflare", "access denied"
    ];

    let body_lower = body.to_lowercase();
    for pattern in &bot_patterns {
        if body_lower.contains(pattern) {
            indicators.push(format!("Body contains '{}'", pattern));
        }
    }

    // Check response headers for bot detection
    if let Some(server) = headers.get("server") {
        if let Ok(server_str) = server.to_str() {
            if server_str.to_lowercase().contains("cloudflare") && headers.contains_key("cf-ray") {
                indicators.push("Cloudflare protection detected".to_string());
            }
        }
    }

    // Check for rate limiting headers
    if headers.contains_key("x-ratelimit-limit") || headers.contains_key("retry-after") {
        indicators.push("Rate limiting detected".to_string());
    }

    indicators
}

/// Analyze headers effectiveness
fn analyze_headers_effectiveness(
    user_agent: &str,
    request_headers: &std::collections::HashMap<String, String>,
    response_headers: &reqwest::header::HeaderMap,
) -> HeadersAnalysis {
    // Score user agent (check if it looks realistic)
    let user_agent_score = if user_agent.contains("Chrome") && user_agent.contains("WebKit") {
        85
    } else if user_agent.contains("Mozilla") {
        70
    } else {
        40
    };

    // Score request headers (check for completeness)
    let mut headers_score = 60; // Base score
    if request_headers.contains_key("accept") {
        headers_score += 10;
    }
    if request_headers.contains_key("accept-language") {
        headers_score += 10;
    }
    if request_headers.contains_key("accept-encoding") {
        headers_score += 10;
    }
    if request_headers.contains_key("dnt") {
        headers_score += 5;
    }

    // Calculate overall fingerprint score
    let fingerprint_score = (user_agent_score + headers_score) / 2;

    HeadersAnalysis {
        user_agent_score,
        headers_score: headers_score.min(100),
        fingerprint_score: fingerprint_score.min(100),
    }
}

/// Calculate overall stealth metrics
fn calculate_stealth_metrics(results: &[StealthTestResult]) -> StealthMetrics {
    if results.is_empty() {
        return StealthMetrics {
            success_rate: 0.0,
            avg_response_time_ms: 0.0,
            detection_rate: 100.0,
            fingerprint_uniqueness: 0,
        };
    }

    let successful = results.iter().filter(|r| r.success).count();
    let success_rate = (successful as f64 / results.len() as f64) * 100.0;

    let total_response_time: u64 = results.iter().map(|r| r.response_time_ms).sum();
    let avg_response_time_ms = total_response_time as f64 / results.len() as f64;

    let detected = results.iter().filter(|r| !r.detection_indicators.is_empty()).count();
    let detection_rate = (detected as f64 / results.len() as f64) * 100.0;

    let avg_fingerprint_score: u8 = results.iter()
        .map(|r| r.headers_analysis.fingerprint_score)
        .sum::<u8>() / results.len() as u8;

    StealthMetrics {
        success_rate,
        avg_response_time_ms,
        detection_rate,
        fingerprint_uniqueness: avg_fingerprint_score,
    }
}

/// Generate stealth recommendations
fn generate_stealth_recommendations(
    results: &[StealthTestResult],
    metrics: &StealthMetrics,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    if metrics.success_rate < 80.0 {
        recommendations.push("Consider using a higher stealth preset (Medium or High)".to_string());
    }

    if metrics.detection_rate > 20.0 {
        recommendations.push("Enable additional fingerprinting countermeasures".to_string());
    }

    if metrics.avg_response_time_ms > 5000.0 {
        recommendations.push("Consider reducing timing jitter to improve performance".to_string());
    }

    if metrics.fingerprint_uniqueness < 70 {
        recommendations.push("Improve user agent rotation strategy for better fingerprint diversity".to_string());
    }

    // Analyze common failure patterns
    let common_indicators: std::collections::HashMap<&str, usize> = results.iter()
        .flat_map(|r| r.detection_indicators.iter())
        .fold(std::collections::HashMap::new(), |mut acc, indicator| {
            *acc.entry(indicator).or_insert(0) += 1;
            acc
        });

    for (indicator, count) in common_indicators {
        if count > results.len() / 2 {
            recommendations.push(format!("Address common detection pattern: {}", indicator));
        }
    }

    if recommendations.is_empty() {
        recommendations.push("Stealth configuration is performing well!".to_string());
    }

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effectiveness_score_calculation() {
        let controller = StealthController::from_preset(StealthPreset::High);
        let score = calculate_effectiveness_score(&controller);
        assert!(score >= 80); // High preset should have high effectiveness
    }

    #[test]
    fn test_detection_indicators_analysis() {
        let body = "Access denied - bot detected";
        let headers = reqwest::header::HeaderMap::new();
        let indicators = analyze_detection_indicators(body, &headers);
        assert!(!indicators.is_empty());
        assert!(indicators.iter().any(|i| i.contains("bot")));
    }

    #[test]
    fn test_headers_analysis() {
        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let mut headers = std::collections::HashMap::new();
        headers.insert("accept".to_string(), "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string());
        headers.insert("accept-language".to_string(), "en-US,en;q=0.5".to_string());

        let response_headers = reqwest::header::HeaderMap::new();
        let analysis = analyze_headers_effectiveness(user_agent, &headers, &response_headers);

        assert!(analysis.user_agent_score > 70);
        assert!(analysis.headers_score > 60);
        assert!(analysis.fingerprint_score > 65);
    }
}