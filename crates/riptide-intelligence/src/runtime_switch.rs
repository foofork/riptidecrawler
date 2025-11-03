//! Runtime provider switching capabilities
//!
//! This module enables:
//! - Hot-swapping providers without downtime
//! - Configuration-driven provider switching
//! - Gradual traffic migration between providers
//! - A/B testing capabilities
//! - Rollback mechanisms

use rand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch, RwLock};
use tracing::{debug, info};

use crate::{
    failover::FailoverManager, metrics::MetricsCollector, registry::LlmRegistry, CompletionRequest,
    CompletionResponse, IntelligenceError, Result,
};

/// Runtime switching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSwitchConfig {
    pub enabled: bool,
    pub default_provider: String,
    pub switch_rules: Vec<SwitchRule>,
    pub gradual_rollout: Option<GradualRolloutConfig>,
    pub ab_testing: Option<ABTestConfig>,
    pub rollback_triggers: Vec<RollbackTrigger>,
    pub max_concurrent_switches: u32,
}

impl Default for RuntimeSwitchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_provider: "default".to_string(),
            switch_rules: Vec::new(),
            gradual_rollout: None,
            ab_testing: None,
            rollback_triggers: Vec::new(),
            max_concurrent_switches: 3,
        }
    }
}

/// Rules for when to switch providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub conditions: Vec<SwitchCondition>,
    pub action: SwitchAction,
    pub cooldown: Duration,
    pub last_triggered: Option<chrono::DateTime<chrono::Utc>>,
}

/// Conditions that trigger a provider switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchCondition {
    ErrorRateAbove {
        threshold: f64,
        window: Duration,
    },
    LatencyAbove {
        threshold: Duration,
        window: Duration,
    },
    CostAbove {
        threshold: f64,
        window: Duration,
    },
    ProviderUnavailable {
        provider: String,
    },
    TimeSchedule {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
    },
    TenantSpecific {
        tenant_id: String,
        provider: String,
    },
    ModelSpecific {
        model: String,
        provider: String,
    },
    RequestPattern {
        pattern: String,
        provider: String,
    },
    ManualTrigger,
}

/// Actions to take when switching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchAction {
    SwitchTo { provider: String },
    RoundRobin { providers: Vec<String> },
    WeightedSwitch { weights: HashMap<String, f64> },
    Failover { primary: String, fallback: String },
    DisableProvider { provider: String },
    Rollback,
}

/// Gradual rollout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradualRolloutConfig {
    pub enabled: bool,
    pub from_provider: String,
    pub to_provider: String,
    pub duration: Duration,
    pub traffic_increment: f64, // percentage per step
    pub step_duration: Duration,
    pub success_threshold: f64, // minimum success rate to continue
    pub auto_rollback: bool,
}

/// A/B testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub enabled: bool,
    pub test_id: String,
    pub control_provider: String,
    pub treatment_provider: String,
    pub traffic_split: f64, // percentage to treatment
    pub duration: Duration,
    pub success_metrics: Vec<String>,
    pub user_allocation: UserAllocationStrategy,
}

/// User allocation strategies for A/B testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAllocationStrategy {
    Random,
    HashBased { field: String }, // hash based on user ID, tenant ID, etc.
    Sequential,
    Geographic { regions: Vec<String> },
}

/// Triggers for automatic rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackTrigger {
    ErrorRateSpike {
        threshold: f64,
        window: Duration,
    },
    LatencySpike {
        threshold: Duration,
        window: Duration,
    },
    CostSpike {
        threshold: f64,
        window: Duration,
    },
    SuccessRateBelow {
        threshold: f64,
        window: Duration,
    },
    ManualTrigger,
    TimeLimit {
        duration: Duration,
    },
}

