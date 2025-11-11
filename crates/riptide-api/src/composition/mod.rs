//! Composition Root - Dependency Injection Container
//!
//! This module provides the ApplicationContext which serves as the composition root
//! for the application, wiring all dependencies together using the Dependency Injection pattern.
//!
//! # Architecture
//!
//! - **ApplicationContext**: Main DI container holding all wired dependencies
//! - **DiConfig**: Configuration for database connections, Redis, and feature flags
//! - **ApplicationContextBuilder**: Fluent API for test overrides
//!
//! # Example - Production
//!
//! ```rust,ignore
//! use riptide_api::composition::{ApplicationContext, DiConfig};
//!
//! let config = DiConfig::from_env()?;
//! let ctx = ApplicationContext::new(&config).await?;
//!
//! // Use wired dependencies
//! let users = ctx.user_repository.find_all(filter).await?;
//! ```
//!
//! # Example - Testing
//!
//! ```rust,ignore
//! use riptide_api::composition::ApplicationContext;
//!
//! let ctx = ApplicationContext::for_testing();
//!
//! // All dependencies use in-memory implementations
//! ctx.user_repository.save(&user).await?;
//! ```

pub mod builder;
pub mod config;

use anyhow::Result;
use riptide_types::ports::infrastructure::{Clock, Entropy, SystemClock, SystemEntropy};
use riptide_types::ports::{CircuitBreaker, CircuitBreakerConfig};
use riptide_types::{EventBus, IdempotencyStore, Repository, TransactionManager};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};

// Stub implementations for testing (would be in separate crates in production)
mod stubs;

#[cfg(not(feature = "postgres"))]
use stubs::{
    InMemoryEventBus, InMemoryIdempotencyStore, InMemoryRepository, InMemoryTransaction,
    InMemoryTransactionManager,
};

#[cfg(feature = "postgres")]
use stubs::{InMemoryEventBus, InMemoryIdempotencyStore, InMemoryRepository};

// PostgreSQL adapters (when postgres feature is enabled)
#[cfg(feature = "postgres")]
use riptide_persistence::adapters::{
    OutboxEventBus, PostgresRepository, PostgresTransaction, PostgresTransactionManager,
};

pub use builder::ApplicationContextBuilder;
pub use config::DiConfig;

/// Application Context - Composition Root for Dependency Injection
///
/// This struct holds all wired dependencies using trait objects (Arc<dyn Trait>).
/// It serves as the composition root where all dependencies are assembled.
///
/// # Design
///
/// - **Production**: `new()` wires real adapters (PostgreSQL, Redis, etc.)
/// - **Testing**: `for_testing()` wires in-memory fake implementations
/// - **Builder**: `builder()` provides fluent API for test overrides
///
/// # Example
///
/// ```rust,ignore
/// // Production
/// let config = DiConfig::from_env()?;
/// let ctx = ApplicationContext::new(&config).await?;
///
/// // Testing
/// let ctx = ApplicationContext::for_testing();
///
/// // Custom testing
/// let ctx = ApplicationContext::builder()
///     .with_fake_clock()
///     .with_custom_user_repository(custom_repo)
///     .build()?;
/// ```
#[derive(Clone)]
pub struct ApplicationContext {
    // === Core Infrastructure ===
    /// System clock (real or fake for testing)
    pub clock: Arc<dyn Clock>,

    /// Entropy source (real or deterministic for testing)
    pub entropy: Arc<dyn Entropy>,

    // === Persistence Layer ===
    /// Transaction manager for ACID operations
    #[cfg(feature = "postgres")]
    pub transaction_manager: Arc<dyn TransactionManager<Transaction = PostgresTransaction>>,

    #[cfg(not(feature = "postgres"))]
    pub transaction_manager: Arc<dyn TransactionManager<Transaction = InMemoryTransaction>>,

    /// User entity repository
    pub user_repository: Arc<dyn Repository<User>>,

    /// Event entity repository
    pub event_repository: Arc<dyn Repository<Event>>,

    // === Event System ===
    /// Event bus for domain events (outbox pattern in production)
    pub event_bus: Arc<dyn EventBus>,

    // === Caching & Idempotency ===
    /// Idempotency store for duplicate request prevention
    pub idempotency_store: Arc<dyn IdempotencyStore>,

    // === Resilience ===
    /// Circuit breaker for fault tolerance
    pub circuit_breaker: Arc<dyn CircuitBreaker>,

    // === Configuration ===
    /// Feature flags and runtime configuration
    pub config: DiConfig,
}

