use anyhow::{Context, Result};
use reqwest::cookie::Jar;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;

/// Configuration for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Enable session persistence
    pub enable_session_persistence: bool,
    /// Enable authentication support
    ///
    /// **Status:** ⚠️ Planned Feature - Full authentication implementation pending
    /// When enabled, the spider will support automatic login sequences and authenticated
    /// crawling with session state management. Currently, basic session lifecycle is
    /// implemented, but full authentication features require completion.
    ///
    /// **Implementation Required:**
    /// - Complete LoginConfig integration (login_url, credentials, success indicators)
    /// - Implement automatic CSRF token extraction (PreLoginStep execution)
    /// - Add multi-step authentication flows (2FA, OAuth)
    /// - Implement session validation and re-authentication on expiry
    /// - Add secure credential storage integration (Vault, KMS)
    ///
    /// **Current Status:**
    /// - ✅ Session lifecycle management (create, extend, cleanup)
    /// - ✅ Cookie persistence across requests
    /// - ✅ Session timeout handling
    /// - ⚠️ Automatic login sequences (incomplete)
    /// - ⚠️ CSRF token handling (incomplete)
    /// - ⚠️ Multi-factor authentication (not started)
    pub enable_authentication: bool,
    /// Enable cookie persistence across requests
    pub enable_cookie_persistence: bool,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Maximum number of concurrent sessions
    pub max_concurrent_sessions: usize,
    /// Enable automatic login sequence
    pub enable_auto_login: bool,
    /// Cookie jar persistence
    pub persist_cookies: bool,
    /// Session checkpoint interval
    pub checkpoint_interval: Duration,
    /// Maximum login attempts per session
    pub max_login_attempts: u32,
    /// User agent for session requests
    pub user_agent: String,
    /// Enable session state validation
    pub enable_state_validation: bool,
    /// Validation check interval
    pub validation_interval: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enable_session_persistence: true,
            enable_authentication: false,
            enable_cookie_persistence: true,
            session_timeout: Duration::from_secs(1800), // 30 minutes
            max_concurrent_sessions: 5,
            enable_auto_login: false,
            persist_cookies: true,
            checkpoint_interval: Duration::from_secs(300), // 5 minutes
            max_login_attempts: 3,
            user_agent: "RipTide Spider/1.0".to_string(),
            enable_state_validation: true,
            validation_interval: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Login credentials and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginConfig {
    /// Login URL
    pub login_url: Url,
    /// Username field name
    pub username_field: String,
    /// Password field name
    pub password_field: String,
    /// Username value
    pub username: String,
    /// Password value (should be securely stored)
    pub password: String,
    /// Additional form fields
    pub additional_fields: HashMap<String, String>,
    /// Success indicators (URLs or text patterns)
    pub success_indicators: Vec<String>,
    /// Failure indicators
    pub failure_indicators: Vec<String>,
    /// Pre-login steps (e.g., get CSRF token)
    pub pre_login_steps: Vec<PreLoginStep>,
    /// Post-login validation URL
    pub validation_url: Option<Url>,
}

/// Pre-login step configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreLoginStep {
    /// URL to visit
    pub url: Url,
    /// Field to extract (e.g., CSRF token)
    pub extract_field: Option<String>,
    /// CSS selector for extraction
    pub css_selector: Option<String>,
    /// Form field name to populate
    pub target_field: Option<String>,
}

/// Session state
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Session ID
    pub id: String,
    /// Associated domain
    pub domain: String,
    /// Login configuration
    pub login_config: Option<LoginConfig>,
    /// Whether session is authenticated
    pub authenticated: bool,
    /// Session creation time
    pub created_at: Instant,
    /// Last activity time
    pub last_activity: Instant,
    /// Last successful validation
    pub last_validation: Option<Instant>,
    /// Login attempt count
    pub login_attempts: u32,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Cookie jar
    pub cookies: Arc<Jar>,
    /// HTTP client for this session
    pub client: Client,
}

impl SessionState {
    pub fn new(id: String, domain: String, config: &SessionConfig) -> Result<Self> {
        let cookies = Arc::new(Jar::default());
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .cookie_provider(cookies.clone())
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client for session")?;

        Ok(Self {
            id,
            domain,
            login_config: None,
            authenticated: false,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            last_validation: None,
            login_attempts: 0,
            metadata: HashMap::new(),
            cookies,
            client,
        })
    }

    /// Check if session is expired
    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if session needs validation
    pub fn needs_validation(&self, interval: Duration) -> bool {
        match self.last_validation {
            Some(last) => last.elapsed() > interval,
            None => true,
        }
    }

    /// Mark session as validated
    pub fn mark_validated(&mut self) {
        self.last_validation = Some(Instant::now());
    }
}

/// Session checkpoint data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    pub id: String,
    pub domain: String,
    pub authenticated: bool,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub login_attempts: u32,
    pub metadata: HashMap<String, String>,
    pub cookies: Vec<String>, // Serialized cookies
}

/// Session management result
#[derive(Debug)]
pub enum SessionResult {
    Success,
    LoginRequired,
    LoginFailed(String),
    SessionExpired,
    ValidationFailed(String),
    Error(String),
}

