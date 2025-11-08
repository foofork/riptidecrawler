//! Stub implementations for testing
//!
//! These are minimal in-memory implementations used for testing the composition root.
//! In production, these would be in separate crates (riptide-test-utils, etc.)

use async_trait::async_trait;
use riptide_types::{
    DomainEvent, EventBus, EventHandler, IdempotencyStore, IdempotencyToken, Repository,
    RepositoryFilter, Result as RiptideResult, RiptideError, SubscriptionId, Transaction,
    TransactionManager,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ============================================================================
// InMemoryRepository
// ============================================================================

pub struct InMemoryRepository<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<T> Default for InMemoryRepository<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> Repository<T> for InMemoryRepository<T>
where
    T: Send + Sync + Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn find_by_id(&self, id: &str) -> RiptideResult<Option<T>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(id).cloned())
    }

    async fn find_all(&self, _filter: RepositoryFilter) -> RiptideResult<Vec<T>> {
        let data = self.data.lock().unwrap();
        Ok(data.values().cloned().collect())
    }

    async fn save(&self, entity: &T) -> RiptideResult<()> {
        let json = serde_json::to_value(entity)
            .map_err(|e| RiptideError::Custom(format!("Failed to serialize: {}", e)))?;

        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RiptideError::Custom("Entity must have an 'id' field".to_string()))?
            .to_string();

        let mut data = self.data.lock().unwrap();
        data.insert(id, entity.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> RiptideResult<()> {
        let mut data = self.data.lock().unwrap();
        data.remove(id);
        Ok(())
    }

    async fn count(&self, _filter: RepositoryFilter) -> RiptideResult<usize> {
        let data = self.data.lock().unwrap();
        Ok(data.len())
    }

    async fn exists(&self, id: &str) -> RiptideResult<bool> {
        let data = self.data.lock().unwrap();
        Ok(data.contains_key(id))
    }
}

// ============================================================================
// InMemoryEventBus
// ============================================================================

pub struct InMemoryEventBus {
    events: Arc<Mutex<Vec<DomainEvent>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn published_events(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: DomainEvent) -> RiptideResult<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event);
        Ok(())
    }

    async fn subscribe(&self, _handler: Arc<dyn EventHandler>) -> RiptideResult<SubscriptionId> {
        // In-memory implementation doesn't support subscriptions
        Ok(format!("in-memory-sub-{}", uuid::Uuid::new_v4()))
    }

    async fn publish_batch(&self, events: Vec<DomainEvent>) -> RiptideResult<()> {
        let mut stored_events = self.events.lock().unwrap();
        stored_events.extend(events);
        Ok(())
    }
}

// ============================================================================
// InMemoryIdempotencyStore
// ============================================================================

pub struct InMemoryIdempotencyStore {
    keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {
    async fn try_acquire(&self, key: &str, ttl: Duration) -> RiptideResult<IdempotencyToken> {
        let mut keys = self.keys.lock().unwrap();

        if keys.contains_key(key) {
            return Err(RiptideError::AlreadyExists(format!(
                "Idempotency key already exists: {}",
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
        let keys = self.keys.lock().unwrap();
        Ok(keys.contains_key(key))
    }

    async fn ttl(&self, _key: &str) -> RiptideResult<Option<Duration>> {
        // In-memory implementation doesn't track TTL
        Ok(Some(Duration::from_secs(3600)))
    }

    async fn store_result(&self, key: &str, result: &[u8], _ttl: Duration) -> RiptideResult<()> {
        let mut keys = self.keys.lock().unwrap();
        keys.insert(key.to_string(), result.to_vec());
        Ok(())
    }

    async fn get_result(&self, key: &str) -> RiptideResult<Option<Vec<u8>>> {
        let keys = self.keys.lock().unwrap();
        Ok(keys.get(key).cloned())
    }

    async fn cleanup_expired(&self) -> RiptideResult<usize> {
        // In-memory implementation doesn't have expiration
        Ok(0)
    }
}

// ============================================================================
// InMemoryTransactionManager
// ============================================================================

pub struct InMemoryTransactionManager;

impl InMemoryTransactionManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InMemoryTransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TransactionManager for InMemoryTransactionManager {
    type Transaction = InMemoryTransaction;

    async fn begin(&self) -> RiptideResult<Self::Transaction> {
        Ok(InMemoryTransaction::new())
    }

    async fn commit(&self, _tx: Self::Transaction) -> RiptideResult<()> {
        Ok(())
    }

    async fn rollback(&self, _tx: Self::Transaction) -> RiptideResult<()> {
        Ok(())
    }
}

// ============================================================================
// InMemoryTransaction
// ============================================================================

pub struct InMemoryTransaction {
    id: String,
}

impl InMemoryTransaction {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Default for InMemoryTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transaction for InMemoryTransaction {
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
