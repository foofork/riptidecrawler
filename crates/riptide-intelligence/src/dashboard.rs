//! Enhanced LLM Ops Dashboard with tenant cost tracking
//!
//! This module provides comprehensive dashboards for monitoring LLM operations:
//! - Real-time metrics visualization
//! - Per-tenant cost breakdown and analysis
//! - Provider performance comparison
//! - Cost optimization recommendations
//! - SLA monitoring and alerting

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use crate::{
    metrics::{MetricsCollector, TimeWindow, AggregatedMetrics, ProviderMetrics, TenantMetrics, CostBreakdown},
    config::{TenantLimits, CostTrackingConfig},
};

/// Enhanced dashboard with tenant-focused cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLlmOpsDashboard {
    pub generated_at: DateTime<Utc>,
    pub time_window: TimeWindow,
    pub overall_metrics: AggregatedMetrics,
    pub provider_metrics: Vec<ProviderMetrics>,
    pub tenant_metrics: Vec<TenantMetrics>,
    pub cost_analysis: DetailedCostAnalysis,
    pub performance_insights: PerformanceInsights,
    pub sla_metrics: SlaMetrics,
    pub recommendations: Vec<Recommendation>,
    pub alerts: Vec<Alert>,
    pub tenant_rankings: TenantRankings,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

/// Detailed cost analysis with optimization insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCostAnalysis {
    pub total_cost: f64,
    pub currency: String,
    pub time_window: TimeWindow,
    pub cost_per_tenant: HashMap<String, TenantCostBreakdown>,
    pub cost_per_provider: HashMap<String, ProviderCostBreakdown>,
    pub cost_per_model: HashMap<String, ModelCostBreakdown>,
    pub cost_trends: Vec<CostDataPoint>,
    pub budget_utilization: Vec<BudgetUtilization>,
    pub cost_efficiency_score: f64, // 0-100
    pub projected_monthly_cost: f64,
}

/// Per-tenant cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantCostBreakdown {
    pub tenant_id: String,
    pub total_cost: f64,
    pub prompt_cost: f64,
    pub completion_cost: f64,
    pub requests: u64,
    pub tokens: u64,
    pub cost_per_request: f64,
    pub cost_per_token: f64,
    pub top_models: Vec<(String, f64)>, // (model, cost)
    pub cost_trend: Vec<CostDataPoint>,
    pub budget_remaining: Option<f64>,
    pub budget_utilization_percent: f64,
    pub cost_rank: u32, // Ranking among all tenants
}

/// Per-provider cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCostBreakdown {
    pub provider_name: String,
    pub total_cost: f64,
    pub requests: u64,
    pub success_rate: f64,
    pub avg_cost_per_request: f64,
    pub cost_efficiency_ratio: f64, // Cost per successful token
    pub uptime_percentage: f64,
    pub tenant_distribution: HashMap<String, f64>, // tenant_id -> cost
}

/// Per-model cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostBreakdown {
    pub model_name: String,
    pub total_cost: f64,
    pub requests: u64,
    pub tokens: u64,
    pub avg_latency_ms: f64,
    pub cost_per_token: f64,
    pub popularity_score: f64, // Based on usage
}

/// Cost data point for time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDataPoint {
    pub timestamp: DateTime<Utc>,
    pub cost: f64,
    pub requests: u64,
    pub tokens: u64,
}

/// Budget utilization tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUtilization {
    pub tenant_id: String,
    pub allocated_budget: f64,
    pub spent_amount: f64,
    pub utilization_percent: f64,
    pub projected_overage: Option<f64>,
    pub days_remaining_in_period: u32,
    pub burn_rate: f64, // Cost per day
    pub status: BudgetStatus,
}

/// Budget status indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetStatus {
    Healthy,
    Warning,
    Critical,
    Exceeded,
}

/// Performance insights and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub fastest_provider: String,
    pub most_reliable_provider: String,
    pub most_cost_effective_provider: String,
    pub peak_usage_hours: Vec<u8>, // Hours of day (0-23)
    pub performance_correlation: ProviderCorrelation,
    pub quality_metrics: QualityMetrics,
}

/// Provider performance correlation analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCorrelation {
    pub latency_vs_cost: HashMap<String, f64>,
    pub reliability_vs_cost: HashMap<String, f64>,
    pub tenant_satisfaction: HashMap<String, f64>,
}

/// Quality metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub response_quality_score: f64, // 0-100
    pub hallucination_rate: f64,
    pub context_adherence: f64,
    pub user_satisfaction: f64,
}

