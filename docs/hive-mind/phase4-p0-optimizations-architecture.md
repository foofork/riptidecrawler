# Phase 4: P0 Critical Performance Optimizations - Architecture Specification

**Document Version:** 1.0
**Date:** October 17, 2025
**Architect:** System Architecture Designer (Hive Mind Collective)
**Phase:** Phase 4 - Critical Performance Optimizations (P0)
**Target:** 50-70% Performance Improvement
**Status:** Architecture Design

---

## Executive Summary

This document specifies the architecture for three critical P0 optimizations that will deliver significant performance improvements to RipTide's direct execution mode:

1. **Browser Pool Pre-warming**: 60-80% headless initialization reduction (3s → 0.5s)
2. **WASM AOT Compilation Caching**: 50-70% WASM initialization reduction (100ms → 30ms)
3. **Adaptive Timeout System**: 30-50% timeout waste reduction

### Performance Impact Projections

| Metric | Current | After P0 | Improvement |
|--------|---------|----------|-------------|
| **WASM Extract** | ~350ms | ~230ms | **34% faster** |
| **Headless Extract** | ~8200ms | ~7015ms | **14% faster** |
| **Render (Simple)** | ~2500ms | ~2130ms | **15% faster** |
| **Memory Peak** | 1.69GB | ~1.03GB | **40% reduction** |
| **Throughput** | 10 req/s | 25 req/s | **2.5x increase** |

---

## Table of Contents

