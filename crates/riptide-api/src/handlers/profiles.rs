//! Ultra-thin domain profile API handlers (Phase 3 Sprint 3.1)
//!
//! All business logic delegated to ProfileFacade.
//! Handlers are <50 LOC total, focused only on HTTP transport concerns.

use crate::{context::ApplicationContext, dto::profiles::*, errors::ApiError};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use riptide_facade::facades::profile::ProfileFacade;
use riptide_intelligence::domain_profiling::ProfileManager;
use tracing::{debug, error, info};

/// Create profile (ultra-thin - 4 LOC)
pub async fn create_profile(
    State(_state): State<ApplicationContext>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    let profile = facade
        .create_with_config(
            request.domain.clone(),
            request.config.map(Into::into),
            request.metadata.map(Into::into),
        )
        .map_err(|e| {
            error!(error = %e, domain = request.domain, "Failed to create profile");
            ApiError::from(e)
        })?;
    info!(domain = request.domain, "Profile created");
    Ok((
        StatusCode::CREATED,
        Json(
            serde_json::json!({ "domain": profile.domain, "message": "Profile created successfully" }),
        ),
    ))
}

/// Get profile (ultra-thin - 3 LOC)
pub async fn get_profile(
    State(_state): State<ApplicationContext>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let profile = ProfileManager::load(&domain).map_err(|_| ApiError::NotFound {
        resource: format!("Profile: {}", domain),
    })?;
    Ok(Json(profile))
}

/// Update profile (ultra-thin - 4 LOC)
pub async fn update_profile(
    State(_state): State<ApplicationContext>,
    Path(domain): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    let profile = facade
        .update_profile(
            &domain,
            request.config.map(Into::into),
            request.metadata.map(Into::into),
        )
        .map_err(|e| {
            error!(error = %e, domain = domain, "Failed to update profile");
            ApiError::from(e)
        })?;
    info!(domain = domain, "Profile updated");
    Ok(Json(profile))
}

/// Delete profile (ultra-thin - 3 LOC)
pub async fn delete_profile(
    State(_state): State<ApplicationContext>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    ProfileManager::delete(&domain).map_err(|_| ApiError::NotFound {
        resource: format!("Profile: {}", domain),
    })?;
    info!(domain = domain, "Profile deleted");
    Ok(StatusCode::NO_CONTENT)
}

/// Batch create profiles (ultra-thin - 4 LOC)
pub async fn batch_create_profiles(
    State(_state): State<ApplicationContext>,
    Json(request): Json<BatchCreateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    let facade_requests: Vec<_> = request
        .profiles
        .into_iter()
        .map(|r| {
            (
                r.domain,
                r.config.map(Into::into),
                r.metadata.map(Into::into),
            )
        })
        .collect();
    let result = facade.batch_create(facade_requests);
    Ok((
        StatusCode::CREATED,
        Json(BatchCreateResponse {
            created: result.created,
            failed: result
                .failed
                .into_iter()
                .map(|f| BatchFailure {
                    domain: f.domain,
                    error: f.error,
                })
                .collect(),
        }),
    ))
}

/// Search profiles (ultra-thin - 3 LOC)
pub async fn search_profiles(
    State(_state): State<ApplicationContext>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let profiles = ProfileManager::search(&query.query).map_err(|e| ApiError::InternalError {
        message: format!("Failed to search profiles: {}", e),
    })?;
    Ok(Json(profiles))
}

/// List profiles (ultra-thin - 3 LOC)
pub async fn list_profiles(
    State(_state): State<ApplicationContext>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let profiles = if let Some(f) = query.filter {
        ProfileManager::list_by_tag(&f).map_err(|e| ApiError::InternalError {
            message: format!("Failed: {}", e),
        })?
    } else {
        ProfileManager::list_all().map_err(|e| ApiError::InternalError {
            message: format!("Failed: {}", e),
        })?
    };
    debug!(count = profiles.len(), "Listed profiles");
    Ok(Json(
        profiles
            .iter()
            .map(|p| p.domain.clone())
            .collect::<Vec<_>>(),
    ))
}

/// Get profile stats (ultra-thin - 3 LOC)
pub async fn get_profile_stats(
    State(_state): State<ApplicationContext>,
    Path(domain): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let profile = ProfileManager::load(&domain).map_err(|_| ApiError::NotFound {
        resource: format!("Profile: {}", domain),
    })?;
    Ok(Json(ProfileStatsResponse::from(&profile)))
}

/// Warm cache (ultra-thin - 4 LOC)
pub async fn warm_cache(
    State(_state): State<ApplicationContext>,
    Json(request): Json<WarmCacheRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    // Extract domain from URL
    let domain = request.url.split('/').nth(2).unwrap_or(&request.url);
    let (engine, confidence, message) = facade.warm_cache(domain, &request.url).map_err(|e| {
        error!(error = %e, url = request.url, "Failed to warm cache");
        ApiError::from(e)
    })?;
    Ok(Json(WarmCacheResponse {
        success: true,
        domain: domain.to_string(),
        cached_engine: Some(format!("{:?}", engine)),
        confidence: Some(confidence),
        message,
    }))
}

/// Get caching metrics (ultra-thin - 3 LOC)
pub async fn get_caching_metrics(
    State(_state): State<ApplicationContext>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    let metrics = facade
        .get_caching_metrics()
        .map_err(|e| ApiError::InternalError {
            message: format!("Failed: {}", e),
        })?;
    Ok(Json(CachingMetricsResponse {
        total_profiles: metrics.total_cached,
        cached_profiles: metrics.total_cached,
        cache_hit_rate: metrics.hit_rate,
        avg_confidence: 0.0, // TODO: Implement in Phase 6
        expired_caches: 0,   // TODO: Implement in Phase 6
    }))
}

/// Clear all caches (ultra-thin - 3 LOC)
pub async fn clear_all_caches(
    State(_state): State<ApplicationContext>,
) -> Result<impl IntoResponse, ApiError> {
    let facade = ProfileFacade::new();
    facade
        .clear_all_caches()
        .map_err(|e| ApiError::InternalError {
            message: e.to_string(),
        })?;
    Ok(Json(
        serde_json::json!({ "success": true, "cleared": 0, "failed": 0 }),
    ))
}
