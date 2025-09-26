// Mock implementations for testing SearchProvider functionality
// These mocks will be used until actual implementations are created

use std::time::Duration;
use tokio::time::sleep;

// Mock HTTP client for testing external API calls
pub struct MockHttpClient {
    pub responses: Vec<MockHttpResponse>,
    pub current_response_index: std::sync::Mutex<usize>,
}

pub struct MockHttpResponse {
    pub status: u16,
    pub body: String,
    pub delay: Duration,
}

impl MockHttpClient {
    pub fn new(responses: Vec<MockHttpResponse>) -> Self {
        Self {
            responses,
            current_response_index: std::sync::Mutex::new(0),
        }
    }

    pub async fn post(&self, _url: &str, _body: String) -> Result<MockHttpResponse, String> {
        let mut index = self.current_response_index.lock().unwrap();

        if *index < self.responses.len() {
            let response = &self.responses[*index];
            *index += 1;

            // Simulate network delay
            if response.delay > Duration::ZERO {
                sleep(response.delay).await;
            }

            Ok(MockHttpResponse {
                status: response.status,
                body: response.body.clone(),
                delay: response.delay,
            })
        } else {
            Err("No more mock responses available".to_string())
        }
    }

    pub fn reset(&self) {
        let mut index = self.current_response_index.lock().unwrap();
        *index = 0;
    }
}

// Mock search provider for testing
pub struct MockSearchProvider {
    pub should_fail: bool,
    pub response_delay: Duration,
    pub failure_message: String,
}

impl MockSearchProvider {
    pub fn new() -> Self {
        Self {
            should_fail: false,
            response_delay: Duration::ZERO,
            failure_message: "Mock provider failure".to_string(),
        }
    }

    pub fn with_failure(mut self, should_fail: bool, message: &str) -> Self {
        self.should_fail = should_fail;
        self.failure_message = message.to_string();
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MockSearchResult>, String> {
        // Simulate response delay
        if self.response_delay > Duration::ZERO {
            sleep(self.response_delay).await;
        }

        // Simulate failure
        if self.should_fail {
            return Err(self.failure_message.clone());
        }

        // Return mock results
        Ok(vec![
            MockSearchResult {
                title: format!("Mock result for: {}", query),
                url: "https://example.com/mock".to_string(),
                snippet: "This is a mock search result for testing".to_string(),
                relevance_score: 0.85,
            }
        ])
    }
}

#[derive(Debug, Clone)]
pub struct MockSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub relevance_score: f64,
}

// Mock circuit breaker for testing
pub struct MockCircuitBreaker {
    pub state: std::sync::Arc<std::sync::Mutex<MockCircuitState>>,
    pub failure_count: std::sync::Arc<std::sync::Mutex<u32>>,
    pub last_failure_time: std::sync::Arc<std::sync::Mutex<Option<std::time::Instant>>>,
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MockCircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl MockCircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: std::sync::Arc::new(std::sync::Mutex::new(MockCircuitState::Closed)),
            failure_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            last_failure_time: std::sync::Arc::new(std::sync::Mutex::new(None)),
            failure_threshold,
            recovery_timeout,
        }
    }

    pub fn can_execute(&self) -> bool {
        let state = self.get_state();
        match state {
            MockCircuitState::Closed | MockCircuitState::HalfOpen => true,
            MockCircuitState::Open => false,
        }
    }

    pub fn get_state(&self) -> MockCircuitState {
        let mut state = self.state.lock().unwrap();

        // Check if we should transition from Open to HalfOpen
        if *state == MockCircuitState::Open {
            if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                if last_failure.elapsed() >= self.recovery_timeout {
                    *state = MockCircuitState::HalfOpen;
                }
            }
        }

        state.clone()
    }

    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();

        *failure_count = 0;
        *state = MockCircuitState::Closed;
    }

    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let mut failure_count = self.failure_count.lock().unwrap();
        let mut last_failure_time = self.last_failure_time.lock().unwrap();

        *failure_count += 1;
        *last_failure_time = Some(std::time::Instant::now());

        if *failure_count >= self.failure_threshold {
            *state = MockCircuitState::Open;
        }
    }

    pub fn get_metrics(&self) -> MockCircuitBreakerMetrics {
        let state = self.get_state();
        let failure_count = *self.failure_count.lock().unwrap();

        MockCircuitBreakerMetrics {
            state,
            failure_count,
            success_count: 0, // Simplified for mock
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockCircuitBreakerMetrics {
    pub state: MockCircuitState,
    pub failure_count: u32,
    pub success_count: u32,
}

// Mock factory for creating test providers
pub struct MockSearchProviderFactory;

impl MockSearchProviderFactory {
    pub fn create_mock_serper() -> MockSearchProvider {
        MockSearchProvider::new()
    }

    pub fn create_mock_none() -> MockSearchProvider {
        MockSearchProvider::new()
            .with_delay(Duration::from_millis(10)) // Very fast for URL detection
    }

    pub fn create_failing_provider() -> MockSearchProvider {
        MockSearchProvider::new()
            .with_failure(true, "API key invalid")
    }
}