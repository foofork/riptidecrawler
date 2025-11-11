//! Builder pattern for ApplicationContext
//!
//! This module provides a fluent API for building ApplicationContext
//! with custom dependency overrides, primarily for testing.
//!
//! # Example
//!
//! ```rust,ignore
//! use riptide_api::composition::ApplicationContext;
//!
//! let ctx = ApplicationContext::builder()
//!     .with_fake_clock()
//!     .with_deterministic_entropy(42)
//!     .with_custom_user_repository(mock_repo)
//!     .build_for_testing();
//! ```

use super::{DiConfig, Event, User};

#[cfg(not(feature = "postgres"))]
use super::{stubs::*, ApplicationContext};

#[cfg(not(feature = "postgres"))]
use anyhow::Result;

use riptide_types::ports::infrastructure::{
    Clock, DeterministicEntropy, Entropy, FakeClock, SystemClock, SystemEntropy,
};
use riptide_types::ports::{CircuitBreaker, CircuitBreakerConfig};
use riptide_types::{EventBus, IdempotencyStore, Repository};

#[cfg(not(feature = "postgres"))]
use riptide_types::TransactionManager;

use std::sync::Arc;

/// Builder for ApplicationContext
///
/// Provides fluent API for constructing ApplicationContext with
/// custom dependency overrides. Useful for testing scenarios.
///
/// # Example
///
/// ```rust,ignore
/// let ctx = ApplicationContextBuilder::new()
///     .with_fake_clock()
///     .with_custom_event_bus(mock_bus)
///     .build_for_testing();
/// ```
pub struct ApplicationContextBuilder {
    clock: Option<Arc<dyn Clock>>,
    entropy: Option<Arc<dyn Entropy>>,
    user_repository: Option<Arc<dyn Repository<User>>>,
    event_repository: Option<Arc<dyn Repository<Event>>>,
    event_bus: Option<Arc<dyn EventBus>>,
    idempotency_store: Option<Arc<dyn IdempotencyStore>>,
    circuit_breaker: Option<Arc<dyn CircuitBreaker>>,
    config: DiConfig,
}

impl ApplicationContextBuilder {
    /// Create new builder with defaults
    pub fn new() -> Self {
        Self {
            clock: None,
            entropy: None,
            user_repository: None,
            event_repository: None,
            event_bus: None,
            idempotency_store: None,
            circuit_breaker: None,
            config: DiConfig::for_testing(),
        }
    }

    // ========================================================================
    // Clock Configuration
    // ========================================================================

    /// Set custom clock implementation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let custom_clock = Arc::new(FakeClock::at_epoch());
    /// builder.with_clock(custom_clock);
    /// ```
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Use fake clock (controllable time for testing)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.with_fake_clock();
    /// ```
    pub fn with_fake_clock(self) -> Self {
        self.with_clock(Arc::new(FakeClock::at_epoch()))
    }

    /// Use system clock (real time)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.with_system_clock();
    /// ```
    pub fn with_system_clock(self) -> Self {
        self.with_clock(Arc::new(SystemClock))
    }

    // ========================================================================
    // Entropy Configuration
    // ========================================================================

    /// Set custom entropy implementation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let custom_entropy = Arc::new(DeterministicEntropy::new(42));
    /// builder.with_entropy(custom_entropy);
    /// ```
    pub fn with_entropy(mut self, entropy: Arc<dyn Entropy>) -> Self {
        self.entropy = Some(entropy);
        self
    }

    /// Use deterministic entropy with seed (for reproducible tests)
    ///
    /// # Arguments
    ///
    /// * `seed` - Seed for pseudo-random number generator
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.with_deterministic_entropy(42);
    /// ```
    pub fn with_deterministic_entropy(self, seed: u64) -> Self {
        self.with_entropy(Arc::new(DeterministicEntropy::new(seed)))
    }

    /// Use system entropy (cryptographically secure randomness)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.with_system_entropy();
    /// ```
    pub fn with_system_entropy(self) -> Self {
        self.with_entropy(Arc::new(SystemEntropy))
    }

    // ========================================================================
    // Repository Configuration
    // ========================================================================

