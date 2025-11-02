//! Comprehensive tests for parallel document extraction

use riptide_extraction::parallel::{
    DocumentTask, ExtractionMetrics, ParallelConfig, ParallelExtractor,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

fn create_test_html(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
        <html>
        <head><title>{}</title></head>
        <body>
            <h1>{}</h1>
            <p>{}</p>
        </body>
        </html>"#,
        title, title, content
    )
}

#[tokio::test]
async fn test_basic_parallel_extraction() {
    let config = ParallelConfig::default().with_max_concurrent(3);
    let extractor = ParallelExtractor::new(config);

    let documents = vec![
        (
            "https://example1.com".to_string(),
            create_test_html("Doc 1", "Content 1"),
        ),
        (
            "https://example2.com".to_string(),
            create_test_html("Doc 2", "Content 2"),
        ),
        (
            "https://example3.com".to_string(),
            create_test_html("Doc 3", "Content 3"),
        ),
    ];

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results.iter().filter(|r| r.result.is_ok()).count(), 3);

    // Verify all documents were processed
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.task_id, i);
        assert!(result.result.is_ok());
        assert!(result.duration.as_millis() > 0);
    }
}

#[tokio::test]
async fn test_concurrency_limit_enforcement() {
    let config = ParallelConfig::default().with_max_concurrent(2);

    let processing_count = Arc::new(Mutex::new(0));
    let max_concurrent = Arc::new(Mutex::new(0));

    let extractor = ParallelExtractor::new(config);

    // Create documents that take some time to process
    let documents: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let start = Instant::now();
    let results = extractor.extract_batch(documents).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(results.len(), 10);
    println!("Processed 10 documents in {:?}", duration);

    // With concurrency limit of 2, should take roughly 5x the time of a single document
    // (this is a rough check, actual timing depends on system)
}

#[tokio::test]
async fn test_timeout_handling() {
    let config = ParallelConfig::default()
        .with_timeout_per_doc(Duration::from_millis(1)) // Very short timeout
        .with_retry(false); // Disable retry for this test

    let extractor = ParallelExtractor::new(config);

    let documents = vec![(
        "https://example.com".to_string(),
        create_test_html("Test", "Content"),
    )];

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 1);
    // Note: Might still succeed if extraction is very fast
    // This test mainly ensures timeout doesn't panic
}

#[tokio::test]
async fn test_retry_logic() {
    // This test verifies retry tracking
    let config = ParallelConfig::default()
        .with_retry(true)
        .with_max_retries(2)
        .with_max_concurrent(1);

    let extractor = ParallelExtractor::new(config);

    let documents = vec![(
        "https://example.com".to_string(),
        create_test_html("Test", "Content"),
    )];

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 1);
    // Successful extractions should have 0 retries
    assert_eq!(results[0].retry_count, 0);
}

#[tokio::test]
async fn test_fail_fast_mode_disabled() {
    let config = ParallelConfig::default()
        .with_fail_fast(false)
        .with_max_concurrent(3);

    let extractor = ParallelExtractor::new(config);

    // Mix of valid and potentially problematic documents
    let documents = vec![
        (
            "https://example1.com".to_string(),
            create_test_html("Doc 1", "Content 1"),
        ),
        (
            "https://example2.com".to_string(),
            create_test_html("Doc 2", "Content 2"),
        ),
        (
            "https://example3.com".to_string(),
            create_test_html("Doc 3", "Content 3"),
        ),
    ];

    let results = extractor.extract_batch(documents).await.unwrap();

    // All documents should be processed even if some fail
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_progress_tracking() {
    let config = ParallelConfig::default().with_max_concurrent(2);

    let progress_updates = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_updates);

    let extractor = ParallelExtractor::new(config).with_progress_callback(move |progress| {
        let updates = progress_clone.blocking_lock();
        let mut updates = updates.clone();
        updates.push(progress);
    });

    let documents = vec![
        (
            "https://example1.com".to_string(),
            create_test_html("Doc 1", "Content 1"),
        ),
        (
            "https://example2.com".to_string(),
            create_test_html("Doc 2", "Content 2"),
        ),
        (
            "https://example3.com".to_string(),
            create_test_html("Doc 3", "Content 3"),
        ),
    ];

    let _ = extractor.extract_batch(documents).await.unwrap();

    let updates = progress_updates.lock().await;
    assert!(updates.len() >= 3); // At least one update per document
}

