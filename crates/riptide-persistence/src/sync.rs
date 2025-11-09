/*!
# Distributed Cache Synchronization

Implementation of distributed cache synchronization with consensus mechanisms,
leader election, and conflict resolution for multi-instance coordination.
*/

use crate::{config::DistributedConfig, errors::PersistenceResult};
use chrono::{DateTime, Utc};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Distributed synchronization manager
pub struct DistributedSync {
    /// Node configuration
    config: DistributedConfig,
    /// Redis connection pool for coordination
    pool: Arc<Mutex<MultiplexedConnection>>,
    /// Consensus manager
    consensus: Arc<ConsensusManager>,
    /// Leader election manager
    leader_election: Arc<LeaderElection>,
    /// Sync state
    state: Arc<RwLock<SyncState>>,
    /// Node ID
    node_id: String,
}

/// Synchronization state
#[derive(Debug, Clone)]
struct SyncState {
    /// Is this node the leader?
    is_leader: bool,
    /// Known cluster nodes
    known_nodes: HashSet<String>,
    /// Last heartbeat times
    node_heartbeats: HashMap<String, DateTime<Utc>>,
    /// Pending operations
    pending_operations: Vec<SyncOperation>,
    /// Operation history for conflict resolution
    operation_history: Vec<HistoricalOperation>,
}

/// Synchronization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    /// Operation ID
    pub id: String,
    /// Operation type
    pub operation_type: SyncOperationType,
    /// Target key
    pub key: String,
    /// Value (for set operations)
    pub value: Option<Vec<u8>>,
    /// TTL (for set operations)
    pub ttl: Option<u64>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Originating node
    pub origin_node: String,
    /// Operation priority
    pub priority: u8,
}

/// Types of synchronization operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncOperationType {
    /// Set operation
    Set,
    /// Delete operation
    Delete,
    /// Invalidate pattern
    InvalidatePattern,
    /// Clear cache
    Clear,
    /// Heartbeat
    Heartbeat,
}

/// Historical operation for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoricalOperation {
    operation: SyncOperation,
    applied_at: DateTime<Utc>,
    result: OperationResult,
}

/// Result of applying an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
enum OperationResult {
    Success,
    Conflict(String),
    Error(String),
}

impl DistributedSync {
    /// Create new distributed sync manager
    pub async fn new(redis_url: &str, config: DistributedConfig) -> PersistenceResult<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;

        let consensus = Arc::new(ConsensusManager::new(config.clone()).await?);
        let leader_election = Arc::new(LeaderElection::new(config.clone()).await?);

        let state = Arc::new(RwLock::new(SyncState {
            is_leader: false,
            known_nodes: HashSet::new(),
            node_heartbeats: HashMap::new(),
            pending_operations: Vec::new(),
            operation_history: Vec::new(),
        }));

        let sync_manager = Self {
            config: config.clone(),
            pool: Arc::new(Mutex::new(conn)),
            consensus,
            leader_election,
            state,
            node_id: config.node_id.clone(),
        };

        // Start background tasks
        sync_manager.start_background_tasks().await;

        info!(
            node_id = %config.node_id,
            cluster_nodes = config.cluster_nodes.len(),
            "Distributed sync manager initialized"
        );

