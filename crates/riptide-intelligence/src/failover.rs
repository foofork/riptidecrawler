//! Automatic failover system for LLM providers
//!
//! This module provides:
//! - Automatic provider failover based on health status
//! - Configurable failover strategies
//! - Circuit breaker integration
//! - Load balancing during failover
//! - Recovery detection and automatic failback

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tracing::{info, warn, debug};

use crate::{
    LlmProvider, CompletionRequest, CompletionResponse, IntelligenceError, Result,
    health::HealthMonitor,
};

/// Failover strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverConfig {
    pub strategy: FailoverStrategy,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub failback_delay: Duration,
    pub health_check_threshold: u32,
    pub circuit_breaker_enabled: bool,
    pub load_balancing_enabled: bool,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            strategy: FailoverStrategy::RoundRobin,
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            failback_delay: Duration::from_secs(60),
            health_check_threshold: 3,
            circuit_breaker_enabled: true,
            load_balancing_enabled: true,
        }
    }
}

/// Available failover strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverStrategy {
    /// Try providers in order of priority
    Priority,
    /// Round-robin through available providers
    RoundRobin,
    /// Choose provider with lowest response time
    LeastLatency,
    /// Choose provider with lowest error rate
    LeastErrors,
    /// Choose provider with lowest cost
    LeastCost,
    /// Random selection from healthy providers
    Random,
    /// Weighted selection based on performance metrics
    Weighted,
}

/// Provider priority and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPriority {
    pub name: String,
    pub priority: u32,
    pub weight: f64,
    pub max_concurrent_requests: u32,
    pub enabled: bool,
}

/// Failover state for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub name: String,
    pub status: ProviderStatus,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub circuit_breaker_open_until: Option<chrono::DateTime<chrono::Utc>>,
    pub current_requests: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
}

/// Status of a provider in the failover system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderStatus {
    Available,
    Degraded,
    Unavailable,
    CircuitOpen,
    Recovering,
}

/// Failover decision result
#[derive(Debug, Clone)]
pub struct FailoverDecision {
    pub selected_provider: String,
    pub strategy_used: FailoverStrategy,
    pub fallback_providers: Vec<String>,
    pub decision_time: Instant,
    pub reason: String,
}

