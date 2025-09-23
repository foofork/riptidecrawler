use anyhow::Result;
use riptide_core::memory_manager::{MemoryManager, MemoryManagerConfig};
use riptide_headless::launcher::{HeadlessLauncher, LauncherConfig};
use riptide_headless::pool::{BrowserPool, BrowserPoolConfig};
use std::time::Duration;
use tokio::time::sleep;
use wasmtime::{Config, Engine};

#[tokio::test]
async fn test_browser_pool_basic_operations() -> Result<()> {
    // Create a browser pool with minimal configuration for testing
    let config = BrowserPoolConfig {
        min_pool_size: 1,
        max_pool_size: 3,
        initial_pool_size: 2,
        idle_timeout: Duration::from_secs(10),
        max_lifetime: Duration::from_secs(60),
        health_check_interval: Duration::from_secs(5),
        memory_threshold_mb: 100,
        enable_recovery: true,
        max_retries: 2,
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .arg("--headless")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Test initial pool state
    let stats = pool.stats().await;
    assert_eq!(stats.available, 2);
    assert_eq!(stats.in_use, 0);
    assert_eq!(stats.total_capacity, 3);

    // Test browser checkout
    let checkout1 = pool.checkout().await?;
    let stats = pool.stats().await;
    assert_eq!(stats.available, 1);
    assert_eq!(stats.in_use, 1);

    // Test browser checkin
    let browser_id = checkout1.browser_id().to_string();
    checkout1.checkin().await?;
    let stats = pool.stats().await;
    assert_eq!(stats.available, 2);
    assert_eq!(stats.in_use, 0);

    // Test multiple checkouts
    let checkout1 = pool.checkout().await?;
    let checkout2 = pool.checkout().await?;
    let checkout3 = pool.checkout().await?;

    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 3);
    assert_eq!(stats.utilization, 100.0);

    // Clean up
    checkout1.checkin().await?;
    checkout2.checkin().await?;
    checkout3.checkin().await?;

    pool.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_memory_manager_basic_operations() -> Result<()> {
    // Create memory manager configuration
    let config = MemoryManagerConfig {
        max_total_memory_mb: 512,
        instance_memory_threshold_mb: 128,
        max_instances: 4,
        min_instances: 1,
        instance_idle_timeout: Duration::from_secs(30),
        monitoring_interval: Duration::from_secs(2),
        gc_interval: Duration::from_secs(5),
        aggressive_gc: false,
        memory_pressure_threshold: 80.0,
    };

    // Create Wasmtime engine
    let mut wasmtime_config = Config::new();
    wasmtime_config.wasm_component_model(true);
    wasmtime_config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    let engine = Engine::new(&wasmtime_config)?;

    let memory_manager = MemoryManager::new(config, engine).await?;

    // Test initial state
    let stats = memory_manager.stats();
    assert_eq!(stats.instances_count, 0);
    assert_eq!(stats.active_instances, 0);
    assert_eq!(stats.total_used_mb, 0);

    // Note: We can't easily test instance creation without a real WASM component file
    // But we can test the memory manager's basic functionality

    // Test garbage collection trigger
    memory_manager.trigger_garbage_collection().await;

    // Test stats after GC
    let stats_after_gc = memory_manager.stats();
    assert!(stats_after_gc.gc_runs >= 1);

    // Clean up
    memory_manager.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_headless_launcher_configuration() -> Result<()> {
    // Create launcher configuration
    let config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: 2,
            initial_pool_size: 1,
            idle_timeout: Duration::from_secs(30),
            max_lifetime: Duration::from_secs(120),
            health_check_interval: Duration::from_secs(10),
            memory_threshold_mb: 200,
            enable_recovery: true,
            max_retries: 3,
        },
        default_stealth_preset: riptide_core::stealth::StealthPreset::Low,
        enable_stealth: true,
        page_timeout: Duration::from_secs(15),
        enable_monitoring: true,
    };

    let launcher = HeadlessLauncher::with_config(config).await?;

    // Test initial statistics
    let stats = launcher.stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);

    // Note: We can't easily test page launching without a real browser environment
    // But we can test the launcher's configuration and initialization

    // Clean up
    launcher.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_memory_pressure_simulation() -> Result<()> {
    // Create memory manager with low thresholds for testing
    let config = MemoryManagerConfig {
        max_total_memory_mb: 100, // Very low for testing
        instance_memory_threshold_mb: 50,
        max_instances: 2,
        min_instances: 1,
        instance_idle_timeout: Duration::from_secs(5),
        monitoring_interval: Duration::from_millis(500),
        gc_interval: Duration::from_secs(1),
        aggressive_gc: true,
        memory_pressure_threshold: 50.0, // Low threshold for testing
    };

    let mut wasmtime_config = Config::new();
    wasmtime_config.wasm_component_model(true);
    let engine = Engine::new(&wasmtime_config)?;

    let memory_manager = MemoryManager::new(config, engine).await?;

    // Monitor for a short period to test GC activation
    sleep(Duration::from_secs(3)).await;

    let stats = memory_manager.stats();
    println!("Memory stats after monitoring: {:?}", stats);

    // Verify GC has been triggered
    assert!(stats.gc_runs >= 2); // Should have run at least twice

    memory_manager.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_browser_pool_health_checks() -> Result<()> {
    let config = BrowserPoolConfig {
        min_pool_size: 1,
        max_pool_size: 2,
        initial_pool_size: 1,
        idle_timeout: Duration::from_secs(5),
        max_lifetime: Duration::from_secs(30),
        health_check_interval: Duration::from_secs(2),
        memory_threshold_mb: 50, // Low threshold to trigger alerts
        enable_recovery: true,
        max_retries: 1,
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .arg("--headless")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Let health checks run for a few cycles
    sleep(Duration::from_secs(6)).await;

    let stats = pool.stats().await;
    println!("Pool stats after health checks: {:?}", stats);

    // Verify pool is still operational
    assert!(stats.total_capacity > 0);

    pool.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_resource_cleanup_on_timeout() -> Result<()> {
    let config = BrowserPoolConfig {
        min_pool_size: 1,
        max_pool_size: 2,
        initial_pool_size: 1,
        idle_timeout: Duration::from_secs(3), // Short timeout for testing
        max_lifetime: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(1),
        memory_threshold_mb: 100,
        enable_recovery: true,
        max_retries: 1,
    };

    let browser_config = chromiumoxide::BrowserConfig::builder()
        .arg("--headless")
        .arg("--no-sandbox")
        .arg("--disable-gpu")
        .build()?;

    let pool = BrowserPool::new(config, browser_config).await?;

    // Check initial state
    let initial_stats = pool.stats().await;
    println!("Initial stats: {:?}", initial_stats);

    // Wait for idle timeout to trigger cleanup
    sleep(Duration::from_secs(8)).await;

    let final_stats = pool.stats().await;
    println!("Final stats: {:?}", final_stats);

    // Verify cleanup behavior (instances may be recreated to maintain min_pool_size)
    assert!(final_stats.available >= 1); // Should maintain minimum

    pool.shutdown().await?;
    Ok(())
}

#[tokio::test]
async fn test_integration_browser_and_memory_coordination() -> Result<()> {
    // Test coordination between browser pool and memory manager
    let launcher_config = LauncherConfig {
        pool_config: BrowserPoolConfig {
            min_pool_size: 1,
            max_pool_size: 3,
            initial_pool_size: 2,
            idle_timeout: Duration::from_secs(15),
            max_lifetime: Duration::from_secs(60),
            health_check_interval: Duration::from_secs(3),
            memory_threshold_mb: 150,
            enable_recovery: true,
            max_retries: 2,
        },
        default_stealth_preset: riptide_core::stealth::StealthPreset::Medium,
        enable_stealth: true,
        page_timeout: Duration::from_secs(20),
        enable_monitoring: true,
    };

    let launcher = HeadlessLauncher::with_config(launcher_config).await?;

    // Create memory manager for WASM components
    let memory_config = MemoryManagerConfig {
        max_total_memory_mb: 256,
        instance_memory_threshold_mb: 64,
        max_instances: 4,
        min_instances: 1,
        instance_idle_timeout: Duration::from_secs(20),
        monitoring_interval: Duration::from_secs(2),
        gc_interval: Duration::from_secs(10),
        aggressive_gc: false,
        memory_pressure_threshold: 75.0,
    };

    let mut wasmtime_config = Config::new();
    wasmtime_config.wasm_component_model(true);
    let engine = Engine::new(&wasmtime_config)?;

    let memory_manager = MemoryManager::new(memory_config, engine).await?;

    // Monitor both systems working together
    sleep(Duration::from_secs(5)).await;

    let launcher_stats = launcher.stats().await;
    let memory_stats = memory_manager.stats();

    println!("Launcher stats: {:?}", launcher_stats);
    println!("Memory stats: {:?}", memory_stats);

    // Verify both systems are operational
    assert!(launcher_stats.pool_utilization >= 0.0);
    assert!(memory_stats.memory_pressure >= 0.0);

    // Clean up both systems
    launcher.shutdown().await?;
    memory_manager.shutdown().await?;

    Ok(())
}