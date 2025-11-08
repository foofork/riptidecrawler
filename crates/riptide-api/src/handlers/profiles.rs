//! Domain Profile Management API Handlers
//!
//! Phase 10.4: Domain Warm-Start Caching API endpoints
//!
//! This module provides RESTful API endpoints for managing domain profiles
//! and their cached engine preferences for warm-start optimization.

#![cfg(feature = "llm")]

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_facade::facades::profile::{
    ProfileConfigRequest as FacadeConfigRequest, ProfileFacade,
    ProfileMetadataRequest as FacadeMetadataRequest,
};
use riptide_intelligence::domain_profiling::{DomainProfile, ProfileManager};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// ============================================================================
// Request/Response Models
// ============================================================================

/// Request body for creating a domain profile
#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProfileRequest {
    pub domain: String,
    #[serde(default)]
    pub config: Option<ProfileConfigRequest>,
    #[serde(default)]
    pub metadata: Option<ProfileMetadataRequest>,
}

/// Profile configuration subset for API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileConfigRequest {
    pub stealth_level: Option<String>,
    pub rate_limit: Option<f64>,
    pub respect_robots_txt: Option<bool>,
    pub ua_strategy: Option<String>,
    pub confidence_threshold: Option<f64>,
    pub enable_javascript: Option<bool>,
    pub request_timeout_secs: Option<u64>,
}

/// Profile metadata subset for API requests
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileMetadataRequest {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
}

/// Request body for updating a domain profile
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub config: Option<ProfileConfigRequest>,
    #[serde(default)]
    pub metadata: Option<ProfileMetadataRequest>,
}

/// Request body for batch creating profiles
#[derive(Debug, Deserialize, Serialize)]
pub struct BatchCreateRequest {
    pub profiles: Vec<CreateProfileRequest>,
}

/// Response for batch operations
#[derive(Debug, Serialize)]
pub struct BatchCreateResponse {
    pub created: Vec<String>,
    pub failed: Vec<BatchFailure>,
}

/// Individual batch operation failure
#[derive(Debug, Serialize)]
pub struct BatchFailure {
    pub domain: String,
    pub error: String,
}

/// Query parameters for searching profiles
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

/// Query parameters for listing profiles
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub filter: Option<String>,
}

/// Profile statistics response
#[derive(Debug, Serialize)]
pub struct ProfileStatsResponse {
    pub domain: String,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub last_accessed: Option<String>,
    pub cache_status: CacheStatusInfo,
}

/// Cache status information
#[derive(Debug, Serialize)]
pub struct CacheStatusInfo {
    pub has_cached_engine: bool,
    pub is_valid: bool,
    pub engine: Option<String>,
    pub confidence: Option<f64>,
    pub expires_at: Option<String>,
}

/// Request body for warming cache
#[derive(Debug, Deserialize)]
pub struct WarmCacheRequest {
    pub url: String,
}

/// Response for cache warming operation
#[derive(Debug, Serialize)]
pub struct WarmCacheResponse {
    pub success: bool,
    pub domain: String,
    pub cached_engine: Option<String>,
    pub confidence: Option<f64>,
    pub message: String,
}

/// Caching metrics response
#[derive(Debug, Serialize)]
pub struct CachingMetricsResponse {
    pub total_profiles: usize,
    pub cached_profiles: usize,
    pub cache_hit_rate: f64,
    pub avg_confidence: f64,
    pub expired_caches: usize,
}

// ============================================================================
// API Endpoint Handlers
// ============================================================================

/// 1. POST /profiles - Create domain profile
///
/// Creates a new domain profile with optional configuration and metadata.
///
/// # Request Body
///
/// ```json
/// {
///   "domain": "example.com",
///   "config": {
///     "stealth_level": "high",
///     "rate_limit": 2.0,
///     "confidence_threshold": 0.8
///   },
///   "metadata": {
///     "description": "E-commerce site",
///     "tags": ["ecommerce", "product-pages"]
///   }
/// }
/// ```
#[tracing::instrument(
    name = "create_profile",
    skip(_state),
    fields(domain = %request.domain)
)]
pub async fn create_profile(
    State(_state): State<AppState>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate domain
    if request.domain.is_empty() {
        return Err(ApiError::validation("Domain cannot be empty"));
    }

    // Convert request DTOs to facade types
    let facade = ProfileFacade::new();
    let config = request.config.map(convert_config_to_facade);
    let metadata = request.metadata.map(convert_metadata_to_facade);

    // Create profile via facade
    let profile = facade
        .create_with_config(request.domain, config, metadata)
        .map_err(|e| {
            error!("Failed to create profile: {}", e);
            ApiError::validation(format!("Failed to create profile: {}", e))
        })?;

    Ok((StatusCode::CREATED, Json(profile)))
}

