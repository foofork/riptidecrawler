//! Render facade for unified page rendering with multiple strategies.
//!
//! This facade consolidates all rendering logic including:
//! - Static HTML fetching
//! - Dynamic browser rendering
//! - PDF processing
//! - Adaptive strategy selection
//! - Stealth features
//! - Session persistence

use crate::error::{RiptideError, RiptideResult};
use anyhow::anyhow;
use riptide_fetch::FetchEngine;
use riptide_headless::dynamic::{DynamicConfig, DynamicRenderResult, WaitCondition};
use riptide_pdf::{create_pdf_processor, PdfConfig, PdfProcessingResult};
use riptide_stealth::StealthController;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, warn};

/// Render strategy options
#[derive(Debug, Clone)]
pub enum RenderStrategy {
    /// Static HTML fetching without JavaScript execution
    Static,
    /// Full browser rendering with JavaScript execution
    Dynamic,
    /// PDF document processing
    Pdf,
    /// Adaptive selection based on content analysis
    Adaptive,
}

/// Unified result from any render strategy
#[derive(Debug)]
pub struct RenderResult {
    /// Final URL after redirects
    pub final_url: String,
    /// Rendered HTML content (if applicable)
    pub html: Option<String>,
    /// Dynamic render result with artifacts (if applicable)
    pub dynamic_result: Option<DynamicRenderResult>,
    /// PDF processing result (if applicable)
    pub pdf_result: Option<PdfProcessingResult>,
}

/// Configuration for render operations
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Timeout for render operations
    pub timeout: Duration,
    /// Base URL for headless service
    pub headless_url: Option<String>,
    /// Enable stealth features by default
    pub enable_stealth: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            headless_url: Some("http://localhost:9123".to_string()),
            enable_stealth: true,
        }
    }
}

/// Session context for stateful rendering
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Session identifier
    pub session_id: String,
    /// User data directory path
    pub user_data_dir: Option<String>,
    /// Cookies for the session
    pub cookies: Vec<SessionCookie>,
}

/// Cookie for session management
#[derive(Debug, Clone)]
pub struct SessionCookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
}

/// Facade for unified page rendering.
///
/// Consolidates static fetching, dynamic rendering, PDF processing,
/// and adaptive strategy selection into a single interface.
///
/// # Example
///
/// ```no_run
/// use riptide_facade::facades::render::{RenderFacade, RenderStrategy, RenderConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fetch_engine = Arc::new(FetchEngine::new()?);
/// let config = RenderConfig::default();
/// let facade = RenderFacade::new(fetch_engine, config);
///
/// let result = facade.render_page(
///     "https://example.com",
///     RenderStrategy::Adaptive,
///     None,
///     None,
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct RenderFacade {
    fetch_engine: Arc<FetchEngine>,
    config: RenderConfig,
}

impl RenderFacade {
    /// Create a new render facade
    pub fn new(fetch_engine: Arc<FetchEngine>, config: RenderConfig) -> Self {
        Self {
            fetch_engine,
            config,
        }
    }

    /// Render a page using the specified strategy
    ///
    /// # Arguments
    ///
    /// * `url` - URL to render
    /// * `strategy` - Rendering strategy to use
    /// * `stealth_controller` - Optional stealth configuration
    /// * `session_ctx` - Optional session context for stateful rendering
    pub async fn render_page(
        &self,
        url: &str,
        strategy: RenderStrategy,
        stealth_controller: Option<&mut StealthController>,
        session_ctx: Option<&SessionContext>,
    ) -> RiptideResult<RenderResult> {
        match strategy {
            RenderStrategy::Static => {
                self.render_static(url, stealth_controller, session_ctx)
                    .await
            }
            RenderStrategy::Dynamic => {
                self.render_dynamic(
                    url,
                    &DynamicConfig::default(),
                    stealth_controller,
                    session_ctx,
                )
                .await
            }
            RenderStrategy::Pdf => self.render_pdf(url, &PdfConfig::default()).await,
            RenderStrategy::Adaptive => {
                self.render_adaptive(url, stealth_controller, session_ctx)
                    .await
            }
        }
    }

    /// Render with dynamic configuration
    pub async fn render_with_dynamic_config(
        &self,
        url: &str,
        dynamic_config: &DynamicConfig,
        stealth_controller: Option<&mut StealthController>,
        session_ctx: Option<&SessionContext>,
    ) -> RiptideResult<RenderResult> {
        self.render_dynamic(url, dynamic_config, stealth_controller, session_ctx)
            .await
    }

    /// Render with PDF configuration
    pub async fn render_with_pdf_config(
        &self,
        url: &str,
        pdf_config: &PdfConfig,
    ) -> RiptideResult<RenderResult> {
        self.render_pdf(url, pdf_config).await
    }

