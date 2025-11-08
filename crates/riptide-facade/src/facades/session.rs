//! Session facade providing business logic for session management
//!
//! This facade coordinates session operations across multiple ports:
//! - SessionStorage: Persisting and retrieving sessions
//! - IdempotencyStore: Preventing duplicate operations
//! - EventBus: Publishing session lifecycle events
//! - Clock: Time management for testing
//!
//! # Business Rules
//!
//! - Sessions have configurable TTL (default: 24 hours)
//! - Session creation is idempotent
//! - Session validation checks expiration
//! - Session refresh extends expiration
//! - Session termination publishes events

use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{Clock, IdempotencyStore, Session, SessionFilter, SessionStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Session lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    SessionCreated {
        session_id: String,
        user_id: String,
        tenant_id: String,
        created_at: SystemTime,
    },
    SessionValidated {
        session_id: String,
        user_id: String,
    },
    SessionRefreshed {
        session_id: String,
        user_id: String,
        new_expires_at: SystemTime,
    },
    SessionTerminated {
        session_id: String,
        user_id: String,
        reason: String,
    },
}

impl SessionEvent {
    #[allow(dead_code)]
    fn event_type(&self) -> &str {
        match self {
            SessionEvent::SessionCreated { .. } => "session.created",
            SessionEvent::SessionValidated { .. } => "session.validated",
            SessionEvent::SessionRefreshed { .. } => "session.refreshed",
            SessionEvent::SessionTerminated { .. } => "session.terminated",
        }
    }

    #[allow(dead_code)]
    fn aggregate_id(&self) -> &str {
        match self {
            SessionEvent::SessionCreated { session_id, .. } => session_id,
            SessionEvent::SessionValidated { session_id, .. } => session_id,
            SessionEvent::SessionRefreshed { session_id, .. } => session_id,
            SessionEvent::SessionTerminated { session_id, .. } => session_id,
        }
    }

    #[allow(dead_code)]
    fn to_domain_event(&self) -> riptide_types::ports::DomainEvent {
        riptide_types::ports::DomainEvent::new(
            self.event_type(),
            self.aggregate_id(),
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null),
        )
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Default session TTL
    pub default_ttl: Duration,

    /// Maximum session TTL
    pub max_ttl: Duration,

    /// Enable session refresh
    pub allow_refresh: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(24 * 3600), // 24 hours
            max_ttl: Duration::from_secs(7 * 24 * 3600), // 7 days
            allow_refresh: true,
        }
    }
}

/// Session facade coordinating session management
pub struct SessionFacade {
    storage: Arc<dyn SessionStorage>,
    idempotency: Arc<dyn IdempotencyStore>,
    clock: Arc<dyn Clock>,
    config: SessionConfig,
}

impl SessionFacade {
    /// Create new session facade
    pub fn new(
        storage: Arc<dyn SessionStorage>,
        idempotency: Arc<dyn IdempotencyStore>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            storage,
            idempotency,
            clock,
            config: SessionConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        storage: Arc<dyn SessionStorage>,
        idempotency: Arc<dyn IdempotencyStore>,
        clock: Arc<dyn Clock>,
        config: SessionConfig,
    ) -> Self {
        Self {
            storage,
            idempotency,
            clock,
            config,
        }
    }

    /// Create a new session with idempotency
    #[instrument(skip(self), fields(user_id = %user_id, tenant_id = %tenant_id))]
    pub async fn create_session(&self, user_id: &str, tenant_id: &str) -> RiptideResult<Session> {
        self.create_session_with_ttl(user_id, tenant_id, None).await
    }