/// 2. GET /profiles/{domain} - Get profile
///
/// Retrieves a domain profile by domain name.
#[tracing::instrument(name = "get_profile", skip(_state), fields(domain = %domain))]
pub async fn get_profile(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Fetching profile for domain: {}", domain);

    let profile = ProfileManager::load(&domain).map_err(|e| {
        error!("Failed to load profile {}: {}", domain, e);
        ApiError::NotFound {
            resource: format!("Profile not found: {}", domain),
        }
    })?;

    Ok(Json(profile))
}

/// 3. PUT /profiles/{domain} - Update profile
///
/// Updates an existing domain profile configuration and metadata.
#[tracing::instrument(name = "update_profile", skip(_state), fields(domain = %domain))]
pub async fn update_profile(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Convert request DTOs to facade types
    let facade = ProfileFacade::new();
    let config = request.config.map(convert_config_to_facade);
    let metadata = request.metadata.map(convert_metadata_to_facade);

    // Update profile via facade
    let profile = facade
        .update_profile(&domain, config, metadata)
        .map_err(|e| {
            error!("Failed to update profile: {}", e);
            ApiError::InternalError {
                message: format!("Failed to update profile: {}", e),
            }
        })?;

    Ok(Json(profile))
}

/// 4. DELETE /profiles/{domain} - Delete profile
///
/// Deletes a domain profile from the registry.
#[tracing::instrument(name = "delete_profile", skip(_state), fields(domain = %domain))]
pub async fn delete_profile(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Deleting profile for domain: {}", domain);

    ProfileManager::delete(&domain).map_err(|e| {
        error!("Failed to delete profile {}: {}", domain, e);
        ApiError::NotFound {
            resource: format!("Profile not found: {}", domain),
        }
    })?;

    info!("Profile deleted successfully: {}", domain);

    Ok(StatusCode::NO_CONTENT)
}

