//! Resource limits and enforcement module
//!
//! This module provides comprehensive resource limiting and enforcement capabilities
//! to prevent resource abuse and ensure system stability under load.

use crate::{PerformanceError, PerformanceTargets, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: f64,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Maximum request rate per second
    pub max_requests_per_second: f64,
    /// Maximum request processing time
    pub max_request_duration: Duration,
    /// Maximum open file descriptors
    pub max_file_descriptors: u32,
    /// Maximum network bandwidth in Mbps
    pub max_network_bandwidth_mbps: f64,
    /// Maximum disk I/O in MB/s
    pub max_disk_io_mbps: f64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 600.0,
            max_cpu_percent: 80.0,
            max_concurrent_requests: 100,
            max_requests_per_second: 1000.0,
            max_request_duration: Duration::from_secs(30),
            max_file_descriptors: 1000,
            max_network_bandwidth_mbps: 1000.0,
            max_disk_io_mbps: 500.0,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Time window for rate limiting
    pub window_duration: Duration,
    /// Maximum requests per window
    pub max_requests_per_window: u32,
    /// Burst allowance (requests above normal rate)
    pub burst_allowance: u32,
    /// Enable adaptive rate limiting
    pub enable_adaptive: bool,
    /// Rate limit per client/IP
    pub per_client_limit: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            window_duration: Duration::from_secs(60), // 1 minute window
            max_requests_per_window: 1000,
            burst_allowance: 100,
            enable_adaptive: true,
            per_client_limit: Some(100),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Time window for failure counting
    pub failure_window: Duration,
    /// Circuit open duration before trying again
    pub open_duration: Duration,
    /// Success threshold to close circuit
    pub success_threshold: u32,
    /// Enable gradual recovery
    pub enable_gradual_recovery: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 10,
            failure_window: Duration::from_secs(60),
            open_duration: Duration::from_secs(30),
            success_threshold: 5,
            enable_gradual_recovery: true,
        }
    }
}

/// Resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub active_requests: u32,
    pub requests_per_second: f64,
    pub open_file_descriptors: u32,
    pub network_bandwidth_mbps: f64,
    pub disk_io_mbps: f64,
    pub violations: Vec<ResourceViolation>,
}

/// Resource violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceViolation {
    pub resource: String,
    pub current_value: f64,
    pub limit_value: f64,
    pub severity: ViolationSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action_taken: String,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationSeverity {
    Warning,
    Critical,
    Emergency,
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Rejecting requests
    HalfOpen, // Testing recovery
}

/// Rate limiter state for a client
#[derive(Debug, Clone)]
pub struct RateLimiterState {
    pub request_count: u32,
    pub window_start: Instant,
    pub last_request: Instant,
    pub burst_used: u32,
}

/// Circuit breaker state tracking
#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure: Option<Instant>,
    pub last_success: Option<Instant>,
    pub state_changed_at: Instant,
}

/// Resource limiter and enforcement engine
pub struct ResourceLimiter {
    limits: ResourceLimits,
    rate_limit_config: RateLimitConfig,
    circuit_breaker_config: CircuitBreakerConfig,
    session_id: Uuid,

    // Concurrency controls
    request_semaphore: Arc<Semaphore>,

    // Rate limiting state
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiterState>>>,
    global_rate_limiter: Arc<RwLock<RateLimiterState>>,

    // Circuit breaker state
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,

    // Resource monitoring
    current_usage: Arc<RwLock<ResourceUsage>>,
    violation_history: Arc<RwLock<Vec<ResourceViolation>>>,

    // Enforcement state
    is_enforcing: Arc<RwLock<bool>>,
}

impl ResourceLimiter {
    /// Create a new resource limiter
    pub fn new(targets: PerformanceTargets) -> Result<Self> {
        let limits = ResourceLimits {
            max_memory_mb: targets.max_memory_mb as f64,
            max_cpu_percent: 80.0, // Default CPU limit
            max_concurrent_requests: 100,
            max_requests_per_second: targets.min_throughput_pps * 2.0, // Allow 2x target throughput
            max_request_duration: Duration::from_millis(targets.p95_latency_ms * 2), // 2x target latency
            max_file_descriptors: 1000,
            max_network_bandwidth_mbps: 1000.0,
            max_disk_io_mbps: 500.0,
        };

        Self::with_config(
            limits,
            RateLimitConfig::default(),
            CircuitBreakerConfig::default(),
        )
    }

