//! Profile Management Routes
//!
//! Phase 10.4: Domain Warm-Start Caching API routes
//!
//! This module defines the routing configuration for domain profile management
//! and engine caching endpoints.

use crate::handlers::profiles;
use crate::state::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

/// Create the profile management routes
///
/// All routes are mounted under `/api/v1/profiles` in the main router.
///
/// # Endpoints
///
/// ## Profile CRUD Operations
/// - `POST /` - Create a new domain profile
/// - `GET /:domain` - Get a specific profile
/// - `PUT /:domain` - Update a profile
/// - `DELETE /:domain` - Delete a profile
/// - `GET /` - List all profiles (with optional filter)
///
/// ## Statistics & Analytics
/// - `GET /:domain/stats` - Get profile usage statistics
/// - `GET /metrics` - Get aggregated caching metrics
///
/// ## Batch Operations
/// - `POST /batch` - Batch create multiple profiles
///
/// ## Search
/// - `GET /search` - Search profiles by query
///
/// ## Cache Management
/// - `POST /:domain/warm` - Warm the engine cache for a domain
/// - `DELETE /clear` - Clear all cached engines
///
/// # Examples
///
/// ```ignore
/// // Create profile
/// POST /api/v1/profiles
/// {
///   "domain": "example.com",
///   "config": {
///     "stealth_level": "high",
///     "rate_limit": 2.0
///   }
/// }
///
/// // Get profile stats
/// GET /api/v1/profiles/example.com/stats
///
/// // Warm cache
/// POST /api/v1/profiles/example.com/warm
/// {
///   "url": "https://example.com/page"
/// }
///
/// // Get caching metrics
/// GET /api/v1/profiles/metrics
/// ```
pub fn profile_routes() -> Router<AppState> {
    Router::new()
        // Profile CRUD operations
        .route("/", post(profiles::create_profile))
        .route("/", get(profiles::list_profiles))
        .route("/:domain", get(profiles::get_profile))
        .route("/:domain", put(profiles::update_profile))
        .route("/:domain", delete(profiles::delete_profile))
        // Statistics and analytics
        .route("/:domain/stats", get(profiles::get_profile_stats))
        .route("/metrics", get(profiles::get_caching_metrics))
        // Batch operations
        .route("/batch", post(profiles::batch_create_profiles))
        // Search
        .route("/search", get(profiles::search_profiles))
        // Cache management
        .route("/:domain/warm", post(profiles::warm_cache))
        .route("/clear", delete(profiles::clear_all_caches))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routes_compilation() {
        // This test ensures the routes compile correctly
        // Actual routing tests would use integration tests with TestClient
        let _router = profile_routes();
    }
}
