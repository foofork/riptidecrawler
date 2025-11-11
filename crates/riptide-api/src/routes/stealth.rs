//! Stealth configuration and testing routes
//!
//! This module defines the HTTP routes for stealth configuration, testing, and capabilities.

use crate::{handlers::stealth, context::ApplicationContext};
use axum::{
    routing::{get, post},
    Router,
};

/// Configure stealth processing routes
///
/// Provides endpoints for:
/// - Stealth configuration management
/// - Stealth effectiveness testing
/// - Stealth capabilities inquiry
/// - Health check for stealth features
pub fn stealth_routes() -> Router<ApplicationContext> {
    Router::new()
        // Stealth configuration endpoint
        .route("/configure", post(stealth::configure_stealth))
        // Stealth testing endpoint
        .route("/test", post(stealth::test_stealth))
        // Stealth capabilities endpoint
        .route("/capabilities", get(stealth::get_stealth_capabilities))
        // Health check for stealth features
        .route("/healthz", get(stealth::stealth_health_check))
}
