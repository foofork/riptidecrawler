//! PostgreSQL adapter for session storage
//!
//! This module implements the `SessionStorage` port using PostgreSQL as the backend.
//! It provides the anti-corruption layer between domain sessions and database schema.
//!
//! # Schema Requirements
//!
//! ```sql
//! CREATE TABLE sessions (
//!     id VARCHAR(255) PRIMARY KEY,
//!     user_id VARCHAR(255) NOT NULL,
//!     tenant_id VARCHAR(255) NOT NULL,
//!     created_at TIMESTAMPTZ NOT NULL,
//!     expires_at TIMESTAMPTZ NOT NULL,
//!     metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
//!     INDEX idx_sessions_user_id (user_id),
//!     INDEX idx_sessions_tenant_id (tenant_id),
//!     INDEX idx_sessions_expires_at (expires_at)
//! );
//! ```
//!
//! # Performance
//!
//! - Uses connection pooling via Arc<PgPool>
//! - Indexes on user_id, tenant_id, expires_at for fast queries
//! - JSONB for flexible metadata storage
//! - Efficient batch cleanup with DELETE WHERE

use async_trait::async_trait;
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::session::{Session, SessionFilter, SessionStorage};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info, instrument};

/// PostgreSQL session storage adapter
pub struct PostgresSessionStorage {
    pool: Arc<PgPool>,
}

impl PostgresSessionStorage {
    /// Create new PostgreSQL session storage
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Convert database row to domain Session
    fn row_to_session(row: &PgRow) -> RiptideResult<Session> {
        let metadata_json: serde_json::Value = row
            .try_get("metadata")
            .map_err(|e| RiptideError::DatabaseError(format!("Failed to get metadata: {}", e)))?;

        let metadata: HashMap<String, String> =
            serde_json::from_value(metadata_json).map_err(|e| {
                RiptideError::SerializationError(format!("Invalid metadata JSON: {}", e))
            })?;

        Ok(Session {
            id: row
                .try_get("id")
                .map_err(|e| RiptideError::DatabaseError(format!("Failed to get id: {}", e)))?,
            user_id: row.try_get("user_id").map_err(|e| {
                RiptideError::DatabaseError(format!("Failed to get user_id: {}", e))
            })?,
            tenant_id: row.try_get("tenant_id").map_err(|e| {
                RiptideError::DatabaseError(format!("Failed to get tenant_id: {}", e))
            })?,
            created_at: row
                .try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map_err(|e| {
                    RiptideError::DatabaseError(format!("Failed to get created_at: {}", e))
                })?
                .into(),
            expires_at: row
                .try_get::<chrono::DateTime<chrono::Utc>, _>("expires_at")
                .map_err(|e| {
                    RiptideError::DatabaseError(format!("Failed to get expires_at: {}", e))
                })?
                .into(),
            metadata,
        })
    }

    /// Convert SystemTime to chrono DateTime for SQL
    fn to_datetime(time: SystemTime) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from(time)
    }
}

#[async_trait]
impl SessionStorage for PostgresSessionStorage {
    #[instrument(skip(self), fields(session_id = %id))]
    async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
        debug!("Fetching session from PostgreSQL");

        let result = sqlx::query(
            "SELECT id, user_id, tenant_id, created_at, expires_at, metadata
             FROM sessions
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| RiptideError::DatabaseError(format!("Failed to fetch session: {}", e)))?;

