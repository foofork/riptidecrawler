//! Tests for API fallback logic
//!
//! Verifies graceful degradation when API is unavailable

use anyhow::Result;
use riptide_cli::api_client::RiptideApiClient;
use riptide_cli::execution_mode::ExecutionMode;
use std::env;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

#[test]
fn test_execution_mode_api_first() {
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiFirst);
    assert!(mode.allows_api());
    assert!(mode.allows_direct());
    assert!(mode.allows_fallback());
}

#[test]
fn test_execution_mode_api_only() {
    let mode = ExecutionMode::from_flags(false, true);
    assert_eq!(mode, ExecutionMode::ApiOnly);
    assert!(mode.allows_api());
    assert!(!mode.allows_direct());
    assert!(!mode.allows_fallback());
}

#[test]
fn test_execution_mode_direct_only() {
    let mode = ExecutionMode::from_flags(true, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    assert!(!mode.allows_api());
    assert!(mode.allows_direct());
    assert!(!mode.allows_fallback());
}

#[test]
fn test_direct_flag_precedence() {
    let mode = ExecutionMode::from_flags(true, true);
    assert_eq!(mode, ExecutionMode::DirectOnly);
}

#[test]
fn test_execution_mode_from_environment() {
    env::set_var("RIPTIDE_EXECUTION_MODE", "direct");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");

    env::set_var("RIPTIDE_EXECUTION_MODE", "api-only");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");

    env::set_var("RIPTIDE_EXECUTION_MODE", "api-first");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiFirst);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_execution_mode_descriptions() {
    assert_eq!(
        ExecutionMode::ApiFirst.description(),
        "API-first with fallback"
    );
    assert_eq!(
        ExecutionMode::ApiOnly.description(),
        "API-only (no fallback)"
    );
    assert_eq!(
        ExecutionMode::DirectOnly.description(),
        "Direct execution (offline)"
    );
}

#[tokio::test]
async fn test_fallback_on_api_unavailable() -> Result<()> {
    // Create client pointing to non-existent server
    let client = RiptideApiClient::new("http://localhost:65535".to_string(), None)?;

    // Check that API is not available
    let is_available = client.is_available().await;
    assert!(!is_available, "API should not be available");

    // In API-first mode, we should fallback to direct execution
    let mode = ExecutionMode::ApiFirst;
    assert!(mode.allows_fallback());

    Ok(())
}

#[tokio::test]
async fn test_no_fallback_in_api_only_mode() -> Result<()> {
    let mode = ExecutionMode::ApiOnly;
    assert!(!mode.allows_fallback());
    assert!(!mode.allows_direct());

    // In API-only mode, if API is unavailable, we should error
    // (not fallback to direct)
    Ok(())
}

#[tokio::test]
async fn test_fallback_workflow_simulation() -> Result<()> {
    let mock_server = MockServer::start().await;

    // First request succeeds
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Subsequent requests fail (simulating server going down)
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    // First check should succeed
    assert!(client.is_available().await);

    // Second check should fail
    assert!(!client.is_available().await);

    // With API-first mode, we should now fallback
    let mode = ExecutionMode::ApiFirst;
    if !client.is_available().await && mode.allows_fallback() {
        // Fallback to direct execution
        assert!(mode.allows_direct());
    }

    Ok(())
}

#[tokio::test]
async fn test_retry_logic_with_transient_errors() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Simulate transient error then success
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    // First attempts should fail
    assert!(!client.is_available().await);
    assert!(!client.is_available().await);

    // Third attempt should succeed
    assert!(client.is_available().await);

    Ok(())
}

#[test]
fn test_fallback_decision_matrix() {
    // Test decision matrix for when to fallback
    let scenarios = vec![
        // (api_available, mode, should_use_api, should_fallback)
        (true, ExecutionMode::ApiFirst, true, false),
        (false, ExecutionMode::ApiFirst, false, true),
        (true, ExecutionMode::ApiOnly, true, false),
        (false, ExecutionMode::ApiOnly, false, false),
        (true, ExecutionMode::DirectOnly, false, false),
        (false, ExecutionMode::DirectOnly, false, false),
    ];

    for (api_available, mode, should_use_api, should_fallback) in scenarios {
        if api_available && mode.allows_api() {
            assert_eq!(should_use_api, true);
        } else if !api_available && mode.allows_fallback() {
            assert_eq!(should_fallback, true);
        }
    }
}

#[tokio::test]
async fn test_connection_timeout_triggers_fallback() -> Result<()> {
    // Point to a non-routable IP to simulate timeout
    let client = RiptideApiClient::new("http://10.255.255.1:8080".to_string(), None)?;

    // This should timeout quickly (within 5 seconds per is_available implementation)
    let start = std::time::Instant::now();
    let is_available = client.is_available().await;
    let duration = start.elapsed();

    assert!(!is_available);
    assert!(
        duration.as_secs() <= 10,
        "Health check timeout took too long"
    );

    // Should trigger fallback in API-first mode
    let mode = ExecutionMode::ApiFirst;
    if !is_available && mode.allows_fallback() {
        assert!(mode.allows_direct());
    }

    Ok(())
}

#[test]
fn test_environment_variable_fallback_config() {
    // Test offline mode via environment
    env::set_var("RIPTIDE_EXECUTION_MODE", "offline");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");

    // Test API-only mode via environment
    env::set_var("RIPTIDE_EXECUTION_MODE", "api_only");
    let mode = ExecutionMode::from_flags(false, false);
    assert_eq!(mode, ExecutionMode::ApiOnly);
    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[test]
fn test_cli_flags_override_environment() {
    // Environment says API-only
    env::set_var("RIPTIDE_EXECUTION_MODE", "api-only");

    // But CLI flag says direct
    let mode = ExecutionMode::from_flags(true, false);
    assert_eq!(mode, ExecutionMode::DirectOnly);

    env::remove_var("RIPTIDE_EXECUTION_MODE");
}

#[tokio::test]
async fn test_gradual_degradation() -> Result<()> {
    let mock_server = MockServer::start().await;

    // Simulate degraded service
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503).set_delay(std::time::Duration::from_secs(6)))
        .mount(&mock_server)
        .await;

    let client = RiptideApiClient::new(mock_server.uri(), None)?;

    // Health check should timeout and return false
    let is_available = client.is_available().await;
    assert!(!is_available);

    Ok(())
}

#[test]
fn test_offline_mode_detection() {
    // Simulate offline detection
    let mode = ExecutionMode::DirectOnly;

    // In offline mode, don't even attempt API calls
    assert!(!mode.allows_api());
    assert!(mode.allows_direct());
}

#[test]
fn test_fallback_strategy_consistency() {
    // Ensure fallback behavior is consistent across multiple checks
    let mode = ExecutionMode::ApiFirst;

    for _ in 0..10 {
        assert!(mode.allows_fallback());
        assert!(mode.allows_api());
        assert!(mode.allows_direct());
    }
}