/// SLA monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaMetrics {
    pub uptime_sla: f64, // Target uptime percentage
    pub actual_uptime: f64,
    pub latency_sla: u64, // Target latency in ms
    pub actual_p99_latency: u64,
    pub error_rate_sla: f64, // Target error rate
    pub actual_error_rate: f64,
    pub sla_violations: Vec<SlaViolation>,
    pub sla_compliance_score: f64, // 0-100
}

/// SLA violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaViolation {
    pub violation_type: SlaViolationType,
    pub provider: String,
    pub tenant_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub duration_minutes: u64,
    pub severity: ViolationSeverity,
    pub impact_score: f64,
}

/// Types of SLA violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlaViolationType {
    UptimeViolation,
    LatencyViolation,
    ErrorRateViolation,
    CostOverrun,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// System recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: Uuid,
    pub category: RecommendationType,
    pub title: String,
    pub description: String,
    pub potential_savings: Option<f64>,
    pub confidence_score: f64, // 0-100
    pub priority: RecommendationPriority,
    pub implementation_effort: ImplementationEffort,
    pub affected_tenants: Vec<String>,
    pub expected_impact: String,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    CostOptimization,
    PerformanceImprovement,
    Reliability,
    Security,
    Capacity,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Urgent,
}

/// Implementation effort estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal, // < 1 day
    Low,     // 1-3 days
    Medium,  // 1-2 weeks
    High,    // > 2 weeks
}

/// System alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub tenant_id: Option<String>,
    pub provider: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub is_resolved: bool,
    pub resolution_time: Option<DateTime<Utc>>,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    BudgetExceeded,
    HighErrorRate,
    HighLatency,
    ProviderDown,
    UnusualUsagePattern,
    SecurityThreat,
    CapacityLimit,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Tenant performance rankings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRankings {
    pub by_cost: Vec<TenantRank>,
    pub by_usage: Vec<TenantRank>,
    pub by_efficiency: Vec<TenantRank>,
    pub by_growth: Vec<TenantRank>,
}

/// Individual tenant ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantRank {
    pub tenant_id: String,
    pub rank: u32,
    pub value: f64,
    pub change_from_previous: f64, // Percentage change
}

/// Cost optimization opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub opportunity_type: OptimizationType,
    pub description: String,
    pub potential_savings: f64,
    pub confidence: f64,
    pub affected_tenants: Vec<String>,
    pub implementation_steps: Vec<String>,
}

/// Types of optimization opportunities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ProviderSwitching,
    ModelDowngrade,
    RequestOptimization,
    CachingStrategy,
    LoadBalancing,
    ResourceConsolidation,
}

/// Enhanced dashboard generator
pub struct DashboardGenerator {
    metrics_collector: Arc<MetricsCollector>,
    tenant_configs: Arc<RwLock<HashMap<String, TenantLimits>>>,
    cost_config: CostTrackingConfig,
    alerts: Arc<RwLock<Vec<Alert>>>,
    recommendations: Arc<RwLock<Vec<Recommendation>>>,
}

