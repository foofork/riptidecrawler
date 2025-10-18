//! RipTide Integrated Cache Module
//!
//! This module provides integrated cache optimization, input validation,
//! and security features with comprehensive middleware support.

use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{header::HeaderMap, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::redis::{
    CacheConfig, CacheEntry, CacheManager, CacheMetadata, ConditionalResult,
};

// Note: These modules are from riptide-core and riptide-security
// They will need to be imported by users of IntegratedCacheManager
// For now, we'll comment out the integrated functionality that requires them

/// Integrated cache configuration combining all security and performance features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegratedCacheConfig {
    /// Cache configuration
    pub cache: CacheConfig,
    /// Security middleware configuration
    pub security: SecurityConfig,
    /// Input validation configuration
    pub validation: ValidationConfig,
    /// Redis connection URL
    pub redis_url: String,
    /// Enable debug logging
    pub debug: bool,
}

impl Default for IntegratedCacheConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            validation: ValidationConfig::default(),
            redis_url: "redis://localhost:6379/0".to_string(),
            debug: false,
        }
    }
}

/// Integrated cache manager with security and validation
pub struct IntegratedCacheManager {
    cache_manager: CacheManager,
    security_middleware: SecurityMiddleware,
    input_validator: CommonValidator,
    config: IntegratedCacheConfig,
}

impl IntegratedCacheManager {
    /// Create new integrated cache manager with default configuration
    pub async fn new(redis_url: &str) -> Result<Self> {
        let config = IntegratedCacheConfig {
            redis_url: redis_url.to_string(),
            ..Default::default()
        };
        Self::new_with_config(config).await
    }

    /// Create new integrated cache manager with custom configuration
    pub async fn new_with_config(config: IntegratedCacheConfig) -> Result<Self> {
        let cache_manager =
            CacheManager::new_with_config(&config.redis_url, config.cache.clone()).await?;
        let security_middleware = SecurityMiddleware::with_defaults()?;
        let input_validator = CommonValidator::new(config.validation.clone());

        Ok(Self {
            cache_manager,
            security_middleware,
            input_validator,
            config,
        })
    }

    /// Validate URL and check cache with conditional GET support
    pub async fn validate_and_check_cache(
        &mut self,
        url: &str,
        extractor_version: &str,
        options: &HashMap<String, String>,
        conditional_request: Option<ConditionalRequest>,
    ) -> Result<CacheCheckResult> {
        // Step 1: Validate URL
        let validated_url = self.input_validator.validate_url(url)?;
        info!(url = %validated_url, "URL validation passed");

        // Step 2: Generate cache key with version awareness
        let cache_key = self
            .cache_manager
            .generate_cache_key(url, extractor_version, options);

        // Step 3: Check cache with conditional support
        if let Some(conditional) = conditional_request {
            match self
                .cache_manager
                .check_conditional::<Vec<u8>>(
                    &cache_key,
                    conditional.if_none_match.as_deref(),
                    conditional.if_modified_since,
                )
                .await?
            {
                ConditionalResult::NotModified(entry) => {
                    debug!(cache_key = %cache_key, "Conditional cache hit - not modified");
                    return Ok(CacheCheckResult::NotModified(entry));
                }
                ConditionalResult::Modified(entry) => {
                    debug!(cache_key = %cache_key, "Conditional cache hit - modified");
                    return Ok(CacheCheckResult::Hit(entry));
                }
                ConditionalResult::Miss => {
                    debug!(cache_key = %cache_key, "Cache miss");
                }
            }
        } else {
            // Regular cache check
            if let Some(entry) = self.cache_manager.get::<Vec<u8>>(&cache_key).await? {
                debug!(cache_key = %cache_key, "Cache hit");
                return Ok(CacheCheckResult::Hit(entry));
            }
        }

        Ok(CacheCheckResult::Miss {
            cache_key,
            validated_url: validated_url.to_string(),
        })
    }

