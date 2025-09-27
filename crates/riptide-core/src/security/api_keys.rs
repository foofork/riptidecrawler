//! API Key Management System for RipTide
//! 
//! Provides secure API key generation, validation, rotation, and rate limiting
//! with per-tenant isolation and comprehensive audit logging.

use crate::security::types::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as AsyncRwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// API Key information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub tenant_id: TenantId,
    pub key_hash: String,
    pub prefix: String,
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub rate_limits: RateLimitConfig,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub usage_count: u64,
    pub metadata: HashMap<String, String>,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        scopes: Vec<String>,
        rate_limits: Option<RateLimitConfig>,
        expires_at: Option<DateTime<Utc>>,
    ) -> (Self, String) {
        let id = Uuid::new_v4().to_string();
        let raw_key = Self::generate_raw_key();
        let key_hash = Self::hash_key(&raw_key);
        let prefix = raw_key[..8].to_string();
        
        let api_key = Self {
            id: id.clone(),
            tenant_id,
            key_hash,
            prefix,
            name,
            description,
            scopes,
            rate_limits: rate_limits.unwrap_or_default(),
            created_at: Utc::now(),
            expires_at,
            last_used_at: None,
            is_active: true,
            usage_count: 0,
            metadata: HashMap::new(),
        };
        
        (api_key, raw_key)
    }
    
    /// Generate a cryptographically secure random API key
    fn generate_raw_key() -> String {
        let key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();
        format!("rpt_{}", key)
    }
    
    /// Hash an API key using SHA-256
    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Verify if a raw key matches this API key
    pub fn verify_key(&self, raw_key: &str) -> bool {
        if !self.is_active {
            return false;
        }
        
        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }
        
        let hash = Self::hash_key(raw_key);
        hash == self.key_hash
    }
    
    /// Check if the API key has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }
    
    /// Update last used timestamp and increment usage count
    pub fn record_usage(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.usage_count += 1;
    }
    
    /// Rotate the API key, generating a new key while preserving metadata
    pub fn rotate(&mut self) -> String {
        let new_raw_key = Self::generate_raw_key();
        self.key_hash = Self::hash_key(&new_raw_key);
        self.prefix = new_raw_key[..8].to_string();
        self.created_at = Utc::now();
        self.usage_count = 0;
        self.last_used_at = None;
        new_raw_key
    }
}

/// Rate limiting tracker for API keys
#[derive(Debug, Clone)]
struct RateLimitTracker {
    minute_count: u32,
    hour_count: u32,
    day_count: u32,
    minute_reset: DateTime<Utc>,
    hour_reset: DateTime<Utc>,
    day_reset: DateTime<Utc>,
    burst_tokens: u32,
}

impl RateLimitTracker {
    fn new(burst_allowance: u32) -> Self {
        let now = Utc::now();
        Self {
            minute_count: 0,
            hour_count: 0,
            day_count: 0,
            minute_reset: now + Duration::minutes(1),
            hour_reset: now + Duration::hours(1),
            day_reset: now + Duration::days(1),
            burst_tokens: burst_allowance,
        }
    }
    
    fn reset_if_needed(&mut self, burst_allowance: u32) {
        let now = Utc::now();
        
        if now >= self.minute_reset {
            self.minute_count = 0;
            self.minute_reset = now + Duration::minutes(1);
            self.burst_tokens = burst_allowance;
        }
        
        if now >= self.hour_reset {
            self.hour_count = 0;
            self.hour_reset = now + Duration::hours(1);
        }
        
        if now >= self.day_reset {
            self.day_count = 0;
            self.day_reset = now + Duration::days(1);
        }
    }
    
    fn can_proceed(&self, limits: &RateLimitConfig) -> bool {
        if self.burst_tokens > 0 {
            return true;
        }
        
        self.minute_count < limits.requests_per_minute
            && self.hour_count < limits.requests_per_hour
            && self.day_count < limits.requests_per_day
    }
    
    fn consume_request(&mut self) {
        if self.burst_tokens > 0 {
            self.burst_tokens -= 1;
        } else {
            self.minute_count += 1;
            self.hour_count += 1;
            self.day_count += 1;
        }
    }
}

/// API Key Manager - handles all API key operations
pub struct ApiKeyManager {
    keys: Arc<AsyncRwLock<HashMap<String, ApiKey>>>,
    rate_trackers: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
    tenant_keys: Arc<AsyncRwLock<HashMap<TenantId, Vec<String>>>>,
}