    /// Create session with custom TTL
    #[instrument(skip(self), fields(user_id = %user_id, tenant_id = %tenant_id))]
    pub async fn create_session_with_ttl(
        &self,
        user_id: &str,
        tenant_id: &str,
        ttl: Option<Duration>,
    ) -> RiptideResult<Session> {
        debug!("Creating session");

        let ttl = ttl.unwrap_or(self.config.default_ttl);
        if ttl > self.config.max_ttl {
            return Err(RiptideError::ValidationError(format!(
                "TTL exceeds maximum of {:?}",
                self.config.max_ttl
            )));
        }

        // Generate unique session ID
        let session_id = format!("session_{}", Uuid::new_v4());

        // Idempotency key for session creation
        let idempotency_key = format!("session_create_{}_{}", user_id, tenant_id);
        let token = self
            .idempotency
            .try_acquire(&idempotency_key, ttl)
            .await
            .map_err(|_| {
                RiptideError::ValidationError("Duplicate session creation request".to_string())
            })?;

        let now = self.clock.now();
        let expires_at = now + ttl;

        let session = Session {
            id: session_id.clone(),
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            created_at: now,
            expires_at,
            metadata: HashMap::new(),
        };

        // Save session
        self.storage.save_session(&session).await?;

        // Release idempotency lock
        self.idempotency.release(token).await?;

        // TODO: Publish event when EventBus is object-safe or use concrete type
        debug!("SessionCreated event: session_id={}", session_id);

        info!("Session created: {}", session_id);
        Ok(session)
    }

    /// Validate session exists and is not expired
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn validate_session(&self, session_id: &str) -> RiptideResult<bool> {
        debug!("Validating session");

        match self.storage.get_session(session_id).await? {
            Some(session) => {
                if session.is_expired() {
                    debug!("Session is expired");
                    // Clean up expired session
                    self.storage.delete_session(session_id).await?;
                    Ok(false)
                } else {
                    debug!("Session is valid");
                    // TODO: Publish validation event when EventBus is object-safe
                    Ok(true)
                }
            }
            None => {
                debug!("Session not found");
                Ok(false)
            }
        }
    }

    /// Get session details
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn get_session(&self, session_id: &str) -> RiptideResult<Option<Session>> {
        self.storage.get_session(session_id).await
    }

    /// Refresh session by extending expiration
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn refresh_session(&self, session_id: &str) -> RiptideResult<Session> {
        debug!("Refreshing session");

        if !self.config.allow_refresh {
            return Err(RiptideError::ValidationError(
                "Session refresh is disabled".to_string(),
            ));
        }

        let mut session =
            self.storage.get_session(session_id).await?.ok_or_else(|| {
                RiptideError::NotFound(format!("Session {} not found", session_id))
            })?;

        if session.is_expired() {
            return Err(RiptideError::ValidationError(
                "Cannot refresh expired session".to_string(),
            ));
        }

        let now = self.clock.now();
        let new_expires_at = now + self.config.default_ttl;

        session.expires_at = new_expires_at;
        self.storage.save_session(&session).await?;

        info!("Session refreshed: {}", session_id);
        // TODO: Publish event when EventBus is object-safe
        Ok(session)
    }

    /// Terminate session (logout)
    #[instrument(skip(self), fields(session_id = %session_id))]
    pub async fn terminate_session(&self, session_id: &str) -> RiptideResult<()> {
        debug!("Terminating session");

        let session = self.storage.get_session(session_id).await?;

        self.storage.delete_session(session_id).await?;

        if let Some(_session) = session {
            // TODO: Publish event when EventBus is object-safe
            info!("Session terminated: {}", session_id);
        }

        Ok(())
    }

    /// List sessions for a user
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_user_sessions(&self, user_id: &str) -> RiptideResult<Vec<Session>> {
        let filter = SessionFilter {
            user_id: Some(user_id.to_string()),
            tenant_id: None,
            active_only: true,
        };

        self.storage.list_sessions(filter).await
    }

    /// List sessions for a tenant
    #[instrument(skip(self), fields(tenant_id = %tenant_id))]
    pub async fn list_tenant_sessions(&self, tenant_id: &str) -> RiptideResult<Vec<Session>> {
        let filter = SessionFilter {
            user_id: None,
            tenant_id: Some(tenant_id.to_string()),
            active_only: true,
        };

        self.storage.list_sessions(filter).await
    }

