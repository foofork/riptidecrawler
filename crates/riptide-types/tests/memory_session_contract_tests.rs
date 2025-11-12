//! Contract tests for InMemorySessionStorage
//!
//! This module runs the complete SessionStorage contract test suite
//! against the InMemorySessionStorage implementation to ensure
//! it adheres to all expected behaviors.

use riptide_types::ports::InMemorySessionStorage;

// Include the contract test module
#[path = "contracts/session_storage_contract.rs"]
mod session_storage_contract;

/// Create a fresh InMemorySessionStorage for each test
fn create_storage() -> InMemorySessionStorage {
    // Use without_cleanup to avoid background task interference in tests
    InMemorySessionStorage::without_cleanup()
}

#[tokio::test]
async fn test_in_memory_crud_operations() {
    let storage = create_storage();
    session_storage_contract::test_crud_operations(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_expiration() {
    let storage = create_storage();
    session_storage_contract::test_expiration(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_multi_tenancy() {
    let storage = create_storage();
    session_storage_contract::test_multi_tenancy(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_user_filtering() {
    let storage = create_storage();
    session_storage_contract::test_user_filtering(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_active_filtering() {
    let storage = create_storage();
    session_storage_contract::test_active_filtering(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_metadata() {
    let storage = create_storage();
    session_storage_contract::test_metadata(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_concurrent_operations() {
    let storage = create_storage();
    session_storage_contract::test_concurrent_operations(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_empty_list() {
    let storage = create_storage();
    session_storage_contract::test_empty_list(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_cleanup_no_expired() {
    let storage = create_storage();
    session_storage_contract::test_cleanup_no_expired(&storage)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_in_memory_all_contracts() {
    let storage = create_storage();
    session_storage_contract::run_all_tests(&storage)
        .await
        .unwrap();
}
