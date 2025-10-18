# Facade Workflow Examples

**Document Version:** 1.0
**Date:** 2025-10-18
**Purpose:** Practical examples of facade composition for common workflows

---

## Overview

This document provides practical, copy-paste ready examples of using the riptide-facade composition layer for common web scraping workflows.

---

## Example 1: Simple Content Extraction

### Use Case
Extract article content from a news website with automatic fallback strategies.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize facade
    let config = RiptideConfig::default()
        .with_user_agent("NewsBot/1.0")
        .with_timeout_secs(30);

    let scraper = ScraperFacade::new(config).await?;

    // Fetch and extract
    let html = scraper.fetch_html("https://example.com/article").await?;

    // Extract with ExtractionFacade
    let config = RiptideConfig::default();
    let extractor = ExtractionFacade::new(config).await?;

    let options = HtmlExtractionOptions {
        clean: true,
        include_metadata: true,
        extract_links: true,
        ..Default::default()
    };

    let data = extractor.extract_html(&html, "https://example.com/article", options).await?;

    println!("Title: {:?}", data.title);
    println!("Content length: {} chars", data.text.len());
    println!("Confidence: {:.2}", data.confidence);
    println!("Links found: {}", data.links.len());

    Ok(())
}
```

---

## Example 2: Browser-Based Dynamic Content

### Use Case
Extract content from JavaScript-heavy single-page application.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize browser facade
    let config = RiptideConfig::default();
    let browser = BrowserFacade::new(config.clone()).await?;
    let extractor = ExtractionFacade::new(config).await?;

    // Launch browser
    let session = browser.launch().await?;

    // Navigate to page
    browser.navigate(&session, "https://spa-example.com").await?;

    // Wait for dynamic content
    let actions = vec![
        BrowserAction::WaitForElement {
            selector: ".content-loaded".to_string(),
            timeout_ms: 5000,
        },
    ];
    browser.perform_actions(&session, &actions).await?;

    // Get rendered content
    let html = browser.get_content(&session).await?;

    // Extract
    let options = HtmlExtractionOptions {
        clean: true,
        as_markdown: true,
        ..Default::default()
    };

    let data = extractor.extract_html(&html, "https://spa-example.com", options).await?;

    println!("Extracted {} chars of content", data.text.len());
    if let Some(markdown) = data.markdown {
        println!("Markdown:\n{}", markdown);
    }

    // Cleanup
    browser.close(session).await?;

    Ok(())
}
```

---

## Example 3: Form Interaction and Extraction

### Use Case
Login to a site, navigate to protected content, and extract data.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let browser = BrowserFacade::new(config.clone()).await?;
    let extractor = ExtractionFacade::new(config).await?;

    let session = browser.launch().await?;

    // Navigate to login page
    browser.navigate(&session, "https://example.com/login").await?;

    // Perform login
    let login_actions = vec![
        BrowserAction::Type {
            selector: "#username".to_string(),
            text: "user@example.com".to_string(),
        },
        BrowserAction::Type {
            selector: "#password".to_string(),
            text: "secure_password".to_string(),
        },
        BrowserAction::Click {
            selector: "#submit-btn".to_string(),
        },
        BrowserAction::WaitForElement {
            selector: ".dashboard".to_string(),
            timeout_ms: 5000,
        },
    ];
    browser.perform_actions(&session, &login_actions).await?;

    // Navigate to protected content
    browser.navigate(&session, "https://example.com/dashboard/data").await?;

    // Extract data
    let html = browser.get_content(&session).await?;
    let data = extractor.extract_html(
        &html,
        "https://example.com/dashboard/data",
        HtmlExtractionOptions {
            clean: true,
            include_metadata: true,
            ..Default::default()
        },
    ).await?;

    println!("Protected content extracted: {} chars", data.text.len());

    browser.close(session).await?;

    Ok(())
}
```

---

## Example 4: Screenshot with OCR

### Use Case
Capture visual content and extract text from images.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let browser = BrowserFacade::new(config).await?;

    let session = browser.launch().await?;

    browser.navigate(&session, "https://charts-example.com").await?;

    // Wait for charts to render
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Take screenshot
    let screenshot_opts = ScreenshotOptions::default()
        .full_page(true)
        .format(ImageFormat::Png)
        .quality(90);

    let screenshot = browser.screenshot(&session, screenshot_opts).await?;

    // Save screenshot
    std::fs::write("chart.png", &screenshot)?;
    println!("Screenshot saved: {} bytes", screenshot.len());

    // Extract visible text
    let text = browser.get_text(&session).await?;
    println!("Visible text: {}", text);

    browser.close(session).await?;

    Ok(())
}
```

