use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

#[cfg(feature = "wasm-pool")]
use riptide_extraction::validate_before_instantiation;
#[cfg(feature = "wasm-pool")]
use wasmtime::component::Component;
#[cfg(feature = "wasm-pool")]
use wasmtime::{Engine, Store};

/// Configuration for memory management and monitoring
#[derive(Clone, Debug)]
pub struct MemoryManagerConfig {
    /// Maximum total memory usage across all WASM instances (MB)
    pub max_total_memory_mb: u64,
    /// Memory threshold for single instance (MB)
    pub instance_memory_threshold_mb: u64,
    /// Maximum number of WASM instances in pool
    pub max_instances: usize,
    /// Minimum number of WASM instances to keep warm
    pub min_instances: usize,
    /// Idle timeout for WASM instances (seconds)
    pub instance_idle_timeout: Duration,
    /// Memory monitoring interval (seconds)
    pub monitoring_interval: Duration,
    /// Garbage collection interval (seconds)
    pub gc_interval: Duration,
    /// Enable aggressive memory management
    pub aggressive_gc: bool,
    /// Memory pressure threshold (percentage)
    pub memory_pressure_threshold: f64,
    /// Cleanup timeout for resource cleanup operations (seconds)
    pub cleanup_timeout: Duration,
    /// Enable WIT (WebAssembly Interface Types) validation before component instantiation
    pub enable_wit_validation: bool,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        Self {
            max_total_memory_mb: 2048,         // 2GB total
            instance_memory_threshold_mb: 256, // 256MB per instance
            max_instances: 8,
            min_instances: 2,
            instance_idle_timeout: Duration::from_secs(60),
            monitoring_interval: Duration::from_secs(5),
            gc_interval: Duration::from_secs(30),
            aggressive_gc: false,
            memory_pressure_threshold: 80.0,         // 80%
            cleanup_timeout: Duration::from_secs(5), // 5 second cleanup timeout
            enable_wit_validation: true,             // Enable WIT validation by default
        }
    }
}

/// Memory usage statistics
#[derive(Clone, Debug, Default)]
pub struct MemoryStats {
    pub total_allocated_mb: u64,
    pub total_used_mb: u64,
    pub instances_count: usize,
    pub active_instances: usize,
    pub idle_instances: usize,
    pub peak_memory_mb: u64,
    pub gc_runs: u64,
    pub memory_pressure: f64,
    pub last_updated: Option<Instant>,
    // P2-1: Stratified pool metrics
    pub pool_hot_count: usize,
    pub pool_warm_count: usize,
    pub pool_cold_count: usize,
    pub pool_hot_hits: u64,
    pub pool_warm_hits: u64,
    pub pool_cold_misses: u64,
    pub pool_promotions: u64,
}

/// Memory alerts and events
#[derive(Debug, Clone)]
pub enum MemoryEvent {
    MemoryThresholdExceeded {
        instance_id: String,
        usage_mb: u64,
        threshold_mb: u64,
    },
    MemoryPressureHigh {
        current_usage: u64,
        max_usage: u64,
        pressure_percent: f64,
    },
    InstanceEvicted {
        instance_id: String,
        reason: String,
        memory_freed_mb: u64,
    },
    GarbageCollectionTriggered {
        instances_affected: usize,
        memory_freed_mb: u64,
    },
    MemoryLeakDetected {
        instance_id: String,
        growth_rate_mb_per_sec: f64,
    },
    PoolExpanded {
        new_size: usize,
        reason: String,
    },
    PoolShrunk {
        new_size: usize,
        reason: String,
    },
}

/// WASM instance with memory tracking
#[cfg(feature = "wasm-pool")]
pub struct TrackedWasmInstance {
    pub id: String,
    pub store: Store<()>,
    pub component: Component,
    pub created_at: Instant,
    pub last_used: Instant,
    pub usage_count: u64,
    pub memory_usage_mb: AtomicU64,
    pub memory_growth_history: VecDeque<(Instant, u64)>,
    pub in_use: bool,
    pub peak_memory_mb: u64,
    /// Pool tier this instance belongs to (hot=0, warm=1, cold=2)
    pub pool_tier: u8,
    /// Access frequency score for promotion decisions
    pub access_frequency: f64,
}

