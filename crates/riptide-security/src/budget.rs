//! Budget Enforcement System for RipTide
//!
//! Provides comprehensive cost tracking, budget enforcement, and circuit breaking
//! to prevent excessive spending on LLM operations.

use crate::types::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, error, info, warn};

/// Model pricing information (cost per 1K tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_name: String,
    pub input_cost_per_1k_tokens: f64,
    pub output_cost_per_1k_tokens: f64,
    pub last_updated: DateTime<Utc>,
}

impl ModelPricing {
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k_tokens;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k_tokens;
        input_cost + output_cost
    }
}

/// Budget usage tracking for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub request_count: u64,
    pub tenant_costs: HashMap<TenantId, f64>,
    pub model_costs: HashMap<String, f64>,
    pub last_updated: DateTime<Utc>,
}

impl BudgetUsage {
    pub fn new(period_start: DateTime<Utc>, period_end: DateTime<Utc>) -> Self {
        Self {
            period_start,
            period_end,
            total_cost_usd: 0.0,
            total_tokens: 0,
            request_count: 0,
            tenant_costs: HashMap::new(),
            model_costs: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    pub fn add_usage(&mut self, tenant_id: &TenantId, cost_info: &CostInfo) {
        self.total_cost_usd += cost_info.estimated_cost_usd;
        self.total_tokens += cost_info.tokens_used;
        self.request_count += 1;

        *self.tenant_costs.entry(tenant_id.clone()).or_insert(0.0) += cost_info.estimated_cost_usd;
        *self
            .model_costs
            .entry(cost_info.model_name.clone())
            .or_insert(0.0) += cost_info.estimated_cost_usd;

        self.last_updated = Utc::now();
    }

    pub fn get_tenant_usage(&self, tenant_id: &TenantId) -> f64 {
        self.tenant_costs.get(tenant_id).copied().unwrap_or(0.0)
    }

    pub fn get_model_usage(&self, model_name: &str) -> f64 {
        self.model_costs.get(model_name).copied().unwrap_or(0.0)
    }
}

/// Circuit breaker states for budget enforcement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Budget exceeded, blocking requests
    HalfOpen, // Testing if budget is available again
}

/// Circuit breaker for budget enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCircuitBreaker {
    pub state: CircuitBreakerState,
    pub opened_at: Option<DateTime<Utc>>,
    pub failure_count: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub grace_period_end: Option<DateTime<Utc>>,
}

impl Default for BudgetCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl BudgetCircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            opened_at: None,
            failure_count: 0,
            last_failure: None,
            grace_period_end: None,
        }
    }

    pub fn can_proceed(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if grace period has expired
                if let Some(grace_end) = self.grace_period_end {
                    Utc::now() > grace_end
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.opened_at = None;
        self.last_failure = None;
        self.grace_period_end = None;
    }

    pub fn record_failure(&mut self, grace_period_minutes: u32) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());

        if self.state == CircuitBreakerState::Closed {
            self.state = CircuitBreakerState::Open;
            self.opened_at = Some(Utc::now());
            self.grace_period_end =
                Some(Utc::now() + Duration::minutes(grace_period_minutes as i64));
        } else if self.state == CircuitBreakerState::HalfOpen {
            self.state = CircuitBreakerState::Open;
            self.grace_period_end =
                Some(Utc::now() + Duration::minutes(grace_period_minutes as i64));
        }
    }

    pub fn try_half_open(&mut self) {
        if self.state == CircuitBreakerState::Open {
            if let Some(grace_end) = self.grace_period_end {
                if Utc::now() > grace_end {
                    self.state = CircuitBreakerState::HalfOpen;
                }
            }
        }
    }
}

/// Budget Manager - handles all budget enforcement and cost tracking
pub struct BudgetManager {
    limits: BudgetLimits,
    model_pricing: Arc<RwLock<HashMap<String, ModelPricing>>>,
    current_usage: Arc<AsyncRwLock<BudgetUsage>>,
    historical_usage: Arc<AsyncRwLock<Vec<BudgetUsage>>>,
    circuit_breaker: Arc<RwLock<BudgetCircuitBreaker>>,
    job_costs: Arc<AsyncRwLock<HashMap<String, f64>>>, // job_id -> cost
}