---

## Example 5: Batch URL Processing

### Use Case
Process multiple URLs in parallel with rate limiting.

```rust
use riptide_facade::prelude::*;
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls = vec![
        "https://example.com/page1",
        "https://example.com/page2",
        "https://example.com/page3",
        // ... up to 100 URLs
    ];

    let config = RiptideConfig::default()
        .with_timeout_secs(30);

    let scraper = Arc::new(ScraperFacade::new(config.clone()).await?);
    let extractor = Arc::new(ExtractionFacade::new(config).await?);

    // Process in parallel with concurrency limit
    let results = stream::iter(urls)
        .map(|url| {
            let scraper = Arc::clone(&scraper);
            let extractor = Arc::clone(&extractor);
            async move {
                // Fetch
                let html = scraper.fetch_html(url).await?;

                // Extract
                let data = extractor.extract_html(
                    &html,
                    url,
                    HtmlExtractionOptions::default(),
                ).await?;

                Ok::<_, RiptideError>((url, data))
            }
        })
        .buffer_unordered(5) // Limit concurrency to 5
        .collect::<Vec<_>>()
        .await;

    // Process results
    let mut successful = 0;
    let mut failed = 0;

    for result in results {
        match result {
            Ok((url, data)) => {
                successful += 1;
                println!("✓ {}: {} chars", url, data.text.len());
            }
            Err(e) => {
                failed += 1;
                println!("✗ Error: {}", e);
            }
        }
    }

    println!("\nSummary: {} successful, {} failed", successful, failed);

    Ok(())
}
```

---

## Example 6: Pipeline-Based Workflow

### Use Case
Use PipelineFacade for complex multi-stage workflow.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let pipeline_facade = PipelineFacade::new(config).await?;

    // Build custom pipeline
    let pipeline = pipeline_facade
        .builder()
        // Stage 1: Fetch content
        .add_stage(PipelineStage::Fetch {
            url: "https://example.com/api/data".to_string(),
            options: FetchOptions::default(),
        })
        // Stage 2: Extract with strategy
        .add_stage(PipelineStage::Extract {
            strategy: ExtractionStrategy::Json,
        })
        // Stage 3: Transform data
        .add_stage(PipelineStage::Transform {
            transformer: Arc::new(JsonTransformer::new()),
        })
        // Stage 4: Validate output
        .add_stage(PipelineStage::Validate {
            validator: Arc::new(SchemaValidator::new()),
        })
        // Stage 5: Store result
        .add_stage(PipelineStage::Store {
            destination: StoreDestination::File("output.json".to_string()),
        })
        // Configure pipeline
        .with_retry(3)
        .with_caching(true)
        .with_parallelism(2)
        .build()
        .await?;

    // Execute pipeline
    let result = pipeline_facade.execute(pipeline).await?;

    println!("Pipeline completed in {:?}", result.total_duration);
    println!("Stages completed: {}/{}", result.stages_completed, 5);

    // Check stage results
    for (idx, stage) in result.stage_results.iter().enumerate() {
        println!(
            "Stage {}: {} - {:?} ({}ms)",
            idx + 1,
            stage.stage_name,
            stage.status,
            stage.duration.as_millis()
        );
    }

    Ok(())
}

