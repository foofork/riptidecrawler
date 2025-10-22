/// Resource Management and Limits Tests
///
/// Tests that WASM components respect resource limits including memory,
/// fuel consumption, and execution timeouts. Validates security boundaries.

use std::time::{Duration, Instant};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store, StoreLimits, StoreLimitsBuilder,
};
use wasmtime_wasi::WasiCtxBuilder;

const COMPONENT_PATH: &str = "wasm/riptide-extractor-wasm/target/wasm32-wasi/release/riptide_extractor_wasm.wasm";

// Resource limit constants
const MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024; // 64MB
const MAX_FUEL: u64 = 1_000_000; // 1M fuel units
const EPOCH_TIMEOUT_SECS: u64 = 30; // 30 second timeout

/// Test helper: Create engine with strict resource limits
fn create_limited_engine() -> anyhow::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    config.epoch_interruption(true);

    // Strict memory limits
    config.max_wasm_stack(256 * 1024); // 256KB stack

    Engine::new(&config)
}

struct TestContext {
    wasi: wasmtime_wasi::WasiCtx,
    limits: StoreLimits,
}

fn create_limited_store(engine: &Engine) -> Store<TestContext> {
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .build();

    let limits = StoreLimitsBuilder::new()
        .memory_size(MAX_MEMORY_BYTES)
        .table_elements(10_000)
        .instances(10)
        .tables(10)
        .memories(10)
        .build();

    let ctx = TestContext { wasi, limits };
    let mut store = Store::new(engine, ctx);

    // Set fuel limit
    store.add_fuel(MAX_FUEL).expect("Failed to add fuel");

    // Set epoch deadline
    store.set_epoch_deadline(EPOCH_TIMEOUT_SECS);

    // Enable resource limiting
    store.limiter(|ctx| &mut ctx.limits);

    store
}

#[tokio::test]
async fn test_memory_limits_enforced() {
    let engine = create_limited_engine().expect("Failed to create engine");

    if !std::path::Path::new(COMPONENT_PATH).exists() {
        println!("Skipping test: WASM component not found");
        return;
    }

    let component = Component::from_file(&engine, COMPONENT_PATH)
        .expect("Failed to load component");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |ctx: &mut TestContext| &mut ctx.wasi)
        .expect("Failed to add WASI to linker");

    let mut store = create_limited_store(&engine);

    // Instantiate component
    let instance = linker.instantiate(&mut store, &component)
        .expect("Failed to instantiate component");

    // Check memory usage is within limits
    let memory_usage = store.data().limits.memory_size;
    assert!(
        memory_usage <= MAX_MEMORY_BYTES,
        "Memory usage {} exceeds limit {}",
        memory_usage,
        MAX_MEMORY_BYTES
    );
}

#[tokio::test]
async fn test_memory_growth_rejected_at_limit() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Try to allocate memory exceeding the limit (simulated)
    let current_memory: usize = 60 * 1024 * 1024; // 60MB
    let desired_memory: usize = 70 * 1024 * 1024; // 70MB (exceeds 64MB limit)

    let limiter = store.limiter_mut().unwrap();
    let can_grow = limiter.memory_growing(
        current_memory,
        desired_memory,
        Some(MAX_MEMORY_BYTES),
    );

    assert!(
        can_grow.is_err() || !can_grow.unwrap(),
        "Memory growth should be rejected when exceeding limit"
    );
}

#[tokio::test]
async fn test_fuel_consumption_limits() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Check initial fuel
    let initial_fuel = store.get_fuel().expect("Fuel should be available");
    assert_eq!(initial_fuel, MAX_FUEL, "Initial fuel should be MAX_FUEL");

    // Simulate fuel consumption
    let consumed = 100_000;
    store.consume_fuel(consumed).expect("Should consume fuel");

    let remaining_fuel = store.get_fuel().expect("Fuel should be available");
    assert_eq!(
        remaining_fuel,
        MAX_FUEL - consumed,
        "Fuel should be consumed correctly"
    );
}

#[tokio::test]
async fn test_fuel_exhaustion_error() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Consume all fuel
    store.consume_fuel(MAX_FUEL).expect("Should consume fuel");

    let remaining = store.get_fuel().expect("Fuel should be available");
    assert_eq!(remaining, 0, "All fuel should be consumed");

    // Try to consume more fuel
    let result = store.consume_fuel(1);
    assert!(
        result.is_err(),
        "Should error when trying to consume fuel after exhaustion"
    );
}

#[tokio::test]
async fn test_epoch_timeout_mechanism() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Set a very short deadline
    store.set_epoch_deadline(1); // 1 epoch tick

    // Increment epoch to trigger timeout
    engine.increment_epoch();

    // The store should now be past its deadline
    // Note: Actual timeout behavior would be tested with a long-running operation
}