#[cfg(feature = "wasm-pool")]
impl TrackedWasmInstance {
    pub fn new(id: String, store: Store<()>, component: Component) -> Self {
        let now = Instant::now();
        Self {
            id,
            store,
            component,
            created_at: now,
            last_used: now,
            usage_count: 0,
            memory_usage_mb: AtomicU64::new(0),
            memory_growth_history: VecDeque::new(),
            in_use: false,
            peak_memory_mb: 0,
            pool_tier: 2, // Start in cold tier
            access_frequency: 0.0,
        }
    }

    /// Update access frequency for pool promotion decisions
    pub fn update_access_frequency(&mut self) {
        let time_since_created = self.created_at.elapsed().as_secs_f64();
        if time_since_created > 0.0 {
            // Exponential moving average with decay
            let new_access = 1.0 / time_since_created;
            self.access_frequency = 0.7 * self.access_frequency + 0.3 * new_access;
        }
    }

    /// Update memory usage and track growth
    pub fn update_memory_usage(&mut self, usage_mb: u64) {
        self.memory_usage_mb.store(usage_mb, Ordering::Relaxed);

        if usage_mb > self.peak_memory_mb {
            self.peak_memory_mb = usage_mb;
        }

        // Track memory growth history (keep last 10 measurements)
        self.memory_growth_history
            .push_back((Instant::now(), usage_mb));
        if self.memory_growth_history.len() > 10 {
            self.memory_growth_history.pop_front();
        }

        self.last_used = Instant::now();
        self.usage_count = self.usage_count.saturating_add(1);
        self.update_access_frequency();
    }

    /// Calculate memory growth rate (MB per second)
    pub fn memory_growth_rate(&self) -> Option<f64> {
        if self.memory_growth_history.len() < 2 {
            return None;
        }

        let (oldest_time, oldest_memory) = self.memory_growth_history.front()?;
        let (newest_time, newest_memory) = self.memory_growth_history.back()?;

        let time_diff = newest_time.duration_since(*oldest_time).as_secs_f64();
        if time_diff > 0.0 {
            let memory_diff = *newest_memory as f64 - *oldest_memory as f64;
            Some(memory_diff / time_diff)
        } else {
            None
        }
    }

    /// Check if instance is idle
    pub fn is_idle(&self, idle_timeout: Duration) -> bool {
        !self.in_use && self.last_used.elapsed() > idle_timeout
    }

    /// Get current memory usage
    pub fn current_memory_mb(&self) -> u64 {
        self.memory_usage_mb.load(Ordering::Relaxed)
    }
}

/// Stratified instance pool with hot/warm/cold tiers
///
/// P2-1: 3-tier pool architecture for 40-60% latency reduction
/// - Hot tier: Ready instantly (0-5ms)
/// - Warm tier: Fast activation (10-50ms)
/// - Cold tier: Create on demand (100-200ms)
#[cfg(feature = "wasm-pool")]
pub struct StratifiedInstancePool {
    hot: VecDeque<TrackedWasmInstance>,
    warm: VecDeque<TrackedWasmInstance>,
    cold: VecDeque<TrackedWasmInstance>,
    hot_capacity: usize,
    warm_capacity: usize,
    // Metrics
    hot_hits: Arc<AtomicU64>,
    warm_hits: Arc<AtomicU64>,
    cold_misses: Arc<AtomicU64>,
    promotions: Arc<AtomicU64>,
}

