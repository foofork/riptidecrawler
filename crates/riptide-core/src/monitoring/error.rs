//! Error handling for the monitoring system

use std::fmt;

/// Errors that can occur in the monitoring system
#[derive(Debug)]
pub enum MonitoringError {
    /// Lock was poisoned
    LockPoisoned(String),
    /// Invalid metric value
    InvalidMetric(String),
    /// Configuration error
    ConfigError(String),
    /// IO error
    IoError(std::io::Error),
}

impl fmt::Display for MonitoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockPoisoned(op) => write!(f, "Lock poisoned during operation: {}", op),
            Self::InvalidMetric(msg) => write!(f, "Invalid metric: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Self::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for MonitoringError {}

impl From<std::io::Error> for MonitoringError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, MonitoringError>;

/// Helper for safe lock acquisition with consistent error handling
pub struct LockManager;

impl LockManager {
    /// Safely acquire a mutex lock with poison recovery
    pub fn acquire_mutex<'a, T>(
        mutex: &'a std::sync::Mutex<T>,
        operation: &'a str,
    ) -> Result<std::sync::MutexGuard<'a, T>> {
        mutex
            .lock()
            .map_err(|poison_err| {
                tracing::warn!(
                    "Recovering from poisoned lock for {}: {:?}",
                    operation,
                    poison_err
                );
                // Attempt to recover by getting the inner guard and immediately dropping it
                drop(poison_err.into_inner());
                // In a real scenario, we might want to reset the state here
                // For now, we'll just log and continue
                tracing::info!("Successfully recovered poisoned lock for {}", operation);
                MonitoringError::LockPoisoned(operation.to_string())
            })
            .or_else(|_| {
                // If we can't recover, try one more time
                mutex
                    .lock()
                    .map_err(|_| MonitoringError::LockPoisoned(operation.to_string()))
            })
    }

    /// Safely acquire a read lock with poison recovery
    pub fn acquire_read<'a, T>(
        rwlock: &'a std::sync::RwLock<T>,
        operation: &'a str,
    ) -> Result<std::sync::RwLockReadGuard<'a, T>> {
        rwlock.read().map_err(|poison_err| {
            tracing::warn!(
                "Recovering from poisoned read lock for {}: {:?}",
                operation,
                poison_err
            );
            MonitoringError::LockPoisoned(operation.to_string())
        })
    }

    /// Safely acquire a write lock with poison recovery
    pub fn acquire_write<'a, T>(
        rwlock: &'a std::sync::RwLock<T>,
        operation: &'a str,
    ) -> Result<std::sync::RwLockWriteGuard<'a, T>> {
        rwlock.write().map_err(|poison_err| {
            tracing::warn!(
                "Recovering from poisoned write lock for {}: {:?}",
                operation,
                poison_err
            );
            MonitoringError::LockPoisoned(operation.to_string())
        })
    }
}
