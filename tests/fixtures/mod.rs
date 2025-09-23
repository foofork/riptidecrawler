/// Test fixtures and utilities for London School TDD
///
/// This module provides mock objects, test data, and utilities following
/// the London School (mockist) approach to TDD.

use mockall::mock;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

pub mod test_data;
pub mod mock_services;
pub mod spa_fixtures;
pub mod contract_definitions;

/// Mock HTTP client for testing network interactions
mock! {
    pub HttpClient {}

    #[async_trait::async_trait]
    impl HttpClientTrait for HttpClient {
        async fn get(&self, url: &str) -> Result<MockResponse, reqwest::Error>;
        async fn post(&self, url: &str, body: &str) -> Result<MockResponse, reqwest::Error>;
        async fn get_with_headers(&self, url: &str, headers: HashMap<String, String>) -> Result<MockResponse, reqwest::Error>;
    }
}

/// Mock WASM component for testing extraction behavior
mock! {
    pub WasmExtractor {}

    impl WasmExtractorTrait for WasmExtractor {
        fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedContent, String>;
        fn validate_html(&self, html: &str) -> Result<bool, String>;
        fn health_check(&self) -> HealthStatus;
        fn get_info(&self) -> ComponentInfo;
    }
}

/// Mock dynamic renderer for testing SPA interactions
mock! {
    pub DynamicRenderer {}

    #[async_trait::async_trait]
    impl DynamicRendererTrait for DynamicRenderer {
        async fn render(&self, url: &str, config: &DynamicConfig) -> Result<RenderResult, String>;
        async fn execute_actions(&self, actions: &[Action]) -> Result<Vec<ActionResult>, String>;
        async fn wait_for_conditions(&self, conditions: &[WaitCondition]) -> Result<bool, String>;
    }
}

/// Mock session manager for testing persistence
mock! {
    pub SessionManager {}

    #[async_trait::async_trait]
    impl SessionManagerTrait for SessionManager {
        async fn create_session(&self, id: &str) -> Result<Session, String>;
        async fn get_session(&self, id: &str) -> Result<Option<Session>, String>;
        async fn update_session(&self, session: &Session) -> Result<(), String>;
        async fn delete_session(&self, id: &str) -> Result<(), String>;
    }
}

#[async_trait::async_trait]
pub trait HttpClientTrait {
    async fn get(&self, url: &str) -> Result<MockResponse, reqwest::Error>;
    async fn post(&self, url: &str, body: &str) -> Result<MockResponse, reqwest::Error>;
    async fn get_with_headers(&self, url: &str, headers: HashMap<String, String>) -> Result<MockResponse, reqwest::Error>;
}

pub trait WasmExtractorTrait {
    fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedContent, String>;
    fn validate_html(&self, html: &str) -> Result<bool, String>;
    fn health_check(&self) -> HealthStatus;
    fn get_info(&self) -> ComponentInfo;
}

#[async_trait::async_trait]
pub trait DynamicRendererTrait {
    async fn render(&self, url: &str, config: &DynamicConfig) -> Result<RenderResult, String>;
    async fn execute_actions(&self, actions: &[Action]) -> Result<Vec<ActionResult>, String>;
    async fn wait_for_conditions(&self, conditions: &[WaitCondition]) -> Result<bool, String>;
}

#[async_trait::async_trait]
pub trait SessionManagerTrait {
    async fn create_session(&self, id: &str) -> Result<Session, String>;
    async fn get_session(&self, id: &str) -> Result<Option<Session>, String>;
    async fn update_session(&self, session: &Session) -> Result<(), String>;
    async fn delete_session(&self, id: &str) -> Result<(), String>;
}

/// Mock response for HTTP operations
#[derive(Clone, Debug)]
pub struct MockResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub url: String,
}

impl MockResponse {
    pub fn new(status: u16, body: String) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body,
            url: "http://example.com".to_string(),
        }
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
}

/// Test data structures
#[derive(Clone, Debug)]
pub struct ExtractedContent {
    pub url: String,
    pub title: Option<String>,
    pub content: String,
    pub links: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub memory_usage: u64,
}

#[derive(Clone, Debug)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DynamicConfig {
    pub actions: Vec<Action>,
    pub wait_conditions: Vec<WaitCondition>,
    pub timeout: Duration,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub action_type: String,
    pub selector: Option<String>,
    pub value: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WaitCondition {
    pub condition_type: String,
    pub selector: Option<String>,
    pub timeout: Duration,
}

#[derive(Clone, Debug)]
pub struct RenderResult {
    pub html: String,
    pub success: bool,
    pub actions_executed: Vec<ActionResult>,
}

#[derive(Clone, Debug)]
pub struct ActionResult {
    pub action: Action,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub created_at: std::time::SystemTime,
    pub last_accessed: std::time::SystemTime,
    pub data: HashMap<String, String>,
}

/// Test URL sets for validation
pub struct TestUrls;

impl TestUrls {
    /// 5-URL mixed validation set
    pub fn mixed_validation_set() -> Vec<(&'static str, &'static str)> {
        vec![
            ("https://example.com/article", "article"),
            ("https://spa-app.com/dashboard", "spa"),
            ("https://docs.example.com/api.pdf", "pdf"),
            ("https://news.com/breaking-news", "news"),
            ("https://ecommerce.com/product/123", "product"),
        ]
    }

    /// SPA fixtures with dynamic content
    pub fn spa_fixtures() -> Vec<(&'static str, DynamicConfig)> {
        vec![
            (
                "https://spa-app.com/dashboard",
                DynamicConfig {
                    actions: vec![
                        Action {
                            action_type: "click".to_string(),
                            selector: Some("#load-more".to_string()),
                            value: None,
                        },
                        Action {
                            action_type: "wait".to_string(),
                            selector: None,
                            value: Some("2000".to_string()),
                        },
                    ],
                    wait_conditions: vec![
                        WaitCondition {
                            condition_type: "element_visible".to_string(),
                            selector: Some(".dynamic-content".to_string()),
                            timeout: Duration::from_secs(10),
                        }
                    ],
                    timeout: Duration::from_secs(30),
                }
            ),
        ]
    }
}