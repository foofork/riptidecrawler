//! Test to validate that all lifetime fixes are working correctly

use std::sync::{Mutex, RwLock};
use riptide_core::monitoring::error::{LockManager, Result};

#[test]
fn test_mutex_lock_lifetime() {
    let data = Mutex::new(42);
    let result: Result<std::sync::MutexGuard<'_, i32>> = LockManager::acquire_mutex(&data, "test_operation");
    assert!(result.is_ok());

    if let Ok(guard) = result {
        assert_eq!(*guard, 42);
    }
}

#[test]
fn test_rwlock_read_lifetime() {
    let data = RwLock::new(vec![1, 2, 3]);
    let result: Result<std::sync::RwLockReadGuard<'_, Vec<i32>>> = LockManager::acquire_read(&data, "test_read");
    assert!(result.is_ok());

    if let Ok(guard) = result {
        assert_eq!(guard.len(), 3);
    }
}

#[test]
fn test_rwlock_write_lifetime() {
    let data = RwLock::new(String::from("test"));
    let result: Result<std::sync::RwLockWriteGuard<'_, String>> = LockManager::acquire_write(&data, "test_write");
    assert!(result.is_ok());

    if let Ok(mut guard) = result {
        guard.push_str("_modified");
        assert_eq!(*guard, "test_modified");
    }
}

#[test]
fn test_telemetry_sla_threshold_no_temporary_value_drop() {
    use riptide_core::telemetry::SlaMonitor;
    use std::time::Duration;

    let mut monitor = SlaMonitor::new();

    // This should not cause any temporary value dropped errors
    monitor.record_metric("test_operation", Duration::from_millis(100), true);
    let status = monitor.get_status();

    assert!(status.operations.contains_key("test_operation"));
}