#[cfg(feature = "wasm-pool")]
impl StratifiedInstancePool {
    pub fn new(hot_cap: usize, warm_cap: usize) -> Self {
        Self {
            hot: VecDeque::with_capacity(hot_cap),
            warm: VecDeque::with_capacity(warm_cap),
            cold: VecDeque::new(),
            hot_capacity: hot_cap,
            warm_capacity: warm_cap,
            hot_hits: Arc::new(AtomicU64::new(0)),
            warm_hits: Arc::new(AtomicU64::new(0)),
            cold_misses: Arc::new(AtomicU64::new(0)),
            promotions: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Acquire an instance from the pool (tries hot → warm → cold)
    pub fn acquire(&mut self) -> Option<TrackedWasmInstance> {
        // Try hot tier first (instant access)
        if let Some(mut instance) = self.hot.pop_front() {
            self.hot_hits.fetch_add(1, Ordering::Relaxed);
            instance.pool_tier = 0;
            debug!(
                instance_id = %instance.id,
                tier = "hot",
                "Instance acquired from hot tier"
            );
            return Some(instance);
        }

        // Try warm tier (fast activation)
        if let Some(mut instance) = self.warm.pop_front() {
            self.warm_hits.fetch_add(1, Ordering::Relaxed);
            instance.pool_tier = 1;
            debug!(
                instance_id = %instance.id,
                tier = "warm",
                "Instance acquired from warm tier"
            );
            return Some(instance);
        }

        // Try cold tier (slower)
        if let Some(mut instance) = self.cold.pop_front() {
            self.cold_misses.fetch_add(1, Ordering::Relaxed);
            instance.pool_tier = 2;
            debug!(
                instance_id = %instance.id,
                tier = "cold",
                "Instance acquired from cold tier"
            );
            return Some(instance);
        }

        // No instances available
        None
    }

    /// Release an instance back to the pool
    pub fn release(&mut self, mut instance: TrackedWasmInstance) {
        let instance_id = instance.id.clone();
        let access_freq = instance.access_frequency;

        // Promote to hot tier if high access frequency and space available
        if access_freq > 0.5 && self.hot.len() < self.hot_capacity {
            instance.pool_tier = 0;
            self.hot.push_back(instance);
            self.promotions.fetch_add(1, Ordering::Relaxed);
            debug!(
                instance_id = %instance_id,
                tier = "hot",
                access_frequency = access_freq,
                "Instance promoted to hot tier"
            );
        }
        // Place in warm tier if moderate frequency and space available
        else if access_freq > 0.2 && self.warm.len() < self.warm_capacity {
            instance.pool_tier = 1;
            self.warm.push_back(instance);
            debug!(
                instance_id = %instance_id,
                tier = "warm",
                access_frequency = access_freq,
                "Instance placed in warm tier"
            );
        }
        // Otherwise place in cold tier
        else {
            instance.pool_tier = 2;
            self.cold.push_back(instance);
            debug!(
                instance_id = %instance_id,
                tier = "cold",
                access_frequency = access_freq,
                "Instance placed in cold tier"
            );
        }
    }

    /// Promote warm instances to hot tier based on usage patterns
    pub fn promote_warm_to_hot(&mut self) {
        let mut promoted_count = 0;

        // Check if hot tier has space
        while self.hot.len() < self.hot_capacity && !self.warm.is_empty() {
            // Find highest frequency warm instance
            let mut best_idx = 0;
            let mut best_freq = 0.0;

            for (idx, instance) in self.warm.iter().enumerate() {
                if instance.access_frequency > best_freq {
                    best_freq = instance.access_frequency;
                    best_idx = idx;
                }
            }

            // Promote if frequency is high enough
            if best_freq > 0.4 {
                if let Some(mut instance) = self.warm.remove(best_idx) {
                    instance.pool_tier = 0;
                    self.hot.push_back(instance);
                    promoted_count = promoted_count.saturating_add(1);
                    self.promotions.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                break; // No more worthy candidates
            }
        }

        if promoted_count > 0 {
            debug!(
                count = promoted_count,
                "Promoted instances from warm to hot tier"
            );
        }
    }

    /// Get pool metrics
    pub fn metrics(&self) -> PoolMetrics {
        PoolMetrics {
            hot_count: self.hot.len(),
            warm_count: self.warm.len(),
            cold_count: self.cold.len(),
            hot_hits: self.hot_hits.load(Ordering::Relaxed),
            warm_hits: self.warm_hits.load(Ordering::Relaxed),
            cold_misses: self.cold_misses.load(Ordering::Relaxed),
            promotions: self.promotions.load(Ordering::Relaxed),
            total_instances: self
                .hot
                .len()
                .saturating_add(self.warm.len())
                .saturating_add(self.cold.len()),
        }
    }

    /// Get total number of instances across all tiers
    pub fn total_count(&self) -> usize {
        self.hot
            .len()
            .saturating_add(self.warm.len())
            .saturating_add(self.cold.len())
    }

    /// Clear all tiers
    pub fn clear(&mut self) {
        self.hot.clear();
        self.warm.clear();
        self.cold.clear();
    }

    /// Remove idle instances from all tiers and return count removed
    pub fn remove_idle(&mut self, idle_timeout: Duration) -> usize {
        let mut removed_count = 0;

        // Remove from cold tier first (least valuable)
        let cold_len = self.cold.len();
        self.cold.retain(|instance| !instance.is_idle(idle_timeout));
        removed_count = removed_count.saturating_add(cold_len.saturating_sub(self.cold.len()));

        // Remove from warm tier
        let warm_len = self.warm.len();
        self.warm.retain(|instance| !instance.is_idle(idle_timeout));
        removed_count = removed_count.saturating_add(warm_len.saturating_sub(self.warm.len()));

        // Remove from hot tier only if severely idle
        let hot_len = self.hot.len();
        self.hot
            .retain(|instance| !instance.is_idle(idle_timeout.saturating_mul(2)));
        removed_count = removed_count.saturating_add(hot_len.saturating_sub(self.hot.len()));

        removed_count
    }
}

// Note: TrackedWasmInstance cannot be cloned due to Store and Component
// We'll use a different approach for remove_idle without cloning

/// Pool metrics for monitoring
#[derive(Debug, Clone)]
pub struct PoolMetrics {
    pub hot_count: usize,
    pub warm_count: usize,
    pub cold_count: usize,
    pub hot_hits: u64,
    pub warm_hits: u64,
    pub cold_misses: u64,
    pub promotions: u64,
    pub total_instances: usize,
}

/// Memory manager for WASM instances with pooling and optimization
#[cfg(feature = "wasm-pool")]
pub struct MemoryManager {
    config: MemoryManagerConfig,
    engine: Engine,
    // P2-1: Replace simple queue with stratified pool
    stratified_pool: Arc<Mutex<StratifiedInstancePool>>,
    in_use_instances: Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
    total_memory_usage: Arc<AtomicU64>,
    total_instances: Arc<AtomicUsize>,
    #[allow(dead_code)] // Used by management task through Arc clone in update_memory_stats
    peak_memory_usage: Arc<AtomicU64>,
    gc_runs: Arc<AtomicU64>,

    // Event system
    event_sender: mpsc::UnboundedSender<MemoryEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<MemoryEvent>>>,

    // Monitoring and management
    #[allow(dead_code)] // Used by management task to send stats updates via watch channel
    stats_sender: watch::Sender<MemoryStats>,
    stats_receiver: watch::Receiver<MemoryStats>,
    shutdown_sender: mpsc::Sender<()>,
    _management_task: tokio::task::JoinHandle<()>,
}

#[cfg(feature = "wasm-pool")]
impl MemoryManager {
    /// Create a new memory manager
    pub async fn new(config: MemoryManagerConfig, engine: Engine) -> Result<Self> {
        info!(
            max_memory_mb = config.max_total_memory_mb,
            max_instances = config.max_instances,
            min_instances = config.min_instances,
            "Initializing memory manager with stratified pool"
        );

        // P2-1: Initialize stratified pool with hot/warm tiers
        // Hot tier: 25% of max_instances, Warm tier: 50% of max_instances
        let hot_capacity = config.max_instances.checked_div(4).unwrap_or(0).max(1);
        let warm_capacity = config.max_instances.checked_div(2).unwrap_or(0).max(2);

        let stratified_pool = Arc::new(Mutex::new(StratifiedInstancePool::new(
            hot_capacity,
            warm_capacity,
        )));

        let in_use_instances = Arc::new(RwLock::new(HashMap::new()));
        let total_memory_usage = Arc::new(AtomicU64::new(0));
        let total_instances = Arc::new(AtomicUsize::new(0));
        let peak_memory_usage = Arc::new(AtomicU64::new(0));
        let gc_runs = Arc::new(AtomicU64::new(0));

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let event_receiver = Arc::new(Mutex::new(event_receiver));

        let (stats_sender, stats_receiver) = watch::channel(MemoryStats::default());

        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);

        // Start management task for monitoring, GC, and pool promotion
        let management_task = {
            let config = config.clone();
            let stratified_pool = stratified_pool.clone();
            let in_use_instances = in_use_instances.clone();
            let total_memory_usage = total_memory_usage.clone();
            let total_instances = total_instances.clone();
            let peak_memory_usage = peak_memory_usage.clone();
            let gc_runs = gc_runs.clone();
            let event_sender = event_sender.clone();
            let stats_sender = stats_sender.clone();

            tokio::spawn(async move {
                let mut monitoring_interval = interval(config.monitoring_interval);
                let mut gc_interval = interval(config.gc_interval);
                let mut promotion_interval = interval(Duration::from_secs(5)); // Pool promotion every 5s

                loop {
                    tokio::select! {
                        _ = monitoring_interval.tick() => {
                            Self::update_memory_stats(
                                &config,
                                &stratified_pool,
                                &in_use_instances,
                                &total_memory_usage,
                                &total_instances,
                                &peak_memory_usage,
                                &gc_runs,
                                &stats_sender,
                                &event_sender,
                            ).await;
                        }
                        _ = gc_interval.tick() => {
                            Self::perform_garbage_collection(
                                &config,
                                &stratified_pool,
                                &in_use_instances,
                                &total_memory_usage,
                                &total_instances,
                                &gc_runs,
                                &event_sender,
                            ).await;
                        }
                        _ = promotion_interval.tick() => {
                            // P2-1: Background promotion task
                            let mut pool = stratified_pool.lock().await;
                            pool.promote_warm_to_hot();
                        }
                        _ = shutdown_receiver.recv() => {
                            info!("Memory manager shutting down");
                            break;
                        }
                    }
                }
            })
        };

        info!(
            hot_capacity = hot_capacity,
            warm_capacity = warm_capacity,
            "Memory manager with stratified pool initialized successfully"
        );

        Ok(Self {
            config,
            engine,
            stratified_pool,
            in_use_instances,
            total_memory_usage,
            total_instances,
            peak_memory_usage,
            gc_runs,
            event_sender,
            event_receiver,
            stats_sender,
            stats_receiver,
            shutdown_sender,
            _management_task: management_task,
        })
    }

    /// Get a WASM instance from the pool or create a new one
    pub async fn get_instance(&self, component_path: &str) -> Result<WasmInstanceHandle> {
        // Check memory pressure before allocation
        let current_memory = self.total_memory_usage.load(Ordering::Relaxed);
        let memory_pressure =
            (current_memory as f64 / self.config.max_total_memory_mb as f64) * 100.0;

        if memory_pressure > self.config.memory_pressure_threshold {
            // Try garbage collection first
            self.trigger_garbage_collection().await;

            // Recheck memory pressure
            let current_memory = self.total_memory_usage.load(Ordering::Relaxed);
            let memory_pressure =
                (current_memory as f64 / self.config.max_total_memory_mb as f64) * 100.0;

            if memory_pressure > self.config.memory_pressure_threshold {
                return Err(anyhow!(
                    "Memory pressure too high: {:.2}% (threshold: {:.2}%)",
                    memory_pressure,
                    self.config.memory_pressure_threshold
                ));
            }
        }

        // P2-1: Try to get instance from stratified pool
        let mut instance = {
            let mut pool = self.stratified_pool.lock().await;
            pool.acquire()
        };

        // Create new instance if none available
        if instance.is_none() {
            let instance_count = self.total_instances.load(Ordering::Relaxed);
            if instance_count >= self.config.max_instances {
                return Err(anyhow!(
                    "Maximum number of instances reached: {}",
                    self.config.max_instances
                ));
            }

            instance = Some(self.create_new_instance(component_path).await?);
        }

        if let Some(mut instance) = instance {
            instance.in_use = true;
            let instance_id = instance.id.clone();
            let tier = instance.pool_tier;

            // Move to in-use collection
            {
                let mut in_use = self.in_use_instances.write().await;
                in_use.insert(instance_id.clone(), instance);
            }

            debug!(
                instance_id = %instance_id,
                tier = tier,
                "WASM instance checked out from tier"
            );

            Ok(WasmInstanceHandle {
                instance_id,
                manager: MemoryManagerRef::new(self),
            })
        } else {
            Err(anyhow!("Failed to obtain WASM instance"))
        }
    }

    /// Return a WASM instance to the pool
    pub async fn return_instance(&self, instance_id: &str) -> Result<()> {
        let instance_option = {
            let mut in_use = self.in_use_instances.write().await;
            in_use.remove(instance_id)
        };

        if let Some(mut instance) = instance_option {
            instance.in_use = false;

            // Check if instance should be kept or discarded
            let should_keep = instance.current_memory_mb()
                <= self.config.instance_memory_threshold_mb
                && !self.is_memory_leak_detected(&instance);

            if should_keep {
                // P2-1: Return to stratified pool (will be placed in appropriate tier)
                let mut pool = self.stratified_pool.lock().await;
                pool.release(instance);

                debug!(instance_id = %instance_id, "WASM instance returned to stratified pool");
            } else {
                // Discard instance
                let memory_freed = instance.current_memory_mb();
                self.total_memory_usage
                    .fetch_sub(memory_freed, Ordering::Relaxed);
                self.total_instances.fetch_sub(1, Ordering::Relaxed);

                let reason =
                    if instance.current_memory_mb() > self.config.instance_memory_threshold_mb {
                        "Memory threshold exceeded"
                    } else {
                        "Memory leak detected"
                    };

                let _ = self.event_sender.send(MemoryEvent::InstanceEvicted {
                    instance_id: instance_id.to_string(),
                    reason: reason.to_string(),
                    memory_freed_mb: memory_freed,
                });

                warn!(
                    instance_id = %instance_id,
                    memory_freed_mb = memory_freed,
                    reason = reason,
                    "WASM instance discarded"
                );
            }
        }

        Ok(())
    }

    /// Trigger garbage collection manually
    pub async fn trigger_garbage_collection(&self) {
        Self::perform_garbage_collection(
            &self.config,
            &self.stratified_pool,
            &self.in_use_instances,
            &self.total_memory_usage,
            &self.total_instances,
            &self.gc_runs,
            &self.event_sender,
        )
        .await;
    }

    /// Get current memory statistics
    pub fn stats(&self) -> MemoryStats {
        self.stats_receiver.borrow().clone()
    }

    /// Get memory events for monitoring
    pub fn events(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<MemoryEvent>>> {
        self.event_receiver.clone()
    }

    /// Create a new WASM instance with error recovery
    async fn create_new_instance(&self, component_path: &str) -> Result<TrackedWasmInstance> {
        let id = uuid::Uuid::new_v4().to_string();

        debug!(instance_id = %id, component_path = %component_path, "Creating new WASM instance");

        let store = Store::new(&self.engine, ());

        // Create component with recovery strategy
        let component = Component::from_file(&self.engine, component_path).map_err(|e| {
            error!(%e, component_path = %component_path, "Failed to create WASM component from file");
            anyhow!("Failed to create WASM component: {}", e)
        })?;

        // P2-2: WIT validation before instantiation
        if self.config.enable_wit_validation {
            debug!(instance_id = %id, "Running WIT validation before component instantiation");
            if let Err(e) = validate_before_instantiation(&component) {
                warn!(instance_id = %id, error = %e, component_path = %component_path,
                      "WIT validation failed for WASM instance");
                error!(%e, component_path = %component_path, "WIT validation failed");
                return Err(anyhow!("WIT validation failed: {}", e));
            }
            debug!(instance_id = %id, "WIT validation passed successfully");
        }

        let instance = TrackedWasmInstance::new(id, store, component);

        self.total_instances.fetch_add(1, Ordering::Relaxed);

        debug!(instance_id = %instance.id, "WASM instance created successfully");

        Ok(instance)
    }

    /// Check if memory leak is detected for an instance
    fn is_memory_leak_detected(&self, instance: &TrackedWasmInstance) -> bool {
        if let Some(growth_rate) = instance.memory_growth_rate() {
            // Consider it a leak if memory is growing faster than 10MB/second
            growth_rate > 10.0
        } else {
            false
        }
    }

    /// Update memory statistics
    #[allow(clippy::too_many_arguments)]
    async fn update_memory_stats(
        config: &MemoryManagerConfig,
        stratified_pool: &Arc<Mutex<StratifiedInstancePool>>,
        in_use_instances: &Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
        total_memory_usage: &Arc<AtomicU64>,
        total_instances: &Arc<AtomicUsize>,
        peak_memory_usage: &Arc<AtomicU64>,
        gc_runs: &Arc<AtomicU64>,
        stats_sender: &watch::Sender<MemoryStats>,
        event_sender: &mpsc::UnboundedSender<MemoryEvent>,
    ) {
        // P2-1: Get stratified pool metrics
        let pool_metrics = {
            let pool = stratified_pool.lock().await;
            pool.metrics()
        };

        let in_use_count = in_use_instances.read().await.len();
        let total_memory = total_memory_usage.load(Ordering::Relaxed);
        let peak_memory = peak_memory_usage.load(Ordering::Relaxed);
        let current_gc_runs = gc_runs.load(Ordering::Relaxed);

        // Update peak memory
        if total_memory > peak_memory {
            peak_memory_usage.store(total_memory, Ordering::Relaxed);
        }

        // Calculate memory pressure
        let memory_pressure = (total_memory as f64 / config.max_total_memory_mb as f64) * 100.0;

        // Check for high memory pressure
        if memory_pressure > config.memory_pressure_threshold {
            let _ = event_sender.send(MemoryEvent::MemoryPressureHigh {
                current_usage: total_memory,
                max_usage: config.max_total_memory_mb,
                pressure_percent: memory_pressure,
            });
        }

        // Check for memory leaks in in-use instances
        {
            let in_use = in_use_instances.read().await;
            for instance in in_use.values() {
                if let Some(growth_rate) = instance.memory_growth_rate() {
                    if growth_rate > 10.0 {
                        // 10MB/second threshold
                        let _ = event_sender.send(MemoryEvent::MemoryLeakDetected {
                            instance_id: instance.id.clone(),
                            growth_rate_mb_per_sec: growth_rate,
                        });
                    }
                }

                // Check instance memory threshold
                let current_memory = instance.current_memory_mb();
                if current_memory > config.instance_memory_threshold_mb {
                    let _ = event_sender.send(MemoryEvent::MemoryThresholdExceeded {
                        instance_id: instance.id.clone(),
                        usage_mb: current_memory,
                        threshold_mb: config.instance_memory_threshold_mb,
                    });
                }
            }
        }

        let stats = MemoryStats {
            total_allocated_mb: config.max_total_memory_mb,
            total_used_mb: total_memory,
            instances_count: total_instances.load(Ordering::Relaxed),
            active_instances: in_use_count,
            idle_instances: pool_metrics.total_instances,
            peak_memory_mb: peak_memory,
            gc_runs: current_gc_runs,
            memory_pressure,
            last_updated: Some(Instant::now()),
            // P2-1: Add stratified pool metrics
            pool_hot_count: pool_metrics.hot_count,
            pool_warm_count: pool_metrics.warm_count,
            pool_cold_count: pool_metrics.cold_count,
            pool_hot_hits: pool_metrics.hot_hits,
            pool_warm_hits: pool_metrics.warm_hits,
            pool_cold_misses: pool_metrics.cold_misses,
            pool_promotions: pool_metrics.promotions,
        };

        let _ = stats_sender.send(stats);
    }

    /// Perform garbage collection on WASM instances
    #[allow(clippy::too_many_arguments)]
    async fn perform_garbage_collection(
        config: &MemoryManagerConfig,
        stratified_pool: &Arc<Mutex<StratifiedInstancePool>>,
        _in_use_instances: &Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
        total_memory_usage: &Arc<AtomicU64>,
        total_instances: &Arc<AtomicUsize>,
        gc_runs: &Arc<AtomicU64>,
        event_sender: &mpsc::UnboundedSender<MemoryEvent>,
    ) {
        let instances_affected;
        let memory_freed;

        // P2-1: Clean up idle instances from stratified pool
        {
            let mut pool = stratified_pool.lock().await;
            let removed_count = pool.remove_idle(config.instance_idle_timeout);

            instances_affected = removed_count;
            // Approximate memory freed (we don't have exact values after removal)
            memory_freed = (removed_count as u64).saturating_mul(50); // Estimate 50MB per instance

            total_memory_usage.fetch_sub(memory_freed, Ordering::Relaxed);
            total_instances.fetch_sub(removed_count, Ordering::Relaxed);

            if removed_count > 0 {
                debug!(
                    instances_removed = removed_count,
                    "Instances garbage collected from stratified pool"
                );
            }
        }

        if instances_affected > 0 {
            gc_runs.fetch_add(1, Ordering::Relaxed);

            let _ = event_sender.send(MemoryEvent::GarbageCollectionTriggered {
                instances_affected,
                memory_freed_mb: memory_freed,
            });

            info!(
                instances_affected = instances_affected,
                memory_freed_mb = memory_freed,
                "Garbage collection completed"
            );
        }
    }

    /// Shutdown the memory manager
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down memory manager");

        // Signal management task to stop
        let _ = self.shutdown_sender.send(()).await;

        // Clean up all instances
        {
            let mut pool = self.stratified_pool.lock().await;
            pool.clear();
        }

        {
            let mut in_use = self.in_use_instances.write().await;
            in_use.clear();
        }

        info!("Memory manager shutdown completed");
        Ok(())
    }
}

/// Reference to memory manager for instance operations
/// Reference to the memory manager for checkout operations
/// Uses Arc clones to maintain strong references for safety
#[derive(Clone)]
#[cfg(feature = "wasm-pool")]
pub struct MemoryManagerRef {
    stratified_pool: Arc<Mutex<StratifiedInstancePool>>,
    in_use_instances: Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
    event_sender: mpsc::UnboundedSender<MemoryEvent>,
    config: MemoryManagerConfig,
}

#[cfg(feature = "wasm-pool")]
impl MemoryManagerRef {
    fn new(manager: &MemoryManager) -> Self {
        Self {
            stratified_pool: Arc::clone(&manager.stratified_pool),
            in_use_instances: Arc::clone(&manager.in_use_instances),
            event_sender: manager.event_sender.clone(),
            config: manager.config.clone(),
        }
    }

    pub async fn return_instance(&self, instance_id: &str) -> Result<()> {
        let mut in_use = self.in_use_instances.write().await;

        if let Some(instance) = in_use.remove(instance_id) {
            drop(in_use);

            let mut pool = self.stratified_pool.lock().await;
            pool.release(instance);

            // Notify about instance return
            let pool_metrics = pool.metrics();
            let _ = self.event_sender.send(MemoryEvent::PoolShrunk {
                new_size: pool_metrics.total_instances,
                reason: format!("Instance {} returned to pool", instance_id),
            });

            Ok(())
        } else {
            Err(anyhow!("Instance {} not found in use", instance_id))
        }
    }
}

/// Handle for a checked-out WASM instance
#[cfg(feature = "wasm-pool")]
pub struct WasmInstanceHandle {
    instance_id: String,
    manager: MemoryManagerRef,
}

#[cfg(feature = "wasm-pool")]
impl WasmInstanceHandle {
    /// Get the instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Manually return the instance to the pool (preferred over drop)
    pub async fn return_to_pool(self) -> Result<()> {
        self.manager.return_instance(&self.instance_id).await
    }

    /// Cleanup with timeout - ensures proper async cleanup
    ///
    /// Uses the configured cleanup_timeout from MemoryManagerConfig.
    /// If you need a custom timeout, use cleanup_with_timeout() instead.
    pub async fn cleanup(self) -> Result<()> {
        let timeout_duration = self.manager.config.cleanup_timeout;
        tokio::time::timeout(
            timeout_duration,
            self.manager.return_instance(&self.instance_id),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Timeout returning instance {} to pool after {:?}",
                self.instance_id,
                timeout_duration
            )
        })?
    }