impl ApiKeyManager {
    /// Create a new API Key Manager
    pub fn new() -> Self {
        Self {
            keys: Arc::new(AsyncRwLock::new(HashMap::new())),
            rate_trackers: Arc::new(RwLock::new(HashMap::new())),
            tenant_keys: Arc::new(AsyncRwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new API key for a tenant
    pub async fn create_api_key(
        &self,
        tenant_id: TenantId,
        name: String,
        description: Option<String>,
        scopes: Vec<String>,
        rate_limits: Option<RateLimitConfig>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(ApiKey, String)> {
        let (api_key, raw_key) = ApiKey::new(
            tenant_id.clone(),
            name,
            description,
            scopes,
            rate_limits,
            expires_at,
        );
        
        // Store the API key
        {
            let mut keys = self.keys.write().await;
            keys.insert(api_key.id.clone(), api_key.clone());
        }
        
        // Track tenant keys
        {
            let mut tenant_keys = self.tenant_keys.write().await;
            tenant_keys
                .entry(tenant_id)
                .or_insert_with(Vec::new)
                .push(api_key.id.clone());
        }
        
        // Initialize rate limiting tracker
        {
            let mut trackers = self.rate_trackers.write()
                .map_err(|e| anyhow!("Failed to acquire rate tracker lock: {}", e))?;
            trackers.insert(
                api_key.id.clone(),
                RateLimitTracker::new(api_key.rate_limits.burst_allowance),
            );
        }
        
        info!(
            api_key_id = %api_key.id,
            tenant_id = %api_key.tenant_id,
            "API key created successfully"
        );
        
        Ok((api_key, raw_key))
    }
    
    /// Validate an API key and check rate limits
    pub async fn validate_api_key(&self, raw_key: &str) -> SecurityResult<(ApiKey, SecurityContext)> {
        // Find the API key by trying to match the hash
        let api_key = {
            let keys = self.keys.read().await;
            keys.values()
                .find(|key| key.verify_key(raw_key))
                .cloned()
        };
        
        let mut api_key = match api_key {
            Some(key) => key,
            None => {
                warn!("Invalid API key attempted");
                return Err(SecurityError::InvalidApiKey("Invalid API key".to_string()));
            }
        };
        
        // Check if expired
        if api_key.is_expired() {
            warn!(api_key_id = %api_key.id, "Expired API key used");
            return Err(SecurityError::ApiKeyExpired(api_key.id.clone()));
        }
        
        // Check rate limits
        self.check_rate_limit(&api_key).await?;
        
        // Update usage
        api_key.record_usage();
        
        // Update in storage
        {
            let mut keys = self.keys.write().await;
            keys.insert(api_key.id.clone(), api_key.clone());
        }
        
        // Create security context
        let context = SecurityContext::new(
            api_key.tenant_id.clone(),
            api_key.id.clone(),
            "0.0.0.0".to_string(), // Will be updated by middleware
        ).with_scopes(api_key.scopes.clone());
        
        debug!(
            api_key_id = %api_key.id,
            tenant_id = %api_key.tenant_id,
            "API key validated successfully"
        );
        
        Ok((api_key, context))
    }
    
    /// Check rate limits for an API key
    async fn check_rate_limit(&self, api_key: &ApiKey) -> SecurityResult<()> {
        let mut trackers = self.rate_trackers.write()
            .map_err(|e| SecurityError::Unknown(format!("Rate tracker lock error: {}", e)))?;
        
        let tracker = trackers
            .entry(api_key.id.clone())
            .or_insert_with(|| RateLimitTracker::new(api_key.rate_limits.burst_allowance));
        
        tracker.reset_if_needed(api_key.rate_limits.burst_allowance);
        
        if !tracker.can_proceed(&api_key.rate_limits) {
            warn!(
                api_key_id = %api_key.id,
                tenant_id = %api_key.tenant_id,
                "Rate limit exceeded"
            );
            return Err(SecurityError::RateLimitExceeded(api_key.tenant_id.to_string()));
        }
        
        tracker.consume_request();
        Ok(())
    }
    
    /// Rotate an API key
    pub async fn rotate_api_key(&self, key_id: &str) -> SecurityResult<String> {
        let mut keys = self.keys.write().await;
        
        if let Some(api_key) = keys.get_mut(key_id) {
            let new_raw_key = api_key.rotate();
            
            info!(
                api_key_id = %api_key.id,
                tenant_id = %api_key.tenant_id,
                "API key rotated successfully"
            );
            
            Ok(new_raw_key)
        } else {
            Err(SecurityError::InvalidApiKey(format!("API key not found: {}", key_id)))
        }
    }
    
    /// Revoke an API key
    pub async fn revoke_api_key(&self, key_id: &str) -> SecurityResult<()> {
        let mut keys = self.keys.write().await;
        
        if let Some(api_key) = keys.get_mut(key_id) {
            api_key.is_active = false;
            
            info!(
                api_key_id = %api_key.id,
                tenant_id = %api_key.tenant_id,
                "API key revoked"
            );
            
            Ok(())
        } else {
            Err(SecurityError::InvalidApiKey(format!("API key not found: {}", key_id)))
        }
    }
    
    /// Get all API keys for a tenant
    pub async fn get_tenant_keys(&self, tenant_id: &TenantId) -> Vec<ApiKey> {
        let keys = self.keys.read().await;
        let tenant_keys = self.tenant_keys.read().await;
        
        if let Some(key_ids) = tenant_keys.get(tenant_id) {
            key_ids
                .iter()
                .filter_map(|id| keys.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get API key by ID
    pub async fn get_api_key(&self, key_id: &str) -> Option<ApiKey> {
        let keys = self.keys.read().await;
        keys.get(key_id).cloned()
    }
    
    /// Update API key metadata
    pub async fn update_api_key_metadata(
        &self,
        key_id: &str,
        metadata: HashMap<String, String>,
    ) -> SecurityResult<()> {
        let mut keys = self.keys.write().await;
        
        if let Some(api_key) = keys.get_mut(key_id) {
            api_key.metadata = metadata;
            Ok(())
        } else {
            Err(SecurityError::InvalidApiKey(format!("API key not found: {}", key_id)))
        }
    }
    
    /// Cleanup expired keys
    pub async fn cleanup_expired_keys(&self) -> usize {
        let mut keys = self.keys.write().await;
        let initial_count = keys.len();
        
        keys.retain(|_, key| !key.is_expired());
        
        let removed_count = initial_count - keys.len();
        if removed_count > 0 {
            info!(removed_count, "Cleaned up expired API keys");
        }
        
        removed_count
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_api_key_creation() {
        let manager = ApiKeyManager::new();
        let tenant_id = TenantId::from("test-tenant");
        
        let result = manager
            .create_api_key(
                tenant_id.clone(),
                "Test Key".to_string(),
                Some("Test Description".to_string()),
                vec!["read".to_string(), "write".to_string()],
                None,
                None,
            )
            .await;
        
        assert!(result.is_ok());
        let (api_key, raw_key) = result.unwrap();
        assert_eq!(api_key.tenant_id, tenant_id);
        assert_eq!(api_key.name, "Test Key");
        assert!(raw_key.starts_with("rpt_"));
        assert!(api_key.verify_key(&raw_key));
    }
    
    #[tokio::test]
    async fn test_api_key_validation() {
        let manager = ApiKeyManager::new();
        let tenant_id = TenantId::from("test-tenant");
        
        let (_, raw_key) = manager
            .create_api_key(
                tenant_id,
                "Test Key".to_string(),
                None,
                vec![],
                None,
                None,
            )
            .await
            .unwrap();
        
        let result = manager.validate_api_key(&raw_key).await;
        assert!(result.is_ok());
        
        let invalid_result = manager.validate_api_key("invalid_key").await;
        assert!(invalid_result.is_err());
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let manager = ApiKeyManager::new();
        let tenant_id = TenantId::from("test-tenant");
        
        let rate_limits = RateLimitConfig {
            requests_per_minute: 2,
            requests_per_hour: 10,
            requests_per_day: 100,
            burst_allowance: 0,
            enable_adaptive_limits: false,
        };
        
        let (_, raw_key) = manager
            .create_api_key(
                tenant_id,
                "Rate Limited Key".to_string(),
                None,
                vec![],
                Some(rate_limits),
                None,
            )
            .await
            .unwrap();
        
        // First two requests should succeed
        assert!(manager.validate_api_key(&raw_key).await.is_ok());
        assert!(manager.validate_api_key(&raw_key).await.is_ok());
        
        // Third request should fail due to rate limiting
        let result = manager.validate_api_key(&raw_key).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::RateLimitExceeded(_)));
    }
    
    #[tokio::test]
    async fn test_api_key_rotation() {
        let manager = ApiKeyManager::new();
        let tenant_id = TenantId::from("test-tenant");
        
        let (api_key, old_raw_key) = manager
            .create_api_key(
                tenant_id,
                "Rotatable Key".to_string(),
                None,
                vec![],
                None,
                None,
            )
            .await
            .unwrap();
        
        let new_raw_key = manager.rotate_api_key(&api_key.id).await.unwrap();
        
        // Old key should no longer work
        assert!(manager.validate_api_key(&old_raw_key).await.is_err());
        
        // New key should work
        assert!(manager.validate_api_key(&new_raw_key).await.is_ok());
    }
    
    #[test]
    fn test_api_key_expiration() {
        let expired_time = Utc::now() - Duration::minutes(1);
        let (mut api_key, _) = ApiKey::new(
            TenantId::from("test"),
            "Expired Key".to_string(),
            None,
            vec![],
            None,
            Some(expired_time),
        );
        
        assert!(api_key.is_expired());
        assert!(!api_key.verify_key("any_key"));
    }
}
