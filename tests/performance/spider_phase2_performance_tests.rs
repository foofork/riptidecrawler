//! Performance tests for Phase 2 spider features
//!
//! Tests:
//! - Large URL collection performance
//! - Serialization performance for thousands of URLs
//! - Memory usage with large discovered_urls arrays
//! - Throughput with result_mode=urls

#[cfg(test)]
mod spider_phase2_performance_tests {
    use riptide_api::dto::{ResultMode, SpiderResultUrls};
    use serde_json;
    use std::time::Instant;

    #[test]
    fn test_serialization_performance_large_url_collection() {
        // Generate 10,000 URLs
        let large_urls: Vec<String> = (0..10_000)
            .map(|i| format!("https://example.com/page/{}", i))
            .collect();

        let result = SpiderResultUrls {
            pages_crawled: 10_000,
            pages_failed: 0,
            duration_seconds: 120.0,
            stop_reason: "max_pages_reached".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: large_urls.clone(),
        };

        // Test serialization performance
        let start = Instant::now();
        let json_str = serde_json::to_string(&result).unwrap();
        let serialize_duration = start.elapsed();

        println!(
            "Serialization of 10k URLs took: {:?} ({} bytes)",
            serialize_duration,
            json_str.len()
        );

        // Should serialize in reasonable time (< 100ms)
        assert!(
            serialize_duration.as_millis() < 100,
            "Serialization should be fast: {:?}",
            serialize_duration
        );

        // Test deserialization performance
        let start = Instant::now();
        let deserialized: SpiderResultUrls = serde_json::from_str(&json_str).unwrap();
        let deserialize_duration = start.elapsed();

        println!("Deserialization of 10k URLs took: {:?}", deserialize_duration);

        // Should deserialize in reasonable time (< 100ms)
        assert!(
            deserialize_duration.as_millis() < 100,
            "Deserialization should be fast: {:?}",
            deserialize_duration
        );

        // Verify data integrity
        assert_eq!(deserialized.discovered_urls.len(), 10_000);
        assert_eq!(deserialized.discovered_urls[0], "https://example.com/page/0");
        assert_eq!(
            deserialized.discovered_urls[9999],
            "https://example.com/page/9999"
        );
    }

    #[test]
    fn test_memory_usage_large_url_arrays() {
        // Test with varying sizes
        let sizes = vec![100, 1_000, 10_000, 50_000];

        for size in sizes {
            let urls: Vec<String> = (0..size)
                .map(|i| format!("https://example.com/page/{}", i))
                .collect();

            let result = SpiderResultUrls {
                pages_crawled: size as u64,
                pages_failed: 0,
                duration_seconds: 60.0,
                stop_reason: "max_pages_reached".to_string(),
                domains: vec!["example.com".to_string()],
                discovered_urls: urls,
            };

            // Serialize and measure size
            let json_str = serde_json::to_string(&result).unwrap();
            let size_kb = json_str.len() / 1024;

            println!(
                "Size {} URLs: {} KB ({} bytes)",
                size,
                size_kb,
                json_str.len()
            );

            // Approximate: each URL ~30 chars = ~30 bytes
            // 10k URLs ≈ 300 KB JSON
            // Should be linear growth
            let expected_max_kb = (size * 50) / 1024; // Conservative estimate
            assert!(
                size_kb < expected_max_kb,
                "Size should be reasonable for {} URLs: {} KB (expected < {} KB)",
                size,
                size_kb,
                expected_max_kb
            );
        }
    }

    #[test]
    fn test_url_deduplication_performance() {
        use std::collections::HashSet;

        // Create URLs with many duplicates
        let mut urls = Vec::new();
        for i in 0..1000 {
            urls.push(format!("https://example.com/page/{}", i % 100)); // 10x duplicates
        }

        let start = Instant::now();
        let unique_urls: HashSet<_> = urls.iter().collect();
        let dedup_duration = start.elapsed();

        println!(
            "Deduplication of 1000 URLs (100 unique) took: {:?}",
            dedup_duration
        );

        assert_eq!(unique_urls.len(), 100);
        assert!(
            dedup_duration.as_micros() < 1000,
            "Deduplication should be very fast: {:?}",
            dedup_duration
        );
    }

    #[test]
    fn test_json_compactness_vs_pretty_print() {
        let urls: Vec<String> = (0..100)
            .map(|i| format!("https://example.com/page/{}", i))
            .collect();

        let result = SpiderResultUrls {
            pages_crawled: 100,
            pages_failed: 0,
            duration_seconds: 10.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: urls,
        };

        // Compact JSON
        let compact = serde_json::to_string(&result).unwrap();

        // Pretty JSON
        let pretty = serde_json::to_string_pretty(&result).unwrap();

        println!("Compact: {} bytes", compact.len());
        println!("Pretty: {} bytes", pretty.len());
        println!(
            "Overhead: {:.1}%",
            ((pretty.len() - compact.len()) as f64 / compact.len() as f64) * 100.0
        );

        // Compact should be significantly smaller
        assert!(compact.len() < pretty.len());
        assert!(
            compact.len() < pretty.len() / 2,
            "Compact JSON should be <50% of pretty-printed"
        );
    }

