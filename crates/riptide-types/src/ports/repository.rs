//! Generic repository pattern for domain entities
//!
//! This module provides backend-agnostic persistence interfaces that enable:
//! - Dependency inversion for data access layers
//! - Testing with in-memory implementations
//! - Swapping database backends without changing domain logic
//! - Transaction management with rollback support
//!
//! # Design Goals
//!
//! - **Domain Purity**: No database-specific types in domain layer
//! - **Testability**: Easy mocking and in-memory testing
//! - **Flexibility**: Support multiple backend implementations (PostgreSQL, MongoDB, etc.)
//! - **ACID Compliance**: Transactional semantics where supported
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_types::ports::{Repository, TransactionManager};
//!
//! async fn example(
//!     repo: &dyn Repository<User>,
//!     tx_manager: &dyn TransactionManager,
//! ) -> Result<()> {
//!     // Find entity
//!     if let Some(user) = repo.find_by_id("user-123").await? {
//!         println!("Found user: {}", user.name);
//!     }
//!
//!     // Transaction management
//!     let mut tx = tx_manager.begin().await?;
//!     repo.save(&user).await?;
//!     tx_manager.commit(tx).await?;
//!
//!     Ok(())
//! }
//! ```

use crate::error::Result as RiptideResult;
use async_trait::async_trait;

/// Filter criteria for repository queries
///
/// Extensible filter structure that backends can interpret
/// according to their query capabilities.
#[derive(Debug, Clone, Default)]
pub struct RepositoryFilter {
    /// Field-based equality filters
    pub fields: std::collections::HashMap<String, serde_json::Value>,
    /// Pagination offset
    pub offset: Option<usize>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Sort fields with direction (field_name, ascending)
    pub sort: Vec<(String, bool)>,
}

impl RepositoryFilter {
    /// Create new empty filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Add field equality filter
    pub fn with_field(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.fields.insert(name.into(), value);
        self
    }

    /// Set pagination offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add sort field
    pub fn with_sort(mut self, field: impl Into<String>, ascending: bool) -> Self {
        self.sort.push((field.into(), ascending));
        self
    }
}

/// Generic repository pattern for domain entities
///
/// Implementations must be thread-safe (`Send + Sync`) and support
/// asynchronous operations. The generic parameter `T` represents the
/// domain entity type.
///
/// # Type Constraints
///
/// - `T: Send + Sync` - Entity must be thread-safe
/// - Implementations may require additional constraints (e.g., `Serialize`, `DeserializeOwned`)
#[async_trait]
pub trait Repository<T>: Send + Sync
where
    T: Send + Sync,
{
    /// Retrieve entity by unique identifier
    ///
    /// # Arguments
    ///
    /// * `id` - Unique entity identifier
    ///
    /// # Returns
    ///
    /// * `Ok(Some(entity))` - Entity found
    /// * `Ok(None)` - Entity not found
    /// * `Err(_)` - Storage backend error
    async fn find_by_id(&self, id: &str) -> RiptideResult<Option<T>>;

    /// Query entities matching filter criteria
    ///
    /// # Arguments
    ///
    /// * `filter` - Query filter with pagination and sorting
    ///
    /// # Returns
    ///
    /// * `Ok(entities)` - Vector of matching entities (may be empty)
    /// * `Err(_)` - Storage backend error
    async fn find_all(&self, filter: RepositoryFilter) -> RiptideResult<Vec<T>>;

    /// Persist entity (insert or update)
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Entity persisted successfully
    /// * `Err(_)` - Storage backend error or validation failure
    ///
    /// # Behavior
    ///
    /// Implementations should use entity ID to determine insert vs update.
    /// Idempotent where supported.
    async fn save(&self, entity: &T) -> RiptideResult<()>;

    /// Delete entity by unique identifier
    ///
    /// # Arguments
    ///
    /// * `id` - Unique entity identifier
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Entity deleted (or didn't exist)
    /// * `Err(_)` - Storage backend error
    ///
    /// # Behavior
    ///
    /// Idempotent - deleting non-existent entity is not an error.
    async fn delete(&self, id: &str) -> RiptideResult<()>;

    /// Count entities matching filter criteria
    ///
    /// # Arguments
    ///
    /// * `filter` - Query filter (offset/limit ignored)
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - Number of matching entities
    /// * `Err(_)` - Storage backend error
    async fn count(&self, filter: RepositoryFilter) -> RiptideResult<usize>;

    /// Check if entity exists by ID
    ///
    /// Default implementation uses `find_by_id`.
    /// Backends can override for efficiency.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique entity identifier
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Entity exists
    /// * `Ok(false)` - Entity doesn't exist
    /// * `Err(_)` - Storage backend error
    async fn exists(&self, id: &str) -> RiptideResult<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

/// Transaction management port
///
/// Provides ACID transaction semantics with explicit commit/rollback.
/// Backends that don't support transactions should provide no-op implementations.
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// Associated transaction type
    ///
    /// Allows backends to provide their own transaction implementations.
    type Transaction: Transaction;

    /// Begin new transaction
    ///
    /// # Returns
    ///
    /// * `Ok(transaction)` - Transaction started successfully
    /// * `Err(_)` - Backend error (connection failure, etc.)
    async fn begin(&self) -> RiptideResult<Self::Transaction>;

    /// Commit transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to commit
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transaction committed successfully
    /// * `Err(_)` - Commit failed (constraint violation, etc.)
    async fn commit(&self, tx: Self::Transaction) -> RiptideResult<()>;

    /// Rollback transaction
    ///
    /// # Arguments
    ///
    /// * `tx` - Transaction to rollback
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Transaction rolled back successfully
    /// * `Err(_)` - Rollback failed (already committed, connection lost, etc.)
    async fn rollback(&self, tx: Self::Transaction) -> RiptideResult<()>;
}

/// Transaction handle with scope-based semantics
///
/// Implementations should support:
/// - Unique transaction identifiers
/// - Scope-based operation execution
/// - Automatic rollback on drop (if not committed)
#[async_trait]
pub trait Transaction: Send + Sync {
    /// Get transaction unique identifier
    ///
    /// Useful for logging and debugging.
    fn id(&self) -> &str;

    /// Execute operation within transaction scope
    ///
    /// # Arguments
    ///
    /// * `f` - Closure to execute within transaction
    ///
    /// # Returns
    ///
    /// * `Ok(result)` - Operation completed successfully
    /// * `Err(_)` - Operation failed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// tx.execute(|| {
    ///     repo.save(&entity)?;
    ///     Ok(entity.id)
    /// }).await?;
    /// ```
    async fn execute<F, R>(&mut self, f: F) -> RiptideResult<R>
    where
        F: FnOnce() -> RiptideResult<R> + Send,
        R: Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_filter_builder() {
        let filter = RepositoryFilter::new()
            .with_field("status", serde_json::json!("active"))
            .with_offset(10)
            .with_limit(20)
            .with_sort("created_at", false);

        assert_eq!(filter.offset, Some(10));
        assert_eq!(filter.limit, Some(20));
        assert_eq!(filter.sort.len(), 1);
        assert_eq!(filter.fields.len(), 1);
    }
}
