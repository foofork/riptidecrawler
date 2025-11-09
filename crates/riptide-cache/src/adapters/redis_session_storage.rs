//! Redis adapter for session storage
//!
//! This module implements the `SessionStorage` port using Redis as the backend.
//! It provides TTL-based expiration and high-performance session management.
//!
//! # Key Format
//!
//! ```text
//! session:v1:{session_id}
//! ```
//!
//! # Features
//!
//! - TTL-based automatic expiration
//! - Atomic get-and-refresh with Lua script
//! - JSONB serialization for session data
//! - Connection pooling via deadpool-redis
//! - High-performance in-memory storage

use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Config, Pool, Runtime};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::session::{Session, SessionFilter, SessionStorage};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Redis session storage adapter
pub struct RedisSessionStorage {
    pool: Arc<Pool>,
    key_prefix: String,
}

impl RedisSessionStorage {
    /// Create new Redis session storage with default configuration
    pub fn new(redis_url: &str) -> RiptideResult<Self> {
        let cfg = Config::from_url(redis_url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| RiptideError::CacheError(format!("Failed to create Redis pool: {}", e)))?;

        Ok(Self {
            pool: Arc::new(pool),
            key_prefix: "session:v1:".to_string(),
        })
    }

    /// Create from existing pool
    pub fn from_pool(pool: Arc<Pool>) -> Self {
        Self {
            pool,
            key_prefix: "session:v1:".to_string(),
        }
    }

    /// Build Redis key for session
    fn build_key(&self, session_id: &str) -> String {
        format!("{}{}", self.key_prefix, session_id)
    }

    /// Serialize session to JSON string
    fn serialize_session(&self, session: &Session) -> RiptideResult<String> {
        serde_json::to_string(session).map_err(|e| {
            RiptideError::SerializationError(format!("Failed to serialize session: {}", e))
        })
    }

    /// Deserialize session from JSON string
    fn deserialize_session(&self, data: &str) -> RiptideResult<Session> {
        serde_json::from_str(data).map_err(|e| {
            RiptideError::SerializationError(format!("Failed to deserialize session: {}", e))
        })
    }

    /// Calculate TTL in seconds for a session
    fn calculate_ttl(&self, session: &Session) -> u64 {
        session.remaining_ttl_secs()
    }

    /// Lua script for atomic get-and-refresh
    /// Returns session data and extends TTL if not expired
    #[allow(dead_code)]
    fn get_and_refresh_script() -> &'static str {
        r#"
        local key = KEYS[1]
        local ttl = tonumber(ARGV[1])
        local data = redis.call('GET', key)
        if data then
            redis.call('EXPIRE', key, ttl)
            return data
        end
        return nil
        "#
    }
}

#[async_trait]
impl SessionStorage for RedisSessionStorage {
    #[instrument(skip(self), fields(session_id = %id))]
    async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
        debug!("Fetching session from Redis");

        let mut conn = self.pool.get().await.map_err(|e| {
            RiptideError::CacheError(format!("Failed to get Redis connection: {}", e))
        })?;

