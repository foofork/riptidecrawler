//! Tenant isolation and resource management system
//!
//! This module provides comprehensive tenant isolation with:
//! - Per-tenant resource limits and quotas
//! - Request throttling and rate limiting
//! - Cost tracking and budget enforcement
//! - Multi-tenant request routing
//! - Tenant-specific provider preferences

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::interval;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{
    config::{TenantIsolationConfig, TenantLimits},
    CompletionRequest, CompletionResponse, IntelligenceError, Result,
};

/// Tenant isolation manager
pub struct TenantIsolationManager {
    config: TenantIsolationConfig,
    tenant_states: Arc<RwLock<HashMap<String, TenantState>>>,
    rate_limiters: Arc<RwLock<HashMap<String, TenantRateLimiter>>>,
    #[allow(dead_code)]
    tenant_providers: Arc<RwLock<HashMap<String, TenantProviderConfig>>>,
    resource_pools: Arc<RwLock<HashMap<String, Arc<Semaphore>>>>,
}

/// Current state of a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantState {
    pub tenant_id: String,
    pub current_requests: u32,
    pub requests_this_minute: u32,
    pub tokens_this_minute: u32,
    pub cost_this_hour: f64,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub last_request_time: Option<chrono::DateTime<chrono::Utc>>,
    pub status: TenantStatus,
    pub violations: Vec<TenantViolation>,
}

/// Tenant operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Throttled,
    Suspended,
    QuotaExceeded,
    BudgetExceeded,
}

/// Tenant limit violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantViolation {
    pub violation_type: ViolationType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub limit_value: f64,
    pub actual_value: f64,
    pub action_taken: ViolationAction,
}

/// Types of tenant violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    RequestRateLimit,
    TokenRateLimit,
    CostLimit,
    ConcurrentRequestLimit,
    ModelRestriction,
}

/// Actions taken for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationAction {
    Warning,
    Throttling,
    RequestRejection,
    TenantSuspension,
}

/// Per-tenant rate limiter
pub struct TenantRateLimiter {
    #[allow(dead_code)]
    tenant_id: String,
    limits: TenantLimits,
    request_timestamps: Vec<Instant>,
    token_count: u32,
    token_reset_time: Instant,
    cost_amount: f64,
    cost_reset_time: Instant,
}

impl TenantRateLimiter {
    pub fn new(tenant_id: String, limits: TenantLimits) -> Self {
        let now = Instant::now();
        Self {
            tenant_id,
            limits,
            request_timestamps: Vec::new(),
            token_count: 0,
            token_reset_time: now + Duration::from_secs(60),
            cost_amount: 0.0,
            cost_reset_time: now + Duration::from_secs(3600),
        }
    }

    /// Check if request is allowed under rate limits
    pub fn check_request_allowed(
        &mut self,
        estimated_tokens: u32,
        estimated_cost: f64,
    ) -> Result<()> {
        let now = Instant::now();

        // Clean old request timestamps (sliding window)
        self.request_timestamps
            .retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(60));

        // Check request rate limit
        if self.request_timestamps.len() >= self.limits.max_requests_per_minute as usize {
            return Err(IntelligenceError::RateLimit {
                retry_after_ms: 60000,
            });
        }

        // Reset token count if needed
        if now > self.token_reset_time {
            self.token_count = 0;
            self.token_reset_time = now + Duration::from_secs(60);
        }

        // Check token rate limit
        if self.token_count + estimated_tokens > self.limits.max_tokens_per_minute {
            return Err(IntelligenceError::RateLimit {
                retry_after_ms: (self.token_reset_time.duration_since(now).as_millis() as u64)
                    .max(1000),
            });
        }

        // Reset cost amount if needed
        if now > self.cost_reset_time {
            self.cost_amount = 0.0;
            self.cost_reset_time = now + Duration::from_secs(3600);
        }

        // Check cost limit
        if self.cost_amount + estimated_cost > self.limits.max_cost_per_hour {
            return Err(IntelligenceError::RateLimit {
                retry_after_ms: (self.cost_reset_time.duration_since(now).as_millis() as u64)
                    .max(60000),
            });
        }

