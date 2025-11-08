//! PostgreSQL implementation of the Repository port
//!
//! This adapter provides:
//! - Generic repository pattern for domain entities
//! - Anti-corruption layer (SQL ↔ Domain types)
//! - Connection pooling via sqlx::PgPool
//! - Proper error handling with RiptideError conversion
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_persistence::adapters::PostgresRepository;
//! use sqlx::PgPool;
//!
//! let pool = PgPool::connect(&database_url).await?;
//! let repo: PostgresRepository<User> = PostgresRepository::new(pool, "users");
//!
//! // Use repository
//! let user = repo.find_by_id("user-123").await?;
//! repo.save(&user).await?;
//! ```

use async_trait::async_trait;
use riptide_types::{Repository, RepositoryFilter, Result as RiptideResult, RiptideError};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, error, instrument};

/// PostgreSQL implementation of the Repository port
///
/// Generic repository that stores entities as JSONB in PostgreSQL.
/// The anti-corruption layer handles conversion between SQL and domain types.
///
/// # Type Parameters
///
/// * `T` - Domain entity type (must be Serialize + DeserializeOwned)
///
/// # Table Schema
///
/// The adapter expects a table with the following structure:
/// ```sql
/// CREATE TABLE table_name (
///     id TEXT PRIMARY KEY,
///     data JSONB NOT NULL,
///     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
///     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
/// );
/// CREATE INDEX idx_table_name_data ON table_name USING gin(data);
/// ```
pub struct PostgresRepository<T> {
    /// PostgreSQL connection pool
    pool: Arc<PgPool>,

    /// Table name for this entity type
    table_name: String,

    /// Phantom data for generic type
    _phantom: PhantomData<T>,
}

impl<T> PostgresRepository<T> {
    /// Create new PostgreSQL repository
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `table_name` - Name of the table to store entities
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let repo = PostgresRepository::<User>::new(pool, "users");
    /// ```
    pub fn new(pool: Arc<PgPool>, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }

    /// Get table name for this repository
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Build WHERE clause from filter fields
    fn build_where_clause(&self, filter: &RepositoryFilter) -> (String, Vec<serde_json::Value>) {
        if filter.fields.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut conditions = Vec::new();
        let mut values = Vec::new();
        let mut param_count = 1;

        for (field, value) in &filter.fields {
            conditions.push(format!("data->>'{}' = ${}", field, param_count));
            values.push(value.clone());
            param_count += 1;
        }

        (format!("WHERE {}", conditions.join(" AND ")), values)
    }

    /// Build ORDER BY clause from filter sort
    fn build_order_clause(&self, filter: &RepositoryFilter) -> String {
        if filter.sort.is_empty() {
            return String::from("ORDER BY created_at DESC");
        }

        let order_parts: Vec<String> = filter
            .sort
            .iter()
            .map(|(field, ascending)| {
                let direction = if *ascending { "ASC" } else { "DESC" };
                format!("data->>'{}' {}", field, direction)
            })
            .collect();

        format!("ORDER BY {}", order_parts.join(", "))
    }