    /// Set custom user repository
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mock_repo = Arc::new(MockUserRepository::new());
    /// builder.with_user_repository(mock_repo);
    /// ```
    pub fn with_user_repository(mut self, repo: Arc<dyn Repository<User>>) -> Self {
        self.user_repository = Some(repo);
        self
    }

    /// Set custom event repository
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mock_repo = Arc::new(MockEventRepository::new());
    /// builder.with_event_repository(mock_repo);
    /// ```
    pub fn with_event_repository(mut self, repo: Arc<dyn Repository<Event>>) -> Self {
        self.event_repository = Some(repo);
        self
    }

    // ========================================================================
    // Event Bus Configuration
    // ========================================================================

    /// Set custom event bus
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mock_bus = Arc::new(MockEventBus::new());
    /// builder.with_event_bus(mock_bus);
    /// ```
    pub fn with_event_bus(mut self, bus: Arc<dyn EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    // ========================================================================
    // Idempotency Store Configuration
    // ========================================================================

    /// Set custom idempotency store
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mock_store = Arc::new(MockIdempotencyStore::new());
    /// builder.with_idempotency_store(mock_store);
    /// ```
    pub fn with_idempotency_store(mut self, store: Arc<dyn IdempotencyStore>) -> Self {
        self.idempotency_store = Some(store);
        self
    }

    // ========================================================================
    // Configuration
    // ========================================================================

    /// Set custom configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let custom_config = DiConfig::from_env()?;
    /// builder.with_config(custom_config);
    /// ```
    pub fn with_config(mut self, config: DiConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable specific feature flag
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.with_feature("browser", true);
    /// ```
    pub fn with_feature(mut self, feature: &str, enabled: bool) -> Self {
        match feature {
            "browser" => self.config.features.enable_browser = enabled,
            "pdf" => self.config.features.enable_pdf = enabled,
            "search" => self.config.features.enable_search = enabled,
            _ => {}
        }
        self
    }

    // ========================================================================
    // Build Methods
    // ========================================================================

    /// Build ApplicationContext for testing with in-memory defaults
    ///
    /// Any dependencies not explicitly set will use in-memory implementations.
    ///
    /// Note: This builder is only available without the `postgres` feature.
    /// With `postgres` enabled, use `ApplicationContext::for_testing()` instead.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = builder.build_for_testing();
    /// ```
    #[cfg(not(feature = "postgres"))]
    pub fn build_for_testing(self) -> ApplicationContext {
        // Use provided implementations or default to in-memory
        let clock = self
            .clock
            .unwrap_or_else(|| Arc::new(FakeClock::at_epoch()));

        let entropy = self
            .entropy
            .unwrap_or_else(|| Arc::new(DeterministicEntropy::default_seed()));

        let user_repository = self
            .user_repository
            .unwrap_or_else(|| Arc::new(InMemoryRepository::<User>::new()));

        let event_repository = self
            .event_repository
            .unwrap_or_else(|| Arc::new(InMemoryRepository::<Event>::new()));

        let event_bus = self
            .event_bus
            .unwrap_or_else(|| Arc::new(InMemoryEventBus::default()));

        let idempotency_store = self
            .idempotency_store
            .unwrap_or_else(|| Arc::new(InMemoryIdempotencyStore::new()));

        let circuit_breaker = self.circuit_breaker.unwrap_or_else(|| {
            use riptide_cache::adapters::StandardCircuitBreakerAdapter;
            StandardCircuitBreakerAdapter::new(CircuitBreakerConfig::default())
                as Arc<dyn CircuitBreaker>
        });

        // For testing, we create a stub transaction manager
        let transaction_manager = Arc::new(InMemoryTransactionManager::new())
            as Arc<dyn TransactionManager<Transaction = InMemoryTransaction>>;

        ApplicationContext {
            clock,
            entropy,
            transaction_manager,
            user_repository,
            event_repository,
            event_bus,
            idempotency_store,
            circuit_breaker,
            config: self.config,
        }
    }

    /// Build ApplicationContext with validation
    ///
    /// Validates that all required dependencies are set.
    ///
    /// Note: This builder is only available without the `postgres` feature.
    /// With `postgres` enabled, use `ApplicationContext::for_testing()` instead.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Required dependencies are missing
    /// - Configuration validation fails
    #[cfg(not(feature = "postgres"))]
    pub fn build(self) -> Result<ApplicationContext> {
        // Validate configuration
        self.config.validate()?;

        // Ensure all required dependencies are set
        if self.clock.is_none() {
            anyhow::bail!("Clock implementation required");
        }
        if self.entropy.is_none() {
            anyhow::bail!("Entropy implementation required");
        }
        if self.user_repository.is_none() {
            anyhow::bail!("User repository implementation required");
        }
        if self.event_repository.is_none() {
            anyhow::bail!("Event repository implementation required");
        }
        if self.event_bus.is_none() {
            anyhow::bail!("Event bus implementation required");
        }
        if self.idempotency_store.is_none() {
            anyhow::bail!("Idempotency store implementation required");
        }

        // All dependencies present, build context
        let transaction_manager = Arc::new(InMemoryTransactionManager::new())
            as Arc<dyn TransactionManager<Transaction = InMemoryTransaction>>;

        let circuit_breaker = self.circuit_breaker.unwrap_or_else(|| {
            use riptide_cache::adapters::StandardCircuitBreakerAdapter;
            StandardCircuitBreakerAdapter::new(CircuitBreakerConfig::default())
                as Arc<dyn CircuitBreaker>
        });

        Ok(ApplicationContext {
            clock: self.clock.unwrap(),
            entropy: self.entropy.unwrap(),
            transaction_manager,
            user_repository: self.user_repository.unwrap(),
            event_repository: self.event_repository.unwrap(),
            event_bus: self.event_bus.unwrap(),
            idempotency_store: self.idempotency_store.unwrap(),
            circuit_breaker,
            config: self.config,
        })
    }
}

impl Default for ApplicationContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementations are in the stubs module

#[cfg(all(test, not(feature = "postgres")))]
mod tests {
    use super::*;

    #[test]
    fn test_builder_with_defaults() {
        let ctx = ApplicationContextBuilder::new().build_for_testing();

        assert!(ctx.is_testing());
        assert_eq!(ctx.clock.timestamp(), 0); // Fake clock at epoch
    }

    #[test]
    fn test_builder_with_custom_clock() {
        let custom_clock = Arc::new(FakeClock::at_epoch()) as Arc<dyn Clock>;

        let ctx = ApplicationContextBuilder::new()
            .with_clock(custom_clock)
            .build_for_testing();

        assert_eq!(ctx.clock.timestamp(), 0);
    }

    #[test]
    fn test_builder_with_system_clock() {
        let ctx = ApplicationContextBuilder::new()
            .with_system_clock()
            .build_for_testing();

        // System clock should return current time (> 0)
        assert!(ctx.clock.timestamp() > 0);
    }

    #[test]
    fn test_builder_with_deterministic_entropy() {
        let ctx = ApplicationContextBuilder::new()
            .with_deterministic_entropy(42)
            .build_for_testing();

        let id = ctx.entropy.random_id();
        assert!(id.starts_with("deterministic-"));
    }

    #[test]
    fn test_builder_with_features() {
        let ctx = ApplicationContextBuilder::new()
            .with_feature("browser", true)
            .with_feature("pdf", true)
            .build_for_testing();

        assert!(ctx.is_feature_enabled("browser"));
        assert!(ctx.is_feature_enabled("pdf"));
        assert!(!ctx.is_feature_enabled("search"));
    }

    #[test]
    fn test_builder_validation_fails_without_required_deps() {
        let result = ApplicationContextBuilder::new().build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validation_succeeds_with_all_deps() {
        let result = ApplicationContextBuilder::new()
            .with_fake_clock()
            .with_deterministic_entropy(42)
            .with_user_repository(Arc::new(InMemoryRepository::<User>::new()))
            .with_event_repository(Arc::new(InMemoryRepository::<Event>::new()))
            .with_event_bus(Arc::new(InMemoryEventBus::default()))
            .with_idempotency_store(Arc::new(InMemoryIdempotencyStore::new()))
            .build();

        assert!(result.is_ok());
    }
}