        Ok(())
    }

    /// Record a successful request
    pub fn record_request(&mut self, tokens_used: u32, cost: f64) {
        self.request_timestamps.push(Instant::now());
        self.token_count += tokens_used;
        self.cost_amount += cost;
    }
}

/// Tenant-specific provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProviderConfig {
    pub tenant_id: String,
    pub preferred_providers: Vec<String>,
    pub blocked_providers: Vec<String>,
    pub allowed_models: Option<Vec<String>>,
    pub cost_optimization_enabled: bool,
    pub custom_routing_rules: Vec<RoutingRule>,
}

/// Custom routing rules for tenants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub condition: RoutingCondition,
    pub target_provider: String,
    pub priority: u32,
}

/// Conditions for routing rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    ModelPattern(String),
    TokenRange { min: u32, max: u32 },
    TimeWindow { start_hour: u8, end_hour: u8 },
    CostThreshold(f64),
}

impl TenantIsolationManager {
    /// Create a new tenant isolation manager
    pub fn new(config: TenantIsolationConfig) -> Self {
        Self {
            config,
            tenant_states: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            tenant_providers: Arc::new(RwLock::new(HashMap::new())),
            resource_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize a tenant with specific limits
    pub async fn initialize_tenant(
        &self,
        tenant_id: String,
        limits: Option<TenantLimits>,
    ) -> Result<()> {
        let tenant_limits = limits.unwrap_or_else(|| self.config.default_limits.clone());

        info!(
            "Initializing tenant: {} with limits: {:?}",
            tenant_id, tenant_limits
        );

        // Initialize tenant state
        let state = TenantState {
            tenant_id: tenant_id.clone(),
            current_requests: 0,
            requests_this_minute: 0,
            tokens_this_minute: 0,
            cost_this_hour: 0.0,
            total_requests: 0,
            total_tokens: 0,
            total_cost: 0.0,
            last_request_time: None,
            status: TenantStatus::Active,
            violations: Vec::new(),
        };

        {
            let mut states = self.tenant_states.write().await;
            states.insert(tenant_id.clone(), state);
        }

        // Initialize rate limiter
        let rate_limiter = TenantRateLimiter::new(tenant_id.clone(), tenant_limits.clone());
        {
            let mut limiters = self.rate_limiters.write().await;
            limiters.insert(tenant_id.clone(), rate_limiter);
        }

        // Initialize resource pool (semaphore for concurrent requests)
        let semaphore = Arc::new(Semaphore::new(
            tenant_limits.max_concurrent_requests as usize,
        ));
        {
            let mut pools = self.resource_pools.write().await;
            pools.insert(tenant_id.clone(), semaphore);
        }

        Ok(())
    }

    /// Check if a request is allowed for a tenant
    pub async fn check_request_allowed(
        &self,
        tenant_id: &str,
        request: &CompletionRequest,
        estimated_cost: f64,
    ) -> Result<RequestPermit> {
        // Get tenant state
        let tenant_state = {
            let states = self.tenant_states.read().await;
            states.get(tenant_id).cloned().ok_or_else(|| {
                IntelligenceError::Configuration(format!("Tenant {} not initialized", tenant_id))
            })?
        };

        // Check if tenant is suspended
        if tenant_state.status == TenantStatus::Suspended {
            return Err(IntelligenceError::Configuration(
                "Tenant is suspended".to_string(),
            ));
        }

        // Estimate token usage from request
        let estimated_tokens = self.estimate_token_usage(request);

        // Check rate limits
        {
            let mut limiters = self.rate_limiters.write().await;
            if let Some(limiter) = limiters.get_mut(tenant_id) {
                limiter.check_request_allowed(estimated_tokens, estimated_cost)?;
            }
        }

        // Check model restrictions
        if let Some(ref allowed_models) = self.get_tenant_limits(tenant_id).await?.allowed_models {
            if !allowed_models.contains(&request.model) {
                return Err(IntelligenceError::InvalidRequest(format!(
                    "Model {} not allowed for tenant {}",
                    request.model, tenant_id
                )));
            }
        }

        // Acquire concurrent request permit
        let permit = {
            let pools = self.resource_pools.read().await;
            if let Some(semaphore) = pools.get(tenant_id) {
                match semaphore.clone().try_acquire_owned() {
                    Ok(permit) => permit,
                    Err(_) => {
                        return Err(IntelligenceError::RateLimit {
                            retry_after_ms: 1000,
                        });
                    }
                }
            } else {
                return Err(IntelligenceError::Configuration(format!(
                    "No resource pool for tenant {}",
                    tenant_id
                )));
            }
        };

        // Update current requests counter
        {
            let mut states = self.tenant_states.write().await;
            if let Some(state) = states.get_mut(tenant_id) {
                state.current_requests += 1;
                state.last_request_time = Some(chrono::Utc::now());
            }
        }

        Ok(RequestPermit {
            tenant_id: tenant_id.to_string(),
            request_id: request.id,
            estimated_tokens,
            estimated_cost,
            _permit: permit,
        })
    }

    /// Record completion of a request
    pub async fn record_request_completion(
        &self,
        permit: RequestPermit,
        response: Option<&CompletionResponse>,
        actual_cost: f64,
    ) {
        let tenant_id = &permit.tenant_id;
        let actual_tokens = response.map(|r| r.usage.total_tokens).unwrap_or(0);

        // Update rate limiter
        {
            let mut limiters = self.rate_limiters.write().await;
            if let Some(limiter) = limiters.get_mut(tenant_id) {
                limiter.record_request(actual_tokens, actual_cost);
            }
        }

        // Update tenant state
        {
            let mut states = self.tenant_states.write().await;
            if let Some(state) = states.get_mut(tenant_id) {
                state.current_requests = state.current_requests.saturating_sub(1);
                state.total_requests += 1;
                state.total_tokens += actual_tokens as u64;
                state.total_cost += actual_cost;
            }
        }

        debug!(
            "Recorded request completion for tenant {} - tokens: {}, cost: {:.4}",
            tenant_id, actual_tokens, actual_cost
        );
    }

    /// Get preferred providers for a tenant
    pub async fn get_tenant_providers(&self, tenant_id: &str) -> Vec<String> {
        if let Some(providers) = self.config.tenant_provider_mapping.get(tenant_id) {
            providers.clone()
        } else {
            // Return all available providers if no specific mapping
            Vec::new()
        }
    }

    /// Update tenant limits
    pub async fn update_tenant_limits(
        &self,
        tenant_id: &str,
        new_limits: TenantLimits,
    ) -> Result<()> {
        // Update rate limiter
        {
            let mut limiters = self.rate_limiters.write().await;
            if let Some(limiter) = limiters.get_mut(tenant_id) {
                limiter.limits = new_limits.clone();
            }
        }

        // Update resource pool
        {
            let mut pools = self.resource_pools.write().await;
            let new_semaphore =
                Arc::new(Semaphore::new(new_limits.max_concurrent_requests as usize));
            pools.insert(tenant_id.to_string(), new_semaphore);
        }

        info!("Updated limits for tenant: {}", tenant_id);
        Ok(())
    }

    /// Get current tenant statistics
    pub async fn get_tenant_stats(&self, tenant_id: &str) -> Option<TenantState> {
        let states = self.tenant_states.read().await;
        states.get(tenant_id).cloned()
    }

    /// Get all tenant statistics
    pub async fn get_all_tenant_stats(&self) -> HashMap<String, TenantState> {
        self.tenant_states.read().await.clone()
    }

    /// Suspend a tenant
    pub async fn suspend_tenant(&self, tenant_id: &str, reason: &str) {
        let mut states = self.tenant_states.write().await;
        if let Some(state) = states.get_mut(tenant_id) {
            state.status = TenantStatus::Suspended;
            state.violations.push(TenantViolation {
                violation_type: ViolationType::RequestRateLimit, // Generic for suspension
                timestamp: chrono::Utc::now(),
                limit_value: 0.0,
                actual_value: 0.0,
                action_taken: ViolationAction::TenantSuspension,
            });
            warn!("Suspended tenant {} - reason: {}", tenant_id, reason);
        }
    }

    /// Resume a suspended tenant
    pub async fn resume_tenant(&self, tenant_id: &str) {
        let mut states = self.tenant_states.write().await;
        if let Some(state) = states.get_mut(tenant_id) {
            state.status = TenantStatus::Active;
            info!("Resumed tenant: {}", tenant_id);
        }
    }

    /// Estimate token usage from request
    fn estimate_token_usage(&self, request: &CompletionRequest) -> u32 {
        // Simple estimation based on message content length
        // In practice, this would use proper tokenization
        let total_chars: usize = request.messages.iter().map(|m| m.content.len()).sum();

        // Rough estimate: 4 chars per token, plus buffer
        ((total_chars / 4) as u32 + 100).min(request.max_tokens.unwrap_or(2048))
    }

    /// Get tenant limits
    async fn get_tenant_limits(&self, tenant_id: &str) -> Result<TenantLimits> {
        Ok(self
            .config
            .per_tenant_limits
            .get(tenant_id)
            .cloned()
            .unwrap_or_else(|| self.config.default_limits.clone()))
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(&self) {
        let states = Arc::clone(&self.tenant_states);
        let limiters = Arc::clone(&self.rate_limiters);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Clean up old violations and reset counters
                {
                    let mut states_guard = states.write().await;
                    let mut limiters_guard = limiters.write().await;

                    for (tenant_id, state) in states_guard.iter_mut() {
                        // Reset minute counters
                        state.requests_this_minute = 0;
                        state.tokens_this_minute = 0;

                        // Clean old violations (older than 24 hours)
                        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
                        state.violations.retain(|v| v.timestamp > cutoff);

                        // Clean rate limiter old requests
                        if let Some(limiter) = limiters_guard.get_mut(tenant_id) {
                            let now = Instant::now();
                            limiter.request_timestamps.retain(|&timestamp| {
                                now.duration_since(timestamp) < Duration::from_secs(60)
                            });
                        }
                    }
                }

                debug!("Completed tenant isolation cleanup cycle");
            }
        });
    }
}