/// Failover events for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailoverEvent {
    ProviderFailover {
        from: String,
        to: String,
        reason: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ProviderRecovered {
        provider: String,
        downtime: Duration,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    CircuitBreakerTripped {
        provider: String,
        failure_count: u32,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    AllProvidersDown {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    FailoverDisabled {
        provider: String,
        reason: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Automatic failover manager
pub struct FailoverManager {
    providers: Arc<RwLock<HashMap<String, Arc<dyn LlmProvider>>>>,
    provider_priorities: Arc<RwLock<HashMap<String, ProviderPriority>>>,
    provider_states: Arc<RwLock<HashMap<String, ProviderState>>>,
    config: FailoverConfig,
    health_monitor: Arc<HealthMonitor>,
    event_tx: mpsc::UnboundedSender<FailoverEvent>,
    current_provider_index: Arc<RwLock<usize>>,
}

impl FailoverManager {
    /// Create a new failover manager
    pub fn new(
        config: FailoverConfig,
        health_monitor: Arc<HealthMonitor>,
    ) -> (Self, mpsc::UnboundedReceiver<FailoverEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let manager = Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            provider_priorities: Arc::new(RwLock::new(HashMap::new())),
            provider_states: Arc::new(RwLock::new(HashMap::new())),
            config,
            health_monitor,
            event_tx,
            current_provider_index: Arc::new(RwLock::new(0)),
        };

        (manager, event_rx)
    }

    /// Add a provider to the failover system
    pub async fn add_provider(
        &self,
        provider: Arc<dyn LlmProvider>,
        priority: ProviderPriority,
    ) -> Result<()> {
        let name = provider.name().to_string();

        // Add to providers map
        {
            let mut providers = self.providers.write().await;
            providers.insert(name.clone(), provider.clone());
        }

        // Set priority configuration
        {
            let mut priorities = self.provider_priorities.write().await;
            priorities.insert(name.clone(), priority);
        }

        // Initialize provider state
        let state = ProviderState {
            name: name.clone(),
            status: ProviderStatus::Available,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_used: None,
            circuit_breaker_open_until: None,
            current_requests: 0,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        };

        {
            let mut states = self.provider_states.write().await;
            states.insert(name.clone(), state);
        }

        // Add to health monitor
        self.health_monitor.add_provider(name.clone(), provider).await;

        info!("Added provider {} to failover system", name);
        Ok(())
    }

    /// Remove a provider from the failover system
    pub async fn remove_provider(&self, name: &str) -> Result<()> {
        {
            let mut providers = self.providers.write().await;
            providers.remove(name);
        }

        {
            let mut priorities = self.provider_priorities.write().await;
            priorities.remove(name);
        }

        {
            let mut states = self.provider_states.write().await;
            states.remove(name);
        }

        self.health_monitor.remove_provider(name).await;

        info!("Removed provider {} from failover system", name);
        Ok(())
    }

    /// Execute a completion request with automatic failover
    pub async fn complete_with_failover(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let decision = self.make_failover_decision().await?;

        debug!("Failover decision: using provider {} with strategy {:?}",
               decision.selected_provider, decision.strategy_used);

        // Try the selected provider first
        match self.try_provider(&decision.selected_provider, request.clone()).await {
            Ok(response) => {
                self.record_success(&decision.selected_provider).await;
                return Ok(response);
            }
            Err(e) => {
                warn!("Primary provider {} failed: {}", decision.selected_provider, e);
                self.record_failure(&decision.selected_provider).await;
            }
        }

        // Try fallback providers
        for fallback_provider in &decision.fallback_providers {
            if self.config.max_retries == 0 {
                break;
            }

            debug!("Trying fallback provider: {}", fallback_provider);

            tokio::time::sleep(self.config.retry_delay).await;

            match self.try_provider(fallback_provider, request.clone()).await {
                Ok(response) => {
                    self.record_success(fallback_provider).await;

                    // Emit failover event
                    let event = FailoverEvent::ProviderFailover {
                        from: decision.selected_provider.clone(),
                        to: fallback_provider.clone(),
                        reason: "Primary provider failed".to_string(),
                        timestamp: chrono::Utc::now(),
                    };
                    let _ = self.event_tx.send(event);

                    return Ok(response);
                }
                Err(e) => {
                    warn!("Fallback provider {} failed: {}", fallback_provider, e);
                    self.record_failure(fallback_provider).await;
                }
            }
        }

        // All providers failed
        let event = FailoverEvent::AllProvidersDown {
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_tx.send(event);

        Err(IntelligenceError::AllProvidersFailed)
    }

    /// Make a failover decision based on the configured strategy
    async fn make_failover_decision(&self) -> Result<FailoverDecision> {
        let available_providers = self.get_available_providers().await;

        if available_providers.is_empty() {
            return Err(IntelligenceError::AllProvidersFailed);
        }

        let selected_provider = match self.config.strategy {
            FailoverStrategy::Priority => {
                self.select_by_priority(&available_providers).await
            }
            FailoverStrategy::RoundRobin => {
                self.select_round_robin(&available_providers).await
            }
            FailoverStrategy::LeastLatency => {
                self.select_by_least_latency(&available_providers).await
            }
            FailoverStrategy::LeastErrors => {
                self.select_by_least_errors(&available_providers).await
            }
            FailoverStrategy::LeastCost => {
                self.select_by_least_cost(&available_providers).await
            }
            FailoverStrategy::Random => {
                self.select_random(&available_providers).await
            }
            FailoverStrategy::Weighted => {
                self.select_weighted(&available_providers).await
            }
        };

        let mut fallback_providers = available_providers;
        fallback_providers.retain(|p| p != &selected_provider);

        Ok(FailoverDecision {
            selected_provider,
            strategy_used: self.config.strategy.clone(),
            fallback_providers,
            decision_time: Instant::now(),
            reason: format!("Selected using {:?} strategy", self.config.strategy),
        })
    }

    /// Get list of available providers
    async fn get_available_providers(&self) -> Vec<String> {
        let states = self.provider_states.read().await;
        let priorities = self.provider_priorities.read().await;

        states
            .values()
            .filter(|state| {
                state.status == ProviderStatus::Available ||
                state.status == ProviderStatus::Degraded
            })
            .filter(|state| {
                priorities.get(&state.name)
                    .map(|p| p.enabled)
                    .unwrap_or(false)
            })
            .filter(|state| {
                // Check circuit breaker
                if let Some(open_until) = state.circuit_breaker_open_until {
                    chrono::Utc::now() > open_until
                } else {
                    true
                }
            })
            .map(|state| state.name.clone())
            .collect()
    }

    /// Select provider by priority
    async fn select_by_priority(&self, providers: &[String]) -> String {
        let priorities = self.provider_priorities.read().await;

        providers
            .iter()
            .min_by_key(|name| {
                priorities.get(*name).map(|p| p.priority).unwrap_or(999)
            })
            .unwrap_or(&providers[0])
            .clone()
    }

    /// Select provider using round-robin
    async fn select_round_robin(&self, providers: &[String]) -> String {
        let mut index = self.current_provider_index.write().await;
        let selected = providers[*index % providers.len()].clone();
        *index += 1;
        selected
    }

    /// Select provider with lowest latency
    async fn select_by_least_latency(&self, providers: &[String]) -> String {
        // This would use metrics from health monitor
        // For now, just return first provider
        providers[0].clone()
    }

    /// Select provider with lowest error rate
    async fn select_by_least_errors(&self, providers: &[String]) -> String {
        let states = self.provider_states.read().await;

        providers
            .iter()
            .min_by(|a, b| {
                let a_error_rate = states.get(*a)
                    .map(|s| s.failed_requests as f64 / (s.total_requests.max(1) as f64))
                    .unwrap_or(0.0);
                let b_error_rate = states.get(*b)
                    .map(|s| s.failed_requests as f64 / (s.total_requests.max(1) as f64))
                    .unwrap_or(0.0);
                a_error_rate.partial_cmp(&b_error_rate).unwrap()
            })
            .unwrap_or(&providers[0])
            .clone()
    }

    /// Select provider with lowest cost
    async fn select_by_least_cost(&self, providers: &[String]) -> String {
        // This would use cost estimation from providers
        // For now, just return first provider
        providers[0].clone()
    }

    /// Select random provider
    async fn select_random(&self, providers: &[String]) -> String {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        providers.choose(&mut rng).unwrap().clone()
    }

    /// Select provider using weighted selection
    async fn select_weighted(&self, providers: &[String]) -> String {
        let priorities = self.provider_priorities.read().await;

        // Simple weighted selection based on priority weights
        let total_weight: f64 = providers
            .iter()
            .map(|name| priorities.get(name).map(|p| p.weight).unwrap_or(1.0))
            .sum();

        if total_weight <= 0.0 {
            return providers[0].clone();
        }

        let mut rng = rand::thread_rng();
        let mut random_weight = rand::Rng::gen_range(&mut rng, 0.0..total_weight);

        for provider in providers {
            let weight = priorities.get(provider).map(|p| p.weight).unwrap_or(1.0);
            if random_weight <= weight {
                return provider.clone();
            }
            random_weight -= weight;
        }

        providers[0].clone()
    }

    /// Try to execute request with a specific provider
    async fn try_provider(
        &self,
        provider_name: &str,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let provider = {
            let providers = self.providers.read().await;
            providers.get(provider_name).cloned()
        };

        match provider {
            Some(provider) => {
                // Increment current requests counter
                {
                    let mut states = self.provider_states.write().await;
                    if let Some(state) = states.get_mut(provider_name) {
                        state.current_requests += 1;
                        state.total_requests += 1;
                        state.last_used = Some(chrono::Utc::now());
                    }
                }

                let result = provider.complete(request).await;

                // Decrement current requests counter
                {
                    let mut states = self.provider_states.write().await;
                    if let Some(state) = states.get_mut(provider_name) {
                        state.current_requests = state.current_requests.saturating_sub(1);
                    }
                }

                result
            }
            None => Err(IntelligenceError::Configuration(
                format!("Provider {} not found", provider_name)
            )),
        }
    }

    /// Record successful request
    async fn record_success(&self, provider_name: &str) {
        let mut states = self.provider_states.write().await;
        if let Some(state) = states.get_mut(provider_name) {
            state.successful_requests += 1;
            state.consecutive_successes += 1;
            state.consecutive_failures = 0;

            // Check if provider recovered
            if state.status != ProviderStatus::Available &&
               state.consecutive_successes >= self.config.health_check_threshold {

                let downtime = state.last_used
                    .map(|last| chrono::Utc::now() - last)
                    .unwrap_or_else(|| chrono::Duration::zero());

                state.status = ProviderStatus::Available;
                state.circuit_breaker_open_until = None;

                let event = FailoverEvent::ProviderRecovered {
                    provider: provider_name.to_string(),
                    downtime: Duration::from_secs(downtime.num_seconds() as u64),
                    timestamp: chrono::Utc::now(),
                };
                let _ = self.event_tx.send(event);

                info!("Provider {} recovered after {} seconds", provider_name, downtime.num_seconds());
            }
        }
    }

    /// Record failed request
    async fn record_failure(&self, provider_name: &str) {
        let mut states = self.provider_states.write().await;
        if let Some(state) = states.get_mut(provider_name) {
            state.failed_requests += 1;
            state.consecutive_failures += 1;
            state.consecutive_successes = 0;

            // Check if circuit breaker should be triggered
            if self.config.circuit_breaker_enabled &&
               state.consecutive_failures >= self.config.health_check_threshold {

                state.status = ProviderStatus::CircuitOpen;
                state.circuit_breaker_open_until = Some(
                    chrono::Utc::now() + chrono::Duration::from_std(self.config.failback_delay).unwrap()
                );

                let event = FailoverEvent::CircuitBreakerTripped {
                    provider: provider_name.to_string(),
                    failure_count: state.consecutive_failures,
                    timestamp: chrono::Utc::now(),
                };
                let _ = self.event_tx.send(event);

                warn!("Circuit breaker opened for provider {} after {} failures",
                     provider_name, state.consecutive_failures);
            }
        }
    }

    /// Get current provider states
    pub async fn get_provider_states(&self) -> HashMap<String, ProviderState> {
        self.provider_states.read().await.clone()
    }

    /// Get failover statistics
    pub async fn get_statistics(&self) -> FailoverStatistics {
        let states = self.provider_states.read().await;
        let total_providers = states.len();
        let available_providers = states.values()
            .filter(|s| s.status == ProviderStatus::Available)
            .count();
        let degraded_providers = states.values()
            .filter(|s| s.status == ProviderStatus::Degraded)
            .count();
        let unavailable_providers = states.values()
            .filter(|s| s.status == ProviderStatus::Unavailable || s.status == ProviderStatus::CircuitOpen)
            .count();

        let total_requests: u64 = states.values().map(|s| s.total_requests).sum();
        let total_failures: u64 = states.values().map(|s| s.failed_requests).sum();

        FailoverStatistics {
            total_providers,
            available_providers,
            degraded_providers,
            unavailable_providers,
            total_requests,
            total_failures,
            overall_error_rate: if total_requests > 0 {
                (total_failures as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Failover system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverStatistics {
    pub total_providers: usize,
    pub available_providers: usize,
    pub degraded_providers: usize,
    pub unavailable_providers: usize,
    pub total_requests: u64,
    pub total_failures: u64,
    pub overall_error_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock_provider::MockLlmProvider, health::HealthMonitorBuilder};

    #[tokio::test]
    async fn test_failover_manager_creation() {
        let health_monitor = Arc::new(HealthMonitorBuilder::new().build());
        let (manager, _rx) = FailoverManager::new(FailoverConfig::default(), health_monitor);

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_providers, 0);
    }

    #[tokio::test]
    async fn test_add_provider() {
        let health_monitor = Arc::new(HealthMonitorBuilder::new().build());
        let (manager, _rx) = FailoverManager::new(FailoverConfig::default(), health_monitor);

        let provider = Arc::new(MockLlmProvider::new());
        let priority = ProviderPriority {
            name: "test".to_string(),
            priority: 1,
            weight: 1.0,
            max_concurrent_requests: 10,
            enabled: true,
        };

        manager.add_provider(provider, priority).await.unwrap();

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_providers, 1);
        assert_eq!(stats.available_providers, 1);
    }
}