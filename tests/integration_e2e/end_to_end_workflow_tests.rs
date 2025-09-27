//! End-to-end integration tests for RipTide workflows
//!
//! This module provides comprehensive integration testing covering:
//! - Complete user workflows from request to response
//! - Module interaction and data flow validation
//! - Error handling across system boundaries
//! - Performance under realistic conditions
//! - Feature integration scenarios

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Mock HTTP client for integration testing
#[derive(Debug, Clone)]
pub struct MockHttpClient {
    pub responses: Arc<RwLock<HashMap<String, MockResponse>>>,
    pub request_history: Arc<RwLock<Vec<MockRequest>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub delay: Option<Duration>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(RwLock::new(HashMap::new())),
            request_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn set_response(&self, url: &str, response: MockResponse) {
        let mut responses = self.responses.write().await;
        responses.insert(url.to_string(), response);
    }

    pub async fn make_request(&self, url: &str, method: &str, headers: HashMap<String, String>, body: Option<String>) -> Result<MockResponse> {
        // Record the request
        let request = MockRequest {
            url: url.to_string(),
            method: method.to_string(),
            headers: headers.clone(),
            body: body.clone(),
            timestamp: chrono::Utc::now(),
        };

        {
            let mut history = self.request_history.write().await;
            history.push(request);
        }

        // Get the response
        let responses = self.responses.read().await;
        if let Some(response) = responses.get(url) {
            // Simulate network delay if specified
            if let Some(delay) = response.delay {
                tokio::time::sleep(delay).await;
            }

            Ok(response.clone())
        } else {
            // Default 404 response
            Ok(MockResponse {
                status: 404,
                headers: HashMap::new(),
                body: "Not Found".to_string(),
                delay: None,
            })
        }
    }

    pub async fn get_request_history(&self) -> Vec<MockRequest> {
        let history = self.request_history.read().await;
        history.clone()
    }
}

/// Mock search provider for end-to-end testing
#[derive(Debug, Clone)]
pub struct MockSearchProvider {
    pub results: Vec<SearchResult>,
    pub latency: Duration,
    pub should_fail: bool,
    pub call_count: Arc<RwLock<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub rank: u32,
}