impl BudgetManager {
    /// Helper to safely create a DateTime from year, month, day
    fn safe_datetime(year: i32, month: u32, day: u32) -> DateTime<Utc> {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap_or_else(Utc::now) // Fallback to now if date is invalid
    }

    /// Create a new Budget Manager with default limits
    pub fn new(limits: Option<BudgetLimits>) -> Self {
        let limits = limits.unwrap_or_default();
        let now = Utc::now();
        let month_start = Self::safe_datetime(now.year(), now.month(), 1);
        let month_end = if now.month() == 12 {
            Self::safe_datetime(now.year() + 1, 1, 1)
        } else {
            Self::safe_datetime(now.year(), now.month() + 1, 1)
        };

        Self {
            limits,
            model_pricing: Arc::new(RwLock::new(Self::default_pricing())),
            current_usage: Arc::new(AsyncRwLock::new(BudgetUsage::new(month_start, month_end))),
            historical_usage: Arc::new(AsyncRwLock::new(Vec::new())),
            circuit_breaker: Arc::new(RwLock::new(BudgetCircuitBreaker::new())),
            job_costs: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }

    /// Initialize with default model pricing
    fn default_pricing() -> HashMap<String, ModelPricing> {
        let mut pricing = HashMap::new();

        // OpenAI GPT-4 pricing (as of 2024)
        pricing.insert(
            "gpt-4".to_string(),
            ModelPricing {
                model_name: "gpt-4".to_string(),
                input_cost_per_1k_tokens: 0.03,
                output_cost_per_1k_tokens: 0.06,
                last_updated: Utc::now(),
            },
        );

        pricing.insert(
            "gpt-4-turbo".to_string(),
            ModelPricing {
                model_name: "gpt-4-turbo".to_string(),
                input_cost_per_1k_tokens: 0.01,
                output_cost_per_1k_tokens: 0.03,
                last_updated: Utc::now(),
            },
        );

        pricing.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricing {
                model_name: "gpt-3.5-turbo".to_string(),
                input_cost_per_1k_tokens: 0.0015,
                output_cost_per_1k_tokens: 0.002,
                last_updated: Utc::now(),
            },
        );

        // Claude pricing
        pricing.insert(
            "claude-3-opus".to_string(),
            ModelPricing {
                model_name: "claude-3-opus".to_string(),
                input_cost_per_1k_tokens: 0.015,
                output_cost_per_1k_tokens: 0.075,
                last_updated: Utc::now(),
            },
        );

        pricing.insert(
            "claude-3-sonnet".to_string(),
            ModelPricing {
                model_name: "claude-3-sonnet".to_string(),
                input_cost_per_1k_tokens: 0.003,
                output_cost_per_1k_tokens: 0.015,
                last_updated: Utc::now(),
            },
        );

        pricing
    }