impl ApplicationContext {
    /// Create production ApplicationContext with real adapters
    ///
    /// This wires all dependencies using production implementations:
    /// - PostgreSQL for persistence
    /// - Redis for caching and idempotency
    /// - Outbox pattern for event publishing
    /// - System clock and entropy
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration loaded from environment or TOML
    ///
    /// # Returns
    ///
    /// Fully wired ApplicationContext ready for use
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Database connection fails
    /// - Redis connection fails
    /// - Configuration validation fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = DiConfig::from_env()?;
    /// let ctx = ApplicationContext::new(&config).await?;
    /// ```
    #[instrument(skip(config))]
    pub async fn new(config: &DiConfig) -> Result<Self> {
        info!("Initializing ApplicationContext");

        // Validate configuration
        config.validate()?;

        // === Wire System Ports ===
        let clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        let entropy = Arc::new(SystemEntropy) as Arc<dyn Entropy>;
        info!("System ports (clock, entropy) initialized");

        // === Wire Persistence Adapters (feature-gated) ===
        #[cfg(feature = "postgres")]
        {
            info!("Wiring PostgreSQL adapters");

            // Create connection pool
            let pool = Arc::new(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(config.database.max_connections)
                    .connect(&config.database.url)
                    .await?,
            );
            info!("PostgreSQL connection pool established");

            // Wire transaction manager
            let transaction_manager = Arc::new(PostgresTransactionManager::new(pool.clone()))
                as Arc<dyn TransactionManager<Transaction = PostgresTransaction>>;
            info!("Transaction manager initialized (PostgreSQL)");

            // Wire repositories
            let user_repository = Arc::new(PostgresRepository::<User>::new(pool.clone(), "users"))
                as Arc<dyn Repository<User>>;
            let event_repository =
                Arc::new(PostgresRepository::<Event>::new(pool.clone(), "events"))
                    as Arc<dyn Repository<Event>>;
            info!("Repositories initialized (PostgreSQL)");

            // Wire event bus with outbox pattern
            let event_bus = Arc::new(OutboxEventBus::new(pool.clone())) as Arc<dyn EventBus>;
            info!("Event bus initialized (Outbox pattern)");

            // Wire idempotency store (still in-memory for now)
            let idempotency_store =
                Arc::new(InMemoryIdempotencyStore::new()) as Arc<dyn IdempotencyStore>;
            info!("Idempotency store initialized (in-memory)");

            // Wire circuit breaker
            let circuit_breaker = {
                use riptide_cache::adapters::StandardCircuitBreakerAdapter;
                StandardCircuitBreakerAdapter::new(CircuitBreakerConfig::default())
                    as Arc<dyn CircuitBreaker>
            };
            info!("Circuit breaker initialized (standard)");

            info!("ApplicationContext initialization complete (PostgreSQL mode)");

            return Ok(Self {
                clock,
                entropy,
                transaction_manager,
                user_repository,
                event_repository,
                event_bus,
                idempotency_store,
                circuit_breaker,
                config: config.clone(),
            });
        }

        #[cfg(not(feature = "postgres"))]
        {
            info!("Using in-memory implementations (postgres feature not enabled)");

            // === Wire Transaction Manager ===
            let transaction_manager = Arc::new(InMemoryTransactionManager::new())
                as Arc<dyn TransactionManager<Transaction = InMemoryTransaction>>;
            info!("Transaction manager initialized (in-memory)");

            // === Wire Repositories ===
            let user_repository =
                Arc::new(InMemoryRepository::<User>::new()) as Arc<dyn Repository<User>>;
            let event_repository =
                Arc::new(InMemoryRepository::<Event>::new()) as Arc<dyn Repository<Event>>;
            info!("Repositories initialized (in-memory)");

            // === Wire Event Bus ===
            let event_bus = Arc::new(InMemoryEventBus::new()) as Arc<dyn EventBus>;
            info!("Event bus initialized (in-memory)");

            // === Wire Idempotency Store ===
            let idempotency_store =
                Arc::new(InMemoryIdempotencyStore::new()) as Arc<dyn IdempotencyStore>;
            info!("Idempotency store initialized (in-memory)");

            // === Wire Circuit Breaker ===
            let circuit_breaker = {
                use riptide_cache::adapters::StandardCircuitBreakerAdapter;
                StandardCircuitBreakerAdapter::new(CircuitBreakerConfig::default())
                    as Arc<dyn CircuitBreaker>
            };
            info!("Circuit breaker initialized (standard)");

            info!("ApplicationContext initialization complete (in-memory mode)");

            Ok(Self {
                clock,
                entropy,
                transaction_manager,
                user_repository,
                event_repository,
                event_bus,
                idempotency_store,
                circuit_breaker,
                config: config.clone(),
            })
        }
    }

