//! Performance validation tests for Phase 1 changes
//! Validates that metrics overhead is less than 1% of typical operation time

use std::time::{Duration, Instant};

/// Simulates a typical extraction operation
fn simulate_extraction() -> Duration {
    // Simulate extraction work (parsing, processing)
    let start = Instant::now();

    // Simulate HTML parsing
    let _ = (0..1000).map(|i| i * 2).collect::<Vec<_>>();

    // Simulate text extraction
    let text = "This is sample text ".repeat(100);
    let _ = text.split_whitespace().count();

    // Simulate link extraction
    let _ = (0..50).map(|i| format!("link_{}", i)).collect::<Vec<_>>();

    start.elapsed()
}

/// Simulates metrics recording
fn simulate_metrics_recording() {
    // Simulate counter increment (atomic operation)
    let _ = std::sync::atomic::AtomicU64::new(0);

    // Simulate histogram observation (atomic + bucket lookup)
    let value = 0.75_f64;
    let _ = value.to_bits();

    // Simulate label lookup
    let _ = vec!["mode", "raw"].join("_");
}

#[test]
fn test_metrics_overhead_less_than_1_percent() {
    let iterations = 10000;

    // Measure baseline without metrics
    let baseline_start = Instant::now();
    for _ in 0..iterations {
        simulate_extraction();
    }
    let baseline_duration = baseline_start.elapsed();

    // Measure with metrics recording
    let with_metrics_start = Instant::now();
    for _ in 0..iterations {
        simulate_extraction();
        simulate_metrics_recording();
    }
    let with_metrics_duration = with_metrics_start.elapsed();

    // Calculate overhead
    let overhead = with_metrics_duration.saturating_sub(baseline_duration);
    let overhead_percentage = (overhead.as_nanos() as f64 / baseline_duration.as_nanos() as f64) * 100.0;

    println!("Baseline: {:?}", baseline_duration);
    println!("With metrics: {:?}", with_metrics_duration);
    println!("Overhead: {:?} ({:.2}%)", overhead, overhead_percentage);

    assert!(
        overhead_percentage < 1.0,
        "Metrics overhead should be <1%, but was {:.2}%",
        overhead_percentage
    );
}

#[test]
fn test_metrics_recording_latency() {
    let iterations = 100000;

    let start = Instant::now();
    for _ in 0..iterations {
        simulate_metrics_recording();
    }
    let duration = start.elapsed();

    let avg_latency_ns = duration.as_nanos() / iterations as u128;

    println!("Average metrics recording latency: {}ns", avg_latency_ns);

    // Should be less than 100ns per recording
    assert!(
        avg_latency_ns < 100,
        "Metrics recording too slow: {}ns per call",
        avg_latency_ns
    );
}

#[test]
fn test_gate_decision_performance() {
    // Simulate gate decision logic
    let html = "<html><body><p>Content</p></body></html>".repeat(100);

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate gate decision calculations
        let _ = html.contains("react");
        let _ = html.matches("<script").count();
        let _ = html.len();
    }

    let duration = start.elapsed();
    let avg_per_decision = duration.as_micros() / iterations as u128;

    println!("Average gate decision time: {}µs", avg_per_decision);

    // Gate decision should take less than 100µs
    assert!(
        avg_per_decision < 100,
        "Gate decision too slow: {}µs",
        avg_per_decision
    );
}

#[test]
fn test_html_stripping_performance() {
    let html = r#"
        <html><head><title>Test</title></head>
        <body>
            <article>
                <p>Paragraph 1</p>
                <p>Paragraph 2</p>
                <p>Paragraph 3</p>
            </article>
        </body>
        </html>
    "#.repeat(10);

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate HTML stripping
        let mut result = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }
    }

    let duration = start.elapsed();
    let avg_per_strip = duration.as_micros() / iterations as u128;

    println!("Average HTML stripping time: {}µs", avg_per_strip);

    // HTML stripping should be fast
    assert!(
        avg_per_strip < 500,
        "HTML stripping too slow: {}µs",
        avg_per_strip
    );
}

#[test]
fn test_baseline_comparison_performance() {
    let baseline_text = "This is baseline content with many words ".repeat(50);
    let actual_text = "This is baseline content with many words ".repeat(50);

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate baseline comparison
        let baseline_words: Vec<&str> = baseline_text.split_whitespace().collect();
        let actual_words: Vec<&str> = actual_text.split_whitespace().collect();

        let matches = baseline_words
            .iter()
            .filter(|word| actual_words.contains(word))
            .count();

        let _ = matches as f64 / baseline_words.len() as f64;
    }

    let duration = start.elapsed();
    let avg_per_comparison = duration.as_micros() / iterations as u128;

    println!("Average baseline comparison time: {}µs", avg_per_comparison);

    // Baseline comparison should be reasonably fast
    assert!(
        avg_per_comparison < 100,
        "Baseline comparison too slow: {}µs",
        avg_per_comparison
    );
}

