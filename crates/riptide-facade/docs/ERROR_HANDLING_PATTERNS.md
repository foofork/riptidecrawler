# Error Handling Patterns for Composition

When composing spider and extractor operations using `.and_extract()`, RipTide provides flexible error handling with three distinct patterns.

## Pattern Overview

### Spider Errors vs Extraction Errors

- **Spider errors abort the stream** - If URL discovery fails, the entire operation stops
- **Extraction errors yield `Result::Err`** - Failed extractions don't stop the stream
- **Stream continues** - Remaining URLs are still processed after extraction failures
- **User chooses** - Select the pattern that fits your use case

## Pattern 1: Filter Errors (Only Successful Extractions)

This pattern filters out all errors and only processes successful extractions.

```rust
use riptide_facade::traits::{Spider, Extractor, Chainable, SpiderOpts};
use futures::StreamExt;

async fn filter_pattern(spider: impl Spider, extractor: impl Extractor) -> Vec<Document> {
    spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor)
        .filter_map(|result| async move { result.ok() })
        .collect()
        .await
}
```

**Use when:**
- You only care about successful extractions
- Failures are expected and acceptable
- You want the simplest code

**Characteristics:**
- Silently ignores errors
- Returns only successful documents
- No error logging or handling

## Pattern 2: Handle Errors Explicitly

This pattern processes each result and handles errors explicitly.

```rust
async fn handle_pattern(spider: impl Spider, extractor: impl Extractor) {
    let mut stream = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .unwrap()
        .and_extract(extractor);

    while let Some(result) = stream.next().await {
        match result {
            Ok(doc) => {
                // Process successful extraction
                println!("Extracted: {}", doc.title);
                doc.to_json_file(&format!("{}.json", doc.title));
            }
            Err(err) => {
                // Handle extraction error
                eprintln!("Extraction failed: {}", err);
                // Log, retry, or take other action
            }
        }
    }
}
```

**Use when:**
- You need to log or report errors
- Different handling for success vs failure
- You want visibility into what failed

**Characteristics:**
- Full control over error handling
- Can log, retry, or take custom actions
- More verbose but flexible

## Pattern 3: Fail Fast (Abort on First Error)

This pattern aborts the entire operation on the first error encountered.

```rust
async fn fail_fast_pattern(spider: impl Spider, extractor: impl Extractor) -> Result<Vec<Document>, RiptideError> {
    spider
        .crawl("https://example.com", SpiderOpts::default())
        .await?
        .and_extract(extractor)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}
```

**Use when:**
- All extractions must succeed
- Any failure invalidates the entire batch
- You want strict error propagation

**Characteristics:**
- Returns error on first failure
- Stops processing remaining URLs
- Most strict error handling

## Concurrency Control

All patterns support concurrent processing with `buffer_unordered`:

```rust
let docs = spider
    .crawl(url, opts)
    .await?
    .and_extract(extractor)
    .buffer_unordered(10)  // Process 10 URLs concurrently
    .filter_map(|r| async move { r.ok() })  // Apply your chosen pattern
    .collect()
    .await;
```

## Partial Success Pattern (Default)

By default, RipTide implements a **partial success pattern**:

1. Spider errors **abort** the stream
2. Extraction errors **yield** `Result::Err` but continue
3. Stream **continues** processing remaining URLs
4. User **chooses** how to handle errors

This gives you maximum flexibility to decide between the three patterns above.

## Example: Production Use Case

```rust
use tracing::{info, warn};

async fn production_crawl(spider: impl Spider, extractor: impl Extractor) -> Vec<Document> {
    let mut docs = Vec::new();
    let mut error_count = 0;

    let mut stream = spider
        .crawl("https://example.com", SpiderOpts::default())
        .await
        .expect("Spider initialization failed");

    let mut stream = stream.and_extract(extractor);

    while let Some(result) = stream.next().await {
        match result {
            Ok(doc) => {
                info!("Successfully extracted: {}", doc.url);
                docs.push(doc);
            }
            Err(err) => {
                warn!("Extraction failed: {}", err);
                error_count += 1;

                // Abort if too many errors
                if error_count > 10 {
                    warn!("Too many errors, aborting");
                    break;
                }
            }
        }
    }

    info!("Completed with {} documents, {} errors", docs.len(), error_count);
    docs
}
```

## Performance Considerations

- **BoxStream overhead**: ~100ns per item (negligible for I/O operations)
- **Concurrent processing**: Use `buffer_unordered(N)` for better throughput
- **Error handling**: Pattern 1 is fastest, Pattern 2 has logging overhead
- **Memory usage**: All patterns stream results, no buffering required
