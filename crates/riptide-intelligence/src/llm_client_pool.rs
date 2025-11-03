//! LLM Client Pool for efficient resource management
//!
//! This module provides a connection pool for LLM providers with:
//! - Semaphore-based concurrency control
//! - Circuit breaker integration for fault tolerance
//! - Automatic failover support
//! - Connection lifecycle management
//! - Health monitoring and metrics
//! - Timeout and retry handling

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

use crate::{
    CircuitBreaker, CircuitBreakerConfig, CompletionRequest, CompletionResponse, FailoverManager,
    IntelligenceError, LlmProvider, LlmRegistry,
};

/// Configuration for the LLM client pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmClientPoolConfig {
    /// Maximum number of concurrent LLM requests
    pub max_concurrent_requests: usize,
    /// Maximum number of pooled connections per provider
    pub max_connections_per_provider: usize,
    /// Connection idle timeout (after which connection is closed)
    pub connection_idle_timeout: Duration,
    /// Request timeout for LLM calls
    pub request_timeout: Duration,
    /// Enable circuit breaker for fault tolerance
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Enable automatic failover
    pub enable_failover: bool,
    /// Maximum retry attempts per request
    pub max_retry_attempts: u32,
    /// Initial backoff duration for retries
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for LlmClientPoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            max_connections_per_provider: 5,
            connection_idle_timeout: Duration::from_secs(300), // 5 minutes
            request_timeout: Duration::from_secs(30),
            enable_circuit_breaker: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
            enable_failover: true,
            max_retry_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// Statistics for monitoring pool performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmClientPoolStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_connections: usize,
    pub available_permits: usize,
    pub max_permits: usize,
    pub circuit_breaker_trips: u64,
    pub failover_count: u64,
    pub avg_request_duration_ms: f64,
    pub retry_count: u64,
}

impl LlmClientPoolStats {
    fn new(max_permits: usize) -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_connections: 0,
            available_permits: max_permits,
            max_permits,
            circuit_breaker_trips: 0,
            failover_count: 0,
            avg_request_duration_ms: 0.0,
            retry_count: 0,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}

/// Pooled LLM client wrapper
#[derive(Clone)]
struct PooledLlmClient {
    provider: Arc<dyn LlmProvider>,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    last_used: Instant,
    request_count: u64,
}

impl PooledLlmClient {
    fn new(
        provider: Arc<dyn LlmProvider>,
        enable_circuit_breaker: bool,
        config: &CircuitBreakerConfig,
    ) -> Self {
        let circuit_breaker = if enable_circuit_breaker {
            Some(Arc::new(CircuitBreaker::with_config(
                provider.clone(),
                config.clone(),
            )))
        } else {
            None
        };

        Self {
            provider,
            circuit_breaker,
            last_used: Instant::now(),
            request_count: 0,
        }
    }

    fn update_usage(&mut self) {
        self.last_used = Instant::now();
        self.request_count += 1;
    }

    fn is_idle(&self, idle_timeout: Duration) -> bool {
        self.last_used.elapsed() > idle_timeout
    }

    async fn execute(&mut self, request: CompletionRequest) -> crate::Result<CompletionResponse> {
        self.update_usage();

        if let Some(ref circuit_breaker) = self.circuit_breaker {
            circuit_breaker.complete(request).await
        } else {
            self.provider.complete(request).await
        }
    }

    fn get_provider(&self) -> &Arc<dyn LlmProvider> {
        &self.provider
    }
}

/// LLM Client Pool for managing LLM provider connections
pub struct LlmClientPool {
    config: LlmClientPoolConfig,
    registry: Arc<LlmRegistry>,
    failover_manager: Option<Arc<FailoverManager>>,
    clients: Arc<RwLock<HashMap<String, Vec<PooledLlmClient>>>>,
    semaphore: Arc<Semaphore>,
    stats: Arc<RwLock<LlmClientPoolStats>>,
    shutdown: Arc<RwLock<bool>>,
}

impl LlmClientPool {
    /// Create a new LLM client pool
    pub fn new(config: LlmClientPoolConfig, registry: Arc<LlmRegistry>) -> Self {
        let max_permits = config.max_concurrent_requests;

        Self {
            semaphore: Arc::new(Semaphore::new(max_permits)),
            stats: Arc::new(RwLock::new(LlmClientPoolStats::new(max_permits))),
            config,
            registry,
            failover_manager: None,
            clients: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Add failover manager for automatic provider failover
    pub fn with_failover(mut self, failover_manager: Arc<FailoverManager>) -> Self {
        self.failover_manager = Some(failover_manager);
        self
    }

    /// Start the pool and initialize background tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting LLM client pool");

        // Spawn health check task if enabled
        if self.config.health_check_interval > Duration::ZERO {
            let pool = self.clone();
            tokio::spawn(async move {
                pool.health_check_loop().await;
            });
        }

        // Spawn idle connection cleanup task
        let pool = self.clone();
        tokio::spawn(async move {
            pool.cleanup_loop().await;
        });

        Ok(())
    }

    /// Stop the pool and cleanup resources
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping LLM client pool");

        let mut shutdown = self.shutdown.write().await;
        *shutdown = true;

        // Wait for active requests to complete
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Clear all clients
        let mut clients = self.clients.write().await;
        clients.clear();

        info!("LLM client pool stopped");
        Ok(())
    }

