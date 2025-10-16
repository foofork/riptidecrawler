use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session data structure for managing cookies, headers, and authentication state
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub cookies: Vec<Cookie>,
    pub headers: HashMap<String, String>,
    pub tags: Vec<String>,
    pub user_agent: Option<String>,
    pub timeout_minutes: u64,
    pub metadata: SessionMetadata,
    pub storage_state: Option<BrowserStorageState>,
}

/// Cookie data structure compatible with browser cookie format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub secure: bool,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub expires: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
}

/// Session metadata for tracking usage statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionMetadata {
    pub requests_count: u64,
    pub last_request_url: Option<String>,
    pub success_count: u64,
    pub error_count: u64,
}

/// Browser storage state (localStorage and sessionStorage)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserStorageState {
    #[serde(rename = "localStorage")]
    pub local_storage: HashMap<String, String>,
    #[serde(rename = "sessionStorage")]
    pub session_storage: HashMap<String, String>,
    pub origins: Vec<OriginStorage>,
}

/// Storage state for a specific origin
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OriginStorage {
    pub origin: String,
    #[serde(rename = "localStorage")]
    pub local_storage: Vec<StorageEntry>,
    #[serde(rename = "sessionStorage")]
    pub session_storage: Vec<StorageEntry>,
}

/// Individual storage entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageEntry {
    pub name: String,
    pub value: String,
}

impl Session {
    /// Create a new session with default values
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_used_at: None,
            cookies: Vec::new(),
            headers: HashMap::new(),
            tags: Vec::new(),
            user_agent: None,
            timeout_minutes: 0,
            metadata: SessionMetadata {
                requests_count: 0,
                last_request_url: None,
                success_count: 0,
                error_count: 0,
            },
            storage_state: None,
        }
    }

    /// Check if session is expired based on timeout
    pub fn is_expired(&self) -> bool {
        if self.timeout_minutes == 0 {
            return false;
        }

        if let Some(last_used) = self.last_used_at {
            let elapsed = Utc::now().signed_duration_since(last_used);
            elapsed.num_minutes() >= self.timeout_minutes as i64
        } else {
            false
        }
    }

    /// Update last used timestamp
    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Convert cookies to browser-compatible format
    pub fn to_cookie_jar(&self) -> Vec<Cookie> {
        self.cookies.clone()
    }

    /// Import cookies from browser cookie jar format
    pub fn from_cookie_jar(name: String, cookies: Vec<Cookie>) -> Self {
        let mut session = Self::new(name);
        session.cookies = cookies;
        session
    }
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            requests_count: 0,
            last_request_url: None,
            success_count: 0,
            error_count: 0,
        }
    }
}