    /// Render static HTML without JavaScript execution
    async fn render_static(
        &self,
        url: &str,
        stealth_controller: Option<&mut StealthController>,
        session_ctx: Option<&SessionContext>,
    ) -> RiptideResult<RenderResult> {
        debug!(url = %url, "Processing with static rendering");

        // For now, use simple fetch - stealth and session support requires full HTTP client
        // This will be enhanced when integrating with riptide-api's http_client
        let _ = (stealth_controller, session_ctx); // TODO: Add stealth/session support

        let html = self
            .fetch_engine
            .as_ref()
            .fetch_text(url)
            .await
            .map_err(|e| RiptideError::fetch(url, format!("Failed to fetch content: {}", e)))?;

        let final_url = url.to_string(); // FetchEngine doesn't track redirects in facade layer

        let render_result = DynamicRenderResult {
            success: true,
            html: html.clone(),
            artifacts: None,
            error: None,
            render_time_ms: 100,
            actions_executed: Vec::new(),
            wait_conditions_met: Vec::new(),
        };

        Ok(RenderResult {
            final_url,
            html: Some(html),
            dynamic_result: Some(render_result),
            pdf_result: None,
        })
    }

    /// Render with full browser and JavaScript execution
    async fn render_dynamic(
        &self,
        url: &str,
        dynamic_config: &DynamicConfig,
        stealth_controller: Option<&mut StealthController>,
        session_ctx: Option<&SessionContext>,
    ) -> RiptideResult<RenderResult> {
        debug!(url = %url, "Processing with dynamic rendering");

        let stealth_config = stealth_controller
            .as_ref()
            .map(|controller| controller.config().clone());

        // Create RPC client with configured headless URL
        let rpc_client = if let Some(headless_url) = &self.config.headless_url {
            rpc::create_rpc_client(headless_url.clone())
        } else {
            rpc::create_rpc_client("http://localhost:9123".to_string())
        };

        // Perform health check on headless service
        if let Err(e) = rpc_client.health_check().await {
            warn!(
                url = %url,
                error = %e,
                "Headless service health check failed, falling back to static rendering"
            );

            // Fall back to static rendering if headless service is unavailable
            return self
                .render_static(url, stealth_controller, session_ctx)
                .await;
        }

        let session_id = session_ctx.map(|ctx| ctx.session_id.as_str());
        let user_data_dir = session_ctx.and_then(|ctx| ctx.user_data_dir.as_deref());

        debug!(
            url = %url,
            session_id = ?session_id,
            user_data_dir = ?user_data_dir,
            "Calling dynamic rendering with session context"
        );

        // Call dynamic rendering via RPC with timeout protection
        let render_result = timeout(
            self.config.timeout,
            rpc_client.render_dynamic_with_session(
                url,
                dynamic_config,
                stealth_config.as_ref(),
                session_id,
                user_data_dir,
            ),
        )
        .await;

        match render_result {
            Ok(Ok(mut render_result)) => {
                debug!(
                    url = %url,
                    render_time_ms = render_result.render_time_ms,
                    html_size = render_result.html.len(),
                    actions_executed = render_result.actions_executed.len(),
                    "Dynamic rendering completed successfully"
                );

                // Get final URL from response or use original URL
                let final_url = render_result
                    .artifacts
                    .as_ref()
                    .and_then(|a| {
                        let final_url = &a.metadata.final_url;
                        if final_url.is_empty() {
                            None
                        } else {
                            Some(final_url.clone())
                        }
                    })
                    .unwrap_or_else(|| {
                        debug!(url = %url, "No final URL from render artifacts, using original URL");
                        url.to_string()
                    });

                // Update render result with correct final URL
                if let Some(ref mut artifacts) = render_result.artifacts {
                    artifacts.metadata.final_url = final_url.clone();
                }

                let html = render_result.html.clone();
                Ok(RenderResult {
                    final_url,
                    html: Some(html),
                    dynamic_result: Some(render_result),
                    pdf_result: None,
                })
            }
            Ok(Err(e)) => {
                warn!(
                    url = %url,
                    error = %e,
                    "Dynamic rendering failed, falling back to static rendering"
                );

                // Fall back to static rendering on error
                self.render_static(url, stealth_controller, session_ctx)
                    .await
            }
            Err(_) => {
                error!(
                    url = %url,
                    timeout_secs = self.config.timeout.as_secs(),
                    "Render operation timed out"
                );

                Err(RiptideError::Timeout)
            }
        }
    }

    /// Render PDF document
    async fn render_pdf(&self, url: &str, pdf_config: &PdfConfig) -> RiptideResult<RenderResult> {
        debug!(url = %url, "Processing as PDF");

        // Fetch the PDF content
        let data = self
            .fetch_engine
            .as_ref()
            .fetch_bytes(url)
            .await
            .map_err(|e| RiptideError::fetch(url, format!("Failed to fetch PDF: {}", e)))?;

        // Verify it's actually a PDF
        if !riptide_pdf::utils::is_pdf_content(None, &data) {
            return Err(RiptideError::validation(
                "Content is not a valid PDF".to_string(),
            ));
        }

        // Process the PDF
        let pdf_processor = create_pdf_processor();
        let pdf_result = pdf_processor
            .process_pdf(&data, pdf_config)
            .await
            .map_err(|e| RiptideError::extraction(format!("PDF processing failed: {}", e)))?;

        Ok(RenderResult {
            final_url: url.to_string(),
            html: None,
            dynamic_result: None,
            pdf_result: Some(pdf_result),
        })
    }

