/*!
# State Management

Comprehensive state persistence, session management, configuration hot-reload,
and checkpoint/restore capabilities for RipTide persistence layer.
*/

use crate::{
    config::StateConfig,
    errors::{PersistenceError, PersistenceResult},
};
use chrono::{DateTime, Utc};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Comprehensive state manager
pub struct StateManager {
    /// Redis connection for persistence
    conn: Arc<Mutex<MultiplexedConnection>>,
    /// Configuration
    config: StateConfig,
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// Configuration manager
    config_manager: Arc<ConfigurationManager>,
    /// Checkpoint manager
    checkpoint_manager: Arc<CheckpointManager>,
    /// Hot reload watcher
    hot_reload_watcher: Option<Arc<HotReloadWatcher>>,
    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,
    /// Session spillover manager for disk overflow
    spillover_manager: Arc<SessionSpilloverManager>,
    /// Memory usage tracker
    memory_tracker: Arc<MemoryTracker>,
}

/// Session state with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
    /// Session data
    pub data: HashMap<String, serde_json::Value>,
    /// User/tenant ID associated with session
    pub user_id: Option<String>,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// TTL in seconds
    pub ttl_seconds: u64,
    /// Session status
    pub status: SessionStatus,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Client IP address
    pub client_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Session source
    pub source: Option<String>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Expired,
    Terminated,
}

/// System checkpoint for state preservation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint ID
    pub id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Checkpoint type
    pub checkpoint_type: CheckpointType,
    /// State snapshot
    pub state_snapshot: StateSnapshot,
    /// Metadata
    pub metadata: CheckpointMetadata,
    /// Compression info
    pub compression: Option<CompressionInfo>,
}

/// Types of checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointType {
    /// Scheduled checkpoint
    Scheduled,
    /// Manual checkpoint
    Manual,
    /// Shutdown checkpoint
    Shutdown,
    /// Emergency checkpoint
    Emergency,
}

/// State snapshot containing all persistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Active sessions
    pub sessions: HashMap<String, SessionState>,
    /// Configuration state
    pub configuration: HashMap<String, serde_json::Value>,
    /// Cache statistics
    pub cache_stats: HashMap<String, serde_json::Value>,
    /// Tenant information
    pub tenant_data: HashMap<String, serde_json::Value>,
    /// System metrics
    pub system_metrics: HashMap<String, serde_json::Value>,
}

/// Checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Version of the checkpoint format
    pub version: String,
    /// Size in bytes
    pub size_bytes: usize,
    /// CRC32 checksum for integrity
    pub checksum: u32,
    /// Node ID that created the checkpoint
    pub created_by: String,
    /// Description
    pub description: Option<String>,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Algorithm used
    pub algorithm: String,
    /// Original size
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Compression ratio
    pub ratio: f32,
}

impl StateManager {
    /// Create new state manager
    pub async fn new(redis_url: &str, config: StateConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;

        let config_manager = Arc::new(ConfigurationManager::new(config.clone()).await?);

        let checkpoint_manager = Arc::new(CheckpointManager::new(config.clone()).await?);

        // Initialize spillover manager
        let spillover_dir = PathBuf::from("./data/sessions/spillover");
        let spillover_manager = Arc::new(SessionSpilloverManager::new(spillover_dir).await?);

        // Initialize memory tracker (default 100MB max, 80% warning threshold)
        let max_memory = 100 * 1024 * 1024; // 100MB
        let memory_tracker = Arc::new(MemoryTracker::new(max_memory, 0.80));

        let (shutdown_tx, _) = broadcast::channel(1);

        let mut state_manager = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config_manager,
            checkpoint_manager,
            hot_reload_watcher: None,
            shutdown_tx,
            spillover_manager,
            memory_tracker,
        };

        // Initialize hot reload if enabled
        if config.enable_hot_reload {
            let watcher = HotReloadWatcher::new(
                config.config_watch_paths.clone(),
                state_manager.config_manager.clone(),
            )
            .await?;
            state_manager.hot_reload_watcher = Some(Arc::new(watcher));
        }