    #[test]
    fn test_result_mode_enum_performance() {
        // Test that enum operations are fast (should be trivial)
        let iterations = 100_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let mode = ResultMode::Urls;
            let _ = serde_json::to_string(&mode).unwrap();
        }
        let duration = start.elapsed();

        println!(
            "Serialized {} ResultMode enums in {:?} ({:.2} ns/op)",
            iterations,
            duration,
            duration.as_nanos() as f64 / iterations as f64
        );

        // Should be extremely fast
        assert!(
            duration.as_millis() < 100,
            "Enum serialization should be very fast"
        );
    }

    #[test]
    fn test_throughput_stats_vs_urls_mode() {
        use riptide_api::dto::SpiderResultStats;

        let iterations = 1000;

        // Test stats mode throughput
        let stats = SpiderResultStats {
            pages_crawled: 100,
            pages_failed: 5,
            duration_seconds: 30.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
        };

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = serde_json::to_string(&stats).unwrap();
        }
        let stats_duration = start.elapsed();

        // Test urls mode throughput
        let urls: Vec<String> = (0..100)
            .map(|i| format!("https://example.com/page/{}", i))
            .collect();

        let result = SpiderResultUrls {
            pages_crawled: 100,
            pages_failed: 5,
            duration_seconds: 30.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: urls,
        };

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = serde_json::to_string(&result).unwrap();
        }
        let urls_duration = start.elapsed();

        println!(
            "Stats mode: {} iterations in {:?} ({:.2} µs/op)",
            iterations,
            stats_duration,
            stats_duration.as_micros() as f64 / iterations as f64
        );

        println!(
            "URLs mode: {} iterations in {:?} ({:.2} µs/op)",
            iterations,
            urls_duration,
            urls_duration.as_micros() as f64 / iterations as f64
        );

        // URLs mode should still be fast, even with 100 URLs
        assert!(
            urls_duration.as_millis() < 1000,
            "URLs mode should complete 1000 iterations quickly"
        );

        // Calculate overhead
        let overhead_factor = urls_duration.as_micros() as f64 / stats_duration.as_micros() as f64;
        println!("URLs mode overhead: {:.2}x", overhead_factor);

        // Overhead should be reasonable (< 10x for 100 URLs)
        assert!(
            overhead_factor < 10.0,
            "URLs mode overhead should be reasonable: {:.2}x",
            overhead_factor
        );
    }

    #[test]
    fn test_extreme_url_lengths() {
        // Test with very long URLs (query params, paths, etc.)
        let long_urls: Vec<String> = (0..100)
            .map(|i| {
                format!(
                    "https://example.com/very/long/path/with/many/segments/and/parameters?param1=value1&param2=value2&param3=value3&id={}&timestamp={}&session={}&token={}",
                    i, i * 1000, i * 2000, i * 3000
                )
            })
            .collect();

        let result = SpiderResultUrls {
            pages_crawled: 100,
            pages_failed: 0,
            duration_seconds: 20.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: long_urls.clone(),
        };

        let start = Instant::now();
        let json_str = serde_json::to_string(&result).unwrap();
        let duration = start.elapsed();

        println!(
            "Serialized 100 long URLs ({} chars avg) in {:?}",
            long_urls[0].len(),
            duration
        );

        // Should still be fast
        assert!(
            duration.as_millis() < 50,
            "Should handle long URLs efficiently"
        );

        // Verify round-trip
        let deserialized: SpiderResultUrls = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.discovered_urls.len(), 100);
        assert_eq!(deserialized.discovered_urls[0], long_urls[0]);
    }

    #[test]
    fn test_concurrent_serialization() {
        use std::sync::Arc;
        use std::thread;

        let urls: Vec<String> = (0..1000)
            .map(|i| format!("https://example.com/page/{}", i))
            .collect();

        let result = Arc::new(SpiderResultUrls {
            pages_crawled: 1000,
            pages_failed: 0,
            duration_seconds: 60.0,
            stop_reason: "completed".to_string(),
            domains: vec!["example.com".to_string()],
            discovered_urls: urls,
        });

        let start = Instant::now();

        // Spawn multiple threads to serialize concurrently
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let result = Arc::clone(&result);
                thread::spawn(move || {
                    for _ in 0..100 {
                        let _ = serde_json::to_string(&*result).unwrap();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();

        println!("Concurrent serialization (4 threads × 100 ops) took: {:?}", duration);

        // Should complete in reasonable time
        assert!(
            duration.as_secs() < 5,
            "Concurrent serialization should be efficient"
        );
    }
}
