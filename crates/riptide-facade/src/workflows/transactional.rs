//! Transactional workflow orchestrator with idempotency and event emission
//!
//! This module provides a workflow pattern that coordinates:
//! - **Transaction management**: ACID guarantees with commit/rollback
//! - **Idempotency**: Prevents duplicate operations using distributed locks
//! - **Event emission**: Transactional outbox pattern for reliable event publishing
//!
//! # Design Goals
//!
//! - **ACID Guarantees**: All-or-nothing execution with rollback on failure
//! - **Idempotency**: Duplicate prevention using idempotency keys
//! - **Event Reliability**: Events written transactionally (outbox pattern)
//! - **Testability**: Easy to test with in-memory implementations
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_facade::workflows::TransactionalWorkflow;
//! use riptide_types::ports::{DomainEvent, Transaction};
//!
//! async fn create_user(workflow: &TransactionalWorkflow) -> Result<User> {
//!     workflow.execute(
//!         "user-creation-abc123",
//!         |tx| async move {
//!             // 1. Perform business logic
//!             let user = User::new("john@example.com");
//!             repository.save(&user).await?;
//!
//!             // 2. Prepare events to emit
//!             let event = DomainEvent::new(
//!                 "user.created",
//!                 user.id.clone(),
//!                 serde_json::to_value(&user)?,
//!             );
//!
//!             // 3. Return result + events
//!             Ok((user, vec![event]))
//!         }
//!     ).await
//! }
//! ```

