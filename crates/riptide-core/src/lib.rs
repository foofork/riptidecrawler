pub mod cache;
pub mod cache_warming;
pub mod cache_warming_integration;
pub mod circuit;
pub mod common;
pub mod component;
pub mod conditional;
pub mod dynamic;
pub mod error;
pub mod events;
pub mod extract;
pub mod fetch;
pub mod gate;
pub mod instance_pool;
pub mod memory_manager;
pub mod pool_health;
pub mod monitoring;
pub mod pdf;
pub mod integrated_cache;
pub mod reliability;
pub mod robots;
// Re-export search functionality from riptide-search crate for backward compatibility
// TODO: Deprecate in Week 8 - use riptide-search directly
pub mod search {
    //! Search provider abstraction - DEPRECATED
    //!
    //! This module re-exports types from the `riptide-search` crate for backward compatibility.
    //!
    //! **DEPRECATION NOTICE**: This re-export will be removed in a future version.
    //! Please migrate to using `riptide-search` crate directly:
    //!
    //! ```rust
    //! // Old (deprecated):
    //! use riptide_core::search::{SearchProvider, SearchBackend};
    //!
    //! // New (recommended):
    //! use riptide_search::{SearchProvider, SearchBackend};
    //! ```

    pub use riptide_search::{
        SearchProvider, SearchHit, SearchBackend, SearchConfig, AdvancedSearchConfig,
        CircuitBreakerConfigOptions, SearchProviderFactory,
        create_search_provider, create_search_provider_from_env,
        SerperProvider, NoneProvider, CircuitBreakerWrapper, CircuitBreakerConfig, CircuitState
    };

    // Re-export the mod structure for backward compatibility
    pub mod providers {
        pub use riptide_search::SerperProvider;
    }

    pub mod circuit_breaker {
        pub use riptide_search::{CircuitBreakerWrapper, CircuitBreakerConfig, CircuitState};
    }

    pub mod none_provider {
        pub use riptide_search::NoneProvider;
    }
}
pub mod security;
// Temporarily disabled due to compilation issues with HtmlDomCrawler
// pub mod spider;
pub mod stealth;
pub mod strategies;
pub mod telemetry;
pub mod types;

#[cfg(feature = "benchmarks")]
pub mod benchmarks;

pub use types::*;
pub use common::{
    validation::{CommonValidator, ValidationConfig, ContentTypeValidator, UrlValidator, SizeValidator, ParameterValidator},
    error_conversions::{IntoCore, WithErrorContext, CoreErrorConverter, ErrorPatterns},
    config_builder::{ConfigBuilder, DefaultConfigBuilder, ConfigValue, ValidationPatterns},
};