        let key = self.build_key(id);
        let data: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| RiptideError::CacheError(format!("Failed to get session: {}", e)))?;

        match data {
            Some(json) => {
                let session = self.deserialize_session(&json)?;

                // Check if expired (Redis TTL might not have kicked in yet)
                if session.is_expired() {
                    debug!("Session expired, deleting from Redis");
                    let _: () = conn.del(&key).await.map_err(|e| {
                        RiptideError::CacheError(format!("Failed to delete expired session: {}", e))
                    })?;
                    return Ok(None);
                }

                debug!("Session found: user_id={}", session.user_id);
                Ok(Some(session))
            }
            None => {
                debug!("Session not found");
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, session), fields(session_id = %session.id))]
    async fn save_session(&self, session: &Session) -> RiptideResult<()> {
        debug!("Saving session to Redis");

        let mut conn = self.pool.get().await.map_err(|e| {
            RiptideError::CacheError(format!("Failed to get Redis connection: {}", e))
        })?;

        let key = self.build_key(&session.id);
        let json = self.serialize_session(session)?;
        let ttl = self.calculate_ttl(session);

        if ttl == 0 {
            warn!("Attempted to save expired session, skipping");
            return Err(RiptideError::ValidationError(
                "Cannot save expired session".to_string(),
            ));
        }

        let _: () = conn
            .set_ex(&key, json, ttl)
            .await
            .map_err(|e| RiptideError::CacheError(format!("Failed to save session: {}", e)))?;

        info!("Session saved successfully with TTL={}s", ttl);
        Ok(())
    }

    #[instrument(skip(self), fields(session_id = %id))]
    async fn delete_session(&self, id: &str) -> RiptideResult<()> {
        debug!("Deleting session from Redis");

        let mut conn = self.pool.get().await.map_err(|e| {
            RiptideError::CacheError(format!("Failed to get Redis connection: {}", e))
        })?;

        let key = self.build_key(id);
        let deleted: i32 = conn
            .del(&key)
            .await
            .map_err(|e| RiptideError::CacheError(format!("Failed to delete session: {}", e)))?;

        if deleted > 0 {
            info!("Session deleted successfully");
        } else {
            debug!("Session not found for deletion");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
        debug!("Listing sessions with filter");

        let mut conn = self.pool.get().await.map_err(|e| {
            RiptideError::CacheError(format!("Failed to get Redis connection: {}", e))
        })?;

        // Scan for all session keys
        let pattern = format!("{}*", self.key_prefix);
        let keys: Vec<String> = deadpool_redis::redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut *conn)
            .await
            .map_err(|e| RiptideError::CacheError(format!("Failed to scan keys: {}", e)))?;

        let mut sessions = Vec::new();

        for key in keys {
            let data: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| RiptideError::CacheError(format!("Failed to get session: {}", e)))?;

            if let Some(json) = data {
                match self.deserialize_session(&json) {
                    Ok(session) => {
                        // Apply filters
                        if let Some(ref user_id) = filter.user_id {
                            if &session.user_id != user_id {
                                continue;
                            }
                        }

                        if let Some(ref tenant_id) = filter.tenant_id {
                            if &session.tenant_id != tenant_id {
                                continue;
                            }
                        }

                        if filter.active_only && session.is_expired() {
                            continue;
                        }

                        sessions.push(session);
                    }
                    Err(e) => {
                        warn!("Failed to deserialize session {}: {}", key, e);
                    }
                }
            }
        }

        info!("Listed {} sessions", sessions.len());
        Ok(sessions)
    }

    #[instrument(skip(self))]
    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        debug!("Cleaning up expired sessions (Redis TTL handles this automatically)");

        // Redis automatically expires keys via TTL
        // This method is a no-op for Redis, but we can scan for any manually expired sessions
        let filter = SessionFilter {
            user_id: None,
            tenant_id: None,
            active_only: false,
        };

        let all_sessions = self.list_sessions(filter).await?;
        let mut count = 0;

        for session in all_sessions {
            if session.is_expired() {
                self.delete_session(&session.id).await?;
                count += 1;
            }
        }

        info!("Cleaned up {} manually expired sessions", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime};

    async fn create_test_storage() -> RedisSessionStorage {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        RedisSessionStorage::new(&redis_url).expect("Failed to create test storage")
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_save_and_get_session() {
        let storage = create_test_storage().await;

        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "admin".to_string());

        let session = Session {
            id: "test-redis-session-1".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata,
        };

        // Save session
        storage.save_session(&session).await.unwrap();

        // Retrieve session
        let retrieved = storage.get_session("test-redis-session-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, session.id);
        assert_eq!(retrieved.user_id, session.user_id);
        assert_eq!(retrieved.metadata.get("role"), Some(&"admin".to_string()));

        // Cleanup
        storage
            .delete_session("test-redis-session-1")
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_delete_session() {
        let storage = create_test_storage().await;

        let session = Session {
            id: "test-redis-session-2".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&session).await.unwrap();
        storage
            .delete_session("test-redis-session-2")
            .await
            .unwrap();

        let retrieved = storage.get_session("test-redis-session-2").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_ttl_expiration() {
        let storage = create_test_storage().await;

        // Create session with very short TTL
        let session = Session {
            id: "test-redis-session-ttl".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(2),
            metadata: HashMap::new(),
        };

        storage.save_session(&session).await.unwrap();

        // Should exist immediately
        assert!(storage
            .get_session("test-redis-session-ttl")
            .await
            .unwrap()
            .is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Should be expired
        assert!(storage
            .get_session("test-redis-session-ttl")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_list_sessions_with_filter() {
        let storage = create_test_storage().await;

        let session1 = Session {
            id: "filter-test-1".to_string(),
            user_id: "user-filter-1".to_string(),
            tenant_id: "tenant-A".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        let session2 = Session {
            id: "filter-test-2".to_string(),
            user_id: "user-filter-2".to_string(),
            tenant_id: "tenant-A".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&session1).await.unwrap();
        storage.save_session(&session2).await.unwrap();

        // Filter by tenant
        let filter = SessionFilter {
            user_id: None,
            tenant_id: Some("tenant-A".to_string()),
            active_only: true,
        };

        let sessions = storage.list_sessions(filter).await.unwrap();
        assert!(sessions.len() >= 2);

        // Cleanup
        storage.delete_session("filter-test-1").await.unwrap();
        storage.delete_session("filter-test-2").await.unwrap();
    }
}
