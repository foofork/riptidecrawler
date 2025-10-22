//! Test utilities for CLI integration tests
//!
//! Provides common helpers, fixtures, and mock server utilities

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Helper to create temporary test directory
pub fn create_temp_test_dir(prefix: &str) -> Result<tempfile::TempDir> {
    Ok(tempfile::Builder::new().prefix(prefix).tempdir()?)
}

/// Get test output directory path
pub fn get_test_output_dir(temp_dir: &tempfile::TempDir, subdir: &str) -> PathBuf {
    temp_dir.path().join(subdir)
}

/// Mock API server builder
pub struct MockApiServerBuilder {
    server: Option<MockServer>,
    with_health: bool,
    with_render: bool,
    with_extract: bool,
    with_screenshot: bool,
    with_auth: bool,
    api_key: Option<String>,
}

impl MockApiServerBuilder {
    pub fn new() -> Self {
        Self {
            server: None,
            with_health: false,
            with_render: false,
            with_extract: false,
            with_screenshot: false,
            with_auth: false,
            api_key: None,
        }
    }

    pub fn with_health_endpoint(mut self) -> Self {
        self.with_health = true;
        self
    }

    pub fn with_render_endpoint(mut self) -> Self {
        self.with_render = true;
        self
    }

    pub fn with_extract_endpoint(mut self) -> Self {
        self.with_extract = true;
        self
    }

    pub fn with_screenshot_endpoint(mut self) -> Self {
        self.with_screenshot = true;
        self
    }

    pub fn with_authentication(mut self, api_key: String) -> Self {
        self.with_auth = true;
        self.api_key = Some(api_key);
        self
    }

    pub async fn build(mut self) -> Result<MockApiServer> {
        let server = MockServer::start().await;

        if self.with_health {
            setup_health_endpoint(&server).await;
        }

        if self.with_render {
            setup_render_endpoint(&server, self.with_auth, self.api_key.as_deref()).await;
        }

        if self.with_extract {
            setup_extract_endpoint(&server, self.with_auth, self.api_key.as_deref()).await;
        }

        if self.with_screenshot {
            setup_screenshot_endpoint(&server, self.with_auth, self.api_key.as_deref()).await;
        }

        Ok(MockApiServer {
            server,
            request_count: Arc::new(Mutex::new(0)),
        })
    }
}

/// Mock API server for testing
pub struct MockApiServer {
    server: MockServer,
    request_count: Arc<Mutex<usize>>,
}

impl MockApiServer {
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    pub async fn request_count(&self) -> usize {
        *self.request_count.lock().await
    }

    pub async fn increment_requests(&self) {
        let mut count = self.request_count.lock().await;
        *count += 1;
    }

    pub fn inner(&self) -> &MockServer {
        &self.server
    }
}

/// Setup health endpoint
async fn setup_health_endpoint(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "version": "1.0.0"
        })))
        .mount(server)
        .await;
}

/// Setup render endpoint
async fn setup_render_endpoint(server: &MockServer, with_auth: bool, api_key: Option<&str>) {
    let mut mock = Mock::given(method("POST")).and(path("/api/v1/render"));

    if with_auth {
        if let Some(key) = api_key {
            mock = mock.and(wiremock::matchers::header("X-API-Key", key));
        }
    }

    mock.respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": true,
        "html": "<html><body><h1>Test</h1></body></html>",
        "metadata": {
            "final_url": "https://example.com",
            "title": "Test",
            "render_time_ms": 100,
            "resources_loaded": 1,
            "cookies_set": 0
        }
    })))
    .mount(server)
    .await;
}

/// Setup extract endpoint
async fn setup_extract_endpoint(server: &MockServer, with_auth: bool, api_key: Option<&str>) {
    let mut mock = Mock::given(method("POST")).and(path("/api/v1/extract"));

    if with_auth {
        if let Some(key) = api_key {
            mock = mock.and(wiremock::matchers::header("X-API-Key", key));
        }
    }

    mock.respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "success": true,
        "data": {
            "title": "Test",
            "content": "Test content"
        },
        "metadata": {
            "url": "https://example.com",
            "extracted_fields": 2,
            "extraction_time_ms": 50
        }
    })))
    .mount(server)
    .await;
}

/// Setup screenshot endpoint
async fn setup_screenshot_endpoint(server: &MockServer, with_auth: bool, api_key: Option<&str>) {
    let mut mock = Mock::given(method("POST")).and(path("/api/v1/screenshot"));

    if with_auth {
        if let Some(key) = api_key {
            mock = mock.and(wiremock::matchers::header("X-API-Key", key));
        }
    }

    // PNG header
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    mock.respond_with(ResponseTemplate::new(200).set_body_bytes(png_data))
        .mount(server)
        .await;
}

/// Create a test render request
pub fn create_test_render_request(url: &str) -> riptide_cli::api_client::RenderRequest {
    riptide_cli::api_client::RenderRequest {
        url: url.to_string(),
        wait_condition: "load".to_string(),
        screenshot_mode: "none".to_string(),
        viewport: riptide_cli::api_client::ViewportConfig {
            width: 1920,
            height: 1080,
        },
        stealth_level: "medium".to_string(),
        javascript_enabled: true,
        extra_timeout: 0,
        user_agent: None,
        proxy: None,
        session_id: None,
    }
}

