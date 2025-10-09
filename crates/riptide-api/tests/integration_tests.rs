//! Integration Tests for Missing API Endpoints
//!
//! This file contains comprehensive TDD tests for the missing API integrations:
//! 1. Table Extraction API Tests
//! 2. LLM Provider Management Tests
//! 3. Advanced Chunking Configuration Tests
//!
//! These tests follow the RED phase of TDD - they will FAIL until the corresponding
//! endpoints are implemented. Each test documents the expected behavior and API contract.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;

// Import from existing API structure - this will fail until create_app exists
// Note: This is intentionally commented out for now as it doesn't exist yet
// use riptide_api::*;

// Test utilities and helpers
mod test_utils {
    use super::*;

    /// Creates a test app instance for integration testing
    /// TODO(P1): Implement app factory for integration testing
    /// PLAN: Create testable app instance with proper configuration
    /// IMPLEMENTATION:
    ///   1. Move app creation logic to lib.rs as public function
    ///   2. Accept test configuration for deterministic behavior
    ///   3. Use in-memory backends for Redis/services where possible
    ///   4. Return configured Router ready for testing
    ///
    ///      DEPENDENCIES: Requires refactoring main.rs app setup
    ///      EFFORT: Medium (4-6 hours)
    ///      PRIORITY: Important for comprehensive testing
    ///      BLOCKER: None
    ///      This should eventually call the real app creation function from riptide_api
    pub fn create_test_app() -> axum::Router {
        // This will fail until the app factory function is created in lib.rs or similar
        // Expected to be something like: riptide_api::create_app_with_config(test_config)
        panic!("create_test_app not yet implemented - need to create app factory function in riptide_api crate");
    }

    /// Helper to make HTTP requests and parse JSON responses
    pub async fn make_json_request(
        app: axum::Router,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let request_builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json");

        let request = if let Some(body_json) = body {
            request_builder
                .body(Body::from(body_json.to_string()))
                .unwrap()
        } else {
            request_builder.body(Body::empty()).unwrap()
        };

        let response = app.oneshot(request).await.expect("Request should succeed");

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Body should be readable");

        let json_body: Value = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body_bytes)
                .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&body_bytes)}))
        };

        (status, json_body)
    }

    /// Sample HTML content with tables for testing table extraction
    pub fn sample_html_with_tables() -> &'static str {
        r#"
        <!DOCTYPE html>
        <html>
        <head><title>Sample Page with Tables</title></head>
        <body>
            <h1>Product Catalog</h1>
            <table id="products" class="data-table">
                <thead>
                    <tr>
                        <th>Product ID</th>
                        <th>Name</th>
                        <th>Price</th>
                        <th>Category</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>001</td>
                        <td>Laptop</td>
                        <td>$999.99</td>
                        <td>Electronics</td>
                    </tr>
                    <tr>
                        <td>002</td>
                        <td>Mouse</td>
                        <td>$24.99</td>
                        <td>Accessories</td>
                    </tr>
                </tbody>
            </table>

            <h2>Complex Table with Spans</h2>
            <table id="complex" class="complex-table">
                <tr>
                    <th colspan="2">Header Span</th>
                    <th>Single</th>
                </tr>
                <tr>
                    <td rowspan="2">Row Span</td>
                    <td>Data 1</td>
                    <td>Data 2</td>
                </tr>
                <tr>
                    <td>Data 3</td>
                    <td>Data 4</td>
                </tr>
            </table>
        </body>
        </html>
        "#
    }

    /// Sample text content for testing advanced chunking
    pub fn sample_long_text() -> &'static str {
        r#"
        Machine learning is a method of data analysis that automates analytical model building.
        It is a branch of artificial intelligence based on the idea that systems can learn from
        data, identify patterns and make decisions with minimal human intervention.

        Deep learning is part of a broader family of machine learning methods based on artificial
        neural networks with representation learning. Learning can be supervised, semi-supervised
        or unsupervised. Deep-learning architectures such as deep neural networks, deep belief
        networks, recurrent neural networks, and convolutional neural networks have been applied
        to fields including computer vision, speech recognition, natural language processing,
        machine translation, bioinformatics and drug design.

        Natural language processing is a subfield of linguistics, computer science, and artificial
        intelligence concerned with the interactions between computers and human language.
        In particular, how to program computers to process and analyze large amounts of natural
        language data. The goal is a computer capable of understanding the contents of documents,
        including the contextual nuances of the language within them.
        "#
    }
}

/// Tests for Table Extraction API endpoints
///
/// These tests validate the table extraction functionality that should:
/// - Extract tables from HTML content
/// - Export tables in different formats (CSV, Markdown)
/// - Handle complex table structures with colspan/rowspan
/// - Provide table metadata and structure information
#[cfg(test)]
mod table_extraction_tests {
    use super::*;
    use test_utils::*;

