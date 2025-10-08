//! Session types and data structures
//!
//! ACTIVELY USED: Core types for session management system integrated throughout the API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Base directory for storing session data
    pub base_data_dir: PathBuf,

    /// Default session TTL in seconds
    pub default_ttl: Duration,

    /// Maximum number of concurrent sessions
    pub max_sessions: usize,

    /// Cleanup interval for expired sessions
    pub cleanup_interval: Duration,

    /// Whether to persist cookies to disk
    pub persist_cookies: bool,

    /// Whether to enable session encryption
    #[allow(dead_code)] // Future feature - encryption not yet implemented
    pub encrypt_session_data: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            base_data_dir: PathBuf::from("/tmp/riptide-sessions"),
            default_ttl: Duration::from_secs(3600 * 24), // 24 hours
            max_sessions: 1000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            persist_cookies: true,
            encrypt_session_data: false,
        }
    }
}

/// Represents a browser session with persistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub session_id: String,

    /// Session creation timestamp
    pub created_at: SystemTime,

    /// Last access timestamp
    pub last_accessed: SystemTime,

    /// Session expiry time
    pub expires_at: SystemTime,

    /// Browser user data directory path
    pub user_data_dir: PathBuf,

    /// Cookie storage
    pub cookies: CookieJar,

    /// Session metadata
    pub metadata: SessionMetadata,

    /// Browser profile configuration
    pub browser_config: BrowserConfig,
}

/// Cookie jar for storing HTTP cookies
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CookieJar {
    /// Domain-indexed cookie storage
    pub cookies: HashMap<String, HashMap<String, Cookie>>,
}

/// HTTP Cookie representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    /// Cookie name
    pub name: String,

    /// Cookie value
    pub value: String,

    /// Cookie domain
    pub domain: Option<String>,

    /// Cookie path
    pub path: Option<String>,

    /// Cookie expiry
    pub expires: Option<SystemTime>,

    /// Secure flag
    pub secure: bool,

    /// HttpOnly flag
    pub http_only: bool,

    /// SameSite attribute
    pub same_site: Option<SameSite>,
}

/// SameSite cookie attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionMetadata {
    /// User agent string used for this session
    pub user_agent: Option<String>,

    /// Viewport size
    pub viewport: Option<Viewport>,

    /// Browser locale
    pub locale: Option<String>,

    /// Timezone
    pub timezone: Option<String>,

    /// Custom tags for session organization
    pub tags: Vec<String>,

    /// Session usage statistics
    pub stats: SessionUsageStats,
}

/// Browser configuration for the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Browser type (chrome, firefox, etc.)
    pub browser_type: BrowserType,

    /// Browser arguments
    pub args: Vec<String>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Whether to run in headless mode
    pub headless: bool,

    /// Custom browser executable path
    pub executable_path: Option<PathBuf>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            browser_type: BrowserType::Chrome,
            args: vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                "--disable-web-security".to_string(),
                "--allow-running-insecure-content".to_string(),
            ],
            env: HashMap::new(),
            headless: true,
            executable_path: None,
        }
    }
}

/// Supported browser types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
}

/// Viewport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
        }
    }
}

/// Session usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionUsageStats {
    /// Number of requests made with this session
    pub request_count: u64,

    /// Total bytes downloaded
    pub bytes_downloaded: u64,

    /// Number of pages visited
    pub pages_visited: u64,

    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,

    /// Last used URL
    pub last_url: Option<String>,
}

/// Session statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of active sessions
    pub total_sessions: usize,

    /// Number of expired sessions cleaned up
    pub expired_sessions_cleaned: usize,

    /// Total disk space used by sessions
    pub total_disk_usage_bytes: u64,

    /// Average session age in seconds
    pub avg_session_age_seconds: f64,

    /// Number of sessions created in the last hour
    pub sessions_created_last_hour: usize,
}

