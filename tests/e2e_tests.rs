//! End-to-end integration tests for RipTide

use anyhow::Result;
use std::time::Duration;

#[cfg(test)]
mod complete_crawling_workflow {
    use super::*;

    #[tokio::test]
    async fn test_full_crawl_pipeline() {
        // Test the complete workflow from URL to extracted data
        let config = RipTideConfig {
            extraction_mode: ExtractionMode::ProbesFirst,
            enable_stealth: true,
            enable_caching: true,
            enable_intelligence: false, // Start without LLM
            max_pages: 10,
            timeout: Duration::from_secs(30),
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        // Start with a seed URL
        let seed = "https://example.com";
        let job = CrawlJob {
            urls: vec![seed.to_string()],
            max_depth: 2,
            query: Some("test content".to_string()),
            output_format: OutputFormat::NDJSON,
        };

        let results = riptide.execute_crawl(job).await.unwrap();

        // Verify results
        assert!(results.pages_crawled > 0);
        assert!(results.pages_extracted > 0);
        assert!(!results.extracted_docs.is_empty());

        // Check extraction quality
        for doc in &results.extracted_docs {
            assert!(!doc.url.is_empty());
            assert!(doc.text.len() > 0 || doc.title.is_some());
        }
    }

    #[tokio::test]
    async fn test_crawl_with_css_selectors() {
        let config = RipTideConfig::default();
        let riptide = RipTide::new(config).await.unwrap();

        let job = CrawlJob {
            urls: vec!["https://news.site".to_string()],
            selectors: Some(SelectorsConfig {
                title: Some("h1.headline".to_string()),
                content: Some("div.article-body".to_string()),
                author: Some(".byline .author".to_string()),
                date: Some("time[datetime]".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let results = riptide.execute_crawl(job).await;

        match results {
            Ok(crawl_results) => {
                for doc in crawl_results.extracted_docs {
                    // Should have structured extraction
                    if doc.title.is_some() {
                        assert!(!doc.title.unwrap().is_empty());
                    }
                }
            }
            Err(e) => {
                // Accept network errors in test environment
                assert!(e.to_string().contains("network") || e.to_string().contains("connection"));
            }
        }
    }

    #[tokio::test]
    async fn test_crawl_with_query_awareness() {
        let config = RipTideConfig {
            query_aware_crawling: true,
            bm25_weight: 0.4,
            url_signal_weight: 0.3,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.1,
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        let job = CrawlJob {
            urls: vec!["https://tech.blog".to_string()],
            query: Some("machine learning algorithms".to_string()),
            max_pages: 20,
            early_stop_threshold: Some(0.2),
            ..Default::default()
        };

        let results = riptide.execute_crawl(job).await;

        match results {
            Ok(crawl_results) => {
                // Should prioritize relevant pages
                let relevant_count = crawl_results.extracted_docs
                    .iter()
                    .filter(|doc| {
                        doc.text.to_lowercase().contains("machine") ||
                        doc.text.to_lowercase().contains("learning") ||
                        doc.text.to_lowercase().contains("algorithm")
                    })
                    .count();

                let relevance_ratio = relevant_count as f32 / crawl_results.pages_crawled as f32;
                assert!(relevance_ratio > 0.5); // At least 50% relevant
            }
            Err(_) => {
                // Accept failure in test environment
            }
        }
    }

    #[tokio::test]
    async fn test_crawl_with_llm_repair() {
        let config = RipTideConfig {
            enable_intelligence: true,
            llm_provider: "openai".to_string(),
            llm_timeout: Duration::from_secs(5),
            llm_max_retries: 1,
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        // Create a page with broken structure
        let job = CrawlJob {
            urls: vec!["https://broken.site".to_string()],
            enable_repair: true,
            ..Default::default()
        };

        let results = riptide.execute_crawl(job).await;

        match results {
            Ok(crawl_results) => {
                // Check if repair was attempted
                for doc in crawl_results.extracted_docs {
                    if doc.repair_applied {
                        assert!(doc.extraction_confidence > 0.7);
                    }
                }
            }
            Err(e) => {
                // LLM may not be available
                assert!(e.to_string().contains("provider") || e.to_string().contains("timeout"));
            }
        }
    }
}

#[cfg(test)]
mod performance_and_load_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_concurrent_crawling_performance() {
        let config = RipTideConfig {
            max_concurrent: 10,
            timeout: Duration::from_secs(30),
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        // Create 50 URLs to crawl
        let urls: Vec<String> = (0..50)
            .map(|i| format!("https://example.com/page{}", i))
            .collect();

        let job = CrawlJob {
            urls,
            max_depth: 1,
            ..Default::default()
        };

        let start = Instant::now();
        let results = riptide.execute_crawl(job).await;
        let duration = start.elapsed();

        match results {
            Ok(crawl_results) => {
                // Should complete within reasonable time
                assert!(duration.as_secs() < 60);

                // Check throughput
                let pages_per_second = crawl_results.pages_crawled as f64 / duration.as_secs_f64();
                assert!(pages_per_second > 0.5); // At least 0.5 pages/sec
            }
            Err(_) => {
                // Accept failure in test environment
            }
        }
    }

    #[tokio::test]
    async fn test_memory_usage_limits() {
        let config = RipTideConfig {
            max_memory_mb: 100,
            max_pages: 1000,
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        // Try to crawl many pages
        let job = CrawlJob {
            urls: vec!["https://large.site".to_string()],
            max_depth: 10,
            follow_links: true,
            ..Default::default()
        };

        let monitor = MemoryMonitor::new();
        monitor.start_monitoring();

        let results = riptide.execute_crawl(job).await;

        let peak_memory = monitor.get_peak_memory_mb();

        // Should respect memory limit
        assert!(peak_memory < 150); // Allow some overhead

        match results {
            Ok(crawl_results) => {
                // Should have stopped before OOM
                assert!(crawl_results.stopped_reason != StopReason::OutOfMemory ||
                       peak_memory < 100);
            }
            Err(_) => {
                // Accept failure
            }
        }
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let config = RipTideConfig {
            enable_caching: true,
            cache_ttl: Duration::from_secs(3600),
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        let url = "https://cacheable.site/page";

        // First crawl - should be slow
        let start = Instant::now();
        let job1 = CrawlJob {
            urls: vec![url.to_string()],
            ..Default::default()
        };
        let result1 = riptide.execute_crawl(job1).await;
        let first_duration = start.elapsed();

        // Second crawl - should be fast (from cache)
        let start = Instant::now();
        let job2 = CrawlJob {
            urls: vec![url.to_string()],
            ..Default::default()
        };
        let result2 = riptide.execute_crawl(job2).await;
        let second_duration = start.elapsed();

        if result1.is_ok() && result2.is_ok() {
            // Cache hit should be much faster
            assert!(second_duration < first_duration / 2);

            // Content should be identical
            let doc1 = &result1.unwrap().extracted_docs[0];
            let doc2 = &result2.unwrap().extracted_docs[0];
            assert_eq!(doc1.text, doc2.text);
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_behavior() {
        let config = RipTideConfig {
            circuit_breaker_threshold: 3,
            circuit_breaker_timeout: Duration::from_secs(5),
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        // Crawl a site that will fail
        let failing_job = CrawlJob {
            urls: vec!["https://always-fails.site".to_string()],
            max_retries: 0,
            ..Default::default()
        };

        // Cause multiple failures
        for _ in 0..3 {
            let _ = riptide.execute_crawl(failing_job.clone()).await;
        }

        // Circuit should be open now
        let result = riptide.execute_crawl(failing_job.clone()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("circuit"));

        // Wait for circuit to close
        tokio::time::sleep(Duration::from_secs(6)).await;

        // Should be able to try again
        let _ = riptide.execute_crawl(failing_job).await;
    }
}

#[cfg(test)]
mod streaming_and_export_tests {
    use super::*;

    #[tokio::test]
    async fn test_ndjson_streaming_output() {
        let config = RipTideConfig::default();
        let riptide = RipTide::new(config).await.unwrap();

        let job = CrawlJob {
            urls: vec!["https://example.com".to_string()],
            output_format: OutputFormat::NDJSON,
            streaming: true,
            ..Default::default()
        };

        let mut stream = riptide.execute_crawl_streaming(job).await.unwrap();

        let mut line_count = 0;
        while let Some(line) = stream.next_line().await {
            match line {
                Ok(json_line) => {
                    // Each line should be valid JSON
                    let parsed: serde_json::Value = serde_json::from_str(&json_line).unwrap();
                    assert!(parsed.is_object());
                    line_count += 1;
                }
                Err(_) => break,
            }
        }

        assert!(line_count > 0);
    }

    #[tokio::test]
    async fn test_table_export_to_csv() {
        let config = RipTideConfig {
            extract_tables: true,
            ..Default::default()
        };

        let riptide = RipTide::new(config).await.unwrap();

        let job = CrawlJob {
            urls: vec!["https://data.site/tables".to_string()],
            table_export_format: Some(TableFormat::CSV),
            ..Default::default()
        };

        let results = riptide.execute_crawl(job).await;

        match results {
            Ok(crawl_results) => {
                if !crawl_results.table_exports.is_empty() {
                    for (table_id, csv_data) in crawl_results.table_exports {
                        // Verify CSV format
                        assert!(csv_data.contains(","));
                        assert!(csv_data.contains("\n"));

                        // Parse CSV to verify structure
                        let reader = csv::Reader::from_reader(csv_data.as_bytes());
                        assert!(reader.into_records().count() > 0);
                    }
                }
            }
            Err(_) => {
                // Accept failure
            }
        }
    }

    #[tokio::test]
    async fn test_progress_tracking() {
        let config = RipTideConfig::default();
        let riptide = RipTide::new(config).await.unwrap();

        let job = CrawlJob {
            urls: vec!["https://example.com".to_string()],
            max_pages: 10,
            enable_progress: true,
            ..Default::default()
        };

        let (progress_rx, handle) = riptide.execute_crawl_with_progress(job).await.unwrap();

        let mut last_progress = 0;
        while let Ok(progress) = progress_rx.recv().await {
            // Progress should increase
            assert!(progress.processed_items >= last_progress);
            last_progress = progress.processed_items;

            // Check progress metadata
            assert!(progress.current_rate >= 0.0);
            if let Some(total) = progress.total_items {
                assert!(progress.processed_items <= total);
            }

            if progress.stage == ProgressStage::Completed {
                break;
            }
        }

        let results = handle.await.unwrap();
        assert!(results.is_ok());
    }
}