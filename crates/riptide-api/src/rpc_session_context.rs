///! Session context management for RPC client stateful rendering
///!
///! This module provides session persistence infrastructure for the RPC client,
///! enabling stateful rendering workflows with session management.
use anyhow::{anyhow, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Session context for stateful rendering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSessionContext {
    /// Unique session identifier
    pub session_id: String,

    /// Session creation timestamp
    pub created_at: SystemTime,

    /// Last accessed timestamp
    pub last_accessed: SystemTime,

    /// Session expiry time
    pub expires_at: SystemTime,

    /// Session-specific state metadata
    pub state: SessionState,

    /// Session-level configuration overrides
    pub config: SessionConfig,
}

/// Session state for tracking rendering context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    /// Number of requests made in this session
    pub request_count: u64,

    /// Last URL rendered in this session
    pub last_url: Option<String>,

    /// Total render time across all requests (ms)
    pub total_render_time_ms: u64,

    /// Average render time per request (ms)
    pub avg_render_time_ms: f64,

    /// Custom metadata for application-specific state
    pub metadata: std::collections::HashMap<String, String>,

    /// Last error message if any
    pub last_error: Option<String>,
}

/// Session configuration for RPC client behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session TTL (time-to-live)
    pub ttl: Duration,

    /// Enable automatic session renewal on access
    pub auto_renew: bool,

    /// Maximum requests per session (0 = unlimited)
    pub max_requests: u64,

    /// Enable session metrics tracking
    pub track_metrics: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(1800), // 30 minutes
            auto_renew: true,
            max_requests: 0, // unlimited
            track_metrics: true,
        }
    }
}

impl RpcSessionContext {
    /// Create a new session context with default configuration
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }

    /// Create a new session context with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        let now = SystemTime::now();
        let session_id = format!("rpc_session_{}", Uuid::new_v4().simple());

        Self {
            session_id: session_id.clone(),
            created_at: now,
            last_accessed: now,
            expires_at: now + config.ttl,
            state: SessionState::default(),
            config,
        }
    }

    /// Create session from existing session ID
    pub fn from_session_id(session_id: String) -> Self {
        let config = SessionConfig::default();
        let now = SystemTime::now();

        Self {
            session_id,
            created_at: now,
            last_accessed: now,
            expires_at: now + config.ttl,
            state: SessionState::default(),
            config,
        }
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Update last accessed time and optionally extend expiry
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
        if self.config.auto_renew {
            self.expires_at = SystemTime::now() + self.config.ttl;
        }
    }

    /// Record a request in session state
    pub fn record_request(&mut self, url: &str, render_time_ms: u64) {
        self.touch();
        self.state.request_count += 1;
        self.state.last_url = Some(url.to_string());
        self.state.total_render_time_ms += render_time_ms;

        if self.config.track_metrics {
            self.state.avg_render_time_ms =
                self.state.total_render_time_ms as f64 / self.state.request_count as f64;
        }
    }

    /// Record an error in session state
    pub fn record_error(&mut self, error: String) {
        self.state.last_error = Some(error);
        self.touch();
    }

    /// Check if session has reached max requests limit
    pub fn is_request_limit_reached(&self) -> bool {
        if self.config.max_requests == 0 {
            return false; // unlimited
        }
        self.state.request_count >= self.config.max_requests
    }

    /// Get session metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.state.metadata.get(key)
    }

    /// Set session metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.state.metadata.insert(key, value);
    }

    /// Get session age in seconds
    pub fn age_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or_default()
            .as_secs()
    }
}

impl Default for RpcSessionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe session storage for RPC client
#[derive(Clone)]
pub struct RpcSessionStore {
    /// In-memory session cache using DashMap for concurrent access
    sessions: Arc<DashMap<String, RpcSessionContext>>,

    /// Default session configuration
    default_config: SessionConfig,
}