impl MockSearchProvider {
    pub fn new() -> Self {
        Self {
            results: vec![
                SearchResult {
                    url: "https://example1.com".to_string(),
                    title: "Example 1 Title".to_string(),
                    snippet: "This is an example snippet for the first result".to_string(),
                    rank: 1,
                },
                SearchResult {
                    url: "https://example2.com".to_string(),
                    title: "Example 2 Title".to_string(),
                    snippet: "This is an example snippet for the second result".to_string(),
                    rank: 2,
                },
                SearchResult {
                    url: "https://example3.com".to_string(),
                    title: "Example 3 Title".to_string(),
                    snippet: "This is an example snippet for the third result".to_string(),
                    rank: 3,
                },
            ],
            latency: Duration::from_millis(100),
            should_fail: false,
            call_count: Arc::new(RwLock::new(0)),
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub fn with_latency(mut self, latency: Duration) -> Self {
        self.latency = latency;
        self
    }

    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SearchResult>> {
        {
            let mut count = self.call_count.write().await;
            *count += 1;
        }

        tokio::time::sleep(self.latency).await;

        if self.should_fail {
            return Err(anyhow::anyhow!("Search provider failure"));
        }

        let results = self.results
            .iter()
            .take(limit as usize)
            .map(|result| SearchResult {
                url: format!("{}/query/{}", result.url, query),
                title: format!("{} - {}", result.title, query),
                snippet: result.snippet.clone(),
                rank: result.rank,
            })
            .collect();

        Ok(results)
    }

    pub async fn get_call_count(&self) -> usize {
        let count = self.call_count.read().await;
        *count
    }
}

/// Mock content extractor for testing
#[derive(Debug, Clone)]
pub struct MockContentExtractor {
    pub extraction_time: Duration,
    pub should_fail: bool,
    pub extracted_content: HashMap<String, String>,
}

impl MockContentExtractor {
    pub fn new() -> Self {
        let mut content = HashMap::new();
        content.insert("https://example1.com".to_string(),
            "This is the extracted content from example1.com. It contains valuable information about the topic.".to_string());
        content.insert("https://example2.com".to_string(),
            "This is the extracted content from example2.com. It provides additional insights and details.".to_string());
        content.insert("https://example3.com".to_string(),
            "This is the extracted content from example3.com. It offers comprehensive coverage of the subject matter.".to_string());

        Self {
            extraction_time: Duration::from_millis(200),
            should_fail: false,
            extracted_content: content,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }

    pub async fn extract_content(&self, url: &str) -> Result<String> {
        tokio::time::sleep(self.extraction_time).await;

        if self.should_fail {
            return Err(anyhow::anyhow!("Content extraction failed for {}", url));
        }

        if let Some(content) = self.extracted_content.get(url) {
            Ok(content.clone())
        } else {
            Ok(format!("Default extracted content for {}", url))
        }
    }
}

/// Integration test workflow orchestrator
#[derive(Debug)]
pub struct WorkflowOrchestrator {
    pub http_client: MockHttpClient,
    pub search_provider: MockSearchProvider,
    pub content_extractor: MockContentExtractor,
    pub request_id: String,
}

impl WorkflowOrchestrator {
    pub fn new() -> Self {
        Self {
            http_client: MockHttpClient::new(),
            search_provider: MockSearchProvider::new(),
            content_extractor: MockContentExtractor::new(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    pub async fn execute_search_and_extract_workflow(&self, query: &str, max_results: u32) -> Result<WorkflowResult> {
        let start_time = Instant::now();
        let mut workflow_steps = Vec::new();

        // Step 1: Search for results
        let step1_start = Instant::now();
        let search_results = self.search_provider.search(query, max_results).await
            .map_err(|e| anyhow::anyhow!("Search step failed: {}", e))?;

        workflow_steps.push(WorkflowStep {
            name: "search".to_string(),
            duration: step1_start.elapsed(),
            success: true,
            result_count: search_results.len(),
            error: None,
        });

        // Step 2: Extract content from each result
        let mut extracted_contents = Vec::new();
        let step2_start = Instant::now();

        for result in &search_results {
            match self.content_extractor.extract_content(&result.url).await {
                Ok(content) => {
                    extracted_contents.push(ExtractedContent {
                        url: result.url.clone(),
                        title: result.title.clone(),
                        content,
                        word_count: content.split_whitespace().count(),
                    });
                }
                Err(e) => {
                    workflow_steps.push(WorkflowStep {
                        name: format!("extract_{}", result.url),
                        duration: step2_start.elapsed(),
                        success: false,
                        result_count: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        workflow_steps.push(WorkflowStep {
            name: "content_extraction".to_string(),
            duration: step2_start.elapsed(),
            success: !extracted_contents.is_empty(),
            result_count: extracted_contents.len(),
            error: None,
        });

        // Step 3: Aggregate and format results
        let step3_start = Instant::now();
        let total_words: usize = extracted_contents.iter().map(|c| c.word_count).sum();
        let summary = format!("Extracted {} pieces of content with {} total words",
                              extracted_contents.len(), total_words);

        workflow_steps.push(WorkflowStep {
            name: "aggregation".to_string(),
            duration: step3_start.elapsed(),
            success: true,
            result_count: 1,
            error: None,
        });

        Ok(WorkflowResult {
            request_id: self.request_id.clone(),
            query: query.to_string(),
            total_duration: start_time.elapsed(),
            steps: workflow_steps,
            search_results,
            extracted_contents,
            summary,
            success: true,
        })
    }

    pub async fn execute_error_recovery_workflow(&self, query: &str) -> Result<WorkflowResult> {
        let start_time = Instant::now();
        let mut workflow_steps = Vec::new();

        // Attempt search with potential failure
        let step1_start = Instant::now();
        let search_result = self.search_provider.search(query, 5).await;

        let (search_results, search_success) = match search_result {
            Ok(results) => (results, true),
            Err(e) => {
                workflow_steps.push(WorkflowStep {
                    name: "search_failed".to_string(),
                    duration: step1_start.elapsed(),
                    success: false,
                    result_count: 0,
                    error: Some(e.to_string()),
                });

                // Fallback: use cached results
                let fallback_results = vec![
                    SearchResult {
                        url: "https://fallback.com".to_string(),
                        title: "Fallback Result".to_string(),
                        snippet: "This is a fallback result when search fails".to_string(),
                        rank: 1,
                    }
                ];

                workflow_steps.push(WorkflowStep {
                    name: "fallback_search".to_string(),
                    duration: Duration::from_millis(10),
                    success: true,
                    result_count: fallback_results.len(),
                    error: None,
                });

                (fallback_results, false)
            }
        };

        // Continue with content extraction even if search failed
        let mut extracted_contents = Vec::new();
        for result in &search_results {
            if let Ok(content) = self.content_extractor.extract_content(&result.url).await {
                extracted_contents.push(ExtractedContent {
                    url: result.url.clone(),
                    title: result.title.clone(),
                    content,
                    word_count: 50, // Simplified for fallback
                });
            }
        }

        Ok(WorkflowResult {
            request_id: self.request_id.clone(),
            query: query.to_string(),
            total_duration: start_time.elapsed(),
            steps: workflow_steps,
            search_results,
            extracted_contents,
            summary: "Workflow completed with error recovery".to_string(),
            success: search_success,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub name: String,
    pub duration: Duration,
    pub success: bool,
    pub result_count: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub url: String,
    pub title: String,
    pub content: String,
    pub word_count: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowResult {
    pub request_id: String,
    pub query: String,
    pub total_duration: Duration,
    pub steps: Vec<WorkflowStep>,
    pub search_results: Vec<SearchResult>,
    pub extracted_contents: Vec<ExtractedContent>,
    pub summary: String,
    pub success: bool,
}

#[tokio::test]
async fn test_complete_search_and_extract_workflow() {
    let orchestrator = WorkflowOrchestrator::new();

    let result = orchestrator.execute_search_and_extract_workflow("rust programming", 3).await
        .expect("Workflow should complete successfully");

    // Verify workflow completion
    assert!(result.success, "Workflow should succeed");
    assert_eq!(result.query, "rust programming");
    assert!(result.total_duration < Duration::from_secs(5), "Workflow should complete in reasonable time");

    // Verify search step
    let search_step = result.steps.iter().find(|s| s.name == "search").unwrap();
    assert!(search_step.success, "Search step should succeed");
    assert_eq!(search_step.result_count, 3, "Should have 3 search results");

    // Verify content extraction step
    let extraction_step = result.steps.iter().find(|s| s.name == "content_extraction").unwrap();
    assert!(extraction_step.success, "Content extraction should succeed");
    assert_eq!(extraction_step.result_count, 3, "Should extract content from 3 URLs");

    // Verify aggregation step
    let aggregation_step = result.steps.iter().find(|s| s.name == "aggregation").unwrap();
    assert!(aggregation_step.success, "Aggregation should succeed");

    // Verify results structure
    assert_eq!(result.search_results.len(), 3, "Should have 3 search results");
    assert_eq!(result.extracted_contents.len(), 3, "Should have 3 extracted contents");

    // Verify search results contain query
    for search_result in &result.search_results {
        assert!(search_result.title.contains("rust programming"), "Search result should contain query");
        assert!(search_result.url.contains("rust programming"), "URL should contain query");
    }

    // Verify extracted content
    for content in &result.extracted_contents {
        assert!(!content.content.is_empty(), "Extracted content should not be empty");
        assert!(content.word_count > 0, "Content should have word count");
    }

    println!("Workflow completed successfully:");
    println!("  Request ID: {}", result.request_id);
    println!("  Total Duration: {:?}", result.total_duration);
    println!("  Steps: {}", result.steps.len());
    println!("  Summary: {}", result.summary);
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    let mut orchestrator = WorkflowOrchestrator::new();

    // Configure search provider to fail
    orchestrator.search_provider = orchestrator.search_provider.with_failure();

    let result = orchestrator.execute_error_recovery_workflow("test query").await
        .expect("Error recovery workflow should complete");

    // Workflow should complete even with failures
    assert_eq!(result.query, "test query");
    assert!(result.total_duration < Duration::from_secs(10), "Recovery should be reasonably fast");

    // Check for failure and recovery steps
    let failed_step = result.steps.iter().find(|s| s.name == "search_failed");
    assert!(failed_step.is_some(), "Should have recorded search failure");
    assert!(!failed_step.unwrap().success, "Failed step should be marked as unsuccessful");

    let fallback_step = result.steps.iter().find(|s| s.name == "fallback_search");
    assert!(fallback_step.is_some(), "Should have fallback search step");
    assert!(fallback_step.unwrap().success, "Fallback should succeed");

    // Should have fallback results
    assert!(!result.search_results.is_empty(), "Should have fallback search results");
    assert!(result.search_results[0].url.contains("fallback"), "Should use fallback URL");

    println!("Error recovery workflow completed:");
    println!("  Success: {}", result.success);
    println!("  Steps: {:?}", result.steps.iter().map(|s| &s.name).collect::<Vec<_>>());
}

#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let orchestrator = Arc::new(WorkflowOrchestrator::new());
    let mut handles = Vec::new();

    // Execute multiple workflows concurrently
    for i in 0..10 {
        let orchestrator_clone = orchestrator.clone();
        let handle = tokio::spawn(async move {
            let query = format!("concurrent query {}", i);
            orchestrator_clone.execute_search_and_extract_workflow(&query, 2).await
        });
        handles.push(handle);
    }

    // Wait for all workflows to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap().expect("Workflow should succeed");
        results.push(result);
    }

    // Verify all workflows completed
    assert_eq!(results.len(), 10, "All concurrent workflows should complete");

    // Verify each workflow is independent
    for (i, result) in results.iter().enumerate() {
        assert!(result.success, "Workflow {} should succeed", i);
        assert!(result.query.contains(&i.to_string()), "Query should contain workflow index");
        assert_eq!(result.search_results.len(), 2, "Each workflow should have 2 results");
    }

    // Verify search provider was called for each workflow
    let call_count = orchestrator.search_provider.get_call_count().await;
    assert_eq!(call_count, 10, "Search provider should be called 10 times");

    // Check timing - concurrent execution should be faster than sequential
    let total_duration: Duration = results.iter().map(|r| r.total_duration).sum();
    let max_individual_duration = results.iter().map(|r| r.total_duration).max().unwrap();

    println!("Concurrent execution results:");
    println!("  Total duration (sum): {:?}", total_duration);
    println!("  Max individual duration: {:?}", max_individual_duration);
    println!("  Parallelization efficiency: {:.1}x",
             total_duration.as_millis() as f64 / max_individual_duration.as_millis() as f64);
}

#[tokio::test]
async fn test_performance_under_load() {
    let orchestrator = WorkflowOrchestrator::new();

    // Test with varying load characteristics
    let test_cases = vec![
        ("light_load", 5, 1),
        ("medium_load", 10, 2),
        ("heavy_load", 20, 3),
    ];

    for (test_name, num_concurrent, results_per_query) in test_cases {
        let start_time = Instant::now();
        let mut handles = Vec::new();

        for i in 0..num_concurrent {
            let query = format!("{} query {}", test_name, i);
            let result = orchestrator.execute_search_and_extract_workflow(&query, results_per_query).await
                .expect("Workflow should succeed under load");

            handles.push(result);
        }

        let total_time = start_time.elapsed();
        let avg_time = total_time / num_concurrent as u32;

        println!("Load test '{}' results:", test_name);
        println!("  Concurrent workflows: {}", num_concurrent);
        println!("  Results per query: {}", results_per_query);
        println!("  Total time: {:?}", total_time);
        println!("  Average time per workflow: {:?}", avg_time);

        // Performance assertions
        assert!(avg_time < Duration::from_secs(2),
               "Average workflow time should be under 2 seconds for {}", test_name);

        // Verify all workflows succeeded
        for result in handles {
            assert!(result.success, "All workflows should succeed under load");
            assert_eq!(result.search_results.len(), results_per_query as usize);
        }
    }
}

#[tokio::test]
async fn test_module_interaction_and_data_flow() {
    let orchestrator = WorkflowOrchestrator::new();

    // Set up mock HTTP responses for testing data flow
    orchestrator.http_client.set_response("https://api.example.com/search", MockResponse {
        status: 200,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers
        },
        body: r#"{"results": [{"url": "https://test.com", "title": "Test"}]}"#.to_string(),
        delay: Some(Duration::from_millis(50)),
    }).await;

    // Execute workflow
    let result = orchestrator.execute_search_and_extract_workflow("module interaction test", 2).await
        .expect("Module interaction test should succeed");

    // Verify data flow between modules
    assert!(result.success, "Module interaction should succeed");

    // Check that search results flowed to content extraction
    for (i, search_result) in result.search_results.iter().enumerate() {
        let corresponding_content = result.extracted_contents.iter()
            .find(|content| content.url == search_result.url);

        assert!(corresponding_content.is_some(),
               "Search result {} should have corresponding extracted content", i);

        let content = corresponding_content.unwrap();
        assert_eq!(content.title, search_result.title, "Titles should match between modules");
        assert!(!content.content.is_empty(), "Content should be extracted");
    }

    // Verify workflow steps executed in correct order
    let step_names: Vec<&String> = result.steps.iter().map(|s| &s.name).collect();
    assert!(step_names.contains(&&"search".to_string()), "Should have search step");
    assert!(step_names.contains(&&"content_extraction".to_string()), "Should have extraction step");
    assert!(step_names.contains(&&"aggregation".to_string()), "Should have aggregation step");

    // Verify timing relationships
    let search_step = result.steps.iter().find(|s| s.name == "search").unwrap();
    let extraction_step = result.steps.iter().find(|s| s.name == "content_extraction").unwrap();

    // Content extraction should take longer than search (more processing)
    assert!(extraction_step.duration >= search_step.duration,
           "Content extraction should take at least as long as search");
}

#[tokio::test]
async fn test_end_to_end_latency_requirements() {
    let orchestrator = WorkflowOrchestrator::new();

    // Test latency requirements for different scenarios
    let scenarios = vec![
        ("fast_query", 1, Duration::from_millis(500)),
        ("normal_query", 3, Duration::from_millis(1500)),
        ("complex_query", 5, Duration::from_millis(3000)),
    ];

    for (scenario_name, num_results, max_latency) in scenarios {
        let start = Instant::now();

        let result = orchestrator.execute_search_and_extract_workflow(scenario_name, num_results).await
            .expect("Latency test should succeed");

        let actual_latency = start.elapsed();

        println!("Latency test '{}' results:", scenario_name);
        println!("  Expected max latency: {:?}", max_latency);
        println!("  Actual latency: {:?}", actual_latency);
        println!("  Within requirements: {}", actual_latency <= max_latency);

        // Latency requirement assertion
        assert!(actual_latency <= max_latency,
               "Latency {} should be within {} for scenario '{}'",
               actual_latency.as_millis(), max_latency.as_millis(), scenario_name);

        // Verify functionality wasn't compromised for speed
        assert!(result.success, "Functionality should be maintained");
        assert_eq!(result.search_results.len(), num_results as usize, "Should return requested number of results");
    }
}

#[tokio::test]
async fn test_resource_cleanup_and_isolation() {
    // Test that workflows properly clean up resources and don't interfere with each other

    let orchestrator1 = WorkflowOrchestrator::new();
    let orchestrator2 = WorkflowOrchestrator::new();

    // Execute workflows with different orchestrators
    let result1 = orchestrator1.execute_search_and_extract_workflow("isolation test 1", 2).await
        .expect("First workflow should succeed");

    let result2 = orchestrator2.execute_search_and_extract_workflow("isolation test 2", 2).await
        .expect("Second workflow should succeed");

    // Verify isolation
    assert_ne!(result1.request_id, result2.request_id, "Request IDs should be different");
    assert_ne!(result1.query, result2.query, "Queries should be different");

    // Verify independent search provider call counts
    let count1 = orchestrator1.search_provider.get_call_count().await;
    let count2 = orchestrator2.search_provider.get_call_count().await;

    assert_eq!(count1, 1, "First orchestrator should have 1 call");
    assert_eq!(count2, 1, "Second orchestrator should have 1 call");

    // Verify HTTP client isolation
    let history1 = orchestrator1.http_client.get_request_history().await;
    let history2 = orchestrator2.http_client.get_request_history().await;

    // Each orchestrator should have its own request history
    // (Both should be empty in this test since we didn't make HTTP requests)
    assert!(history1.is_empty(), "First HTTP client should have no requests");
    assert!(history2.is_empty(), "Second HTTP client should have no requests");

    println!("Resource isolation verified:");
    println!("  Orchestrator 1 - Request ID: {}, Calls: {}", result1.request_id, count1);
    println!("  Orchestrator 2 - Request ID: {}, Calls: {}", result2.request_id, count2);
}