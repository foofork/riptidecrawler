//! Browser facade for headless browser automation.
//!
//! Provides a high-level API for browser automation, including:
//! - Browser instance management with pooling
//! - CDP (Chrome DevTools Protocol) integration
//! - Navigation and content extraction
//! - Screenshot capture
//! - JavaScript execution
//! - Browser actions (click, type, wait, scroll)
//! - Cookie and storage management
//! - Stealth features for anti-detection

use crate::{config::RiptideConfig, error::RiptideResult, RiptideError};
use riptide_engine::{HeadlessLauncher, LaunchSession};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

/// Browser facade providing simplified headless browser automation.
///
/// This facade integrates multiple Riptide components:
/// - `riptide-engine`: CDP protocol and browser pool management
/// - `riptide-headless`: Browser lifecycle and session management
/// - `riptide-stealth`: Anti-detection features (optional)
///
/// # Example
///
/// ```no_run
/// use riptide_facade::{BrowserFacade, RiptideConfig, ScreenshotOptions};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RiptideConfig::default();
/// let facade = BrowserFacade::new(config).await?;
///
/// // Launch browser and navigate
/// let session = facade.launch().await?;
/// facade.navigate(&session, "https://example.com").await?;
///
/// // Take a screenshot
/// let screenshot = facade.screenshot(&session, ScreenshotOptions::default()).await?;
///
/// // Clean up
/// facade.close(session).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct BrowserFacade {
    config: Arc<RiptideConfig>,
    launcher: Arc<HeadlessLauncher>,
}

/// A managed browser session with automatic resource cleanup.
///
/// The session holds a reference to the underlying browser instance
/// and will automatically return it to the pool when dropped.
pub struct BrowserSession<'a> {
    session: LaunchSession<'a>,
}

/// Options for taking screenshots.
///
/// # Example
///
/// ```
/// use riptide_facade::ScreenshotOptions;
///
/// let options = ScreenshotOptions::default()
///     .full_page(true)
///     .with_viewport(1920, 1080);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    /// Capture the full page (scrollable area) instead of just the viewport
    pub full_page: bool,

    /// Viewport width in pixels
    pub width: Option<u32>,

    /// Viewport height in pixels
    pub height: Option<u32>,

    /// Image quality (0-100, only applies to JPEG)
    pub quality: Option<u8>,

    /// Image format (PNG or JPEG)
    pub format: ImageFormat,
}

/// Image format for screenshots.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageFormat {
    /// PNG format (lossless)
    Png,
    /// JPEG format (lossy)
    Jpeg,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            full_page: false,
            width: None,
            height: None,
            quality: Some(90),
            format: ImageFormat::Png,
        }
    }
}

impl ScreenshotOptions {
    /// Create options for a full-page screenshot.
    pub fn full_page(mut self, full_page: bool) -> Self {
        self.full_page = full_page;
        self
    }

    /// Set the viewport dimensions.
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the image quality (0-100, JPEG only).
    pub fn quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality.min(100));
        self
    }

    /// Set the image format.
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }
}

/// Browser actions for interaction automation.
///
/// # Example
///
/// ```
/// use riptide_facade::BrowserAction;
///
/// let actions = vec![
///     BrowserAction::Click { selector: "#submit-btn".to_string() },
///     BrowserAction::Type { selector: "#email".to_string(), text: "user@example.com".to_string() },
///     BrowserAction::Wait { duration_ms: 1000 },
/// ];
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    /// Click an element by CSS selector
    Click { selector: String },

    /// Type text into an element
    Type { selector: String, text: String },

    /// Wait for a specified duration
    Wait { duration_ms: u64 },

    /// Wait for an element to appear
    WaitForElement { selector: String, timeout_ms: u64 },

    /// Scroll to an element
    ScrollTo { selector: String },

    /// Scroll by pixels
    ScrollBy { x: i32, y: i32 },

    /// Submit a form
    Submit { selector: String },

    /// Focus an element
    Focus { selector: String },
}

/// Cookie data for cookie management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<i64>,
    pub http_only: Option<bool>,
    pub secure: Option<bool>,
    pub same_site: Option<String>,
}

impl BrowserFacade {
    /// Create a new browser facade with the given configuration.
    ///
    /// This initializes the browser pool and prepares for browser sessions.
    ///
    /// # Errors
    ///
    /// Returns an error if the browser launcher cannot be initialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use riptide_facade::{BrowserFacade, RiptideConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = RiptideConfig::default();
    /// let facade = BrowserFacade::new(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: RiptideConfig) -> RiptideResult<Self> {
        // Initialize the headless launcher with browser pool
        let launcher = HeadlessLauncher::new().await.map_err(|e| {
            RiptideError::config(format!("Failed to initialize browser launcher: {}", e))
        })?;

