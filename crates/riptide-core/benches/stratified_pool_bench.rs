/// Stratified Instance Pool Performance Benchmarks
///
/// Comprehensive benchmark suite for P2-1 stratified WASM pool implementation.
/// Tests the 3-tier architecture (hot/warm/cold) for expected 40-60% latency improvements.
///
/// Run benchmarks with:
/// ```bash
/// cargo bench --bench stratified_pool_bench --features benchmarks
/// ```
///
/// Generate detailed report:
/// ```bash
/// cargo bench --bench stratified_pool_bench --features benchmarks -- --save-baseline stratified-v1
/// ```
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Import stratified pool components
use riptide_core::memory_manager::{StratifiedInstancePool, TrackedWasmInstance};
use wasmtime::{component::Component, Config, Engine, Store};

/// Baseline SimpleQueue implementation for comparison
struct SimpleQueue {
    queue: VecDeque<TrackedWasmInstance>,
}

impl SimpleQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn acquire(&mut self) -> Option<TrackedWasmInstance> {
        self.queue.pop_front()
    }

    fn release(&mut self, instance: TrackedWasmInstance) {
        self.queue.push_back(instance);
    }
}

/// Helper function to create test WASM instances
fn create_test_instance(engine: &Engine, id: String) -> TrackedWasmInstance {
    let store = Store::new(engine, ());
    // Create a minimal mock component for benchmarking
    let component = create_mock_component(engine);
    let mut instance = TrackedWasmInstance::new(id, store, component);

    // Simulate some usage to set access frequency
    instance.usage_count = 10;
    instance.update_access_frequency();

    instance
}

/// Create a minimal mock component for benchmarking
fn create_mock_component(engine: &Engine) -> Component {
    // Simple WAT module that does nothing
    let wat = r#"
        (component
            (core module $m)
            (core instance $i (instantiate $m))
        )
    "#;
    Component::new(engine, wat).expect("Failed to create mock component")
}

/// Initialize test engine
fn init_test_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(false); // Simpler for benchmarking
    Engine::new(&config).expect("Failed to create engine")
}

