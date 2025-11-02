//! Parallel Extraction Example
//!
//! This example demonstrates how to use the parallel extraction features
//! to process multiple documents concurrently with progress tracking and metrics.
//!
//! Run with: cargo run --example parallel_extraction_example

use riptide_extraction::parallel::{ParallelConfig, ParallelExtractor};
use std::time::{Duration, Instant};

fn create_sample_html(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>{}</title>
            <meta name="description" content="Sample document for parallel extraction">
        </head>
        <body>
            <header>
                <h1>{}</h1>
            </header>
            <main>
                <article>
                    <h2>Introduction</h2>
                    <p>{}</p>

                    <h2>Main Content</h2>
                    <p>This is a sample document created for testing parallel extraction capabilities.</p>
                    <p>It contains multiple paragraphs and structured content to simulate real-world documents.</p>

                    <h2>Features</h2>
                    <ul>
                        <li>Concurrent processing</li>
                        <li>Progress tracking</li>
                        <li>Error handling</li>
                        <li>Performance metrics</li>
                    </ul>

                    <h2>Conclusion</h2>
                    <p>Parallel extraction enables efficient batch processing of documents.</p>
                </article>
            </main>
            <footer>
                <p>&copy; 2024 RipTide Extraction Example</p>
            </footer>
        </body>
        </html>"#,
        title, title, content
    )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Parallel Extraction Example ===\n");

    // Example 1: Basic Parallel Extraction
    println!("Example 1: Basic Parallel Extraction");
    println!("-------------------------------------");

    let config = ParallelConfig::default()
        .with_max_concurrent(5)
        .with_timeout_per_doc(Duration::from_secs(30));

    let extractor = ParallelExtractor::new(config);

    let documents = vec![
        (
            "https://example.com/doc1",
            create_sample_html("Document 1", "First document content"),
        ),
        (
            "https://example.com/doc2",
            create_sample_html("Document 2", "Second document content"),
        ),
        (
            "https://example.com/doc3",
            create_sample_html("Document 3", "Third document content"),
        ),
        (
            "https://example.com/doc4",
            create_sample_html("Document 4", "Fourth document content"),
        ),
        (
            "https://example.com/doc5",
            create_sample_html("Document 5", "Fifth document content"),
        ),
    ];

    let start = Instant::now();
    let results = extractor.extract_batch(documents).await?;
    let duration = start.elapsed();

    println!("Processed {} documents in {:?}", results.len(), duration);
    println!(
        "Success rate: {}/{}\n",
        results.iter().filter(|r| r.result.is_ok()).count(),
        results.len()
    );

    // Example 2: Large Batch with Progress Tracking
    println!("Example 2: Large Batch with Progress Tracking");
    println!("----------------------------------------------");

    let config = ParallelConfig::default()
        .with_max_concurrent(10)
        .with_retry(true)
        .with_max_retries(2);

    let extractor = ParallelExtractor::new(config).with_progress_callback(|progress| {
        println!(
            "Progress: {}/{} completed ({} succeeded, {} failed) - {:.1}% | ETA: {}ms",
            progress.completed,
            progress.total,
            progress.succeeded,
            progress.failed,
            (progress.completed as f64 / progress.total as f64) * 100.0,
            progress.estimated_remaining_ms
        );
    });

    let documents: Vec<_> = (1..=50)
        .map(|i| {
            (
                format!("https://example.com/article{}", i),
                create_sample_html(
                    &format!("Article {}", i),
                    &format!("Content for article number {}", i),
                ),
            )
        })
        .collect();

    let start = Instant::now();
    let results = extractor.extract_batch(documents).await?;
    let duration = start.elapsed();

    let metrics = extractor.calculate_metrics(&results, duration);

    println!("\n=== Metrics ===");
    println!("Total Processed: {}", metrics.total_processed);
    println!("Succeeded: {}", metrics.total_succeeded);
    println!("Failed: {}", metrics.total_failed);
    println!(
        "Avg Processing Time: {:.2}ms",
        metrics.avg_processing_time_ms
    );
    println!(
        "Min Processing Time: {:.2}ms",
        metrics.min_processing_time_ms
    );
    println!(
        "Max Processing Time: {:.2}ms",
        metrics.max_processing_time_ms
    );
    println!(
        "Throughput: {:.2} docs/sec",
        metrics.throughput_docs_per_sec
    );
    println!("Total Time: {}ms", metrics.total_time_ms);
    println!("Total Retries: {}\n", metrics.total_retries);

    // Example 3: Streaming Results
    println!("Example 3: Streaming Results");
    println!("-----------------------------");

    let config = ParallelConfig::default().with_max_concurrent(3);
    let extractor = ParallelExtractor::new(config);

    let documents = vec![
        (
            "https://news.example.com/1",
            create_sample_html("News Article 1", "Breaking news content"),
        ),
        (
            "https://news.example.com/2",
            create_sample_html("News Article 2", "Technology update"),
        ),
        (
            "https://news.example.com/3",
            create_sample_html("News Article 3", "Sports highlights"),
        ),
    ];

    let mut rx = extractor.extract_batch_streaming(documents).await?;

    println!("Streaming results as they complete...");
    let mut count = 0;
    while let Some(result) = rx.recv().await {
        count += 1;
        match result.result {
            Ok(doc) => {
                println!(
                    "✓ [{}] {} - {} ({:?})",
                    result.task_id, result.url, doc.title, result.duration
                );
            }
            Err(e) => {
                println!("✗ [{}] {} - Error: {}", result.task_id, result.url, e);
            }
        }
    }
    println!("Streamed {} results\n", count);

    // Example 4: Priority Queue
    println!("Example 4: Priority Queue");
    println!("-------------------------");

    use riptide_extraction::parallel::DocumentTask;

    let config = ParallelConfig::default().with_max_concurrent(2);
    let extractor = ParallelExtractor::new(config);

    let tasks = vec![
        DocumentTask {
            id: 0,
            url: "https://example.com/low-priority".to_string(),
            html: create_sample_html("Low Priority", "Less important content"),
            priority: 1,
        },
        DocumentTask {
            id: 1,
            url: "https://example.com/high-priority".to_string(),
            html: create_sample_html("High Priority", "Critical content"),
            priority: 10,
        },
        DocumentTask {
            id: 2,
            url: "https://example.com/medium-priority".to_string(),
            html: create_sample_html("Medium Priority", "Moderately important content"),
            priority: 5,
        },
    ];

    let results = extractor.extract_tasks(tasks).await?;

    println!("Processing order (by priority):");
    for result in results {
        println!(
            "Task {}: {} - {:?}",
            result.task_id, result.url, result.duration
        );
    }

    println!("\n=== Example Complete ===");
    Ok(())
}