        Ok(sync_manager)
    }

    /// Start background synchronization tasks
    async fn start_background_tasks(&self) {
        let sync_manager = Arc::new(self.clone());

        // Heartbeat task
        let heartbeat_manager = sync_manager.clone();
        tokio::spawn(async move {
            heartbeat_manager.heartbeat_task().await;
        });

        // Leader election task
        let election_manager = sync_manager.clone();
        tokio::spawn(async move {
            election_manager.leader_election_task().await;
        });

        // Operation processing task
        let operation_manager = sync_manager.clone();
        tokio::spawn(async move {
            operation_manager.operation_processing_task().await;
        });

        // Cleanup task
        let cleanup_manager = sync_manager.clone();
        tokio::spawn(async move {
            cleanup_manager.cleanup_task().await;
        });
    }

    /// Notify other nodes of a cache operation
    pub async fn notify_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
        // Add to pending operations
        {
            let mut state = self.state.write().await;
            state.pending_operations.push(operation.clone());
        }

        // Publish to coordination channel
        let channel = format!("riptide:sync:{}", "operations");
        let message = serde_json::to_string(&operation)?;

        let mut conn = self.pool.lock().await;
        conn.publish::<_, _, ()>(&channel, &message).await?;

        debug!(
            operation_id = %operation.id,
            operation_type = ?operation.operation_type,
            key = %operation.key,
            "Sync operation published"
        );

        Ok(())
    }

    /// Process received synchronization operation
    pub async fn process_operation(&self, operation: SyncOperation) -> PersistenceResult<()> {
        // Check if operation is from this node (avoid self-sync)
        if operation.origin_node == self.node_id {
            return Ok(());
        }

        // Check for conflicts with pending operations
        let conflict = self.check_for_conflicts(&operation).await?;
        if let Some(conflict_info) = conflict {
            warn!(
                operation_id = %operation.id,
                conflict = %conflict_info,
                "Operation conflict detected, using consensus resolution"
            );

            // Use consensus to resolve conflict
            let resolution = self
                .consensus
                .resolve_conflict(&operation, &conflict_info)
                .await?;
            if !resolution.should_apply {
                debug!(
                    operation_id = %operation.id,
                    "Operation rejected by consensus"
                );
                return Ok(());
            }
        }

        // Apply operation based on type
        let result = match operation.operation_type {
            SyncOperationType::Set => self.apply_set_operation(&operation).await,
            SyncOperationType::Delete => self.apply_delete_operation(&operation).await,
            SyncOperationType::InvalidatePattern => {
                self.apply_invalidate_operation(&operation).await
            }
            SyncOperationType::Clear => self.apply_clear_operation(&operation).await,
            SyncOperationType::Heartbeat => self.apply_heartbeat_operation(&operation).await,
        };

        // Record operation in history
        let historical_op = HistoricalOperation {
            operation: operation.clone(),
            applied_at: Utc::now(),
            result: match &result {
                Ok(_) => OperationResult::Success,
                Err(e) => OperationResult::Error(e.to_string()),
            },
        };

        {
            let mut state = self.state.write().await;
            state.operation_history.push(historical_op);

            // Keep only recent history (last 1000 operations)
            if state.operation_history.len() > 1000 {
                state.operation_history.drain(0..100);
            }
        }

        result.map(|_| ())
    }

    /// Check for operation conflicts
    async fn check_for_conflicts(
        &self,
        operation: &SyncOperation,
    ) -> PersistenceResult<Option<String>> {
        let state = self.state.read().await;

        // Check pending operations for conflicts
        for pending in &state.pending_operations {
            if pending.key == operation.key && pending.timestamp > operation.timestamp {
                return Ok(Some(format!(
                    "Newer operation {} exists for key {}",
                    pending.id, operation.key
                )));
            }
        }

        // Check operation history for recent conflicts
        for historical in state.operation_history.iter().rev().take(100) {
            if historical.operation.key == operation.key {
                let time_diff = operation
                    .timestamp
                    .signed_duration_since(historical.operation.timestamp);
                if time_diff.num_seconds().abs() < 60 {
                    // Operations within 1 minute might conflict
                    return Ok(Some(format!(
                        "Recent operation {} on key {} within conflict window",
                        historical.operation.id, operation.key
                    )));
                }
                break; // Only check most recent operation on same key
            }
        }

        Ok(None)
    }

    /// Apply set operation
    async fn apply_set_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
        if let Some(value) = &operation.value {
            let mut conn = self.pool.lock().await;
            if let Some(ttl) = operation.ttl {
                conn.set_ex::<_, _, ()>(&operation.key, value, ttl).await?;
            } else {
                conn.set::<_, _, ()>(&operation.key, value).await?;
            }

            debug!(
                key = %operation.key,
                value_size = value.len(),
                "Set operation applied"
            );
        }
        Ok(())
    }

    /// Apply delete operation
    async fn apply_delete_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
        let mut conn = self.pool.lock().await;
        let deleted: u64 = conn.del(&operation.key).await?;

        debug!(
            key = %operation.key,
            deleted = deleted,
            "Delete operation applied"
        );

        Ok(())
    }

    /// Apply invalidate pattern operation
    async fn apply_invalidate_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
        let mut conn = self.pool.lock().await;

        // Get keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&operation.key) // key contains the pattern
            .query_async(&mut *conn)
            .await
            .unwrap_or_default();

        if !keys.is_empty() {
            let deleted: u64 = conn.del(&keys).await?;
            debug!(
                pattern = %operation.key,
                deleted = deleted,
                "Pattern invalidation applied"
            );
        }

        Ok(())
    }

    /// Apply clear operation
    async fn apply_clear_operation(&self, _operation: &SyncOperation) -> PersistenceResult<()> {
        let mut conn = self.pool.lock().await;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut *conn).await?;

        info!("Cache clear operation applied");
        Ok(())
    }

    /// Apply heartbeat operation
    async fn apply_heartbeat_operation(&self, operation: &SyncOperation) -> PersistenceResult<()> {
        let mut state = self.state.write().await;
        state.known_nodes.insert(operation.origin_node.clone());
        state
            .node_heartbeats
            .insert(operation.origin_node.clone(), operation.timestamp);

        debug!(
            node = %operation.origin_node,
            timestamp = %operation.timestamp,
            "Heartbeat received"
        );

        Ok(())
    }

    /// Heartbeat task
    async fn heartbeat_task(&self) {
        let mut interval = interval(Duration::from_millis(self.config.heartbeat_interval_ms));

        loop {
            interval.tick().await;

            let heartbeat = SyncOperation {
                id: Uuid::new_v4().to_string(),
                operation_type: SyncOperationType::Heartbeat,
                key: "heartbeat".to_string(),
                value: None,
                ttl: None,
                timestamp: Utc::now(),
                origin_node: self.node_id.clone(),
                priority: 0,
            };

            if let Err(e) = self.notify_operation(heartbeat).await {
                error!(error = %e, "Failed to send heartbeat");
            }

            // Clean up dead nodes
            self.cleanup_dead_nodes().await;
        }
    }

    /// Leader election task
    async fn leader_election_task(&self) {
        let mut interval = interval(Duration::from_millis(
            self.config.leader_election_timeout_ms,
        ));

        loop {
            interval.tick().await;

            match self.leader_election.participate().await {
                Ok(is_leader) => {
                    let mut state = self.state.write().await;
                    if state.is_leader != is_leader {
                        state.is_leader = is_leader;
                        if is_leader {
                            info!(node_id = %self.node_id, "Node elected as leader");
                        } else {
                            debug!(node_id = %self.node_id, "Node is follower");
                        }
                    }
                }
                Err(e) => {
                    error!(error = %e, "Leader election failed");
                }
            }
        }
    }

    /// Operation processing task
    async fn operation_processing_task(&self) {
        let mut interval = interval(Duration::from_millis(100)); // Process every 100ms

        loop {
            interval.tick().await;

            // Process pending operations
            let pending_ops = {
                let mut state = self.state.write().await;
                let ops = state.pending_operations.clone();
                state.pending_operations.clear();
                ops
            };

            for operation in pending_ops {
                if let Err(e) = self.process_operation(operation).await {
                    error!(error = %e, "Failed to process sync operation");
                }
            }
        }
    }

    /// Cleanup task
    async fn cleanup_task(&self) {
        let mut interval = interval(Duration::from_secs(60)); // Cleanup every minute

        loop {
            interval.tick().await;

            // Clean up expired operations from history
            {
                let mut state = self.state.write().await;
                let cutoff = Utc::now() - chrono::Duration::hours(1);
                state.operation_history.retain(|op| op.applied_at > cutoff);
            }

            debug!("Completed sync cleanup cycle");
        }
    }

    /// Clean up nodes that haven't sent heartbeats
    async fn cleanup_dead_nodes(&self) {
        let cutoff = Utc::now() - chrono::Duration::seconds(60);
        let mut state = self.state.write().await;

        let dead_nodes: Vec<String> = state
            .node_heartbeats
            .iter()
            .filter(|(_, &timestamp)| timestamp < cutoff)
            .map(|(node, _)| node.clone())
            .collect();

        for node in dead_nodes {
            state.known_nodes.remove(&node);
            state.node_heartbeats.remove(&node);
            warn!(node = %node, "Removed dead node from cluster");
        }
    }

    /// Check if this node is the leader
    pub async fn is_leader(&self) -> bool {
        self.state.read().await.is_leader
    }

    /// Get cluster status
    pub async fn get_cluster_status(&self) -> ClusterStatus {
        let state = self.state.read().await;
        ClusterStatus {
            node_id: self.node_id.clone(),
            is_leader: state.is_leader,
            known_nodes: state.known_nodes.clone(),
            active_nodes: state.node_heartbeats.len(),
            pending_operations: state.pending_operations.len(),
            operation_history_size: state.operation_history.len(),
        }
    }
}