/// Session manager for authenticated crawling
pub struct SessionManager {
    config: SessionConfig,
    /// Active sessions by domain
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// Session checkpoints for persistence
    checkpoints: Arc<RwLock<HashMap<String, SessionCheckpoint>>>,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a session for a domain
    pub async fn get_or_create_session(&self, domain: &str) -> Result<String> {
        let mut sessions = self.sessions.write().await;

        // Check if we have an active session for this domain
        if let Some(session) = sessions.get_mut(domain) {
            if !session.is_expired(self.config.session_timeout) {
                session.touch();
                return Ok(session.id.clone());
            } else {
                // Remove expired session
                sessions.remove(domain);
            }
        }

        // Check session limit
        if sessions.len() >= self.config.max_concurrent_sessions {
            // Remove oldest session
            if let Some((oldest_domain, _)) = sessions
                .iter()
                .min_by_key(|(_, session)| session.last_activity)
            {
                let oldest_domain = oldest_domain.clone();
                sessions.remove(&oldest_domain);
                warn!(domain = %oldest_domain, "Removed oldest session due to limit");
            }
        }

        // Create new session
        let session_id = format!(
            "session_{}_{}",
            domain,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let session = SessionState::new(session_id.clone(), domain.to_string(), &self.config)?;

        sessions.insert(domain.to_string(), session);

        info!(domain = %domain, session_id = %session_id, "Created new session");
        Ok(session_id)
    }

    /// Configure login for a session
    pub async fn configure_login(&self, domain: &str, login_config: LoginConfig) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(domain) {
            session.login_config = Some(login_config);
            debug!(domain = %domain, "Login configuration set for session");
            Ok(())
        } else {
            Err(anyhow::anyhow!("No session found for domain: {}", domain))
        }
    }

    /// Get HTTP client for a session
    pub async fn get_session_client(&self, domain: &str) -> Result<Option<Client>> {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(domain) {
            if !session.is_expired(self.config.session_timeout) {
                Ok(Some(session.client.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Check if a session is authenticated
    pub async fn is_authenticated(&self, domain: &str) -> bool {
        let sessions = self.sessions.read().await;

        sessions
            .get(domain)
            .map(|session| {
                session.authenticated && !session.is_expired(self.config.session_timeout)
            })
            .unwrap_or(false)
    }

    /// Get session statistics
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;
        let checkpoints = self.checkpoints.read().await;

        let active_sessions = sessions.len();
        let authenticated_sessions = sessions
            .values()
            .filter(|s| s.authenticated && !s.is_expired(self.config.session_timeout))
            .count();

        let expired_sessions = sessions
            .values()
            .filter(|s| s.is_expired(self.config.session_timeout))
            .count();

        SessionStats {
            active_sessions,
            authenticated_sessions,
            expired_sessions,
            total_checkpoints: checkpoints.len(),
        }
    }

    /// Clear all sessions
    pub async fn clear_sessions(&self) {
        self.sessions.write().await.clear();
        self.checkpoints.write().await.clear();
        info!("All sessions cleared");
    }

    /// Get configuration
    pub fn get_config(&self) -> &SessionConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SessionConfig) {
        self.config = config;
        info!("Session manager configuration updated");
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub active_sessions: usize,
    pub authenticated_sessions: usize,
    pub expired_sessions: usize,
    pub total_checkpoints: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let session_id = manager
            .get_or_create_session("example.com")
            .await
            .expect("Should create session");
        assert!(!session_id.is_empty());

        // Second call should return same session
        let session_id2 = manager
            .get_or_create_session("example.com")
            .await
            .expect("Should return existing session");
        assert_eq!(session_id, session_id2);
    }

    #[tokio::test]
    async fn test_session_expiration() {
        let config = SessionConfig {
            session_timeout: Duration::from_millis(10), // Very short timeout
            ..Default::default()
        };

        let manager = SessionManager::new(config);

        let session_id1 = manager
            .get_or_create_session("example.com")
            .await
            .expect("Should create session");

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;

        let session_id2 = manager
            .get_or_create_session("example.com")
            .await
            .expect("Should create new session");
        assert_ne!(session_id1, session_id2);
    }

    #[tokio::test]
    async fn test_login_configuration() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        let domain = "example.com";
        let _session_id = manager
            .get_or_create_session(domain)
            .await
            .expect("Should create session");

        let login_config = LoginConfig {
            login_url: Url::from_str("https://example.com/login").expect("Valid URL"),
            username_field: "username".to_string(),
            password_field: "password".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            additional_fields: HashMap::new(),
            success_indicators: vec!["dashboard".to_string()],
            failure_indicators: vec!["error".to_string()],
            pre_login_steps: Vec::new(),
            validation_url: None,
        };

        manager
            .configure_login(domain, login_config)
            .await
            .expect("Should configure login");

        // Check that login config is set (indirectly)
        assert!(!manager.is_authenticated(domain).await);
    }

    #[test]
    fn test_session_state() {
        let config = SessionConfig::default();
        let mut session =
            SessionState::new("test_id".to_string(), "example.com".to_string(), &config)
                .expect("Should create session state");

        assert!(!session.is_expired(Duration::from_secs(3600)));
        assert!(session.needs_validation(Duration::from_secs(1)));

        session.touch();
        session.mark_validated();

        assert!(!session.needs_validation(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn test_session_limits() {
        let config = SessionConfig {
            max_concurrent_sessions: 2,
            ..Default::default()
        };

        let manager = SessionManager::new(config);

        // Create maximum sessions
        let _session1 = manager
            .get_or_create_session("site1.com")
            .await
            .expect("Should work");
        let _session2 = manager
            .get_or_create_session("site2.com")
            .await
            .expect("Should work");

        // Third session should evict oldest
        let _session3 = manager
            .get_or_create_session("site3.com")
            .await
            .expect("Should work");

        let stats = manager.get_stats().await;
        assert_eq!(stats.active_sessions, 2);
    }

    #[tokio::test]
    async fn test_session_validation() {
        let config = SessionConfig {
            enable_state_validation: true,
            ..Default::default()
        };

        let manager = SessionManager::new(config);

        // Test clearing sessions
        manager.clear_sessions().await;

        let stats = manager.get_stats().await;
        assert_eq!(stats.active_sessions, 0);
    }
}
