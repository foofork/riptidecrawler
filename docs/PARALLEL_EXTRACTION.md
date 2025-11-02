# Parallel Document Extraction

High-performance parallel processing for batch document extraction using Tokio async runtime.

## Overview

The parallel extraction module enables efficient concurrent processing of multiple documents with configurable concurrency limits, automatic retry, progress tracking, and comprehensive error handling.

## Features

- **Configurable Concurrency**: Control maximum parallel tasks
- **Automatic Retry**: Retry failed extractions with exponential backoff
- **Progress Tracking**: Real-time progress callbacks for UI/monitoring
- **Timeout Management**: Per-document timeout controls
- **Fail-Fast Mode**: Option to stop all processing on first error
- **Streaming Results**: Get results as they complete via channels
- **Resource Management**: Efficient memory and CPU utilization
- **Metrics**: Comprehensive performance tracking

## Usage

### Basic Parallel Extraction

```rust
use riptide_extraction::parallel::{ParallelExtractor, ParallelConfig};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ParallelConfig::default()
        .with_max_concurrent(10)
        .with_timeout_per_doc(Duration::from_secs(30));

    let extractor = ParallelExtractor::new(config);

    let documents = vec![
        ("https://example.com", "<html>...</html>"),
        ("https://example.org", "<html>...</html>"),
    ];

    let results = extractor.extract_batch(documents).await?;

    for result in results {
        match result.result {
            Ok(doc) => println!("✓ {}: {}", result.url, doc.title),
            Err(e) => println!("✗ {}: {}", result.url, e),
        }
    }

    Ok(())
}
```

### Progress Tracking

```rust
let extractor = ParallelExtractor::new(config)
    .with_progress_callback(|progress| {
        println!(
            "Progress: {}/{} ({:.1}% complete)",
            progress.completed,
            progress.total,
            (progress.completed as f64 / progress.total as f64) * 100.0
        );
    });

let results = extractor.extract_batch(documents).await?;
```

### Streaming Results

```rust
let mut rx = extractor.extract_batch_streaming(documents).await?;

while let Some(result) = rx.recv().await {
    println!("Received result for: {}", result.url);
}
```

### Priority Queue

```rust
use riptide_extraction::parallel::DocumentTask;

let tasks = vec![
    DocumentTask {
        id: 0,
        url: "https://example.com".to_string(),
        html: "<html>...</html>".to_string(),
        priority: 10,  // Higher priority processed first
    },
    // ... more tasks
];

let results = extractor.extract_tasks(tasks).await?;
```

### Custom Configuration

```rust
let config = ParallelConfig::default()
    .with_max_concurrent(20)              // Max 20 concurrent tasks
    .with_timeout_per_doc(Duration::from_secs(60))  // 60s timeout per doc
    .with_retry(true)                      // Enable automatic retry
    .with_max_retries(3)                   // Max 3 retries per document
    .with_fail_fast(false);                // Continue on errors

let extractor = ParallelExtractor::new(config);
```

## Configuration Options

### ParallelConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_concurrent` | `usize` | `10` | Maximum number of concurrent extraction tasks |
| `timeout_per_doc` | `Duration` | `30s` | Timeout for each document extraction |
| `retry_failed` | `bool` | `true` | Whether to automatically retry failed extractions |
| `max_retries` | `usize` | `3` | Maximum number of retries per document |
| `fail_fast` | `bool` | `false` | Stop all processing on first error |
| `retry_backoff_multiplier` | `f64` | `2.0` | Exponential backoff multiplier for retries |
| `initial_retry_delay` | `Duration` | `100ms` | Initial retry delay |

## Metrics

The extractor provides comprehensive metrics:

```rust
let metrics = extractor.calculate_metrics(&results, duration);

println!("Total Processed: {}", metrics.total_processed);
println!("Success Rate: {}/{}", metrics.total_succeeded, metrics.total_processed);
println!("Avg Time: {:.2}ms", metrics.avg_processing_time_ms);
println!("Throughput: {:.2} docs/sec", metrics.throughput_docs_per_sec);
```

### Available Metrics

- `total_processed`: Total documents processed
- `total_succeeded`: Total successful extractions
- `total_failed`: Total failed extractions
- `avg_processing_time_ms`: Average processing time per document
- `min_processing_time_ms`: Minimum processing time
- `max_processing_time_ms`: Maximum processing time
- `throughput_docs_per_sec`: Documents processed per second
- `total_time_ms`: Total processing time
- `peak_concurrent`: Peak number of concurrent tasks
- `total_retries`: Total number of retry attempts

## Performance

### Benchmarks

Parallel extraction provides significant performance improvements:

