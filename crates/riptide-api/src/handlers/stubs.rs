//! Stub handlers for disabled features
//!
//! This module provides HTTP 501 "Not Implemented" responses for endpoints
//! that are disabled at compile time via feature flags.
#![allow(dead_code)]

use crate::errors::ApiError;
use axum::{
    extract::{Json, Path, Query},
    response::IntoResponse,
};
use serde_json::Value;
use std::collections::HashMap;

/// Generic handler that returns HTTP 501 for any disabled feature endpoint
macro_rules! feature_stub {
    ($handler_name:ident, $feature:expr) => {
        pub async fn $handler_name() -> impl IntoResponse {
            ApiError::feature_not_enabled($feature)
        }
    };
    ($handler_name:ident, $feature:expr, Json) => {
        pub async fn $handler_name(_body: Json<Value>) -> impl IntoResponse {
            ApiError::feature_not_enabled($feature)
        }
    };
    ($handler_name:ident, $feature:expr, Path) => {
        pub async fn $handler_name(_path: Path<String>) -> impl IntoResponse {
            ApiError::feature_not_enabled($feature)
        }
    };
    ($handler_name:ident, $feature:expr, PathJson) => {
        pub async fn $handler_name(_path: Path<String>, _body: Json<Value>) -> impl IntoResponse {
            ApiError::feature_not_enabled($feature)
        }
    };
    ($handler_name:ident, $feature:expr, Query) => {
        pub async fn $handler_name(_query: Query<HashMap<String, String>>) -> impl IntoResponse {
            ApiError::feature_not_enabled($feature)
        }
    };
}

// LLM Feature Stubs
feature_stub!(llm_list_providers_stub, "llm");
feature_stub!(llm_get_current_provider_stub, "llm");
feature_stub!(llm_switch_provider_stub, "llm", Json);
feature_stub!(llm_get_config_stub, "llm");
feature_stub!(llm_update_config_stub, "llm", Json);

// Profile Management Stubs (LLM feature)
feature_stub!(profile_create_stub, "llm", Json);
feature_stub!(profile_list_stub, "llm", Query);
feature_stub!(profile_get_stub, "llm", Path);
feature_stub!(profile_update_stub, "llm", PathJson);
feature_stub!(profile_delete_stub, "llm", Path);
feature_stub!(profile_stats_stub, "llm", Path);
feature_stub!(profile_metrics_stub, "llm");
feature_stub!(profile_batch_create_stub, "llm", Json);
feature_stub!(profile_search_stub, "llm", Query);
feature_stub!(profile_warm_cache_stub, "llm", PathJson);
feature_stub!(profile_clear_caches_stub, "llm");

// Extraction Feature Stubs
feature_stub!(extraction_extract_stub, "extraction", Json);
feature_stub!(extraction_chunk_stub, "extraction", Json);
feature_stub!(extraction_tables_stub, "extraction", Json);
feature_stub!(extraction_strategies_stub, "extraction", Json);
feature_stub!(table_export_stub, "extraction", Path);

// Browser Feature Stubs
feature_stub!(browser_render_stub, "browser", Json);

// Spider Feature Stubs
feature_stub!(spider_crawl_stub, "spider", Json);
feature_stub!(spider_stream_stub, "spider", Json);

// Workers Feature Stubs
feature_stub!(workers_submit_stub, "workers", Json);
feature_stub!(workers_status_stub, "workers", Path);

// Fetch Feature Stubs
feature_stub!(fetch_content_stub, "fetch", Json);

// Persistence Feature Stubs
feature_stub!(admin_cache_clear_stub, "persistence");
feature_stub!(admin_stats_stub, "persistence");

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_feature_stub_returns_501() {
        let response = llm_list_providers_stub().await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_feature_stub_with_json() {
        let body = Json(serde_json::json!({"test": "data"}));
        let response = llm_switch_provider_stub(body).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_feature_stub_with_path() {
        let path = Path("example.com".to_string());
        let response = profile_get_stub(path).await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    }
}