1. [System Context](#1-system-context)
2. [Optimization 1: Browser Pool Pre-warming](#2-optimization-1-browser-pool-pre-warming)
3. [Optimization 2: WASM AOT Compilation Caching](#3-optimization-2-wasm-aot-compilation-caching)
4. [Optimization 3: Adaptive Timeout System](#4-optimization-3-adaptive-timeout-system)
5. [Integration Architecture](#5-integration-architecture)
6. [Configuration Schema](#6-configuration-schema)
7. [Performance Impact Analysis](#7-performance-impact-analysis)
8. [Risk Assessment](#8-risk-assessment)
9. [Implementation Roadmap](#9-implementation-roadmap)

---

## 1. System Context

### 1.1 Current Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Command Layer                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                   Engine Selection Layer                    │
│  • Heuristic analysis    • Domain caching                   │
│  • Strategy selection    • Fallback chains                  │
└────────────────────────┬────────────────────────────────────┘
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
    ┌──────────┐  ┌────────────┐  ┌──────────┐
    │   WASM   │  │  Headless  │  │  Stealth │
    │  Engine  │  │   Engine   │  │  Engine  │
    └──────────┘  └────────────┘  └──────────┘
         │              │               │
         │         ❌ SLOW INIT      ❌ SLOW INIT
         │         (1-3 seconds)    (1-3 seconds)
         │
    ❌ COMPILATION
    (50-200ms per call)
```

**Current Bottlenecks:**
- **WASM**: Module compilation on every invocation (50-200ms)
- **Headless/Stealth**: Chrome process startup for each request (1-3s)
- **Navigation**: Fixed 30s timeout regardless of page characteristics

### 1.2 Target Architecture (Phase 4)

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Command Layer                       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Enhanced Engine Selection Layer                │
│  • Cached decisions      • Adaptive timeouts                │
│  • Performance feedback  • Timeout profiles                 │
└────────────────────────┬────────────────────────────────────┘
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
    ┌──────────────┐  ┌─────────────────┐  ┌──────────────┐
    │  WASM Engine │  │  Browser Pool   │  │Stealth Engine│
    │  + AOT Cache │  │  Manager        │  │  + Pool      │
    └──────────────┘  └─────────────────┘  └──────────────┘
         │                    │                    │
    ✅ PRE-COMPILED      ✅ WARM INSTANCES    ✅ WARM INSTANCES
    (~30ms)             (0.5s checkout)     (0.5s checkout)
         │                    │                    │
         └────────────────────┴────────────────────┘
                              │
                              ▼
                 ┌────────────────────────┐
                 │  Adaptive Timeout      │
                 │  Manager               │
                 │  • Domain profiles     │
                 │  • Learning engine     │
                 │  • Smart retry         │
                 └────────────────────────┘
```

---

## 2. Optimization 1: Browser Pool Pre-warming

### 2.1 Architecture Overview

The Browser Pool Pre-warming system maintains a pool of warm, ready-to-use browser instances to eliminate the 1-3 second Chrome startup overhead.

```
┌─────────────────────────────────────────────────────────────┐
│                   Browser Pool Manager                      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Pool State Management                      │   │
│  │  • Available instances queue                         │   │
│  │  • In-use instances tracking                         │   │
│  │  • Health check scheduler                            │   │
│  │  • Auto-restart coordinator                          │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Instance Management                        │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐           │   │
│  │  │Instance 1│  │Instance 2│  │Instance 3│           │   │
│  │  │ (Ready)  │  │(In-Use)  │  │ (Ready)  │           │   │
│  │  └──────────┘  └──────────┘  └──────────┘           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Health Monitoring                          │   │
│  │  • Every 30s health check                            │   │
│  │  • Memory leak detection                             │   │
│  │  • Response time monitoring                          │   │
│  │  • Auto-restart on failure                           │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Component Specification

#### 2.2.1 Browser Pool Manager

**Location:** `crates/riptide-headless/src/pool/manager.rs`

**Core Data Structure:**
```rust
pub struct BrowserPoolManager {
    /// Configuration for the pool
    config: PoolConfig,

    /// Available (warm) browser instances
    available: Arc<Mutex<VecDeque<BrowserInstance>>>,

    /// Currently in-use instances
    in_use: Arc<Mutex<HashMap<Uuid, BrowserInstance>>>,

    /// Health check task handle
    health_checker: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// Metrics collector
    metrics: Arc<PoolMetrics>,

    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

pub struct PoolConfig {
    /// Minimum number of warm instances
    pub min_instances: usize,

    /// Maximum number of instances (including in-use)
    pub max_instances: usize,

    /// Target number of available instances
    pub target_available: usize,

    /// Health check interval
    pub health_check_interval: Duration,

    /// Instance idle timeout before recycling
    pub idle_timeout: Duration,

    /// Maximum checkout duration
    pub max_checkout_duration: Duration,

    /// Enable auto-restart on failure
    pub auto_restart: bool,

    /// Memory limit per instance (MB)
    pub memory_limit_mb: usize,
}

pub struct BrowserInstance {
    /// Unique instance ID
    pub id: Uuid,

    /// Underlying browser connection
    pub browser: Arc<Browser>,

    /// Creation timestamp
    pub created_at: Instant,

    /// Last used timestamp
    pub last_used: Instant,

    /// Total requests served
    pub request_count: u64,

    /// Current state
    pub state: InstanceState,

    /// Health status
    pub health: HealthStatus,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstanceState {
    /// Initializing (Chrome starting up)
    Initializing,

    /// Ready and available for checkout
    Available,

    /// Currently processing a request
    InUse,

    /// Health check in progress
    HealthCheck,

    /// Recycling (cleaning up resources)
    Recycling,

    /// Failed and pending restart
    Failed,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_check: Instant,
    pub memory_usage_mb: usize,
    pub response_time_ms: u64,
    pub failure_count: u32,
}
```

**Core APIs:**

```rust
impl BrowserPoolManager {
    /// Create and initialize the pool
    pub async fn new(config: PoolConfig) -> Result<Self> {
        let manager = Self {
            config,
            available: Arc::new(Mutex::new(VecDeque::new())),
            in_use: Arc::new(Mutex::new(HashMap::new())),
            health_checker: Arc::new(Mutex::new(None)),
            metrics: Arc::new(PoolMetrics::default()),
            shutdown: Arc::new(AtomicBool::new(false)),
        };

        // Pre-warm initial instances
        manager.warmup().await?;

        // Start health check task
        manager.start_health_checker();

        Ok(manager)
    }

    /// Check out a browser instance from the pool
    pub async fn checkout(&self) -> Result<PooledBrowser> {
        let start = Instant::now();

        // Try to get available instance
        let instance = {
            let mut available = self.available.lock().await;
            available.pop_front()
        };

        let instance = match instance {
            Some(mut inst) => {
                inst.state = InstanceState::InUse;
                inst.last_used = Instant::now();
                inst
            }
            None => {
                // No available instances, create new if under limit
                if self.total_instances().await < self.config.max_instances {
                    self.create_instance().await?
                } else {
                    // Wait for instance to become available (with timeout)
                    self.wait_for_instance().await?
                }
            }
        };

        // Track in-use
        {
            let mut in_use = self.in_use.lock().await;
            in_use.insert(instance.id, instance.clone());
        }

        // Record metrics
        self.metrics.record_checkout(start.elapsed());

        Ok(PooledBrowser::new(instance, self.clone()))
    }

    /// Return a browser instance to the pool
    pub async fn checkin(&self, instance_id: Uuid) -> Result<()> {
        // Remove from in-use
        let mut instance = {
            let mut in_use = self.in_use.lock().await;
            in_use.remove(&instance_id)
                .ok_or(PoolError::InstanceNotFound)?
        };

        // Update state
        instance.state = InstanceState::Available;
        instance.request_count += 1;

        // Check if instance should be recycled
        if self.should_recycle(&instance) {
            self.recycle_instance(instance).await?;
        } else {
            // Return to available pool
            let mut available = self.available.lock().await;
            available.push_back(instance);
        }

        Ok(())
    }

    /// Pre-warm instances during startup
    async fn warmup(&self) -> Result<()> {
        let mut tasks = vec![];

        for _ in 0..self.config.target_available {
            let manager = self.clone();
            tasks.push(tokio::spawn(async move {
                manager.create_instance().await
            }));
        }

        // Wait for all instances to be ready
        let results = join_all(tasks).await;
        let instances: Result<Vec<_>> = results.into_iter()
            .map(|r| r?)
            .collect();

        let instances = instances?;

        // Add to available pool
        {
            let mut available = self.available.lock().await;
            available.extend(instances);
        }

        info!("Browser pool warmed up with {} instances",
              self.config.target_available);

        Ok(())
    }

    /// Create a new browser instance
    async fn create_instance(&self) -> Result<BrowserInstance> {
        let start = Instant::now();

        let browser = launch_chromium().await?;

        let instance = BrowserInstance {
            id: Uuid::new_v4(),
            browser: Arc::new(browser),
            created_at: Instant::now(),
            last_used: Instant::now(),
            request_count: 0,
            state: InstanceState::Available,
            health: HealthStatus {
                is_healthy: true,
                last_check: Instant::now(),
                memory_usage_mb: 0,
                response_time_ms: 0,
                failure_count: 0,
            },
        };

        self.metrics.record_instance_created(start.elapsed());

        Ok(instance)
    }

    /// Health check background task
    fn start_health_checker(&self) {
        let manager = self.clone();
        let handle = tokio::spawn(async move {
            manager.health_check_loop().await;
        });

        *self.health_checker.lock().await = Some(handle);
    }

    async fn health_check_loop(&self) {
        let mut interval = tokio::time::interval(
            self.config.health_check_interval
        );

        while !self.shutdown.load(Ordering::Relaxed) {
            interval.tick().await;

            if let Err(e) = self.perform_health_check().await {
                error!("Health check failed: {}", e);
            }
        }
    }

    async fn perform_health_check(&self) -> Result<()> {
        // Check available instances
        let instances: Vec<BrowserInstance> = {
            let available = self.available.lock().await;
            available.iter().cloned().collect()
        };

        for mut instance in instances {
            instance.state = InstanceState::HealthCheck;

            // Perform health check
            let health = self.check_instance_health(&instance).await?;
            instance.health = health;

            if !health.is_healthy {
                warn!("Instance {} unhealthy, recycling", instance.id);
                self.recycle_instance(instance).await?;
            } else {
                instance.state = InstanceState::Available;
            }
        }

        // Ensure minimum instances
        self.ensure_minimum_instances().await?;

        Ok(())
    }

    async fn check_instance_health(
        &self,
        instance: &BrowserInstance
    ) -> Result<HealthStatus> {
        let start = Instant::now();

        // Create test page
        let page = instance.browser.new_page("about:blank").await?;

        // Measure response time
        let response_time = start.elapsed();

        // Check memory usage
        let memory = page.evaluate("window.performance.memory")
            .await
            .unwrap_or_default();

        // Close test page
        page.close().await?;

        Ok(HealthStatus {
            is_healthy: response_time.as_millis() < 1000,
            last_check: Instant::now(),
            memory_usage_mb: memory.used_js_heap_size / 1024 / 1024,
            response_time_ms: response_time.as_millis() as u64,
            failure_count: 0,
        })
    }

    /// Determine if instance should be recycled
    fn should_recycle(&self, instance: &BrowserInstance) -> bool {
        // Recycle after max requests
        if instance.request_count > 100 {
            return true;
        }

        // Recycle if idle too long
        if instance.last_used.elapsed() > self.config.idle_timeout {
            return true;
        }

        // Recycle if memory exceeds limit
        if instance.health.memory_usage_mb > self.config.memory_limit_mb {
            return true;
        }

        false
    }

    /// Recycle an instance and create new one
    async fn recycle_instance(&self, instance: BrowserInstance) -> Result<()> {
        // Close browser
        instance.browser.close().await?;

        // Create replacement if needed
        if self.config.auto_restart {
            let new_instance = self.create_instance().await?;
            let mut available = self.available.lock().await;
            available.push_back(new_instance);
        }

        Ok(())
    }

    /// Ensure minimum number of instances
    async fn ensure_minimum_instances(&self) -> Result<()> {
        let current = self.available.lock().await.len();

        if current < self.config.min_instances {
            let needed = self.config.min_instances - current;

            for _ in 0..needed {
                let instance = self.create_instance().await?;
                self.available.lock().await.push_back(instance);
            }

            info!("Created {} instances to meet minimum", needed);
        }

        Ok(())
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down browser pool...");

        // Signal shutdown
        self.shutdown.store(true, Ordering::Relaxed);

        // Stop health checker
        if let Some(handle) = self.health_checker.lock().await.take() {
            handle.abort();
        }

        // Close all instances
        let instances: Vec<BrowserInstance> = {
            let mut available = self.available.lock().await;
            available.drain(..).collect()
        };

        for instance in instances {
            instance.browser.close().await?;
        }

        info!("Browser pool shutdown complete");
        Ok(())
    }
}
```

#### 2.2.2 Pooled Browser Handle

**Purpose:** RAII-style handle that automatically returns instance to pool

```rust
pub struct PooledBrowser {
    instance: Option<BrowserInstance>,
    pool: BrowserPoolManager,
}

impl PooledBrowser {
    fn new(instance: BrowserInstance, pool: BrowserPoolManager) -> Self {
        Self {
            instance: Some(instance),
            pool,
        }
    }

    /// Get access to the browser
    pub fn browser(&self) -> &Browser {
        &self.instance.as_ref().unwrap().browser
    }

    /// Create new page
    pub async fn new_page(&self, url: &str) -> Result<Page> {
        self.browser().new_page(url).await
    }
}

impl Drop for PooledBrowser {
    fn drop(&mut self) {
        if let Some(instance) = self.instance.take() {
            let pool = self.pool.clone();
            let instance_id = instance.id;

            // Return to pool asynchronously
            tokio::spawn(async move {
                if let Err(e) = pool.checkin(instance_id).await {
                    error!("Failed to return instance to pool: {}", e);
                }
            });
        }
    }
}
```

#### 2.2.3 Pool Metrics

```rust
#[derive(Default)]
pub struct PoolMetrics {
    pub total_checkouts: AtomicU64,
    pub total_checkins: AtomicU64,
    pub total_created: AtomicU64,
    pub total_recycled: AtomicU64,
    pub checkout_times: Arc<Mutex<Vec<Duration>>>,
    pub creation_times: Arc<Mutex<Vec<Duration>>>,
}

impl PoolMetrics {
    pub fn record_checkout(&self, duration: Duration) {
        self.total_checkouts.fetch_add(1, Ordering::Relaxed);
        self.checkout_times.lock().unwrap().push(duration);
    }

    pub fn record_instance_created(&self, duration: Duration) {
        self.total_created.fetch_add(1, Ordering::Relaxed);
        self.creation_times.lock().unwrap().push(duration);
    }

    pub fn avg_checkout_time(&self) -> Duration {
        let times = self.checkout_times.lock().unwrap();
        if times.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }
}
```

### 2.3 Integration Points

#### 2.3.1 Engine Integration

**File:** `crates/riptide-headless/src/engine.rs`

```rust
pub struct HeadlessEngine {
    pool: Arc<BrowserPoolManager>,
    // ... existing fields
}

impl HeadlessEngine {
    pub async fn new() -> Result<Self> {
        let config = PoolConfig {
            min_instances: 1,
            max_instances: 5,
            target_available: 2,
            health_check_interval: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_checkout_duration: Duration::from_secs(60),
            auto_restart: true,
            memory_limit_mb: 500,
        };

        let pool = BrowserPoolManager::new(config).await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn extract(&self, url: &str) -> Result<Content> {
        // Checkout browser from pool (fast!)
        let browser = self.pool.checkout().await?;

        // Use browser
        let page = browser.new_page(url).await?;
        let content = self.extract_from_page(&page).await?;

        // Browser automatically returned to pool on drop
        Ok(content)
    }
}
```

### 2.4 Configuration

**File:** `~/.riptide/pool-config.toml`

```toml
[browser_pool]
enabled = true
min_instances = 1
max_instances = 5
target_available = 2
health_check_interval_secs = 30
idle_timeout_secs = 300
max_checkout_duration_secs = 60
auto_restart = true
memory_limit_mb = 500

[browser_pool.advanced]
# Enable aggressive recycling
aggressive_recycling = false
max_requests_per_instance = 100
recycle_on_memory_threshold = true
```

---

## 3. Optimization 2: WASM AOT Compilation Caching

### 3.1 Architecture Overview

The WASM AOT (Ahead-of-Time) Compilation Caching system pre-compiles WASM modules and caches the compiled artifacts on disk to eliminate the 50-200ms compilation overhead.

```
┌─────────────────────────────────────────────────────────────┐
│                  WASM Cache Manager                         │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Cache Directory Structure                  │   │
│  │  ~/.riptide/wasm-cache/                              │   │
│  │  ├── modules/                                        │   │
│  │  │   ├── <hash>.cwasm   (compiled module)           │   │
│  │  │   └── <hash>.meta    (metadata)                  │   │
│  │  ├── cache.db           (SQLite index)              │   │
│  │  └── lock              (file lock)                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Compilation Pipeline                       │   │
│  │                                                      │   │
│  │  Source WASM  →  Hash Check  →  Cache Hit?          │   │
│  │      │              │              ├─ YES → Load    │   │
│  │      │              │              └─ NO  → Compile │   │
│  │      │              │                      ↓         │   │
│  │      │              │                   Cache Store │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Component Specification

#### 3.2.1 WASM Cache Manager

**Location:** `crates/riptide-cli/src/wasm/cache_manager.rs`

**Core Data Structure:**

```rust
pub struct WasmCacheManager {
    /// Cache directory path
    cache_dir: PathBuf,

    /// SQLite connection for index
    db: Arc<Mutex<Connection>>,

    /// Wasmtime engine with caching enabled
    engine: Engine,

    /// Cache configuration
    config: CacheConfig,

    /// Metrics
    metrics: Arc<CacheMetrics>,
}

pub struct CacheConfig {
    /// Cache directory path
    pub cache_dir: PathBuf,

    /// Enable AOT compilation
    pub aot_enabled: bool,

    /// Maximum cache size (MB)
    pub max_cache_size_mb: usize,

    /// Cache entry TTL
    pub entry_ttl: Duration,

    /// Enable parallel compilation
    pub parallel_compilation: bool,

    /// Compression for cached modules
    pub compress_cached_modules: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Module hash (SHA-256 of source WASM)
    pub module_hash: String,

    /// Path to compiled module
    pub compiled_path: PathBuf,

    /// Original module path
    pub source_path: PathBuf,

    /// Compilation timestamp
    pub compiled_at: SystemTime,

    /// Last access timestamp
    pub last_accessed: SystemTime,

    /// Access count
    pub access_count: u64,

    /// Compiled module size (bytes)
    pub size_bytes: u64,

    /// Source module hash
    pub source_hash: String,
}

#[derive(Default)]
pub struct CacheMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub compilations: AtomicU64,
    pub load_times: Arc<Mutex<Vec<Duration>>>,
    pub compilation_times: Arc<Mutex<Vec<Duration>>>,
}
```

**Core APIs:**

```rust
impl WasmCacheManager {
    /// Initialize cache manager
    pub fn new(config: CacheConfig) -> Result<Self> {
        // Create cache directory
        fs::create_dir_all(&config.cache_dir)?;
        fs::create_dir_all(config.cache_dir.join("modules"))?;

        // Initialize SQLite index
        let db = Connection::open(config.cache_dir.join("cache.db"))?;
        Self::init_db(&db)?;

        // Configure Wasmtime engine with caching
        let engine = Self::create_engine(&config)?;

        Ok(Self {
            cache_dir: config.cache_dir.clone(),
            db: Arc::new(Mutex::new(db)),
            engine,
            config,
            metrics: Arc::new(CacheMetrics::default()),
        })
    }

    /// Create Wasmtime engine with caching enabled
    fn create_engine(config: &CacheConfig) -> Result<Engine> {
        let mut engine_config = wasmtime::Config::new();

        // Enable AOT compilation
        engine_config.strategy(wasmtime::Strategy::Cranelift)?;

        // Enable module caching to disk
        if config.aot_enabled {
            engine_config.cache_config_load_default()?;
        }

        // Optimization settings
        engine_config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        // Enable parallel compilation
        if config.parallel_compilation {
            engine_config.parallel_compilation(true);
        }

        Ok(Engine::new(&engine_config)?)
    }

    /// Get or compile WASM module
    pub async fn get_or_compile(
        &self,
        wasm_path: &Path
    ) -> Result<Module> {
        let start = Instant::now();

        // Calculate source hash
        let source_hash = self.calculate_file_hash(wasm_path).await?;

        // Check cache
        if let Some(entry) = self.get_cache_entry(&source_hash).await? {
            // Cache hit!
            self.metrics.hits.fetch_add(1, Ordering::Relaxed);

            let module = self.load_cached_module(&entry).await?;

            // Update access stats
            self.update_access_stats(&source_hash).await?;

            let load_time = start.elapsed();
            self.metrics.load_times.lock().unwrap().push(load_time);

            info!("WASM cache HIT: {} ({:?})",
                  wasm_path.display(), load_time);

            return Ok(module);
        }

        // Cache miss - compile
        self.metrics.misses.fetch_add(1, Ordering::Relaxed);

        let module = self.compile_and_cache(wasm_path, &source_hash).await?;

        let compile_time = start.elapsed();
        self.metrics.compilation_times.lock().unwrap().push(compile_time);

        info!("WASM cache MISS: {} - compiled in {:?}",
              wasm_path.display(), compile_time);

        Ok(module)
    }

    /// Compile WASM module and cache result
    async fn compile_and_cache(
        &self,
        wasm_path: &Path,
        source_hash: &str
    ) -> Result<Module> {
        self.metrics.compilations.fetch_add(1, Ordering::Relaxed);

        // Read source WASM
        let wasm_bytes = tokio::fs::read(wasm_path).await?;

        // Compile module
        let module = Module::from_binary(&self.engine, &wasm_bytes)?;

        // Serialize compiled module
        let compiled_bytes = module.serialize()?;

        // Optionally compress
        let final_bytes = if self.config.compress_cached_modules {
            self.compress(&compiled_bytes)?
        } else {
            compiled_bytes
        };

        // Generate cache filename
        let cache_filename = format!("{}.cwasm", source_hash);
        let cache_path = self.cache_dir.join("modules").join(&cache_filename);

        // Write to cache
        tokio::fs::write(&cache_path, &final_bytes).await?;

        // Create cache entry
        let entry = CacheEntry {
            module_hash: source_hash.to_string(),
            compiled_path: cache_path.clone(),
            source_path: wasm_path.to_path_buf(),
            compiled_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
            size_bytes: final_bytes.len() as u64,
            source_hash: source_hash.to_string(),
        };

        // Store in database
        self.store_cache_entry(&entry).await?;

        Ok(module)
    }

    /// Load cached compiled module
    async fn load_cached_module(&self, entry: &CacheEntry) -> Result<Module> {
        // Read compiled module
        let mut bytes = tokio::fs::read(&entry.compiled_path).await?;

        // Decompress if needed
        if self.config.compress_cached_modules {
            bytes = self.decompress(&bytes)?;
        }

        // Deserialize module
        let module = unsafe {
            Module::deserialize(&self.engine, &bytes)?
        };

        Ok(module)
    }

    /// Calculate SHA-256 hash of file
    async fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let bytes = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Get cache entry from database
    async fn get_cache_entry(&self, hash: &str) -> Result<Option<CacheEntry>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            "SELECT * FROM cache_entries WHERE module_hash = ?1"
        )?;

        let entry = stmt.query_row([hash], |row| {
            Ok(CacheEntry {
                module_hash: row.get(0)?,
                compiled_path: PathBuf::from(row.get::<_, String>(1)?),
                source_path: PathBuf::from(row.get::<_, String>(2)?),
                compiled_at: row.get(3)?,
                last_accessed: row.get(4)?,
                access_count: row.get(5)?,
                size_bytes: row.get(6)?,
                source_hash: row.get(7)?,
            })
        });

        match entry {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store cache entry in database
    async fn store_cache_entry(&self, entry: &CacheEntry) -> Result<()> {
        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO cache_entries
             (module_hash, compiled_path, source_path, compiled_at,
              last_accessed, access_count, size_bytes, source_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &entry.module_hash,
                entry.compiled_path.to_str(),
                entry.source_path.to_str(),
                entry.compiled_at,
                entry.last_accessed,
                entry.access_count,
                entry.size_bytes,
                &entry.source_hash,
            ],
        )?;

        Ok(())
    }

    /// Update access statistics
    async fn update_access_stats(&self, hash: &str) -> Result<()> {
        let db = self.db.lock().unwrap();

        db.execute(
            "UPDATE cache_entries
             SET last_accessed = ?1, access_count = access_count + 1
             WHERE module_hash = ?2",
            params![SystemTime::now(), hash],
        )?;

        Ok(())
    }

    /// Initialize database schema
    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache_entries (
                module_hash TEXT PRIMARY KEY,
                compiled_path TEXT NOT NULL,
                source_path TEXT NOT NULL,
                compiled_at INTEGER NOT NULL,
                last_accessed INTEGER NOT NULL,
                access_count INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                source_hash TEXT NOT NULL
            )",
            [],
        )?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_accessed
             ON cache_entries(last_accessed)",
            [],
        )?;

        Ok(())
    }

    /// Evict old cache entries (LRU)
    pub async fn evict_old_entries(&self) -> Result<()> {
        let db = self.db.lock().unwrap();

        // Calculate cache size
        let total_size: i64 = db.query_row(
            "SELECT SUM(size_bytes) FROM cache_entries",
            [],
            |row| row.get(0),
        )?;

        let max_size = (self.config.max_cache_size_mb * 1024 * 1024) as i64;

        if total_size > max_size {
            // Evict least recently used
            let mut stmt = db.prepare(
                "SELECT module_hash, compiled_path
                 FROM cache_entries
                 ORDER BY last_accessed ASC
                 LIMIT 10"
            )?;

            let entries: Vec<(String, PathBuf)> = stmt.query_map([], |row| {
                Ok((row.get(0)?, PathBuf::from(row.get::<_, String>(1)?)))
            })?.collect::<Result<Vec<_>, _>>()?;

            for (hash, path) in entries {
                // Delete file
                tokio::fs::remove_file(&path).await?;

                // Delete from database
                db.execute(
                    "DELETE FROM cache_entries WHERE module_hash = ?1",
                    [&hash],
                )?;
            }

            info!("Evicted {} old cache entries", entries.len());
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let hits = self.metrics.hits.load(Ordering::Relaxed);
        let misses = self.metrics.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let avg_load_time = {
            let times = self.metrics.load_times.lock().unwrap();
            if times.is_empty() {
                Duration::ZERO
            } else {
                times.iter().sum::<Duration>() / times.len() as u32
            }
        };

        let avg_compile_time = {
            let times = self.metrics.compilation_times.lock().unwrap();
            if times.is_empty() {
                Duration::ZERO
            } else {
                times.iter().sum::<Duration>() / times.len() as u32
            }
        };

        CacheStats {
            hits,
            misses,
            hit_rate,
            avg_load_time,
            avg_compile_time,
            total_entries: self.count_entries(),
        }
    }
}
```

### 3.3 Integration Points

#### 3.3.1 WASM Engine Integration

**File:** `crates/riptide-cli/src/wasm/engine.rs`

```rust
pub struct WasmEngine {
    cache_manager: Arc<WasmCacheManager>,
    // ... existing fields
}

impl WasmEngine {
    pub fn new() -> Result<Self> {
        let config = CacheConfig {
            cache_dir: dirs::home_dir()
                .unwrap()
                .join(".riptide")
                .join("wasm-cache"),
            aot_enabled: true,
            max_cache_size_mb: 500,
            entry_ttl: Duration::from_secs(30 * 24 * 3600), // 30 days
            parallel_compilation: true,
            compress_cached_modules: true,
        };

        let cache_manager = WasmCacheManager::new(config)?;

        Ok(Self {
            cache_manager: Arc::new(cache_manager),
        })
    }

    pub async fn extract(&self, url: &str, html: &str) -> Result<Content> {
        // Get or compile WASM module (fast with caching!)
        let module = self.cache_manager
            .get_or_compile(Path::new("wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm"))
            .await?;

        // Use module for extraction
        let result = self.execute_extraction(module, url, html).await?;

        Ok(result)
    }
}
```

### 3.4 Configuration

**File:** `~/.riptide/wasm-cache-config.toml`

```toml
[wasm_cache]
enabled = true
cache_dir = "~/.riptide/wasm-cache"
max_cache_size_mb = 500
entry_ttl_days = 30
parallel_compilation = true
compress_cached_modules = true

[wasm_cache.compilation]
aot_enabled = true
optimization_level = "speed"
parallel = true

[wasm_cache.maintenance]
# Run eviction check every hour
eviction_check_interval_secs = 3600
# Keep at least this many entries
min_entries = 5
```

---

## 4. Optimization 3: Adaptive Timeout System

### 4.1 Architecture Overview

The Adaptive Timeout System learns optimal navigation timeouts based on domain characteristics and historical data, reducing wasted time waiting for unresponsive pages.

```
┌─────────────────────────────────────────────────────────────┐
│              Adaptive Timeout Manager                       │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Timeout Learning Engine                      │   │
│  │  • Response time tracking                            │   │
│  │  • Success rate analysis                             │   │
│  │  • Domain profile building                           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Domain Timeout Profiles                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │   │
│  │  │  example.com │  │  github.com  │  │ docs.rs  │  │   │
│  │  │  ⏱️ 5-10s    │  │  ⏱️ 10-15s   │  │ ⏱️ 3-8s  │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Smart Retry Logic                            │   │
│  │  • Exponential backoff                               │   │
│  │  • Timeout escalation                                │   │
│  │  • Failure pattern detection                         │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Component Specification

#### 4.2.1 Adaptive Timeout Manager

**Location:** `crates/riptide-cli/src/adaptive/timeout_manager.rs`

**Core Data Structure:**

```rust
pub struct AdaptiveTimeoutManager {
    /// Domain timeout profiles
    profiles: Arc<RwLock<HashMap<String, DomainProfile>>>,

    /// Configuration
    config: TimeoutConfig,

    /// Metrics
    metrics: Arc<TimeoutMetrics>,

    /// Persistence (SQLite)
    db: Arc<Mutex<Connection>>,
}

pub struct TimeoutConfig {
    /// Minimum timeout (safety floor)
    pub min_timeout_secs: u64,

    /// Maximum timeout (safety ceiling)
    pub max_timeout_secs: u64,

    /// Default timeout for unknown domains
    pub default_timeout_secs: u64,

    /// Learning rate (0.0-1.0)
    pub learning_rate: f64,

    /// Minimum samples before adapting
    pub min_samples: usize,

    /// Profile persistence enabled
    pub persist_profiles: bool,

    /// Profile TTL
    pub profile_ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    /// Domain name
    pub domain: String,

    /// Current recommended timeout
    pub timeout_secs: u64,

    /// Response time statistics
    pub stats: ResponseStats,

    /// Last updated timestamp
    pub last_updated: SystemTime,

    /// Sample count
    pub sample_count: usize,

    /// Success rate
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStats {
    /// Average response time (ms)
    pub avg_ms: u64,

    /// P50 response time (ms)
    pub p50_ms: u64,

    /// P95 response time (ms)
    pub p95_ms: u64,

    /// P99 response time (ms)
    pub p99_ms: u64,

    /// Standard deviation
    pub stddev_ms: u64,

    /// Recent response times (sliding window)
    pub recent_times: VecDeque<u64>,
}

#[derive(Default)]
pub struct TimeoutMetrics {
    pub total_requests: AtomicU64,
    pub timeouts: AtomicU64,
    pub successful: AtomicU64,
    pub time_saved_ms: AtomicU64,
}
```

**Core APIs:**

```rust
impl AdaptiveTimeoutManager {
    /// Initialize manager
    pub fn new(config: TimeoutConfig) -> Result<Self> {
        let db_path = dirs::home_dir()
            .unwrap()
            .join(".riptide")
            .join("timeout-profiles.db");

        let db = Connection::open(db_path)?;
        Self::init_db(&db)?;

        let manager = Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(TimeoutMetrics::default()),
            db: Arc::new(Mutex::new(db)),
        };

        // Load persisted profiles
        if config.persist_profiles {
            manager.load_profiles()?;
        }

        Ok(manager)
    }

    /// Get timeout for domain
    pub async fn get_timeout(&self, url: &Url) -> Duration {
        let domain = self.extract_domain(url);

        let profiles = self.profiles.read().await;

        if let Some(profile) = profiles.get(&domain) {
            // Use learned timeout
            Duration::from_secs(profile.timeout_secs)
        } else {
            // Use default for unknown domain
            Duration::from_secs(self.config.default_timeout_secs)
        }
    }

    /// Record navigation result
    pub async fn record_result(
        &self,
        url: &Url,
        duration: Duration,
        success: bool
    ) -> Result<()> {
        let domain = self.extract_domain(url);

        // Update metrics
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        if success {
            self.metrics.successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics.timeouts.fetch_add(1, Ordering::Relaxed);
        }

        // Update profile
        let mut profiles = self.profiles.write().await;

        let profile = profiles.entry(domain.clone())
            .or_insert_with(|| DomainProfile::new(&domain));

        // Update statistics
        profile.update(duration.as_millis() as u64, success);

        // Adapt timeout based on statistics
        let new_timeout = self.calculate_adaptive_timeout(profile);
        profile.timeout_secs = new_timeout;

        // Persist to database
        if self.config.persist_profiles {
            self.save_profile(profile).await?;
        }

        Ok(())
    }

    /// Calculate adaptive timeout based on profile
    fn calculate_adaptive_timeout(&self, profile: &DomainProfile) -> u64 {
        // Not enough samples - use default
        if profile.sample_count < self.config.min_samples {
            return self.config.default_timeout_secs;
        }

        // Calculate timeout based on P95 + buffer
        let p95_secs = (profile.stats.p95_ms as f64 / 1000.0).ceil() as u64;

        // Add 50% buffer for variability
        let timeout = (p95_secs as f64 * 1.5).ceil() as u64;

        // Clamp to min/max bounds
        timeout.clamp(
            self.config.min_timeout_secs,
            self.config.max_timeout_secs
        )
    }

    /// Smart retry with escalating timeouts
    pub async fn retry_with_backoff<F, Fut, T>(
        &self,
        url: &Url,
        operation: F,
        max_retries: usize
    ) -> Result<T>
    where
        F: Fn(Duration) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut timeout = self.get_timeout(url).await;
        let mut attempt = 0;

        loop {
            let start = Instant::now();

            match tokio::time::timeout(timeout, operation(timeout)).await {
                Ok(Ok(result)) => {
                    // Success!
                    let duration = start.elapsed();
                    self.record_result(url, duration, true).await?;

                    // Calculate time saved vs max timeout
                    let max_timeout = Duration::from_secs(
                        self.config.max_timeout_secs
                    );
                    if duration < max_timeout {
                        let saved = max_timeout - duration;
                        self.metrics.time_saved_ms.fetch_add(
                            saved.as_millis() as u64,
                            Ordering::Relaxed
                        );
                    }

                    return Ok(result);
                }
                Ok(Err(e)) => {
                    // Operation failed (not timeout)
                    return Err(e);
                }
                Err(_) => {
                    // Timeout!
                    let duration = start.elapsed();
                    self.record_result(url, duration, false).await?;

                    attempt += 1;

                    if attempt >= max_retries {
                        return Err(anyhow::anyhow!(
                            "Max retries exceeded for {}", url
                        ));
                    }

                    // Exponential backoff: increase timeout by 50%
                    timeout = Duration::from_secs(
                        ((timeout.as_secs() as f64) * 1.5).ceil() as u64
                    ).min(Duration::from_secs(self.config.max_timeout_secs));

                    info!(
                        "Retrying {} with increased timeout: {:?} (attempt {}/{})",
                        url, timeout, attempt + 1, max_retries
                    );
                }
            }
        }
    }

    /// Extract domain from URL
    fn extract_domain(&self, url: &Url) -> String {
        url.host_str()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Load profiles from database
    fn load_profiles(&self) -> Result<()> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            "SELECT domain, timeout_secs, stats_json, last_updated,
                    sample_count, success_rate
             FROM timeout_profiles"
        )?;

        let profiles = stmt.query_map([], |row| {
            let stats_json: String = row.get(2)?;
            let stats: ResponseStats = serde_json::from_str(&stats_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            Ok(DomainProfile {
                domain: row.get(0)?,
                timeout_secs: row.get(1)?,
                stats,
                last_updated: row.get(3)?,
                sample_count: row.get(4)?,
                success_rate: row.get(5)?,
            })
        })?;

        let mut profile_map = self.profiles.blocking_write();

        for profile_result in profiles {
            let profile = profile_result?;
            profile_map.insert(profile.domain.clone(), profile);
        }

        info!("Loaded {} timeout profiles from database", profile_map.len());

        Ok(())
    }

    /// Save profile to database
    async fn save_profile(&self, profile: &DomainProfile) -> Result<()> {
        let db = self.db.lock().unwrap();

        let stats_json = serde_json::to_string(&profile.stats)?;

        db.execute(
            "INSERT OR REPLACE INTO timeout_profiles
             (domain, timeout_secs, stats_json, last_updated,
              sample_count, success_rate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &profile.domain,
                profile.timeout_secs,
                &stats_json,
                profile.last_updated,
                profile.sample_count,
                profile.success_rate,
            ],
        )?;

        Ok(())
    }

    /// Initialize database schema
    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS timeout_profiles (
                domain TEXT PRIMARY KEY,
                timeout_secs INTEGER NOT NULL,
                stats_json TEXT NOT NULL,
                last_updated INTEGER NOT NULL,
                sample_count INTEGER NOT NULL,
                success_rate REAL NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> TimeoutStats {
        let total = self.metrics.total_requests.load(Ordering::Relaxed);
        let timeouts = self.metrics.timeouts.load(Ordering::Relaxed);
        let successful = self.metrics.successful.load(Ordering::Relaxed);
        let time_saved = self.metrics.time_saved_ms.load(Ordering::Relaxed);

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let timeout_rate = if total > 0 {
            (timeouts as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        TimeoutStats {
            total_requests: total,
            timeouts,
            successful,
            success_rate,
            timeout_rate,
            time_saved_secs: time_saved / 1000,
        }
    }
}

impl DomainProfile {
    fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            timeout_secs: 30, // Default
            stats: ResponseStats::default(),
            last_updated: SystemTime::now(),
            sample_count: 0,
            success_rate: 1.0,
        }
    }

    fn update(&mut self, response_time_ms: u64, success: bool) {
        self.sample_count += 1;
        self.last_updated = SystemTime::now();

        // Update success rate with exponential moving average
        let alpha = 0.1; // Learning rate
        self.success_rate = alpha * (if success { 1.0 } else { 0.0 })
            + (1.0 - alpha) * self.success_rate;

        // Update response time statistics
        self.stats.update(response_time_ms);
    }
}

impl ResponseStats {
    fn update(&mut self, response_time_ms: u64) {
        // Add to recent times (sliding window of 100 samples)
        self.recent_times.push_back(response_time_ms);
        if self.recent_times.len() > 100 {
            self.recent_times.pop_front();
        }

        // Recalculate statistics
        let mut sorted: Vec<u64> = self.recent_times.iter().copied().collect();
        sorted.sort_unstable();

        let len = sorted.len();
        if len == 0 {
            return;
        }

        self.avg_ms = sorted.iter().sum::<u64>() / len as u64;
        self.p50_ms = sorted[len / 2];
        self.p95_ms = sorted[(len * 95) / 100];
        self.p99_ms = sorted[(len * 99) / 100];

        // Calculate standard deviation
        let variance: f64 = sorted.iter()
            .map(|&x| {
                let diff = x as f64 - self.avg_ms as f64;
                diff * diff
            })
            .sum::<f64>() / len as f64;

        self.stddev_ms = variance.sqrt() as u64;
    }
}
```

### 4.3 Integration Points

#### 4.3.1 Engine Integration

**File:** `crates/riptide-headless/src/navigation.rs`

```rust
pub struct Navigator {
    timeout_manager: Arc<AdaptiveTimeoutManager>,
}

impl Navigator {
    pub async fn navigate_with_adaptive_timeout(
        &self,
        page: &Page,
        url: &Url
    ) -> Result<()> {
        // Use adaptive timeout with smart retry
        self.timeout_manager.retry_with_backoff(
            url,
            |timeout| async move {
                page.goto(url.as_str())
                    .timeout(timeout)
                    .await
            },
            3 // Max 3 retries
        ).await?;

        Ok(())
    }
}
```

### 4.4 Configuration

**File:** `~/.riptide/adaptive-timeout-config.toml`

```toml
[adaptive_timeout]
enabled = true
min_timeout_secs = 5
max_timeout_secs = 60
default_timeout_secs = 30
learning_rate = 0.1
min_samples = 5
persist_profiles = true
profile_ttl_days = 30

[adaptive_timeout.retry]
max_retries = 3
backoff_multiplier = 1.5
```

---

## 5. Integration Architecture

### 5.1 Unified Integration Layer

**File:** `crates/riptide-cli/src/optimizations/mod.rs`

```rust
pub struct OptimizedExecutor {
    /// Browser pool for headless/stealth engines
    browser_pool: Arc<BrowserPoolManager>,

    /// WASM cache manager
    wasm_cache: Arc<WasmCacheManager>,

    /// Adaptive timeout manager
    timeout_manager: Arc<AdaptiveTimeoutManager>,

    /// Existing engine cache
    engine_cache: Arc<EngineCache>,

    /// Performance monitor
    perf_monitor: Arc<PerformanceMonitor>,
}

impl OptimizedExecutor {
    pub async fn new() -> Result<Self> {
        // Initialize browser pool
        let pool_config = PoolConfig::from_env()?;
        let browser_pool = BrowserPoolManager::new(pool_config).await?;

        // Initialize WASM cache
        let wasm_config = CacheConfig::from_env()?;
        let wasm_cache = WasmCacheManager::new(wasm_config)?;

        // Initialize adaptive timeouts
        let timeout_config = TimeoutConfig::from_env()?;
        let timeout_manager = AdaptiveTimeoutManager::new(timeout_config)?;

        // Initialize engine cache (Phase 3)
        let engine_cache = EngineCache::new();

        // Initialize performance monitor
        let perf_monitor = PerformanceMonitor::new();

        Ok(Self {
            browser_pool: Arc::new(browser_pool),
            wasm_cache: Arc::new(wasm_cache),
            timeout_manager: Arc::new(timeout_manager),
            engine_cache: Arc::new(engine_cache),
            perf_monitor: Arc::new(perf_monitor),
        })
    }

    pub async fn extract(&self, url: &str) -> Result<Content> {
        let url_parsed = Url::parse(url)?;

        // Start performance tracking
        let perf_tracker = self.perf_monitor.start_tracking();

        // Select engine (using Phase 3 caching)
        let engine = self.engine_cache.get_or_select(&url_parsed).await?;

        perf_tracker.stage("engine_selection");

        // Execute extraction based on engine
        let content = match engine {
            Engine::Wasm => {
                self.extract_with_wasm(&url_parsed).await?
            }
            Engine::Headless => {
                self.extract_with_headless(&url_parsed).await?
            }
            Engine::Stealth => {
                self.extract_with_stealth(&url_parsed).await?
            }
            Engine::Spider => {
                self.extract_with_spider(&url_parsed).await?
            }
        };

        perf_tracker.stage("extraction");

        // Record performance metrics
        perf_tracker.finish();

        Ok(content)
    }

    async fn extract_with_wasm(&self, url: &Url) -> Result<Content> {
        // Get or compile WASM module (Phase 4 Opt 2)
        let module = self.wasm_cache.get_or_compile(
            Path::new("wasm/riptide-extractor-wasm/target/wasm32-wasip2/release/riptide_extractor_wasm.wasm")
        ).await?;

        // Execute extraction
        // ... existing WASM extraction logic

        Ok(content)
    }

    async fn extract_with_headless(&self, url: &Url) -> Result<Content> {
        // Checkout browser from pool (Phase 4 Opt 1)
        let browser = self.browser_pool.checkout().await?;

        // Create page
        let page = browser.new_page("about:blank").await?;

        // Navigate with adaptive timeout (Phase 4 Opt 3)
        self.timeout_manager.retry_with_backoff(
            url,
            |timeout| async {
                page.goto(url.as_str())
                    .timeout(timeout)
                    .await
            },
            3
        ).await?;

        // Extract content
        // ... existing headless extraction logic

        // Browser automatically returned to pool on drop

        Ok(content)
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Graceful shutdown of all optimization systems
        self.browser_pool.shutdown().await?;
        // WASM cache doesn't need shutdown
        // Timeout manager auto-persists

        Ok(())
    }
}
```

### 5.2 CLI Integration

**File:** `crates/riptide-cli/src/main.rs`

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize optimized executor
    let executor = OptimizedExecutor::new().await?;

    // Register shutdown handler
    let executor_clone = executor.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        executor_clone.shutdown().await.unwrap();
    });

    // Parse CLI args
    let cli = Cli::parse();

    // Execute command
    match cli.command {
        Command::Extract { url, .. } => {
            let content = executor.extract(&url).await?;
            println!("{}", content);
        }
        // ... other commands
    }

    Ok(())
}
```

---

## 6. Configuration Schema

### 6.1 Unified Configuration File

**File:** `~/.riptide/config.toml`

```toml
[optimizations]
# Enable Phase 4 optimizations
enabled = true

[optimizations.browser_pool]
enabled = true
min_instances = 1
max_instances = 5
target_available = 2
health_check_interval_secs = 30
idle_timeout_secs = 300
max_checkout_duration_secs = 60
auto_restart = true
memory_limit_mb = 500

[optimizations.wasm_cache]
enabled = true
cache_dir = "~/.riptide/wasm-cache"
max_cache_size_mb = 500
entry_ttl_days = 30
parallel_compilation = true
compress_cached_modules = true
aot_enabled = true

[optimizations.adaptive_timeout]
enabled = true
min_timeout_secs = 5
max_timeout_secs = 60
default_timeout_secs = 30
learning_rate = 0.1
min_samples = 5
persist_profiles = true
profile_ttl_days = 30
max_retries = 3
backoff_multiplier = 1.5

[performance]
# Performance monitoring
track_metrics = true
export_metrics = true
metrics_export_interval_secs = 300
```

### 6.2 Environment Variables

```bash
# Browser Pool
RIPTIDE_POOL_ENABLED=true
RIPTIDE_POOL_MIN_INSTANCES=1
RIPTIDE_POOL_MAX_INSTANCES=5

# WASM Cache
RIPTIDE_WASM_CACHE_ENABLED=true
RIPTIDE_WASM_CACHE_DIR=~/.riptide/wasm-cache
RIPTIDE_WASM_CACHE_SIZE_MB=500

# Adaptive Timeout
RIPTIDE_ADAPTIVE_TIMEOUT_ENABLED=true
RIPTIDE_TIMEOUT_MIN=5
RIPTIDE_TIMEOUT_MAX=60
RIPTIDE_TIMEOUT_DEFAULT=30
```

---

## 7. Performance Impact Analysis

### 7.1 Projected Performance Improvements

#### Before Phase 4 Optimizations

| Metric | WASM | Headless | Stealth | Notes |
|--------|------|----------|---------|-------|
| **Cold Start** | 350ms | 8200ms | 8500ms | First request |
| **Warm (Cached)** | 50ms | 8200ms | 8500ms | Subsequent, same domain |
| **Throughput** | ~100 req/s | ~10 req/s | ~8 req/s | Parallel capacity |
| **Memory Peak** | 150MB | 1500MB | 1500MB | Per instance |

#### After Phase 4 Optimizations

| Metric | WASM | Headless | Stealth | Improvement |
|--------|------|----------|---------|-------------|
| **Cold Start** | 230ms | 7015ms | 7250ms | **34%, 14%, 14%** |
| **Warm (Cached)** | 30ms | 500ms | 500ms | **40%, 94%, 94%** |
| **Throughput** | ~150 req/s | ~25 req/s | ~20 req/s | **1.5x, 2.5x, 2.5x** |
| **Memory Peak** | 150MB | 1030MB | 1030MB | **0%, 31%, 31%** |

### 7.2 Optimization Breakdown

#### Optimization 1: Browser Pool (Headless/Stealth)
- **Cold Start Impact**: Minimal (browser still needs initialization)
- **Warm Start Impact**: **94% reduction** (8200ms → 500ms checkout time)
- **Memory Impact**: +200MB for pool (2 warm instances)
- **Best for**: Repeated extractions, batch processing

#### Optimization 2: WASM Cache
- **Cold Start Impact**: **34% reduction** (350ms → 230ms)
- **Warm Start Impact**: **40% reduction** (50ms → 30ms)
- **Memory Impact**: +20MB for cache
- **Best for**: All WASM extractions

#### Optimization 3: Adaptive Timeout
- **Time Saved**: 30-50% reduction in timeout waste
- **Average Savings**: ~5-10 seconds per failed navigation
- **Success Rate**: Increased by intelligent retry
- **Best for**: Slow/unreliable sites

### 7.3 Combined Impact

**Scenario 1: 100 Extractions (Same Domain)**
- **Before**: ~820 seconds (8.2s avg × 100)
- **After**: ~50 seconds (0.5s avg × 100)
- **Improvement**: **94% faster**

**Scenario 2: 100 Extractions (Mixed Domains)**
- **Before**: ~650 seconds (average across engines)
- **After**: ~300 seconds (optimized)
- **Improvement**: **54% faster**

**Scenario 3: WASM-Only Workload**
- **Before**: 350ms per extraction
- **After**: 30-230ms per extraction
- **Improvement**: **34-91% faster**

---

## 8. Risk Assessment

### 8.1 Browser Pool Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Memory Overhead** | Medium | Configurable pool size, aggressive recycling |
| **Resource Leaks** | Medium | Health checks every 30s, auto-restart |
| **Stale Instances** | Low | Idle timeout, request count limits |
| **Startup Complexity** | Low | Graceful degradation if pool fails |

### 8.2 WASM Cache Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Disk Space** | Low | Configurable max size, LRU eviction |
| **Cache Invalidation** | Medium | SHA-256 hash checking, TTL expiration |
| **Corrupt Cache** | Low | Fallback to compilation on load failure |
| **Security** | Low | Hash verification, read-only module usage |

### 8.3 Adaptive Timeout Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Too Aggressive** | Medium | Safety floor (5s min), learning rate tuning |
| **Profile Poisoning** | Low | Exponential moving average, sample minimums |
| **Storage Growth** | Low | Profile TTL, periodic cleanup |
| **Cold Start** | Low | Sensible defaults for unknown domains |

### 8.4 Integration Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| **Backward Compatibility** | Low | Feature flags, graceful degradation |
| **Configuration Complexity** | Medium | Sensible defaults, auto-detection |
| **Testing Coverage** | Medium | Comprehensive test suite (Phase 4 testing) |
| **Production Deployment** | Low | Phased rollout, monitoring |

---

## 9. Implementation Roadmap

### 9.1 Week 1: Core Implementation

#### Day 1-2: Browser Pool
- [ ] Implement `BrowserPoolManager`
- [ ] Implement `PooledBrowser` RAII handle
- [ ] Health check infrastructure
- [ ] Unit tests

#### Day 3-4: WASM Cache
- [ ] Implement `WasmCacheManager`
- [ ] SQLite persistence layer
- [ ] AOT compilation integration
- [ ] Unit tests

#### Day 5: Adaptive Timeout
- [ ] Implement `AdaptiveTimeoutManager`
- [ ] Domain profile learning
- [ ] Smart retry logic
- [ ] Unit tests

### 9.2 Week 2: Integration & Testing

#### Day 6-7: Integration
- [ ] Integrate browser pool with headless/stealth engines
- [ ] Integrate WASM cache with WASM engine
- [ ] Integrate adaptive timeouts with navigation
- [ ] Update `OptimizedExecutor`

#### Day 8-9: Testing
- [ ] Integration tests for each optimization
- [ ] End-to-end tests
- [ ] Performance benchmarks
- [ ] Stress tests

#### Day 10: Documentation & Polish
- [ ] Update user documentation
- [ ] Configuration examples
- [ ] Performance reports
- [ ] Code review

### 9.3 Week 3: Deployment & Monitoring

#### Day 11-12: Phased Rollout
- [ ] Deploy to dev environment
- [ ] Monitor performance metrics
- [ ] Fix issues
- [ ] Deploy to staging

#### Day 13-14: Production
- [ ] Production deployment
- [ ] Monitor real-world performance
- [ ] Collect user feedback
- [ ] Iterate on configuration

#### Day 15: Final Report
- [ ] Performance analysis report
- [ ] Lessons learned
- [ ] Phase 5 planning (P1 optimizations)

---

## 10. Success Criteria

### 10.1 Performance Targets

✅ **WASM Extract**: ≥30% improvement (350ms → ≤245ms)
✅ **Headless Extract**: ≥10% improvement (8200ms → ≤7380ms)
✅ **Warm Checkout**: ≥90% improvement (8200ms → ≤820ms)
✅ **Memory Peak**: ≤40% reduction (1690MB → ≤1014MB)
✅ **Throughput**: ≥2x improvement (10 req/s → ≥20 req/s)

### 10.2 Quality Targets

✅ **Test Coverage**: ≥90%
✅ **Code Quality**: ≥85/100
✅ **Documentation**: Complete
✅ **Build Status**: Passing
✅ **Zero Regression**: No performance regressions

### 10.3 Operational Targets

✅ **Resource Leaks**: Zero detected
✅ **Crash Rate**: <0.1%
✅ **Cache Hit Rate**: ≥80% after warmup
✅ **Timeout Success Rate**: ≥95%

---

## Appendix A: Performance Monitoring

### CLI Commands

```bash
# View browser pool status
riptide pool status

# View WASM cache statistics
riptide wasm cache-stats

# View adaptive timeout profiles
riptide timeout profiles

# Comprehensive performance report
riptide performance report
```

### Metrics Output

```json
{
  "browser_pool": {
    "total_instances": 5,
    "available": 2,
    "in_use": 3,
    "avg_checkout_time_ms": 50,
    "total_checkouts": 1234,
    "health_check_failures": 0
  },
  "wasm_cache": {
    "total_entries": 15,
    "cache_size_mb": 45,
    "hit_rate": 87.5,
    "avg_load_time_ms": 30,
    "avg_compile_time_ms": 230
  },
  "adaptive_timeout": {
    "total_requests": 5678,
    "timeouts": 234,
    "success_rate": 95.9,
    "time_saved_secs": 12345,
    "learned_domains": 42
  }
}
```

---

## Appendix B: Testing Strategy

### Test Categories

**Unit Tests:**
- Browser pool manager operations
- WASM cache operations
- Adaptive timeout calculations
- Profile learning algorithms

**Integration Tests:**
- Engine integration with optimizations
- Multi-optimization scenarios
- Error handling and recovery
- Resource cleanup

**Performance Tests:**
- Benchmark before/after comparisons
- Stress tests (1000+ concurrent)
- Memory leak detection
- Cache efficiency

**End-to-End Tests:**
- Real-world extraction scenarios
- Multi-domain batch processing
- Long-running stability tests

---

## Appendix C: Configuration Tuning Guide

### Small Workloads (<100 req/day)
```toml
[optimizations.browser_pool]
min_instances = 1
max_instances = 2
target_available = 1

[optimizations.wasm_cache]
max_cache_size_mb = 100
```

### Medium Workloads (100-1000 req/day)
```toml
[optimizations.browser_pool]
min_instances = 2
max_instances = 5
target_available = 2

[optimizations.wasm_cache]
max_cache_size_mb = 500
```

### Large Workloads (>1000 req/day)
```toml
[optimizations.browser_pool]
min_instances = 3
max_instances = 10
target_available = 5

[optimizations.wasm_cache]
max_cache_size_mb = 1000
```

---

**End of Architecture Specification**

*This document defines the complete architecture for Phase 4 P0 optimizations. Implementation agents should refer to this specification for detailed component designs, APIs, and integration patterns.*

**Next Steps:**
1. Implementation team review
2. Technical validation
3. Begin Week 1 implementation
4. Coordinate via Hive Mind collective memory

---

**Document Status:** ✅ COMPLETE
**Ready for Implementation:** YES
**Collective Approval:** PENDING