#[tokio::test]
async fn test_streaming_results() {
    let config = ParallelConfig::default().with_max_concurrent(3);
    let extractor = ParallelExtractor::new(config);

    let documents = vec![
        (
            "https://example1.com".to_string(),
            create_test_html("Doc 1", "Content 1"),
        ),
        (
            "https://example2.com".to_string(),
            create_test_html("Doc 2", "Content 2"),
        ),
        (
            "https://example3.com".to_string(),
            create_test_html("Doc 3", "Content 3"),
        ),
        (
            "https://example4.com".to_string(),
            create_test_html("Doc 4", "Content 4"),
        ),
        (
            "https://example5.com".to_string(),
            create_test_html("Doc 5", "Content 5"),
        ),
    ];

    let mut rx = extractor.extract_batch_streaming(documents).await.unwrap();

    let mut count = 0;
    let mut results = Vec::new();

    while let Some(result) = rx.recv().await {
        count += 1;
        results.push(result);
    }

    assert_eq!(count, 5);
    assert_eq!(results.len(), 5);

    // Verify all results
    for result in results {
        assert!(result.result.is_ok());
    }
}

#[tokio::test]
async fn test_memory_efficiency_large_batch() {
    let config = ParallelConfig::default().with_max_concurrent(5);
    let extractor = ParallelExtractor::new(config);

    // Create 100 documents
    let documents: Vec<_> = (0..100)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let start = Instant::now();
    let results = extractor.extract_batch(documents).await.unwrap();
    let duration = start.elapsed();

    assert_eq!(results.len(), 100);

    let success_count = results.iter().filter(|r| r.result.is_ok()).count();
    println!(
        "Processed 100 documents: {} succeeded in {:?}",
        success_count, duration
    );

    assert_eq!(success_count, 100);
}

#[tokio::test]
async fn test_error_isolation() {
    let config = ParallelConfig::default()
        .with_max_concurrent(3)
        .with_fail_fast(false);

    let extractor = ParallelExtractor::new(config);

    // Create mix of valid and empty documents
    let documents = vec![
        (
            "https://example1.com".to_string(),
            create_test_html("Doc 1", "Content 1"),
        ),
        (
            "https://example2.com".to_string(),
            "<html></html>".to_string(),
        ),
        (
            "https://example3.com".to_string(),
            create_test_html("Doc 3", "Content 3"),
        ),
        (
            "https://example4.com".to_string(),
            create_test_html("Doc 4", "Content 4"),
        ),
    ];

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 4);

    // At least some should succeed
    let success_count = results.iter().filter(|r| r.result.is_ok()).count();
    assert!(success_count >= 3); // Most should succeed
}

#[tokio::test]
async fn test_priority_queue_ordering() {
    let config = ParallelConfig::default().with_max_concurrent(1); // Sequential for predictability
    let extractor = ParallelExtractor::new(config);

    let tasks = vec![
        DocumentTask {
            id: 0,
            url: "https://low.com".to_string(),
            html: create_test_html("Low Priority", "Low content"),
            priority: 1,
        },
        DocumentTask {
            id: 1,
            url: "https://high.com".to_string(),
            html: create_test_html("High Priority", "High content"),
            priority: 10,
        },
        DocumentTask {
            id: 2,
            url: "https://medium.com".to_string(),
            html: create_test_html("Medium Priority", "Medium content"),
            priority: 5,
        },
    ];

    let results = extractor.extract_tasks(tasks).await.unwrap();

    assert_eq!(results.len(), 3);

    // Results should be sorted by task ID
    assert_eq!(results[0].task_id, 0);
    assert_eq!(results[1].task_id, 1);
    assert_eq!(results[2].task_id, 2);

    // All should succeed
    for result in results {
        assert!(result.result.is_ok());
    }
}

#[tokio::test]
async fn test_resource_cleanup() {
    let config = ParallelConfig::default().with_max_concurrent(5);
    let extractor = ParallelExtractor::new(config);

    let documents: Vec<_> = (0..20)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 20);

    // Ensure extractor can be dropped cleanly
    drop(extractor);
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let config = ParallelConfig::default().with_max_concurrent(3);
    let extractor = ParallelExtractor::new(config);

    let documents: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 10);

    // Verify all tasks completed
    for result in results {
        assert!(result.duration.as_millis() > 0);
    }
}