/// Request permit for tenant isolation
pub struct RequestPermit {
    pub tenant_id: String,
    pub request_id: Uuid,
    pub estimated_tokens: u32,
    pub estimated_cost: f64,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        debug!("Request permit dropped for tenant: {}", self.tenant_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompletionRequest, Message};

    #[tokio::test]
    async fn test_tenant_isolation_manager() {
        let config = TenantIsolationConfig::default();
        let manager = TenantIsolationManager::new(config);

        let tenant_id = "test_tenant";
        let limits = TenantLimits {
            max_requests_per_minute: 10,
            max_tokens_per_minute: 1000,
            max_cost_per_hour: 1.0,
            max_concurrent_requests: 5,
            allowed_models: Some(vec!["gpt-4".to_string()]),
            priority: 100,
        };

        manager
            .initialize_tenant(tenant_id.to_string(), Some(limits))
            .await
            .unwrap();

        let request = CompletionRequest::new("gpt-4", vec![Message::user("Test message")]);

        let permit = manager
            .check_request_allowed(tenant_id, &request, 0.01)
            .await
            .unwrap();
        assert_eq!(permit.tenant_id, tenant_id);

        manager.record_request_completion(permit, None, 0.01).await;

        let stats = manager.get_tenant_stats(tenant_id).await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.total_cost, 0.01);
    }

    #[test]
    fn test_rate_limiter() {
        let limits = TenantLimits::default();
        let mut limiter = TenantRateLimiter::new("test".to_string(), limits);

        // Should allow first request
        assert!(limiter.check_request_allowed(100, 0.01).is_ok());
        limiter.record_request(100, 0.01);

        // Should still allow more requests within limits
        assert!(limiter.check_request_allowed(100, 0.01).is_ok());
    }

    #[test]
    fn test_tenant_status() {
        let status = TenantStatus::Active;
        assert_eq!(status, TenantStatus::Active);

        let suspended = TenantStatus::Suspended;
        assert_eq!(suspended, TenantStatus::Suspended);
    }
}
