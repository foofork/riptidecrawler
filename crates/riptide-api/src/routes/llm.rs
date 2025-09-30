//! LLM provider management routes configuration

use crate::handlers::llm;
use axum::{routing::{get, post}, Router};
use crate::state::AppState;

/// Create LLM management routes
pub fn llm_routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(llm::list_providers))
        .route("/providers/switch", post(llm::switch_provider))
        .route("/config", get(llm::get_config))
        .route("/config", post(llm::update_config))
}