    /// Check if a job can proceed within budget limits
    pub async fn check_budget_for_job(
        &self,
        job_id: &str,
        tenant_id: &TenantId,
        estimated_tokens: u64,
        model_name: &str,
    ) -> SecurityResult<CostInfo> {
        // Check circuit breaker first
        {
            let mut breaker = self.circuit_breaker.write().map_err(|e| {
                SecurityError::Unknown(format!("Circuit breaker lock error: {}", e))
            })?;

            breaker.try_half_open();

            if !breaker.can_proceed() {
                warn!(
                    tenant_id = %tenant_id,
                    job_id = job_id,
                    "Job blocked by budget circuit breaker"
                );
                return Err(SecurityError::BudgetLimitExceeded(
                    "Budget circuit breaker is open - spending limits exceeded".to_string(),
                ));
            }
        }

        // Calculate estimated cost
        let estimated_cost = self
            .calculate_estimated_cost(estimated_tokens, model_name)
            .await?;
        let cost_info = CostInfo::new(
            estimated_tokens,
            estimated_cost,
            model_name.to_string(),
            "llm_request".to_string(),
        );

        // Check per-job limit
        if estimated_cost > self.limits.per_job_limit_usd {
            warn!(
                tenant_id = %tenant_id,
                job_id = job_id,
                estimated_cost = estimated_cost,
                limit = self.limits.per_job_limit_usd,
                "Job exceeds per-job budget limit"
            );

            self.record_budget_violation().await;
            return Err(SecurityError::BudgetLimitExceeded(format!(
                "Job cost ${:.2} exceeds per-job limit ${:.2}",
                estimated_cost, self.limits.per_job_limit_usd
            )));
        }

        // Check monthly limits
        let current_usage = self.current_usage.read().await;
        let projected_total = current_usage.total_cost_usd + estimated_cost;

        if projected_total > self.limits.global_monthly_limit_usd {
            warn!(
                tenant_id = %tenant_id,
                job_id = job_id,
                current_usage = current_usage.total_cost_usd,
                estimated_cost = estimated_cost,
                projected_total = projected_total,
                monthly_limit = self.limits.global_monthly_limit_usd,
                "Job would exceed global monthly budget limit"
            );

            drop(current_usage);
            self.record_budget_violation().await;
            return Err(SecurityError::BudgetLimitExceeded(format!(
                "Total monthly cost would reach ${:.2}, exceeding limit ${:.2}",
                projected_total, self.limits.global_monthly_limit_usd
            )));
        }

        // Check tenant-specific limit if configured
        if let Some(tenant_limit) = self.limits.per_tenant_monthly_limit_usd {
            let tenant_usage = current_usage.get_tenant_usage(tenant_id);
            let projected_tenant_total = tenant_usage + estimated_cost;

            if projected_tenant_total > tenant_limit {
                warn!(
                    tenant_id = %tenant_id,
                    job_id = job_id,
                    current_tenant_usage = tenant_usage,
                    estimated_cost = estimated_cost,
                    projected_tenant_total = projected_tenant_total,
                    tenant_limit = tenant_limit,
                    "Job would exceed tenant monthly budget limit"
                );

                drop(current_usage);
                self.record_budget_violation().await;
                return Err(SecurityError::BudgetLimitExceeded(format!(
                    "Tenant monthly cost would reach ${:.2}, exceeding limit ${:.2}",
                    projected_tenant_total, tenant_limit
                )));
            }
        }

        // Check warning threshold
        let usage_percentage = (projected_total / self.limits.global_monthly_limit_usd) * 100.0;
        if usage_percentage >= self.limits.warning_threshold_percent {
            warn!(
                tenant_id = %tenant_id,
                job_id = job_id,
                usage_percentage = usage_percentage,
                warning_threshold = self.limits.warning_threshold_percent,
                "Approaching budget warning threshold"
            );
        }

        // Track the job cost
        {
            let mut job_costs = self.job_costs.write().await;
            job_costs.insert(job_id.to_string(), estimated_cost);
        }

        debug!(
            tenant_id = %tenant_id,
            job_id = job_id,
            estimated_cost = estimated_cost,
            "Job approved for budget"
        );

        Ok(cost_info)
    }

    /// Record actual usage after job completion
    pub async fn record_usage(
        &self,
        job_id: &str,
        tenant_id: &TenantId,
        actual_cost_info: CostInfo,
    ) -> Result<()> {
        // Update current usage
        {
            let mut current_usage = self.current_usage.write().await;
            current_usage.add_usage(tenant_id, &actual_cost_info);
        }

        // Remove from pending job costs
        {
            let mut job_costs = self.job_costs.write().await;
            job_costs.remove(job_id);
        }

        // Reset circuit breaker on successful completion
        {
            let mut breaker = self
                .circuit_breaker
                .write()
                .map_err(|e| anyhow!("Circuit breaker lock error: {}", e))?;
            breaker.record_success();
        }

        info!(
            tenant_id = %tenant_id,
            job_id = job_id,
            actual_cost = actual_cost_info.estimated_cost_usd,
            tokens_used = actual_cost_info.tokens_used,
            model = actual_cost_info.model_name,
            "Usage recorded successfully"
        );

        Ok(())
    }