        // Start background tasks
        state_manager.start_background_tasks().await;

        info!(
            session_timeout = config.session_timeout_seconds,
            hot_reload = config.enable_hot_reload,
            checkpoint_interval = config.checkpoint_interval_seconds,
            spillover_enabled = true,
            "State manager initialized"
        );

        Ok(state_manager)
    }

    /// Start background maintenance tasks
    async fn start_background_tasks(&self) {
        let sessions = Arc::clone(&self.sessions);
        let config = self.config.clone();

        // Session cleanup task
        let cleanup_sessions = sessions.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute
            loop {
                interval.tick().await;
                Self::cleanup_expired_sessions(&cleanup_sessions, config.session_timeout_seconds)
                    .await;
            }
        });

        // Checkpoint task
        if self.config.checkpoint_interval_seconds > 0 {
            let _checkpoint_manager = Arc::clone(&self.checkpoint_manager);
            let checkpoint_interval = self.config.checkpoint_interval_seconds;
            let state_manager_weak = Arc::downgrade(&Arc::new(self.clone()));

            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(checkpoint_interval));
                loop {
                    interval.tick().await;
                    if let Some(state_manager) = state_manager_weak.upgrade() {
                        if let Err(e) = state_manager
                            .create_checkpoint(CheckpointType::Scheduled, None)
                            .await
                        {
                            error!(error = %e, "Failed to create scheduled checkpoint");
                        }
                    } else {
                        break; // StateManager has been dropped
                    }
                }
            });
        }

        // Disk spillover monitoring task
        let spillover_sessions = Arc::clone(&self.sessions);
        let spillover_manager = Arc::clone(&self.spillover_manager);
        let memory_tracker = Arc::clone(&self.memory_tracker);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
            loop {
                interval.tick().await;

                // Check if memory usage exceeds threshold
                if memory_tracker.should_spill().await {
                    let usage_pct = memory_tracker.get_usage_percentage().await;
                    warn!(
                        memory_usage_pct = usage_pct,
                        "Memory threshold exceeded, initiating spillover"
                    );

                    // Get LRU sessions for eviction
                    let lru_sessions = spillover_manager.get_lru_sessions(10).await;

                    // Spill sessions to disk
                    let sessions_read = spillover_sessions.read().await;
                    for session_id in lru_sessions {
                        if let Some(session) = sessions_read.get(&session_id) {
                            if let Err(e) =
                                spillover_manager.spill_session(&session_id, session).await
                            {
                                error!(
                                    session_id = %session_id,
                                    error = %e,
                                    "Failed to spill session to disk"
                                );
                            } else {
                                // Update memory tracker (decrease usage)
                                let session_size = MemoryTracker::estimate_session_size(session);
                                memory_tracker.update_usage(-(session_size as i64)).await;
                            }
                        }
                    }

                    // Remove spilled sessions from memory after successful spillover
                    drop(sessions_read);
                    let mut sessions_write = spillover_sessions.write().await;
                    for session_id in spillover_manager.get_lru_sessions(10).await {
                        sessions_write.remove(&session_id);
                    }
                }
            }
        });
    }

    /// Create new session
    pub async fn create_session(
        &self,
        user_id: Option<String>,
        metadata: SessionMetadata,
        ttl_seconds: Option<u64>,
    ) -> PersistenceResult<String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let user_id_clone = user_id.clone();
        let session = SessionState {
            id: session_id.clone(),
            created_at: now,
            last_accessed: now,
            data: HashMap::new(),
            user_id,
            metadata,
            ttl_seconds: ttl_seconds.unwrap_or(self.config.session_timeout_seconds),
            status: SessionStatus::Active,
        };

        // Store in Redis
        let session_key = format!("riptide:session:{}", session_id);
        let session_data = serde_json::to_vec(&session)?;

        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(&session_key, &session_data, session.ttl_seconds)
            .await?;

        let ttl_seconds = session.ttl_seconds;

        // Store in memory
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        debug!(
            session_id = %session_id,
            user_id = ?user_id_clone,
            ttl_seconds = ttl_seconds,
            "Session created"
        );

        Ok(session_id)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> PersistenceResult<Option<SessionState>> {
        // Try memory first
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(session_id) {
                if session.status == SessionStatus::Active {
                    // Update LRU access tracker
                    self.spillover_manager.update_access(session_id).await;
                    // Update last accessed
                    self.update_session_access(session_id).await?;
                    return Ok(Some(session.clone()));
                }
            }
        }

        // Try disk spillover storage
        if let Some(spilled_session) = self.spillover_manager.restore_session(session_id).await? {
            debug!(session_id = %session_id, "Session restored from disk spillover");

            // Check if expired
            let age = Utc::now().signed_duration_since(spilled_session.created_at);
            if age.num_seconds() > spilled_session.ttl_seconds as i64 {
                self.spillover_manager
                    .remove_spilled_session(session_id)
                    .await?;
                return Ok(None);
            }

            // Restore to memory
            let session_size = MemoryTracker::estimate_session_size(&spilled_session);
            self.memory_tracker.update_usage(session_size as i64).await;

            {
                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.to_string(), spilled_session.clone());
            }

            // Update LRU access tracker
            self.spillover_manager.update_access(session_id).await;

            // Remove from disk after successful restore
            self.spillover_manager
                .remove_spilled_session(session_id)
                .await?;

            return Ok(Some(spilled_session));
        }

        // Try Redis
        let session_key = format!("riptide:session:{}", session_id);
        let mut conn = self.conn.lock().await;
        let session_data: Option<Vec<u8>> = conn.get(&session_key).await?;

        if let Some(data) = session_data {
            let mut session: SessionState = serde_json::from_slice(&data)?;

            // Check if expired
            let age = Utc::now().signed_duration_since(session.created_at);
            if age.num_seconds() > session.ttl_seconds as i64 {
                session.status = SessionStatus::Expired;
                self.terminate_session(session_id).await?;
                return Ok(None);
            }

            // Update last accessed
            session.last_accessed = Utc::now();
            self.update_session(session_id, &session).await?;

            // Update LRU access tracker
            self.spillover_manager.update_access(session_id).await;

            debug!(session_id = %session_id, "Session retrieved from Redis");
            Ok(Some(session))
        } else {
            debug!(session_id = %session_id, "Session not found");
            Ok(None)
        }
    }

    /// Update session data
    pub async fn update_session_data(
        &self,
        session_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> PersistenceResult<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.data.insert(key.to_string(), value);
            session.last_accessed = Utc::now();
            self.update_session(session_id, &session).await?;

            debug!(
                session_id = %session_id,
                key = %key,
                "Session data updated"
            );
            Ok(())
        } else {
            Err(PersistenceError::state("Session not found"))
        }
    }

    /// Update entire session
    async fn update_session(
        &self,
        session_id: &str,
        session: &SessionState,
    ) -> PersistenceResult<()> {
        let session_key = format!("riptide:session:{}", session_id);
        let session_data = serde_json::to_vec(session)?;

        let mut conn = self.conn.lock().await;
        conn.set_ex::<_, _, ()>(&session_key, &session_data, session.ttl_seconds)
            .await?;

        // Update memory
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), session.clone());
        }

        Ok(())
    }

    /// Update session last accessed time
    async fn update_session_access(&self, session_id: &str) -> PersistenceResult<()> {
        // Avoid recursion by directly accessing the session storage
        let session_key = format!("riptide:session:{}", session_id);
        let mut conn = self.conn.lock().await;
        let session_data: Option<Vec<u8>> = conn.get(&session_key).await?;

        if let Some(data) = session_data {
            if let Ok(mut session) = serde_json::from_slice::<SessionState>(&data) {
                session.last_accessed = Utc::now();
                let updated_data = serde_json::to_vec(&session)?;
                conn.set_ex::<_, _, ()>(&session_key, &updated_data, session.ttl_seconds)
                    .await?;
            }
        }
        Ok(())
    }

    /// Terminate session
    pub async fn terminate_session(&self, session_id: &str) -> PersistenceResult<bool> {
        let session_key = format!("riptide:session:{}", session_id);

        // Remove from Redis
        let mut conn = self.conn.lock().await;
        let deleted: u64 = conn.del(&session_key).await?;

        // Remove from memory
        let session_size = {
            let mut sessions = self.sessions.write().await;
            let session_size = sessions
                .get(session_id)
                .map(MemoryTracker::estimate_session_size)
                .unwrap_or(0);
            sessions.remove(session_id);
            session_size
        };

        // Update memory tracker
        if session_size > 0 {
            self.memory_tracker
                .update_usage(-(session_size as i64))
                .await;
        }

        // Remove from disk spillover if exists
        let _ = self
            .spillover_manager
            .remove_spilled_session(session_id)
            .await;

        debug!(
            session_id = %session_id,
            deleted = deleted > 0,
            "Session terminated"
        );

        Ok(deleted > 0)
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> PersistenceResult<Vec<SessionState>> {
        let sessions = self.sessions.read().await;
        let active_sessions = sessions
            .values()
            .filter(|session| session.status == SessionStatus::Active)
            .cloned()
            .collect();

        Ok(active_sessions)
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(
        sessions: &Arc<RwLock<HashMap<String, SessionState>>>,
        timeout_seconds: u64,
    ) {
        let cutoff = Utc::now() - chrono::Duration::seconds(timeout_seconds as i64);
        let expired_sessions: Vec<String>;

        {
            let sessions_read = sessions.read().await;
            expired_sessions = sessions_read
                .iter()
                .filter(|(_, session)| session.last_accessed < cutoff)
                .map(|(id, _)| id.clone())
                .collect();
        }

        if !expired_sessions.is_empty() {
            let mut sessions_write = sessions.write().await;
            for session_id in &expired_sessions {
                sessions_write.remove(session_id);
            }

            info!(
                expired_count = expired_sessions.len(),
                "Cleaned up expired sessions"
            );
        }
    }

    /// Create system checkpoint
    pub async fn create_checkpoint(
        &self,
        checkpoint_type: CheckpointType,
        description: Option<String>,
    ) -> PersistenceResult<String> {
        let checkpoint_id = Uuid::new_v4().to_string();
        let state_snapshot = self.create_state_snapshot().await?;

        let checkpoint = Checkpoint {
            id: checkpoint_id.clone(),
            created_at: Utc::now(),
            checkpoint_type,
            state_snapshot,
            metadata: CheckpointMetadata {
                version: "1.0.0".to_string(),
                size_bytes: 0, // Will be calculated after serialization
                checksum: 0,   // Will be calculated after serialization
                created_by: "state_manager".to_string(),
                description,
            },
            compression: None,
        };

        // Serialize and optionally compress
        let checkpoint_data = if self.config.checkpoint_compression {
            self.compress_checkpoint(&checkpoint).await?
        } else {
            serde_json::to_vec(&checkpoint)?
        };

        // Calculate metadata
        let checksum = crc32fast::hash(&checkpoint_data);
        let mut final_checkpoint = checkpoint;
        final_checkpoint.metadata.size_bytes = checkpoint_data.len();
        final_checkpoint.metadata.checksum = checksum;

        // Store checkpoint
        self.checkpoint_manager
            .store_checkpoint(&checkpoint_id, &checkpoint_data)
            .await?;

        info!(
            checkpoint_id = %checkpoint_id,
            checkpoint_type = ?final_checkpoint.checkpoint_type,
            size_bytes = checkpoint_data.len(),
            "Checkpoint created"
        );

        Ok(checkpoint_id)
    }

    /// Restore from checkpoint
    pub async fn restore_from_checkpoint(&self, checkpoint_id: &str) -> PersistenceResult<()> {
        let checkpoint_data = self
            .checkpoint_manager
            .load_checkpoint(checkpoint_id)
            .await?;

        let checkpoint: Checkpoint = if self.config.checkpoint_compression {
            self.decompress_checkpoint(&checkpoint_data).await?
        } else {
            serde_json::from_slice(&checkpoint_data)?
        };

        // Verify checksum
        let calculated_checksum = crc32fast::hash(&checkpoint_data);
        if calculated_checksum != checkpoint.metadata.checksum {
            return Err(PersistenceError::data_integrity(
                "Checkpoint checksum mismatch",
            ));
        }

        // Restore state
        self.restore_state_snapshot(&checkpoint.state_snapshot)
            .await?;

        info!(
            checkpoint_id = %checkpoint_id,
            created_at = %checkpoint.created_at,
            "State restored from checkpoint"
        );

        Ok(())
    }

    /// Create comprehensive state snapshot
    async fn create_state_snapshot(&self) -> PersistenceResult<StateSnapshot> {
        let sessions = {
            let sessions_read = self.sessions.read().await;
            sessions_read.clone()
        };

        let configuration = self.config_manager.get_current_config().await?;

        // In a real implementation, you would gather actual cache stats, tenant data, etc.
        let cache_stats = HashMap::new();
        let tenant_data = HashMap::new();
        let system_metrics = HashMap::new();

        Ok(StateSnapshot {
            sessions,
            configuration,
            cache_stats,
            tenant_data,
            system_metrics,
        })
    }

    /// Restore state from snapshot
    async fn restore_state_snapshot(&self, snapshot: &StateSnapshot) -> PersistenceResult<()> {
        // Restore sessions
        {
            let mut sessions = self.sessions.write().await;
            sessions.clear();
            sessions.extend(snapshot.sessions.clone());
        }

        // Restore configuration
        self.config_manager
            .restore_configuration(&snapshot.configuration)
            .await?;

        // In a real implementation, restore other state components

        info!(
            sessions_count = snapshot.sessions.len(),
            config_keys = snapshot.configuration.len(),
            "State snapshot restored"
        );

        Ok(())
    }

    /// Compress checkpoint data
    async fn compress_checkpoint(&self, checkpoint: &Checkpoint) -> PersistenceResult<Vec<u8>> {
        let data = serde_json::to_vec(checkpoint)?;
        let compressed = lz4_flex::compress_prepend_size(&data);

        debug!(
            original_size = data.len(),
            compressed_size = compressed.len(),
            ratio = compressed.len() as f32 / data.len() as f32,
            "Checkpoint compressed"
        );

        Ok(compressed)
    }

    /// Decompress checkpoint data
    async fn decompress_checkpoint(&self, data: &[u8]) -> PersistenceResult<Checkpoint> {
        let decompressed = lz4_flex::decompress_size_prepended(data).map_err(|e| {
            PersistenceError::compression(format!("LZ4 decompression failed: {}", e))
        })?;

        let checkpoint: Checkpoint = serde_json::from_slice(&decompressed)?;
        Ok(checkpoint)
    }

    /// Initiate graceful shutdown
    pub async fn shutdown_gracefully(&self) -> PersistenceResult<()> {
        info!("Initiating graceful shutdown");

        // Create shutdown checkpoint
        if self.config.enable_graceful_shutdown {
            self.create_checkpoint(
                CheckpointType::Shutdown,
                Some("Graceful shutdown".to_string()),
            )
            .await?;
        }

        // Notify all background tasks to shutdown
        let _ = self.shutdown_tx.send(());

        // Close sessions gracefully
        let active_sessions = self.get_active_sessions().await?;
        for session in active_sessions {
            if let Err(e) = self.terminate_session(&session.id).await {
                warn!(session_id = %session.id, error = %e, "Failed to terminate session during shutdown");
            }
        }

        info!("Graceful shutdown completed");
        Ok(())
    }
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            config: self.config.clone(),
            sessions: Arc::clone(&self.sessions),
            config_manager: Arc::clone(&self.config_manager),
            checkpoint_manager: Arc::clone(&self.checkpoint_manager),
            hot_reload_watcher: self.hot_reload_watcher.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
            spillover_manager: Arc::clone(&self.spillover_manager),
            memory_tracker: Arc::clone(&self.memory_tracker),
        }
    }
}

