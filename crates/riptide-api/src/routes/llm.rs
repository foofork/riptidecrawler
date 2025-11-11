//! LLM provider management routes configuration

#[cfg(feature = "llm")]
use crate::handlers::llm;
use crate::context::ApplicationContext;
use axum::{
    routing::{get, post},
    Router,
};

/// Create LLM management routes
#[cfg(feature = "llm")]
pub fn llm_routes() -> Router<ApplicationContext> {
    Router::new()
        .route("/providers", get(llm::list_providers))
        .route("/providers/current", get(llm::get_current_provider_info))
        .route("/providers/switch", post(llm::switch_provider))
        .route("/config", get(llm::get_config))
        .route("/config", post(llm::update_config))
}

/// Create stub LLM routes when feature is disabled
/// Returns HTTP 501 "Not Implemented" for all LLM endpoints
#[cfg(not(feature = "llm"))]
pub fn llm_routes() -> Router<ApplicationContext> {
    use crate::handlers::stubs::*;

    Router::new()
        .route("/providers", get(llm_list_providers_stub))
        .route("/providers/current", get(llm_get_current_provider_stub))
        .route("/providers/switch", post(llm_switch_provider_stub))
        .route("/config", get(llm_get_config_stub))
        .route("/config", post(llm_update_config_stub))
}
