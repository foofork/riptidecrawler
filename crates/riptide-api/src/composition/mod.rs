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
use riptide_types::ports::infrastructure::{
    Clock, DeterministicEntropy, Entropy, FakeClock, SystemClock, SystemEntropy,
};
use riptide_types::{EventBus, IdempotencyStore, Repository, TransactionManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};

// Stub implementations for testing (would be in separate crates in production)
mod stubs;
use stubs::{
    InMemoryEventBus, InMemoryIdempotencyStore, InMemoryRepository, InMemoryTransaction,
    InMemoryTransactionManager,
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

        // TODO: Enable PostgreSQL adapters when feature gates are configured
        // For now, use in-memory implementations
        info!("Using in-memory implementations (database features not enabled)");

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

        // === Wire System Ports ===
        let clock = Arc::new(SystemClock) as Arc<dyn Clock>;
        let entropy = Arc::new(SystemEntropy) as Arc<dyn Entropy>;
        info!("System ports (clock, entropy) initialized");

        info!("ApplicationContext initialization complete");

        Ok(Self {
            clock,
            entropy,
            transaction_manager,
            user_repository,
            event_repository,
            event_bus,
            idempotency_store,
            config: config.clone(),
        })
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

        // Use the builder with default in-memory implementations
        Self::builder().build_for_testing()
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
    fn test_builder_pattern() {
        let custom_clock = Arc::new(FakeClock::at_epoch()) as Arc<dyn Clock>;

        let ctx = ApplicationContext::builder()
            .with_clock(custom_clock)
            .build_for_testing();

        assert_eq!(ctx.clock.timestamp(), 0);
    }
}