impl RpcSessionStore {
    /// Create a new session store with default configuration
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            default_config: SessionConfig::default(),
        }
    }

    /// Create a new session store with custom default configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            default_config: config,
        }
    }

    /// Get or create a session
    pub fn get_or_create(&self, session_id: &str) -> RpcSessionContext {
        // Check if session exists and is valid
        if let Some(mut entry) = self.sessions.get_mut(session_id) {
            let session = entry.value_mut();
            if !session.is_expired() {
                session.touch();
                debug!(
                    session_id = %session_id,
                    request_count = session.state.request_count,
                    "Retrieved existing RPC session"
                );
                return session.clone();
            } else {
                // Session expired, remove it
                debug!(session_id = %session_id, "RPC session expired, creating new one");
            }
        }

        // Create new session
        let session = RpcSessionContext::from_session_id(session_id.to_string());
        self.sessions
            .insert(session_id.to_string(), session.clone());

        info!(
            session_id = %session_id,
            ttl_seconds = self.default_config.ttl.as_secs(),
            "Created new RPC session"
        );

        session
    }

    /// Get an existing session without creating one
    pub fn get(&self, session_id: &str) -> Option<RpcSessionContext> {
        self.sessions.get(session_id).map(|entry| {
            let mut session = entry.value().clone();
            session.touch();
            session
        })
    }

    /// Update an existing session
    pub fn update(&self, session: RpcSessionContext) -> Result<()> {
        if session.is_expired() {
            return Err(anyhow!(
                "Cannot update expired session: {}",
                session.session_id
            ));
        }

        self.sessions
            .insert(session.session_id.clone(), session.clone());

        debug!(
            session_id = %session.session_id,
            request_count = session.state.request_count,
            "Updated RPC session"
        );

        Ok(())
    }

    /// Remove a session
    pub fn remove(&self, session_id: &str) -> Option<RpcSessionContext> {
        let removed = self.sessions.remove(session_id);
        if removed.is_some() {
            info!(session_id = %session_id, "Removed RPC session");
        }
        removed.map(|(_, session)| session)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&self) -> usize {
        let now = SystemTime::now();
        let expired_keys: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| entry.value().expires_at <= now)
            .map(|entry| entry.key().clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.sessions.remove(&key);
        }

        if count > 0 {
            info!(count = count, "Cleaned up expired RPC sessions");
        }

        count
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// List all active session IDs
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(&self, interval: Duration) {
        let store = self.clone();
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let cleaned = store.cleanup_expired();
                if cleaned > 0 {
                    debug!(
                        count = cleaned,
                        "Background cleanup removed expired RPC sessions"
                    );
                }
            }
        });

        info!(
            interval_seconds = interval.as_secs(),
            "Started RPC session cleanup task"
        );
    }
}

impl Default for RpcSessionStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Session metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// Total number of active sessions
    pub active_sessions: usize,

    /// Total requests across all sessions
    pub total_requests: u64,

    /// Average session age in seconds
    pub avg_session_age_seconds: f64,

    /// Total render time across all sessions (ms)
    pub total_render_time_ms: u64,

    /// Average render time per request (ms)
    pub avg_render_time_ms: f64,
}