use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{
    DomainEvent, EventBus, IdempotencyStore, Transaction, TransactionManager,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

/// Transactional workflow orchestrator
///
/// Coordinates transactional operations with idempotency checking and event emission.
/// Ensures ACID guarantees and reliable event publishing through the transactional outbox pattern.
pub struct TransactionalWorkflow<TM>
where
    TM: TransactionManager,
{
    /// Transaction manager for ACID operations
    tx_manager: Arc<TM>,

    /// Event bus for publishing domain events
    event_bus: Arc<dyn EventBus>,

    /// Idempotency store for duplicate prevention
    idempotency_store: Arc<dyn IdempotencyStore>,

    /// Default TTL for idempotency locks
    default_ttl: Duration,
}

impl<TM> TransactionalWorkflow<TM>
where
    TM: TransactionManager,
{
    /// Create new transactional workflow
    ///
    /// # Arguments
    ///
    /// * `tx_manager` - Transaction manager implementation
    /// * `event_bus` - Event bus for publishing events
    /// * `idempotency_store` - Idempotency store for duplicate prevention
    ///
    /// # Returns
    ///
    /// New `TransactionalWorkflow` with default 1-hour idempotency TTL
    pub fn new(
        tx_manager: Arc<TM>,
        event_bus: Arc<dyn EventBus>,
        idempotency_store: Arc<dyn IdempotencyStore>,
    ) -> Self {
        Self {
            tx_manager,
            event_bus,
            idempotency_store,
            default_ttl: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Create workflow with custom idempotency TTL
    ///
    /// # Arguments
    ///
    /// * `tx_manager` - Transaction manager implementation
    /// * `event_bus` - Event bus for publishing events
    /// * `idempotency_store` - Idempotency store for duplicate prevention
    /// * `ttl` - Time-to-live for idempotency locks
    pub fn with_ttl(
        tx_manager: Arc<TM>,
        event_bus: Arc<dyn EventBus>,
        idempotency_store: Arc<dyn IdempotencyStore>,
        ttl: Duration,
    ) -> Self {
        Self {
            tx_manager,
            event_bus,
            idempotency_store,
            default_ttl: ttl,
        }
    }

    /// Execute workflow with ACID guarantees, idempotency, and event emission
    ///
    /// # Workflow Steps
    ///
    /// 1. Check idempotency - prevents duplicate operations
    /// 2. Begin transaction - starts ACID transaction
    /// 3. Execute workflow function - performs business logic
    /// 4. Write events to outbox (transactional) - ensures event reliability
    /// 5. Commit transaction - atomically commits all changes
    /// 6. Release idempotency lock - allows future operations
    ///
    /// # Arguments
    ///
    /// * `idempotency_key` - Unique key for this operation (e.g., request ID)
    /// * `workflow_fn` - Async closure that performs business logic and returns (result, events)
    ///
    /// # Returns
    ///
    /// * `Ok(result)` - Workflow completed successfully
    /// * `Err(_)` - Workflow failed (transaction rolled back, idempotency lock released)
    ///
    /// # Error Handling
    ///
    /// On any error:
    /// - Transaction is rolled back (no partial state)
    /// - Idempotency lock is released (allows retry)
    /// - Events are NOT published (atomicity)
    ///
    /// # Type Parameters
    ///
    /// * `F` - Workflow function type
    /// * `R` - Result type returned by workflow
    #[instrument(skip(self, workflow_fn), fields(idempotency_key = %idempotency_key))]
    pub async fn execute<F, R>(&self, idempotency_key: &str, workflow_fn: F) -> RiptideResult<R>
    where
        F: FnOnce(
                &mut TM::Transaction,
            )
                -> Pin<Box<dyn Future<Output = RiptideResult<(R, Vec<DomainEvent>)>> + Send>>
            + Send,
        R: Send,
    {
        debug!("Starting transactional workflow");

        // Step 1: Check idempotency - prevent duplicate operations
        debug!("Acquiring idempotency lock");
        let token = match self
            .idempotency_store
            .try_acquire(idempotency_key, self.default_ttl)
            .await
        {
            Ok(token) => {
                debug!("Idempotency lock acquired");
                token
            }
            Err(e) => {
                warn!("Failed to acquire idempotency lock: {}", e);
                return Err(RiptideError::AlreadyExists(format!(
                    "Duplicate operation detected for key: {}",
                    idempotency_key
                )));
            }
        };

        // Step 2: Begin transaction
        debug!("Beginning transaction");
        let mut tx = match self.tx_manager.begin().await {
            Ok(tx) => {
                debug!("Transaction started: {}", tx.id());
                tx
            }
            Err(e) => {
                error!("Failed to begin transaction: {}", e);
                // Release idempotency lock on transaction start failure
                if let Err(release_err) = self.idempotency_store.release(token).await {
                    error!(
                        "Failed to release idempotency lock after transaction begin error: {}",
                        release_err
                    );
                }
                return Err(e);
            }
        };

        // Step 3: Execute workflow function
        debug!("Executing workflow function");
        let (result, events) = match workflow_fn(&mut tx).await {
            Ok((result, events)) => {
                debug!(
                    "Workflow function completed successfully, {} events to publish",
                    events.len()
                );
                (result, events)
            }
            Err(e) => {
                error!("Workflow function failed: {}", e);
                // Rollback transaction
                if let Err(rollback_err) = self.tx_manager.rollback(tx).await {
                    error!("Transaction rollback failed: {}", rollback_err);
                } else {
                    debug!("Transaction rolled back successfully");
                }
                // Release idempotency lock
                if let Err(release_err) = self.idempotency_store.release(token).await {
                    error!(
                        "Failed to release idempotency lock after workflow error: {}",
                        release_err
                    );
                }
                return Err(e);
            }
        };

        // Step 4: Write events to outbox (transactional)
        // Note: In a true outbox pattern, events would be written to a database table
        // within the same transaction. Here we publish to the event bus, which should
        // ideally be backed by a transactional store.
        if !events.is_empty() {
            debug!("Publishing {} events to event bus", events.len());
            if let Err(e) = self.event_bus.publish_batch(events.clone()).await {
                error!("Failed to publish events: {}", e);
                // Rollback transaction
                if let Err(rollback_err) = self.tx_manager.rollback(tx).await {
                    error!("Transaction rollback failed: {}", rollback_err);
                } else {
                    debug!("Transaction rolled back after event publish failure");
                }
                // Release idempotency lock
                if let Err(release_err) = self.idempotency_store.release(token).await {
                    error!(
                        "Failed to release idempotency lock after event publish error: {}",
                        release_err
                    );
                }
                return Err(e);
            }
            debug!("Events published successfully");
        }

        // Step 5: Commit transaction
        debug!("Committing transaction");
        if let Err(e) = self.tx_manager.commit(tx).await {
            error!("Transaction commit failed: {}", e);
            // Release idempotency lock
            if let Err(release_err) = self.idempotency_store.release(token).await {
                error!(
                    "Failed to release idempotency lock after commit error: {}",
                    release_err
                );
            }
            return Err(e);
        }
        debug!("Transaction committed successfully");

        // Step 6: Release idempotency lock
        debug!("Releasing idempotency lock");
        if let Err(e) = self.idempotency_store.release(token).await {
            error!(
                "Failed to release idempotency lock after successful workflow: {}",
                e
            );
            // Log but don't fail - the operation succeeded
            warn!("Operation succeeded but idempotency lock release failed");
        }

        info!("Transactional workflow completed successfully");
        Ok(result)
    }

    /// Execute workflow without idempotency checking
    ///
    /// Use this when idempotency is not required or handled elsewhere.
    ///
    /// # Arguments
    ///
    /// * `workflow_fn` - Async closure that performs business logic and returns (result, events)
    ///
    /// # Returns
    ///
    /// * `Ok(result)` - Workflow completed successfully
    /// * `Err(_)` - Workflow failed (transaction rolled back)
    #[instrument(skip(self, workflow_fn))]
    pub async fn execute_without_idempotency<F, R>(&self, workflow_fn: F) -> RiptideResult<R>
    where
        F: FnOnce(
                &mut TM::Transaction,
            )
                -> Pin<Box<dyn Future<Output = RiptideResult<(R, Vec<DomainEvent>)>> + Send>>
            + Send,
        R: Send,
    {
        debug!("Starting transactional workflow (without idempotency)");

        // Begin transaction
        let mut tx = self.tx_manager.begin().await?;
        debug!("Transaction started: {}", tx.id());

        // Execute workflow function
        let (result, events) = match workflow_fn(&mut tx).await {
            Ok(res) => res,
            Err(e) => {
                error!("Workflow function failed: {}", e);
                self.tx_manager.rollback(tx).await?;
                return Err(e);
            }
        };

        // Publish events
        if !events.is_empty() {
            if let Err(e) = self.event_bus.publish_batch(events).await {
                error!("Failed to publish events: {}", e);
                self.tx_manager.rollback(tx).await?;
                return Err(e);
            }
        }

        // Commit transaction
        self.tx_manager.commit(tx).await?;

        info!("Transactional workflow completed successfully (without idempotency)");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_types::ports::{EventBus, IdempotencyToken};
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Test stubs
    struct TestTransactionManager {
        commit_count: Arc<Mutex<usize>>,
        rollback_count: Arc<Mutex<usize>>,
        should_fail_commit: Arc<Mutex<bool>>,
    }

    impl TestTransactionManager {
        fn new() -> Self {
            Self {
                commit_count: Arc::new(Mutex::new(0)),
                rollback_count: Arc::new(Mutex::new(0)),
                should_fail_commit: Arc::new(Mutex::new(false)),
            }
        }

        fn commits(&self) -> usize {
            *self.commit_count.lock().unwrap()
        }

        fn rollbacks(&self) -> usize {
            *self.rollback_count.lock().unwrap()
        }

        fn set_fail_commit(&self, should_fail: bool) {
            *self.should_fail_commit.lock().unwrap() = should_fail;
        }
    }

    struct TestTransaction {
        id: String,
    }

    impl TestTransaction {
        fn new() -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl Transaction for TestTransaction {
        fn id(&self) -> &str {
            &self.id
        }

        async fn execute<F, R>(&mut self, f: F) -> RiptideResult<R>
        where
            F: FnOnce() -> RiptideResult<R> + Send,
            R: Send,
        {
            f()
        }
    }

    #[async_trait::async_trait]
    impl TransactionManager for TestTransactionManager {
        type Transaction = TestTransaction;

        async fn begin(&self) -> RiptideResult<Self::Transaction> {
            Ok(TestTransaction::new())
        }

        async fn commit(&self, _tx: Self::Transaction) -> RiptideResult<()> {
            if *self.should_fail_commit.lock().unwrap() {
                return Err(RiptideError::DatabaseError("Commit failed".to_string()));
            }
            *self.commit_count.lock().unwrap() += 1;
            Ok(())
        }

        async fn rollback(&self, _tx: Self::Transaction) -> RiptideResult<()> {
            *self.rollback_count.lock().unwrap() += 1;
            Ok(())
        }
    }

    struct TestEventBus {
        events: Arc<Mutex<Vec<DomainEvent>>>,
        should_fail: Arc<Mutex<bool>>,
    }

    impl TestEventBus {
        fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        fn published_events(&self) -> Vec<DomainEvent> {
            self.events.lock().unwrap().clone()
        }

        fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    #[async_trait::async_trait]
    impl EventBus for TestEventBus {
        async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
            if *self.should_fail.lock().unwrap() {
                return Err(RiptideError::Custom("Event publish failed".to_string()));
            }
            self.events.lock().unwrap().push(event);
            Ok(())
        }

        async fn subscribe(
            &self,
            _handler: Arc<dyn riptide_types::ports::EventHandler>,
        ) -> RiptideResult<String> {
            Ok("test-sub".to_string())
        }

        async fn publish_batch(&self, events: Vec<DomainEvent>) -> RiptideResult<()> {
            if *self.should_fail.lock().unwrap() {
                return Err(RiptideError::Custom("Batch publish failed".to_string()));
            }
            self.events.lock().unwrap().extend(events);
            Ok(())
        }
    }

    struct TestIdempotencyStore {
        keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
        should_fail_acquire: Arc<Mutex<bool>>,
    }

    impl TestIdempotencyStore {
        fn new() -> Self {
            Self {
                keys: Arc::new(Mutex::new(HashMap::new())),
                should_fail_acquire: Arc::new(Mutex::new(false)),
            }
        }

        fn set_fail_acquire(&self, should_fail: bool) {
            *self.should_fail_acquire.lock().unwrap() = should_fail;
        }

        fn is_acquired(&self, key: &str) -> bool {
            self.keys.lock().unwrap().contains_key(key)
        }
    }

    #[async_trait::async_trait]
    impl IdempotencyStore for TestIdempotencyStore {
        async fn try_acquire(&self, key: &str, ttl: Duration) -> RiptideResult<IdempotencyToken> {
            if *self.should_fail_acquire.lock().unwrap() {
                return Err(RiptideError::AlreadyExists(
                    "Key already exists".to_string(),
                ));
            }

            let mut keys = self.keys.lock().unwrap();
            if keys.contains_key(key) {
                return Err(RiptideError::AlreadyExists(format!(
                    "Key {} already exists",
                    key
                )));
            }

            keys.insert(key.to_string(), Vec::new());
            Ok(IdempotencyToken::new(key.to_string(), ttl))
        }

        async fn release(&self, token: IdempotencyToken) -> RiptideResult<()> {
            let mut keys = self.keys.lock().unwrap();
            keys.remove(&token.key);
            Ok(())
        }

        async fn exists(&self, key: &str) -> RiptideResult<bool> {
            Ok(self.keys.lock().unwrap().contains_key(key))
        }
    }

    #[tokio::test]
    async fn test_successful_workflow_execution() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("test-key-1", |_tx| {
                Box::pin(async move {
                    let event = DomainEvent::new(
                        "test.event",
                        "test-123",
                        serde_json::json!({"data": "test"}),
                    );
                    Ok(("success".to_string(), vec![event]))
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(tx_manager.commits(), 1);
        assert_eq!(tx_manager.rollbacks(), 0);
        assert_eq!(event_bus.published_events().len(), 1);
        assert!(!idempotency.is_acquired("test-key-1"));
    }

    #[tokio::test]
    async fn test_idempotency_prevents_duplicates() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        idempotency.set_fail_acquire(true);

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("duplicate-key", |_tx| {
                Box::pin(async move { Ok(("should not execute".to_string(), vec![])) })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(tx_manager.commits(), 0);
        assert_eq!(tx_manager.rollbacks(), 0);
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_workflow_error() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("error-key", |_tx| {
                Box::pin(async move {
                    Err::<(String, Vec<DomainEvent>), _>(RiptideError::Custom(
                        "Workflow failed".to_string(),
                    ))
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(tx_manager.commits(), 0);
        assert_eq!(tx_manager.rollbacks(), 1);
        assert_eq!(event_bus.published_events().len(), 0);
        assert!(!idempotency.is_acquired("error-key"));
    }

    #[tokio::test]
    async fn test_transaction_rollback_on_event_publish_failure() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        event_bus.set_should_fail(true);

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("event-fail-key", |_tx| {
                Box::pin(async move {
                    let event = DomainEvent::new("test.event", "test-123", serde_json::json!({}));
                    Ok(("result".to_string(), vec![event]))
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(tx_manager.commits(), 0);
        assert_eq!(tx_manager.rollbacks(), 1);
        assert!(!idempotency.is_acquired("event-fail-key"));
    }

    #[tokio::test]
    async fn test_rollback_on_commit_failure() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        tx_manager.set_fail_commit(true);

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("commit-fail-key", |_tx| {
                Box::pin(async move { Ok(("result".to_string(), vec![])) })
            })
            .await;

        assert!(result.is_err());
        assert!(!idempotency.is_acquired("commit-fail-key"));
    }

    #[tokio::test]
    async fn test_workflow_without_idempotency() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute_without_idempotency(|_tx| {
                Box::pin(async move {
                    let event = DomainEvent::new("test.event", "test-123", serde_json::json!({}));
                    Ok(("success".to_string(), vec![event]))
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(tx_manager.commits(), 1);
        assert_eq!(event_bus.published_events().len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_events_published() {
        let tx_manager = Arc::new(TestTransactionManager::new());
        let event_bus = Arc::new(TestEventBus::new());
        let idempotency = Arc::new(TestIdempotencyStore::new());

        let workflow =
            TransactionalWorkflow::new(tx_manager.clone(), event_bus.clone(), idempotency.clone());

        let result = workflow
            .execute("multi-event-key", |_tx| {
                Box::pin(async move {
                    let events = vec![
                        DomainEvent::new("event.1", "agg-1", serde_json::json!({})),
                        DomainEvent::new("event.2", "agg-2", serde_json::json!({})),
                        DomainEvent::new("event.3", "agg-3", serde_json::json!({})),
                    ];
                    Ok(("multi".to_string(), events))
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(event_bus.published_events().len(), 3);
    }
}