    /// Acquire a client from the pool for the specified provider
    async fn acquire_client(&self, provider_name: &str) -> Result<PooledLlmClient> {
        let mut clients = self.clients.write().await;

        // Check if we have an available client for this provider
        if let Some(provider_clients) = clients.get_mut(provider_name) {
            if let Some(client) = provider_clients.pop() {
                // Check if client is still healthy (not idle too long)
                if !client.is_idle(self.config.connection_idle_timeout) {
                    debug!("Reusing existing client for provider: {}", provider_name);
                    return Ok(client);
                }
                debug!("Client for {} is idle, creating new one", provider_name);
            }
        }

        // No available client, create a new one
        debug!("Creating new client for provider: {}", provider_name);

        let provider = self
            .registry
            .get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider {} not found in registry", provider_name))?;

        Ok(PooledLlmClient::new(
            provider,
            self.config.enable_circuit_breaker,
            &self.config.circuit_breaker_config,
        ))
    }

    /// Return a client to the pool
    async fn return_client(&self, provider_name: &str, client: PooledLlmClient) {
        let mut clients = self.clients.write().await;

        let provider_clients = clients
            .entry(provider_name.to_string())
            .or_insert_with(Vec::new);

        // Only return to pool if we haven't exceeded max connections
        if provider_clients.len() < self.config.max_connections_per_provider {
            provider_clients.push(client);
            debug!("Client returned to pool for provider: {}", provider_name);
        } else {
            debug!(
                "Pool full for provider {}, discarding client",
                provider_name
            );
        }
    }

    /// Execute a completion request with pool management
    pub async fn complete(
        &self,
        request: CompletionRequest,
        provider_name: &str,
    ) -> crate::Result<CompletionResponse> {
        // Check if pool is shutting down
        if *self.shutdown.read().await {
            return Err(IntelligenceError::Provider(
                "Pool is shutting down".to_string(),
            ));
        }

        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await.map_err(|_| {
            IntelligenceError::Provider("Failed to acquire semaphore permit".to_string())
        })?;

        let start_time = Instant::now();

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.available_permits = self.semaphore.available_permits();
            stats.active_connections =
                self.config.max_concurrent_requests - stats.available_permits;
        }

        // Execute with retry logic
        let result = self.execute_with_retry(request, provider_name).await;

        // Update stats based on result
        let duration = start_time.elapsed();
        {
            let mut stats = self.stats.write().await;

            match &result {
                Ok(_) => stats.successful_requests += 1,
                Err(_) => stats.failed_requests += 1,
            }

            // Update average duration
            let duration_ms = duration.as_millis() as f64;
            if stats.total_requests == 1 {
                stats.avg_request_duration_ms = duration_ms;
            } else {
                stats.avg_request_duration_ms = (stats.avg_request_duration_ms
                    * (stats.total_requests - 1) as f64
                    + duration_ms)
                    / stats.total_requests as f64;
            }

            stats.available_permits = self.semaphore.available_permits();
        }

