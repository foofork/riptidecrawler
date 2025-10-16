//! Comprehensive test suite for Intelligence/LLM metrics collection system
//!
//! Tests cover:
//! - Request lifecycle tracking (start, success, error)
//! - Aggregated metrics calculation
//! - Provider and tenant breakdown
//! - Error breakdown analysis
//! - Cost tracking and breakdown
//! - Time window filtering
//! - Percentile calculations (p50, p95, p99)
//! - Concurrent access patterns
//! - Edge cases and data validation
//! - Dashboard generation

use chrono::{Duration as ChronoDuration, Utc};
use riptide_intelligence::metrics::{
    AggregatedMetrics, MetricsCollector, RequestMetrics, TimeWindow,
};
use riptide_intelligence::{CompletionRequest, CompletionResponse, Cost, Message, Usage};
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_collector_creation() {
    let collector = MetricsCollector::new(30);
    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    assert_eq!(metrics.request_count, 0);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.success_rate, 100.0); // Default when no requests
}

#[tokio::test]
async fn test_single_request_success_lifecycle() {
    let collector = MetricsCollector::new(30);

    let request = CompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message::user("test message")],
    );

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
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.total_tokens, 30);
    assert_eq!(metrics.prompt_tokens, 10);
    assert_eq!(metrics.completion_tokens, 20);
    assert_eq!(metrics.total_cost, 0.03); // 0.01 + 0.02
    assert_eq!(metrics.success_rate, 100.0);
}

#[tokio::test]
async fn test_single_request_error_lifecycle() {
    let collector = MetricsCollector::new(30);

    let request = CompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message::user("test message")],
    );

    let request_id = collector
        .start_request(&request, "openai", Some("tenant1".to_string()))
        .await;

    collector
        .complete_request_error(request_id, "RateLimitError", "Rate limit exceeded")
        .await;

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    assert_eq!(metrics.request_count, 1);
    assert_eq!(metrics.success_count, 0);
    assert_eq!(metrics.error_count, 1);
    assert_eq!(metrics.success_rate, 0.0);
    assert_eq!(metrics.error_rate, 100.0);
}

#[tokio::test]
async fn test_multiple_requests_mixed() {
    let collector = MetricsCollector::new(30);

    // 3 successful requests
    for i in 0..3 {
        let request = CompletionRequest::new(
            "gpt-4".to_string(),
            vec![Message::user(&format!("test {}", i))],
        );

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

    // 2 failed requests
    for i in 0..2 {
        let request = CompletionRequest::new(
            "gpt-4".to_string(),
            vec![Message::user(&format!("test {}", i))],
        );

        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        collector
            .complete_request_error(request_id, "APIError", "API error")
            .await;
    }

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    assert_eq!(metrics.request_count, 5);
    assert_eq!(metrics.success_count, 3);
    assert_eq!(metrics.error_count, 2);
    assert_eq!(metrics.success_rate, 60.0); // 3/5 * 100
    assert_eq!(metrics.error_rate, 40.0);
    assert_eq!(metrics.total_tokens, 90); // 3 * 30
}

#[tokio::test]
async fn test_provider_metrics_breakdown() {
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
            .complete_request_success(request_id, &response, Some(Cost::new(0.01, 0.02, "USD")))
            .await;
    }

    // Anthropic requests
    for _ in 0..2 {
        let request = CompletionRequest::new("claude-3".to_string(), vec![Message::user("test")]);
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
            .complete_request_success(request_id, &response, Some(Cost::new(0.015, 0.025, "USD")))
            .await;
    }

    let provider_metrics = collector.get_provider_metrics(TimeWindow::LastHour).await;

    assert_eq!(provider_metrics.len(), 2);

    // Find OpenAI metrics
    let openai = provider_metrics
        .iter()
        .find(|p| p.provider_name == "openai")
        .unwrap();
    assert_eq!(openai.metrics.request_count, 3);
    assert_eq!(openai.metrics.total_tokens, 90); // 3 * 30

    // Find Anthropic metrics
    let anthropic = provider_metrics
        .iter()
        .find(|p| p.provider_name == "anthropic")
        .unwrap();
    assert_eq!(anthropic.metrics.request_count, 2);
    assert_eq!(anthropic.metrics.total_tokens, 80); // 2 * 40
}

