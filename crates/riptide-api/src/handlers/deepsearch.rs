//! Deep search handler - <50 LOC after facade refactoring
use crate::context::ApplicationContext;
use crate::errors::ApiError;
use axum::{extract::State, Json};
use riptide_facade::facades::deep_search::{
    DeepSearchFacade, DeepSearchRequest, DeepSearchResponse,
};
use serde::Deserialize;
use tracing::instrument;

#[derive(Debug, Deserialize)]
pub struct DeepSearchRequestDTO {
    pub query: String,
    pub max_depth: Option<usize>,
    pub max_results: Option<usize>,
    pub search_backends: Option<Vec<String>>,
}

#[instrument(skip(_state))]
pub async fn handle_deep_search(
    State(_state): State<ApplicationContext>,
    Json(req): Json<DeepSearchRequestDTO>,
) -> Result<Json<DeepSearchResponse>, ApiError> {
    DeepSearchFacade::new()
        .deep_search(DeepSearchRequest {
            query: req.query,
            max_depth: req.max_depth,
            max_results: req.max_results,
            search_backends: req.search_backends,
        })
        .await
        .map(Json)
        .map_err(|e| ApiError::internal(format!("Deep search failed: {}", e)))
}