- **10x speedup** for 50+ documents with `max_concurrent=10`
- **< 5% memory overhead** vs sequential processing
- **Linear scaling** up to CPU core count
- **Efficient resource utilization** with semaphore-based concurrency control

### Performance Tips

1. **Tune Concurrency**: Set `max_concurrent` to 2-3x CPU cores for I/O-bound tasks
2. **Batch Size**: Process 50-100 documents per batch for optimal throughput
3. **Timeout**: Set reasonable timeouts to prevent hanging tasks
4. **Retry**: Enable retry for production use with network operations
5. **Streaming**: Use streaming for large batches to reduce memory usage

## Error Handling

### Fail-Fast Mode

```rust
// Stop all processing on first error
let config = ParallelConfig::default().with_fail_fast(true);
```

### Graceful Degradation

```rust
// Continue processing even if some documents fail
let config = ParallelConfig::default().with_fail_fast(false);

let results = extractor.extract_batch(documents).await?;

for result in results {
    match result.result {
        Ok(doc) => process_document(doc),
        Err(e) => log_error(&result.url, &e),
    }
}
```

### Retry Logic

The extractor automatically retries failed extractions with exponential backoff:

```rust
let config = ParallelConfig::default()
    .with_retry(true)
    .with_max_retries(3)
    .with_initial_retry_delay(Duration::from_millis(100));
```

Retry delays follow exponential backoff:
- Retry 1: 100ms
- Retry 2: 200ms
- Retry 3: 400ms

## Architecture

### Components

1. **ParallelExtractor**: Main coordinator for parallel extraction
2. **ParallelConfig**: Configuration for concurrency and retry behavior
3. **DocumentTask**: Task representation with priority support
4. **ExtractionResult**: Result of a document extraction
5. **ExtractionProgress**: Progress tracking state
6. **ExtractionMetrics**: Performance metrics

### Implementation Details

- **Tokio Runtime**: Leverages Tokio's async runtime for concurrency
- **Semaphore**: Controls concurrency limits using `tokio::sync::Semaphore`
- **JoinSet**: Manages spawned tasks with `tokio::task::JoinSet`
- **Work-Stealing Queue**: Uses `VecDeque` for efficient task distribution
- **Progress Tracking**: Thread-safe progress updates with `Arc<RwLock>`

## Examples

See the full example in `examples/parallel_extraction_example.rs`:

```bash
cargo run --example parallel_extraction_example
```

## Testing

The module includes comprehensive tests covering:

- Basic parallel extraction
- Concurrency limit enforcement
- Timeout handling
- Retry logic
- Fail-fast mode
- Progress tracking
- Streaming results
- Memory efficiency (100+ documents)
- Error isolation
- Priority queue ordering
- Resource cleanup
- Graceful shutdown
- Concurrent extraction with different configs
- Performance benchmarks

Run tests:

```bash
cargo test --package riptide-extraction --test parallel_extraction_tests
```

## Best Practices

1. **Start Conservative**: Begin with `max_concurrent=5-10` and tune based on metrics
2. **Monitor Progress**: Use progress callbacks for long-running batches
3. **Handle Errors**: Always check `result.result` for errors
4. **Set Timeouts**: Configure reasonable per-document timeouts
5. **Use Streaming**: For large batches, use streaming to reduce memory
6. **Track Metrics**: Monitor metrics to optimize configuration
7. **Test Thoroughly**: Test with various batch sizes and error conditions

## Troubleshooting

### Performance Issues

- **Low throughput**: Increase `max_concurrent`
- **High memory usage**: Reduce `max_concurrent` or use streaming
- **Timeouts**: Increase `timeout_per_doc` or investigate slow documents

### Error Handling

- **All tasks failing**: Check network connectivity or HTML format
- **Some tasks failing**: Review individual error messages in results
- **Retries not working**: Verify `retry_failed=true` and `max_retries > 0`

## Integration

The parallel extractor integrates seamlessly with the existing extraction pipeline:

```rust
use riptide_extraction::{UnifiedExtractor, parallel::ParallelExtractor};

// Use custom extractor
let unified = UnifiedExtractor::default();
let config = ParallelConfig::default();
let parallel = ParallelExtractor::with_extractor(config, unified);
```

## Future Enhancements

- Adaptive concurrency based on system load
- Per-document resource limits
- Advanced scheduling algorithms
- Distributed extraction across multiple nodes
- Integration with message queues
- Persistent retry queue

## Related Documentation

- [Unified Extractor](../crates/riptide-extraction/src/unified_extractor.rs)
- [CSS Extraction](../crates/riptide-extraction/src/css_extraction.rs)
- [Regex Extraction](../crates/riptide-extraction/src/regex_extraction.rs)

## License

See the main project LICENSE file.
