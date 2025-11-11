//! Common test utilities for facade integration tests
//!
//! This module provides helper functions for creating test orchestrators
//! without requiring direct dependency on riptide-api in non-test code.

use riptide_api::health::HealthChecker;
use riptide_api::pipeline::PipelineOrchestrator;
use riptide_api::state::{AppConfig, AppState};
use riptide_api::strategies_pipeline::StrategiesPipelineOrchestrator;
use riptide_types::config::CrawlOptions;
use std::sync::Arc;

/// Create a minimal AppState for testing
///
/// This is used to create orchestrators in integration tests.
/// Only use in test code.
pub async fn create_test_state() -> AppState {
    let config = AppConfig::default();
    let health_checker = Arc::new(HealthChecker::new());
    AppState::new(config, health_checker)
        .await
        .expect("Failed to create test AppState")
}

/// Create a PipelineOrchestrator for testing
pub async fn create_test_pipeline_orchestrator(
    state: AppState,
    options: CrawlOptions,
) -> Arc<PipelineOrchestrator> {
    Arc::new(PipelineOrchestrator::new(state, options))
}

/// Create a StrategiesPipelineOrchestrator for testing
pub async fn create_test_strategies_orchestrator(
    state: AppState,
    options: CrawlOptions,
) -> Arc<StrategiesPipelineOrchestrator> {
    Arc::new(StrategiesPipelineOrchestrator::new(state, options, None))
}

/// Create both orchestrators for testing the CrawlFacade
pub async fn create_test_orchestrators(
    options: CrawlOptions,
) -> (
    Arc<PipelineOrchestrator>,
    Arc<StrategiesPipelineOrchestrator>,
) {
    let state = create_test_state().await;
    let pipeline = Arc::new(PipelineOrchestrator::new(state.clone(), options.clone()));
    let strategies = Arc::new(StrategiesPipelineOrchestrator::new(state, options, None));
    (pipeline, strategies)
}
