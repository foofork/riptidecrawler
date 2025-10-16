//! LLM Operations metrics and dashboards
//!
//! This module provides comprehensive metrics collection and dashboard generation for:
//! - Latency tracking per provider and tenant
//! - Error rate monitoring
//! - Cost tracking and spend analysis
//! - Request volume and throughput
//! - Provider performance comparison

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{CompletionRequest, CompletionResponse, Cost, Usage};

/// Time window for aggregating metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeWindow {
    Last5Minutes,
    Last15Minutes,
    LastHour,
    Last6Hours,
    Last24Hours,
    Last7Days,
    Last30Days,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// Metric data point with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Request metrics for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub request_id: Uuid,
    pub tenant_id: Option<String>,
    pub provider_name: String,
    pub model_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub usage: Option<Usage>,
    pub cost: Option<Cost>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl RequestMetrics {
    pub fn new(
        request: &CompletionRequest,
        provider_name: &str,
        tenant_id: Option<String>,
    ) -> Self {
        Self {
            request_id: request.id,
            tenant_id,
            provider_name: provider_name.to_string(),
            model_name: request.model.clone(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            success: false,
            error_type: None,
            error_message: None,
            usage: None,
            cost: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }

    pub fn complete_success(&mut self, response: &CompletionResponse, cost: Option<Cost>) {
        self.end_time = Some(Utc::now());
        self.duration_ms =
            Some((self.end_time.unwrap() - self.start_time).num_milliseconds() as u64);
        self.success = true;
        self.usage = Some(response.usage.clone());
        self.cost = cost;
        self.prompt_tokens = response.usage.prompt_tokens;
        self.completion_tokens = response.usage.completion_tokens;
        self.total_tokens = response.usage.total_tokens;
    }

    pub fn complete_error(&mut self, error_type: &str, error_message: &str) {
        self.end_time = Some(Utc::now());
        self.duration_ms =
            Some((self.end_time.unwrap() - self.start_time).num_milliseconds() as u64);
        self.success = false;
        self.error_type = Some(error_type.to_string());
        self.error_message = Some(error_message.to_string());
    }
}

/// Aggregated metrics for a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub time_window: TimeWindow,
    pub request_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: u64,
    pub min_latency_ms: u64,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_cost: f64,
    pub avg_cost_per_request: f64,
    pub requests_per_minute: f64,
    pub tokens_per_second: f64,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            time_window: TimeWindow::LastHour,
            request_count: 0,
            success_count: 0,
            error_count: 0,
            success_rate: 100.0,
            error_rate: 0.0,
            avg_latency_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            max_latency_ms: 0,
            min_latency_ms: 0,
            total_tokens: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_cost: 0.0,
            avg_cost_per_request: 0.0,
            requests_per_minute: 0.0,
            tokens_per_second: 0.0,
        }
    }
}

/// Metrics grouped by provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub provider_name: String,
    pub metrics: AggregatedMetrics,
    pub model_breakdown: HashMap<String, AggregatedMetrics>,
}

/// Metrics grouped by tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMetrics {
    pub tenant_id: String,
    pub metrics: AggregatedMetrics,
    pub provider_breakdown: HashMap<String, AggregatedMetrics>,
    pub model_breakdown: HashMap<String, AggregatedMetrics>,
}

/// Error breakdown by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBreakdown {
    pub error_type: String,
    pub count: u64,
    pub percentage: f64,
    pub avg_latency_ms: f64,
    pub providers_affected: Vec<String>,
}

/// Cost breakdown analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub total_cost: f64,
    pub currency: String,
    pub by_provider: HashMap<String, f64>,
    pub by_tenant: HashMap<String, f64>,
    pub by_model: HashMap<String, f64>,
    pub prompt_cost: f64,
    pub completion_cost: f64,
}