    /// Create a new resource limiter with custom configuration
    pub fn with_config(
        limits: ResourceLimits,
        rate_limit_config: RateLimitConfig,
        circuit_breaker_config: CircuitBreakerConfig,
    ) -> Result<Self> {
        let session_id = Uuid::new_v4();

        info!(
            session_id = %session_id,
            "Creating resource limiter with limits: {:?}",
            limits
        );

        let request_semaphore = Arc::new(Semaphore::new(limits.max_concurrent_requests as usize));

        let initial_usage = ResourceUsage {
            timestamp: chrono::Utc::now(),
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            active_requests: 0,
            requests_per_second: 0.0,
            open_file_descriptors: 0,
            network_bandwidth_mbps: 0.0,
            disk_io_mbps: 0.0,
            violations: Vec::new(),
        };

        Ok(Self {
            limits,
            rate_limit_config,
            circuit_breaker_config,
            session_id,
            request_semaphore,
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            global_rate_limiter: Arc::new(RwLock::new(RateLimiterState {
                request_count: 0,
                window_start: Instant::now(),
                last_request: Instant::now(),
                burst_used: 0,
            })),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            current_usage: Arc::new(RwLock::new(initial_usage)),
            violation_history: Arc::new(RwLock::new(Vec::new())),
            is_enforcing: Arc::new(RwLock::new(false)),
        })
    }