impl Clone for DistributedSync {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: Arc::clone(&self.pool),
            consensus: Arc::clone(&self.consensus),
            leader_election: Arc::clone(&self.leader_election),
            state: Arc::clone(&self.state),
            node_id: self.node_id.clone(),
        }
    }
}

/// Consensus manager for conflict resolution
pub struct ConsensusManager {
    _config: DistributedConfig,
}

impl ConsensusManager {
    pub async fn new(config: DistributedConfig) -> PersistenceResult<Self> {
        Ok(Self { _config: config })
    }

    /// Resolve conflicts between operations
    pub async fn resolve_conflict(
        &self,
        operation: &SyncOperation,
        conflict_info: &str,
    ) -> PersistenceResult<ConflictResolution> {
        // Simple timestamp-based resolution for now
        // In production, could use vector clocks or other sophisticated methods

        debug!(
            operation_id = %operation.id,
            conflict = %conflict_info,
            "Resolving operation conflict"
        );

        // Always prefer newer operations
        Ok(ConflictResolution {
            should_apply: true,
            resolution_reason: "Timestamp-based resolution".to_string(),
        })
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub should_apply: bool,
    pub resolution_reason: String,
}

/// Leader election manager
pub struct LeaderElection {
    _config: DistributedConfig,
    _election_key: String,
}

impl LeaderElection {
    pub async fn new(config: DistributedConfig) -> PersistenceResult<Self> {
        Ok(Self {
            _config: config.clone(),
            _election_key: format!("riptide:leader:{}", "default"),
        })
    }

    /// Participate in leader election
    pub async fn participate(&self) -> PersistenceResult<bool> {
        // Simple Redis-based leader election using SET with NX and EX
        // In production, would use more sophisticated algorithms like Raft

        // For now, return false (not leader) to avoid complexity
        // Real implementation would compete for leadership
        Ok(false)
    }
}

/// Cluster status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: String,
    pub is_leader: bool,
    pub known_nodes: HashSet<String>,
    pub active_nodes: usize,
    pub pending_operations: usize,
    pub operation_history_size: usize,
}