impl RpcSessionStore {
    /// Get aggregated session metrics
    pub fn get_metrics(&self) -> SessionMetrics {
        let sessions: Vec<_> = self.sessions.iter().map(|e| e.value().clone()).collect();
        let active_count = sessions.len();

        if active_count == 0 {
            return SessionMetrics {
                active_sessions: 0,
                total_requests: 0,
                avg_session_age_seconds: 0.0,
                total_render_time_ms: 0,
                avg_render_time_ms: 0.0,
            };
        }

        let total_requests: u64 = sessions.iter().map(|s| s.state.request_count).sum();
        let total_age: u64 = sessions.iter().map(|s| s.age_seconds()).sum();
        let total_render_time: u64 = sessions.iter().map(|s| s.state.total_render_time_ms).sum();

        SessionMetrics {
            active_sessions: active_count,
            total_requests,
            avg_session_age_seconds: total_age as f64 / active_count as f64,
            total_render_time_ms: total_render_time,
            avg_render_time_ms: if total_requests > 0 {
                total_render_time as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_context_creation() {
        let session = RpcSessionContext::new();
        assert!(!session.session_id.is_empty());
        assert!(!session.is_expired());
        assert_eq!(session.state.request_count, 0);
    }

    #[test]
    fn test_session_context_expiry() {
        let mut config = SessionConfig::default();
        config.ttl = Duration::from_millis(10);
        config.auto_renew = false;

        let session = RpcSessionContext::with_config(config);
        assert!(!session.is_expired());

        std::thread::sleep(Duration::from_millis(20));
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_context_touch() {
        let mut config = SessionConfig::default();
        config.ttl = Duration::from_millis(100);
        config.auto_renew = true;

        let mut session = RpcSessionContext::with_config(config);
        let initial_expiry = session.expires_at;

        std::thread::sleep(Duration::from_millis(20));
        session.touch();

        assert!(session.expires_at > initial_expiry);
    }

    #[test]
    fn test_session_record_request() {
        let mut session = RpcSessionContext::new();
        assert_eq!(session.state.request_count, 0);

        session.record_request("https://example.com", 1000);
        assert_eq!(session.state.request_count, 1);
        assert_eq!(
            session.state.last_url,
            Some("https://example.com".to_string())
        );
        assert_eq!(session.state.total_render_time_ms, 1000);
        assert_eq!(session.state.avg_render_time_ms, 1000.0);

        session.record_request("https://example.com/page2", 2000);
        assert_eq!(session.state.request_count, 2);
        assert_eq!(session.state.total_render_time_ms, 3000);
        assert_eq!(session.state.avg_render_time_ms, 1500.0);
    }

    #[test]
    fn test_session_metadata() {
        let mut session = RpcSessionContext::new();
        session.set_metadata("user_id".to_string(), "12345".to_string());
        session.set_metadata("theme".to_string(), "dark".to_string());

        assert_eq!(session.get_metadata("user_id"), Some(&"12345".to_string()));
        assert_eq!(session.get_metadata("theme"), Some(&"dark".to_string()));
        assert_eq!(session.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_session_store_get_or_create() {
        let store = RpcSessionStore::new();
        let session_id = "test_session_123";

        let session1 = store.get_or_create(session_id);
        assert_eq!(session1.session_id, session_id);
        assert_eq!(session1.state.request_count, 0);

        // Get the same session again
        let session2 = store.get_or_create(session_id);
        assert_eq!(session2.session_id, session_id);
    }

    #[test]
    fn test_session_store_update() {
        let store = RpcSessionStore::new();
        let mut session = store.get_or_create("test_session");

        session.record_request("https://example.com", 1000);
        assert!(store.update(session.clone()).is_ok());

        let retrieved = store.get("test_session").unwrap();
        assert_eq!(retrieved.state.request_count, 1);
    }

    #[test]
    fn test_session_store_remove() {
        let store = RpcSessionStore::new();
        let session_id = "test_session";

        store.get_or_create(session_id);
        assert_eq!(store.session_count(), 1);

        let removed = store.remove(session_id);
        assert!(removed.is_some());
        assert_eq!(store.session_count(), 0);
    }

    #[test]
    fn test_session_store_cleanup() {
        let mut config = SessionConfig::default();
        config.ttl = Duration::from_millis(50);
        config.auto_renew = false;

        let store = RpcSessionStore::with_config(config);

        // Create sessions
        store.get_or_create("session1");
        store.get_or_create("session2");
        assert_eq!(store.session_count(), 2);

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(100));

        // Cleanup should remove expired sessions
        let cleaned = store.cleanup_expired();
        assert_eq!(cleaned, 2);
        assert_eq!(store.session_count(), 0);
    }

    #[test]
    fn test_session_metrics() {
        let store = RpcSessionStore::new();

        let mut session1 = store.get_or_create("session1");
        session1.record_request("https://example.com", 1000);
        store.update(session1).unwrap();

        let mut session2 = store.get_or_create("session2");
        session2.record_request("https://example.com", 2000);
        session2.record_request("https://example.com/page2", 3000);
        store.update(session2).unwrap();

        let metrics = store.get_metrics();
        assert_eq!(metrics.active_sessions, 2);
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.total_render_time_ms, 6000);
        assert_eq!(metrics.avg_render_time_ms, 2000.0);
    }

    #[test]
    fn test_request_limit() {
        let mut config = SessionConfig::default();
        config.max_requests = 2;

        let mut session = RpcSessionContext::with_config(config);
        assert!(!session.is_request_limit_reached());

        session.record_request("https://example.com", 1000);
        assert!(!session.is_request_limit_reached());

        session.record_request("https://example.com", 1000);
        assert!(session.is_request_limit_reached());
    }
}