/// Create a test extract request
pub fn create_test_extract_request(url: &str, selectors: Vec<String>) -> riptide_cli::api_client::ExtractRequest {
    riptide_cli::api_client::ExtractRequest {
        url: url.to_string(),
        selectors,
        schema: None,
        wasm_module: None,
    }
}

/// Create a test screenshot request
pub fn create_test_screenshot_request(url: &str) -> riptide_cli::api_client::ScreenshotRequest {
    riptide_cli::api_client::ScreenshotRequest {
        url: url.to_string(),
        viewport: riptide_cli::api_client::ViewportConfig {
            width: 1920,
            height: 1080,
        },
        full_page: false,
        wait_condition: Some("load".to_string()),
        selector: None,
    }
}

/// Assert that a directory exists and contains files
pub fn assert_directory_has_files(dir: &PathBuf) -> Result<()> {
    assert!(dir.exists(), "Directory does not exist: {:?}", dir);

    let entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .collect();

    assert!(!entries.is_empty(), "Directory is empty: {:?}", dir);
    Ok(())
}

/// Assert that a file exists and has content
pub fn assert_file_has_content(file: &PathBuf) -> Result<()> {
    assert!(file.exists(), "File does not exist: {:?}", file);

    let metadata = std::fs::metadata(file)?;
    assert!(metadata.len() > 0, "File is empty: {:?}", file);

    Ok(())
}

/// Wait for condition with timeout
pub async fn wait_for_condition<F, Fut>(mut condition: F, timeout_secs: u64) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    anyhow::bail!("Condition not met within {} seconds", timeout_secs)
}

/// Environment variable guard for tests
pub struct EnvGuard {
    var: String,
    old_value: Option<String>,
}

impl EnvGuard {
    pub fn new(var: &str, value: &str) -> Self {
        let old_value = std::env::var(var).ok();
        std::env::set_var(var, value);
        Self {
            var: var.to_string(),
            old_value,
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old_value {
            Some(val) => std::env::set_var(&self.var, val),
            None => std::env::remove_var(&self.var),
        }
    }
}

/// Test fixture for API client
pub struct ApiClientFixture {
    pub client: riptide_cli::api_client::RiptideApiClient,
    pub server: MockApiServer,
    _temp_dir: tempfile::TempDir,
}

impl ApiClientFixture {
    pub async fn new() -> Result<Self> {
        let server = MockApiServerBuilder::new()
            .with_health_endpoint()
            .with_render_endpoint()
            .with_extract_endpoint()
            .with_screenshot_endpoint()
            .build()
            .await?;

        let client = riptide_cli::api_client::RiptideApiClient::new(
            server.uri(),
            None,
        )?;

        let temp_dir = create_temp_test_dir("riptide-test")?;

        Ok(Self {
            client,
            server,
            _temp_dir: temp_dir,
        })
    }

    pub async fn with_auth(api_key: String) -> Result<Self> {
        let server = MockApiServerBuilder::new()
            .with_health_endpoint()
            .with_render_endpoint()
            .with_extract_endpoint()
            .with_screenshot_endpoint()
            .with_authentication(api_key.clone())
            .build()
            .await?;

        let client = riptide_cli::api_client::RiptideApiClient::new(
            server.uri(),
            Some(api_key),
        )?;

        let temp_dir = create_temp_test_dir("riptide-test")?;

        Ok(Self {
            client,
            server,
            _temp_dir: temp_dir,
        })
    }

    pub fn output_dir(&self) -> PathBuf {
        self._temp_dir.path().to_path_buf()
    }
}

/// Performance timer for benchmarking
pub struct PerfTimer {
    start: std::time::Instant,
    label: String,
}

impl PerfTimer {
    pub fn new(label: &str) -> Self {
        Self {
            start: std::time::Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn assert_under_ms(&self, max_ms: u128) {
        let elapsed = self.elapsed_ms();
        assert!(
            elapsed <= max_ms,
            "{} took {}ms, expected under {}ms",
            self.label, elapsed, max_ms
        );
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            println!("⏱️  {} completed in {}ms", self.label, self.elapsed_ms());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_builder() -> Result<()> {
        let server = MockApiServerBuilder::new()
            .with_health_endpoint()
            .build()
            .await?;

        assert!(!server.uri().is_empty());
        Ok(())
    }

    #[test]
    fn test_temp_directory_creation() -> Result<()> {
        let temp_dir = create_temp_test_dir("test")?;
        assert!(temp_dir.path().exists());
        Ok(())
    }

    #[test]
    fn test_env_guard() {
        let original = std::env::var("TEST_VAR").ok();

        {
            let _guard = EnvGuard::new("TEST_VAR", "test_value");
            assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
        }

        // After guard drops, value should be restored
        assert_eq!(std::env::var("TEST_VAR").ok(), original);
    }

    #[tokio::test]
    async fn test_api_client_fixture() -> Result<()> {
        let fixture = ApiClientFixture::new().await?;
        assert!(fixture.client.is_available().await);
        Ok(())
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10);
    }
}