    /// Cleanup expired sessions
    #[instrument(skip(self))]
    pub async fn cleanup_expired_sessions(&self) -> RiptideResult<usize> {
        debug!("Cleaning up expired sessions");
        self.storage.cleanup_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ports::{IdempotencyToken, InMemoryCache, SystemClock};
    use std::sync::Mutex;

    // Mock implementations for testing
    struct InMemorySessionStorage {
        sessions: Arc<Mutex<HashMap<String, Session>>>,
    }

    impl InMemorySessionStorage {
        fn new() -> Self {
            Self {
                sessions: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl SessionStorage for InMemorySessionStorage {
        async fn get_session(&self, id: &str) -> RiptideResult<Option<Session>> {
            Ok(self.sessions.lock().unwrap().get(id).cloned())
        }

        async fn save_session(&self, session: &Session) -> RiptideResult<()> {
            self.sessions
                .lock()
                .unwrap()
                .insert(session.id.clone(), session.clone());
            Ok(())
        }

        async fn delete_session(&self, id: &str) -> RiptideResult<()> {
            self.sessions.lock().unwrap().remove(id);
            Ok(())
        }

        async fn list_sessions(&self, filter: SessionFilter) -> RiptideResult<Vec<Session>> {
            let sessions = self.sessions.lock().unwrap();
            let filtered: Vec<_> = sessions
                .values()
                .filter(|s| {
                    if let Some(ref user_id) = filter.user_id {
                        if &s.user_id != user_id {
                            return false;
                        }
                    }
                    if let Some(ref tenant_id) = filter.tenant_id {
                        if &s.tenant_id != tenant_id {
                            return false;
                        }
                    }
                    if filter.active_only && s.is_expired() {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn cleanup_expired(&self) -> RiptideResult<usize> {
            let mut sessions = self.sessions.lock().unwrap();
            let before = sessions.len();
            sessions.retain(|_, s| !s.is_expired());
            Ok(before - sessions.len())
        }
    }

    use async_trait::async_trait;

    struct MockIdempotencyStore;

    #[async_trait]
    impl IdempotencyStore for MockIdempotencyStore {
        async fn try_acquire(&self, key: &str, _ttl: Duration) -> RiptideResult<IdempotencyToken> {
            Ok(IdempotencyToken {
                key: key.to_string(),
                acquired_at: SystemTime::now(),
            })
        }

        async fn release(&self, _token: IdempotencyToken) -> RiptideResult<()> {
            Ok(())
        }

        async fn is_acquired(&self, _key: &str) -> RiptideResult<bool> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let idempotency = Arc::new(MockIdempotencyStore);
        let clock = Arc::new(SystemClock);

        let facade = SessionFacade::new(storage.clone(), idempotency, clock);

        let session = facade
            .create_session("user-123", "tenant-456")
            .await
            .unwrap();
        assert_eq!(session.user_id, "user-123");
        assert_eq!(session.tenant_id, "tenant-456");
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn test_validate_session() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let idempotency = Arc::new(MockIdempotencyStore);
        let clock = Arc::new(SystemClock);

        let facade = SessionFacade::new(storage.clone(), idempotency, clock);

        let session = facade
            .create_session("user-123", "tenant-456")
            .await
            .unwrap();
        let is_valid = facade.validate_session(&session.id).await.unwrap();
        assert!(is_valid);

        let is_valid_fake = facade.validate_session("fake-session").await.unwrap();
        assert!(!is_valid_fake);
    }

    #[tokio::test]
    async fn test_terminate_session() {
        let storage = Arc::new(InMemorySessionStorage::new());
        let idempotency = Arc::new(MockIdempotencyStore);
        let clock = Arc::new(SystemClock);

        let facade = SessionFacade::new(storage.clone(), idempotency, clock);

        let session = facade
            .create_session("user-123", "tenant-456")
            .await
            .unwrap();
        facade.terminate_session(&session.id).await.unwrap();

        let is_valid = facade.validate_session(&session.id).await.unwrap();
        assert!(!is_valid);
    }
}