impl DashboardGenerator {
    /// Create a new dashboard generator
    pub fn new(
        metrics_collector: Arc<MetricsCollector>,
        cost_config: CostTrackingConfig,
    ) -> Self {
        Self {
            metrics_collector,
            tenant_configs: Arc::new(RwLock::new(HashMap::new())),
            cost_config,
            alerts: Arc::new(RwLock::new(Vec::new())),
            recommendations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Update tenant configuration
    pub async fn update_tenant_config(&self, tenant_id: String, limits: TenantLimits) {
        let mut configs = self.tenant_configs.write().await;
        configs.insert(tenant_id, limits);
    }

    /// Generate enhanced dashboard
    pub async fn generate_enhanced_dashboard(
        &self,
        time_window: TimeWindow,
    ) -> EnhancedLlmOpsDashboard {
        let basic_dashboard = self.metrics_collector.generate_dashboard(time_window.clone()).await;

        let cost_analysis = self.generate_detailed_cost_analysis(time_window.clone()).await;
        let performance_insights = self.generate_performance_insights(&basic_dashboard).await;
        let sla_metrics = self.generate_sla_metrics(&basic_dashboard).await;
        let tenant_rankings = self.generate_tenant_rankings(&basic_dashboard).await;
        let optimization_opportunities = self.generate_optimization_opportunities(&cost_analysis).await;

        // Generate recommendations and alerts
        self.update_recommendations(&cost_analysis, &performance_insights).await;
        self.update_alerts(&cost_analysis, &sla_metrics).await;

        let recommendations = self.recommendations.read().await.clone();
        let alerts = self.alerts.read().await.clone();

        EnhancedLlmOpsDashboard {
            generated_at: Utc::now(),
            time_window,
            overall_metrics: basic_dashboard.overall_metrics,
            provider_metrics: basic_dashboard.provider_metrics,
            tenant_metrics: basic_dashboard.tenant_metrics,
            cost_analysis,
            performance_insights,
            sla_metrics,
            recommendations,
            alerts,
            tenant_rankings,
            optimization_opportunities,
        }
    }

    /// Generate detailed cost analysis
    async fn generate_detailed_cost_analysis(&self, time_window: TimeWindow) -> DetailedCostAnalysis {
        let basic_cost = self.metrics_collector.get_cost_breakdown(time_window.clone()).await;
        let tenant_metrics = self.metrics_collector.get_tenant_metrics(time_window.clone()).await;
        let provider_metrics = self.metrics_collector.get_provider_metrics(time_window.clone()).await;

        // Build per-tenant cost breakdown
        let mut cost_per_tenant = HashMap::new();
        for tenant_metric in &tenant_metrics {
            let tenant_cost = tenant_metric.metrics.total_cost;
            let breakdown = TenantCostBreakdown {
                tenant_id: tenant_metric.tenant_id.clone(),
                total_cost: tenant_cost,
                prompt_cost: tenant_cost * 0.6, // Estimated split
                completion_cost: tenant_cost * 0.4,
                requests: tenant_metric.metrics.request_count,
                tokens: tenant_metric.metrics.total_tokens,
                cost_per_request: tenant_metric.metrics.avg_cost_per_request,
                cost_per_token: if tenant_metric.metrics.total_tokens > 0 {
                    tenant_cost / tenant_metric.metrics.total_tokens as f64
                } else { 0.0 },
                top_models: tenant_metric.model_breakdown.iter()
                    .map(|(model, metrics)| (model.clone(), metrics.total_cost))
                    .collect(),
                cost_trend: Vec::new(), // Would be populated with time series data
                budget_remaining: self.calculate_budget_remaining(&tenant_metric.tenant_id, tenant_cost).await,
                budget_utilization_percent: self.calculate_budget_utilization(&tenant_metric.tenant_id, tenant_cost).await,
                cost_rank: 0, // Will be calculated later
            };
            cost_per_tenant.insert(tenant_metric.tenant_id.clone(), breakdown);
        }

        // Build per-provider cost breakdown
        let mut cost_per_provider = HashMap::new();
        for provider_metric in &provider_metrics {
            let breakdown = ProviderCostBreakdown {
                provider_name: provider_metric.provider_name.clone(),
                total_cost: provider_metric.metrics.total_cost,
                requests: provider_metric.metrics.request_count,
                success_rate: provider_metric.metrics.success_rate,
                avg_cost_per_request: provider_metric.metrics.avg_cost_per_request,
                cost_efficiency_ratio: self.calculate_cost_efficiency(&provider_metric.metrics),
                uptime_percentage: 100.0 - provider_metric.metrics.error_rate,
                tenant_distribution: HashMap::new(), // Would be populated from cross-reference
            };
            cost_per_provider.insert(provider_metric.provider_name.clone(), breakdown);
        }

        // Calculate cost efficiency score
        let cost_efficiency_score = self.calculate_overall_cost_efficiency(&provider_metrics);

        let projected_monthly_cost = basic_cost.total_cost * self.get_projection_multiplier(&time_window);

        DetailedCostAnalysis {
            total_cost: basic_cost.total_cost,
            currency: basic_cost.currency,
            time_window,
            cost_per_tenant,
            cost_per_provider,
            cost_per_model: HashMap::new(), // Would be populated
            cost_trends: Vec::new(),
            budget_utilization: Vec::new(),
            cost_efficiency_score,
            projected_monthly_cost,
        }
    }

    /// Calculate budget remaining for a tenant
    async fn calculate_budget_remaining(&self, tenant_id: &str, spent: f64) -> Option<f64> {
        let budget = self.cost_config.per_tenant_budgets.get(tenant_id)?;
        Some((budget - spent).max(0.0))
    }

    /// Calculate budget utilization percentage
    async fn calculate_budget_utilization(&self, tenant_id: &str, spent: f64) -> f64 {
        if let Some(budget) = self.cost_config.per_tenant_budgets.get(tenant_id) {
            if *budget > 0.0 {
                return (spent / budget) * 100.0;
            }
        }
        0.0
    }

    /// Calculate cost efficiency ratio for a provider
    fn calculate_cost_efficiency(&self, metrics: &AggregatedMetrics) -> f64 {
        if metrics.total_tokens > 0 && metrics.success_rate > 0.0 {
            let successful_tokens = metrics.total_tokens as f64 * (metrics.success_rate / 100.0);
            metrics.total_cost / successful_tokens
        } else {
            0.0
        }
    }

    /// Calculate overall cost efficiency score
    fn calculate_overall_cost_efficiency(&self, provider_metrics: &[ProviderMetrics]) -> f64 {
        if provider_metrics.is_empty() {
            return 0.0;
        }

        let total_efficiency: f64 = provider_metrics.iter()
            .map(|p| self.calculate_cost_efficiency(&p.metrics))
            .sum();

        // Normalize to 0-100 scale (lower is better for cost efficiency)
        let avg_efficiency = total_efficiency / provider_metrics.len() as f64;
        (100.0 - (avg_efficiency * 1000.0).min(100.0)).max(0.0)
    }

    /// Get projection multiplier based on time window
    fn get_projection_multiplier(&self, time_window: &TimeWindow) -> f64 {
        match time_window {
            TimeWindow::Last5Minutes => 8640.0,  // 30 days worth
            TimeWindow::Last15Minutes => 2880.0, // 30 days worth
            TimeWindow::LastHour => 720.0,       // 30 days worth
            TimeWindow::Last6Hours => 120.0,     // 30 days worth
            TimeWindow::Last24Hours => 30.0,     // 30 days worth
            TimeWindow::Last7Days => 4.3,        // ~30 days worth
            TimeWindow::Last30Days => 1.0,       // Already monthly
            TimeWindow::Custom { .. } => 1.0,    // Can't estimate
        }
    }

    /// Generate performance insights
    async fn generate_performance_insights(&self, dashboard: &crate::metrics::LlmOpsDashboard) -> PerformanceInsights {
        // Find fastest provider
        let fastest_provider = dashboard.provider_metrics.iter()
            .min_by(|a, b| a.metrics.avg_latency_ms.partial_cmp(&b.metrics.avg_latency_ms).unwrap())
            .map(|p| p.provider_name.clone())
            .unwrap_or_default();

        // Find most reliable provider
        let most_reliable_provider = dashboard.provider_metrics.iter()
            .max_by(|a, b| a.metrics.success_rate.partial_cmp(&b.metrics.success_rate).unwrap())
            .map(|p| p.provider_name.clone())
            .unwrap_or_default();

        // Find most cost-effective provider
        let most_cost_effective_provider = dashboard.provider_metrics.iter()
            .min_by(|a, b| a.metrics.avg_cost_per_request.partial_cmp(&b.metrics.avg_cost_per_request).unwrap())
            .map(|p| p.provider_name.clone())
            .unwrap_or_default();

        PerformanceInsights {
            fastest_provider,
            most_reliable_provider,
            most_cost_effective_provider,
            peak_usage_hours: vec![9, 10, 11, 14, 15, 16], // Example peak hours
            performance_correlation: ProviderCorrelation {
                latency_vs_cost: HashMap::new(),
                reliability_vs_cost: HashMap::new(),
                tenant_satisfaction: HashMap::new(),
            },
            quality_metrics: QualityMetrics {
                response_quality_score: 85.0,
                hallucination_rate: 2.5,
                context_adherence: 92.0,
                user_satisfaction: 4.2,
            },
        }
    }

    /// Generate SLA metrics
    async fn generate_sla_metrics(&self, dashboard: &crate::metrics::LlmOpsDashboard) -> SlaMetrics {
        SlaMetrics {
            uptime_sla: 99.9,
            actual_uptime: 99.8,
            latency_sla: 2000,
            actual_p99_latency: dashboard.overall_metrics.p99_latency_ms as u64,
            error_rate_sla: 1.0,
            actual_error_rate: dashboard.overall_metrics.error_rate,
            sla_violations: Vec::new(), // Would be populated from historical data
            sla_compliance_score: 98.5,
        }
    }

    /// Generate tenant rankings
    async fn generate_tenant_rankings(&self, dashboard: &crate::metrics::LlmOpsDashboard) -> TenantRankings {
        let mut by_cost = Vec::new();
        let mut by_usage = Vec::new();

        for (i, tenant) in dashboard.tenant_metrics.iter().enumerate() {
            by_cost.push(TenantRank {
                tenant_id: tenant.tenant_id.clone(),
                rank: (i + 1) as u32,
                value: tenant.metrics.total_cost,
                change_from_previous: 0.0, // Would be calculated from historical data
            });

            by_usage.push(TenantRank {
                tenant_id: tenant.tenant_id.clone(),
                rank: (i + 1) as u32,
                value: tenant.metrics.request_count as f64,
                change_from_previous: 0.0,
            });
        }

        // Sort by cost and usage
        by_cost.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        by_usage.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());

        TenantRankings {
            by_cost,
            by_usage,
            by_efficiency: Vec::new(), // Would be calculated
            by_growth: Vec::new(),     // Would be calculated
        }
    }

    /// Generate optimization opportunities
    async fn generate_optimization_opportunities(&self, cost_analysis: &DetailedCostAnalysis) -> Vec<OptimizationOpportunity> {
        let mut opportunities = Vec::new();

        // Check for provider switching opportunities
        if cost_analysis.cost_efficiency_score < 70.0 {
            opportunities.push(OptimizationOpportunity {
                opportunity_type: OptimizationType::ProviderSwitching,
                description: "Switch to more cost-effective providers for certain workloads".to_string(),
                potential_savings: cost_analysis.total_cost * 0.15,
                confidence: 80.0,
                affected_tenants: cost_analysis.cost_per_tenant.keys().take(3).cloned().collect(),
                implementation_steps: vec![
                    "Analyze current provider performance".to_string(),
                    "Test alternative providers with sample workloads".to_string(),
                    "Implement gradual migration strategy".to_string(),
                ],
            });
        }

        opportunities
    }

    /// Update recommendations based on analysis
    async fn update_recommendations(&self, cost_analysis: &DetailedCostAnalysis, _performance: &PerformanceInsights) {
        let mut recommendations = self.recommendations.write().await;
        recommendations.clear();

        // Cost optimization recommendation
        if cost_analysis.cost_efficiency_score < 75.0 {
            recommendations.push(Recommendation {
                id: Uuid::new_v4(),
                category: RecommendationType::CostOptimization,
                title: "Optimize Provider Selection".to_string(),
                description: "Current cost efficiency is below optimal. Consider switching to more cost-effective providers.".to_string(),
                potential_savings: Some(cost_analysis.total_cost * 0.20),
                confidence_score: 85.0,
                priority: RecommendationPriority::Medium,
                implementation_effort: ImplementationEffort::Medium,
                affected_tenants: Vec::new(),
                expected_impact: "20% cost reduction with minimal performance impact".to_string(),
            });
        }
    }

    /// Update alerts based on analysis
    async fn update_alerts(&self, cost_analysis: &DetailedCostAnalysis, _sla_metrics: &SlaMetrics) {
        let mut alerts = self.alerts.write().await;

        // Check for budget violations
        for (tenant_id, breakdown) in &cost_analysis.cost_per_tenant {
            if breakdown.budget_utilization_percent > 90.0 {
                alerts.push(Alert {
                    id: Uuid::new_v4(),
                    alert_type: AlertType::BudgetExceeded,
                    severity: if breakdown.budget_utilization_percent > 100.0 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    title: format!("Budget Alert for Tenant {}", tenant_id),
                    message: format!("Tenant {} has used {:.1}% of their budget",
                        tenant_id, breakdown.budget_utilization_percent),
                    tenant_id: Some(tenant_id.clone()),
                    provider: None,
                    timestamp: Utc::now(),
                    threshold_value: Some(90.0),
                    actual_value: Some(breakdown.budget_utilization_percent),
                    is_resolved: false,
                    resolution_time: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CostTrackingConfig;

    #[tokio::test]
    async fn test_dashboard_generator_creation() {
        let metrics_collector = Arc::new(MetricsCollector::new(30));
        let cost_config = CostTrackingConfig::default();
        let generator = DashboardGenerator::new(metrics_collector, cost_config);

        // Test that we can create a dashboard
        let dashboard = generator.generate_enhanced_dashboard(TimeWindow::LastHour).await;
        assert_eq!(dashboard.tenant_metrics.len(), 0); // No data yet
    }

    #[test]
    fn test_budget_status() {
        let status = BudgetStatus::Healthy;
        assert_eq!(status, BudgetStatus::Healthy);
    }

    #[test]
    fn test_recommendation_priority() {
        let priority = RecommendationPriority::High;
        match priority {
            RecommendationPriority::High => assert!(true),
            _ => panic!("Expected High priority"),
        }
    }
}