/// Configuration manager with hot reload support
pub struct ConfigurationManager {
    _config: StateConfig,
    current_config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    config_version: Arc<RwLock<u64>>,
}

impl ConfigurationManager {
    async fn new(config: StateConfig) -> PersistenceResult<Self> {
        Ok(Self {
            _config: config,
            current_config: Arc::new(RwLock::new(HashMap::new())),
            config_version: Arc::new(RwLock::new(1)),
        })
    }

    async fn get_current_config(&self) -> PersistenceResult<HashMap<String, serde_json::Value>> {
        let config = self.current_config.read().await;
        Ok(config.clone())
    }

    async fn restore_configuration(
        &self,
        configuration: &HashMap<String, serde_json::Value>,
    ) -> PersistenceResult<()> {
        let mut config = self.current_config.write().await;
        config.clear();
        config.extend(configuration.clone());
        Ok(())
    }

    async fn reload_configuration(&self, file_path: &Path) -> PersistenceResult<()> {
        let content = fs::read_to_string(file_path).await?;
        let new_config: HashMap<String, serde_json::Value> = serde_yaml::from_str(&content)
            .map_err(|e| PersistenceError::configuration(format!("YAML parse error: {}", e)))?;

        {
            let mut config = self.current_config.write().await;
            config.clear();
            config.extend(new_config);
        }

        {
            let mut version = self.config_version.write().await;
            *version += 1;
        }

        info!(file = %file_path.display(), "Configuration reloaded");
        Ok(())
    }
}