        match result {
            Some(row) => {
                let session = Self::row_to_session(&row)?;
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
        debug!("Saving session to PostgreSQL");

        let metadata_json = serde_json::to_value(&session.metadata).map_err(|e| {
            RiptideError::SerializationError(format!("Failed to serialize metadata: {}", e))
        })?;

        sqlx::query(
            "INSERT INTO sessions (id, user_id, tenant_id, created_at, expires_at, metadata)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
                user_id = EXCLUDED.user_id,
                tenant_id = EXCLUDED.tenant_id,
                created_at = EXCLUDED.created_at,
                expires_at = EXCLUDED.expires_at,
                metadata = EXCLUDED.metadata",
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.tenant_id)
        .bind(Self::to_datetime(session.created_at))
        .bind(Self::to_datetime(session.expires_at))
        .bind(metadata_json)
        .execute(&*self.pool)
        .await
        .map_err(|e| RiptideError::DatabaseError(format!("Failed to save session: {}", e)))?;

        info!("Session saved successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(session_id = %id))]
    async fn delete_session(&self, id: &str) -> RiptideResult<()> {
        debug!("Deleting session from PostgreSQL");

        let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| RiptideError::DatabaseError(format!("Failed to delete session: {}", e)))?;

        if result.rows_affected() > 0 {
            info!("Session deleted successfully");
        } else {
            debug!("Session not found for deletion");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
        debug!("Listing sessions with filter");

        let mut query = String::from(
            "SELECT id, user_id, tenant_id, created_at, expires_at, metadata FROM sessions WHERE 1=1"
        );

        let mut bindings = Vec::new();

        if let Some(user_id) = &filter.user_id {
            bindings.push(user_id.clone());
            query.push_str(&format!(" AND user_id = ${}", bindings.len()));
        }

        if let Some(tenant_id) = &filter.tenant_id {
            bindings.push(tenant_id.clone());
            query.push_str(&format!(" AND tenant_id = ${}", bindings.len()));
        }

        if filter.active_only {
            query.push_str(" AND expires_at > NOW()");
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query);
        for binding in bindings {
            q = q.bind(binding);
        }

        let rows = q
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| RiptideError::DatabaseError(format!("Failed to list sessions: {}", e)))?;

        let sessions: Result<Vec<_>, _> = rows.iter().map(Self::row_to_session).collect();
        let sessions = sessions?;

        info!("Listed {} sessions", sessions.len());
        Ok(sessions)
    }

    #[instrument(skip(self))]
    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        debug!("Cleaning up expired sessions");

        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < NOW()")
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                RiptideError::DatabaseError(format!("Failed to cleanup sessions: {}", e))
            })?;

        let count = result.rows_affected() as usize;
        info!("Cleaned up {} expired sessions", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    async fn create_test_pool() -> Arc<PgPool> {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/riptide_test".to_string());

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create test pool");

        Arc::new(pool)
    }

    async fn setup_schema(pool: &PgPool) {
        sqlx::query("DROP TABLE IF EXISTS sessions")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE sessions (
                id VARCHAR(255) PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                tenant_id VARCHAR(255) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ NOT NULL,
                metadata JSONB NOT NULL DEFAULT '{}'::jsonb
            )",
        )
        .execute(pool)
        .await
        .unwrap();

        sqlx::query("CREATE INDEX idx_sessions_user_id ON sessions(user_id)")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("CREATE INDEX idx_sessions_tenant_id ON sessions(tenant_id)")
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("CREATE INDEX idx_sessions_expires_at ON sessions(expires_at)")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL
    async fn test_save_and_get_session() {
        let pool = create_test_pool().await;
        setup_schema(&pool).await;

        let storage = PostgresSessionStorage::new(pool);

        let mut metadata = HashMap::new();
        metadata.insert("role".to_string(), "admin".to_string());

        let session = Session {
            id: "test-session-1".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata,
        };

        // Save session
        storage.save_session(&session).await.unwrap();

        // Retrieve session
        let retrieved = storage.get_session("test-session-1").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, session.id);
        assert_eq!(retrieved.user_id, session.user_id);
        assert_eq!(retrieved.metadata.get("role"), Some(&"admin".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL
    async fn test_delete_session() {
        let pool = create_test_pool().await;
        setup_schema(&pool).await;

        let storage = PostgresSessionStorage::new(pool);

        let session = Session {
            id: "test-session-2".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&session).await.unwrap();
        storage.delete_session("test-session-2").await.unwrap();

        let retrieved = storage.get_session("test-session-2").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL
    async fn test_cleanup_expired() {
        let pool = create_test_pool().await;
        setup_schema(&pool).await;

        let storage = PostgresSessionStorage::new(pool);

        // Create expired session
        let expired = Session {
            id: "expired-session".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7200),
            expires_at: SystemTime::now() - Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        // Create active session
        let active = Session {
            id: "active-session".to_string(),
            user_id: "user-123".to_string(),
            tenant_id: "tenant-456".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            metadata: HashMap::new(),
        };

        storage.save_session(&expired).await.unwrap();
        storage.save_session(&active).await.unwrap();

        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);

        assert!(storage
            .get_session("expired-session")
            .await
            .unwrap()
            .is_none());
        assert!(storage
            .get_session("active-session")
            .await
            .unwrap()
            .is_some());
    }
}