    /// Process and cache response with security validation
    pub async fn process_and_cache_response(
        &mut self,
        cache_key: &str,
        url: &str,
        response: Response,
        extractor_version: &str,
        options: &HashMap<String, String>,
        content: &[u8],
    ) -> Result<CachedContent> {
        // Step 1: Validate content type
        if let Some(content_type) = response.headers().get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                self.input_validator.validate_content_type(ct_str)?;
                info!(content_type = ct_str, "Content type validation passed");
            }
        }

        // Step 2: Validate content size
        self.input_validator.validate_content_size(content.len())?;
        self.security_middleware
            .validate_request_size(content.len())?;
        info!(
            content_size = content.len(),
            "Content size validation passed"
        );

        // Step 3: Extract conditional information
        let conditional_info = extract_conditional_info(&response).await;

        // Step 4: Generate ETag if not present
        let etag = conditional_info
            .etag
            .unwrap_or_else(|| generate_etag(content));

        // Step 5: Create cache metadata
        let metadata = CacheMetadata {
            extractor_version: extractor_version.to_string(),
            options_hash: Self::hash_options(options),
            url_hash: Self::hash_url(url),
            content_type: response
                .headers()
                .get("content-type")
                .and_then(|ct| ct.to_str().ok())
                .map(|s| s.to_string()),
        };

        // Step 6: Cache the content with TTL
        self.cache_manager
            .set(
                cache_key,
                &content.to_vec(),
                metadata.clone(),
                Some(etag.clone()),
                conditional_info.last_modified,
                None, // Use default TTL
            )
            .await?;

        info!(
            cache_key = %cache_key,
            content_size = content.len(),
            etag = %etag,
            "Content cached successfully"
        );

        Ok(CachedContent {
            content: content.to_vec(),
            etag,
            last_modified: conditional_info.last_modified,
            metadata,
            cached_at: Utc::now(),
        })
    }

    /// Apply security headers to outgoing response
    pub fn apply_security_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        self.security_middleware.apply_security_headers(headers)
    }

    /// Validate incoming request headers
    pub fn validate_request_headers(&self, headers: &HeaderMap) -> Result<HeaderMap> {
        // Convert HeaderMap to vector for validation
        let header_vec: Vec<(String, String)> = headers
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.to_string(), v.to_string()))
            })
            .collect();

        self.input_validator.validate_headers(&header_vec)?;

        // Clone headers for sanitization
        let mut sanitized_headers = headers.clone();
        self.security_middleware
            .sanitize_headers(&mut sanitized_headers)?;
        Ok(sanitized_headers)
    }

    /// Get comprehensive cache statistics
    pub async fn get_cache_stats(&mut self) -> Result<IntegratedCacheStats> {
        let cache_stats = self.cache_manager.get_stats().await?;

        Ok(IntegratedCacheStats {
            cache_stats,
            security_config: self.config.security.clone(),
            validation_config: self.config.validation.clone(),
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    /// Clear all cached content
    pub async fn clear_cache(&mut self) -> Result<u64> {
        self.cache_manager.clear_cache().await
    }

    /// Helper to hash options for cache key generation
    fn hash_options(options: &HashMap<String, String>) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        let mut sorted_options: Vec<_> = options.iter().collect();
        sorted_options.sort_by_key(|(k, _)| *k);

        for (key, value) in sorted_options {
            hasher.update(key.as_bytes());
            hasher.update(value.as_bytes());
        }

        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Helper to hash URL for cache metadata
    fn hash_url(url: &str) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Get configuration
    pub fn get_config(&self) -> &IntegratedCacheConfig {
        &self.config
    }
}

/// Result of cache check operation
#[derive(Debug)]
pub enum CacheCheckResult {
    /// Cache hit with entry
    Hit(CacheEntry<Vec<u8>>),
    /// Content not modified (304)
    NotModified(CacheEntry<Vec<u8>>),
    /// Cache miss
    Miss {
        cache_key: String,
        validated_url: String,
    },
}

/// Cached content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedContent {
    /// Cached content bytes
    pub content: Vec<u8>,
    /// ETag for conditional requests
    pub etag: String,
    /// Last modified timestamp
    pub last_modified: Option<DateTime<Utc>>,
    /// Cache metadata
    pub metadata: CacheMetadata,
    /// When content was cached
    pub cached_at: DateTime<Utc>,
}

/// Comprehensive integrated cache statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegratedCacheStats {
    /// Cache statistics
    pub cache_stats: crate::cache::CacheStats,
    /// Security configuration summary
    pub security_config: SecurityConfig,
    /// Validation configuration summary
    pub validation_config: ValidationConfig,
    /// System uptime in seconds
    pub uptime: u64,
}

/// Convenience function to create integrated cache manager with optimal defaults
pub async fn create_optimized_integrated_cache_manager(
    redis_url: &str,
) -> Result<IntegratedCacheManager> {
    let config = IntegratedCacheConfig {
        redis_url: redis_url.to_string(),
        cache: CacheConfig {
            default_ttl: 24 * 60 * 60,          // 24 hours
            max_content_size: 20 * 1024 * 1024, // 20MB
            cache_version: "v1".to_string(),
            enable_etag: true,
            enable_last_modified: true,
        },
        security: SecurityConfig {
            enable_cors: true,
            cors_allowed_origins: vec!["*".to_string()],
            enable_xss_protection: true,
            enable_content_type_protection: true,
            enable_frame_protection: true,
            enable_hsts: true,
            max_request_size: 20 * 1024 * 1024,
            rate_limit: Some(crate::security::RateLimitConfig {
                requests_per_window: 1000,
                window_seconds: 60,
                burst_size: 50,
            }),
        },
        validation: ValidationConfig::default(),
        debug: false,
    };

    IntegratedCacheManager::new_with_config(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_integrated_cache_config_creation() {
        let config = IntegratedCacheConfig::default();
        assert_eq!(config.redis_url, "redis://localhost:6379/0");
        assert_eq!(config.cache.default_ttl, 24 * 60 * 60);
        assert_eq!(config.security.max_request_size, 20 * 1024 * 1024);
    }

    #[test]
    fn test_hash_options() {
        let mut options = HashMap::new();
        options.insert("mode".to_string(), "article".to_string());
        options.insert("language".to_string(), "en".to_string());

        let hash1 = IntegratedCacheManager::hash_options(&options);
        let hash2 = IntegratedCacheManager::hash_options(&options);
        assert_eq!(hash1, hash2);

        // Different order should produce same hash
        let mut options2 = HashMap::new();
        options2.insert("language".to_string(), "en".to_string());
        options2.insert("mode".to_string(), "article".to_string());
        let hash3 = IntegratedCacheManager::hash_options(&options2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_hash_url() {
        let url = "https://example.com/page";
        let hash1 = IntegratedCacheManager::hash_url(url);
        let hash2 = IntegratedCacheManager::hash_url(url);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 16);
    }
}