#[tokio::test]
async fn test_tenant_metrics_breakdown() {
    let collector = MetricsCollector::new(30);

    // Tenant 1 requests
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
            .complete_request_success(request_id, &response, Some(Cost::new(0.01, 0.02, "USD")))
            .await;
    }

    // Tenant 2 requests
    for _ in 0..2 {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant2".to_string()))
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

    let tenant_metrics = collector.get_tenant_metrics(TimeWindow::LastHour).await;

    assert_eq!(tenant_metrics.len(), 2);

    let tenant1 = tenant_metrics
        .iter()
        .find(|t| t.tenant_id == "tenant1")
        .unwrap();
    assert_eq!(tenant1.metrics.request_count, 3);

    let tenant2 = tenant_metrics
        .iter()
        .find(|t| t.tenant_id == "tenant2")
        .unwrap();
    assert_eq!(tenant2.metrics.request_count, 2);
}

#[tokio::test]
async fn test_error_breakdown() {
    let collector = MetricsCollector::new(30);

    // Create requests with different error types
    for _ in 0..3 {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;
        collector
            .complete_request_error(request_id, "RateLimitError", "Rate limit exceeded")
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

    for _ in 0..1 {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;
        collector
            .complete_request_error(request_id, "TimeoutError", "Request timeout")
            .await;
    }

    let error_breakdown = collector.get_error_breakdown(TimeWindow::LastHour).await;

    assert_eq!(error_breakdown.len(), 3);

    // Errors should be sorted by count descending
    assert_eq!(error_breakdown[0].error_type, "RateLimitError");
    assert_eq!(error_breakdown[0].count, 3);
    assert_eq!(error_breakdown[0].percentage, 50.0); // 3/6 * 100

    assert_eq!(error_breakdown[1].error_type, "APIError");
    assert_eq!(error_breakdown[1].count, 2);
    assert!((error_breakdown[1].percentage - 33.33).abs() < 0.1);

    assert_eq!(error_breakdown[2].error_type, "TimeoutError");
    assert_eq!(error_breakdown[2].count, 1);
    assert!((error_breakdown[2].percentage - 16.67).abs() < 0.1);
}

#[tokio::test]
async fn test_cost_breakdown() {
    let collector = MetricsCollector::new(30);

    // OpenAI requests with cost
    for _ in 0..2 {
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

    // Anthropic requests with cost
    for _ in 0..1 {
        let request = CompletionRequest::new("claude-3".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "anthropic", Some("tenant2".to_string()))
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
            .complete_request_success(request_id, &response, Some(Cost::new(0.015, 0.025, "USD")))
            .await;
    }

    let cost_breakdown = collector.get_cost_breakdown(TimeWindow::LastHour).await;

    assert_eq!(cost_breakdown.total_cost, 0.10); // (2 * 0.03) + 0.04
    assert_eq!(cost_breakdown.currency, "USD");
    assert_eq!(cost_breakdown.prompt_cost, 0.035); // (2 * 0.01) + 0.015
    assert_eq!(cost_breakdown.completion_cost, 0.065); // (2 * 0.02) + 0.025

    assert_eq!(cost_breakdown.by_provider["openai"], 0.06); // 2 * 0.03
    assert_eq!(cost_breakdown.by_provider["anthropic"], 0.04);

    assert_eq!(cost_breakdown.by_tenant["tenant1"], 0.06);
    assert_eq!(cost_breakdown.by_tenant["tenant2"], 0.04);

    assert_eq!(cost_breakdown.by_model["gpt-4"], 0.06);
    assert_eq!(cost_breakdown.by_model["claude-3"], 0.04);
}

#[tokio::test]
async fn test_latency_percentiles() {
    let collector = MetricsCollector::new(30);

    // Create requests with known latencies
    let latencies = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];

    for latency_ms in latencies {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(latency_ms)).await;

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

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    // P50 should be around 500ms (median)
    assert!(metrics.p50_latency_ms >= 400.0 && metrics.p50_latency_ms <= 600.0);

    // P95 should be around 950ms
    assert!(metrics.p95_latency_ms >= 850.0 && metrics.p95_latency_ms <= 1050.0);

    // P99 should be around 990ms
    assert!(metrics.p99_latency_ms >= 900.0 && metrics.p99_latency_ms <= 1100.0);

    // Max should be >= 1000ms
    assert!(metrics.max_latency_ms >= 1000);
}

#[tokio::test]
async fn test_time_window_filtering() {
    let collector = MetricsCollector::new(30);

    // Create a request
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

    // Should be visible in LastHour window
    let metrics_hour = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
    assert_eq!(metrics_hour.request_count, 1);

    // Should be visible in Last24Hours window
    let metrics_day = collector.get_aggregated_metrics(TimeWindow::Last24Hours).await;
    assert_eq!(metrics_day.request_count, 1);

    // Custom time window (last 5 seconds)
    let now = Utc::now();
    let metrics_custom = collector
        .get_aggregated_metrics(TimeWindow::Custom {
            start: now - ChronoDuration::seconds(5),
            end: now,
        })
        .await;
    assert_eq!(metrics_custom.request_count, 1);

    // Custom time window (future - should be empty)
    let metrics_future = collector
        .get_aggregated_metrics(TimeWindow::Custom {
            start: now + ChronoDuration::hours(1),
            end: now + ChronoDuration::hours(2),
        })
        .await;
    assert_eq!(metrics_future.request_count, 0);
}

#[tokio::test]
async fn test_metrics_cleanup() {
    let collector = MetricsCollector::new(0); // 0 days retention

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

    // Verify metric exists
    let metrics_before = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
    assert_eq!(metrics_before.request_count, 1);

    // Wait a bit and cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    collector.cleanup_old_metrics().await;

    // For 0-day retention, nothing should be cleaned immediately
    // The metric is still within "today"
}

#[tokio::test]
async fn test_concurrent_request_recording() {
    let collector = Arc::new(MetricsCollector::new(30));
    let mut handles = vec![];

    // Spawn 10 tasks, each recording 10 successful requests
    for _ in 0..10 {
        let collector_clone = Arc::clone(&collector);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
                let request_id = collector_clone
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

                collector_clone
                    .complete_request_success(request_id, &response, None)
                    .await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
    assert_eq!(metrics.request_count, 100); // 10 tasks * 10 requests
    assert_eq!(metrics.success_count, 100);
    assert_eq!(metrics.total_tokens, 3000); // 100 * 30
}

#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let collector = Arc::new(MetricsCollector::new(30));
    let mut handles = vec![];

    // Task 1: Success recordings
    let c1 = Arc::clone(&collector);
    handles.push(tokio::spawn(async move {
        for _ in 0..20 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = c1
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

            c1.complete_request_success(request_id, &response, None)
                .await;
        }
    }));

    // Task 2: Error recordings
    let c2 = Arc::clone(&collector);
    handles.push(tokio::spawn(async move {
        for _ in 0..10 {
            let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
            let request_id = c2
                .start_request(&request, "openai", Some("tenant1".to_string()))
                .await;

            c2.complete_request_error(request_id, "APIError", "Error")
                .await;
        }
    }));

    // Task 3: Dashboard generation
    let c3 = Arc::clone(&collector);
    handles.push(tokio::spawn(async move {
        for _ in 0..5 {
            let _dashboard = c3.generate_dashboard(TimeWindow::LastHour).await;
        }
    }));

    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
    assert_eq!(metrics.request_count, 30); // 20 + 10
    assert_eq!(metrics.success_count, 20);
    assert_eq!(metrics.error_count, 10);
}

