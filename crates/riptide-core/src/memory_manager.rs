use crate::error::CoreError;
use crate::{report_error, report_panic_prevention};
use anyhow::{anyhow, Result};
use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use wasmtime::component::Component;
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
            memory_pressure_threshold: 80.0, // 80%
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
}

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
        self.usage_count += 1;
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

/// Memory manager for WASM instances with pooling and optimization
pub struct MemoryManager {
    config: MemoryManagerConfig,
    engine: Engine,
    available_instances: Arc<Mutex<VecDeque<TrackedWasmInstance>>>,
    in_use_instances: Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
    total_memory_usage: Arc<AtomicU64>,
    total_instances: Arc<AtomicUsize>,
    #[allow(dead_code)] // TODO: wire into metrics
    peak_memory_usage: Arc<AtomicU64>,
    gc_runs: Arc<AtomicU64>,

    // Event system
    event_sender: mpsc::UnboundedSender<MemoryEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<MemoryEvent>>>,

    // Monitoring and management
    #[allow(dead_code)] // TODO: send stats summary at end-of-run
    stats_sender: watch::Sender<MemoryStats>,
    stats_receiver: watch::Receiver<MemoryStats>,
    shutdown_sender: mpsc::Sender<()>,
    _management_task: tokio::task::JoinHandle<()>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub async fn new(config: MemoryManagerConfig, engine: Engine) -> Result<Self> {
        info!(
            max_memory_mb = config.max_total_memory_mb,
            max_instances = config.max_instances,
            min_instances = config.min_instances,
            "Initializing memory manager"
        );

        let available_instances = Arc::new(Mutex::new(VecDeque::new()));
        let in_use_instances = Arc::new(RwLock::new(HashMap::new()));
        let total_memory_usage = Arc::new(AtomicU64::new(0));
        let total_instances = Arc::new(AtomicUsize::new(0));
        let peak_memory_usage = Arc::new(AtomicU64::new(0));
        let gc_runs = Arc::new(AtomicU64::new(0));

        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let event_receiver = Arc::new(Mutex::new(event_receiver));

        let (stats_sender, stats_receiver) = watch::channel(MemoryStats::default());

        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);

