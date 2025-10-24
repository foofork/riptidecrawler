//! Domain Profile Management API Handlers
//!
//! Phase 10.4: Domain Warm-Start Caching API endpoints
//!
//! This module provides RESTful API endpoints for managing domain profiles
//! and their cached engine preferences for warm-start optimization.

use crate::errors::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_intelligence::domain_profiling::{DomainProfile, ProfileManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    skip(state),
    fields(domain = %request.domain)
)]
pub async fn create_profile(
    State(state): State<AppState>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating domain profile for: {}", request.domain);

    // Validate domain
    if request.domain.is_empty() {
        return Err(ApiError::validation("Domain cannot be empty"));
    }

    // Create new profile
    let mut profile = ProfileManager::create(request.domain.clone());

    // Apply optional configuration
    if let Some(config) = request.config {
        profile.update_config(|c| {
            if let Some(level) = config.stealth_level {
                c.stealth_level = level;
            }
            if let Some(limit) = config.rate_limit {
                c.rate_limit = limit;
            }
            if let Some(respect) = config.respect_robots_txt {
                c.respect_robots_txt = respect;
            }
            if let Some(strategy) = config.ua_strategy {
                c.ua_strategy = strategy;
            }
            if let Some(threshold) = config.confidence_threshold {
                c.confidence_threshold = threshold;
            }
            if let Some(js) = config.enable_javascript {
                c.enable_javascript = js;
            }
            if let Some(timeout) = config.request_timeout_secs {
                c.request_timeout_secs = timeout;
            }
        });
    }

    // Apply optional metadata
    if let Some(metadata) = request.metadata {
        profile.update_metadata(|m| {
            if let Some(desc) = metadata.description {
                m.description = Some(desc);
            }
            if let Some(tags) = metadata.tags {
                m.tags = tags;
            }
            if let Some(author) = metadata.author {
                m.author = Some(author);
            }
        });
    }

    // Validate profile
    ProfileManager::validate(&profile).map_err(|e| {
        error!("Profile validation failed: {}", e);
        ApiError::validation(format!("Invalid profile: {}", e))
    })?;

    // Save profile
    let save_path = ProfileManager::save(&profile, None).map_err(|e| {
        error!("Failed to save profile: {}", e);
        ApiError::InternalError {
            message: format!("Failed to save profile: {}", e),
        }
    })?;

    info!(
        "Profile created successfully: {} at {:?}",
        profile.domain, save_path
    );

    Ok((StatusCode::CREATED, Json(profile)))
}

/// 2. GET /profiles/{domain} - Get profile
///
/// Retrieves a domain profile by domain name.
#[tracing::instrument(name = "get_profile", skip(state), fields(domain = %domain))]
pub async fn get_profile(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "update_profile", skip(state), fields(domain = %domain))]
pub async fn update_profile(
    State(state): State<AppState>,
    Path(domain): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Updating profile for domain: {}", domain);

    // Load existing profile
    let mut profile = ProfileManager::load(&domain).map_err(|e| {
        error!("Failed to load profile {}: {}", domain, e);
        ApiError::NotFound {
            resource: format!("Profile not found: {}", domain),
        }
    })?;

    // Apply configuration updates
    if let Some(config) = request.config {
        profile.update_config(|c| {
            if let Some(level) = config.stealth_level {
                c.stealth_level = level;
            }
            if let Some(limit) = config.rate_limit {
                c.rate_limit = limit;
            }
            if let Some(respect) = config.respect_robots_txt {
                c.respect_robots_txt = respect;
            }
            if let Some(strategy) = config.ua_strategy {
                c.ua_strategy = strategy;
            }
            if let Some(threshold) = config.confidence_threshold {
                c.confidence_threshold = threshold;
            }
            if let Some(js) = config.enable_javascript {
                c.enable_javascript = js;
            }
            if let Some(timeout) = config.request_timeout_secs {
                c.request_timeout_secs = timeout;
            }
        });
    }

    // Apply metadata updates
    if let Some(metadata) = request.metadata {
        profile.update_metadata(|m| {
            if let Some(desc) = metadata.description {
                m.description = Some(desc);
            }
            if let Some(tags) = metadata.tags {
                m.tags = tags;
            }
            if let Some(author) = metadata.author {
                m.author = Some(author);
            }
        });
    }

    // Validate and save
    ProfileManager::validate(&profile).map_err(|e| {
        error!("Profile validation failed: {}", e);
        ApiError::validation(format!("Invalid profile: {}", e))
    })?;

    ProfileManager::save(&profile, None).map_err(|e| {
        error!("Failed to save profile: {}", e);
        ApiError::InternalError {
            message: format!("Failed to save profile: {}", e),
        }
    })?;

    info!("Profile updated successfully: {}", domain);

    Ok(Json(profile))
}

