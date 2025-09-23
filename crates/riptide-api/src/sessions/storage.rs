use super::types::{Session, SessionConfig, SessionError, SessionStats};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Thread-safe session storage with TTL and cleanup mechanisms
#[derive(Clone)]
pub struct SessionStorage {
    /// In-memory session cache for fast access
    sessions: Arc<RwLock<HashMap<String, Session>>>,

    /// Storage configuration
    config: SessionConfig,

    /// Statistics tracking
    stats: Arc<RwLock<SessionStorageStats>>,
}

/// Internal storage statistics
#[derive(Debug, Default)]
struct SessionStorageStats {
    /// Total sessions created
    sessions_created: u64,

    /// Total sessions expired
    sessions_expired: u64,

    /// Total cleanup operations
    cleanup_operations: u64,

    /// Last cleanup timestamp
    last_cleanup: Option<SystemTime>,

    /// Disk usage in bytes
    disk_usage_bytes: u64,
}

impl SessionStorage {
    /// Create a new session storage with the given configuration
    pub async fn new(config: SessionConfig) -> Result<Self> {
        // Ensure base directory exists
        if !config.base_data_dir.exists() {
            fs::create_dir_all(&config.base_data_dir).await.map_err(|e| {
                SessionError::DirectoryCreationFailed {
                    path: config.base_data_dir.to_string_lossy().to_string(),
                }
            })?;
            info!(
                path = %config.base_data_dir.display(),
                "Created session storage directory"
            );
        }

        let storage = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(SessionStorageStats::default())),
        };

        // Load existing sessions from disk
        storage.load_existing_sessions().await?;

        // Start background cleanup task
        storage.start_cleanup_task();

        Ok(storage)
    }

    /// Get a session by ID, loading from disk if not in memory
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        // First check memory cache
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                if session.is_expired() {
                    // Session is expired, remove it
                    drop(sessions);
                    self.remove_session(session_id).await?;
                    return Ok(None);
                }
                return Ok(Some(session.clone()));
            }
        }

        // Try to load from disk
        self.load_session_from_disk(session_id).await
    }

    /// Store or update a session
    pub async fn store_session(&self, mut session: Session) -> Result<(), SessionError> {
        // Update last accessed time
        session.touch(self.config.default_ttl);

        // Check session limit
        {
            let sessions = self.sessions.read().await;
            if sessions.len() >= self.config.max_sessions
                && !sessions.contains_key(&session.session_id) {
                return Err(SessionError::MaxSessionsReached {
                    max_sessions: self.config.max_sessions,
                });
            }
        }

        // Store in memory
        {
            let mut sessions = self.sessions.write().await;
            let is_new = !sessions.contains_key(&session.session_id);
            sessions.insert(session.session_id.clone(), session.clone());

            if is_new {
                let mut stats = self.stats.write().await;
                stats.sessions_created += 1;
            }
        }

        // Persist to disk
        self.persist_session_to_disk(&session).await?;

        debug!(
            session_id = %session.session_id,
            expires_at = ?session.expires_at,
            "Session stored successfully"
        );

        Ok(())
    }

    /// Create a new session with the given ID
    pub async fn create_session(&self, session_id: String) -> Result<Session, SessionError> {
        // Validate session ID format
        if session_id.is_empty() || session_id.len() > 128 {
            return Err(SessionError::InvalidSessionId { session_id });
        }

        let session = Session::new(session_id, &self.config);

        // Ensure user data directory exists
        if !session.user_data_dir.exists() {
            fs::create_dir_all(&session.user_data_dir).await.map_err(|e| {
                SessionError::DirectoryCreationFailed {
                    path: session.user_data_dir.to_string_lossy().to_string(),
                }
            })?;
        }

        self.store_session(session.clone()).await?;

        info!(
            session_id = %session.session_id,
            user_data_dir = %session.user_data_dir.display(),
            "Created new session"
        );

        Ok(session)
    }

    /// Remove a session from storage and disk
    pub async fn remove_session(&self, session_id: &str) -> Result<(), SessionError> {
        // Remove from memory
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(session) = session {
            // Remove from disk
            self.remove_session_from_disk(&session).await?;

            debug!(
                session_id = %session_id,
                "Session removed successfully"
            );
        }

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let now = SystemTime::now();
        let mut expired_sessions = Vec::new();

        // Find expired sessions
        {
            let sessions = self.sessions.read().await;
            for (id, session) in sessions.iter() {
                if session.expires_at <= now {
                    expired_sessions.push(id.clone());
                }
            }
        }

        // Remove expired sessions
        let mut removed_count = 0;
        for session_id in expired_sessions {
            if let Err(e) = self.remove_session(&session_id).await {
                warn!(
                    session_id = %session_id,
                    error = %e,
                    "Failed to remove expired session"
                );
            } else {
                removed_count += 1;
            }
        }

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.sessions_expired += removed_count as u64;
            stats.cleanup_operations += 1;
            stats.last_cleanup = Some(now);
        }

        if removed_count > 0 {
            info!(
                expired_count = removed_count,
                "Cleaned up expired sessions"
            );
        }

        Ok(removed_count)
    }

    /// Get session statistics
    pub async fn get_stats(&self) -> Result<SessionStats> {
        let sessions = self.sessions.read().await;
        let storage_stats = self.stats.read().await;

        // Calculate average session age
        let total_age: Duration = sessions
            .values()
            .map(|s| s.created_at.elapsed().unwrap_or_default())
            .sum();
        let avg_age_seconds = if sessions.is_empty() {
            0.0
        } else {
            total_age.as_secs_f64() / sessions.len() as f64
        };

        // Count sessions created in the last hour
        let one_hour_ago = SystemTime::now() - Duration::from_secs(3600);
        let sessions_created_last_hour = sessions
            .values()
            .filter(|s| s.created_at > one_hour_ago)
            .count();

        Ok(SessionStats {
            total_sessions: sessions.len(),
            expired_sessions_cleaned: storage_stats.sessions_expired as usize,
            total_disk_usage_bytes: storage_stats.disk_usage_bytes,
            avg_session_age_seconds: avg_age_seconds,
            sessions_created_last_hour,
        })
    }

    /// Get all active session IDs
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Load existing sessions from disk on startup
    async fn load_existing_sessions(&self) -> Result<()> {
        let mut loaded_count = 0;

        let mut entries = fs::read_dir(&self.config.base_data_dir).await.map_err(|e| {
            SessionError::IoError {
                error: format!("Failed to read session directory: {}", e),
            }
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            SessionError::IoError {
                error: format!("Failed to read directory entry: {}", e),
            }
        })? {
            let path = entry.path();
            if path.is_dir() {
                let session_file = path.join("session.json");
                if session_file.exists() {
                    match self.load_session_file(&session_file).await {
                        Ok(Some(session)) => {
                            if !session.is_expired() {
                                let mut sessions = self.sessions.write().await;
                                sessions.insert(session.session_id.clone(), session);
                                loaded_count += 1;
                            }
                        }
                        Ok(None) => {
                            // Session was expired or invalid, ignore
                        }
                        Err(e) => {
                            warn!(
                                path = %session_file.display(),
                                error = %e,
                                "Failed to load session file"
                            );
                        }
                    }
                }
            }
        }

        if loaded_count > 0 {
            info!(
                loaded_count = loaded_count,
                "Loaded existing sessions from disk"
            );
        }

        Ok(())
    }

    /// Load a specific session from disk
    async fn load_session_from_disk(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        let session_file = self.config.base_data_dir
            .join(session_id)
            .join("session.json");

        if !session_file.exists() {
            return Ok(None);
        }

        self.load_session_file(&session_file).await
    }

    /// Load session from a specific file
    async fn load_session_file(&self, session_file: &PathBuf) -> Result<Option<Session>, SessionError> {
        let content = fs::read_to_string(session_file).await.map_err(|e| {
            SessionError::IoError {
                error: format!("Failed to read session file: {}", e),
            }
        })?;

        let session: Session = serde_json::from_str(&content).map_err(|e| {
            SessionError::DeserializationError {
                error: format!("Failed to deserialize session: {}", e),
            }
        })?;

        if session.is_expired() {
            // Clean up expired session
            if let Err(e) = self.remove_session_from_disk(&session).await {
                warn!(
                    session_id = %session.session_id,
                    error = %e,
                    "Failed to remove expired session from disk"
                );
            }
            return Ok(None);
        }

        // Add to memory cache
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session.session_id.clone(), session.clone());
        }

        debug!(
            session_id = %session.session_id,
            "Loaded session from disk"
        );

        Ok(Some(session))
    }

    /// Persist session to disk
    async fn persist_session_to_disk(&self, session: &Session) -> Result<(), SessionError> {
        let session_dir = &session.user_data_dir;
        let session_file = session_dir.join("session.json");

        // Ensure directory exists
        fs::create_dir_all(session_dir).await.map_err(|e| {
            SessionError::DirectoryCreationFailed {
                path: session_dir.to_string_lossy().to_string(),
            }
        })?;

        // Serialize session data
        let content = serde_json::to_string_pretty(session).map_err(|e| {
            SessionError::SerializationError {
                error: format!("Failed to serialize session: {}", e),
            }
        })?;

        // Write to file
        fs::write(&session_file, content).await.map_err(|e| {
            SessionError::IoError {
                error: format!("Failed to write session file: {}", e),
            }
        })?;

        debug!(
            session_id = %session.session_id,
            file = %session_file.display(),
            "Session persisted to disk"
        );

        Ok(())
    }

    /// Remove session data from disk
    async fn remove_session_from_disk(&self, session: &Session) -> Result<(), SessionError> {
        if session.user_data_dir.exists() {
            fs::remove_dir_all(&session.user_data_dir).await.map_err(|e| {
                SessionError::IoError {
                    error: format!("Failed to remove session directory: {}", e),
                }
            })?;

            debug!(
                session_id = %session.session_id,
                dir = %session.user_data_dir.display(),
                "Session directory removed from disk"
            );
        }

        Ok(())
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let storage = self.clone();
        let cleanup_interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                if let Err(e) = storage.cleanup_expired().await {
                    error!(
                        error = %e,
                        "Background cleanup task failed"
                    );
                }
            }
        });

        debug!(
            interval_seconds = cleanup_interval.as_secs(),
            "Started background session cleanup task"
        );
    }
}