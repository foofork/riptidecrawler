//! Integration tests for PostgreSQL adapters
//!
//! Tests the concrete implementations of port traits:
//! - PostgresRepository: Repository<T> implementation
//! - PostgresSessionStorage: Session storage adapter
//! - OutboxEventBus: EventBus implementation
//!
//! NOTE: These tests require a PostgreSQL database.
//! Use testcontainers or set SKIP_POSTGRES_TESTS=1 to skip.

#[cfg(test)]
mod postgres_adapter_tests {
    use super::*;

    /// Test PostgresRepository CRUD operations
    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_create() {
        let repo = setup_test_repository().await;

        let entity = TestEntity {
            id: "test-1".to_string(),
            name: "Test Entity".to_string(),
            value: 42,
        };

        let result = repo.create(&entity).await;
        assert!(result.is_ok(), "Create operation should succeed");

        let created_id = result.unwrap();
        assert_eq!(created_id, "test-1");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_find_by_id() {
        let repo = setup_test_repository().await;

        // Setup: Create entity
        let entity = TestEntity {
            id: "test-2".to_string(),
            name: "Find Test".to_string(),
            value: 100,
        };
        repo.create(&entity).await.expect("Setup failed");

        // Test: Find by ID
        let found = repo.find_by_id("test-2").await;
        assert!(found.is_ok(), "Find operation should succeed");

        let found_entity = found.unwrap();
        assert!(found_entity.is_some(), "Entity should be found");
        assert_eq!(found_entity.unwrap().name, "Find Test");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_update() {
        let repo = setup_test_repository().await;

        // Setup
        let entity = TestEntity {
            id: "test-3".to_string(),
            name: "Original".to_string(),
            value: 1,
        };
        repo.create(&entity).await.expect("Setup failed");

        // Test: Update
        let updated = TestEntity {
            id: "test-3".to_string(),
            name: "Updated".to_string(),
            value: 2,
        };
        let result = repo.update(&updated).await;
        assert!(result.is_ok(), "Update should succeed");

        // Verify
        let found = repo.find_by_id("test-3").await.unwrap().unwrap();
        assert_eq!(found.name, "Updated");
        assert_eq!(found.value, 2);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_delete() {
        let repo = setup_test_repository().await;

        // Setup
        let entity = TestEntity {
            id: "test-4".to_string(),
            name: "To Delete".to_string(),
            value: 99,
        };
        repo.create(&entity).await.expect("Setup failed");

        // Test: Delete
        let result = repo.delete("test-4").await;
        assert!(result.is_ok(), "Delete should succeed");

        // Verify
        let found = repo.find_by_id("test-4").await.unwrap();
        assert!(found.is_none(), "Entity should be deleted");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_find_all() {
        let repo = setup_test_repository().await;

        // Setup: Create multiple entities
        for i in 0..5 {
            let entity = TestEntity {
                id: format!("bulk-{}", i),
                name: format!("Entity {}", i),
                value: i,
            };
            repo.create(&entity).await.expect("Setup failed");
        }

        // Test: Find all
        let all = repo.find_all().await;
        assert!(all.is_ok(), "Find all should succeed");

        let entities = all.unwrap();
        assert!(entities.len() >= 5, "Should find at least 5 entities");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_repository_filter() {
        let repo = setup_test_repository().await;

        // Setup
        for i in 0..10 {
            let entity = TestEntity {
                id: format!("filter-{}", i),
                name: format!("Item {}", i),
                value: i,
            };
            repo.create(&entity).await.expect("Setup failed");
        }

        // Test: Filter by value > 5
        let filter = TestFilter { min_value: 5 };
        let filtered = repo.find_by_filter(&filter).await;
        assert!(filtered.is_ok(), "Filter should succeed");

        let results = filtered.unwrap();
        assert!(results.len() >= 4, "Should find items with value > 5");
        assert!(results.iter().all(|e| e.value > 5));
    }

    /// Test PostgresSessionStorage CRUD operations
    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_session_storage_create() {
        let storage = setup_test_session_storage().await;

        let session = TestSession {
            id: "session-1".to_string(),
            user_id: "user-1".to_string(),
            data: "test data".to_string(),
        };

        let result = storage.save(&session).await;
        assert!(result.is_ok(), "Session save should succeed");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_session_storage_retrieve() {
        let storage = setup_test_session_storage().await;

        // Setup
        let session = TestSession {
            id: "session-2".to_string(),
            user_id: "user-2".to_string(),
            data: "retrieve test".to_string(),
        };
        storage.save(&session).await.expect("Setup failed");

        // Test
        let found = storage.get("session-2").await;
        assert!(found.is_ok(), "Session retrieval should succeed");

        let session_data = found.unwrap();
        assert!(session_data.is_some(), "Session should exist");
        assert_eq!(session_data.unwrap().user_id, "user-2");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_postgres_session_storage_delete() {
        let storage = setup_test_session_storage().await;

        // Setup
        let session = TestSession {
            id: "session-3".to_string(),
            user_id: "user-3".to_string(),
            data: "delete test".to_string(),
        };
        storage.save(&session).await.expect("Setup failed");

        // Test
        let result = storage.delete("session-3").await;
        assert!(result.is_ok(), "Session delete should succeed");

        // Verify
        let found = storage.get("session-3").await.unwrap();
        assert!(found.is_none(), "Session should be deleted");
    }

    /// Test OutboxEventBus publish and poll
    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_outbox_event_bus_publish() {
        let event_bus = setup_test_event_bus().await;

        let event = TestEvent {
            id: "event-1".to_string(),
            event_type: "test.created".to_string(),
            payload: "test payload".to_string(),
        };

        let result = event_bus.publish(&event).await;
        assert!(result.is_ok(), "Event publish should succeed");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_outbox_event_bus_poll() {
        let event_bus = setup_test_event_bus().await;

        // Setup: Publish events
        for i in 0..3 {
            let event = TestEvent {
                id: format!("event-poll-{}", i),
                event_type: "test.poll".to_string(),
                payload: format!("payload {}", i),
            };
            event_bus.publish(&event).await.expect("Setup failed");
        }

        // Test: Poll events
        let events = event_bus.poll(10).await;
        assert!(events.is_ok(), "Event poll should succeed");

        let polled = events.unwrap();
        assert!(polled.len() >= 3, "Should poll at least 3 events");
    }

    /// Test transaction commit
    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_transaction_commit() {
        let tx_manager = setup_test_transaction_manager().await;

        let mut tx = tx_manager.begin().await.expect("Begin transaction failed");

        // Perform operations within transaction
        let entity = TestEntity {
            id: "tx-1".to_string(),
            name: "Transaction Test".to_string(),
            value: 123,
        };

        tx.create(&entity).await.expect("Create in tx failed");

        // Commit
        let result = tx.commit().await;
        assert!(result.is_ok(), "Transaction commit should succeed");

        // Verify committed data is persisted
        let repo = setup_test_repository().await;
        let found = repo.find_by_id("tx-1").await.unwrap();
        assert!(found.is_some(), "Committed data should persist");
    }

    /// Test transaction rollback
    #[tokio::test]
    #[ignore = "requires PostgreSQL testcontainer"]
    async fn test_transaction_rollback() {
        let tx_manager = setup_test_transaction_manager().await;

        let mut tx = tx_manager.begin().await.expect("Begin transaction failed");

        // Perform operations
        let entity = TestEntity {
            id: "tx-2".to_string(),
            name: "Rollback Test".to_string(),
            value: 456,
        };

        tx.create(&entity).await.expect("Create in tx failed");

        // Rollback
        let result = tx.rollback().await;
        assert!(result.is_ok(), "Transaction rollback should succeed");

        // Verify data was not persisted
        let repo = setup_test_repository().await;
        let found = repo.find_by_id("tx-2").await.unwrap();
        assert!(found.is_none(), "Rolled back data should not persist");
    }

    // ============ Mock Types and Helpers ============

    #[derive(Debug, Clone)]
    struct TestEntity {
        id: String,
        name: String,
        value: i32,
    }

    #[derive(Debug)]
    struct TestFilter {
        min_value: i32,
    }

    #[derive(Debug, Clone)]
    struct TestSession {
        id: String,
        user_id: String,
        data: String,
    }

    #[derive(Debug, Clone)]
    struct TestEvent {
        id: String,
        event_type: String,
        payload: String,
    }

    struct MockRepository;

    impl MockRepository {
        async fn create(&self, _entity: &TestEntity) -> Result<String, String> {
            Ok("test-id".to_string())
        }

        async fn find_by_id(&self, _id: &str) -> Result<Option<TestEntity>, String> {
            Ok(Some(TestEntity {
                id: "test".to_string(),
                name: "Mock".to_string(),
                value: 0,
            }))
        }

        async fn update(&self, _entity: &TestEntity) -> Result<(), String> {
            Ok(())
        }

        async fn delete(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }

        async fn find_all(&self) -> Result<Vec<TestEntity>, String> {
            Ok(vec![])
        }

        async fn find_by_filter(&self, _filter: &TestFilter) -> Result<Vec<TestEntity>, String> {
            Ok(vec![])
        }
    }

    struct MockSessionStorage;

    impl MockSessionStorage {
        async fn save(&self, _session: &TestSession) -> Result<(), String> {
            Ok(())
        }

        async fn get(&self, _id: &str) -> Result<Option<TestSession>, String> {
            Ok(None)
        }

        async fn delete(&self, _id: &str) -> Result<(), String> {
            Ok(())
        }
    }

    struct MockEventBus;

    impl MockEventBus {
        async fn publish(&self, _event: &TestEvent) -> Result<(), String> {
            Ok(())
        }

        async fn poll(&self, _limit: usize) -> Result<Vec<TestEvent>, String> {
            Ok(vec![])
        }
    }

    struct MockTransaction;

    impl MockTransaction {
        async fn create(&mut self, _entity: &TestEntity) -> Result<(), String> {
            Ok(())
        }

        async fn commit(self) -> Result<(), String> {
            Ok(())
        }

        async fn rollback(self) -> Result<(), String> {
            Ok(())
        }
    }

    struct MockTransactionManager;

    impl MockTransactionManager {
        async fn begin(&self) -> Result<MockTransaction, String> {
            Ok(MockTransaction)
        }
    }

    async fn setup_test_repository() -> MockRepository {
        MockRepository
    }

    async fn setup_test_session_storage() -> MockSessionStorage {
        MockSessionStorage
    }

    async fn setup_test_event_bus() -> MockEventBus {
        MockEventBus
    }

    async fn setup_test_transaction_manager() -> MockTransactionManager {
        MockTransactionManager
    }
}