// Custom transformer implementation
#[derive(Debug)]
struct JsonTransformer;

impl JsonTransformer {
    fn new() -> Self {
        Self
    }
}

impl Transformer for JsonTransformer {
    fn transform(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = RiptideResult<serde_json::Value>> + Send + '_>> {
        Box::pin(async move {
            // Transform JSON structure
            let mut output = serde_json::Map::new();
            if let Some(obj) = input.as_object() {
                // Extract specific fields
                if let Some(title) = obj.get("title") {
                    output.insert("title".to_string(), title.clone());
                }
                if let Some(content) = obj.get("content") {
                    output.insert("content".to_string(), content.clone());
                }
            }
            Ok(serde_json::Value::Object(output))
        })
    }
}

// Custom validator implementation
#[derive(Debug)]
struct SchemaValidator;

impl SchemaValidator {
    fn new() -> Self {
        Self
    }
}

impl Validator for SchemaValidator {
    fn validate(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = RiptideResult<serde_json::Value>> + Send + '_>> {
        Box::pin(async move {
            // Validate required fields
            if let Some(obj) = input.as_object() {
                if !obj.contains_key("title") {
                    return Err(RiptideError::validation("Missing required field: title"));
                }
                if !obj.contains_key("content") {
                    return Err(RiptideError::validation("Missing required field: content"));
                }
            } else {
                return Err(RiptideError::validation("Input is not a JSON object"));
            }
            Ok(input)
        })
    }
}
```

---

## Example 7: Pre-Built Pipeline Templates

### Use Case
Use pre-built pipeline templates for common workflows.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let pipeline_facade = PipelineFacade::new(config).await?;

    // Example 7a: Web scraping pipeline
    let scraping_pipeline = pipeline_facade
        .web_scraping_pipeline("https://example.com")
        .await?;

    let result = pipeline_facade.execute(scraping_pipeline).await?;
    println!("Web scraping completed: {:?}", result.total_duration);

    // Example 7b: PDF extraction pipeline
    let pdf_pipeline = pipeline_facade
        .pdf_extraction_pipeline("https://example.com/document.pdf")
        .await?;

    let result = pipeline_facade.execute(pdf_pipeline).await?;
    println!("PDF extraction completed: {:?}", result.total_duration);

    // Example 7c: Browser automation pipeline
    let browser_pipeline = pipeline_facade
        .browser_automation_pipeline(
            "https://spa-example.com",
            vec![
                BrowserAction::Wait { duration_ms: 2000 },
                BrowserAction::Click {
                    selector: ".load-more".to_string(),
                },
            ],
        )
        .await?;

    let result = pipeline_facade.execute(browser_pipeline).await?;
    println!("Browser automation completed: {:?}", result.total_duration);

    Ok(())
}
```

---

## Example 8: Error Handling and Retry