    /// Calculate estimated cost for tokens and model
    async fn calculate_estimated_cost(&self, tokens: u64, model_name: &str) -> SecurityResult<f64> {
        let pricing = self
            .model_pricing
            .read()
            .map_err(|e| SecurityError::Unknown(format!("Pricing lock error: {}", e)))?;

        if let Some(model_pricing) = pricing.get(model_name) {
            // Assume roughly equal input/output tokens for estimation
            let estimated_input_tokens = tokens / 2;
            let estimated_output_tokens = tokens / 2;
            Ok(model_pricing.calculate_cost(estimated_input_tokens, estimated_output_tokens))
        } else {
            // Use a conservative default rate if model not found
            warn!(
                model_name = model_name,
                "Unknown model, using default pricing"
            );
            Ok((tokens as f64 / 1000.0) * 0.05) // $0.05 per 1K tokens default
        }
    }

    /// Record a budget violation
    async fn record_budget_violation(&self) {
        let mut breaker = match self.circuit_breaker.write() {
            Ok(breaker) => breaker,
            Err(e) => {
                error!("Failed to acquire circuit breaker lock: {}", e);
                return;
            }
        };

        breaker.record_failure(self.limits.grace_period_minutes);

        if breaker.state == CircuitBreakerState::Open {
            error!(
                grace_period_minutes = self.limits.grace_period_minutes,
                "Budget circuit breaker opened due to limit violation"
            );
        }
    }

    /// Get current budget usage
    pub async fn get_current_usage(&self) -> BudgetUsage {
        self.current_usage.read().await.clone()
    }

    /// Get budget usage for a specific tenant
    pub async fn get_tenant_usage(&self, tenant_id: &TenantId) -> f64 {
        let usage = self.current_usage.read().await;
        usage.get_tenant_usage(tenant_id)
    }

    /// Get remaining budget
    pub async fn get_remaining_budget(&self) -> f64 {
        let usage = self.current_usage.read().await;
        (self.limits.global_monthly_limit_usd - usage.total_cost_usd).max(0.0)
    }

    /// Check if we're approaching budget limits
    pub async fn check_budget_health(&self) -> BudgetHealthStatus {
        let usage = self.current_usage.read().await;
        let usage_percentage =
            (usage.total_cost_usd / self.limits.global_monthly_limit_usd) * 100.0;
        let remaining = self.limits.global_monthly_limit_usd - usage.total_cost_usd;

        let is_circuit_open = self
            .circuit_breaker
            .read()
            .map(|breaker| breaker.state == CircuitBreakerState::Open)
            .unwrap_or(true); // If lock poisoned, treat as open for safety

        BudgetHealthStatus {
            current_usage_usd: usage.total_cost_usd,
            monthly_limit_usd: self.limits.global_monthly_limit_usd,
            usage_percentage,
            remaining_budget_usd: remaining,
            is_circuit_breaker_open: is_circuit_open,
            warning_threshold_reached: usage_percentage >= self.limits.warning_threshold_percent,
            days_remaining_in_month: Self::days_remaining_in_month(),
            projected_monthly_cost: Self::project_monthly_cost(&usage),
        }
    }

    /// Update model pricing
    pub async fn update_model_pricing(
        &self,
        model_name: String,
        pricing: ModelPricing,
    ) -> Result<()> {
        let mut pricing_map = self
            .model_pricing
            .write()
            .map_err(|e| anyhow!("Pricing lock error: {}", e))?;

        pricing_map.insert(model_name.clone(), pricing);

        info!(model_name = model_name, "Model pricing updated");
        Ok(())
    }

    /// Reset monthly usage (typically called at month start)
    pub async fn reset_monthly_usage(&self) -> Result<()> {
        let mut current_usage = self.current_usage.write().await;
        let mut historical_usage = self.historical_usage.write().await;

        // Archive current usage
        historical_usage.push(current_usage.clone());

        // Reset for new month
        let now = Utc::now();
        let month_start = Self::safe_datetime(now.year(), now.month(), 1);
        let month_end = if now.month() == 12 {
            Self::safe_datetime(now.year() + 1, 1, 1)
        } else {
            Self::safe_datetime(now.year(), now.month() + 1, 1)
        };

        *current_usage = BudgetUsage::new(month_start, month_end);

        // Reset circuit breaker
        let mut breaker = self
            .circuit_breaker
            .write()
            .map_err(|e| anyhow!("Circuit breaker lock error: {}", e))?;
        breaker.record_success();

        info!("Monthly budget usage reset");
        Ok(())
    }

