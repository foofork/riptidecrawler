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

#[cfg(feature = "postgres")]
pub mod postgres_repository;

#[cfg(feature = "postgres")]
pub mod postgres_transaction;

#[cfg(feature = "postgres")]
pub mod outbox_event_bus;

// Re-export adapters when features are enabled
#[cfg(feature = "postgres")]
pub use postgres_repository::PostgresRepository;

#[cfg(feature = "postgres")]
pub use postgres_transaction::{PostgresTransaction, PostgresTransactionManager};

#[cfg(feature = "postgres")]
pub use outbox_event_bus::{OutboxEventBus, OutboxPublisher};