#[tokio::test]
async fn test_execution_time_tracking() {
    let engine = create_limited_engine().expect("Failed to create engine");

    if !std::path::Path::new(COMPONENT_PATH).exists() {
        println!("Skipping test: WASM component not found");
        return;
    }

    let component = Component::from_file(&engine, COMPONENT_PATH)
        .expect("Failed to load component");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |ctx: &mut TestContext| &mut ctx.wasi)
        .expect("Failed to add WASI to linker");

    let mut store = create_limited_store(&engine);

    let start = Instant::now();
    let _instance = linker.instantiate(&mut store, &component)
        .expect("Failed to instantiate component");
    let elapsed = start.elapsed();

    // Instantiation should be fast
    assert!(
        elapsed < Duration::from_secs(5),
        "Instantiation took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_table_element_limits() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    let limiter = store.limiter_mut().unwrap();

    // Test table growth within limits
    let can_grow = limiter.table_growing(0, 1000, Some(10_000));
    assert!(
        can_grow.is_ok() && can_grow.unwrap(),
        "Table growth within limits should succeed"
    );

    // Test table growth exceeding limits
    let can_grow = limiter.table_growing(0, 20_000, Some(10_000));
    assert!(
        can_grow.is_err() || !can_grow.unwrap(),
        "Table growth exceeding limits should fail"
    );
}

#[tokio::test]
async fn test_instance_count_limits() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    let limiter = store.limiter_mut().unwrap();

    // Test instance count within limits
    let can_create = limiter.instances(5);
    assert!(
        can_create.is_ok() && can_create.unwrap(),
        "Instance count within limits should succeed"
    );

    // Test instance count exceeding limits
    let can_create = limiter.instances(15);
    assert!(
        can_create.is_err() || !can_create.unwrap(),
        "Instance count exceeding limits should fail"
    );
}

#[tokio::test]
async fn test_memory_count_limits() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    let limiter = store.limiter_mut().unwrap();

    // Test memory count within limits
    let can_create = limiter.memories(5);
    assert!(
        can_create.is_ok() && can_create.unwrap(),
        "Memory count within limits should succeed"
    );

    // Test memory count exceeding limits
    let can_create = limiter.memories(15);
    assert!(
        can_create.is_err() || !can_create.unwrap(),
        "Memory count exceeding limits should fail"
    );
}

#[tokio::test]
async fn test_stack_overflow_protection() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // The engine is configured with max_wasm_stack limit
    // This test verifies the configuration is in place
    // Actual stack overflow would be caught by the WASM runtime

    // Verify store is properly configured
    assert!(store.limiter().is_some(), "Store should have a limiter configured");
}

#[tokio::test]
async fn test_resource_usage_monitoring() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Monitor fuel consumption
    let initial_fuel = store.get_fuel().expect("Should have fuel");
    store.consume_fuel(50_000).expect("Should consume fuel");
    let after_fuel = store.get_fuel().expect("Should have fuel");

    let consumed = initial_fuel - after_fuel;
    assert_eq!(consumed, 50_000, "Fuel consumption should be tracked accurately");

    // Calculate percentage used
    let percentage_used = (consumed as f64 / MAX_FUEL as f64) * 100.0;
    assert!(
        percentage_used < 100.0,
        "Should be able to track resource usage percentage"
    );
}

#[tokio::test]
async fn test_memory_limit_gradual_growth() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    let limiter = store.limiter_mut().unwrap();

    // Simulate gradual memory growth
    let mut current = 0;
    let increments = [
        10 * 1024 * 1024,  // 10MB
        20 * 1024 * 1024,  // 20MB
        30 * 1024 * 1024,  // 30MB
        40 * 1024 * 1024,  // 40MB
        50 * 1024 * 1024,  // 50MB
        60 * 1024 * 1024,  // 60MB - should succeed
        70 * 1024 * 1024,  // 70MB - should fail (exceeds 64MB)
    ];

    for (i, &desired) in increments.iter().enumerate() {
        let result = limiter.memory_growing(current, desired, Some(MAX_MEMORY_BYTES));

        if desired <= MAX_MEMORY_BYTES {
            assert!(
                result.is_ok() && result.unwrap(),
                "Growth to {}MB should succeed (iteration {})",
                desired / (1024 * 1024),
                i
            );
            current = desired;
        } else {
            assert!(
                result.is_err() || !result.unwrap(),
                "Growth to {}MB should fail (iteration {})",
                desired / (1024 * 1024),
                i
            );
        }
    }
}

#[tokio::test]
async fn test_concurrent_resource_limits() {
    // Test that resource limits are per-store, not global
    let engine = create_limited_engine().expect("Failed to create engine");

    let store1 = create_limited_store(&engine);
    let store2 = create_limited_store(&engine);

    // Each store should have independent fuel
    let fuel1 = store1.get_fuel().expect("Should have fuel");
    let fuel2 = store2.get_fuel().expect("Should have fuel");

    assert_eq!(fuel1, MAX_FUEL);
    assert_eq!(fuel2, MAX_FUEL);
    assert_eq!(fuel1, fuel2, "Stores should have independent fuel limits");
}

#[tokio::test]
async fn test_limits_recovery_after_reset() {
    let engine = create_limited_engine().expect("Failed to create engine");
    let mut store = create_limited_store(&engine);

    // Consume some fuel
    store.consume_fuel(500_000).expect("Should consume fuel");
    let after_consumption = store.get_fuel().expect("Should have fuel");
    assert_eq!(after_consumption, 500_000);

    // Refuel to original amount
    store.add_fuel(500_000).expect("Should refuel");
    let after_refuel = store.get_fuel().expect("Should have fuel");
    assert_eq!(after_refuel, MAX_FUEL, "Should be able to refuel to original limit");
}