    /// Create testing ApplicationContext with in-memory implementations
    ///
    /// This provides a lightweight context for unit/integration testing:
    /// - In-memory repositories (HashMap-based)
    /// - In-memory event bus
    /// - In-memory idempotency store
    /// - Fake clock (controllable time)
    /// - Deterministic entropy (reproducible randomness)
    ///
    /// # Returns
    ///
    /// ApplicationContext with all in-memory implementations
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// #[tokio::test]
    /// async fn test_user_service() {
    ///     let ctx = ApplicationContext::for_testing();
    ///
    ///     let service = UserService::new(ctx);
    ///     let user = service.create_user("test@example.com").await?;
    ///
    ///     assert!(user.id.starts_with("deterministic-"));
    /// }
    /// ```
    pub fn for_testing() -> Self {
        info!("Creating ApplicationContext (testing mode - in-memory)");

        // Without postgres feature, use the builder
        #[cfg(not(feature = "postgres"))]
        {
            Self::builder().build_for_testing()
        }

        // With postgres feature, manually construct with in-memory implementations
        // Note: We use a type-erased transaction manager to avoid the type mismatch
        #[cfg(feature = "postgres")]
        {
            use riptide_types::ports::infrastructure::{DeterministicEntropy, FakeClock};

            let clock = Arc::new(FakeClock::at_epoch()) as Arc<dyn Clock>;
            let entropy = Arc::new(DeterministicEntropy::new(42)) as Arc<dyn Entropy>;

            // For testing with postgres feature, we create a stub that satisfies the type requirements
            // We can't use InMemoryTransactionManager because it returns InMemoryTransaction
            // Instead, we'll create a minimal stub inline
            struct StubTransactionManager;

            #[async_trait::async_trait]
            impl TransactionManager for StubTransactionManager {
                type Transaction = PostgresTransaction;

                async fn begin(&self) -> riptide_types::Result<Self::Transaction> {
                    unimplemented!(
                        "StubTransactionManager is for testing only - don't call begin()"
                    )
                }

                async fn commit(&self, _tx: Self::Transaction) -> riptide_types::Result<()> {
                    unimplemented!(
                        "StubTransactionManager is for testing only - don't call commit()"
                    )
                }

                async fn rollback(&self, _tx: Self::Transaction) -> riptide_types::Result<()> {
                    unimplemented!(
                        "StubTransactionManager is for testing only - don't call rollback()"
                    )
                }
            }

            let transaction_manager = Arc::new(StubTransactionManager)
                as Arc<dyn TransactionManager<Transaction = PostgresTransaction>>;

            let user_repository =
                Arc::new(InMemoryRepository::<User>::new()) as Arc<dyn Repository<User>>;
            let event_repository =
                Arc::new(InMemoryRepository::<Event>::new()) as Arc<dyn Repository<Event>>;

            let event_bus = Arc::new(InMemoryEventBus::new()) as Arc<dyn EventBus>;

            let idempotency_store =
                Arc::new(InMemoryIdempotencyStore::new()) as Arc<dyn IdempotencyStore>;

            let circuit_breaker = {
                use riptide_cache::adapters::StandardCircuitBreakerAdapter;
                StandardCircuitBreakerAdapter::new(CircuitBreakerConfig::default())
                    as Arc<dyn CircuitBreaker>
            };

            let config = DiConfig::default();

            Self {
                clock,
                entropy,
                transaction_manager,
                user_repository,
                event_repository,
                event_bus,
                idempotency_store,
                circuit_breaker,
                config,
            }
        }
    }

    /// Get builder for custom ApplicationContext configuration
    ///
    /// Useful for tests that need specific dependency overrides.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = ApplicationContext::builder()
    ///     .with_fake_clock()
    ///     .with_deterministic_entropy(42)
    ///     .with_custom_user_repository(mock_repo)
    ///     .build_for_testing();
    /// ```
    pub fn builder() -> ApplicationContextBuilder {
        ApplicationContextBuilder::new()
    }

    /// Check if running in testing mode
    pub fn is_testing(&self) -> bool {
        self.config.is_testing
    }

    /// Get feature flag status
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "browser" => self.config.features.enable_browser,
            "pdf" => self.config.features.enable_pdf,
            "search" => self.config.features.enable_search,
            _ => false,
        }
    }
}

// ============================================================================
// Placeholder Domain Types
// ============================================================================
// These would normally be in riptide-types or domain modules
// For now, we define them here to make the composition compile

/// User entity (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Event entity (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_testing_creates_context() {
        let ctx = ApplicationContext::for_testing();

        // Verify testing mode
        assert!(ctx.is_testing());

        // Verify clock is controllable
        let timestamp1 = ctx.clock.timestamp();
        let timestamp2 = ctx.clock.timestamp();
        assert_eq!(timestamp1, timestamp2); // Fake clock doesn't advance

        // Verify entropy is deterministic
        let id1 = ctx.entropy.random_id();
        let id2 = ctx.entropy.random_id();
        assert_ne!(id1, id2); // Different calls produce different IDs
        assert!(id1.starts_with("deterministic-"));
    }

    #[test]
    fn test_is_feature_enabled() {
        let ctx = ApplicationContext::for_testing();

        // Default test config has features disabled
        assert!(!ctx.is_feature_enabled("browser"));
        assert!(!ctx.is_feature_enabled("pdf"));
        assert!(!ctx.is_feature_enabled("search"));
        assert!(!ctx.is_feature_enabled("unknown"));
    }

    #[test]
    #[cfg(not(feature = "postgres"))]
    fn test_builder_pattern() {
        let custom_clock = Arc::new(riptide_types::FakeClock::at_epoch()) as Arc<dyn Clock>;

        let ctx = ApplicationContext::builder()
            .with_clock(custom_clock)
            .build_for_testing();

        assert_eq!(ctx.clock.timestamp(), 0);
    }
}
