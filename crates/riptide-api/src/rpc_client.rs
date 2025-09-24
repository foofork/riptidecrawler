use anyhow::{anyhow, Result};
use reqwest::Client;
use riptide_core::dynamic::{DynamicConfig, DynamicRenderResult, PageAction, RenderArtifacts};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// RPC v2 client for communicating with headless browser service
#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl RpcClient {
    /// Create a new RPC client with default configuration
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:9123".to_string(),
            timeout: Duration::from_secs(3), // Hard cap as per requirements
        }
    }

    /// Create a new RPC client with custom base URL
    pub fn with_url(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            timeout: Duration::from_secs(3),
        }
    }

    /// Render a page with dynamic configuration
    pub async fn render_dynamic(
        &self,
        url: &str,
        config: &DynamicConfig,
        stealth_config: Option<&riptide_core::stealth::StealthConfig>,
    ) -> Result<DynamicRenderResult> {
        let start_time = Instant::now();

        info!(url = %url, "Starting dynamic render via RPC v2");

        // Convert dynamic config to headless browser format
        let request = HeadlessRenderRequest {
            url: url.to_string(),
            session_id: None, // TODO: Implement session persistence
            actions: Some(convert_actions(&config.actions)),
            timeouts: Some(HeadlessTimeouts {
                nav_ms: Some(1000),
                idle_after_dcl_ms: Some(1000),
                hard_cap_ms: Some(3000), // Hard cap requirement
            }),
            artifacts: Some(HeadlessArtifacts {
                screenshot: config.capture_artifacts,
                mhtml: config.capture_artifacts,
            }),
            stealth_config: stealth_config.cloned(),
        };

        debug!(
            url = %url,
            actions_count = request.actions.as_ref().map(|a| a.len()).unwrap_or(0),
            "Sending render request to headless service"
        );

        // Make HTTP request to headless service with timeout
        let response = tokio::time::timeout(
            self.timeout,
            self.client
                .post(&format!("{}/render", self.base_url))
                .json(&request)
                .send(),
        )
        .await
        .map_err(|_| anyhow!("RPC request timed out after {}s", self.timeout.as_secs()))?
        .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Headless service returned error: {}",
                response.status()
            ));
        }

        let headless_response: HeadlessRenderResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        let render_time_ms = start_time.elapsed().as_millis() as u64;

        // Convert back to DynamicRenderResult
        let result = DynamicRenderResult {
            success: true,
            html: headless_response.html,
            artifacts: convert_artifacts(headless_response.artifacts),
            error: None,
            render_time_ms,
            actions_executed: extract_action_names(&config.actions),
            wait_conditions_met: vec!["dom_content_loaded".to_string()],
        };

        info!(
            url = %url,
            render_time_ms = render_time_ms,
            html_size = result.html.len(),
            "Dynamic render completed successfully"
        );

        Ok(result)
    }

    /// Health check for the headless service
    pub async fn health_check(&self) -> Result<()> {
        let response = tokio::time::timeout(
            Duration::from_secs(2),
            self.client.get(&format!("{}/health", self.base_url)).send(),
        )
        .await
        .map_err(|_| anyhow!("Health check timed out"))?
        .map_err(|e| anyhow!("Health check request failed: {}", e))?;

        if response.status().is_success() {
            debug!("Headless service health check passed");
            Ok(())
        } else {
            Err(anyhow!(
                "Headless service health check failed: {}",
                response.status()
            ))
        }
    }
}

/// Request format for headless browser service
#[derive(Debug, Serialize)]
struct HeadlessRenderRequest {
    url: String,
    session_id: Option<String>,
    actions: Option<Vec<HeadlessPageAction>>,
    timeouts: Option<HeadlessTimeouts>,
    artifacts: Option<HeadlessArtifacts>,
    stealth_config: Option<riptide_core::stealth::StealthConfig>,
}

/// Timeout configuration for headless browser
#[derive(Debug, Serialize)]
struct HeadlessTimeouts {
    nav_ms: Option<u64>,
    idle_after_dcl_ms: Option<u64>,
    hard_cap_ms: Option<u64>,
}

/// Artifacts configuration for headless browser
#[derive(Debug, Serialize)]
struct HeadlessArtifacts {
    screenshot: bool,
    mhtml: bool,
}

/// Page action format for headless browser
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HeadlessPageAction {
    WaitForCss {
        css: String,
        timeout_ms: Option<u64>,
    },
    WaitForJs {
        expr: String,
        timeout_ms: Option<u64>,
    },
    Scroll {
        steps: u32,
        step_px: u32,
        delay_ms: u64,
    },
    Js {
        code: String,
    },
    Click {
        css: String,
    },
    Type {
        css: String,
        text: String,
        delay_ms: Option<u64>,
    },
}

/// Response format from headless browser service
#[derive(Debug, Deserialize)]
struct HeadlessRenderResponse {
    final_url: String,
    html: String,
    session_id: Option<String>,
    artifacts: HeadlessArtifactsOut,
}

/// Output artifacts from headless browser
#[derive(Debug, Deserialize)]
struct HeadlessArtifactsOut {
    screenshot_b64: Option<String>,
    mhtml_b64: Option<String>,
}

