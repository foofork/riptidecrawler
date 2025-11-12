//! System capabilities detection for deployment mode reporting
//!
//! This module provides runtime detection of system capabilities to inform users
//! about the current deployment configuration (minimal, enhanced, or distributed).

use serde::{Deserialize, Serialize};

/// System capabilities representing the current deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemCapabilities {
    /// Cache backend in use: "memory" or "redis"
    pub cache_backend: String,

    /// Whether async job workers are enabled
    pub async_jobs: bool,

    /// Whether system is capable of distributed operation (Redis + workers)
    pub distributed: bool,

    /// Whether cache persists across restarts (Redis backend)
    pub persistent_cache: bool,

    /// Whether sessions persist across restarts (Redis backend)
    pub session_persistence: bool,

    /// Deployment mode: "minimal", "enhanced", or "distributed"
    pub deployment_mode: String,
}

impl SystemCapabilities {
    /// Detect system capabilities from cache backend and worker configuration
    ///
    /// # Deployment Modes
    ///
    /// - **Minimal**: In-memory cache, no workers (suitable for development/testing)
    /// - **Enhanced**: Redis cache, no workers (persistent cache, single-instance)
    /// - **Distributed**: Redis cache + workers (full multi-instance capability)
    ///
    /// # Arguments
    ///
    /// * `cache_backend` - The cache backend type ("memory" or "redis")
    /// * `workers_enabled` - Whether the worker service is enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_api::capabilities::SystemCapabilities;
    ///
    /// // Minimal deployment (development)
    /// let caps = SystemCapabilities::detect("memory", false);
    /// assert_eq!(caps.deployment_mode, "minimal");
    /// assert!(!caps.persistent_cache);
    ///
    /// // Enhanced deployment (Redis cache)
    /// let caps = SystemCapabilities::detect("redis", false);
    /// assert_eq!(caps.deployment_mode, "enhanced");
    /// assert!(caps.persistent_cache);
    ///
    /// // Distributed deployment (Redis + workers)
    /// let caps = SystemCapabilities::detect("redis", true);
    /// assert_eq!(caps.deployment_mode, "distributed");
    /// assert!(caps.distributed);
    /// ```
    pub fn detect(cache_backend: &str, workers_enabled: bool) -> Self {
        let has_redis = cache_backend.eq_ignore_ascii_case("redis");

        // Determine deployment mode based on configuration
        let deployment_mode = match (has_redis, workers_enabled) {
            (false, false) => "minimal",
            (true, false) => "enhanced",
            (true, true) => "distributed",
            (false, true) => "invalid", // Can't have workers without Redis
        };

        Self {
            cache_backend: cache_backend.to_string(),
            async_jobs: workers_enabled,
            distributed: has_redis && workers_enabled,
            persistent_cache: has_redis,
            session_persistence: has_redis,
            deployment_mode: deployment_mode.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_deployment() {
        let caps = SystemCapabilities::detect("memory", false);

        assert_eq!(caps.cache_backend, "memory");
        assert!(!caps.async_jobs);
        assert!(!caps.distributed);
        assert!(!caps.persistent_cache);
        assert!(!caps.session_persistence);
        assert_eq!(caps.deployment_mode, "minimal");
    }

    #[test]
    fn test_enhanced_deployment() {
        let caps = SystemCapabilities::detect("redis", false);

        assert_eq!(caps.cache_backend, "redis");
        assert!(!caps.async_jobs);
        assert!(!caps.distributed);
        assert!(caps.persistent_cache);
        assert!(caps.session_persistence);
        assert_eq!(caps.deployment_mode, "enhanced");
    }

    #[test]
    fn test_distributed_deployment() {
        let caps = SystemCapabilities::detect("redis", true);

        assert_eq!(caps.cache_backend, "redis");
        assert!(caps.async_jobs);
        assert!(caps.distributed);
        assert!(caps.persistent_cache);
        assert!(caps.session_persistence);
        assert_eq!(caps.deployment_mode, "distributed");
    }

    #[test]
    fn test_invalid_deployment() {
        let caps = SystemCapabilities::detect("memory", true);

        assert_eq!(caps.cache_backend, "memory");
        assert!(caps.async_jobs);
        assert!(!caps.distributed);
        assert!(!caps.persistent_cache);
        assert!(!caps.session_persistence);
        assert_eq!(caps.deployment_mode, "invalid");
    }

    #[test]
    fn test_case_insensitive_backend() {
        let caps = SystemCapabilities::detect("REDIS", false);
        assert_eq!(caps.deployment_mode, "enhanced");
        assert!(caps.persistent_cache);
    }
}