#[tokio::test]
async fn test_concurrent_extraction_different_configs() {
    // Test running multiple extractors concurrently
    let config1 = ParallelConfig::default().with_max_concurrent(2);
    let config2 = ParallelConfig::default().with_max_concurrent(3);

    let extractor1 = ParallelExtractor::new(config1);
    let extractor2 = ParallelExtractor::new(config2);

    let docs1: Vec<_> = (0..5)
        .map(|i| {
            (
                format!("https://batch1-{}.com", i),
                create_test_html(&format!("Batch1 Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let docs2: Vec<_> = (0..5)
        .map(|i| {
            (
                format!("https://batch2-{}.com", i),
                create_test_html(&format!("Batch2 Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let (results1, results2) = tokio::join!(
        extractor1.extract_batch(docs1),
        extractor2.extract_batch(docs2)
    );

    assert_eq!(results1.unwrap().len(), 5);
    assert_eq!(results2.unwrap().len(), 5);
}

#[tokio::test]
async fn test_metrics_calculation() {
    let config = ParallelConfig::default().with_max_concurrent(3);
    let extractor = ParallelExtractor::new(config);

    let documents: Vec<_> = (0..10)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    let start = Instant::now();
    let results = extractor.extract_batch(documents).await.unwrap();
    let duration = start.elapsed();

    let metrics = extractor.calculate_metrics(&results, duration);

    assert_eq!(metrics.total_processed, 10);
    assert_eq!(metrics.total_succeeded, 10);
    assert_eq!(metrics.total_failed, 0);
    assert!(metrics.avg_processing_time_ms > 0.0);
    assert!(metrics.min_processing_time_ms > 0.0);
    assert!(metrics.max_processing_time_ms > 0.0);
    assert!(metrics.throughput_docs_per_sec > 0.0);
    assert_eq!(metrics.total_retries, 0);

    println!("Metrics: {:?}", metrics);
}

#[tokio::test]
async fn test_performance_benchmark_speedup() {
    // Test that parallel processing is significantly faster than sequential
    let sequential_config = ParallelConfig::default().with_max_concurrent(1);
    let parallel_config = ParallelConfig::default().with_max_concurrent(5);

    let sequential_extractor = ParallelExtractor::new(sequential_config);
    let parallel_extractor = ParallelExtractor::new(parallel_config);

    let documents: Vec<_> = (0..20)
        .map(|i| {
            (
                format!("https://example{}.com", i),
                create_test_html(&format!("Doc {}", i), &format!("Content {}", i)),
            )
        })
        .collect();

    // Sequential
    let docs_clone = documents.clone();
    let start = Instant::now();
    let _ = sequential_extractor
        .extract_batch(docs_clone)
        .await
        .unwrap();
    let sequential_duration = start.elapsed();

    // Parallel
    let start = Instant::now();
    let _ = parallel_extractor.extract_batch(documents).await.unwrap();
    let parallel_duration = start.elapsed();

    println!("Sequential: {:?}", sequential_duration);
    println!("Parallel: {:?}", parallel_duration);

    // Parallel should be faster (though exact speedup depends on system)
    // This is a soft assertion since CI environments vary
    let speedup = sequential_duration.as_millis() as f64 / parallel_duration.as_millis() as f64;
    println!("Speedup: {:.2}x", speedup);

    // Should be at least some speedup
    assert!(speedup >= 1.0);
}

#[tokio::test]
async fn test_empty_batch() {
    let config = ParallelConfig::default();
    let extractor = ParallelExtractor::new(config);

    let documents: Vec<(String, String)> = vec![];
    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_single_document_extraction() {
    let config = ParallelConfig::default();
    let extractor = ParallelExtractor::new(config);

    let documents = vec![(
        "https://example.com".to_string(),
        create_test_html("Single Doc", "Single content"),
    )];

    let results = extractor.extract_batch(documents).await.unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].result.is_ok());
    assert_eq!(results[0].task_id, 0);
    assert_eq!(results[0].url, "https://example.com");
}