/// LLM Operations dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmOpsDashboard {
    pub generated_at: DateTime<Utc>,
    pub time_window: TimeWindow,
    pub overall_metrics: AggregatedMetrics,
    pub provider_metrics: Vec<ProviderMetrics>,
    pub tenant_metrics: Vec<TenantMetrics>,
    pub error_breakdown: Vec<ErrorBreakdown>,
    pub cost_breakdown: CostBreakdown,
    pub top_models: Vec<(String, u64)>, // (model_name, request_count)
    pub latency_percentiles: Vec<MetricPoint>,
    pub throughput_over_time: Vec<MetricPoint>,
    pub error_rate_over_time: Vec<MetricPoint>,
    pub cost_over_time: Vec<MetricPoint>,
}

/// Metrics collector and analyzer
pub struct MetricsCollector {
    request_metrics: Arc<RwLock<Vec<RequestMetrics>>>,
    max_retention_days: u32,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(max_retention_days: u32) -> Self {
        Self {
            request_metrics: Arc::new(RwLock::new(Vec::new())),
            max_retention_days,
        }
    }

    /// Record the start of a request
    pub async fn start_request(
        &self,
        request: &CompletionRequest,
        provider_name: &str,
        tenant_id: Option<String>,
    ) -> Uuid {
        let metrics = RequestMetrics::new(request, provider_name, tenant_id);
        let request_id = metrics.request_id;

        let mut metrics_store = self.request_metrics.write().await;
        metrics_store.push(metrics);

        request_id
    }

    /// Record successful completion of a request
    pub async fn complete_request_success(
        &self,
        request_id: Uuid,
        response: &CompletionResponse,
        cost: Option<Cost>,
    ) {
        let mut metrics_store = self.request_metrics.write().await;
        if let Some(metrics) = metrics_store
            .iter_mut()
            .find(|m| m.request_id == request_id)
        {
            metrics.complete_success(response, cost);
        }
    }

    /// Record failed completion of a request
    pub async fn complete_request_error(
        &self,
        request_id: Uuid,
        error_type: &str,
        error_message: &str,
    ) {
        let mut metrics_store = self.request_metrics.write().await;
        if let Some(metrics) = metrics_store
            .iter_mut()
            .find(|m| m.request_id == request_id)
        {
            metrics.complete_error(error_type, error_message);
        }
    }

    /// Generate aggregated metrics for a time window
    pub async fn get_aggregated_metrics(&self, time_window: TimeWindow) -> AggregatedMetrics {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        self.calculate_aggregated_metrics(&filtered_metrics, time_window)
    }

    /// Generate metrics by provider
    pub async fn get_provider_metrics(&self, time_window: TimeWindow) -> Vec<ProviderMetrics> {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        let mut provider_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
        for metric in &filtered_metrics {
            provider_groups
                .entry(metric.provider_name.clone())
                .or_default()
                .push(metric);
        }

        let mut result = Vec::new();
        for (provider_name, provider_metrics) in provider_groups {
            let aggregated =
                self.calculate_aggregated_metrics(&provider_metrics, time_window.clone());

            // Group by model
            let mut model_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
            for metric in &provider_metrics {
                model_groups
                    .entry(metric.model_name.clone())
                    .or_default()
                    .push(metric);
            }

            let mut model_breakdown = HashMap::new();
            for (model_name, model_metrics) in model_groups {
                let model_aggregated =
                    self.calculate_aggregated_metrics(&model_metrics, time_window.clone());
                model_breakdown.insert(model_name, model_aggregated);
            }

            result.push(ProviderMetrics {
                provider_name,
                metrics: aggregated,
                model_breakdown,
            });
        }

        result
    }