        result
    }

    /// Execute request with retry logic and exponential backoff
    async fn execute_with_retry(
        &self,
        request: CompletionRequest,
        provider_name: &str,
    ) -> crate::Result<CompletionResponse> {
        let mut backoff = self.config.initial_backoff;
        let mut last_error: Option<IntelligenceError> = None;

        for attempt in 0..self.config.max_retry_attempts {
            // Acquire client
            let mut client = match self.acquire_client(provider_name).await {
                Ok(client) => client,
                Err(e) => {
                    if self.config.enable_failover {
                        if let Some(failover_manager) = &self.failover_manager {
                            debug!(
                                "Provider {} unavailable, attempting failover",
                                provider_name
                            );

                            let mut stats = self.stats.write().await;
                            stats.failover_count += 1;
                            drop(stats);

                            return failover_manager.complete_with_failover(request).await;
                        }
                    }
                    return Err(IntelligenceError::Provider(e.to_string()));
                }
            };

            // Execute request with timeout
            let result =
                tokio::time::timeout(self.config.request_timeout, client.execute(request.clone()))
                    .await;

            // Return client to pool
            self.return_client(provider_name, client).await;

            match result {
                Ok(Ok(response)) => {
                    if attempt > 0 {
                        let mut stats = self.stats.write().await;
                        stats.retry_count += attempt as u64;
                    }
                    return Ok(response);
                }
                Ok(Err(e)) => {
                    last_error = Some(e.clone());

                    // Handle circuit breaker open error
                    if matches!(e, IntelligenceError::CircuitOpen { .. }) {
                        let mut stats = self.stats.write().await;
                        stats.circuit_breaker_trips += 1;
                        drop(stats);

                        if self.config.enable_failover {
                            if let Some(failover_manager) = &self.failover_manager {
                                warn!(
                                    "Circuit breaker open for {}, attempting failover",
                                    provider_name
                                );
                                return failover_manager.complete_with_failover(request).await;
                            }
                        }
                        return Err(e);
                    }

                    // Retry for retryable errors
                    if attempt < self.config.max_retry_attempts.saturating_sub(1) {
                        match &e {
                            IntelligenceError::Network(_)
                            | IntelligenceError::Timeout { .. }
                            | IntelligenceError::RateLimit { .. } => {
                                warn!(
                                    "Request failed (attempt {}): {}, retrying in {:?}",
                                    attempt.saturating_add(1),
                                    e,
                                    backoff
                                );
                                tokio::time::sleep(backoff).await;
                                backoff = std::cmp::min(
                                    Duration::from_secs_f64(
                                        backoff.as_secs_f64() * self.config.backoff_multiplier,
                                    ),
                                    self.config.max_backoff,
                                );
                                continue;
                            }
                            _ => return Err(e),
                        }
                    }
                }
                Err(_) => {
                    // Timeout occurred
                    let timeout_ms =
                        u64::try_from(self.config.request_timeout.as_millis()).unwrap_or(u64::MAX);
                    last_error = Some(IntelligenceError::Timeout { timeout_ms });

                    if attempt < self.config.max_retry_attempts.saturating_sub(1) {
                        warn!(
                            "Request timeout (attempt {}), retrying in {:?}",
                            attempt.saturating_add(1),
                            backoff
                        );
                        tokio::time::sleep(backoff).await;
                        backoff = std::cmp::min(
                            Duration::from_secs_f64(
                                backoff.as_secs_f64() * self.config.backoff_multiplier,
                            ),
                            self.config.max_backoff,
                        );
                        continue;
                    }
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap_or_else(|| {
            IntelligenceError::Provider("Request failed after all retries".to_string())
        }))
    }

    /// Get pool statistics
    pub async fn stats(&self) -> LlmClientPoolStats {
        self.stats.read().await.clone()
    }

    /// Health check loop
    async fn health_check_loop(&self) {
        loop {
            if *self.shutdown.read().await {
                break;
            }

            tokio::time::sleep(self.config.health_check_interval).await;

            // Perform health checks on all providers
            let clients = self.clients.read().await;
            for (provider_name, provider_clients) in clients.iter() {
                if let Some(client) = provider_clients.first() {
                    match client.get_provider().health_check().await {
                        Ok(_) => {
                            debug!("Health check passed for provider: {}", provider_name);
                        }
                        Err(e) => {
                            warn!("Health check failed for provider {}: {}", provider_name, e);
                        }
                    }
                }
            }
        }
    }

    /// Cleanup idle connections loop
    async fn cleanup_loop(&self) {
        loop {
            if *self.shutdown.read().await {
                break;
            }

            // Check every minute
            tokio::time::sleep(Duration::from_secs(60)).await;

            let mut clients = self.clients.write().await;
            for (provider_name, provider_clients) in clients.iter_mut() {
                let initial_count = provider_clients.len();
                provider_clients
                    .retain(|client| !client.is_idle(self.config.connection_idle_timeout));
                let removed = initial_count - provider_clients.len();

                if removed > 0 {
                    debug!(
                        "Cleaned up {} idle clients for provider: {}",
                        removed, provider_name
                    );
                }
            }
        }
    }
}

impl Clone for LlmClientPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            registry: self.registry.clone(),
            failover_manager: self.failover_manager.clone(),
            clients: self.clients.clone(),
            semaphore: self.semaphore.clone(),
            stats: self.stats.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LlmRegistry;

    #[tokio::test]
    async fn test_pool_creation() {
        let config = LlmClientPoolConfig::default();
        let registry = Arc::new(LlmRegistry::new());
        let pool = LlmClientPool::new(config, registry);

        let stats = pool.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let config = LlmClientPoolConfig::default();
        let registry = Arc::new(LlmRegistry::new());
        let pool = LlmClientPool::new(config, registry);

        let stats = pool.stats().await;
        assert_eq!(stats.max_permits, 10);
        assert_eq!(stats.available_permits, 10);
    }
}