    /// Cleanup with custom timeout - for cases where you need a different timeout
    #[allow(dead_code)] // Public API for custom timeout scenarios
    pub async fn cleanup_with_timeout(self, timeout_duration: Duration) -> Result<()> {
        tokio::time::timeout(
            timeout_duration,
            self.manager.return_instance(&self.instance_id),
        )
        .await
        .map_err(|_| {
            anyhow!(
                "Timeout returning instance {} to pool after {:?}",
                self.instance_id,
                timeout_duration
            )
        })?
    }
}

#[cfg(feature = "wasm-pool")]
impl Drop for WasmInstanceHandle {
    fn drop(&mut self) {
        warn!(
            instance_id = %self.instance_id,
            "WasmInstanceHandle dropped without explicit cleanup - spawning best-effort background task"
        );

        let instance_id = self.instance_id.clone();
        let manager = self.manager.clone();

        // Best-effort cleanup in background (not guaranteed to complete)
        tokio::spawn(async move {
            if let Err(e) = manager.return_instance(&instance_id).await {
                error!(
                    instance_id = %instance_id,
                    error = %e,
                    "Failed to return WASM instance during drop"
                );
            }
        });
    }
}

#[cfg(all(test, feature = "wasm-pool"))]
mod tests {
    use super::*;
    use wasmtime::{Config, Engine};

    #[tokio::test]
    async fn test_memory_manager_creation() -> Result<(), Box<dyn std::error::Error>> {
        let config = MemoryManagerConfig {
            min_instances: 1,
            max_instances: 3,
            ..Default::default()
        };

        let mut wasmtime_config = Config::new();
        wasmtime_config.wasm_component_model(true);
        let engine = Engine::new(&wasmtime_config)?;

        let manager = MemoryManager::new(config, engine).await?;
        let stats = manager.stats();
        assert_eq!(stats.instances_count, 0);
        assert_eq!(stats.active_instances, 0);

        let _ = manager.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_stats_tracking() -> Result<(), Box<dyn std::error::Error>> {
        let config = MemoryManagerConfig::default();
        let mut wasmtime_config = Config::new();
        wasmtime_config.wasm_component_model(true);
        let engine = Engine::new(&wasmtime_config)?;

        let manager = MemoryManager::new(config, engine).await?;

        let initial_stats = manager.stats();
        assert_eq!(initial_stats.total_used_mb, 0);
        assert_eq!(initial_stats.instances_count, 0);

        let _ = manager.shutdown().await;
        Ok(())
    }
}