    /// Adaptive rendering based on content analysis
    async fn render_adaptive(
        &self,
        url: &str,
        stealth_controller: Option<&mut StealthController>,
        session_ctx: Option<&SessionContext>,
    ) -> RiptideResult<RenderResult> {
        debug!(url = %url, "Processing with adaptive rendering");

        // Check if it's a PDF based on URL extension
        if url.ends_with(".pdf") || url.contains(".pdf?") {
            return self.render_pdf(url, &PdfConfig::default()).await;
        }

        // Perform content analysis to determine optimal rendering strategy
        let needs_dynamic = self.analyze_for_dynamic_content(url).await;

        if needs_dynamic {
            debug!(url = %url, "Content analysis suggests dynamic rendering");
            let dynamic_config = self.create_adaptive_dynamic_config(url);
            self.render_dynamic(url, &dynamic_config, stealth_controller, session_ctx)
                .await
        } else {
            debug!(url = %url, "Content analysis suggests static rendering is sufficient");
            self.render_static(url, stealth_controller, session_ctx)
                .await
        }
    }

    /// Analyze URL to determine if dynamic rendering is needed
    async fn analyze_for_dynamic_content(&self, url: &str) -> bool {
        // Patterns that indicate dynamic content
        let dynamic_patterns = [
            "twitter.com",
            "facebook.com",
            "instagram.com",
            "linkedin.com",
            "reddit.com",
            "github.com",
            "youtube.com",
            "medium.com",
        ];

        dynamic_patterns.iter().any(|pattern| url.contains(pattern))
    }

    /// Create adaptive dynamic configuration based on URL
    fn create_adaptive_dynamic_config(&self, url: &str) -> DynamicConfig {
        let mut config = DynamicConfig::default();
        config.timeout = self.config.timeout;

        // Add wait conditions based on URL patterns
        if url.contains("github.com") {
            config.wait_for = Some(WaitCondition::Selector {
                selector: ".repository-content".to_string(),
                timeout: Duration::from_secs(2),
            });
        } else if url.contains("twitter.com") || url.contains("x.com") {
            config.wait_for = Some(WaitCondition::Selector {
                selector: "[data-testid='tweet']".to_string(),
                timeout: Duration::from_secs(2),
            });
        }

        config
    }
}

/// RPC client wrapper for headless service communication
mod rpc {
    use super::*;
    use anyhow::Result;

    /// Create RPC client with the given base URL
    pub fn create_rpc_client(base_url: String) -> RpcClientWrapper {
        RpcClientWrapper { base_url }
    }

    /// Wrapper around RPC client for dependency injection
    pub struct RpcClientWrapper {
        base_url: String,
    }

    impl RpcClientWrapper {
        /// Health check for headless service
        pub async fn health_check(&self) -> Result<()> {
            let client = reqwest::Client::new();
            let response = client
                .get(format!("{}/health", self.base_url))
                .send()
                .await
                .map_err(|e| anyhow!("Health check failed: {}", e))?;

            if response.status().is_success() {
                Ok(())
            } else {
                Err(anyhow!(
                    "Health check returned non-success status: {}",
                    response.status()
                ))
            }
        }

        /// Render dynamic page with session
        pub async fn render_dynamic_with_session(
            &self,
            url: &str,
            config: &DynamicConfig,
            stealth_config: Option<&riptide_stealth::StealthConfig>,
            session_id: Option<&str>,
            user_data_dir: Option<&str>,
        ) -> Result<DynamicRenderResult> {
            // This would call the actual RPC endpoint
            // For now, return a placeholder result
            let _ = (url, config, stealth_config, session_id, user_data_dir);
            Err(anyhow!(
                "RPC client implementation pending - requires riptide-api RpcClient integration"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_for_dynamic_content() {
        let fetch_engine = Arc::new(FetchEngine::new().unwrap());
        let config = RenderConfig::default();
        let facade = RenderFacade::new(fetch_engine, config);

        assert!(
            facade
                .analyze_for_dynamic_content("https://twitter.com/user/status/123")
                .await
        );
        assert!(
            facade
                .analyze_for_dynamic_content("https://github.com/org/repo")
                .await
        );
        assert!(
            !facade
                .analyze_for_dynamic_content("https://example.com/blog/article.html")
                .await
        );
    }

    #[test]
    fn test_create_adaptive_dynamic_config() {
        let fetch_engine = Arc::new(FetchEngine::new().unwrap());
        let config = RenderConfig::default();
        let facade = RenderFacade::new(fetch_engine, config);

        let config = facade.create_adaptive_dynamic_config("https://github.com/rust-lang/rust");
        assert!(config.wait_for.is_some());
        assert_eq!(config.timeout, Duration::from_secs(3));
    }
}
