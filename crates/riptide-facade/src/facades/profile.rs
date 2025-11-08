//! Profile facade for domain profile management.
//!
//! Provides a high-level interface for managing domain profiles including
//! configuration, batch operations, and cache warming.

#![cfg(feature = "llm")]

use anyhow::Result;
use riptide_intelligence::domain_profiling::{DomainProfile, ProfileManager};
use riptide_reliability::engine_selection::Engine;

/// Request for creating a profile with configuration
#[derive(Debug, Clone)]
pub struct ProfileConfigRequest {
    pub stealth_level: Option<String>,
    pub rate_limit: Option<f64>,
    pub respect_robots_txt: Option<bool>,
    pub ua_strategy: Option<String>,
    pub confidence_threshold: Option<f64>,
    pub enable_javascript: Option<bool>,
    pub request_timeout_secs: Option<u64>,
}

/// Request for profile metadata
#[derive(Debug, Clone)]
pub struct ProfileMetadataRequest {
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
}

/// Result of batch create operation
#[derive(Debug)]
pub struct BatchCreateResult {
    pub created: Vec<String>,
    pub failed: Vec<BatchFailure>,
}

/// Individual batch operation failure
#[derive(Debug)]
pub struct BatchFailure {
    pub domain: String,
    pub error: String,
}

/// Profile facade for domain profile management.
///
/// This facade encapsulates all business logic for profile operations,
/// keeping handlers thin and focused on transport concerns.
pub struct ProfileFacade;

impl ProfileFacade {
    /// Create a new ProfileFacade instance.
    pub fn new() -> Self {
        Self
    }

    /// Create a domain profile with optional configuration and metadata.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name
    /// * `config` - Optional configuration settings
    /// * `metadata` - Optional metadata
    ///
    /// # Returns
    ///
    /// Returns the created and validated profile.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile validation fails
    /// - Profile save operation fails
    pub fn create_with_config(
        &self,
        domain: String,
        config: Option<ProfileConfigRequest>,
        metadata: Option<ProfileMetadataRequest>,
    ) -> Result<DomainProfile> {
        // Create base profile
        let mut profile = ProfileManager::create(domain);

        // Apply optional configuration
        if let Some(cfg) = config {
            profile.update_config(|c| {
                if let Some(level) = cfg.stealth_level {
                    c.stealth_level = level;
                }
                if let Some(limit) = cfg.rate_limit {
                    c.rate_limit = limit;
                }
                if let Some(respect) = cfg.respect_robots_txt {
                    c.respect_robots_txt = respect;
                }
                if let Some(strategy) = cfg.ua_strategy {
                    c.ua_strategy = strategy;
                }
                if let Some(threshold) = cfg.confidence_threshold {
                    c.confidence_threshold = threshold;
                }
                if let Some(js) = cfg.enable_javascript {
                    c.enable_javascript = js;
                }
                if let Some(timeout) = cfg.request_timeout_secs {
                    c.request_timeout_secs = timeout;
                }
            });
        }

        // Apply optional metadata
        if let Some(meta) = metadata {
            profile.update_metadata(|m| {
                if let Some(desc) = meta.description {
                    m.description = Some(desc);
                }
                if let Some(tags) = meta.tags {
                    m.tags = tags;
                }
                if let Some(author) = meta.author {
                    m.author = Some(author);
                }
            });
        }

        // Validate profile
        ProfileManager::validate(&profile)?;

        // Save profile
        ProfileManager::save(&profile, None)?;

        Ok(profile)
    }

    /// Batch create multiple profiles.
    ///
    /// # Arguments
    ///
    /// * `requests` - List of profile creation requests
    ///
    /// # Returns
    ///
    /// Returns a result containing lists of successful and failed creations.
    pub fn batch_create(
        &self,
        requests: Vec<(
            String,
            Option<ProfileConfigRequest>,
            Option<ProfileMetadataRequest>,
        )>,
    ) -> BatchCreateResult {
        let mut created = Vec::new();
        let mut failed = Vec::new();

        for (domain, config, metadata) in requests {
            let domain_clone = domain.clone();
            match self.create_with_config(domain, config, metadata) {
                Ok(_) => created.push(domain_clone),
                Err(e) => failed.push(BatchFailure {
                    domain: domain_clone,
                    error: e.to_string(),
                }),
            }
        }

        BatchCreateResult { created, failed }
    }

    /// Warm the cache for a domain profile.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name
    /// * `url` - URL to analyze for cache warming
    ///
    /// # Returns
    ///
    /// Returns tuple of (engine, confidence, message).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile not found
    /// - Cache warming fails
    /// - Profile save fails
    pub fn warm_cache(&self, domain: &str, _url: &str) -> Result<(Engine, f64, String)> {
        // Load profile
        let mut profile = ProfileManager::load(domain)?;

        // For now, simulate cache warming with default engine
        // In production, this would:
        // 1. Fetch and analyze the URL
        // 2. Determine optimal engine
        // 3. Cache the engine preference
        let simulated_engine = Engine::Wasm;
        let simulated_confidence = 0.85;

        profile.cache_engine(simulated_engine, simulated_confidence);

        // Save updated profile
        ProfileManager::save(&profile, None)?;

        let message = format!("Cache warmed successfully for domain: {}", domain);
        Ok((simulated_engine, simulated_confidence, message))
    }

    /// Update an existing profile with configuration and metadata.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name
    /// * `config` - Optional configuration updates
    /// * `metadata` - Optional metadata updates
    ///
    /// # Returns
    ///
    /// Returns the updated profile.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile not found
    /// - Validation fails
    /// - Save operation fails
    pub fn update_profile(
        &self,
        domain: &str,
        config: Option<ProfileConfigRequest>,
        metadata: Option<ProfileMetadataRequest>,
    ) -> Result<DomainProfile> {
        // Load existing profile
        let mut profile = ProfileManager::load(domain)?;

        // Apply configuration updates
        if let Some(cfg) = config {
            profile.update_config(|c| {
                if let Some(level) = cfg.stealth_level {
                    c.stealth_level = level;
                }
                if let Some(limit) = cfg.rate_limit {
                    c.rate_limit = limit;
                }
                if let Some(respect) = cfg.respect_robots_txt {
                    c.respect_robots_txt = respect;
                }
                if let Some(strategy) = cfg.ua_strategy {
                    c.ua_strategy = strategy;
                }
                if let Some(threshold) = cfg.confidence_threshold {
                    c.confidence_threshold = threshold;
                }
                if let Some(js) = cfg.enable_javascript {
                    c.enable_javascript = js;
                }
                if let Some(timeout) = cfg.request_timeout_secs {
                    c.request_timeout_secs = timeout;
                }
            });
        }

        // Apply metadata updates
        if let Some(meta) = metadata {
            profile.update_metadata(|m| {
                if let Some(desc) = meta.description {
                    m.description = Some(desc);
                }
                if let Some(tags) = meta.tags {
                    m.tags = tags;
                }
                if let Some(author) = meta.author {
                    m.author = Some(author);
                }
            });
        }

        // Validate and save
        ProfileManager::validate(&profile)?;
        ProfileManager::save(&profile, None)?;

        Ok(profile)
    }
}

impl Default for ProfileFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_facade_creation() {
        let facade = ProfileFacade::new();
        assert!(std::mem::size_of_val(&facade) == 0); // Zero-sized type
    }
}
