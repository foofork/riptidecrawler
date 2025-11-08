//! Infrastructure adapters implementing port traits
//!
//! This module contains concrete implementations of port traits defined in riptide-types.
//! Adapters provide the anti-corruption layer between domain logic and infrastructure.
//!
//! # Available Adapters
//!
//! - `postgres_repository`: PostgreSQL implementation of `Repository<T>`
//! - `postgres_transaction`: PostgreSQL transaction management
//! - `outbox_event_bus`: Transactional Outbox pattern for event publishing
//! - `prometheus_metrics`: Prometheus metrics collector

#[cfg(feature = "postgres")]
pub mod postgres_repository;

#[cfg(feature = "postgres")]
pub mod postgres_transaction;

#[cfg(feature = "postgres")]
pub mod outbox_event_bus;

#[cfg(feature = "postgres")]
pub mod postgres_session_storage;

pub mod prometheus_metrics;

// Re-export adapters when features are enabled
#[cfg(feature = "postgres")]
pub use postgres_repository::PostgresRepository;

#[cfg(feature = "postgres")]
pub use postgres_transaction::{PostgresTransaction, PostgresTransactionManager};

#[cfg(feature = "postgres")]
pub use outbox_event_bus::{OutboxEventBus, OutboxPublisher};

#[cfg(feature = "postgres")]
pub use postgres_session_storage::PostgresSessionStorage;

pub use prometheus_metrics::PrometheusMetrics;