/// Session store errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session expired: {session_id}")]
    #[allow(dead_code)] // Public API - will be used in session expiration logic
    SessionExpired { session_id: String },

    #[error("Maximum number of sessions reached: {max_sessions}")]
    MaxSessionsReached { max_sessions: usize },

    #[error("Failed to create session directory: {path}")]
    DirectoryCreationFailed { path: String },

    #[error("Failed to serialize session data: {error}")]
    SerializationError { error: String },

    #[error("Failed to deserialize session data: {error}")]
    DeserializationError { error: String },

    #[error("IO error: {error}")]
    IoError { error: String },

    #[error("Invalid session ID format: {session_id}")]
    InvalidSessionId { session_id: String },
}

impl Session {
    /// Create a new session with the given ID and configuration
    pub fn new(session_id: String, config: &SessionConfig) -> Self {
        let now = SystemTime::now();
        let expires_at = now + config.default_ttl;
        let user_data_dir = config.base_data_dir.join(&session_id);

        Self {
            session_id,
            created_at: now,
            last_accessed: now,
            expires_at,
            user_data_dir,
            cookies: CookieJar::default(),
            metadata: SessionMetadata::default(),
            browser_config: BrowserConfig::default(),
        }
    }

    /// Generate a new unique session ID
    pub fn generate_session_id() -> String {
        format!("session_{}", Uuid::new_v4().simple())
    }

    /// Check if the session has expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Update the last accessed time and extend expiry
    pub fn touch(&mut self, ttl: Duration) {
        let now = SystemTime::now();
        self.last_accessed = now;
        self.expires_at = now + ttl;
    }

    /// Get the user data directory path
    #[allow(dead_code)] // Public API - used for session directory access
    pub fn get_user_data_dir(&self) -> &PathBuf {
        &self.user_data_dir
    }
}

impl CookieJar {
    /// Add a cookie to the jar
    pub fn set_cookie(&mut self, domain: &str, cookie: Cookie) {
        self.cookies
            .entry(domain.to_string())
            .or_default()
            .insert(cookie.name.clone(), cookie);
    }

    /// Get a cookie by domain and name
    pub fn get_cookie(&self, domain: &str, name: &str) -> Option<&Cookie> {
        self.cookies.get(domain)?.get(name)
    }

    /// Get all cookies for a domain
    pub fn get_cookies_for_domain(&self, domain: &str) -> Option<&HashMap<String, Cookie>> {
        self.cookies.get(domain)
    }

    /// Remove a cookie
    pub fn remove_cookie(&mut self, domain: &str, name: &str) -> Option<Cookie> {
        self.cookies.get_mut(domain)?.remove(name)
    }

    /// Clear all cookies
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Get all cookies as a flat list
    pub fn all_cookies(&self) -> Vec<&Cookie> {
        self.cookies
            .values()
            .flat_map(|domain_cookies| domain_cookies.values())
            .collect()
    }

    /// Remove expired cookies
    #[allow(dead_code)] // Public API - should be called during cleanup
    pub fn remove_expired(&mut self) {
        let now = SystemTime::now();
        for domain_cookies in self.cookies.values_mut() {
            domain_cookies.retain(|_, cookie| cookie.expires.is_none_or(|expires| expires > now));
        }

        // Remove empty domains
        self.cookies
            .retain(|_, domain_cookies| !domain_cookies.is_empty());
    }
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            domain: None,
            path: None,
            expires: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Create a cookie with domain
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Create a cookie with path
    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Create a cookie with expiry
    pub fn with_expires(mut self, expires: SystemTime) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Mark cookie as secure
    pub fn secure(mut self) -> Self {
        self.secure = true;
        self
    }

    /// Mark cookie as HTTP only
    pub fn http_only(mut self) -> Self {
        self.http_only = true;
        self
    }

    /// Set SameSite attribute
    #[allow(dead_code)] // Public API - builder pattern for cookie configuration
    pub fn with_same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Check if the cookie has expired
    #[allow(dead_code)] // Public API - used for cookie validation
    pub fn is_expired(&self) -> bool {
        self.expires
            .is_some_and(|expires| SystemTime::now() > expires)
    }
}
