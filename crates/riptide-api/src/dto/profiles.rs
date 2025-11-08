//! Profiles API DTOs - Request/Response types for domain profile management
//!
//! Extracted from handlers/profiles.rs (Phase 3 Sprint 3.1)
//! Contains all 9 DTOs + conversion helpers

use riptide_facade::facades::profile::{
    ProfileConfigRequest as FacadeConfigRequest, ProfileMetadataRequest as FacadeMetadataRequest,
};
use riptide_intelligence::domain_profiling::DomainProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateProfileRequest {
    pub domain: String,
    #[serde(default)]
    pub config: Option<ProfileConfigRequest>,
    #[serde(default)]
    pub metadata: Option<ProfileMetadataRequest>,
}

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

impl From<ProfileConfigRequest> for FacadeConfigRequest {
    fn from(c: ProfileConfigRequest) -> Self {
        Self {
            stealth_level: c.stealth_level,
            rate_limit: c.rate_limit,
            respect_robots_txt: c.respect_robots_txt,
            ua_strategy: c.ua_strategy,
            confidence_threshold: c.confidence_threshold,
            enable_javascript: c.enable_javascript,
            request_timeout_secs: c.request_timeout_secs,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileMetadataRequest {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
}

impl From<ProfileMetadataRequest> for FacadeMetadataRequest {
    fn from(m: ProfileMetadataRequest) -> Self {
        Self {
            description: m.description,
            tags: m.tags,
            author: m.author,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub config: Option<ProfileConfigRequest>,
    #[serde(default)]
    pub metadata: Option<ProfileMetadataRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchCreateRequest {
    pub profiles: Vec<CreateProfileRequest>,
}

#[derive(Debug, Serialize)]
pub struct BatchCreateResponse {
    pub created: Vec<String>,
    pub failed: Vec<BatchFailure>,
}

#[derive(Debug, Serialize)]
pub struct BatchFailure {
    pub domain: String,
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileStatsResponse {
    pub domain: String,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub last_accessed: Option<String>,
    pub cache_status: CacheStatusInfo,
}

impl From<&DomainProfile> for ProfileStatsResponse {
    fn from(profile: &DomainProfile) -> Self {
        Self {
            domain: profile.domain.clone(),
            total_requests: profile.total_requests,
            success_rate: profile.success_rate,
            avg_response_time_ms: profile.avg_response_time_ms,
            last_accessed: profile.last_accessed.as_ref().map(|t| t.to_rfc3339()),
            cache_status: CacheStatusInfo::from_profile(profile),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStatusInfo {
    pub has_cached_engine: bool,
    pub is_valid: bool,
    pub engine: Option<String>,
    pub confidence: Option<f64>,
    pub expires_at: Option<String>,
}

impl CacheStatusInfo {
    pub fn from_profile(profile: &DomainProfile) -> Self {
        if let Some((engine, confidence, expires_at)) = profile.get_cached_engine_info() {
            Self {
                has_cached_engine: true,
                is_valid: profile.is_cache_valid(),
                engine: Some(format!("{:?}", engine)),
                confidence: Some(confidence),
                expires_at: Some(expires_at.to_rfc3339()),
            }
        } else {
            Self {
                has_cached_engine: false,
                is_valid: false,
                engine: None,
                confidence: None,
                expires_at: None,
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WarmCacheRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct WarmCacheResponse {
    pub success: bool,
    pub domain: String,
    pub cached_engine: Option<String>,
    pub confidence: Option<f64>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct CachingMetricsResponse {
    pub total_profiles: usize,
    pub cached_profiles: usize,
    pub cache_hit_rate: f64,
    pub avg_confidence: f64,
    pub expired_caches: usize,
}