    /// Start resource enforcement
    pub async fn start_enforcement(&mut self) -> Result<()> {
        let mut is_enforcing = self.is_enforcing.write().await;
        if *is_enforcing {
            warn!(session_id = %self.session_id, "Resource enforcement already started");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Starting resource enforcement");
        *is_enforcing = true;

        // Start monitoring tasks
        self.start_monitoring_tasks().await?;

        info!(session_id = %self.session_id, "Resource enforcement started successfully");
        Ok(())
    }

    /// Stop resource enforcement
    pub async fn stop_enforcement(&mut self) -> Result<()> {
        let mut is_enforcing = self.is_enforcing.write().await;
        if !*is_enforcing {
            warn!(session_id = %self.session_id, "Resource enforcement not running");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Stopping resource enforcement");
        *is_enforcing = false;

        info!(session_id = %self.session_id, "Resource enforcement stopped successfully");
        Ok(())
    }

    /// Acquire request permit (blocks if limit reached)
    pub async fn acquire_request_permit(&self) -> Result<RequestPermit> {
        // Check rate limits first
        self.check_rate_limits(None).await?;

        // Check circuit breaker
        self.check_circuit_breaker("global").await?;

        // Acquire semaphore permit
        let permit = self
            .request_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| {
                PerformanceError::ResourceLimitExceeded(
                    "Max concurrent requests reached".to_string(),
                )
            })?;

        Ok(RequestPermit {
            _permit: permit,
            session_id: self.session_id,
            acquired_at: Instant::now(),
        })
    }

    /// Check rate limits for a client
    pub async fn check_rate_limits(&self, client_id: Option<&str>) -> Result<()> {
        if !self.rate_limit_config.enabled {
            return Ok(());
        }

        // Check global rate limit
        self.check_global_rate_limit().await?;

        // Check per-client rate limit if specified
        if let Some(client) = client_id {
            if let Some(per_client_limit) = self.rate_limit_config.per_client_limit {
                self.check_client_rate_limit(client, per_client_limit)
                    .await?;
            }
        }

        Ok(())
    }

    /// Check circuit breaker state
    pub async fn check_circuit_breaker(&self, service: &str) -> Result<()> {
        if !self.circuit_breaker_config.enabled {
            return Ok(());
        }

        let breakers = self.circuit_breakers.read().await;
        if let Some(breaker) = breakers.get(service) {
            match breaker.state {
                CircuitState::Open => {
                    // Check if we should transition to half-open
                    if breaker.state_changed_at.elapsed()
                        >= self.circuit_breaker_config.open_duration
                    {
                        drop(breakers);
                        self.transition_circuit_breaker(service, CircuitState::HalfOpen)
                            .await?;
                    } else {
                        return Err(PerformanceError::ResourceLimitExceeded(format!(
                            "Circuit breaker open for service: {}",
                            service
                        )));
                    }
                }
                CircuitState::HalfOpen => {
                    // Allow limited requests to test recovery
                    debug!(
                        session_id = %self.session_id,
                        service = service,
                        "Circuit breaker in half-open state, allowing test request"
                    );
                }
                CircuitState::Closed => {
                    // Normal operation
                }
            }
        }

        Ok(())
    }

    /// Record request success for circuit breaker
    pub async fn record_success(&self, service: &str) -> Result<()> {
        if !self.circuit_breaker_config.enabled {
            return Ok(());
        }

        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers
            .entry(service.to_string())
            .or_insert_with(|| CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                last_success: Some(Instant::now()),
                state_changed_at: Instant::now(),
            });

        breaker.last_success = Some(Instant::now());
        breaker.success_count += 1;

        // Check for state transitions
        match breaker.state {
            CircuitState::HalfOpen => {
                if breaker.success_count >= self.circuit_breaker_config.success_threshold {
                    breaker.state = CircuitState::Closed;
                    breaker.failure_count = 0;
                    breaker.success_count = 0;
                    breaker.state_changed_at = Instant::now();

                    info!(
                        session_id = %self.session_id,
                        service = service,
                        "Circuit breaker closed after successful recovery"
                    );
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                if breaker.failure_count > 0 {
                    breaker.failure_count = 0;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Record request failure for circuit breaker
    pub async fn record_failure(&self, service: &str) -> Result<()> {
        if !self.circuit_breaker_config.enabled {
            return Ok(());
        }

        let mut breakers = self.circuit_breakers.write().await;
        let breaker = breakers
            .entry(service.to_string())
            .or_insert_with(|| CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: Some(Instant::now()),
                last_success: None,
                state_changed_at: Instant::now(),
            });

        breaker.last_failure = Some(Instant::now());
        breaker.failure_count += 1;

        // Check for state transitions
        if (breaker.state == CircuitState::Closed || breaker.state == CircuitState::HalfOpen)
            && breaker.failure_count >= self.circuit_breaker_config.failure_threshold
        {
            breaker.state = CircuitState::Open;
            breaker.state_changed_at = Instant::now();
            breaker.success_count = 0; // Reset success count

            warn!(
                session_id = %self.session_id,
                service = service,
                failure_count = breaker.failure_count,
                "Circuit breaker opened due to failures"
            );
        }

        Ok(())
    }

    /// Get current resource usage
    pub async fn get_current_usage(&self) -> Result<ResourceUsage> {
        let usage = self.current_usage.read().await;
        Ok(usage.clone())
    }

    /// Get violation history
    pub async fn get_violation_history(&self) -> Result<Vec<ResourceViolation>> {
        let history = self.violation_history.read().await;
        Ok(history.clone())
    }

    /// Update resource usage and check limits
    pub async fn update_resource_usage(&self, usage: ResourceUsage) -> Result<()> {
        let violations = self.check_resource_limits(&usage).await?;

        let mut current_usage = self.current_usage.write().await;
        current_usage.memory_usage_mb = usage.memory_usage_mb;
        current_usage.cpu_usage_percent = usage.cpu_usage_percent;
        current_usage.active_requests = usage.active_requests;
        current_usage.requests_per_second = usage.requests_per_second;
        current_usage.open_file_descriptors = usage.open_file_descriptors;
        current_usage.network_bandwidth_mbps = usage.network_bandwidth_mbps;
        current_usage.disk_io_mbps = usage.disk_io_mbps;
        current_usage.violations = violations.clone();
        current_usage.timestamp = chrono::Utc::now();

        // Record violations
        if !violations.is_empty() {
            let mut history = self.violation_history.write().await;
            history.extend(violations);

            // Keep only recent violations (last 24 hours)
            let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
            history.retain(|v| v.timestamp >= cutoff);
        }

        Ok(())
    }

    /// Start background monitoring tasks
    async fn start_monitoring_tasks(&self) -> Result<()> {
        let session_id = self.session_id;
        let is_enforcing = Arc::clone(&self.is_enforcing);
        let current_usage = Arc::clone(&self.current_usage);
        let limits = self.limits.clone();

        // Resource monitoring task
        tokio::spawn(async move {
            debug!(session_id = %session_id, "Starting resource monitoring task");

            while *is_enforcing.read().await {
                // Collect current resource usage
                if let Ok(usage) = Self::collect_resource_usage_impl(&limits).await {
                    let mut current = current_usage.write().await;
                    *current = usage;
                }

                tokio::time::sleep(Duration::from_secs(5)).await;
            }

            debug!(session_id = %session_id, "Resource monitoring task stopped");
        });

        Ok(())
    }

    /// Check global rate limit
    async fn check_global_rate_limit(&self) -> Result<()> {
        let mut limiter = self.global_rate_limiter.write().await;
        let now = Instant::now();

        // Reset window if needed
        if now.duration_since(limiter.window_start) >= self.rate_limit_config.window_duration {
            limiter.request_count = 0;
            limiter.window_start = now;
            limiter.burst_used = 0;
        }

        // Check if we're within limits
        let max_requests =
            self.rate_limit_config.max_requests_per_window + self.rate_limit_config.burst_allowance;

        if limiter.request_count >= max_requests {
            return Err(PerformanceError::ResourceLimitExceeded(
                "Global rate limit exceeded".to_string(),
            ));
        }

        limiter.request_count += 1;
        limiter.last_request = now;

        if limiter.request_count > self.rate_limit_config.max_requests_per_window {
            limiter.burst_used += 1;
        }

        Ok(())
    }

    /// Check per-client rate limit
    async fn check_client_rate_limit(&self, client_id: &str, limit: u32) -> Result<()> {
        let mut limiters = self.rate_limiters.write().await;
        let now = Instant::now();

        let limiter = limiters
            .entry(client_id.to_string())
            .or_insert_with(|| RateLimiterState {
                request_count: 0,
                window_start: now,
                last_request: now,
                burst_used: 0,
            });

        // Reset window if needed
        if now.duration_since(limiter.window_start) >= self.rate_limit_config.window_duration {
            limiter.request_count = 0;
            limiter.window_start = now;
            limiter.burst_used = 0;
        }

        // Check if client is within limits
        if limiter.request_count >= limit {
            return Err(PerformanceError::ResourceLimitExceeded(format!(
                "Rate limit exceeded for client: {}",
                client_id
            )));
        }

        limiter.request_count += 1;
        limiter.last_request = now;

        Ok(())
    }

    /// Transition circuit breaker state
    async fn transition_circuit_breaker(
        &self,
        service: &str,
        new_state: CircuitState,
    ) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().await;
        if let Some(breaker) = breakers.get_mut(service) {
            breaker.state = new_state.clone();
            breaker.state_changed_at = Instant::now();

            info!(
                session_id = %self.session_id,
                service = service,
                new_state = format!("{:?}", new_state),
                "Circuit breaker state transition"
            );
        }

        Ok(())
    }

    /// Check resource limits and generate violations
    async fn check_resource_limits(&self, usage: &ResourceUsage) -> Result<Vec<ResourceViolation>> {
        let mut violations = Vec::new();

        // Memory check
        if usage.memory_usage_mb > self.limits.max_memory_mb {
            violations.push(ResourceViolation {
                resource: "memory".to_string(),
                current_value: usage.memory_usage_mb,
                limit_value: self.limits.max_memory_mb,
                severity: if usage.memory_usage_mb > self.limits.max_memory_mb * 1.2 {
                    ViolationSeverity::Emergency
                } else if usage.memory_usage_mb > self.limits.max_memory_mb * 1.1 {
                    ViolationSeverity::Critical
                } else {
                    ViolationSeverity::Warning
                },
                timestamp: chrono::Utc::now(),
                action_taken: "Memory limit exceeded".to_string(),
            });
        }

        // CPU check
        if usage.cpu_usage_percent > self.limits.max_cpu_percent {
            violations.push(ResourceViolation {
                resource: "cpu".to_string(),
                current_value: usage.cpu_usage_percent,
                limit_value: self.limits.max_cpu_percent,
                severity: if usage.cpu_usage_percent > 95.0 {
                    ViolationSeverity::Emergency
                } else if usage.cpu_usage_percent > 90.0 {
                    ViolationSeverity::Critical
                } else {
                    ViolationSeverity::Warning
                },
                timestamp: chrono::Utc::now(),
                action_taken: "CPU limit exceeded".to_string(),
            });
        }

        // Request rate check
        if usage.requests_per_second > self.limits.max_requests_per_second {
            violations.push(ResourceViolation {
                resource: "request_rate".to_string(),
                current_value: usage.requests_per_second,
                limit_value: self.limits.max_requests_per_second,
                severity: ViolationSeverity::Warning,
                timestamp: chrono::Utc::now(),
                action_taken: "Request rate limit exceeded".to_string(),
            });
        }

        Ok(violations)
    }

    /// Collect current resource usage (static implementation)
    async fn collect_resource_usage_impl(_limits: &ResourceLimits) -> Result<ResourceUsage> {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll simulate realistic values

        let memory_usage_mb = 200.0 + (rand::random::<f64>() * 150.0);
        let cpu_usage_percent = 30.0 + (rand::random::<f64>() * 40.0);
        let active_requests = rand::random::<u32>() % 50;
        let requests_per_second = 50.0 + (rand::random::<f64>() * 100.0);
        let open_file_descriptors = 100 + (rand::random::<u32>() % 200);
        let network_bandwidth_mbps = rand::random::<f64>() * 100.0;
        let disk_io_mbps = rand::random::<f64>() * 50.0;

        Ok(ResourceUsage {
            timestamp: chrono::Utc::now(),
            memory_usage_mb,
            cpu_usage_percent,
            active_requests,
            requests_per_second,
            open_file_descriptors,
            network_bandwidth_mbps,
            disk_io_mbps,
            violations: Vec::new(),
        })
    }
}

/// Request permit that enforces resource limits
pub struct RequestPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    session_id: Uuid,
    acquired_at: Instant,
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        let duration = self.acquired_at.elapsed();
        debug!(
            session_id = %self.session_id,
            duration_ms = duration.as_millis(),
            "Request permit released"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_limiter_creation() {
        let targets = crate::PerformanceTargets::default();
        let limiter = ResourceLimiter::new(targets).unwrap();
        assert!(!limiter.session_id.is_nil());
    }

    #[tokio::test]
    async fn test_request_permit_acquisition() {
        let targets = crate::PerformanceTargets::default();
        let limiter = ResourceLimiter::new(targets).unwrap();

        let permit = limiter.acquire_request_permit().await.unwrap();
        assert!(permit.acquired_at.elapsed() < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let targets = crate::PerformanceTargets::default();
        let mut limiter = ResourceLimiter::new(targets).unwrap();

        // Set very low rate limit for testing
        limiter.rate_limit_config.max_requests_per_window = 2;
        limiter.rate_limit_config.burst_allowance = 0;

        // First two requests should succeed
        assert!(limiter.check_rate_limits(None).await.is_ok());
        assert!(limiter.check_rate_limits(None).await.is_ok());

        // Third request should fail
        assert!(limiter.check_rate_limits(None).await.is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let targets = crate::PerformanceTargets::default();
        let mut limiter = ResourceLimiter::new(targets).unwrap();

        // Set low failure threshold for testing
        limiter.circuit_breaker_config.failure_threshold = 2;

        let service = "test_service";

        // Record failures to open circuit
        limiter.record_failure(service).await.unwrap();
        limiter.record_failure(service).await.unwrap();

        // Circuit should now be open
        assert!(limiter.check_circuit_breaker(service).await.is_err());
    }

    #[tokio::test]
    async fn test_resource_usage_update() {
        let targets = crate::PerformanceTargets::default();
        let limiter = ResourceLimiter::new(targets).unwrap();

        let usage = ResourceUsage {
            timestamp: chrono::Utc::now(),
            memory_usage_mb: 300.0,
            cpu_usage_percent: 50.0,
            active_requests: 10,
            requests_per_second: 100.0,
            open_file_descriptors: 150,
            network_bandwidth_mbps: 50.0,
            disk_io_mbps: 25.0,
            violations: Vec::new(),
        };

        limiter.update_resource_usage(usage.clone()).await.unwrap();

        let current = limiter.get_current_usage().await.unwrap();
        assert_eq!(current.memory_usage_mb, usage.memory_usage_mb);
        assert_eq!(current.cpu_usage_percent, usage.cpu_usage_percent);
    }
}