        // Start management task for monitoring and garbage collection
        let management_task = {
            let config = config.clone();
            let available_instances = available_instances.clone();
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

                loop {
                    tokio::select! {
                        _ = monitoring_interval.tick() => {
                            Self::update_memory_stats(
                                &config,
                                &available_instances,
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
                                &available_instances,
                                &in_use_instances,
                                &total_memory_usage,
                                &total_instances,
                                &gc_runs,
                                &event_sender,
                            ).await;
                        }
                        _ = shutdown_receiver.recv() => {
                            info!("Memory manager shutting down");
                            break;
                        }
                    }
                }
            })
        };

        info!("Memory manager initialized successfully");

        Ok(Self {
            config,
            engine,
            available_instances,
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

        // Try to get available instance
        let mut instance = {
            let mut available = self.available_instances.lock().await;
            available.pop_front()
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

            // Move to in-use collection
            {
                let mut in_use = self.in_use_instances.write().await;
                in_use.insert(instance_id.clone(), instance);
            }

            debug!(instance_id = %instance_id, "WASM instance checked out");

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
                // Return to available pool
                let mut available = self.available_instances.lock().await;
                available.push_back(instance);

                debug!(instance_id = %instance_id, "WASM instance returned to pool");
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
            &self.available_instances,
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
            let error = CoreError::from(e); // Use From trait conversion
            report_error!(&error, "create_wasm_instance", "component_path" => component_path);
            error
        })?;

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
        available_instances: &Arc<Mutex<VecDeque<TrackedWasmInstance>>>,
        in_use_instances: &Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
        total_memory_usage: &Arc<AtomicU64>,
        total_instances: &Arc<AtomicUsize>,
        peak_memory_usage: &Arc<AtomicU64>,
        gc_runs: &Arc<AtomicU64>,
        stats_sender: &watch::Sender<MemoryStats>,
        event_sender: &mpsc::UnboundedSender<MemoryEvent>,
    ) {
        let available_count = available_instances.lock().await.len();
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
            idle_instances: available_count,
            peak_memory_mb: peak_memory,
            gc_runs: current_gc_runs,
            memory_pressure,
            last_updated: Some(Instant::now()),
        };

        let _ = stats_sender.send(stats);
    }

    /// Perform garbage collection on WASM instances
    #[allow(clippy::too_many_arguments)]
    async fn perform_garbage_collection(
        config: &MemoryManagerConfig,
        available_instances: &Arc<Mutex<VecDeque<TrackedWasmInstance>>>,
        _in_use_instances: &Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
        total_memory_usage: &Arc<AtomicU64>,
        total_instances: &Arc<AtomicUsize>,
        gc_runs: &Arc<AtomicU64>,
        event_sender: &mpsc::UnboundedSender<MemoryEvent>,
    ) {
        let mut instances_affected = 0;
        let mut memory_freed = 0u64;

        // Clean up idle instances
        {
            let mut available = available_instances.lock().await;
            let mut i = 0;

            while i < available.len() {
                let instance = &available[i];
                let should_remove = instance.is_idle(config.instance_idle_timeout)
                    || instance.current_memory_mb() > config.instance_memory_threshold_mb
                    || (config.aggressive_gc && available.len() > config.min_instances);

                if should_remove {
                    // Prevent potential panic with bounds checking
                    let instance = match available.get(i) {
                        Some(_) => available
                            .remove(i)
                            .expect("Index verified to exist through bounds check"),
                        None => {
                            let error_msg =
                                format!("Index {} out of bounds during garbage collection", i);
                            error!("{}", error_msg);
                            report_panic_prevention!(
                                "garbage_collection",
                                "vector_index_out_of_bounds",
                                "bounds_check_and_skip"
                            );
                            i += 1;
                            continue;
                        }
                    };
                    let freed = instance.current_memory_mb();
                    memory_freed += freed;
                    instances_affected += 1;

                    total_memory_usage.fetch_sub(freed, Ordering::Relaxed);
                    total_instances.fetch_sub(1, Ordering::Relaxed);

                    debug!(
                        instance_id = %instance.id,
                        memory_freed_mb = freed,
                        "Instance garbage collected"
                    );
                } else {
                    i += 1;
                }
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
            let mut available = self.available_instances.lock().await;
            available.clear();
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
pub struct MemoryManagerRef {
    available_instances: Arc<Mutex<VecDeque<TrackedWasmInstance>>>,
    in_use_instances: Arc<RwLock<HashMap<String, TrackedWasmInstance>>>,
    event_sender: mpsc::UnboundedSender<MemoryEvent>,
}

impl MemoryManagerRef {
    fn new(manager: &MemoryManager) -> Self {
        Self {
            available_instances: Arc::clone(&manager.available_instances),
            in_use_instances: Arc::clone(&manager.in_use_instances),
            event_sender: manager.event_sender.clone(),
        }
    }

    pub async fn return_instance(&self, instance_id: &str) -> Result<()> {
        let mut in_use = self.in_use_instances.write().await;

        if let Some(instance) = in_use.remove(instance_id) {
            drop(in_use);

            let mut available = self.available_instances.lock().await;
            available.push_back(instance);

            // Notify about instance return
            let _ = self.event_sender.send(MemoryEvent::PoolShrunk {
                new_size: available.len(),
                reason: format!("Instance {} returned to pool", instance_id),
            });

            Ok(())
        } else {
            Err(anyhow!("Instance {} not found in use", instance_id))
        }
    }
}

/// Handle for a checked-out WASM instance
pub struct WasmInstanceHandle {
    instance_id: String,
    manager: MemoryManagerRef,
}

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
    pub async fn cleanup(self) -> Result<()> {
        tokio::time::timeout(
            Duration::from_secs(5),
            self.manager.return_instance(&self.instance_id),
        )
        .await
        .map_err(|_| anyhow!("Timeout returning instance {} to pool", self.instance_id))?
    }
}

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

#[cfg(test)]
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