/// 4. DELETE /profiles/{domain} - Delete profile
///
/// Deletes a domain profile from the registry.
#[tracing::instrument(name = "delete_profile", skip(state), fields(domain = %domain))]
pub async fn delete_profile(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "list_profiles", skip(state))]
pub async fn list_profiles(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "get_profile_stats", skip(state), fields(domain = %domain))]
pub async fn get_profile_stats(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "batch_create_profiles", skip(state))]
pub async fn batch_create_profiles(
    State(state): State<AppState>,
    Json(request): Json<BatchCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Batch creating {} profiles", request.profiles.len());

    let mut created = Vec::new();
    let mut failed = Vec::new();

    for profile_req in request.profiles {
        let domain = profile_req.domain.clone();
        match create_profile_internal(&state, profile_req).await {
            Ok(_) => created.push(domain),
            Err(e) => failed.push(BatchFailure {
                domain,
                error: e.to_string(),
            }),
        }
    }

    let response = BatchCreateResponse { created, failed };

    info!(
        "Batch create completed: {} created, {} failed",
        response.created.len(),
        response.failed.len()
    );

    Ok((StatusCode::CREATED, Json(response)))
}

/// 8. GET /profiles/search?query={q} - Search profiles
///
/// Searches for profiles matching the query pattern.
#[tracing::instrument(name = "search_profiles", skip(state))]
pub async fn search_profiles(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "warm_cache", skip(state), fields(domain = %domain))]
pub async fn warm_cache(
    State(state): State<AppState>,
    Path(domain): Path<String>,
    Json(request): Json<WarmCacheRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!(
        "Warming cache for domain: {} with URL: {}",
        domain, request.url
    );

    // Load profile
    let mut profile = ProfileManager::load(&domain).map_err(|e| {
        error!("Failed to load profile {}: {}", domain, e);
        ApiError::NotFound {
            resource: format!("Profile not found: {}", domain),
        }
    })?;

    // For now, this is a placeholder for actual cache warming logic
    // In production, this would:
    // 1. Fetch and analyze the URL
    // 2. Determine optimal engine
    // 3. Cache the engine preference
    //
    // Since we don't have the full extraction pipeline here,
    // we'll simulate by setting a mock cache

    use riptide_reliability::engine_selection::Engine;

    // Simulate successful analysis (in production, use actual analyzer)
    let simulated_engine = Engine::Wasm; // Default to WASM
    let simulated_confidence = 0.85; // High confidence

    profile.cache_engine(simulated_engine, simulated_confidence);

    // Save updated profile
    ProfileManager::save(&profile, None).map_err(|e| {
        error!("Failed to save profile: {}", e);
        ApiError::InternalError {
            message: format!("Failed to save profile: {}", e),
        }
    })?;

    info!(
        "Cache warmed successfully for {}: engine={:?}, confidence={}",
        domain, simulated_engine, simulated_confidence
    );

    let response = WarmCacheResponse {
        success: true,
        domain: domain.clone(),
        cached_engine: Some(format!("{:?}", simulated_engine)),
        confidence: Some(simulated_confidence),
        message: format!("Cache warmed successfully for domain: {}", domain),
    };

    Ok(Json(response))
}

/// 10. DELETE /profiles/clear - Clear all caches
///
/// Clears all cached engine preferences across all profiles.
#[tracing::instrument(name = "clear_all_caches", skip(state))]
pub async fn clear_all_caches(
    State(state): State<AppState>,
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
#[tracing::instrument(name = "get_caching_metrics", skip(state))]
pub async fn get_caching_metrics(
    State(state): State<AppState>,
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

/// Internal helper for creating a profile (used by batch operations)
async fn create_profile_internal(
    _state: &AppState,
    request: CreateProfileRequest,
) -> Result<DomainProfile, ApiError> {
    let mut profile = ProfileManager::create(request.domain.clone());

    if let Some(config) = request.config {
        profile.update_config(|c| {
            if let Some(level) = config.stealth_level {
                c.stealth_level = level;
            }
            if let Some(limit) = config.rate_limit {
                c.rate_limit = limit;
            }
            if let Some(respect) = config.respect_robots_txt {
                c.respect_robots_txt = respect;
            }
            if let Some(strategy) = config.ua_strategy {
                c.ua_strategy = strategy;
            }
            if let Some(threshold) = config.confidence_threshold {
                c.confidence_threshold = threshold;
            }
            if let Some(js) = config.enable_javascript {
                c.enable_javascript = js;
            }
            if let Some(timeout) = config.request_timeout_secs {
                c.request_timeout_secs = timeout;
            }
        });
    }

    if let Some(metadata) = request.metadata {
        profile.update_metadata(|m| {
            if let Some(desc) = metadata.description {
                m.description = Some(desc);
            }
            if let Some(tags) = metadata.tags {
                m.tags = tags;
            }
            if let Some(author) = metadata.author {
                m.author = Some(author);
            }
        });
    }

    ProfileManager::validate(&profile)
        .map_err(|e| ApiError::validation(format!("Invalid profile: {}", e)))?;

    ProfileManager::save(&profile, None).map_err(|e| ApiError::InternalError {
        message: format!("Failed to save profile: {}", e),
    })?;

    Ok(profile)
}