    /// Test the main table extraction endpoint
    ///
    /// Expected behavior:
    /// - POST /api/v1/tables/extract accepts HTML content or URL
    /// - Returns structured table data with metadata
    /// - Identifies all tables in the content
    /// - Handles both direct HTML content and URL fetching
    #[tokio::test]
    async fn test_table_extraction_from_html() {
        let app = create_test_app();

        let request_body = json!({
            "html_content": sample_html_with_tables(),
            "extract_options": {
                "include_headers": true,
                "preserve_formatting": true,
                "detect_data_types": true
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/tables/extract", Some(request_body)).await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Table extraction endpoint should exist and return OK"
        );

        // Expected response structure
        assert!(
            response["tables"].is_array(),
            "Response should contain tables array"
        );
        let tables = response["tables"].as_array().unwrap();
        assert_eq!(tables.len(), 2, "Should detect 2 tables in sample HTML");

        // Validate first table (products table)
        let products_table = &tables[0];
        assert_eq!(
            products_table["id"], "products",
            "First table should have id 'products'"
        );
        assert_eq!(
            products_table["rows"], 3,
            "Products table should have 3 rows (including header)"
        );
        assert_eq!(
            products_table["columns"], 4,
            "Products table should have 4 columns"
        );

        // Validate table data structure
        assert!(
            products_table["headers"].is_array(),
            "Should include headers"
        );
        assert!(
            products_table["data"].is_array(),
            "Should include data rows"
        );
        assert!(
            products_table["metadata"].is_object(),
            "Should include metadata"
        );

        // Validate complex table with spans
        let complex_table = &tables[1];
        assert_eq!(
            complex_table["id"], "complex",
            "Second table should have id 'complex'"
        );
        assert_eq!(
            complex_table["has_spans"], true,
            "Should detect colspan/rowspan usage"
        );
    }

    /// Test table extraction from URL
    ///
    /// Expected behavior:
    /// - Fetch HTML content from provided URL
    /// - Extract tables from fetched content
    /// - Handle network errors gracefully
    #[tokio::test]
    async fn test_table_extraction_from_url() {
        let app = create_test_app();

        let request_body = json!({
            "url": "https://example.com/tables",
            "extract_options": {
                "timeout_seconds": 10,
                "follow_redirects": true,
                "user_agent": "RiptideBot/1.0"
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/tables/extract", Some(request_body)).await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "URL-based table extraction should work"
        );
        assert!(
            response["url"].is_string(),
            "Response should include source URL"
        );
        assert!(
            response["tables"].is_array(),
            "Response should contain extracted tables"
        );
        assert!(
            response["extraction_time_ms"].is_number(),
            "Should include timing metrics"
        );
    }

    /// Test CSV export functionality
    ///
    /// Expected behavior:
    /// - GET /api/v1/tables/{id}/export?format=csv
    /// - Returns properly formatted CSV with headers
    /// - Handles special characters and escaping
    /// - Includes appropriate content-type header
    #[tokio::test]
    async fn test_table_csv_export() {
        let app = create_test_app();

        // First extract tables to get a table ID (this would normally be done in a previous step)
        let table_id = "table_12345"; // This would come from the extraction response

        let (status, _response) = make_json_request(
            app,
            "GET",
            &format!("/api/v1/tables/{}/export?format=csv", table_id),
            None,
        )
        .await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(status, StatusCode::OK, "CSV export should return OK");

        // TODO(P1): Validate CSV content structure
        // VALIDATION CHECKLIST:
        //   1. Check content-type header is text/csv
        //   2. Validate CSV format with proper escaping
        //   3. Ensure headers are included if requested
        //   4. Verify row count matches table rows
        //   5. Check special characters are properly escaped
        // EFFORT: Low (1-2 hours)
        // BLOCKER: Requires endpoint implementation first
        // - Check content-type header is text/csv
        // - Validate CSV format with proper escaping
        // - Ensure headers are included if requested
    }

    /// Test Markdown export functionality
    ///
    /// Expected behavior:
    /// - GET /api/v1/tables/{id}/export?format=markdown
    /// - Returns table in Markdown table format
    /// - Preserves table structure and alignment
    /// - Includes appropriate content-type header
    #[tokio::test]
    async fn test_table_markdown_export() {
        let app = create_test_app();

        let table_id = "table_12345";

        let (status, _response) = make_json_request(
            app,
            "GET",
            &format!("/api/v1/tables/{}/export?format=markdown", table_id),
            None,
        )
        .await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(status, StatusCode::OK, "Markdown export should return OK");

        // TODO(P1): Validate Markdown table format
        // VALIDATION CHECKLIST:
        //   1. Check content includes proper table syntax with |
        //   2. Verify alignment row with ---
        //   3. Ensure special characters are escaped properly
        //   4. Validate table headers match structure
        //   5. Check cell content formatting
        // EFFORT: Low (1-2 hours)
        // BLOCKER: Requires endpoint implementation first
        // - Check content includes proper table syntax with |
        // - Verify alignment row with ---
        // - Ensure special characters are escaped properly
    }

    /// Test table extraction with complex span handling
    ///
    /// Expected behavior:
    /// - Properly handle colspan and rowspan attributes
    /// - Maintain table structure integrity
    /// - Provide span information in metadata
    #[tokio::test]
    async fn test_complex_table_span_handling() {
        let app = create_test_app();

        let complex_html = r#"
        <table>
            <tr>
                <th colspan="3">Quarterly Sales Report</th>
                <th rowspan="2">Notes</th>
            </tr>
            <tr>
                <th>Q1</th>
                <th>Q2</th>
                <th>Q3</th>
            </tr>
            <tr>
                <td>$10K</td>
                <td>$15K</td>
                <td>$12K</td>
                <td rowspan="2">Good performance</td>
            </tr>
        </table>
        "#;

        let request_body = json!({
            "html_content": complex_html,
            "extract_options": {
                "preserve_spans": true,
                "normalize_structure": false
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/tables/extract", Some(request_body)).await;

        // This test will FAIL until complex span handling is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Complex table extraction should succeed"
        );

        let table = &response["tables"][0];
        assert!(table["spans"].is_array(), "Should include span information");
        assert!(
            table["normalized_structure"].is_object(),
            "Should provide normalized view"
        );
    }

    /// Test edge cases and error handling for table extraction
    ///
    /// Expected behavior:
    /// - Handle malformed HTML gracefully
    /// - Return appropriate errors for invalid requests
    /// - Handle empty or no-table content
    #[tokio::test]
    async fn test_table_extraction_edge_cases() {
        let app = create_test_app();

        // Test with no tables
        let no_tables_request = json!({
            "html_content": "<html><body><p>No tables here!</p></body></html>"
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/tables/extract",
            Some(no_tables_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "Should handle no-table content gracefully"
        );
        assert_eq!(
            response["tables"].as_array().unwrap().len(),
            0,
            "Should return empty tables array"
        );

        // Test with malformed HTML
        let malformed_request = json!({
            "html_content": "<table><tr><td>Broken table structure</tr></table>"
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/tables/extract",
            Some(malformed_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "Should handle malformed HTML gracefully"
        );
        assert!(
            response["warnings"].is_array(),
            "Should include warnings for malformed content"
        );

        // Test with invalid request format
        let invalid_request = json!({
            "invalid_field": "should cause validation error"
        });

        let (status, _response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/tables/extract",
            Some(invalid_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Should return 400 for invalid requests"
        );
    }
}

/// Tests for LLM Provider Management API endpoints
///
/// These tests validate LLM provider management functionality that should:
/// - List available LLM providers
/// - Switch between different providers
/// - Configure provider-specific settings
/// - Handle failover chains for reliability
#[cfg(test)]
mod llm_provider_tests {
    use super::*;
    use test_utils::*;

    /// Test listing available LLM providers
    ///
    /// Expected behavior:
    /// - GET /api/v1/llm/providers returns list of available providers
    /// - Includes provider capabilities and status
    /// - Shows configuration requirements for each provider
    #[tokio::test]
    async fn test_list_llm_providers() {
        let app = create_test_app();

        let (status, response) = make_json_request(app, "GET", "/api/v1/llm/providers", None).await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "LLM providers list endpoint should exist"
        );

        assert!(
            response["providers"].is_array(),
            "Response should contain providers array"
        );
        let providers = response["providers"].as_array().unwrap();
        assert!(
            !providers.is_empty(),
            "Should have at least one provider configured"
        );

        // Validate provider structure
        for provider in providers {
            assert!(provider["name"].is_string(), "Provider should have name");
            assert!(
                provider["type"].is_string(),
                "Provider should have type (openai, anthropic, etc.)"
            );
            assert!(
                provider["status"].is_string(),
                "Provider should have status (available, unavailable, etc.)"
            );
            assert!(
                provider["capabilities"].is_array(),
                "Provider should list capabilities"
            );
            assert!(
                provider["config_required"].is_array(),
                "Provider should list required config fields"
            );
        }

        // Check for common providers
        let provider_names: Vec<&str> = providers
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert!(
            provider_names.contains(&"openai"),
            "Should include OpenAI provider"
        );
        assert!(
            provider_names.contains(&"anthropic"),
            "Should include Anthropic provider"
        );
    }

    /// Test getting current active LLM provider
    ///
    /// Expected behavior:
    /// - GET /api/v1/llm/providers/current returns currently active provider
    /// - Includes provider configuration and status
    /// - Shows usage statistics if available
    #[tokio::test]
    async fn test_get_current_llm_provider() {
        let app = create_test_app();

        let (status, response) =
            make_json_request(app, "GET", "/api/v1/llm/providers/current", None).await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Current provider endpoint should exist"
        );

        assert!(
            response["provider"].is_object(),
            "Response should contain current provider info"
        );
        let provider = &response["provider"];

        assert!(
            provider["name"].is_string(),
            "Current provider should have name"
        );
        assert!(
            provider["status"].is_string(),
            "Current provider should have status"
        );
        assert!(
            provider["last_used"].is_string(),
            "Should include last used timestamp"
        );
        assert!(
            provider["usage_stats"].is_object(),
            "Should include usage statistics"
        );
    }

    /// Test switching LLM providers
    ///
    /// Expected behavior:
    /// - POST /api/v1/llm/providers/switch changes active provider
    /// - Validates provider exists and is configured
    /// - Returns confirmation of switch with new provider details
    #[tokio::test]
    async fn test_switch_llm_provider() {
        let app = create_test_app();

        let switch_request = json!({
            "provider": "anthropic",
            "config": {
                "model": "claude-3-sonnet",
                "temperature": 0.7,
                "max_tokens": 4000
            },
            "validate_config": true
        });

        let (status, response) = make_json_request(
            app,
            "POST",
            "/api/v1/llm/providers/switch",
            Some(switch_request),
        )
        .await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(status, StatusCode::OK, "Provider switch should succeed");

        assert_eq!(
            response["switched"], true,
            "Should confirm successful switch"
        );
        assert_eq!(
            response["new_provider"]["name"], "anthropic",
            "Should return new provider info"
        );
        assert_eq!(
            response["config_validated"], true,
            "Should confirm config validation"
        );
    }

    /// Test invalid provider switch scenarios
    ///
    /// Expected behavior:
    /// - Reject switches to non-existent providers
    /// - Validate required configuration parameters
    /// - Handle provider unavailability gracefully
    #[tokio::test]
    async fn test_invalid_provider_switch() {
        let app = create_test_app();

        // Test switch to non-existent provider
        let invalid_provider_request = json!({
            "provider": "nonexistent_provider"
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/llm/providers/switch",
            Some(invalid_provider_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Should reject non-existent provider"
        );
        assert!(
            response["error"].as_str().unwrap().contains("not found"),
            "Error should mention provider not found"
        );

        // Test switch with invalid configuration
        let invalid_config_request = json!({
            "provider": "openai",
            "config": {
                "invalid_param": "should cause validation error"
            }
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/llm/providers/switch",
            Some(invalid_config_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Should reject invalid configuration"
        );
        assert!(
            response["validation_errors"].is_array(),
            "Should include validation errors"
        );
    }

    /// Test LLM provider configuration management
    ///
    /// Expected behavior:
    /// - GET /api/v1/llm/config returns current configuration
    /// - POST /api/v1/llm/config updates configuration
    /// - Validates configuration parameters
    /// - Supports provider-specific settings
    #[tokio::test]
    async fn test_llm_provider_configuration() {
        let app = create_test_app();

        // Test getting current configuration
        let (status, response) =
            make_json_request(app.clone(), "GET", "/api/v1/llm/config", None).await;

        // This test will FAIL until the endpoint is implemented
        assert_eq!(status, StatusCode::OK, "Config get endpoint should exist");

        assert!(
            response["provider"].is_string(),
            "Config should include current provider"
        );
        assert!(
            response["config"].is_object(),
            "Config should include provider settings"
        );
        assert!(
            response["failover_chain"].is_array(),
            "Config should include failover chain"
        );

        // Test updating configuration
        let update_config = json!({
            "provider": "openai",
            "config": {
                "model": "gpt-4",
                "temperature": 0.3,
                "max_tokens": 2000,
                "timeout_seconds": 30
            },
            "failover_chain": ["anthropic", "local"]
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/llm/config",
            Some(update_config),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Config update should succeed");
        assert_eq!(
            response["updated"], true,
            "Should confirm configuration update"
        );
        assert_eq!(
            response["validated"], true,
            "Should confirm configuration validation"
        );
    }

    /// Test LLM failover chain configuration
    ///
    /// Expected behavior:
    /// - Support configuring multiple providers in failover order
    /// - Automatically switch on provider failure
    /// - Track failover events and statistics
    #[tokio::test]
    async fn test_llm_failover_chain() {
        let app = create_test_app();

        let failover_config = json!({
            "failover_enabled": true,
            "primary_provider": "openai",
            "failover_chain": ["anthropic", "local"],
            "failover_conditions": {
                "timeout_seconds": 30,
                "max_retries": 3,
                "error_types": ["timeout", "rate_limit", "service_unavailable"]
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/llm/config", Some(failover_config)).await;

        // This test will FAIL until failover functionality is implemented
        assert_eq!(status, StatusCode::OK, "Failover config should be accepted");
        assert_eq!(response["failover_enabled"], true, "Should enable failover");
        assert!(
            response["failover_chain"].is_array(),
            "Should set failover chain"
        );

        // TODO(P1): Test actual failover behavior
        // TEST PLAN: Simulate provider failures and verify failover
        // IMPLEMENTATION:
        //   1. Mock primary provider to return errors
        //   2. Verify system switches to secondary provider
        //   3. Test failover chain order is respected
        //   4. Validate metrics track failover events
        //   5. Test recovery back to primary when available
        // DEPENDENCIES: Requires provider simulation/mocking infrastructure
        // EFFORT: High (8-10 hours for comprehensive failover testing)
        // PRIORITY: Important for reliability validation
        // BLOCKER: Requires failover implementation first
        // This would require more complex integration testing with provider simulation
    }

    /// Test LLM provider health and status monitoring
    ///
    /// Expected behavior:
    /// - Monitor provider availability and response times
    /// - Track usage statistics and quotas
    /// - Provide health check endpoints for each provider
    #[tokio::test]
    async fn test_llm_provider_health_monitoring() {
        let app = create_test_app();

        let (status, response) = make_json_request(
            app,
            "GET",
            "/api/v1/llm/providers?include_health=true",
            None,
        )
        .await;

        // This test will FAIL until health monitoring is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Provider list with health should work"
        );

        let providers = response["providers"].as_array().unwrap();
        for provider in providers {
            assert!(
                provider["health"].is_object(),
                "Each provider should include health info"
            );
            let health = &provider["health"];
            assert!(health["status"].is_string(), "Health should include status");
            assert!(
                health["last_check"].is_string(),
                "Health should include last check time"
            );
            assert!(
                health["response_time_ms"].is_number(),
                "Health should include response time"
            );
        }
    }
}

/// Tests for Advanced Chunking Configuration API functionality
///
/// These tests validate advanced text chunking features that should:
/// - Support multiple chunking strategies (topic-based, sliding window)
/// - Allow configuration of chunking parameters
/// - Meet performance requirements (<200ms)
/// - Integrate with existing crawl endpoints
#[cfg(test)]
mod advanced_chunking_tests {
    use super::*;
    use test_utils::*;

    /// Test chunking strategy parameter in crawl requests
    ///
    /// Expected behavior:
    /// - Accept chunking_mode parameter in /crawl endpoint
    /// - Apply specified chunking strategy to extracted content
    /// - Return chunked content with metadata about chunking applied
    #[tokio::test]
    async fn test_crawl_with_chunking_strategy() {
        let app = create_test_app();

        let crawl_request = json!({
            "urls": ["https://example.com/long-article"],
            "chunking_config": {
                "chunking_mode": "topic",
                "chunk_size": 1000,
                "overlap_size": 100,
                "min_chunk_size": 200
            },
            "extraction_config": {
                "include_metadata": true,
                "preserve_formatting": false
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/crawl", Some(crawl_request)).await;

        // This test will FAIL until chunking integration is implemented
        assert_eq!(status, StatusCode::OK, "Crawl with chunking should succeed");

        assert!(
            response["results"].is_array(),
            "Should return crawl results"
        );
        let results = response["results"].as_array().unwrap();

        for result in results {
            assert!(
                result["chunks"].is_array(),
                "Each result should contain chunks"
            );
            assert!(
                result["chunking_metadata"].is_object(),
                "Should include chunking metadata"
            );

            let chunks = result["chunks"].as_array().unwrap();
            assert!(!chunks.is_empty(), "Should produce at least one chunk");

            // Validate chunk structure
            for chunk in chunks {
                assert!(chunk["content"].is_string(), "Chunk should have content");
                assert!(chunk["chunk_index"].is_number(), "Chunk should have index");
                assert!(
                    chunk["start_offset"].is_number(),
                    "Chunk should have start offset"
                );
                assert!(
                    chunk["end_offset"].is_number(),
                    "Chunk should have end offset"
                );
                assert!(
                    chunk["topic_score"].is_number(),
                    "Topic chunking should include topic score"
                );
            }
        }
    }

    /// Test topic-based chunking with TextTiling algorithm
    ///
    /// Expected behavior:
    /// - Use TextTiling or similar algorithm for topic boundary detection
    /// - Produce semantically coherent chunks
    /// - Include topic coherence scores for each chunk
    #[tokio::test]
    async fn test_topic_based_chunking() {
        let app = create_test_app();

        let chunking_request = json!({
            "content": sample_long_text(),
            "chunking_mode": "topic",
            "algorithm": "texttiling",
            "parameters": {
                "w": 20,           // Window size for TextTiling
                "k": 6,            // Number of sentences per window
                "similarity_threshold": 0.6
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/content/chunk", Some(chunking_request)).await;

        // This test will FAIL until topic chunking endpoint is implemented
        assert_eq!(status, StatusCode::OK, "Topic chunking should succeed");

        assert!(response["chunks"].is_array(), "Should return chunks array");
        let chunks = response["chunks"].as_array().unwrap();
        assert!(
            chunks.len() >= 2,
            "Should produce multiple topic-based chunks"
        );

        // Validate topic coherence
        for chunk in chunks {
            assert!(
                chunk["topic_score"].as_f64().unwrap() > 0.0,
                "Should have positive topic score"
            );
            assert!(
                chunk["boundary_strength"].is_number(),
                "Should include boundary strength"
            );
            assert!(
                chunk["keywords"].is_array(),
                "Should extract keywords for each chunk"
            );
        }

        // Check that chunks follow logical topic boundaries
        assert!(
            response["algorithm_metadata"].is_object(),
            "Should include algorithm metadata"
        );
        let metadata = &response["algorithm_metadata"];
        assert_eq!(
            metadata["algorithm"], "texttiling",
            "Should confirm algorithm used"
        );
        assert!(
            metadata["boundary_detection_stats"].is_object(),
            "Should include detection stats"
        );
    }

    /// Test sliding window chunking strategy
    ///
    /// Expected behavior:
    /// - Create overlapping chunks with specified window size
    /// - Maintain consistent chunk sizes (except possibly the last chunk)
    /// - Include proper overlap between adjacent chunks
    #[tokio::test]
    async fn test_sliding_window_chunking() {
        let app = create_test_app();

        let chunking_request = json!({
            "content": sample_long_text(),
            "chunking_mode": "sliding",
            "parameters": {
                "window_size": 500,
                "overlap_size": 100,
                "step_size": 400,        // window_size - overlap_size
                "preserve_sentences": true
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/content/chunk", Some(chunking_request)).await;

        // This test will FAIL until sliding window chunking is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Sliding window chunking should succeed"
        );

        let chunks = response["chunks"].as_array().unwrap();
        assert!(
            chunks.len() >= 2,
            "Should produce multiple overlapping chunks"
        );

        // Validate chunk sizes and overlaps
        for (i, chunk) in chunks.iter().enumerate() {
            let _content = chunk["content"].as_str().unwrap();
            let start_offset = chunk["start_offset"].as_u64().unwrap();
            let end_offset = chunk["end_offset"].as_u64().unwrap();

            // Check chunk size (allow some flexibility for sentence boundaries)
            let chunk_size = end_offset - start_offset;
            assert!(chunk_size >= 400, "Chunk should be at least min size");
            assert!(chunk_size <= 600, "Chunk should not exceed max size");

            // Check overlap with next chunk (except for last chunk)
            if i < chunks.len() - 1 {
                let next_chunk = &chunks[i + 1];
                let next_start = next_chunk["start_offset"].as_u64().unwrap();
                let overlap = end_offset - next_start;
                assert!(overlap >= 80, "Should have minimum overlap");
                assert!(overlap <= 120, "Should not exceed maximum overlap");
            }
        }
    }

    /// Test chunking performance requirements (<200ms)
    ///
    /// Expected behavior:
    /// - Process standard document sizes within 200ms
    /// - Scale reasonably with document length
    /// - Provide performance metrics in response
    #[tokio::test]
    async fn test_chunking_performance() {
        let app = create_test_app();

        // Generate larger test content
        let large_content = sample_long_text().repeat(10); // ~5KB of text

        let chunking_request = json!({
            "content": large_content,
            "chunking_mode": "sliding",
            "parameters": {
                "window_size": 1000,
                "overlap_size": 200
            },
            "include_performance_metrics": true
        });

        let start_time = std::time::Instant::now();

        let (status, response) =
            make_json_request(app, "POST", "/api/v1/content/chunk", Some(chunking_request)).await;

        let elapsed = start_time.elapsed();

        // This test will FAIL until chunking performance meets requirements
        assert_eq!(status, StatusCode::OK, "Chunking should succeed");
        assert!(
            elapsed.as_millis() < 200,
            "Chunking should complete in under 200ms"
        );

        // Validate performance metrics in response
        assert!(
            response["performance"].is_object(),
            "Should include performance metrics"
        );
        let perf = &response["performance"];
        assert!(
            perf["processing_time_ms"].is_number(),
            "Should include processing time"
        );
        assert!(
            perf["chunks_per_second"].is_number(),
            "Should include throughput metric"
        );

        let processing_time = perf["processing_time_ms"].as_f64().unwrap();
        assert!(
            processing_time < 200.0,
            "Reported processing time should be under 200ms"
        );
    }

    /// Test chunking configuration validation and edge cases
    ///
    /// Expected behavior:
    /// - Validate chunking parameters for logical consistency
    /// - Handle edge cases like very short content
    /// - Provide helpful error messages for invalid configurations
    #[tokio::test]
    async fn test_chunking_configuration_validation() {
        let app = create_test_app();

        // Test invalid chunking mode
        let invalid_mode_request = json!({
            "content": "Short content",
            "chunking_mode": "invalid_mode"
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/content/chunk",
            Some(invalid_mode_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Should reject invalid chunking mode"
        );
        assert!(
            response["error"]
                .as_str()
                .unwrap()
                .contains("chunking_mode"),
            "Error should mention chunking mode"
        );

        // Test invalid parameters (overlap larger than window)
        let invalid_params_request = json!({
            "content": sample_long_text(),
            "chunking_mode": "sliding",
            "parameters": {
                "window_size": 100,
                "overlap_size": 200  // Invalid: overlap > window
            }
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/content/chunk",
            Some(invalid_params_request),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "Should reject invalid parameters"
        );
        assert!(
            response["validation_errors"].is_array(),
            "Should include validation errors"
        );

        // Test very short content
        let short_content_request = json!({
            "content": "Too short",
            "chunking_mode": "topic"
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/content/chunk",
            Some(short_content_request),
        )
        .await;

        // Should handle gracefully, possibly returning single chunk
        assert_eq!(
            status,
            StatusCode::OK,
            "Should handle short content gracefully"
        );
        let chunks = response["chunks"].as_array().unwrap();
        assert_eq!(chunks.len(), 1, "Short content should produce single chunk");
    }

    /// Test chunking integration with existing extraction pipeline
    ///
    /// Expected behavior:
    /// - Apply chunking to extracted content from crawl operations
    /// - Preserve extraction metadata across chunks
    /// - Support chunking in streaming operations
    #[tokio::test]
    async fn test_chunking_pipeline_integration() {
        let app = create_test_app();

        let integrated_request = json!({
            "urls": ["https://example.com/article"],
            "extraction_config": {
                "extract_content": true,
                "extract_links": true,
                "extract_images": false
            },
            "chunking_config": {
                "chunking_mode": "topic",
                "chunk_size": 800,
                "preserve_metadata": true
            },
            "output_format": "enhanced"
        });

        let (status, response) =
            make_json_request(app, "POST", "/crawl", Some(integrated_request)).await;

        // This test will FAIL until pipeline integration is complete
        assert_eq!(
            status,
            StatusCode::OK,
            "Integrated crawl with chunking should succeed"
        );

        let results = response["results"].as_array().unwrap();
        for result in results {
            // Should include both extraction and chunking results
            assert!(result["url"].is_string(), "Should preserve URL metadata");
            assert!(
                result["extracted_content"].is_object(),
                "Should include extraction results"
            );
            assert!(
                result["chunks"].is_array(),
                "Should include chunked content"
            );
            assert!(
                result["links"].is_array(),
                "Should preserve extracted links"
            );

            // Verify chunks maintain source metadata
            let chunks = result["chunks"].as_array().unwrap();
            for chunk in chunks {
                assert!(
                    chunk["source_url"].is_string(),
                    "Chunks should include source URL"
                );
                assert!(
                    chunk["extraction_metadata"].is_object(),
                    "Chunks should preserve extraction metadata"
                );
            }
        }
    }

    /// Test chunking with different content types and formats
    ///
    /// Expected behavior:
    /// - Handle different content types (HTML, markdown, plain text)
    /// - Preserve formatting when requested
    /// - Apply appropriate preprocessing based on content type
    #[tokio::test]
    async fn test_chunking_content_types() {
        let app = create_test_app();

        // Test HTML content chunking
        let html_request = json!({
            "content": "<html><body><h1>Title</h1><p>Paragraph 1</p><p>Paragraph 2</p></body></html>",
            "content_type": "text/html",
            "chunking_mode": "sliding",
            "parameters": {
                "window_size": 200,
                "preserve_html_structure": true
            }
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/content/chunk",
            Some(html_request),
        )
        .await;

        // This test will FAIL until content type handling is implemented
        assert_eq!(status, StatusCode::OK, "HTML chunking should succeed");

        let chunks = response["chunks"].as_array().unwrap();
        for chunk in chunks {
            assert!(
                chunk["content_type"].is_string(),
                "Chunks should preserve content type"
            );
            assert!(
                chunk["html_metadata"].is_object(),
                "HTML chunks should include structural metadata"
            );
        }

        // Test Markdown content chunking
        let markdown_request = json!({
            "content": "# Heading 1\n\nContent paragraph.\n\n## Heading 2\n\nMore content here.",
            "content_type": "text/markdown",
            "chunking_mode": "topic",
            "parameters": {
                "respect_markdown_structure": true
            }
        });

        let (status, response) = make_json_request(
            app.clone(),
            "POST",
            "/api/v1/content/chunk",
            Some(markdown_request),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Markdown chunking should succeed");

        let chunks = response["chunks"].as_array().unwrap();
        for chunk in chunks {
            assert!(
                chunk["markdown_metadata"].is_object(),
                "Markdown chunks should include structural metadata"
            );
        }
    }
}

/// Integration tests that combine multiple API features
///
/// These tests validate that the different API components work together correctly:
/// - Table extraction with LLM analysis
/// - Chunked content processing with LLM providers
/// - End-to-end workflows combining all features
#[cfg(test)]
mod integration_workflow_tests {
    use super::*;
    use test_utils::*;

    /// Test end-to-end workflow: crawl -> extract tables -> analyze with LLM
    ///
    /// Expected behavior:
    /// - Crawl a page with tables
    /// - Extract tables automatically
    /// - Use LLM to analyze table content
    /// - Return structured analysis results
    #[tokio::test]
    async fn test_table_extraction_llm_analysis_workflow() {
        let app = create_test_app();

        let workflow_request = json!({
            "urls": ["https://example.com/financial-reports"],
            "extraction_config": {
                "extract_tables": true,
                "table_analysis": {
                    "enabled": true,
                    "analysis_type": "summary",
                    "llm_provider": "anthropic"
                }
            },
            "chunking_config": {
                "chunking_mode": "topic",
                "chunk_size": 1500
            }
        });

        let (status, response) =
            make_json_request(app, "POST", "/crawl", Some(workflow_request)).await;

        // This test will FAIL until the integrated workflow is implemented
        assert_eq!(status, StatusCode::OK, "Integrated workflow should succeed");

        let results = response["results"].as_array().unwrap();
        for result in results {
            if !result["tables"].as_array().unwrap().is_empty() {
                assert!(
                    result["table_analysis"].is_object(),
                    "Should include LLM table analysis"
                );
                let analysis = &result["table_analysis"];
                assert!(
                    analysis["summary"].is_string(),
                    "Should include table summary"
                );
                assert!(analysis["insights"].is_array(), "Should include insights");
                assert!(
                    analysis["llm_provider_used"].is_string(),
                    "Should track which LLM was used"
                );
            }
        }
    }

    /// Test content chunking with LLM-powered topic analysis
    ///
    /// Expected behavior:
    /// - Apply advanced chunking to long content
    /// - Use LLM to improve topic boundary detection
    /// - Provide topic labels and summaries for each chunk
    #[tokio::test]
    async fn test_llm_enhanced_chunking_workflow() {
        let app = create_test_app();

        let enhanced_chunking_request = json!({
            "content": sample_long_text(),
            "chunking_mode": "llm_enhanced",
            "llm_config": {
                "provider": "openai",
                "model": "gpt-4",
                "enhancement_type": "topic_detection"
            },
            "parameters": {
                "target_chunk_size": 1000,
                "max_chunks": 10,
                "include_summaries": true
            }
        });

        let (status, response) = make_json_request(
            app,
            "POST",
            "/api/v1/content/chunk",
            Some(enhanced_chunking_request),
        )
        .await;

        // This test will FAIL until LLM-enhanced chunking is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "LLM-enhanced chunking should succeed"
        );

        let chunks = response["chunks"].as_array().unwrap();
        for chunk in chunks {
            assert!(
                chunk["topic_label"].is_string(),
                "LLM should provide topic labels"
            );
            assert!(
                chunk["summary"].is_string(),
                "LLM should provide chunk summaries"
            );
            assert!(
                chunk["coherence_score"].is_number(),
                "Should include LLM-computed coherence score"
            );
        }
    }

    /// Test failover scenario: primary LLM fails, switches to backup
    ///
    /// Expected behavior:
    /// - Start request with primary LLM provider
    /// - Simulate failure of primary provider
    /// - Automatically switch to backup provider
    /// - Complete request successfully with backup
    /// - Log failover event for monitoring
    #[tokio::test]
    async fn test_llm_failover_scenario() {
        let app = create_test_app();

        // This test would require more complex setup to simulate provider failures
        // For now, we'll test the configuration and expect the failover logic to exist

        let failover_request = json!({
            "content": "Analyze this content",
            "analysis_type": "sentiment",
            "llm_config": {
                "primary_provider": "openai",
                "failover_chain": ["anthropic", "local"],
                "failover_on_errors": ["timeout", "rate_limit", "service_unavailable"]
            }
        });

        let (status, response) = make_json_request(
            app,
            "POST",
            "/api/v1/content/analyze",
            Some(failover_request),
        )
        .await;

        // This test will FAIL until failover logic is implemented
        assert_eq!(
            status,
            StatusCode::OK,
            "Analysis with failover config should succeed"
        );

        // The response should include information about which provider was actually used
        assert!(
            response["provider_used"].is_string(),
            "Should indicate which provider was used"
        );
        assert_eq!(
            response["failover_triggered"], true,
            "Should indicate if failover was triggered"
        );

        if response["failover_triggered"] == true {
            assert!(
                response["failover_events"].is_array(),
                "Should log failover events"
            );
        }
    }

    /// Test performance under load: concurrent requests with different configurations
    ///
    /// Expected behavior:
    /// - Handle multiple concurrent requests efficiently
    /// - Maintain performance requirements across all endpoints
    /// - Properly manage resources and connections
    #[tokio::test]
    async fn test_concurrent_request_performance() {
        let app = create_test_app();

        // This test would spawn multiple concurrent requests
        // For TDD purposes, we'll define the expected behavior

        let concurrent_requests = [
            json!({"urls": ["https://example1.com"], "chunking_mode": "topic"}),
            json!({"urls": ["https://example2.com"], "chunking_mode": "sliding"}),
            json!({"html_content": sample_html_with_tables(), "extract_tables": true}),
        ];

        // In a real implementation, this would use tokio::spawn to run requests concurrently
        // and measure total time vs sequential time

        let start_time = std::time::Instant::now();

        for (i, request) in concurrent_requests.iter().enumerate() {
            let (status, _response) =
                make_json_request(app.clone(), "POST", "/crawl", Some(request.clone())).await;

            // Each individual request should succeed
            assert_eq!(
                status,
                StatusCode::OK,
                "Concurrent request {} should succeed",
                i
            );
        }

        let total_time = start_time.elapsed();

        // This assertion will FAIL until proper concurrent handling is implemented
        assert!(
            total_time.as_millis() < 1000,
            "Concurrent requests should complete efficiently"
        );
    }
}