    /// Build LIMIT/OFFSET clause from filter pagination
    fn build_pagination_clause(&self, filter: &RepositoryFilter) -> String {
        let mut parts = Vec::new();

        if let Some(limit) = filter.limit {
            parts.push(format!("LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            parts.push(format!("OFFSET {}", offset));
        }

        parts.join(" ")
    }
}

#[async_trait]
impl<T> Repository<T> for PostgresRepository<T>
where
    T: Send + Sync + Serialize + DeserializeOwned,
{
    #[instrument(skip(self), fields(table = %self.table_name, id = %id))]
    async fn find_by_id(&self, id: &str) -> RiptideResult<Option<T>> {
        debug!("Finding entity by ID");

        // Anti-corruption: SQL -> Domain type
        let query = format!("SELECT data FROM {} WHERE id = $1", self.table_name);

        let row: Option<(serde_json::Value,)> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| {
                error!("Failed to query database: {}", e);
                RiptideError::Storage(format!("Failed to find entity: {}", e))
            })?;

        match row {
            Some((data,)) => {
                let entity: T = serde_json::from_value(data).map_err(|e| {
                    error!("Failed to deserialize entity: {}", e);
                    RiptideError::Custom(format!("Failed to deserialize entity: {}", e))
                })?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self, filter), fields(table = %self.table_name))]
    async fn find_all(&self, filter: RepositoryFilter) -> RiptideResult<Vec<T>> {
        debug!("Finding entities with filter");

        // Build dynamic query
        let (where_clause, where_values) = self.build_where_clause(&filter);
        let order_clause = self.build_order_clause(&filter);
        let pagination_clause = self.build_pagination_clause(&filter);

        let query = format!(
            "SELECT data FROM {} {} {} {}",
            self.table_name, where_clause, order_clause, pagination_clause
        );

        // Execute query with bound parameters
        let mut query_builder = sqlx::query_as::<_, (serde_json::Value,)>(&query);
        for value in where_values {
            query_builder = query_builder.bind(value);
        }

        let rows = query_builder.fetch_all(&*self.pool).await.map_err(|e| {
            error!("Failed to query database: {}", e);
            RiptideError::Storage(format!("Failed to find entities: {}", e))
        })?;

        // Anti-corruption: SQL -> Domain types
        let entities: Result<Vec<T>, _> = rows
            .into_iter()
            .map(|(data,)| serde_json::from_value(data))
            .collect();

        entities.map_err(|e| {
            error!("Failed to deserialize entities: {}", e);
            RiptideError::Custom(format!("Failed to deserialize entities: {}", e))
        })
    }

    #[instrument(skip(self, entity), fields(table = %self.table_name))]
    async fn save(&self, entity: &T) -> RiptideResult<()> {
        debug!("Saving entity");

        // Anti-corruption: Domain type -> SQL
        let data = serde_json::to_value(entity).map_err(|e| {
            error!("Failed to serialize entity: {}", e);
            RiptideError::Custom(format!("Failed to serialize entity: {}", e))
        })?;

        // Extract ID from entity (assumes entity has "id" field)
        let id = data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RiptideError::Custom("Entity must have an 'id' field".to_string()))?
            .to_string();

        // Upsert (INSERT ... ON CONFLICT UPDATE)
        let query = format!(
            "INSERT INTO {} (id, data, created_at, updated_at)
             VALUES ($1, $2, NOW(), NOW())
             ON CONFLICT (id)
             DO UPDATE SET data = EXCLUDED.data, updated_at = NOW()",
            self.table_name
        );

        sqlx::query(&query)
            .bind(&id)
            .bind(&data)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!("Failed to save entity: {}", e);
                RiptideError::Storage(format!("Failed to save entity: {}", e))
            })?;

        debug!("Entity saved successfully");
        Ok(())
    }

    #[instrument(skip(self), fields(table = %self.table_name, id = %id))]
    async fn delete(&self, id: &str) -> RiptideResult<()> {
        debug!("Deleting entity");

        let query = format!("DELETE FROM {} WHERE id = $1", self.table_name);

        sqlx::query(&query)
            .bind(id)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!("Failed to delete entity: {}", e);
                RiptideError::Storage(format!("Failed to delete entity: {}", e))
            })?;

        debug!("Entity deleted successfully");
        Ok(())
    }

    #[instrument(skip(self, filter), fields(table = %self.table_name))]
    async fn count(&self, filter: RepositoryFilter) -> RiptideResult<usize> {
        debug!("Counting entities");

        let (where_clause, where_values) = self.build_where_clause(&filter);
        let query = format!("SELECT COUNT(*) FROM {} {}", self.table_name, where_clause);

        let mut query_builder = sqlx::query_as::<_, (i64,)>(&query);
        for value in where_values {
            query_builder = query_builder.bind(value);
        }

        let (count,) = query_builder.fetch_one(&*self.pool).await.map_err(|e| {
            error!("Failed to count entities: {}", e);
            RiptideError::Storage(format!("Failed to count entities: {}", e))
        })?;

        Ok(count as usize)
    }

    #[instrument(skip(self), fields(table = %self.table_name, id = %id))]
    async fn exists(&self, id: &str) -> RiptideResult<bool> {
        debug!("Checking if entity exists");

        let query = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE id = $1)",
            self.table_name
        );

        let (exists,): (bool,) = sqlx::query_as(&query)
            .bind(id)
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| {
                error!("Failed to check existence: {}", e);
                RiptideError::Storage(format!("Failed to check existence: {}", e))
            })?;

        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEntity {
        id: String,
        name: String,
        value: i32,
    }

    #[test]
    fn test_build_where_clause() {
        let repo = PostgresRepository::<TestEntity>::new(
            Arc::new(PgPool::connect_lazy("postgres://localhost").unwrap()),
            "test_table",
        );

        let filter = RepositoryFilter::new()
            .with_field("status", serde_json::json!("active"))
            .with_field("type", serde_json::json!("user"));

        let (clause, values) = repo.build_where_clause(&filter);

        assert!(clause.contains("WHERE"));
        assert!(clause.contains("data->>'status'"));
        assert!(clause.contains("data->>'type'"));
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_build_order_clause() {
        let repo = PostgresRepository::<TestEntity>::new(
            Arc::new(PgPool::connect_lazy("postgres://localhost").unwrap()),
            "test_table",
        );

        let filter = RepositoryFilter::new()
            .with_sort("created_at", false)
            .with_sort("name", true);

        let clause = repo.build_order_clause(&filter);

        assert!(clause.contains("ORDER BY"));
        assert!(clause.contains("DESC"));
        assert!(clause.contains("ASC"));
    }

    #[test]
    fn test_build_pagination_clause() {
        let repo = PostgresRepository::<TestEntity>::new(
            Arc::new(PgPool::connect_lazy("postgres://localhost").unwrap()),
            "test_table",
        );

        let filter = RepositoryFilter::new().with_limit(20).with_offset(10);

        let clause = repo.build_pagination_clause(&filter);

        assert!(clause.contains("LIMIT 20"));
        assert!(clause.contains("OFFSET 10"));
    }
}