#[tokio::test]
async fn test_edge_case_no_requests() {
    let collector = MetricsCollector::new(30);
    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    assert_eq!(metrics.request_count, 0);
    assert_eq!(metrics.success_rate, 100.0); // Default
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.avg_latency_ms, 0.0);
    assert_eq!(metrics.total_cost, 0.0);
}

#[tokio::test]
async fn test_edge_case_all_errors() {
    let collector = MetricsCollector::new(30);

    for _ in 0..10 {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;
        collector
            .complete_request_error(request_id, "Error", "Error message")
            .await;
    }

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    assert_eq!(metrics.success_rate, 0.0);
    assert_eq!(metrics.error_rate, 100.0);
    assert_eq!(metrics.total_tokens, 0);
    assert_eq!(metrics.total_cost, 0.0);
}

#[tokio::test]
async fn test_dashboard_generation() {
    let collector = MetricsCollector::new(30);

    // Create some varied data
    for i in 0..5 {
        let request = CompletionRequest::new("gpt-4".to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some(format!("tenant{}", i % 2)))
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

    let dashboard = collector.generate_dashboard(TimeWindow::LastHour).await;

    assert_eq!(dashboard.overall_metrics.request_count, 5);
    assert!(!dashboard.provider_metrics.is_empty());
    assert!(!dashboard.tenant_metrics.is_empty());
    assert_eq!(dashboard.cost_breakdown.total_cost, 0.15); // 5 * 0.03
}

#[tokio::test]
async fn test_requests_per_minute_calculation() {
    let collector = MetricsCollector::new(30);

    // Create 60 requests
    for _ in 0..60 {
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

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    // For a 1-hour window, 60 requests = 1 request per minute
    assert_eq!(metrics.requests_per_minute, 1.0);
}

#[tokio::test]
async fn test_tokens_per_second_calculation() {
    let collector = MetricsCollector::new(30);

    // Create requests with total 3600 tokens
    for _ in 0..120 {
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

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    // 120 requests * 30 tokens = 3600 tokens
    // 1 hour = 3600 seconds
    // 3600 tokens / 3600 seconds = 1 token/sec
    assert_eq!(metrics.tokens_per_second, 1.0);
}

#[tokio::test]
async fn test_avg_cost_per_request() {
    let collector = MetricsCollector::new(30);

    for _ in 0..10 {
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

    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;

    // 10 requests * $0.03 = $0.30 total / 10 requests = $0.03 average
    assert_eq!(metrics.avg_cost_per_request, 0.03);
}

#[tokio::test]
async fn test_model_breakdown_in_provider_metrics() {
    let collector = MetricsCollector::new(30);

    // OpenAI with different models
    for model in &["gpt-4", "gpt-3.5-turbo", "gpt-4"] {
        let request = CompletionRequest::new(model.to_string(), vec![Message::user("test")]);
        let request_id = collector
            .start_request(&request, "openai", Some("tenant1".to_string()))
            .await;

        let response = CompletionResponse::new(
            request.id,
            "response",
            model,
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

    let provider_metrics = collector.get_provider_metrics(TimeWindow::LastHour).await;
    let openai = provider_metrics
        .iter()
        .find(|p| p.provider_name == "openai")
        .unwrap();

    assert_eq!(openai.model_breakdown.len(), 2); // gpt-4 and gpt-3.5-turbo
    assert_eq!(openai.model_breakdown["gpt-4"].request_count, 2);
    assert_eq!(openai.model_breakdown["gpt-3.5-turbo"].request_count, 1);
}

#[tokio::test]
async fn test_nonexistent_request_id() {
    let collector = MetricsCollector::new(30);

    let fake_id = uuid::Uuid::new_v4();

    let response = CompletionResponse::new(
        fake_id,
        "response",
        "gpt-4",
        Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
    );

    // Should not panic when completing nonexistent request
    collector
        .complete_request_success(fake_id, &response, None)
        .await;

    // Should not affect metrics
    let metrics = collector.get_aggregated_metrics(TimeWindow::LastHour).await;
    assert_eq!(metrics.request_count, 0);
}