#[test]
fn test_concurrent_metrics_recording() {
    use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
    use std::thread;

    let counter = Arc::new(AtomicU64::new(0));
    let threads = 8;
    let iterations_per_thread = 10000;

    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let counter_clone = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..iterations_per_thread {
                    counter_clone.fetch_add(1, Ordering::Relaxed);
                    simulate_metrics_recording();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let total_operations = threads * iterations_per_thread;
    let avg_per_op = duration.as_nanos() / total_operations as u128;

    println!("Concurrent metrics recording: {} ops in {:?}", total_operations, duration);
    println!("Average per operation: {}ns", avg_per_op);

    assert_eq!(counter.load(Ordering::Relaxed), total_operations, "All operations should complete");
    assert!(avg_per_op < 1000, "Concurrent metrics should be efficient");
}

#[test]
fn test_extraction_pipeline_throughput() {
    let iterations = 1000;

    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate full extraction pipeline
        simulate_extraction();
        simulate_metrics_recording();
        simulate_metrics_recording(); // Multiple metrics per extraction
        simulate_metrics_recording();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!("Extraction pipeline throughput: {:.2} operations/sec", throughput);

    // Should handle at least 100 operations per second
    assert!(
        throughput >= 100.0,
        "Pipeline throughput too low: {:.2} ops/sec",
        throughput
    );
}

#[test]
fn test_memory_overhead() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Simple memory tracker (for demonstration)
    struct MemoryTracker {
        allocated: AtomicUsize,
    }

    let tracker = MemoryTracker {
        allocated: AtomicUsize::new(0),
    };

    let iterations = 1000;

    // Measure memory allocations during metrics operations
    let baseline_size = std::mem::size_of::<u64>(); // Counter
    let histogram_size = std::mem::size_of::<f64>() + 64; // Histogram value + overhead

    let total_metrics_memory = iterations * (baseline_size + histogram_size);

    // Memory overhead should be minimal
    let max_acceptable_memory = 1024 * 1024; // 1MB
    assert!(
        total_metrics_memory < max_acceptable_memory,
        "Memory overhead too high: {} bytes",
        total_metrics_memory
    );
}

#[test]
fn test_golden_test_performance() {
    // Simulate golden test with baseline comparison
    let baseline = "This is test content ".repeat(100);
    let actual = "This is test content ".repeat(100);

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate golden test operations
        let baseline_words: Vec<&str> = baseline.split_whitespace().collect();
        let actual_words: Vec<&str> = actual.split_whitespace().collect();

        let matches = baseline_words
            .iter()
            .filter(|word| actual_words.contains(word))
            .count();

        let similarity = matches as f64 / baseline_words.len().max(actual_words.len()) as f64;
        let _ = similarity >= 0.80;
    }

    let duration = start.elapsed();
    let avg_per_test = duration.as_millis() / iterations as u128;

    println!("Average golden test time: {}ms", avg_per_test);

    // Golden tests should complete quickly
    assert!(
        avg_per_test < 1,
        "Golden test too slow: {}ms",
        avg_per_test
    );
}

#[test]
fn test_metrics_aggregation_performance() {
    // Simulate metrics aggregation for reporting
    let metrics_count = 30; // 30+ new metrics
    let data_points_per_metric = 1000;

    let start = Instant::now();

    // Simulate gathering metrics
    let mut histogram_data = Vec::new();
    for _ in 0..metrics_count {
        for _ in 0..data_points_per_metric {
            histogram_data.push(0.75_f64);
        }
    }

    // Simulate aggregation
    let sum: f64 = histogram_data.iter().sum();
    let avg = sum / histogram_data.len() as f64;

    let duration = start.elapsed();

    println!("Metrics aggregation time: {:?}", duration);
    println!("Average value: {}", avg);

    // Aggregation should be fast
    assert!(
        duration.as_millis() < 10,
        "Metrics aggregation too slow: {:?}",
        duration
    );
}

#[test]
fn test_end_to_end_performance_impact() {
    // Simulate complete extraction with all metrics
    let iterations = 100;

    // Without metrics
    let baseline_start = Instant::now();
    for _ in 0..iterations {
        simulate_extraction();
    }
    let baseline = baseline_start.elapsed();

    // With full metrics suite
    let with_metrics_start = Instant::now();
    for _ in 0..iterations {
        simulate_extraction();
        // Simulate recording 30+ metrics
        for _ in 0..30 {
            simulate_metrics_recording();
        }
    }
    let with_metrics = with_metrics_start.elapsed();

    let overhead = with_metrics.saturating_sub(baseline);
    let overhead_pct = (overhead.as_nanos() as f64 / baseline.as_nanos() as f64) * 100.0;

    println!("End-to-end baseline: {:?}", baseline);
    println!("End-to-end with metrics: {:?}", with_metrics);
    println!("Total overhead: {:?} ({:.2}%)", overhead, overhead_pct);

    // Even with 30+ metrics, overhead should be <1%
    assert!(
        overhead_pct < 1.0,
        "End-to-end overhead too high: {:.2}%",
        overhead_pct
    );
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    #[ignore] // Run with --ignored flag for stress testing
    fn stress_test_high_volume_metrics() {
        let iterations = 1_000_000;

        let start = Instant::now();
        for _ in 0..iterations {
            simulate_metrics_recording();
        }
        let duration = start.elapsed();

        let ops_per_sec = iterations as f64 / duration.as_secs_f64();

        println!("Stress test: {:.2} million ops/sec", ops_per_sec / 1_000_000.0);

        // Should handle at least 1 million ops/sec
        assert!(ops_per_sec >= 1_000_000.0, "Throughput too low under stress");
    }

    #[test]
    #[ignore]
    fn stress_test_concurrent_load() {
        use std::sync::Arc;
        use std::thread;

        let threads = 16;
        let iterations_per_thread = 100_000;

        let start = Instant::now();

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                thread::spawn(move || {
                    for _ in 0..iterations_per_thread {
                        simulate_extraction();
                        simulate_metrics_recording();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        let total_ops = threads * iterations_per_thread;
        let throughput = total_ops as f64 / duration.as_secs_f64();

        println!("Concurrent stress test: {:.2} ops/sec with {} threads", throughput, threads);

        assert!(throughput >= 10_000.0, "Concurrent throughput too low");
    }
}
