//! Integration tests for core interactions between components
//! Tests the interaction between riptide-core, riptide-intelligence, and HTML extraction

use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::time::timeout;

use riptide_intelligence::*;
use riptide_intelligence::mock_provider::*;
use riptide_intelligence::timeout::*;
use riptide_intelligence::circuit_breaker::*;
use riptide_intelligence::fallback::*;
use riptide_intelligence::registry::*;

use riptide_core::strategies::extraction::*;
use riptide_core::strategies::extraction::css_json::*;
use riptide_core::strategies::extraction::regex::*;
use riptide_core::strategies::RegexPattern;

/// Test module for core-intelligence integration
mod core_intelligence_integration {
    use super::*;

    #[tokio::test]
    async fn test_intelligence_client_with_html_extraction() {
        // Set up intelligence client
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        registry.register_provider("default", provider).unwrap();
        let client = IntelligenceClient::new(registry, "default");

        // HTML content to process
        let html = r#"
        <html>
        <head>
            <title>AI-Enhanced Extraction Test</title>
            <meta name="description" content="Testing AI-enhanced content extraction">
        </head>
        <body>
            <article class="content">
                <h1>Main Content</h1>
                <p>This content will be extracted and potentially enhanced by AI.</p>
                <p>Contact us at ai@example.com for more information.</p>
            </article>
        </body>
        </html>
        "#;

        // First extract with CSS
        let css_result = extract_default(html, "https://example.com").await.unwrap();
        assert_eq!(css_result.title, "AI-Enhanced Extraction Test");
        assert!(css_result.content.contains("This content will be extracted"));

        // Test AI provider capabilities
        let capabilities = client.capabilities("default").unwrap();
        assert!(capabilities.supports_embeddings);
        assert_eq!(capabilities.provider_name, "mock");

        // Test AI completion with extracted content
        let request = CompletionRequest::new(
            "mock-gpt-3.5",
            vec![Message::user(format!("Summarize this content: {}", css_result.content))]
        );
        let ai_response = client.complete(request).await.unwrap();
        assert!(ai_response.content.contains("Mock response"));

        // Test embedding generation
        let embedding = client.embed(&css_result.content).await.unwrap();
        assert_eq!(embedding.len(), 768);
    }

    #[tokio::test]
    async fn test_intelligence_with_timeout_and_fallback() {
        // Set up multiple providers with different behaviors
        let registry = LlmRegistry::new();
        
        let slow_provider = Arc::new(MockLlmProvider::with_name("slow").with_delay(2000));
        let fast_provider = Arc::new(MockLlmProvider::with_name("fast"));
        let failing_provider = Arc::new(MockLlmProvider::with_name("failing").always_fail());

        // Wrap providers with timeout
        let timeout_slow = Arc::new(TimeoutWrapper::with_timeout(slow_provider, Duration::from_millis(500)));
        let timeout_fast = Arc::new(TimeoutWrapper::new(fast_provider));
        let timeout_failing = Arc::new(TimeoutWrapper::new(failing_provider));

        // Create fallback chain
        let providers = vec![
            timeout_failing as Arc<dyn LlmProvider>,
            timeout_slow as Arc<dyn LlmProvider>,
            timeout_fast as Arc<dyn LlmProvider>,
        ];
        let fallback_chain = Arc::new(FallbackChain::new(providers, FallbackStrategy::FirstAvailable));

        registry.register_provider("fallback", fallback_chain).unwrap();
        let client = IntelligenceClient::new(registry, "fallback");

        // Test that fallback works correctly
        let request = CompletionRequest::new("mock", vec![Message::user("test")]);
        let result = client.complete(request).await;
        
        assert!(result.is_ok(), "Fallback chain should have succeeded with fast provider");
    }