/// Convert riptide-core PageActions to headless browser format
fn convert_actions(actions: &[PageAction]) -> Vec<HeadlessPageAction> {
    actions
        .iter()
        .filter_map(|action| match action {
            PageAction::Click { selector, .. } => Some(HeadlessPageAction::Click {
                css: selector.clone(),
            }),
            PageAction::Type {
                selector,
                text,
                wait_after,
                ..
            } => Some(HeadlessPageAction::Type {
                css: selector.clone(),
                text: text.clone(),
                delay_ms: wait_after.map(|d| d.as_millis() as u64),
            }),
            PageAction::Evaluate { script, .. } => Some(HeadlessPageAction::Js {
                code: script.clone(),
            }),
            PageAction::Wait(wait_condition) => {
                // Convert wait conditions to appropriate headless actions
                match wait_condition {
                    riptide_core::dynamic::WaitCondition::Selector { selector, timeout } => {
                        Some(HeadlessPageAction::WaitForCss {
                            css: selector.clone(),
                            timeout_ms: Some(timeout.as_millis() as u64),
                        })
                    }
                    riptide_core::dynamic::WaitCondition::Javascript { script, timeout } => {
                        Some(HeadlessPageAction::WaitForJs {
                            expr: script.clone(),
                            timeout_ms: Some(timeout.as_millis() as u64),
                        })
                    }
                    // For other wait conditions, we'll implement them as JavaScript checks
                    _ => None,
                }
            }
            // For now, we'll skip actions that don't have direct equivalents
            _ => {
                warn!("Skipping unsupported action: {:?}", action);
                None
            }
        })
        .collect()
}

/// Convert headless artifacts to riptide-core format
fn convert_artifacts(artifacts: HeadlessArtifactsOut) -> Option<RenderArtifacts> {
    if artifacts.screenshot_b64.is_none() && artifacts.mhtml_b64.is_none() {
        return None;
    }

    Some(RenderArtifacts {
        screenshot: artifacts.screenshot_b64,
        mhtml: artifacts.mhtml_b64,
        metadata: riptide_core::dynamic::PageMetadata {
            title: None,
            description: None,
            og_tags: std::collections::HashMap::new(),
            twitter_tags: std::collections::HashMap::new(),
            json_ld: Vec::new(),
            final_url: String::new(), // Will be populated from response
            headers: std::collections::HashMap::new(),
            timing: None,
        },
        console_logs: Vec::new(),
        network_activity: Vec::new(),
    })
}

/// Extract action names for tracking
fn extract_action_names(actions: &[PageAction]) -> Vec<String> {
    actions
        .iter()
        .map(|action| match action {
            PageAction::Click { .. } => "click".to_string(),
            PageAction::Type { .. } => "type".to_string(),
            PageAction::Evaluate { .. } => "evaluate".to_string(),
            PageAction::Wait(_) => "wait".to_string(),
            PageAction::Navigate { .. } => "navigate".to_string(),
            PageAction::SetCookies { .. } => "set_cookies".to_string(),
            PageAction::Hover { .. } => "hover".to_string(),
            PageAction::Screenshot { .. } => "screenshot".to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_core::dynamic::{PageAction, WaitCondition};
    use std::time::Duration;

    #[test]
    fn test_convert_actions() {
        let actions = vec![
            PageAction::Click {
                selector: ".button".to_string(),
                wait_after: None,
            },
            PageAction::Type {
                selector: "#input".to_string(),
                text: "test".to_string(),
                clear_first: true,
                wait_after: Some(Duration::from_millis(500)),
            },
            PageAction::Wait(WaitCondition::Selector {
                selector: ".content".to_string(),
                timeout: Duration::from_secs(5),
            }),
        ];

        let converted = convert_actions(&actions);
        assert_eq!(converted.len(), 3);

        match &converted[0] {
            HeadlessPageAction::Click { css } => assert_eq!(css, ".button"),
            _ => panic!("Expected Click action"),
        }

        match &converted[1] {
            HeadlessPageAction::Type {
                css,
                text,
                delay_ms,
            } => {
                assert_eq!(css, "#input");
                assert_eq!(text, "test");
                assert_eq!(*delay_ms, Some(500));
            }
            _ => panic!("Expected Type action"),
        }

        match &converted[2] {
            HeadlessPageAction::WaitForCss { css, timeout_ms } => {
                assert_eq!(css, ".content");
                assert_eq!(*timeout_ms, Some(5000));
            }
            _ => panic!("Expected WaitForCss action"),
        }
    }

    #[test]
    fn test_extract_action_names() {
        let actions = vec![
            PageAction::Click {
                selector: ".button".to_string(),
                wait_after: None,
            },
            PageAction::Type {
                selector: "#input".to_string(),
                text: "test".to_string(),
                clear_first: true,
                wait_after: None,
            },
        ];

        let names = extract_action_names(&actions);
        assert_eq!(names, vec!["click", "type"]);
    }

    #[tokio::test]
    async fn test_rpc_client_creation() {
        let client = RpcClient::new();
        assert_eq!(client.base_url, "http://localhost:9123");
        assert_eq!(client.timeout, Duration::from_secs(3));
    }
}