        Ok(Self {
            config: Arc::new(config),
            launcher: Arc::new(launcher),
        })
    }

    /// Launch a new browser session.
    ///
    /// This retrieves a browser instance from the pool (or creates a new one)
    /// and returns a managed session. The browser will be returned to the pool
    /// when the session is closed or dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if a browser cannot be launched or retrieved from the pool.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// let session = facade.launch().await?;
    /// // Use session...
    /// facade.close(session).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn launch(&self) -> RiptideResult<BrowserSession<'_>> {
        let session = self
            .launcher
            .launch_page("about:blank", None)
            .await
            .map_err(|e| RiptideError::config(format!("Failed to launch browser: {}", e)))?;

        Ok(BrowserSession { session })
    }

    /// Navigate to a URL in the given browser session.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session to navigate
    /// * `url` - The URL to navigate to
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or navigation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// # let session = facade.launch().await?;
    /// facade.navigate(&session, "https://example.com").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn navigate(&self, session: &BrowserSession<'_>, url: &str) -> RiptideResult<()> {
        // Validate URL
        let _ = Url::parse(url)?;

        session
            .session
            .page()
            .goto(url)
            .await
            .map_err(|e| RiptideError::Fetch(format!("Navigation failed: {}", e)))?;

        Ok(())
    }

    /// Take a screenshot of the current page.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    /// * `options` - Screenshot configuration options
    ///
    /// # Returns
    ///
    /// Returns the screenshot as a byte vector in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if the screenshot cannot be captured.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig, ScreenshotOptions};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// # let session = facade.launch().await?;
    /// # facade.navigate(&session, "https://example.com").await?;
    /// let options = ScreenshotOptions::default().full_page(true);
    /// let screenshot = facade.screenshot(&session, options).await?;
    /// std::fs::write("screenshot.png", screenshot)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn screenshot(
        &self,
        session: &BrowserSession<'_>,
        options: ScreenshotOptions,
    ) -> RiptideResult<Vec<u8>> {
        use chromiumoxide_cdp::cdp::browser_protocol::page::{
            CaptureScreenshotFormat, CaptureScreenshotParams,
        };

        let page = session.session.page();

        // Build screenshot parameters
        let params = CaptureScreenshotParams {
            capture_beyond_viewport: Some(options.full_page),
            format: Some(match options.format {
                ImageFormat::Png => CaptureScreenshotFormat::Png,
                ImageFormat::Jpeg => CaptureScreenshotFormat::Jpeg,
            }),
            quality: options.quality.map(|q| q as i64),
            ..Default::default()
        };

        let screenshot = page
            .screenshot(params)
            .await
            .map_err(|e| RiptideError::Fetch(format!("Screenshot failed: {}", e)))?;

        Ok(screenshot)
    }

    /// Execute JavaScript in the browser context.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    /// * `script` - The JavaScript code to execute
    ///
    /// # Returns
    ///
    /// Returns the result of the script execution as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// # let session = facade.launch().await?;
    /// # facade.navigate(&session, "https://example.com").await?;
    /// let result = facade.execute_script(&session, "document.title").await?;
    /// println!("Page title: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_script(
        &self,
        session: &BrowserSession<'_>,
        script: &str,
    ) -> RiptideResult<serde_json::Value> {
        let page = session.session.page();

        let result = page
            .evaluate(script)
            .await
            .map_err(|e| RiptideError::Fetch(format!("Script execution failed: {}", e)))?;

        let value = result
            .into_value()
            .map_err(|e| RiptideError::Fetch(format!("Failed to parse script result: {}", e)))?;

        Ok(value)
    }

    /// Get the HTML content of the current page.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    ///
    /// # Returns
    ///
    /// Returns the page HTML as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if content retrieval fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// # let session = facade.launch().await?;
    /// # facade.navigate(&session, "https://example.com").await?;
    /// let html = facade.get_content(&session).await?;
    /// println!("Page length: {} bytes", html.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_content(&self, session: &BrowserSession<'_>) -> RiptideResult<String> {
        let page = session.session.page();

        page.content()
            .await
            .map_err(|e| RiptideError::Fetch(format!("Failed to get page content: {}", e)))
    }

    /// Get the rendered text content of the page (without HTML tags).
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    ///
    /// # Returns
    ///
    /// Returns the page text as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if text extraction fails.
    pub async fn get_text(&self, session: &BrowserSession<'_>) -> RiptideResult<String> {
        let script = "document.body.innerText";
        let result = self.execute_script(session, script).await?;

        result
            .as_str()
            .map(String::from)
            .ok_or_else(|| RiptideError::Extraction("Failed to extract text content".to_string()))
    }

    /// Perform a sequence of browser actions.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    /// * `actions` - Slice of actions to perform in order
    ///
    /// # Errors
    ///
    /// Returns an error if any action fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use riptide_facade::{BrowserFacade, RiptideConfig, BrowserAction};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let facade = BrowserFacade::new(RiptideConfig::default()).await?;
    /// # let session = facade.launch().await?;
    /// # facade.navigate(&session, "https://example.com").await?;
    /// let actions = vec![
    ///     BrowserAction::Type {
    ///         selector: "#search".to_string(),
    ///         text: "Rust programming".to_string(),
    ///     },
    ///     BrowserAction::Click { selector: "#submit".to_string() },
    ///     BrowserAction::Wait { duration_ms: 1000 },
    /// ];
    /// facade.perform_actions(&session, &actions).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn perform_actions(
        &self,
        session: &BrowserSession<'_>,
        actions: &[BrowserAction],
    ) -> RiptideResult<()> {
        let page = session.session.page();

        for action in actions {
            match action {
                BrowserAction::Click { selector } => {
                    page.find_element(selector)
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Element not found: {}", e)))?
                        .click()
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Click failed: {}", e)))?;
                }
                BrowserAction::Type { selector, text } => {
                    page.find_element(selector)
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Element not found: {}", e)))?
                        .type_str(text)
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Type failed: {}", e)))?;
                }
                BrowserAction::Wait { duration_ms } => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(*duration_ms)).await;
                }
                BrowserAction::WaitForElement {
                    selector,
                    timeout_ms,
                } => {
                    let timeout = tokio::time::Duration::from_millis(*timeout_ms);
                    tokio::time::timeout(timeout, async {
                        loop {
                            if page.find_element(selector).await.is_ok() {
                                break;
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    })
                    .await
                    .map_err(|_| RiptideError::Timeout)?;
                }
                BrowserAction::ScrollTo { selector } => {
                    let script = format!(
                        "document.querySelector('{}').scrollIntoView({{behavior: 'smooth'}})",
                        selector
                    );
                    self.execute_script(session, &script).await?;
                }
                BrowserAction::ScrollBy { x, y } => {
                    let script = format!("window.scrollBy({}, {})", x, y);
                    self.execute_script(session, &script).await?;
                }
                BrowserAction::Submit { selector } => {
                    let script = format!("document.querySelector('{}').submit()", selector);
                    self.execute_script(session, &script).await?;
                }
                BrowserAction::Focus { selector } => {
                    page.find_element(selector)
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Element not found: {}", e)))?
                        .focus()
                        .await
                        .map_err(|e| RiptideError::Fetch(format!("Focus failed: {}", e)))?;
                }
            }
        }

        Ok(())
    }

    /// Get all cookies for the current page.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    ///
    /// # Returns
    ///
    /// Returns a vector of cookies.
    ///
    /// # Errors
    ///
    /// Returns an error if cookie retrieval fails.
    pub async fn get_cookies(&self, session: &BrowserSession<'_>) -> RiptideResult<Vec<Cookie>> {
        let page = session.session.page();

        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| RiptideError::Fetch(format!("Failed to get cookies: {}", e)))?;

        Ok(cookies
            .into_iter()
            .map(|c| Cookie {
                name: c.name,
                value: c.value,
                domain: Some(c.domain),
                path: Some(c.path),
                expires: Some(c.expires as i64),
                http_only: Some(c.http_only),
                secure: Some(c.secure),
                same_site: c.same_site.map(|s| format!("{:?}", s)),
            })
            .collect())
    }

    /// Set cookies for the current page.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    /// * `cookies` - Cookies to set
    ///
    /// # Errors
    ///
    /// Returns an error if cookie setting fails.
    pub async fn set_cookies(
        &self,
        session: &BrowserSession<'_>,
        cookies: &[Cookie],
    ) -> RiptideResult<()> {
        use chromiumoxide_cdp::cdp::browser_protocol::network::{CookieParam, SetCookiesParams};

        let page = session.session.page();

        let cookie_params: Vec<CookieParam> = cookies
            .iter()
            .map(|c| {
                let mut param = CookieParam::new(c.name.clone(), c.value.clone());
                if let Some(domain) = &c.domain {
                    param.domain = Some(domain.clone());
                }
                if let Some(path) = &c.path {
                    param.path = Some(path.clone());
                }
                param
            })
            .collect();

        page.execute(SetCookiesParams::new(cookie_params))
            .await
            .map_err(|e| RiptideError::Fetch(format!("Failed to set cookies: {}", e)))?;

        Ok(())
    }

    /// Get local storage data.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    ///
    /// # Returns
    ///
    /// Returns local storage as a JSON object.
    ///
    /// # Errors
    ///
    /// Returns an error if storage access fails.
    pub async fn get_local_storage(
        &self,
        session: &BrowserSession<'_>,
    ) -> RiptideResult<serde_json::Value> {
        let script = "JSON.stringify(localStorage)";
        let result = self.execute_script(session, script).await?;

        if let Some(storage_str) = result.as_str() {
            serde_json::from_str(storage_str)
                .map_err(|e| RiptideError::Extraction(format!("Failed to parse storage: {}", e)))
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// Set local storage item.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session
    /// * `key` - Storage key
    /// * `value` - Storage value
    ///
    /// # Errors
    ///
    /// Returns an error if storage write fails.
    pub async fn set_local_storage_item(
        &self,
        session: &BrowserSession<'_>,
        key: &str,
        value: &str,
    ) -> RiptideResult<()> {
        let script = format!(
            "localStorage.setItem('{}', '{}')",
            key.replace('\'', "\\'"),
            value.replace('\'', "\\'")
        );
        self.execute_script(session, &script).await?;
        Ok(())
    }

    /// Close the browser session and return the browser to the pool.
    ///
    /// # Arguments
    ///
    /// * `session` - The browser session to close
    ///
    /// # Errors
    ///
    /// Returns an error if closing fails (though the browser will still be
    /// returned to the pool via Drop).
    pub async fn close(&self, session: BrowserSession<'_>) -> RiptideResult<()> {
        // The LaunchSession will automatically return the browser to the pool
        // when dropped, so we just need to drop it here
        drop(session);
        Ok(())
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RiptideConfig {
        &self.config
    }

    /// Get launcher statistics.
    ///
    /// # Returns
    ///
    /// Returns pool statistics including browser count and health metrics.
    pub async fn stats(&self) -> RiptideResult<String> {
        let stats = self.launcher.stats().await;
        Ok(format!("{:#?}", stats))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_facade_creation() {
        let config = RiptideConfig::default();
        let result = BrowserFacade::new(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_browser_facade_config_access() {
        let config = RiptideConfig::default().with_user_agent("TestBot/1.0");
        let facade = BrowserFacade::new(config).await.unwrap();
        assert_eq!(facade.config().user_agent, "TestBot/1.0");
    }

    #[tokio::test]
    async fn test_screenshot_options_builder() {
        let options = ScreenshotOptions::default()
            .full_page(true)
            .with_viewport(1920, 1080)
            .quality(85);

        assert!(options.full_page);
        assert_eq!(options.width, Some(1920));
        assert_eq!(options.height, Some(1080));
        assert_eq!(options.quality, Some(85));
    }

    #[tokio::test]
    async fn test_browser_action_serialization() {
        let action = BrowserAction::Click {
            selector: "#button".to_string(),
        };
        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("Click"));
        assert!(serialized.contains("#button"));
    }

    #[tokio::test]
    async fn test_cookie_creation() {
        let cookie = Cookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: Some(".example.com".to_string()),
            path: Some("/".to_string()),
            expires: Some(1234567890),
            http_only: Some(true),
            secure: Some(true),
            same_site: Some("Strict".to_string()),
        };

        assert_eq!(cookie.name, "session");
        assert_eq!(cookie.value, "abc123");
        assert_eq!(cookie.domain, Some(".example.com".to_string()));
    }

    // Integration test (requires browser)
    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored flag
    async fn test_browser_launch_and_close() {
        let config = RiptideConfig::default();
        let facade = BrowserFacade::new(config).await.unwrap();

        let session = facade.launch().await.unwrap();
        facade.close(session).await.unwrap();
    }

    // Integration test (requires browser and network)
    #[tokio::test]
    #[ignore]
    async fn test_browser_navigation() {
        let config = RiptideConfig::default();
        let facade = BrowserFacade::new(config).await.unwrap();

        let session = facade.launch().await.unwrap();
        let result = facade.navigate(&session, "https://example.com").await;
        assert!(result.is_ok());

        facade.close(session).await.unwrap();
    }

    // Integration test (requires browser and network)
    #[tokio::test]
    #[ignore]
    async fn test_browser_screenshot() {
        let config = RiptideConfig::default();
        let facade = BrowserFacade::new(config).await.unwrap();

        let session = facade.launch().await.unwrap();
        facade
            .navigate(&session, "https://example.com")
            .await
            .unwrap();

        let options = ScreenshotOptions::default();
        let screenshot = facade.screenshot(&session, options).await.unwrap();
        assert!(!screenshot.is_empty());

        facade.close(session).await.unwrap();
    }

    // Integration test (requires browser and network)
    #[tokio::test]
    #[ignore]
    async fn test_browser_content() {
        let config = RiptideConfig::default();
        let facade = BrowserFacade::new(config).await.unwrap();

        let session = facade.launch().await.unwrap();
        facade
            .navigate(&session, "https://example.com")
            .await
            .unwrap();

        let content = facade.get_content(&session).await.unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("<html"));

        facade.close(session).await.unwrap();
    }
}