/// Checkpoint manager for persistent storage
pub struct CheckpointManager {
    config: StateConfig,
    checkpoints_dir: PathBuf,
}

impl CheckpointManager {
    async fn new(config: StateConfig) -> PersistenceResult<Self> {
        let checkpoints_dir = PathBuf::from("./data/checkpoints");
        fs::create_dir_all(&checkpoints_dir).await?;

        Ok(Self {
            config,
            checkpoints_dir,
        })
    }

    async fn store_checkpoint(&self, checkpoint_id: &str, data: &[u8]) -> PersistenceResult<()> {
        let file_path = self.checkpoints_dir.join(format!("{}.ckpt", checkpoint_id));
        fs::write(&file_path, data).await?;

        // Clean up old checkpoints
        self.cleanup_old_checkpoints().await?;

        debug!(
            checkpoint_id = %checkpoint_id,
            file_path = %file_path.display(),
            size_bytes = data.len(),
            "Checkpoint stored"
        );

        Ok(())
    }

    async fn load_checkpoint(&self, checkpoint_id: &str) -> PersistenceResult<Vec<u8>> {
        let file_path = self.checkpoints_dir.join(format!("{}.ckpt", checkpoint_id));
        let data = fs::read(&file_path).await?;

        debug!(
            checkpoint_id = %checkpoint_id,
            file_path = %file_path.display(),
            size_bytes = data.len(),
            "Checkpoint loaded"
        );

        Ok(data)
    }

