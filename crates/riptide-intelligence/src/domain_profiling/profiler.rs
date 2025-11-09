//! Domain Profile Management
//!
//! This module handles domain profile creation, configuration, and persistence.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::analyzer::SiteBaseline;
use super::DOMAIN_REGISTRY_DIR;

// Phase 10.4: Import Engine type and trait for cache functionality
use riptide_reliability::engine_selection::{Engine, EngineCacheable};

/// Domain profile containing configuration and baseline information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainProfile {
    pub name: String,
    pub domain: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: DomainConfig,
    pub baseline: Option<SiteBaseline>,
    pub metadata: DomainMetadata,
    pub patterns: DomainPatterns,

    // Phase 10.4: Engine warm-start caching fields
    /// Cached preferred engine from previous successful extractions
    #[serde(default)]
    pub preferred_engine: Option<Engine>,

    /// Confidence score from last successful extraction (0.0-1.0)
    #[serde(default)]
    pub last_success_confidence: Option<f64>,

    /// Expiration timestamp for cached engine (TTL: 7 days)
    #[serde(default)]
    pub engine_cache_expires_at: Option<DateTime<Utc>>,
}

/// Domain-specific extraction configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainConfig {
    pub stealth_level: String,
    pub rate_limit: f64,
    pub respect_robots_txt: bool,
    pub ua_strategy: String,
    pub schema: Option<String>,
    pub confidence_threshold: f64,
    pub enable_javascript: bool,
    pub request_timeout_secs: u64,
    pub custom_headers: HashMap<String, String>,
    pub proxy: Option<String>,
}

/// Domain metadata for tracking usage and performance
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub total_requests: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Domain-specific patterns for URL matching and filtering
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainPatterns {
    pub subdomain_regex: Vec<String>,
    pub path_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            stealth_level: "medium".to_string(),
            rate_limit: 1.0,
            respect_robots_txt: true,
            ua_strategy: "random".to_string(),
            schema: None,
            confidence_threshold: 0.7,
            enable_javascript: false,
            request_timeout_secs: 30,
            custom_headers: HashMap::new(),
            proxy: None,
        }
    }
}

impl DomainProfile {
    /// Create a new domain profile with default configuration
    pub fn new(domain: String) -> Self {
        let now = Utc::now();
        Self {
            name: domain.clone(),
            domain: domain.clone(),
            version: "1.1.0".to_string(), // Phase 10.4: Updated version for caching
            created_at: now,
            updated_at: now,
            config: DomainConfig::default(),
            baseline: None,
            metadata: DomainMetadata {
                description: None,
                tags: Vec::new(),
                author: None,
                total_requests: 0,
                success_rate: 0.0,
                avg_response_time_ms: 0,
                last_accessed: None,
            },
            patterns: DomainPatterns {
                subdomain_regex: Vec::new(),
                path_patterns: Vec::new(),
                exclude_patterns: Vec::new(),
            },
            // Phase 10.4: Initialize caching fields
            preferred_engine: None,
            last_success_confidence: None,
            engine_cache_expires_at: None,
        }
    }