/// 5. GET /profiles - List all profiles
///
/// Lists all domain profiles, optionally filtered by domain pattern.
///
/// Query parameters:
/// - `filter`: Optional domain name filter pattern
#[tracing::instrument(name = "list_profiles", skip(_state))]
pub async fn list_profiles(
    State(_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Listing profiles with filter: {:?}", query.filter);

    let profiles = if let Some(filter) = query.filter.as_deref() {
        ProfileManager::list_filtered(filter)
    } else {
        ProfileManager::list_all()
    }
    .map_err(|e| {
        error!("Failed to list profiles: {}", e);
        ApiError::InternalError {
            message: format!("Failed to list profiles: {}", e),
        }
    })?;

    Ok(Json(profiles))
}

/// 6. GET /profiles/{domain}/stats - Get usage stats
///
/// Retrieves usage statistics and cache status for a domain profile.
#[tracing::instrument(name = "get_profile_stats", skip(_state), fields(domain = %domain))]
pub async fn get_profile_stats(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Fetching stats for domain: {}", domain);

    let profile = ProfileManager::load(&domain).map_err(|e| {
        error!("Failed to load profile {}: {}", domain, e);
        ApiError::NotFound {
            resource: format!("Profile not found: {}", domain),
        }
    })?;

    // Build cache status
    let cache_status =
        if let Some((engine, confidence, expires_at)) = profile.get_cached_engine_info() {
            CacheStatusInfo {
                has_cached_engine: true,
                is_valid: profile.is_cache_valid(),
                engine: Some(format!("{:?}", engine)),
                confidence: Some(confidence),
                expires_at: Some(expires_at.to_rfc3339()),
            }
        } else {
            CacheStatusInfo {
                has_cached_engine: false,
                is_valid: false,
                engine: None,
                confidence: None,
                expires_at: None,
            }
        };

    let stats = ProfileStatsResponse {
        domain: profile.domain,
        total_requests: profile.metadata.total_requests,
        success_rate: profile.metadata.success_rate,
        avg_response_time_ms: profile.metadata.avg_response_time_ms,
        last_accessed: profile.metadata.last_accessed.map(|dt| dt.to_rfc3339()),
        cache_status,
    };

    Ok(Json(stats))
}

/// 7. POST /profiles/batch - Batch create
///
/// Creates multiple domain profiles in a single request.
///
/// # Request Body
///
/// ```json
/// {
///   "profiles": [
///     {"domain": "example1.com"},
///     {"domain": "example2.com", "config": {...}}
///   ]
/// }
/// ```
#[tracing::instrument(name = "batch_create_profiles", skip(_state))]
pub async fn batch_create_profiles(
    State(_state): State<AppState>,
    Json(request): Json<BatchCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Convert requests to facade format
    let facade = ProfileFacade::new();
    let requests: Vec<_> = request
        .profiles
        .into_iter()
        .map(|req| {
            (
                req.domain,
                req.config.map(convert_config_to_facade),
                req.metadata.map(convert_metadata_to_facade),
            )
        })
        .collect();

    // Execute batch create via facade
    let result = facade.batch_create(requests);

    let response = BatchCreateResponse {
        created: result.created,
        failed: result
            .failed
            .into_iter()
            .map(|f| BatchFailure {
                domain: f.domain,
                error: f.error,
            })
            .collect(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// 8. GET /profiles/search?query={q} - Search profiles
///
/// Searches for profiles matching the query pattern.
#[tracing::instrument(name = "search_profiles", skip(_state))]
pub async fn search_profiles(
    State(_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Searching profiles with query: {}", query.query);

    let profiles = ProfileManager::list_filtered(&query.query).map_err(|e| {
        error!("Failed to search profiles: {}", e);
        ApiError::InternalError {
            message: format!("Failed to search profiles: {}", e),
        }
    })?;

    Ok(Json(profiles))
}

/// 9. POST /profiles/{domain}/warm - Warm cache
///
/// Warms the engine cache for a domain by analyzing a URL.
///
/// # Request Body
///
/// ```json
/// {
///   "url": "https://example.com/page"
/// }
/// ```
#[tracing::instrument(name = "warm_cache", skip(_state), fields(domain = %domain))]
pub async fn warm_cache(
    State(_state): State<AppState>,
    Path(domain): Path<String>,
    Json(request): Json<WarmCacheRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Warm cache via facade
    let facade = ProfileFacade::new();
    let (engine, confidence, message) = facade.warm_cache(&domain, &request.url).map_err(|e| {
        error!("Failed to warm cache: {}", e);
        ApiError::InternalError {
            message: format!("Failed to warm cache: {}", e),
        }
    })?;

    let response = WarmCacheResponse {
        success: true,
        domain,
        cached_engine: Some(format!("{:?}", engine)),
        confidence: Some(confidence),
        message,
    };

    Ok(Json(response))
}

/// 10. DELETE /profiles/clear - Clear all caches
///
/// Clears all cached engine preferences across all profiles.
#[tracing::instrument(name = "clear_all_caches", skip(_state))]
pub async fn clear_all_caches(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Clearing all profile caches");

    let profiles = ProfileManager::list_all().map_err(|e| {
        error!("Failed to list profiles: {}", e);
        ApiError::InternalError {
            message: format!("Failed to list profiles: {}", e),
        }
    })?;

    let mut cleared = 0;
    let mut failed = 0;

    for mut profile in profiles {
        if profile.preferred_engine.is_some() {
            profile.invalidate_cache();
            match ProfileManager::save(&profile, None) {
                Ok(_) => cleared += 1,
                Err(e) => {
                    error!("Failed to save profile {}: {}", profile.domain, e);
                    failed += 1;
                }
            }
        }
    }

    info!(
        "Cache clearing completed: {} cleared, {} failed",
        cleared, failed
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "cleared": cleared,
        "failed": failed,
        "message": format!("Cleared {} profile caches", cleared)
    })))
}

/// 11. GET /profiles/metrics - Get caching metrics
///
/// Retrieves aggregated metrics about profile caching status.
#[tracing::instrument(name = "get_caching_metrics", skip(_state))]
pub async fn get_caching_metrics(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!("Fetching caching metrics");

    let profiles = ProfileManager::list_all().map_err(|e| {
        error!("Failed to list profiles: {}", e);
        ApiError::InternalError {
            message: format!("Failed to list profiles: {}", e),
        }
    })?;

    let total_profiles = profiles.len();
    let mut cached_profiles = 0;
    let mut expired_caches = 0;
    let mut total_confidence = 0.0;
    let mut confidence_count = 0;

    for profile in &profiles {
        if profile.preferred_engine.is_some() {
            cached_profiles += 1;
            if !profile.is_cache_valid() {
                expired_caches += 1;
            }
            if let Some(confidence) = profile.last_success_confidence {
                total_confidence += confidence;
                confidence_count += 1;
            }
        }
    }

    let cache_hit_rate = if total_profiles > 0 {
        (cached_profiles as f64 / total_profiles as f64) * 100.0
    } else {
        0.0
    };

    let avg_confidence = if confidence_count > 0 {
        total_confidence / confidence_count as f64
    } else {
        0.0
    };

    let metrics = CachingMetricsResponse {
        total_profiles,
        cached_profiles,
        cache_hit_rate,
        avg_confidence,
        expired_caches,
    };

    Ok(Json(metrics))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert API config request to facade config request
fn convert_config_to_facade(config: ProfileConfigRequest) -> FacadeConfigRequest {
    FacadeConfigRequest {
        stealth_level: config.stealth_level,
        rate_limit: config.rate_limit,
        respect_robots_txt: config.respect_robots_txt,
        ua_strategy: config.ua_strategy,
        confidence_threshold: config.confidence_threshold,
        enable_javascript: config.enable_javascript,
        request_timeout_secs: config.request_timeout_secs,
    }
}

/// Convert API metadata request to facade metadata request
fn convert_metadata_to_facade(metadata: ProfileMetadataRequest) -> FacadeMetadataRequest {
    FacadeMetadataRequest {
        description: metadata.description,
        tags: metadata.tags,
        author: metadata.author,
    }
}