    async fn cleanup_old_checkpoints(&self) -> PersistenceResult<()> {
        let mut entries = fs::read_dir(&self.checkpoints_dir).await?;
        let mut checkpoints = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".ckpt") {
                    let metadata = entry.metadata().await?;
                    if let Ok(modified) = metadata.modified() {
                        checkpoints.push((entry.path(), modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        checkpoints.sort_by(|a, b| b.1.cmp(&a.1));

        // Keep only the configured number of checkpoints
        let max_checkpoints = self.config.max_checkpoints as usize;
        if checkpoints.len() > max_checkpoints {
            for (path, _) in checkpoints.into_iter().skip(max_checkpoints) {
                if let Err(e) = fs::remove_file(&path).await {
                    warn!(file = %path.display(), error = %e, "Failed to remove old checkpoint");
                }
            }
        }

        Ok(())
    }
}

/// Hot reload watcher for configuration files
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    _config_manager: Arc<ConfigurationManager>,
}

impl HotReloadWatcher {
    async fn new(
        watch_paths: Vec<String>,
        config_manager: Arc<ConfigurationManager>,
    ) -> PersistenceResult<Self> {
        let config_manager_clone = Arc::clone(&config_manager);

        let mut watcher = notify::recommended_watcher(
            move |result: Result<Event, notify::Error>| match result {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in &event.paths {
                            if path.extension().and_then(|s| s.to_str()) == Some("yml")
                                || path.extension().and_then(|s| s.to_str()) == Some("yaml")
                            {
                                let config_manager = Arc::clone(&config_manager_clone);
                                let path = path.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = config_manager.reload_configuration(&path).await
                                    {
                                        error!(file = %path.display(), error = %e, "Failed to reload configuration");
                                    }
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "File watch error");
                }
            },
        )?;

        // Watch all configured paths
        for path_str in &watch_paths {
            let path = Path::new(path_str);
            if path.exists() {
                watcher.watch(path, RecursiveMode::Recursive)?;
                info!(path = %path.display(), "Watching configuration directory");
            } else {
                warn!(path = %path.display(), "Configuration path does not exist");
            }
        }

        Ok(Self {
            _watcher: watcher,
            _config_manager: config_manager,
        })
    }
}

/// Session spillover manager for disk-based overflow handling
pub struct SessionSpilloverManager {
    /// Spillover directory for session persistence
    spillover_dir: PathBuf,
    /// LRU access tracker for eviction
    access_tracker: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Spillover metrics
    metrics: Arc<RwLock<SpilloverMetrics>>,
}

/// Spillover metrics tracking
#[derive(Debug, Clone, Default)]
pub struct SpilloverMetrics {
    pub total_spilled: u64,
    pub total_restored: u64,
    pub spill_operations: u64,
    pub restore_operations: u64,
    pub avg_spill_time_ms: f64,
    pub avg_restore_time_ms: f64,
}

impl SessionSpilloverManager {
    /// Create new spillover manager
    async fn new(spillover_dir: PathBuf) -> PersistenceResult<Self> {
        // Create spillover directory if it doesn't exist
        fs::create_dir_all(&spillover_dir).await?;

        Ok(Self {
            spillover_dir,
            access_tracker: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(SpilloverMetrics::default())),
        })
    }

    /// Spill session to disk with atomic writes
    async fn spill_session(
        &self,
        session_id: &str,
        session: &SessionState,
    ) -> PersistenceResult<()> {
        let start = std::time::Instant::now();

        // Serialize session data
        let session_data = serde_json::to_vec(session)?;

        // Write to temporary file first (atomic write pattern)
        let temp_file_path = self.spillover_dir.join(format!("{}.tmp", session_id));
        let final_file_path = self.spillover_dir.join(format!("{}.session", session_id));

        // Write to temp file
        fs::write(&temp_file_path, &session_data).await?;

        // Atomically rename to final location
        fs::rename(&temp_file_path, &final_file_path).await?;

        // Update metrics
        let elapsed = start.elapsed().as_millis() as f64;
        let mut metrics = self.metrics.write().await;
        metrics.total_spilled += 1;
        metrics.spill_operations += 1;

        // Update running average
        let count = metrics.spill_operations as f64;
        metrics.avg_spill_time_ms = (metrics.avg_spill_time_ms * (count - 1.0) + elapsed) / count;

        debug!(
            session_id = %session_id,
            elapsed_ms = elapsed,
            size_bytes = session_data.len(),
            "Session spilled to disk"
        );

        Ok(())
    }

    /// Restore session from disk
    async fn restore_session(&self, session_id: &str) -> PersistenceResult<Option<SessionState>> {
        let start = std::time::Instant::now();
        let file_path = self.spillover_dir.join(format!("{}.session", session_id));

        // Check if file exists
        if !tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
            return Ok(None);
        }

        // Read session data
        let session_data = fs::read(&file_path).await?;
        let session: SessionState = serde_json::from_slice(&session_data)?;

        // Update metrics
        let elapsed = start.elapsed().as_millis() as f64;
        let mut metrics = self.metrics.write().await;
        metrics.total_restored += 1;
        metrics.restore_operations += 1;

        // Update running average
        let count = metrics.restore_operations as f64;
        metrics.avg_restore_time_ms =
            (metrics.avg_restore_time_ms * (count - 1.0) + elapsed) / count;

        debug!(
            session_id = %session_id,
            elapsed_ms = elapsed,
            size_bytes = session_data.len(),
            "Session restored from disk"
        );

        Ok(Some(session))
    }