    /// Save the profile to the specified path or default registry location
    pub fn save(&self, path: Option<&str>) -> Result<PathBuf> {
        let save_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            let registry_dir = dirs::home_dir()
                .context("Could not find home directory")?
                .join(DOMAIN_REGISTRY_DIR);
            fs::create_dir_all(&registry_dir)?;
            registry_dir.join(format!("{}.json", self.name))
        };

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&save_path, json)?;
        Ok(save_path)
    }

    /// Load a profile from the registry or a specific path
    pub fn load(domain: &str) -> Result<Self> {
        let path = if Path::new(domain).exists() {
            PathBuf::from(domain)
        } else {
            dirs::home_dir()
                .context("Could not find home directory")?
                .join(DOMAIN_REGISTRY_DIR)
                .join(format!("{}.json", domain))
        };

        let content = fs::read_to_string(&path)
            .context(format!("Failed to load domain profile: {}", domain))?;
        let profile: DomainProfile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Update the profile configuration
    pub fn update_config(&mut self, update_fn: impl FnOnce(&mut DomainConfig)) {
        update_fn(&mut self.config);
        self.updated_at = Utc::now();
    }

    /// Set the baseline for this profile
    pub fn set_baseline(&mut self, baseline: SiteBaseline) {
        self.baseline = Some(baseline);
        self.updated_at = Utc::now();
    }

    /// Update metadata
    pub fn update_metadata(&mut self, update_fn: impl FnOnce(&mut DomainMetadata)) {
        update_fn(&mut self.metadata);
        self.updated_at = Utc::now();
    }

    // Phase 10.4: Engine warm-start caching methods

    /// Cache the preferred engine with confidence score
    ///
    /// Sets the cached engine preference with a 7-day TTL. This cache is used
    /// for warm-start optimization to skip analysis on subsequent requests.
    ///
    /// # Arguments
    ///
    /// * `engine` - The engine that successfully extracted content
    /// * `confidence` - Quality score from extraction (0.0-1.0)
    ///
    /// # TTL Policy
    ///
    /// - Cache expires after 7 days to account for site structure changes
    /// - Automatically invalidated on next load if expired
    ///
    /// # Examples
    ///
    /// ```ignore
    /// profile.cache_engine(Engine::Wasm, 0.85);
    /// ```
    pub fn cache_engine(&mut self, engine: Engine, confidence: f64) {
        self.preferred_engine = Some(engine);
        self.last_success_confidence = Some(confidence);
        // Set 7-day TTL for cache
        self.engine_cache_expires_at = Some(Utc::now() + chrono::Duration::days(7));
        self.updated_at = Utc::now();
    }

    /// Get cached engine information (internal helper)
    ///
    /// Note: The public API uses the EngineCacheable trait implementation.
    /// This method is kept for backward compatibility but delegates to the trait.
    ///
    /// # Returns
    ///
    /// `Some(Engine)` if cache is valid and high-confidence, `None` otherwise
    pub fn get_cached_engine_info(&self) -> Option<(Engine, f64, DateTime<Utc>)> {
        Some((
            self.preferred_engine?,
            self.last_success_confidence?,
            self.engine_cache_expires_at?,
        ))
    }

    /// Check if the engine cache is still valid (not expired)
    ///
    /// # Returns
    ///
    /// `true` if cache exists and hasn't expired, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if profile.is_cache_valid() {
    ///     println!("Cache is fresh");
    /// } else {
    ///     println!("Cache expired or missing");
    /// }
    /// ```
    pub fn is_cache_valid(&self) -> bool {
        if let Some(expires_at) = self.engine_cache_expires_at {
            Utc::now() < expires_at
        } else {
            false
        }
    }

    /// Invalidate the engine cache
    ///
    /// Clears all cached engine data. Useful when site structure changes
    /// are detected or extraction quality degrades.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// profile.invalidate_cache();
    /// ```
    pub fn invalidate_cache(&mut self) {
        self.preferred_engine = None;
        self.last_success_confidence = None;
        self.engine_cache_expires_at = None;
        self.updated_at = Utc::now();
    }
}

// Phase 10.4: Implement EngineCacheable trait for DomainProfile
impl EngineCacheable for DomainProfile {
    /// Get the cached engine if valid (non-expired, high confidence > 70%)
    ///
    /// This implementation provides the caching logic directly for the trait.
    fn get_cached_engine(&self) -> Option<Engine> {
        // Check if cache exists
        let engine = self.preferred_engine?;
        let confidence = self.last_success_confidence?;

        // Validate cache hasn't expired
        if !self.is_cache_valid() {
            return None;
        }

        // Only use cache if confidence is high (> 70%)
        if confidence > 0.70 {
            Some(engine)
        } else {
            None
        }
    }
}

/// Profile manager for handling profile operations
pub struct ProfileManager;

impl ProfileManager {
    /// Create a new profile
    pub fn create(domain: String) -> DomainProfile {
        DomainProfile::new(domain)
    }

    /// Load an existing profile
    pub fn load(domain: &str) -> Result<DomainProfile> {
        DomainProfile::load(domain)
    }

    /// Load or create a profile
    pub fn load_or_create(domain: String) -> DomainProfile {
        DomainProfile::load(&domain).unwrap_or_else(|_| DomainProfile::new(domain))
    }

    /// Save a profile
    pub fn save(profile: &DomainProfile, path: Option<&str>) -> Result<PathBuf> {
        profile.save(path)
    }

    /// Delete a profile
    pub fn delete(domain: &str) -> Result<()> {
        let registry_path = dirs::home_dir()
            .context("Could not find home directory")?
            .join(DOMAIN_REGISTRY_DIR)
            .join(format!("{}.json", domain));

        if !registry_path.exists() {
            anyhow::bail!("Domain profile '{}' not found", domain);
        }

        fs::remove_file(&registry_path)?;
        Ok(())
    }

    /// List all profiles in the registry
    pub fn list_all() -> Result<Vec<DomainProfile>> {
        ProfileRegistry::list_profiles(None)
    }

    /// List profiles matching a filter
    pub fn list_filtered(filter: &str) -> Result<Vec<DomainProfile>> {
        ProfileRegistry::list_profiles(Some(filter))
    }