    /// Generate metrics by tenant
    pub async fn get_tenant_metrics(&self, time_window: TimeWindow) -> Vec<TenantMetrics> {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        let mut tenant_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
        for metric in &filtered_metrics {
            if let Some(tenant_id) = &metric.tenant_id {
                tenant_groups
                    .entry(tenant_id.clone())
                    .or_default()
                    .push(metric);
            }
        }

        let mut result = Vec::new();
        for (tenant_id, tenant_metrics) in tenant_groups {
            let aggregated =
                self.calculate_aggregated_metrics(&tenant_metrics, time_window.clone());

            // Group by provider
            let mut provider_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
            for metric in &tenant_metrics {
                provider_groups
                    .entry(metric.provider_name.clone())
                    .or_default()
                    .push(metric);
            }

            let mut provider_breakdown = HashMap::new();
            for (provider_name, provider_metrics) in provider_groups {
                let provider_aggregated =
                    self.calculate_aggregated_metrics(&provider_metrics, time_window.clone());
                provider_breakdown.insert(provider_name, provider_aggregated);
            }

            // Group by model
            let mut model_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
            for metric in &tenant_metrics {
                model_groups
                    .entry(metric.model_name.clone())
                    .or_default()
                    .push(metric);
            }

            let mut model_breakdown = HashMap::new();
            for (model_name, model_metrics) in model_groups {
                let model_aggregated =
                    self.calculate_aggregated_metrics(&model_metrics, time_window.clone());
                model_breakdown.insert(model_name, model_aggregated);
            }

            result.push(TenantMetrics {
                tenant_id,
                metrics: aggregated,
                provider_breakdown,
                model_breakdown,
            });
        }

        result
    }

    /// Generate error breakdown
    pub async fn get_error_breakdown(&self, time_window: TimeWindow) -> Vec<ErrorBreakdown> {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        let failed_metrics: Vec<_> = filtered_metrics
            .iter()
            .filter(|m| !m.success && m.error_type.is_some())
            .collect();

        let total_errors = failed_metrics.len() as u64;
        if total_errors == 0 {
            return Vec::new();
        }

        let mut error_groups: HashMap<String, Vec<&RequestMetrics>> = HashMap::new();
        for metric in failed_metrics {
            if let Some(error_type) = &metric.error_type {
                error_groups
                    .entry(error_type.clone())
                    .or_default()
                    .push(metric);
            }
        }

        let mut result = Vec::new();
        for (error_type, error_metrics) in error_groups {
            let count = error_metrics.len() as u64;
            let percentage = (count as f64 / total_errors as f64) * 100.0;

            let avg_latency_ms = error_metrics
                .iter()
                .filter_map(|m| m.duration_ms)
                .map(|d| d as f64)
                .sum::<f64>()
                / error_metrics.len().max(1) as f64;

            let providers_affected: Vec<String> = error_metrics
                .iter()
                .map(|m| m.provider_name.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            result.push(ErrorBreakdown {
                error_type,
                count,
                percentage,
                avg_latency_ms,
                providers_affected,
            });
        }

        // Sort by count descending
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result
    }

    /// Generate cost breakdown
    pub async fn get_cost_breakdown(&self, time_window: TimeWindow) -> CostBreakdown {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        let mut total_cost = 0.0;
        let mut currency = "USD".to_string();
        let mut by_provider: HashMap<String, f64> = HashMap::new();
        let mut by_tenant: HashMap<String, f64> = HashMap::new();
        let mut by_model: HashMap<String, f64> = HashMap::new();
        let mut prompt_cost = 0.0;
        let mut completion_cost = 0.0;

        for metric in filtered_metrics {
            if let Some(cost) = &metric.cost {
                total_cost += cost.total_cost;
                currency = cost.currency.clone();
                prompt_cost += cost.prompt_cost;
                completion_cost += cost.completion_cost;

                *by_provider
                    .entry(metric.provider_name.clone())
                    .or_insert(0.0) += cost.total_cost;
                *by_model.entry(metric.model_name.clone()).or_insert(0.0) += cost.total_cost;

                if let Some(tenant_id) = &metric.tenant_id {
                    *by_tenant.entry(tenant_id.clone()).or_insert(0.0) += cost.total_cost;
                }
            }
        }

        CostBreakdown {
            total_cost,
            currency,
            by_provider,
            by_tenant,
            by_model,
            prompt_cost,
            completion_cost,
        }
    }

    /// Generate complete LLM Ops dashboard
    pub async fn generate_dashboard(&self, time_window: TimeWindow) -> LlmOpsDashboard {
        let overall_metrics = self.get_aggregated_metrics(time_window.clone()).await;
        let provider_metrics = self.get_provider_metrics(time_window.clone()).await;
        let tenant_metrics = self.get_tenant_metrics(time_window.clone()).await;
        let error_breakdown = self.get_error_breakdown(time_window.clone()).await;
        let cost_breakdown = self.get_cost_breakdown(time_window.clone()).await;

        // Get top models by request count
        let top_models = self.get_top_models(time_window.clone()).await;

        // Generate time series data
        let latency_percentiles = self.generate_latency_time_series(time_window.clone()).await;
        let throughput_over_time = self
            .generate_throughput_time_series(time_window.clone())
            .await;
        let error_rate_over_time = self
            .generate_error_rate_time_series(time_window.clone())
            .await;
        let cost_over_time = self.generate_cost_time_series(time_window.clone()).await;

        LlmOpsDashboard {
            generated_at: Utc::now(),
            time_window,
            overall_metrics,
            provider_metrics,
            tenant_metrics,
            error_breakdown,
            cost_breakdown,
            top_models,
            latency_percentiles,
            throughput_over_time,
            error_rate_over_time,
            cost_over_time,
        }
    }

    /// Clean up old metrics beyond retention period
    pub async fn cleanup_old_metrics(&self) {
        let cutoff_time = Utc::now() - chrono::Duration::days(self.max_retention_days as i64);
        let mut metrics_store = self.request_metrics.write().await;
        metrics_store.retain(|m| m.start_time > cutoff_time);
    }

    /// Filter metrics by time window
    fn filter_metrics_by_window<'a>(
        &self,
        metrics: &'a [RequestMetrics],
        time_window: &TimeWindow,
    ) -> Vec<&'a RequestMetrics> {
        let (start_time, end_time) = self.get_time_range(time_window);

