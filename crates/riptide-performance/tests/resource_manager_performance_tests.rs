//! Performance and concurrency tests for ResourceManager
//!
//! NOTE: These tests require riptide-api which is not available as a dependency.
//! These tests are disabled until the dependency structure is updated.
//!
//! Tests cover:
//! - Throughput benchmarks
//! - Latency measurements
//! - Concurrency limits
//! - Resource utilization
//! - Scalability

// Tests disabled due to missing riptide-api dependency
#![cfg(all(test, feature = "integration-tests-disabled"))]

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// All tests disabled due to missing riptide-api dependency
#[allow(dead_code)]
async fn perf_resource_acquisition_latency() -> Result<()> {
    // Measure latency of resource acquisition
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let iterations = 100;
    let mut latencies = Vec::new();

    for i in 0..iterations {
        let start = Instant::now();

        let result = manager
            .acquire_render_resources(&format!("https://perf{}.com", i))
            .await;

        let latency = start.elapsed();
        latencies.push(latency);

        if let Ok(ResourceResult::Success(guard)) = result {
            drop(guard);
        }

        // Small delay to avoid rate limiting
        sleep(Duration::from_millis(50)).await;
    }

    // Calculate statistics
    let avg_latency = latencies.iter().sum::<Duration>() / iterations as u32;
    let max_latency = latencies.iter().max().unwrap();
    let min_latency = latencies.iter().min().unwrap();

    println!("Acquisition Latency:");
    println!("  Average: {:?}", avg_latency);
    println!("  Min: {:?}", min_latency);
    println!("  Max: {:?}", max_latency);

    // Verify reasonable latency (adjust based on system)
    assert!(avg_latency < Duration::from_secs(1));

    Ok(())
}

