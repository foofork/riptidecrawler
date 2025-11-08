//! Strategies facade for extraction strategy selection.

use crate::error::RiptideResult;
use crate::RiptideError;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::info;

#[derive(Clone)]
pub struct StrategiesFacade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub url: String,
    pub force_strategy: Option<String>,
    pub enable_probe: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResponse {
    pub recommended_strategy: String,
    pub confidence_score: f32,
    pub reasoning: String,
    pub alternatives: Vec<AlternativeStrategy>,
    pub processing_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeStrategy {
    pub strategy: String,
    pub score: f32,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

impl StrategiesFacade {
    pub fn new() -> Self {
        Self
    }

    pub async fn select_strategy(
        &self,
        request: StrategyRequest,
    ) -> RiptideResult<StrategyResponse> {
        let start_time = Instant::now();
        info!(url = %request.url, "Selecting extraction strategy");

        if request.url.is_empty() {
            return Err(RiptideError::validation("URL cannot be empty"));
        }

        // If force_strategy is provided, validate and use it
        if let Some(ref forced) = request.force_strategy {
            if !self.is_valid_strategy(forced) {
                return Err(RiptideError::Validation(format!(
                    "Invalid strategy '{}'. Supported: wasm, headless, fetch",
                    forced
                )));
            }

            return Ok(StrategyResponse {
                recommended_strategy: forced.clone(),
                confidence_score: 1.0,
                reasoning: "Strategy forced by user request".to_string(),
                alternatives: vec![],
                processing_time_ms: start_time.elapsed().as_millis(),
            });
        }

        // Analyze URL and select strategy
        let (strategy, confidence, reasoning, alternatives) = self
            .analyze_url(&request.url, request.enable_probe.unwrap_or(false))
            .await?;

        Ok(StrategyResponse {
            recommended_strategy: strategy,
            confidence_score: confidence,
            reasoning,
            alternatives,
            processing_time_ms: start_time.elapsed().as_millis(),
        })
    }

    async fn analyze_url(
        &self,
        url: &str,
        _enable_probe: bool,
    ) -> RiptideResult<(String, f32, String, Vec<AlternativeStrategy>)> {
        // Simple heuristics for strategy selection
        let url_lower = url.to_lowercase();

        if url_lower.contains("pdf") || url_lower.ends_with(".pdf") {
            return Ok((
                "fetch".to_string(),
                0.95,
                "PDF file detected, using fetch strategy".to_string(),
                vec![],
            ));
        }

        if url_lower.contains("javascript")
            || url_lower.contains("react")
            || url_lower.contains("vue")
            || url_lower.contains("angular")
        {
            return Ok((
                "headless".to_string(),
                0.85,
                "JavaScript-heavy site detected, using headless browser".to_string(),
                vec![AlternativeStrategy {
                    strategy: "wasm".to_string(),
                    score: 0.6,
                    pros: vec!["Faster".to_string(), "Lower resource usage".to_string()],
                    cons: vec!["May miss dynamic content".to_string()],
                }],
            ));
        }

        // Default to WASM for better performance
        Ok((
            "wasm".to_string(),
            0.8,
            "Standard web content, using WASM extraction for optimal performance".to_string(),
            vec![
                AlternativeStrategy {
                    strategy: "fetch".to_string(),
                    score: 0.7,
                    pros: vec!["Simpler".to_string(), "Reliable".to_string()],
                    cons: vec!["Slower".to_string()],
                },
                AlternativeStrategy {
                    strategy: "headless".to_string(),
                    score: 0.6,
                    pros: vec!["Handles JS".to_string()],
                    cons: vec!["Higher resource usage".to_string(), "Slower".to_string()],
                },
            ],
        ))
    }

    fn is_valid_strategy(&self, strategy: &str) -> bool {
        matches!(strategy, "wasm" | "headless" | "fetch")
    }
}

impl Default for StrategiesFacade {
    fn default() -> Self {
        Self::new()
    }
}