    fn days_remaining_in_month() -> u32 {
        let now = Utc::now();
        let month_end = if now.month() == 12 {
            Self::safe_datetime(now.year() + 1, 1, 1)
        } else {
            Self::safe_datetime(now.year(), now.month() + 1, 1)
        };

        (month_end - now).num_days() as u32
    }

    fn project_monthly_cost(usage: &BudgetUsage) -> f64 {
        let days_elapsed = (Utc::now() - usage.period_start).num_days() as f64;
        if days_elapsed <= 0.0 {
            return usage.total_cost_usd;
        }

        let daily_average = usage.total_cost_usd / days_elapsed;
        let days_in_month = (usage.period_end - usage.period_start).num_days() as f64;
        daily_average * days_in_month
    }
}

/// Budget health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetHealthStatus {
    pub current_usage_usd: f64,
    pub monthly_limit_usd: f64,
    pub usage_percentage: f64,
    pub remaining_budget_usd: f64,
    pub is_circuit_breaker_open: bool,
    pub warning_threshold_reached: bool,
    pub days_remaining_in_month: u32,
    pub projected_monthly_cost: f64,
}

impl Default for BudgetManager {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_budget_manager_creation() {
        let manager = BudgetManager::new(None);
        let health = manager.check_budget_health().await;

        assert_eq!(health.current_usage_usd, 0.0);
        assert_eq!(health.monthly_limit_usd, 2000.0);
        assert!(!health.is_circuit_breaker_open);
    }

    #[tokio::test]
    async fn test_budget_check_within_limits() {
        let manager = BudgetManager::new(None);
        let tenant_id = TenantId::from("test-tenant");

        let result = manager
            .check_budget_for_job("job-1", &tenant_id, 1000, "gpt-3.5-turbo")
            .await;

        assert!(result.is_ok());
        let cost_info = result.unwrap();
        assert!(cost_info.estimated_cost_usd > 0.0);
    }

    #[tokio::test]
    async fn test_per_job_limit_exceeded() {
        let limits = BudgetLimits {
            per_job_limit_usd: 0.01, // Very low limit
            ..Default::default()
        };
        let manager = BudgetManager::new(Some(limits));
        let tenant_id = TenantId::from("test-tenant");

        let result = manager
            .check_budget_for_job("job-1", &tenant_id, 10000, "gpt-4") // High token count
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityError::BudgetLimitExceeded(_)
        ));
    }

    #[tokio::test]
    async fn test_usage_recording() {
        let manager = BudgetManager::new(None);
        let tenant_id = TenantId::from("test-tenant");

        let cost_info = CostInfo::new(1000, 0.05, "gpt-3.5-turbo".to_string(), "test".to_string());

        let result = manager.record_usage("job-1", &tenant_id, cost_info).await;
        assert!(result.is_ok());

        let usage = manager.get_current_usage().await;
        assert_eq!(usage.total_cost_usd, 0.05);
        assert_eq!(usage.total_tokens, 1000);
        assert_eq!(usage.request_count, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let limits = BudgetLimits {
            global_monthly_limit_usd: 0.01, // Very low limit
            grace_period_minutes: 1,
            ..Default::default()
        };
        let manager = BudgetManager::new(Some(limits));
        let tenant_id = TenantId::from("test-tenant");

        // First request should trigger circuit breaker
        let result = manager
            .check_budget_for_job("job-1", &tenant_id, 10000, "gpt-4")
            .await;

        assert!(result.is_err());

        // Second request should be blocked by circuit breaker
        let result2 = manager
            .check_budget_for_job("job-2", &tenant_id, 100, "gpt-3.5-turbo")
            .await;

        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("circuit breaker"));
    }

    #[test]
    fn test_model_pricing_calculation() {
        let pricing = ModelPricing {
            model_name: "test-model".to_string(),
            input_cost_per_1k_tokens: 0.01,
            output_cost_per_1k_tokens: 0.02,
            last_updated: Utc::now(),
        };

        let cost = pricing.calculate_cost(1000, 1000);
        assert_eq!(cost, 0.03); // 0.01 + 0.02
    }
}