    #[tokio::test]
    async fn test_circuit_breaker_with_html_processing() {
        // Set up failing provider with circuit breaker
        let failing_provider = Arc::new(MockLlmProvider::new().fail_after(2));
        let config = CircuitBreakerConfig {
            consecutive_failure_threshold: 3,
            ..Default::default()
        };
        let circuit_breaker = Arc::new(CircuitBreaker::with_config(failing_provider, config));

        let registry = LlmRegistry::new();
        registry.register_provider("circuit", circuit_breaker.clone()).unwrap();
        let client = IntelligenceClient::new(registry, "circuit");

        // Process HTML content
        let html = "<html><head><title>Circuit Test</title></head><body><p>Test content</p></body></html>";
        let css_result = extract_default(html, "https://example.com").await.unwrap();

        // Make requests until circuit opens
        for i in 0..5 {
            let request = CompletionRequest::new(
                "mock",
                vec![Message::user(format!("Process content {}: {}", i, css_result.content))]
            );
            let result = client.complete(request).await;
            
            if i < 2 {
                assert!(result.is_ok(), "Request {} should succeed", i);
            } else {
                // After circuit opens, should get circuit breaker errors
                if circuit_breaker.state() == CircuitState::Open {
                    assert!(matches!(result, Err(IntelligenceError::CircuitOpen { .. })));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_html_processing_with_ai() {
        // Set up intelligence client
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        registry.register_provider("default", provider).unwrap();
        let client = Arc::new(IntelligenceClient::new(registry, "default"));

        // Different HTML documents to process
        let html_docs = vec![
            ("doc1", "<html><head><title>Document 1</title></head><body><p>Content 1</p></body></html>"),
            ("doc2", "<html><head><title>Document 2</title></head><body><p>Content 2</p></body></html>"),
            ("doc3", "<html><head><title>Document 3</title></head><body><p>Content 3</p></body></html>"),
        ];

        let mut handles = vec![];
        
        for (name, html) in html_docs {
            let client_clone = client.clone();
            let html_str = html.to_string();
            let name_str = name.to_string();
            
            let handle = tokio::spawn(async move {
                // Extract HTML content
                let css_result = extract_default(&html_str, &format!("https://example.com/{}", name_str))
                    .await.unwrap();
                
                // Process with AI
                let request = CompletionRequest::new(
                    "mock-gpt-3.5",
                    vec![Message::user(format!("Analyze: {}", css_result.content))]
                );
                let ai_result = client_clone.complete(request).await.unwrap();
                
                (name_str, css_result, ai_result)
            });
            
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;
        
        assert_eq!(results.len(), 3);
        for result in results {
            let (name, css_result, ai_result) = result.unwrap();
            assert!(css_result.title.contains("Document"));
            assert!(ai_result.content.contains("Mock response"));
            println!("Processed {}: {} -> {}", name, css_result.title, ai_result.content.chars().take(50).collect::<String>());
        }
    }
}

/// Test module for end-to-end extraction workflows
mod end_to_end_workflows {
    use super::*;

    #[tokio::test]
    async fn test_multi_strategy_extraction_workflow() {
        let html = r#"
        <html>
        <head>
            <title>Multi-Strategy Test Page</title>
            <meta name="description" content="Testing multiple extraction strategies">
            <meta name="author" content="Test Author">
        </head>
        <body>
            <article class="main-content">
                <h1>Multi-Strategy Extraction</h1>
                <div class="content">
                    <p>This page will be processed using multiple extraction strategies.</p>
                    <p>Contact information: support@example.com, phone: 123-456-7890</p>
                    <p>Website: https://www.example.com</p>
                </div>
                <div class="metadata">
                    <span class="author">Test Author</span>
                    <time class="published">2023-01-01</time>
                    <div class="tags">
                        <span class="tag">testing</span>
                        <span class="tag">extraction</span>
                    </div>
                </div>
            </article>
        </body>
        </html>
        "#;

        // Strategy 1: CSS extraction
        let css_result = extract_default(html, "https://example.com").await.unwrap();
        assert_eq!(css_result.strategy_used, "css_json");
        assert_eq!(css_result.title, "Multi-Strategy Test Page");
        assert!(css_result.content.contains("multiple extraction strategies"));

        // Strategy 2: Regex extraction
        let regex_patterns = vec![
            RegexPattern {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
                field: "emails".to_string(),
                required: false,
            },
            RegexPattern {
                name: "phone".to_string(),
                pattern: r"\b\d{3}-\d{3}-\d{4}\b".to_string(),
                field: "phones".to_string(),
                required: false,
            },
            RegexPattern {
                name: "url".to_string(),
                pattern: r"https?://[^\s<>]+".to_string(),
                field: "urls".to_string(),
                required: false,
            },
        ];
        
        let regex_result = extract(html, "https://example.com", &regex_patterns).await.unwrap();
        assert_eq!(regex_result.strategy_used, "regex");
        assert_eq!(regex_result.title, "Multi-Strategy Test Page");
        
        // Verify regex extracted contact information
        assert!(regex_result.content.contains("support@example.com"));
        assert!(regex_result.content.contains("123-456-7890"));
        assert!(regex_result.content.contains("https://www.example.com"));

        // Strategy 3: Custom selectors
        let mut custom_selectors = HashMap::new();
        custom_selectors.insert("title".to_string(), "h1".to_string());
        custom_selectors.insert("author".to_string(), ".author".to_string());
        custom_selectors.insert("content".to_string(), ".content".to_string());
        custom_selectors.insert("tags".to_string(), ".tag".to_string());
        custom_selectors.insert("date".to_string(), ".published".to_string());

        let custom_result = extract(html, "https://example.com", &custom_selectors).await.unwrap();
        assert_eq!(custom_result.title, "Multi-Strategy Extraction");
        assert!(custom_result.content.contains("multiple extraction strategies"));

        // Compare results
        println!("CSS confidence: {}", css_result.extraction_confidence);
        println!("Regex confidence: {}", regex_result.extraction_confidence);
        println!("Custom confidence: {}", custom_result.extraction_confidence);
        
        // All strategies should successfully extract content
        assert!(!css_result.content.is_empty());
        assert!(!regex_result.content.is_empty());
        assert!(!custom_result.content.is_empty());
    }

    #[tokio::test]
    async fn test_extraction_with_ai_enhancement() {
        // Set up AI provider
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        registry.register_provider("enhancer", provider).unwrap();
        let client = IntelligenceClient::new(registry, "enhancer");

        // HTML with structured data
        let html = r#"
        <html>
        <head>
            <title>Product Review: Smartphone XYZ</title>
            <meta name="description" content="Comprehensive review of the latest smartphone">
        </head>
        <body>
            <article>
                <h1>Smartphone XYZ Review</h1>
                <div class="rating">4.5/5 stars</div>
                <div class="content">
                    <h2>Design and Build Quality</h2>
                    <p>The smartphone features a premium aluminum body with excellent build quality.</p>
                    
                    <h2>Performance</h2>
                    <p>Powered by the latest processor, performance is exceptional for both daily tasks and gaming.</p>
                    
                    <h2>Camera</h2>
                    <p>The camera system produces stunning photos with excellent low-light performance.</p>
                    
                    <h2>Battery Life</h2>
                    <p>Battery easily lasts a full day with moderate to heavy usage.</p>
                </div>
                <div class="pros-cons">
                    <h3>Pros:</h3>
                    <ul>
                        <li>Excellent build quality</li>
                        <li>Fast performance</li>
                        <li>Great camera</li>
                    </ul>
                    <h3>Cons:</h3>
                    <ul>
                        <li>Expensive</li>
                        <li>No headphone jack</li>
                    </ul>
                </div>
            </article>
        </body>
        </html>
        "#;

        // Extract content using CSS
        let extracted = extract_default(html, "https://reviews.example.com/smartphone-xyz").await.unwrap();
        
        // Simulate AI enhancement workflow
        let enhancement_tasks = vec![
            ("summarize", "Provide a brief summary of this product review"),
            ("sentiment", "Analyze the sentiment of this review (positive/negative/neutral)"),
            ("key_points", "Extract the main pros and cons mentioned in the review"),
        ];

        for (task_name, task_prompt) in enhancement_tasks {
            let request = CompletionRequest::new(
                "mock-gpt-3.5",
                vec![
                    Message::system(task_prompt),
                    Message::user(extracted.content.clone())
                ]
            );
            
            let ai_result = client.complete(request).await.unwrap();
            assert!(ai_result.content.contains("Mock response"));
            assert!(ai_result.usage.total_tokens > 0);
            
            println!("{}: {}", task_name, ai_result.content.chars().take(100).collect::<String>());
        }

        // Test embedding generation for semantic search
        let embedding = client.embed(&extracted.content).await.unwrap();
        assert_eq!(embedding.len(), 768);
        
        // Simulate storing for semantic search
        let document_embedding = EmbeddedDocument {
            id: "smartphone-xyz-review".to_string(),
            url: extracted.url,
            title: extracted.title,
            content: extracted.content,
            embedding,
            metadata: std::collections::HashMap::new(),
        };
        
        assert!(!document_embedding.id.is_empty());
        assert!(!document_embedding.content.is_empty());
        assert_eq!(document_embedding.embedding.len(), 768);
    }

    #[derive(Debug, Clone)]
    struct EmbeddedDocument {
        id: String,
        url: String,
        title: String,
        content: String,
        embedding: Vec<f32>,
        metadata: HashMap<String, String>,
    }

    #[tokio::test]
    async fn test_error_recovery_workflow() {
        // Set up providers with different failure modes
        let registry = LlmRegistry::new();
        
        // Provider that fails after 2 requests
        let failing_provider = Arc::new(MockLlmProvider::with_name("failing").fail_after(2));
        let timeout_failing = Arc::new(TimeoutWrapper::new(failing_provider));
        
        // Backup provider that works
        let backup_provider = Arc::new(MockLlmProvider::with_name("backup"));
        let timeout_backup = Arc::new(TimeoutWrapper::new(backup_provider));
        
        // Fallback chain
        let providers = vec![
            timeout_failing as Arc<dyn LlmProvider>,
            timeout_backup as Arc<dyn LlmProvider>,
        ];
        let fallback_chain = Arc::new(FallbackChain::new(providers, FallbackStrategy::FirstAvailable));
        
        registry.register_provider("resilient", fallback_chain).unwrap();
        let client = IntelligenceClient::new(registry, "resilient");

        // HTML documents to process
        let html_docs = vec![
            "<html><head><title>Doc 1</title></head><body><p>Content 1</p></body></html>",
            "<html><head><title>Doc 2</title></head><body><p>Content 2</p></body></html>",
            "<html><head><title>Doc 3</title></head><body><p>Content 3</p></body></html>",
            "<html><head><title>Doc 4</title></head><body><p>Content 4</p></body></html>",
        ];

        let mut all_succeeded = true;
        
        for (i, html) in html_docs.iter().enumerate() {
            // Extract content
            let extracted = extract_default(html, &format!("https://example.com/doc{}", i + 1))
                .await.unwrap();
            
            // Try to enhance with AI
            let request = CompletionRequest::new(
                "mock",
                vec![Message::user(format!("Enhance: {}", extracted.content))]
            );
            
            match client.complete(request).await {
                Ok(response) => {
                    println!("Document {}: Successfully enhanced", i + 1);
                    assert!(response.content.contains("Mock response"));
                },
                Err(e) => {
                    println!("Document {}: Failed to enhance: {}", i + 1, e);
                    all_succeeded = false;
                }
            }
        }
        
        // With fallback, most requests should succeed
        assert!(all_succeeded, "Fallback chain should handle provider failures");
    }
}

/// Test module for performance and scalability
mod performance_integration {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_high_throughput_extraction() {
        // Generate multiple HTML documents
        let html_template = |i: usize| format!(r#"
        <html>
        <head>
            <title>Document {}</title>
            <meta name="description" content="This is test document number {}">
        </head>
        <body>
            <article class="content">
                <h1>Content for Document {}</h1>
                <p>This is the main content for document number {}. It contains various information that needs to be extracted efficiently.</p>
                <div class="metadata">
                    <span class="author">Author {}</span>
                    <time class="date">2023-01-{:02}</time>
                </div>
            </article>
        </body>
        </html>
        "#, i, i, i, i, i % 10, (i % 28) + 1);

        let num_documents = 50;
        let start = Instant::now();
        
        let mut handles = vec![];
        
        for i in 0..num_documents {
            let html = html_template(i);
            let handle = tokio::spawn(async move {
                extract_default(&html, &format!("https://example.com/doc{}", i)).await
            });
            handles.push(handle);
        }
        
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();
        
        // Verify all extractions succeeded
        let mut successful = 0;
        for (i, result) in results.iter().enumerate() {
            match result {
                Ok(Ok(extracted)) => {
                    assert_eq!(extracted.title, format!("Document {}", i));
                    assert!(extracted.content.contains(&format!("document number {}", i)));
                    successful += 1;
                },
                _ => println!("Failed extraction for document {}", i),
            }
        }
        
        assert_eq!(successful, num_documents);
        
        // Performance check - should handle 50 documents in reasonable time
        let avg_time_per_doc = duration.as_millis() as f64 / num_documents as f64;
        println!("Processed {} documents in {}ms (avg: {:.2}ms per doc)", 
                num_documents, duration.as_millis(), avg_time_per_doc);
        
        assert!(avg_time_per_doc < 100.0, "Average extraction time should be under 100ms per document");
    }

    #[tokio::test]
    async fn test_memory_efficiency_with_large_content() {
        // Create a large HTML document
        let mut large_html = String::from(r#"
        <html>
        <head>
            <title>Large Content Test</title>
            <meta name="description" content="Testing memory efficiency with large content">
        </head>
        <body>
            <article class="content">
        "#);
        
        // Add substantial content
        for i in 0..500 {
            large_html.push_str(&format!(
                "<section id='section-{}'>\n", i
            ));
            large_html.push_str(&format!(
                "<h2>Section {} Title</h2>\n", i
            ));
            for j in 0..10 {
                large_html.push_str(&format!(
                    "<p>This is paragraph {} in section {}. It contains substantial text to test memory efficiency during extraction. The content includes various HTML structures and substantial text that needs to be processed efficiently without excessive memory usage.</p>\n",
                    j, i
                ));
            }
            large_html.push_str("</section>\n");
        }
        
        large_html.push_str("</article></body></html>");
        
        println!("Testing with HTML document of {} bytes", large_html.len());
        
        // Test extraction performance
        let start = Instant::now();
        let result = extract_default(&large_html, "https://example.com/large-content").await.unwrap();
        let duration = start.elapsed();
        
        assert_eq!(result.title, "Large Content Test");
        assert!(result.content.len() > 50000); // Should extract substantial content
        assert!(duration.as_millis() < 10000); // Should complete within 10 seconds
        
        println!("Extracted {} bytes of content in {}ms", result.content.len(), duration.as_millis());
        
        // Memory should be released after extraction
        drop(large_html);
        drop(result);
        
        // Force garbage collection if available
        // In a real test environment, you might check actual memory usage here
    }

    #[tokio::test]
    async fn test_concurrent_ai_processing() {
        // Set up intelligence client
        let registry = LlmRegistry::new();
        let provider = Arc::new(MockLlmProvider::new());
        registry.register_provider("concurrent", provider).unwrap();
        let client = Arc::new(IntelligenceClient::new(registry, "concurrent"));

        let num_concurrent = 20;
        let start = Instant::now();
        
        let mut handles = vec![];
        
        for i in 0..num_concurrent {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                // Extract some content
                let html = format!(
                    "<html><head><title>Concurrent Test {}</title></head><body><p>Content {}</p></body></html>",
                    i, i
                );
                let extracted = extract_default(&html, &format!("https://example.com/{}", i))
                    .await.unwrap();
                
                // Process with AI
                let request = CompletionRequest::new(
                    "mock-gpt-3.5",
                    vec![Message::user(format!("Process: {}", extracted.content))]
                );
                
                let ai_result = client_clone.complete(request).await.unwrap();
                
                // Generate embedding
                let embedding = client_clone.embed(&extracted.content).await.unwrap();
                
                (i, extracted, ai_result, embedding)
            });
            handles.push(handle);
        }
        
        let results = futures::future::join_all(handles).await;
        let duration = start.elapsed();
        
        // Verify all concurrent operations succeeded
        assert_eq!(results.len(), num_concurrent);
        for (i, result) in results.iter().enumerate() {
            let (doc_id, extracted, ai_result, embedding) = result.as_ref().unwrap();
            assert_eq!(*doc_id, i);
            assert_eq!(extracted.title, format!("Concurrent Test {}", i));
            assert!(ai_result.content.contains("Mock response"));
            assert_eq!(embedding.len(), 768);
        }
        
        let avg_time = duration.as_millis() as f64 / num_concurrent as f64;
        println!("Processed {} concurrent requests in {}ms (avg: {:.2}ms per request)", 
                num_concurrent, duration.as_millis(), avg_time);
        
        // Concurrent processing should be efficient
        assert!(avg_time < 50.0, "Average concurrent processing time should be under 50ms");
    }
}