        metrics
            .iter()
            .filter(|m| m.start_time >= start_time && m.start_time <= end_time)
            .collect()
    }

    /// Get time range for a time window
    fn get_time_range(&self, time_window: &TimeWindow) -> (DateTime<Utc>, DateTime<Utc>) {
        let now = Utc::now();
        match time_window {
            TimeWindow::Last5Minutes => (now - chrono::Duration::minutes(5), now),
            TimeWindow::Last15Minutes => (now - chrono::Duration::minutes(15), now),
            TimeWindow::LastHour => (now - chrono::Duration::hours(1), now),
            TimeWindow::Last6Hours => (now - chrono::Duration::hours(6), now),
            TimeWindow::Last24Hours => (now - chrono::Duration::hours(24), now),
            TimeWindow::Last7Days => (now - chrono::Duration::days(7), now),
            TimeWindow::Last30Days => (now - chrono::Duration::days(30), now),
            TimeWindow::Custom { start, end } => (*start, *end),
        }
    }

    /// Calculate aggregated metrics from a collection of request metrics
    fn calculate_aggregated_metrics(
        &self,
        metrics: &[&RequestMetrics],
        time_window: TimeWindow,
    ) -> AggregatedMetrics {
        if metrics.is_empty() {
            return AggregatedMetrics {
                time_window,
                ..Default::default()
            };
        }

        let request_count = metrics.len() as u64;
        let success_count = metrics.iter().filter(|m| m.success).count() as u64;
        let error_count = request_count - success_count;

        let success_rate = if request_count > 0 {
            (success_count as f64 / request_count as f64) * 100.0
        } else {
            100.0
        };
        let error_rate = 100.0 - success_rate;

        // Calculate latency percentiles
        let mut latencies: Vec<u64> = metrics.iter().filter_map(|m| m.duration_ms).collect();
        latencies.sort_unstable();

        let (
            avg_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            min_latency_ms,
        ) = if latencies.is_empty() {
            (0.0, 0.0, 0.0, 0.0, 0, 0)
        } else {
            let avg = latencies.iter().sum::<u64>() as f64 / latencies.len() as f64;
            let p50 = latencies[latencies.len() * 50 / 100] as f64;
            let p95 = latencies[latencies.len() * 95 / 100] as f64;
            let p99 = latencies[latencies.len() * 99 / 100] as f64;
            let max = *latencies.last().unwrap();
            let min = *latencies.first().unwrap();
            (avg, p50, p95, p99, max, min)
        };

        // Calculate token and cost metrics
        let total_tokens = metrics.iter().map(|m| m.total_tokens as u64).sum();
        let prompt_tokens = metrics.iter().map(|m| m.prompt_tokens as u64).sum();
        let completion_tokens = metrics.iter().map(|m| m.completion_tokens as u64).sum();

        let total_cost = metrics
            .iter()
            .filter_map(|m| m.cost.as_ref().map(|c| c.total_cost))
            .sum::<f64>();

        let avg_cost_per_request = if request_count > 0 {
            total_cost / request_count as f64
        } else {
            0.0
        };

        // Calculate time-based rates
        let (start_time, end_time) = self.get_time_range(&time_window);
        let duration_minutes = (end_time - start_time).num_minutes().max(1) as f64;
        let duration_seconds = duration_minutes * 60.0;

        let requests_per_minute = request_count as f64 / duration_minutes;
        let tokens_per_second = total_tokens as f64 / duration_seconds;

        AggregatedMetrics {
            time_window,
            request_count,
            success_count,
            error_count,
            success_rate,
            error_rate,
            avg_latency_ms,
            p50_latency_ms,
            p95_latency_ms,
            p99_latency_ms,
            max_latency_ms,
            min_latency_ms,
            total_tokens,
            prompt_tokens,
            completion_tokens,
            total_cost,
            avg_cost_per_request,
            requests_per_minute,
            tokens_per_second,
        }
    }

    /// Get top models by request count
    async fn get_top_models(&self, time_window: TimeWindow) -> Vec<(String, u64)> {
        let metrics_store = self.request_metrics.read().await;
        let filtered_metrics = self.filter_metrics_by_window(&metrics_store, &time_window);

        let mut model_counts: HashMap<String, u64> = HashMap::new();
        for metric in filtered_metrics {
            *model_counts.entry(metric.model_name.clone()).or_insert(0) += 1;
        }

        let mut result: Vec<_> = model_counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10); // Top 10 models
        result
    }

    /// Generate latency percentile time series
    async fn generate_latency_time_series(&self, _time_window: TimeWindow) -> Vec<MetricPoint> {
        // Implementation would generate time series data points
        // For now, return empty vec
        Vec::new()
    }

    /// Generate throughput time series
    async fn generate_throughput_time_series(&self, _time_window: TimeWindow) -> Vec<MetricPoint> {
        // Implementation would generate time series data points
        // For now, return empty vec
        Vec::new()
    }

    /// Generate error rate time series
    async fn generate_error_rate_time_series(&self, _time_window: TimeWindow) -> Vec<MetricPoint> {
        // Implementation would generate time series data points
        // For now, return empty vec
        Vec::new()
    }

    /// Generate cost time series
    async fn generate_cost_time_series(&self, _time_window: TimeWindow) -> Vec<MetricPoint> {
        // Implementation would generate time series data points
        // For now, return empty vec
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new(30);
        let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
        assert_eq!(metrics.request_count, 0);
    }

    #[tokio::test]
    async fn test_request_metrics_lifecycle() {
        let collector = MetricsCollector::new(30);

        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);

        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        let response = CompletionResponse::new(
            request.id,
            "test response",
            "gpt-4",
            Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        );

        let cost = Cost::new(0.01, 0.02, "USD");
        collector
            .complete_request_success(request_id, &response, Some(cost))
            .await;

        let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.total_tokens, 30);
    }

    #[tokio::test]
    async fn test_request_error_lifecycle() {
        let collector = MetricsCollector::new(30);

        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        collector
            .complete_request_error(request_id, "RateLimitError", "Rate limit exceeded")
            .await;

        let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
        assert_eq!(metrics.request_count, 1);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_provider_breakdown() {
        let collector = MetricsCollector::new(30);

        // OpenAI requests
        for _ in 0..3 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = collector
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;

            let response = CompletionResponse::new(
                request.id,
                "response",
                "gpt-4",
                Usage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
            );

            collector
                .complete_request_success(request_id, &response, None)
                .await;
        }

        // Anthropic requests
        for _ in 0..2 {
            let request =
                CompletionRequest::new("claude-3".to_string(), vec![Message::user("test")]);
            let request_id = collector
                .start_request(&request, "anthropic", Some("tenant1".to_string()))
                .await;

            let response = CompletionResponse::new(
                request.id,
                "response",
                "claude-3",
                Usage {
                    prompt_tokens: 15,
                    completion_tokens: 25,
                    total_tokens: 40,
                },
            );

            collector
                .complete_request_success(request_id, &response, None)
                .await;
        }

        let provider_metrics = collector.get_provider_metrics(TimeWindow::LastHour).await;
        assert_eq!(provider_metrics.len(), 2);

        let openai = provider_metrics
            .iter()
            .find(|p| p.provider_name == "openai")
            .unwrap();
        assert_eq!(openai.metrics.request_count, 3);
    }

    #[tokio::test]
    async fn test_error_breakdown() {
        let collector = MetricsCollector::new(30);

        // Different error types
        for _ in 0..3 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = collector
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;
            collector
                .complete_request_error(request_id, "RateLimitError", "Rate limit")
                .await;
        }

        for _ in 0..2 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = collector
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;
            collector
                .complete_request_error(request_id, "APIError", "API error")
                .await;
        }

        let error_breakdown = collector.get_error_breakdown(TimeWindow::LastHour).await;
        assert_eq!(error_breakdown.len(), 2);
        assert_eq!(error_breakdown[0].count, 3); // Sorted by count
    }

    #[tokio::test]
    async fn test_cost_breakdown() {
        let collector = MetricsCollector::new(30);

        for _ in 0..5 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = collector
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;

            let response = CompletionResponse::new(
                request.id,
                "response",
                "gpt-4",
                Usage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                },
            );

            collector
                .complete_request_success(request_id, &response, Some(Cost::new(0.01, 0.02, "USD")))
                .await;
        }

        let cost_breakdown = collector.get_cost_breakdown(TimeWindow::LastHour).await;
        assert_eq!(cost_breakdown.total_cost, 0.15); // 5 * 0.03
        assert_eq!(cost_breakdown.currency, "USD");
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        use std::sync::Arc;

        let collector = Arc::new(MetricsCollector::new(30));
        let mut handles = vec![];

        for _ in 0..5 {
            let c = Arc::clone(&collector);
            let handle = tokio::spawn(async move {
                for _ in 0..20 {
                    let request =
                        CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
                    let request_id = c
                        .start_request(&request, "openai", Some("tenant1".to_string()))
                        .await;

                    let response = CompletionResponse::new(
                        request.id,
                        "response",
                        "gpt-4",
                        Usage {
                            prompt_tokens: 10,
                            completion_tokens: 20,
                            total_tokens: 30,
                        },
                    );

                    c.complete_request_success(request_id, &response, None)
                        .await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
        assert_eq!(metrics.request_count, 100); // 5 * 20
    }

    #[tokio::test]
    async fn test_time_window_filtering() {
        let collector = MetricsCollector::new(30);

        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        let response = CompletionResponse::new(
            request.id,
            "response",
            "gpt-4",
            Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        );

        collector
            .complete_request_success(request_id, &response, None)
            .await;

        // Should be visible in different time windows
        let metrics_hour = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
        assert_eq!(metrics_hour.request_count, 1);

        let metrics_day = collector
            .get_aggregated_metrics(TimeWindow::Last24Hours)
            .await;
        assert_eq!(metrics_day.request_count, 1);
    }
}