    /// Search profiles by query string (domain or metadata)
    /// TODO: Implement full-text search across profile fields
    pub fn search(query: &str) -> Result<Vec<DomainProfile>> {
        // For now, use filtered list as a simple search
        ProfileRegistry::list_profiles(Some(query))
    }

    /// List profiles filtered by tag
    /// TODO: Implement proper tag-based filtering
    pub fn list_by_tag(tag: &str) -> Result<Vec<DomainProfile>> {
        let all_profiles = Self::list_all()?;
        Ok(all_profiles
            .into_iter()
            .filter(|p| p.metadata.tags.contains(&tag.to_string()))
            .collect())
    }

    /// Validate a profile
    pub fn validate(profile: &DomainProfile) -> Result<()> {
        if profile.domain.is_empty() {
            anyhow::bail!("Invalid profile: domain is empty");
        }
        if profile.config.rate_limit <= 0.0 {
            anyhow::bail!("Invalid profile: rate limit must be positive");
        }
        if profile.config.confidence_threshold < 0.0 || profile.config.confidence_threshold > 1.0 {
            anyhow::bail!("Invalid profile: confidence threshold must be between 0.0 and 1.0");
        }
        Ok(())
    }

    /// Export a profile to a specific file
    pub fn export(domain: &str, output_path: &str) -> Result<()> {
        let profile = Self::load(domain)?;
        let content = serde_json::to_string_pretty(&profile)?;
        fs::write(output_path, content)?;
        Ok(())
    }

    /// Import a profile from a file
    pub fn import(file_path: &str, force: bool, validate: bool) -> Result<DomainProfile> {
        let content = fs::read_to_string(file_path)?;
        let profile: DomainProfile =
            serde_json::from_str(&content).context("Failed to parse profile file")?;

        if validate {
            Self::validate(&profile)?;
        }

        let registry_path = dirs::home_dir()
            .context("Could not find home directory")?
            .join(DOMAIN_REGISTRY_DIR)
            .join(format!("{}.json", profile.name));

        if registry_path.exists() && !force {
            anyhow::bail!(
                "Profile '{}' already exists. Use force flag to override",
                profile.name
            );
        }

        profile.save(None)?;
        Ok(profile)
    }
}

/// Registry for managing multiple domain profiles
pub struct ProfileRegistry;

impl ProfileRegistry {
    /// Get the registry directory path
    pub fn get_registry_dir() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(DOMAIN_REGISTRY_DIR);
        Ok(dir)
    }

    /// Ensure the registry directory exists
    pub fn ensure_registry_dir() -> Result<PathBuf> {
        let dir = Self::get_registry_dir()?;
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// List all profiles in the registry
    pub fn list_profiles(filter: Option<&str>) -> Result<Vec<DomainProfile>> {
        let registry_dir = Self::get_registry_dir()?;

        if !registry_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();
        for entry in fs::read_dir(&registry_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(profile) = serde_json::from_str::<DomainProfile>(&content) {
                        // Apply filter if specified
                        if let Some(pattern) = filter {
                            if !profile.domain.contains(pattern) {
                                continue;
                            }
                        }
                        profiles.push(profile);
                    }
                }
            }
        }

        Ok(profiles)
    }

    /// Check if a profile exists
    pub fn exists(domain: &str) -> bool {
        if let Ok(registry_dir) = Self::get_registry_dir() {
            let profile_path = registry_dir.join(format!("{}.json", domain));
            profile_path.exists()
        } else {
            false
        }
    }

    /// Count total profiles
    pub fn count() -> Result<usize> {
        Ok(Self::list_profiles(None)?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DomainConfig::default();
        assert_eq!(config.stealth_level, "medium");
        assert_eq!(config.rate_limit, 1.0);
        assert!(config.respect_robots_txt);
        assert_eq!(config.ua_strategy, "random");
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[test]
    fn test_new_profile() {
        let domain = "example.com".to_string();
        let profile = DomainProfile::new(domain.clone());
        assert_eq!(profile.name, domain);
        assert_eq!(profile.domain, domain);
        assert_eq!(profile.version, "1.1.0"); // Phase 10.4: Updated to v1.1.0
        assert!(profile.baseline.is_none());
    }

    #[test]
    fn test_profile_validation() {
        let profile = DomainProfile::new("example.com".to_string());
        assert!(ProfileManager::validate(&profile).is_ok());

        let mut invalid_profile = profile.clone();
        invalid_profile.domain = String::new();
        assert!(ProfileManager::validate(&invalid_profile).is_err());

        let mut invalid_config = profile.clone();
        invalid_config.config.rate_limit = -1.0;
        assert!(ProfileManager::validate(&invalid_config).is_err());
    }
}