/// Current switching state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchState {
    pub current_provider: String,
    pub target_provider: Option<String>,
    pub switch_progress: f64, // 0.0 to 1.0
    pub switch_start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub active_rules: Vec<String>,
    pub traffic_distribution: HashMap<String, f64>,
    pub rollout_state: Option<RolloutState>,
    pub ab_test_state: Option<ABTestState>,
}

/// Gradual rollout state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutState {
    pub config: GradualRolloutConfig,
    pub current_traffic_to_new: f64,
    pub step_number: u32,
    pub last_step_time: chrono::DateTime<chrono::Utc>,
    pub success_rate: f64,
    pub should_continue: bool,
}

/// A/B test state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestState {
    pub config: ABTestConfig,
    pub control_metrics: HashMap<String, f64>,
    pub treatment_metrics: HashMap<String, f64>,
    pub user_assignments: HashMap<String, String>, // user_id -> provider
    pub statistical_significance: Option<f64>,
}

/// Switch events for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchEvent {
    ProviderSwitched {
        from: String,
        to: String,
        rule_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RolloutStarted {
        from: String,
        to: String,
        duration: Duration,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RolloutCompleted {
        from: String,
        to: String,
        success: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ABTestStarted {
        test_id: String,
        control: String,
        treatment: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ABTestCompleted {
        test_id: String,
        winner: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RollbackTriggered {
        provider: String,
        trigger: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    RuleTriggered {
        rule_id: String,
        condition: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Runtime provider switching manager
pub struct RuntimeSwitchManager {
    registry: Arc<LlmRegistry>,
    failover_manager: Option<Arc<FailoverManager>>,
    metrics_collector: Arc<MetricsCollector>,
    config: Arc<RwLock<RuntimeSwitchConfig>>,
    state: Arc<RwLock<SwitchState>>,
    event_tx: mpsc::UnboundedSender<SwitchEvent>,
    config_watcher: watch::Receiver<RuntimeSwitchConfig>,
    config_sender: watch::Sender<RuntimeSwitchConfig>,
}

impl RuntimeSwitchManager {
    /// Create a new runtime switch manager
    pub fn new(
        registry: Arc<LlmRegistry>,
        failover_manager: Option<Arc<FailoverManager>>,
        metrics_collector: Arc<MetricsCollector>,
        initial_config: RuntimeSwitchConfig,
    ) -> (Self, mpsc::UnboundedReceiver<SwitchEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (config_sender, config_watcher) = watch::channel(initial_config.clone());

        let initial_state = SwitchState {
            current_provider: initial_config.default_provider.clone(),
            target_provider: None,
            switch_progress: 1.0,
            switch_start_time: None,
            active_rules: Vec::new(),
            traffic_distribution: HashMap::new(),
            rollout_state: None,
            ab_test_state: None,
        };

        let manager = Self {
            registry,
            failover_manager,
            metrics_collector,
            config: Arc::new(RwLock::new(initial_config)),
            state: Arc::new(RwLock::new(initial_state)),
            event_tx,
            config_watcher,
            config_sender,
        };

        (manager, event_rx)
    }

    /// Start the runtime switching service
    pub async fn start(&self) -> Result<()> {
        info!("Starting runtime switch manager");

        // Start configuration watcher
        let config_watcher = self.config_watcher.clone();
        let config = Arc::clone(&self.config);
        tokio::spawn(async move {
            let mut watcher = config_watcher;
            while watcher.changed().await.is_ok() {
                let new_config = watcher.borrow().clone();
                *config.write().await = new_config;
                info!("Runtime switch configuration updated");
            }
        });

        // Start rule evaluation loop
        let state = Arc::clone(&self.state);
        let config = Arc::clone(&self.config);
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;

                let config_guard = config.read().await;
                if !config_guard.enabled {
                    continue;
                }

                // Evaluate switch rules
                for rule in &config_guard.switch_rules {
                    if !rule.enabled {
                        continue;
                    }

                    // Check cooldown
                    if let Some(last_triggered) = rule.last_triggered {
                        let elapsed = chrono::Utc::now() - last_triggered;
                        if let Ok(cooldown_duration) = chrono::Duration::from_std(rule.cooldown) {
                            if elapsed < cooldown_duration {
                                continue;
                            }
                        }
                    }

                    // Evaluate conditions
                    let should_trigger =
                        Self::evaluate_rule_conditions(&rule.conditions, &metrics_collector).await;

                    if should_trigger {
                        let event = SwitchEvent::RuleTriggered {
                            rule_id: rule.id.clone(),
                            condition: "conditions met".to_string(),
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = event_tx.send(event);

                        // Execute switch action would be implemented here
                        info!("Switch rule {} triggered", rule.id);
                    }
                }

                // Update gradual rollout if active
                let mut state_guard = state.write().await;
                if let Some(ref mut rollout_state) = state_guard.rollout_state {
                    Self::update_gradual_rollout(rollout_state).await;
                }
                drop(state_guard);
            }
        });

        Ok(())
    }

    /// Execute a completion with runtime provider selection
    pub async fn complete_with_runtime_switch(
        &self,
        request: CompletionRequest,
        tenant_id: Option<String>,
    ) -> Result<CompletionResponse> {
        let provider_name = self
            .select_provider_for_request(&request, &tenant_id)
            .await?;

        debug!(
            "Selected provider {} for request {}",
            provider_name, request.id
        );

        // Start metrics collection
        let request_id = self
            .metrics_collector
            .start_request(&request, &provider_name, tenant_id)
            .await;

        // Get provider from registry
        let provider = self.registry.get_provider(&provider_name).ok_or_else(|| {
            IntelligenceError::Configuration(format!("Provider {} not found", provider_name))
        })?;

        // Clone request for potential failover
        let request_clone = request.clone();

        // Execute request
        match provider.complete(request).await {
            Ok(response) => {
                // Record success metrics
                let cost = provider.estimate_cost(response.usage.total_tokens as usize);
                self.metrics_collector
                    .complete_request_success(request_id, &response, Some(cost))
                    .await;

                Ok(response)
            }
            Err(e) => {
                // Record error metrics
                self.metrics_collector
                    .complete_request_error(request_id, "provider_error", &e.to_string())
                    .await;

                // Check if we should trigger failover
                if let Some(ref failover_manager) = self.failover_manager {
                    // Use failover manager for automatic failover
                    return failover_manager.complete_with_failover(request_clone).await;
                }

                Err(e)
            }
        }
    }

    /// Select provider for a specific request
    async fn select_provider_for_request(
        &self,
        request: &CompletionRequest,
        tenant_id: &Option<String>,
    ) -> Result<String> {
        let state = self.state.read().await;
        let config = self.config.read().await;

        // Check A/B testing
        if let Some(ref ab_test) = state.ab_test_state {
            if ab_test.config.enabled {
                return Ok(self.select_for_ab_test(request, tenant_id, ab_test).await);
            }
        }

        // Check gradual rollout
        if let Some(ref rollout) = state.rollout_state {
            if rollout.should_continue {
                return Ok(self.select_for_rollout(rollout).await);
            }
        }

        // Check tenant-specific rules
        if let Some(tenant_id) = tenant_id {
            for rule in &config.switch_rules {
                if !rule.enabled {
                    continue;
                }

                for condition in &rule.conditions {
                    if let SwitchCondition::TenantSpecific {
                        tenant_id: rule_tenant,
                        provider,
                    } = condition
                    {
                        if tenant_id == rule_tenant {
                            return Ok(provider.clone());
                        }
                    }
                }
            }
        }

        // Check model-specific rules
        for rule in &config.switch_rules {
            if !rule.enabled {
                continue;
            }

            for condition in &rule.conditions {
                if let SwitchCondition::ModelSpecific { model, provider } = condition {
                    if &request.model == model {
                        return Ok(provider.clone());
                    }
                }
            }
        }

        // Use current default provider
        Ok(state.current_provider.clone())
    }

    /// Select provider for A/B testing
    async fn select_for_ab_test(
        &self,
        _request: &CompletionRequest,
        tenant_id: &Option<String>,
        ab_test: &ABTestState,
    ) -> String {
        // Simple implementation - in practice this would be more sophisticated
        let anonymous = "anonymous".to_string();
        let user_key = tenant_id.as_ref().unwrap_or(&anonymous);

        match ab_test.config.user_allocation {
            UserAllocationStrategy::Random => {
                if rand::random::<f64>() < ab_test.config.traffic_split {
                    ab_test.config.treatment_provider.clone()
                } else {
                    ab_test.config.control_provider.clone()
                }
            }
            UserAllocationStrategy::HashBased { .. } => {
                // Simple hash-based allocation
                let hash = self.hash_string(user_key);
                if ((hash % 100) as f64) < ab_test.config.traffic_split * 100.0 {
                    ab_test.config.treatment_provider.clone()
                } else {
                    ab_test.config.control_provider.clone()
                }
            }
            _ => ab_test.config.control_provider.clone(),
        }
    }

    /// Select provider for gradual rollout
    async fn select_for_rollout(&self, rollout: &RolloutState) -> String {
        if rand::random::<f64>() < rollout.current_traffic_to_new {
            rollout.config.to_provider.clone()
        } else {
            rollout.config.from_provider.clone()
        }
    }

    /// Start a gradual rollout
    pub async fn start_gradual_rollout(&self, config: GradualRolloutConfig) -> Result<()> {
        let mut state = self.state.write().await;

        let rollout_state = RolloutState {
            current_traffic_to_new: 0.0,
            step_number: 0,
            last_step_time: chrono::Utc::now(),
            success_rate: 100.0,
            should_continue: true,
            config: config.clone(),
        };

        state.rollout_state = Some(rollout_state);

        let event = SwitchEvent::RolloutStarted {
            from: config.from_provider,
            to: config.to_provider,
            duration: config.duration,
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_tx.send(event);

        info!("Started gradual rollout");
        Ok(())
    }

    /// Start an A/B test
    pub async fn start_ab_test(&self, config: ABTestConfig) -> Result<()> {
        let mut state = self.state.write().await;

        let ab_test_state = ABTestState {
            control_metrics: HashMap::new(),
            treatment_metrics: HashMap::new(),
            user_assignments: HashMap::new(),
            statistical_significance: None,
            config: config.clone(),
        };

        state.ab_test_state = Some(ab_test_state);

        let event = SwitchEvent::ABTestStarted {
            test_id: config.test_id.clone(),
            control: config.control_provider,
            treatment: config.treatment_provider,
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_tx.send(event);

        info!("Started A/B test: {}", config.test_id);
        Ok(())
    }

    /// Switch to a specific provider immediately
    pub async fn switch_to_provider(&self, provider_name: String) -> Result<()> {
        let mut state = self.state.write().await;
        let old_provider = state.current_provider.clone();
        state.current_provider = provider_name.clone();
        state.switch_progress = 1.0;

        let event = SwitchEvent::ProviderSwitched {
            from: old_provider,
            to: provider_name.clone(),
            rule_id: "manual".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_tx.send(event);

        info!("Switched to provider: {}", provider_name);
        Ok(())
    }

    /// Update configuration at runtime
    pub async fn update_config(&self, new_config: RuntimeSwitchConfig) -> Result<()> {
        self.config_sender
            .send(new_config)
            .map_err(|_| IntelligenceError::Configuration("Failed to update config".to_string()))?;

        info!("Runtime switch configuration updated");
        Ok(())
    }

    /// Get current switching state
    pub async fn get_state(&self) -> SwitchState {
        self.state.read().await.clone()
    }

    /// Evaluate rule conditions
    async fn evaluate_rule_conditions(
        conditions: &[SwitchCondition],
        _metrics_collector: &MetricsCollector,
    ) -> bool {
        for condition in conditions {
            match condition {
                SwitchCondition::ManualTrigger => return true,
                SwitchCondition::ErrorRateAbove { .. } => {
                    // Implementation would check actual error rates
                    continue;
                }
                SwitchCondition::LatencyAbove { .. } => {
                    // Implementation would check actual latency
                    continue;
                }
                _ => continue,
            }
        }
        false
    }

    /// Update gradual rollout progress
    async fn update_gradual_rollout(rollout_state: &mut RolloutState) {
        let now = chrono::Utc::now();
        let elapsed_since_step = now - rollout_state.last_step_time;

        if let Ok(step_duration) = chrono::Duration::from_std(rollout_state.config.step_duration) {
            if elapsed_since_step >= step_duration {
                // Move to next step
                if rollout_state.current_traffic_to_new < 1.0 {
                    rollout_state.current_traffic_to_new += rollout_state.config.traffic_increment;
                    rollout_state.current_traffic_to_new =
                        rollout_state.current_traffic_to_new.min(1.0);
                    rollout_state.step_number += 1;
                    rollout_state.last_step_time = now;

                    // Check if we should continue based on success rate
                    if rollout_state.success_rate < rollout_state.config.success_threshold {
                        rollout_state.should_continue = false;
                        info!(
                            "Rollout stopped due to low success rate: {}",
                            rollout_state.success_rate
                        );
                    }
                } else {
                    // Rollout completed
                    rollout_state.should_continue = false;
                    info!("Gradual rollout completed");
                }
            }
        }
    }

    /// Simple hash function for user allocation
    fn hash_string(&self, s: &str) -> u32 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Trigger a manual rollback
    pub async fn trigger_rollback(&self, reason: String) -> Result<()> {
        let mut state = self.state.write().await;

        // Reset to previous provider if we have rollout state
        if let Some(ref rollout) = state.rollout_state {
            state.current_provider = rollout.config.from_provider.clone();
            state.rollout_state = None;
        }

        // Clear A/B test state
        state.ab_test_state = None;

        let event = SwitchEvent::RollbackTriggered {
            provider: state.current_provider.clone(),
            trigger: reason,
            timestamp: chrono::Utc::now(),
        };
        let _ = self.event_tx.send(event);

        info!("Manual rollback triggered");
        Ok(())
    }

    /// Get traffic distribution for monitoring
    pub async fn get_traffic_distribution(&self) -> HashMap<String, f64> {
        let state = self.state.read().await;

        let mut distribution = HashMap::new();

        // Check for rollout distribution
        if let Some(ref rollout) = state.rollout_state {
            distribution.insert(
                rollout.config.from_provider.clone(),
                1.0 - rollout.current_traffic_to_new,
            );
            distribution.insert(
                rollout.config.to_provider.clone(),
                rollout.current_traffic_to_new,
            );
        } else if let Some(ref ab_test) = state.ab_test_state {
            // A/B test distribution
            distribution.insert(
                ab_test.config.control_provider.clone(),
                1.0 - ab_test.config.traffic_split,
            );
            distribution.insert(
                ab_test.config.treatment_provider.clone(),
                ab_test.config.traffic_split,
            );
        } else {
            // Single provider
            distribution.insert(state.current_provider.clone(), 1.0);
        }

        distribution
    }

    /// Check if a provider switch is currently in progress
    pub async fn is_switch_in_progress(&self) -> bool {
        let state = self.state.read().await;
        state.rollout_state.is_some() || state.ab_test_state.is_some()
    }

    /// Get switch progress (0.0 to 1.0)
    pub async fn get_switch_progress(&self) -> f64 {
        let state = self.state.read().await;

        if let Some(ref rollout) = state.rollout_state {
            rollout.current_traffic_to_new
        } else {
            state.switch_progress
        }
    }
}