#[tokio::test]
async fn perf_concurrent_throughput() -> Result<()> {
    // Measure throughput with concurrent requests
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let concurrent_requests = 100;
    let success_count = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let handles: Vec<_> = (0..concurrent_requests)
        .map(|i| {
            let mgr = Arc::clone(&manager);
            let counter = Arc::clone(&success_count);
            tokio::spawn(async move {
                match mgr
                    .acquire_render_resources(&format!("https://throughput{}.com", i))
                    .await
                {
                    Ok(ResourceResult::Success(_)) => {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {}
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();
    let success = success_count.load(Ordering::Relaxed);

    let throughput = success as f64 / duration.as_secs_f64();

    println!("Concurrent Throughput:");
    println!("  Total requests: {}", concurrent_requests);
    println!("  Successful: {}", success);
    println!("  Duration: {:?}", duration);
    println!("  Throughput: {:.2} req/s", throughput);

    assert!(success > 0, "Some requests should succeed");

    Ok(())
}

#[tokio::test]
async fn perf_pdf_semaphore_contention() -> Result<()> {
    // Test PDF semaphore under contention
    let mut config = ApiConfig::default();
    config.pdf.max_concurrent = 5;

    let manager = Arc::new(ResourceManager::new(config).await?);

    let requests = 50;
    let start = Instant::now();
    let success_count = Arc::new(AtomicU64::new(0));

    let handles: Vec<_> = (0..requests)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            let counter = Arc::clone(&success_count);
            tokio::spawn(async move {
                if let Ok(ResourceResult::Success(guard)) = mgr.acquire_pdf_resources().await {
                    counter.fetch_add(1, Ordering::Relaxed);
                    sleep(Duration::from_millis(10)).await;
                    drop(guard);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();
    let success = success_count.load(Ordering::Relaxed);

    println!("PDF Semaphore Contention:");
    println!("  Requests: {}", requests);
    println!("  Successful: {}", success);
    println!("  Duration: {:?}", duration);
    println!(
        "  Throughput: {:.2} req/s",
        success as f64 / duration.as_secs_f64()
    );

    Ok(())
}

#[tokio::test]
async fn perf_memory_tracking_overhead() -> Result<()> {
    // Measure overhead of memory tracking
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        manager.memory_manager.track_allocation(1).await;
        manager.memory_manager.track_deallocation(1).await;
    }

    let duration = start.elapsed();
    let ops_per_sec = (iterations * 2) as f64 / duration.as_secs_f64();

    println!("Memory Tracking Performance:");
    println!("  Operations: {}", iterations * 2);
    println!("  Duration: {:?}", duration);
    println!("  Ops/sec: {:.0}", ops_per_sec);

    // Should be very fast (millions of ops/sec)
    assert!(ops_per_sec > 10000.0);

    Ok(())
}

#[tokio::test]
async fn perf_rate_limiter_throughput() -> Result<()> {
    // Measure rate limiter throughput
    let mut config = ApiConfig::default();
    config.rate_limiting.requests_per_second_per_host = 100.0;
    config.rate_limiting.burst_capacity_per_host = 200.0;
    config.rate_limiting.enabled = true;

    let manager = ResourceManager::new(config).await?;

    let iterations = 500;
    let start = Instant::now();
    let mut success_count = 0;

    for _ in 0..iterations {
        if manager
            .rate_limiter
            .check_rate_limit("perftest.com")
            .await
            .is_ok()
        {
            success_count += 1;
        }
    }

    let duration = start.elapsed();

    println!("Rate Limiter Throughput:");
    println!("  Requests: {}", iterations);
    println!("  Successful: {}", success_count);
    println!("  Duration: {:?}", duration);
    println!(
        "  Throughput: {:.2} req/s",
        success_count as f64 / duration.as_secs_f64()
    );

    Ok(())
}

#[tokio::test]
async fn perf_wasm_instance_lookup() -> Result<()> {
    // Measure WASM instance lookup performance
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    // Create instances
    for i in 0..10 {
        manager
            .wasm_manager
            .test_acquire_instance(&format!("worker_{}", i))
            .await?;
    }

    let iterations = 1000;
    let start = Instant::now();

    for i in 0..iterations {
        let worker_id = format!("worker_{}", i % 10);
        manager
            .wasm_manager
            .test_acquire_instance(&worker_id)
            .await?;
    }

    let duration = start.elapsed();
    let ops_per_sec = iterations as f64 / duration.as_secs_f64();

    println!("WASM Instance Lookup:");
    println!("  Lookups: {}", iterations);
    println!("  Duration: {:?}", duration);
    println!("  Ops/sec: {:.0}", ops_per_sec);

    assert!(ops_per_sec > 100.0);

    Ok(())
}

#[tokio::test]
async fn perf_resource_status_query() -> Result<()> {
    // Measure resource status query performance
    let config = ApiConfig::default();
    let manager = ResourceManager::new(config).await?;

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = manager.get_resource_status().await;
    }

    let duration = start.elapsed();
    let queries_per_sec = iterations as f64 / duration.as_secs_f64();

    println!("Resource Status Query:");
    println!("  Queries: {}", iterations);
    println!("  Duration: {:?}", duration);
    println!("  Queries/sec: {:.0}", queries_per_sec);

    assert!(queries_per_sec > 100.0);

    Ok(())
}

#[tokio::test]
async fn perf_concurrent_memory_operations() -> Result<()> {
    // Test concurrent memory operation performance
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let workers = 10;
    let ops_per_worker = 1000;
    let start = Instant::now();

    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let mgr = Arc::clone(&manager);
            tokio::spawn(async move {
                for _ in 0..ops_per_worker {
                    mgr.memory_manager.track_allocation(1).await;
                    mgr.memory_manager.track_deallocation(1).await;
                }
            })
        })
        .collect();

    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();
    let total_ops = workers * ops_per_worker * 2;
    let ops_per_sec = total_ops as f64 / duration.as_secs_f64();

    println!("Concurrent Memory Operations:");
    println!("  Workers: {}", workers);
    println!("  Ops per worker: {}", ops_per_worker * 2);
    println!("  Total ops: {}", total_ops);
    println!("  Duration: {:?}", duration);
    println!("  Ops/sec: {:.0}", ops_per_sec);

    Ok(())
}

#[tokio::test]
async fn perf_mixed_workload_simulation() -> Result<()> {
    // Simulate realistic mixed workload
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let duration_secs = 2;
    let start = Instant::now();

    let render_counter = Arc::new(AtomicU64::new(0));
    let pdf_counter = Arc::new(AtomicU64::new(0));
    let memory_counter = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();

    // Render workers
    for i in 0..5 {
        let mgr = Arc::clone(&manager);
        let counter = Arc::clone(&render_counter);
        handles.push(tokio::spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(duration_secs);
            while Instant::now() < deadline {
                if let Ok(ResourceResult::Success(_)) = mgr
                    .acquire_render_resources(&format!("https://mixed{}.com", i))
                    .await
                {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
                sleep(Duration::from_millis(100)).await;
            }
        }));
    }

    // PDF workers
    for _ in 0..3 {
        let mgr = Arc::clone(&manager);
        let counter = Arc::clone(&pdf_counter);
        handles.push(tokio::spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(duration_secs);
            while Instant::now() < deadline {
                if let Ok(ResourceResult::Success(_)) = mgr.acquire_pdf_resources().await {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
                sleep(Duration::from_millis(150)).await;
            }
        }));
    }

    // Memory workers
    for _ in 0..2 {
        let mgr = Arc::clone(&manager);
        let counter = Arc::clone(&memory_counter);
        handles.push(tokio::spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(duration_secs);
            while Instant::now() < deadline {
                mgr.memory_manager.track_allocation(10).await;
                sleep(Duration::from_millis(50)).await;
                mgr.memory_manager.track_deallocation(10).await;
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();

    println!("Mixed Workload Simulation:");
    println!("  Duration: {:?}", duration);
    println!("  Render ops: {}", render_counter.load(Ordering::Relaxed));
    println!("  PDF ops: {}", pdf_counter.load(Ordering::Relaxed));
    println!("  Memory ops: {}", memory_counter.load(Ordering::Relaxed));

    let status = manager.get_resource_status().await;
    println!("  Final status: {:?}", status);

    Ok(())
}

#[tokio::test]
async fn perf_scalability_test() -> Result<()> {
    // Test scalability with increasing load
    let config = ApiConfig::default();
    let manager = Arc::new(ResourceManager::new(config).await?);

    let load_levels = vec![10, 50, 100, 200];

    for load in load_levels {
        let start = Instant::now();
        let success_count = Arc::new(AtomicU64::new(0));

        let handles: Vec<_> = (0..load)
            .map(|_| {
                let mgr = Arc::clone(&manager);
                let counter = Arc::clone(&success_count);
                tokio::spawn(async move {
                    mgr.memory_manager.track_allocation(1).await;
                    sleep(Duration::from_millis(10)).await;
                    mgr.memory_manager.track_deallocation(1).await;
                    counter.fetch_add(1, Ordering::Relaxed);
                })
            })
            .collect();

        for handle in handles {
            handle.await?;
        }

        let duration = start.elapsed();
        let throughput = success_count.load(Ordering::Relaxed) as f64 / duration.as_secs_f64();

        println!("Scalability Test - Load {}:", load);
        println!("  Duration: {:?}", duration);
        println!("  Throughput: {:.2} ops/s", throughput);
    }

    Ok(())
}
