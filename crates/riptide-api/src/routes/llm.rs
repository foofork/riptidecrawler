//! LLM provider management routes configuration

use crate::handlers::llm;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};

/// Create LLM management routes
pub fn llm_routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(llm::list_providers))
        .route("/providers/current", get(llm::get_current_provider_info))
        .route("/providers/switch", post(llm::switch_provider))
        .route("/config", get(llm::get_config))
        .route("/config", post(llm::update_config))
}