/// Benchmark: Hot tier acquisition (target: 0-5ms)
fn bench_hot_tier_acquisition(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_tier_acquisition");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);

    let engine = init_test_engine();

    // Pre-populate hot tier with various sizes
    for hot_size in [1, 2, 4, 8].iter() {
        let mut pool = StratifiedInstancePool::new(*hot_size, 8);

        // Fill hot tier
        for i in 0..*hot_size {
            let instance = create_test_instance(&engine, format!("hot-{}", i));
            pool.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("hot_tier_size", hot_size),
            hot_size,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();
                        let instance = pool.acquire();
                        total_duration += start.elapsed();

                        // Return instance for next iteration
                        if let Some(inst) = instance {
                            pool.release(inst);
                        }
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Warm tier acquisition (target: 10-50ms)
fn bench_warm_tier_acquisition(c: &mut Criterion) {
    let mut group = c.benchmark_group("warm_tier_acquisition");
    group.measurement_time(Duration::from_secs(10));

    let engine = init_test_engine();

    for warm_size in [2, 4, 8, 16].iter() {
        let mut pool = StratifiedInstancePool::new(2, *warm_size);

        // Fill warm tier (hot tier is empty, so acquisitions come from warm)
        for i in 0..*warm_size {
            let mut instance = create_test_instance(&engine, format!("warm-{}", i));
            instance.pool_tier = 1;
            instance.access_frequency = 0.3; // Moderate frequency
            pool.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("warm_tier_size", warm_size),
            warm_size,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();
                        let instance = pool.acquire();
                        total_duration += start.elapsed();

                        if let Some(inst) = instance {
                            pool.release(inst);
                        }
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Cold tier acquisition (baseline: 100-200ms simulated)
fn bench_cold_tier_acquisition(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_tier_acquisition");

    let engine = init_test_engine();

    for cold_size in [4, 8, 16].iter() {
        let mut pool = StratifiedInstancePool::new(2, 4);

        // Fill cold tier
        for i in 0..*cold_size {
            let mut instance = create_test_instance(&engine, format!("cold-{}", i));
            instance.pool_tier = 2;
            instance.access_frequency = 0.1; // Low frequency
            pool.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("cold_tier_size", cold_size),
            cold_size,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();
                        let instance = pool.acquire();
                        total_duration += start.elapsed();

                        if let Some(inst) = instance {
                            pool.release(inst);
                        }
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Promotion effectiveness
fn bench_promotion_effectiveness(c: &mut Criterion) {
    let mut group = c.benchmark_group("promotion_effectiveness");

    let engine = init_test_engine();
    let mut pool = StratifiedInstancePool::new(4, 8);

    // Create instances with varying access patterns
    for i in 0..12 {
        let mut instance = create_test_instance(&engine, format!("promo-{}", i));
        instance.access_frequency = 0.1 + (i as f64 * 0.05); // Gradual increase
        pool.release(instance);
    }

    group.bench_function("promote_warm_to_hot", |b| {
        b.iter(|| {
            black_box(pool.promote_warm_to_hot());
        });
    });

    group.finish();
}

/// Benchmark: Hit rate tracking accuracy
fn bench_hit_rate_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_rate_tracking");
    group.throughput(Throughput::Elements(100));

    let engine = init_test_engine();

    // Test different tier distributions
    let scenarios = vec![
        ("hot_dominant", 8, 4, 2),   // 70% hot tier capacity
        ("balanced", 4, 8, 4),       // Balanced distribution
        ("cold_dominant", 2, 4, 10), // More cold tier
    ];

    for (scenario_name, hot_cap, warm_cap, cold_count) in scenarios {
        let mut pool = StratifiedInstancePool::new(hot_cap, warm_cap);

        // Populate tiers
        for i in 0..hot_cap {
            let mut instance =
                create_test_instance(&engine, format!("{}-hot-{}", scenario_name, i));
            instance.access_frequency = 0.8;
            pool.release(instance);
        }

        for i in 0..warm_cap {
            let mut instance =
                create_test_instance(&engine, format!("{}-warm-{}", scenario_name, i));
            instance.access_frequency = 0.4;
            pool.release(instance);
        }

        for i in 0..cold_count {
            let mut instance =
                create_test_instance(&engine, format!("{}-cold-{}", scenario_name, i));
            instance.access_frequency = 0.1;
            pool.release(instance);
        }

        group.bench_function(scenario_name, |b| {
            b.iter(|| {
                // Simulate 100 acquisitions
                for _ in 0..100 {
                    if let Some(instance) = pool.acquire() {
                        pool.release(instance);
                    }
                }

                // Get metrics
                black_box(pool.metrics());
            });
        });
    }

    group.finish();
}

/// Benchmark: Stratified pool vs Simple queue comparison
fn bench_stratified_vs_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("stratified_vs_baseline");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(500);

    let engine = init_test_engine();

    // Access patterns: hot-biased (70%), uniform (50%), cold-start (10%)
    let access_patterns = vec![
        ("hot_biased", vec![0.7, 0.2, 0.1]), // 70% hot, 20% warm, 10% cold
        ("uniform", vec![0.33, 0.33, 0.34]), // Uniform distribution
        ("cold_start", vec![0.1, 0.3, 0.6]), // Cold start scenario
    ];

    for (pattern_name, tier_probabilities) in access_patterns {
        // Benchmark stratified pool
        let mut stratified_pool = StratifiedInstancePool::new(4, 8);

        // Populate with instances simulating the access pattern
        for i in 0..16 {
            let mut instance =
                create_test_instance(&engine, format!("strat-{}-{}", pattern_name, i));

            // Set access frequency based on pattern
            if i < 4 {
                instance.access_frequency = tier_probabilities[0];
            } else if i < 12 {
                instance.access_frequency = tier_probabilities[1];
            } else {
                instance.access_frequency = tier_probabilities[2];
            }

            stratified_pool.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("stratified", pattern_name),
            pattern_name,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();

                        // Acquire and release
                        if let Some(instance) = stratified_pool.acquire() {
                            total_duration += start.elapsed();
                            stratified_pool.release(instance);
                        }
                    }

                    total_duration
                });
            },
        );

        // Benchmark baseline simple queue
        let mut simple_queue = SimpleQueue::new();

        for i in 0..16 {
            let instance = create_test_instance(&engine, format!("simple-{}-{}", pattern_name, i));
            simple_queue.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("simple_queue", pattern_name),
            pattern_name,
            |b, _| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::ZERO;

                    for _ in 0..iters {
                        let start = Instant::now();

                        // Acquire and release
                        if let Some(instance) = simple_queue.acquire() {
                            total_duration += start.elapsed();
                            simple_queue.release(instance);
                        }
                    }

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Load patterns (sequential, random, burst)
fn bench_load_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_patterns");
    group.measurement_time(Duration::from_secs(10));

    let engine = init_test_engine();

    // Sequential access pattern
    group.bench_function("sequential_access", |b| {
        let mut pool = StratifiedInstancePool::new(4, 8);

        // Pre-populate
        for i in 0..12 {
            let mut instance = create_test_instance(&engine, format!("seq-{}", i));
            instance.access_frequency = 0.5;
            pool.release(instance);
        }

        b.iter(|| {
            // Sequential acquire/release pattern
            for _ in 0..100 {
                if let Some(instance) = pool.acquire() {
                    pool.release(instance);
                }
            }
        });
    });

    // Burst pattern
    group.bench_function("burst_access", |b| {
        let mut pool = StratifiedInstancePool::new(4, 8);

        for i in 0..12 {
            let mut instance = create_test_instance(&engine, format!("burst-{}", i));
            instance.access_frequency = 0.5;
            pool.release(instance);
        }

        b.iter(|| {
            let mut held_instances = Vec::new();

            // Burst acquire
            for _ in 0..5 {
                if let Some(instance) = pool.acquire() {
                    held_instances.push(instance);
                }
            }

            // Burst release
            for instance in held_instances.drain(..) {
                pool.release(instance);
            }
        });
    });

    group.finish();
}

/// Benchmark: Metrics overhead
fn bench_metrics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_overhead");

    let engine = init_test_engine();
    let mut pool = StratifiedInstancePool::new(4, 8);

    for i in 0..12 {
        let instance = create_test_instance(&engine, format!("metrics-{}", i));
        pool.release(instance);
    }

    group.bench_function("get_metrics", |b| {
        b.iter(|| {
            black_box(pool.metrics());
        });
    });

    group.bench_function("acquire_with_metrics", |b| {
        b.iter(|| {
            if let Some(instance) = pool.acquire() {
                let _ = black_box(pool.metrics());
                pool.release(instance);
            }
        });
    });

    group.finish();
}

/// Benchmark: Concurrent access simulation
fn bench_concurrent_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_simulation");

    let engine = init_test_engine();

    // Simulate concurrent access by rapidly acquiring/releasing
    for thread_count in [2, 4, 8].iter() {
        let mut pool = StratifiedInstancePool::new(4, 8);

        for i in 0..16 {
            let instance = create_test_instance(&engine, format!("concurrent-{}", i));
            pool.release(instance);
        }

        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    // Simulate multiple threads by doing multiple acquire/release cycles
                    for _ in 0..(threads * 10) {
                        if let Some(instance) = pool.acquire() {
                            pool.release(instance);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory efficiency
fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");

    let engine = init_test_engine();

    for total_instances in [8, 16, 32, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("total_instances", total_instances),
            total_instances,
            |b, &count| {
                b.iter(|| {
                    let mut pool = StratifiedInstancePool::new(
                        count / 4, // 25% hot
                        count / 2, // 50% warm
                    );

                    // Create and manage instances
                    for i in 0..count {
                        let instance = create_test_instance(&engine, format!("mem-{}", i));
                        pool.release(instance);
                    }

                    // Get metrics to measure overhead
                    black_box(pool.metrics());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    stratified_pool_benches,
    bench_hot_tier_acquisition,
    bench_warm_tier_acquisition,
    bench_cold_tier_acquisition,
    bench_promotion_effectiveness,
    bench_hit_rate_tracking,
    bench_stratified_vs_baseline,
    bench_load_patterns,
    bench_metrics_overhead,
    bench_concurrent_simulation,
    bench_memory_efficiency
);

criterion_main!(stratified_pool_benches);