### Use Case
Robust error handling with retry logic and fallback strategies.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default()
        .with_timeout_secs(30);

    let scraper = ScraperFacade::new(config.clone()).await?;
    let extractor = ExtractionFacade::new(config).await?;

    let url = "https://unreliable-example.com";

    // Retry logic with exponential backoff
    let mut attempts = 0;
    let max_attempts = 3;
    let mut last_error = None;

    while attempts < max_attempts {
        attempts += 1;

        match scraper.fetch_html(url).await {
            Ok(html) => {
                // Try multiple extraction strategies with fallback
                let strategies = vec![
                    ExtractionStrategy::Wasm,
                    ExtractionStrategy::HtmlCss,
                    ExtractionStrategy::Fallback,
                ];

                match extractor.extract_with_fallback(&html, url, &strategies).await {
                    Ok(data) => {
                        println!("✓ Extracted successfully after {} attempts", attempts);
                        println!("  Strategy used: {}", data.strategy_used);
                        println!("  Confidence: {:.2}", data.confidence);
                        return Ok(());
                    }
                    Err(e) => {
                        last_error = Some(e);
                        if attempts < max_attempts {
                            let backoff = tokio::time::Duration::from_millis(100 * 2u64.pow(attempts - 1));
                            println!("Extraction failed, retrying in {:?}", backoff);
                            tokio::time::sleep(backoff).await;
                        }
                    }
                }
            }
            Err(e) => {
                last_error = Some(e);
                if attempts < max_attempts {
                    let backoff = tokio::time::Duration::from_millis(100 * 2u64.pow(attempts - 1));
                    println!("Fetch failed, retrying in {:?}", backoff);
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    // All attempts failed
    if let Some(error) = last_error {
        println!("✗ Failed after {} attempts: {}", attempts, error);
        Err(error.into())
    } else {
        Err("Unknown error".into())
    }
}
```

---

## Example 9: Schema-Based Extraction

### Use Case
Extract structured data using predefined schema.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let scraper = ScraperFacade::new(config.clone()).await?;
    let extractor = ExtractionFacade::new(config).await?;

    // Fetch HTML
    let html = scraper.fetch_html("https://ecommerce-example.com/product/123").await?;

    // Define extraction schema
    let mut schema = Schema {
        fields: HashMap::new(),
    };

    schema.fields.insert(
        "product_name".to_string(),
        FieldSpec {
            selector: "h1.product-title".to_string(),
            required: true,
            field_type: FieldType::Text,
        },
    );

    schema.fields.insert(
        "price".to_string(),
        FieldSpec {
            selector: "span.price".to_string(),
            required: true,
            field_type: FieldType::Number,
        },
    );

    schema.fields.insert(
        "description".to_string(),
        FieldSpec {
            selector: "div.product-description".to_string(),
            required: false,
            field_type: FieldType::Text,
        },
    );

    schema.fields.insert(
        "availability".to_string(),
        FieldSpec {
            selector: "span.stock-status".to_string(),
            required: true,
            field_type: FieldType::Text,
        },
    );

    // Extract structured data
    let data = extractor.extract_schema(&html, "https://ecommerce-example.com/product/123", &schema).await?;

    // Parse result
    println!("Product Name: {}", data["product_name"].as_str().unwrap_or("N/A"));
    println!("Price: ${}", data["price"].as_f64().unwrap_or(0.0));
    println!("Description: {}", data["description"].as_str().unwrap_or("N/A"));
    println!("Availability: {}", data["availability"].as_str().unwrap_or("N/A"));

    Ok(())
}
```

---

## Example 10: Cookie and Session Management

### Use Case
Manage cookies and session state across multiple requests.

```rust
use riptide_facade::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RiptideConfig::default();
    let browser = BrowserFacade::new(config).await?;

    let session = browser.launch().await?;

    // Set initial cookies
    let cookies = vec![
        Cookie {
            name: "session_id".to_string(),
            value: "abc123xyz".to_string(),
            domain: Some(".example.com".to_string()),
            path: Some("/".to_string()),
            expires: Some((std::time::SystemTime::now() + std::time::Duration::from_secs(3600))
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64),
            http_only: Some(true),
            secure: Some(true),
            same_site: Some("Strict".to_string()),
        },
    ];

    browser.set_cookies(&session, &cookies).await?;

    // Navigate to page
    browser.navigate(&session, "https://example.com/dashboard").await?;

    // Get updated cookies
    let updated_cookies = browser.get_cookies(&session).await?;
    println!("Cookies received: {}", updated_cookies.len());
    for cookie in &updated_cookies {
        println!("  {}: {}", cookie.name, cookie.value);
    }

    // Access local storage
    let storage = browser.get_local_storage(&session).await?;
    println!("Local storage: {:?}", storage);

    // Set local storage item
    browser.set_local_storage_item(&session, "user_preference", "dark_mode").await?;

    browser.close(session).await?;

    Ok(())
}
```

---

## Common Patterns Summary

### Pattern 1: Simple Fetch + Extract
```rust
let scraper = ScraperFacade::new(config).await?;
let extractor = ExtractionFacade::new(config).await?;
let html = scraper.fetch_html(url).await?;
let data = extractor.extract_html(&html, url, options).await?;
```

### Pattern 2: Browser + Actions + Extract
```rust
let browser = BrowserFacade::new(config).await?;
let session = browser.launch().await?;
browser.navigate(&session, url).await?;
browser.perform_actions(&session, &actions).await?;
let html = browser.get_content(&session).await?;
```

### Pattern 3: Multi-Strategy Fallback
```rust
let strategies = vec![
    ExtractionStrategy::Wasm,
    ExtractionStrategy::HtmlCss,
    ExtractionStrategy::Fallback,
];
let data = extractor.extract_with_fallback(&html, url, &strategies).await?;
```

### Pattern 4: Pipeline Execution
```rust
let pipeline = facade.builder()
    .add_stage(stage1)
    .add_stage(stage2)
    .with_retry(3)
    .build().await?;
let result = facade.execute(pipeline).await?;
```

### Pattern 5: Parallel Batch Processing
```rust
stream::iter(urls)
    .map(|url| process_url(url))
    .buffer_unordered(5)
    .collect::<Vec<_>>()
    .await;
```

---

## Error Handling Patterns

### Pattern 1: Retry with Backoff
```rust
let mut attempts = 0;
while attempts < max_attempts {
    match operation().await {
        Ok(result) => return Ok(result),
        Err(e) if e.is_retryable() => {
            let backoff = Duration::from_millis(100 * 2u64.pow(attempts));
            tokio::time::sleep(backoff).await;
            attempts += 1;
        }
        Err(e) => return Err(e),
    }
}
```

### Pattern 2: Fallback Chain
```rust
operation_primary().await
    .or_else(|_| operation_fallback1().await)
    .or_else(|_| operation_fallback2().await)
    .or_else(|_| operation_final_fallback().await)
```

### Pattern 3: Timeout Wrapper
```rust
tokio::time::timeout(
    Duration::from_secs(30),
    operation()
).await??
```

---

## Best Practices

1. **Always use appropriate timeouts**
   ```rust
   .with_timeout_secs(30)
   ```

2. **Implement proper error handling**
   ```rust
   match result {
       Ok(data) => handle_success(data),
       Err(e) => handle_error(e),
   }
   ```

3. **Clean up resources**
   ```rust
   browser.close(session).await?;
   ```

4. **Use connection pooling for batch operations**
   ```rust
   let facade = Arc::new(ScraperFacade::new(config).await?);
   // Share facade across tasks
   ```

5. **Implement rate limiting**
   ```rust
   .buffer_unordered(5) // Limit concurrency
   ```

6. **Cache expensive operations**
   ```rust
   .with_caching(true)
   ```

7. **Use structured logging**
   ```rust
   tracing::info!(url = %url, "Fetching content");
   ```

---

## Performance Tips

1. **Reuse facade instances** - Don't create new facades for each request
2. **Use parallelism wisely** - Balance between throughput and resource usage
3. **Enable caching** - Reduce redundant operations
4. **Set appropriate timeouts** - Prevent hanging requests
5. **Monitor memory usage** - Clean up resources promptly
6. **Use connection pooling** - Reduce connection overhead
7. **Implement circuit breakers** - Protect against cascading failures

---

## Conclusion

These examples demonstrate practical usage patterns for the riptide-facade composition layer. They cover:

- Basic single-URL extraction
- Browser-based dynamic content
- Form interaction and authentication
- Screenshot capture
- Batch processing
- Pipeline workflows
- Error handling and retry logic
- Schema-based extraction
- Cookie and session management

For more details, see:
- [Facade Composition Patterns](facade-composition-patterns.md)
- [API Documentation](../api/README.md)
- [Performance Tuning Guide](../performance/tuning.md)