    /// Remove spilled session from disk
    async fn remove_spilled_session(&self, session_id: &str) -> PersistenceResult<()> {
        let file_path = self.spillover_dir.join(format!("{}.session", session_id));

        if tokio::fs::try_exists(&file_path).await.unwrap_or(false) {
            fs::remove_file(&file_path).await?;
            debug!(session_id = %session_id, "Spilled session removed from disk");
        }

        Ok(())
    }

    /// Update access time for LRU tracking
    async fn update_access(&self, session_id: &str) {
        let mut tracker = self.access_tracker.write().await;
        tracker.insert(session_id.to_string(), Utc::now());
    }

    /// Get least recently used sessions for eviction
    async fn get_lru_sessions(&self, count: usize) -> Vec<String> {
        let tracker = self.access_tracker.read().await;
        let mut sessions: Vec<_> = tracker.iter().collect();

        // Sort by access time (oldest first)
        sessions.sort_by(|a, b| a.1.cmp(b.1));

        sessions
            .into_iter()
            .take(count)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get spillover metrics
    #[allow(dead_code)]
    async fn get_metrics(&self) -> SpilloverMetrics {
        self.metrics.read().await.clone()
    }
}

/// Memory usage tracker for automatic spillover
pub struct MemoryTracker {
    /// Current estimated memory usage in bytes
    current_usage: Arc<RwLock<u64>>,
    /// Maximum memory threshold in bytes
    max_memory: u64,
    /// Warning threshold (percentage of max)
    warning_threshold: f64,
}

impl MemoryTracker {
    /// Create new memory tracker
    fn new(max_memory: u64, warning_threshold: f64) -> Self {
        Self {
            current_usage: Arc::new(RwLock::new(0)),
            max_memory,
            warning_threshold,
        }
    }

    /// Update memory usage estimate
    async fn update_usage(&self, delta: i64) {
        let mut usage = self.current_usage.write().await;
        if delta < 0 {
            *usage = usage.saturating_sub(delta.unsigned_abs());
        } else {
            *usage += delta as u64;
        }
    }

    /// Get current memory usage
    async fn get_usage(&self) -> u64 {
        *self.current_usage.read().await
    }

    /// Check if memory usage exceeds threshold
    async fn should_spill(&self) -> bool {
        let usage = self.get_usage().await;
        usage as f64 >= self.max_memory as f64 * self.warning_threshold
    }

    /// Get memory usage percentage
    async fn get_usage_percentage(&self) -> f64 {
        let usage = self.get_usage().await;
        (usage as f64 / self.max_memory as f64) * 100.0
    }

    /// Calculate estimated session memory size
    fn estimate_session_size(session: &SessionState) -> u64 {
        // Rough estimate: serialize and measure
        serde_json::to_vec(session)
            .map(|v| v.len() as u64)
            .unwrap_or(1024) // Default 1KB estimate
    }
}
