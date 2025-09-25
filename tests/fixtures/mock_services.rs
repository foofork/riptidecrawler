// Mock services for testing
use mockall::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[automock]
#[async_trait]
pub trait HttpService {
    async fn send_request(&self, request: MockRequest) -> Result<MockResponse, String>;
}

#[automock]
#[async_trait]
pub trait ExtractionService {
    async fn extract(&self, html: &str, selector: &str) -> Result<Vec<String>, String>;
}

#[automock]
#[async_trait]
pub trait RenderingService {
    async fn render(&self, url: &str) -> Result<String, String>;
}

#[automock]
pub trait ValidationService {
    fn validate_url(&self, url: &str) -> Result<(), String>;
    fn validate_content(&self, content: &str) -> Result<(), String>;
}

pub struct MockServiceBuilder;

impl MockServiceBuilder {
    pub fn http_service() -> MockHttpService {
        MockHttpService::new()
    }

    pub fn extraction_service() -> MockExtractionService {
        MockExtractionService::new()
    }

    pub fn rendering_service() -> MockRenderingService {
        MockRenderingService::new()
    }

    pub fn validation_service() -> MockValidationService {
        MockValidationService::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_services_available() {
        let mut http_service = MockServiceBuilder::http_service();
        http_service
            .expect_send_request()
            .returning(|_| Ok(MockResponse {
                status: 200,
                body: "OK".to_string(),
                headers: HashMap::new(),
            }));

        let request = MockRequest {
            url: "http://test.com".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        };

        let response = http_service.send_request(request).await.unwrap();
        assert_eq!(response.status, 200);
    }
}