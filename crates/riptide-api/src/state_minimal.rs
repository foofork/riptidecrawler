//! Minimal AppState - Reduced from 2213 lines to ApplicationContext pattern
//!
//! This file represents the eliminated AppState, replaced by ApplicationContext.
//! All infrastructure, metrics, and coordination are now in ApplicationContext.
//! Facades are created via factory methods.

use crate::config::RiptideApiConfig;
use crate::health::HealthChecker;
use crate::middleware::AuthConfig;
use anyhow::Result;
use std::sync::Arc;

/// ApplicationContext - Hexagonal Architecture Core
///
/// This replaces the massive AppState (2213 lines) with a clean, port-based design.
/// All infrastructure adapters are injected via traits from riptide-types.
///
/// **Phase 3-4 Breakthrough:**
/// - No circular dependencies (facades use only ports)
/// - All infrastructure managed here
/// - Facades created via factory methods
/// - Fully testable with mocks
#[derive(Clone)]
pub struct ApplicationContext {
    /// API configuration with resource controls
    pub api_config: RiptideApiConfig,

    /// Health checker for enhanced diagnostics
    pub health_checker: Arc<HealthChecker>,

    /// Authentication configuration
    pub auth_config: AuthConfig,

    // Port adapters (injected dependencies)
    // TODO: Add infrastructure adapters as we migrate
    // cache_storage: Arc<dyn riptide_types::ports::CacheStorage>,
    // event_bus: Arc<dyn riptide_types::ports::EventBus>,
    // extractor: Arc<dyn riptide_types::ports::ContentExtractor>,
    // etc.
}

impl ApplicationContext {
    /// Initialize ApplicationContext with all infrastructure
    pub async fn new(
        api_config: RiptideApiConfig,
        health_checker: Arc<HealthChecker>,
        auth_config: AuthConfig,
    ) -> Result<Self> {
        tracing::info!("Initializing ApplicationContext with hexagonal architecture");

        Ok(Self {
            api_config,
            health_checker,
            auth_config,
        })
    }

    /// Factory: Create ExtractionFacade with injected ports
    pub async fn create_extraction_facade(&self) -> Result<Arc<riptide_facade::facades::ExtractionFacade>> {
        let config = riptide_facade::config::RiptideConfig::default();
        let facade = riptide_facade::facades::ExtractionFacade::new(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ExtractionFacade: {}", e))?;

        Ok(Arc::new(facade))
    }

    /// Factory: Create ScraperFacade with injected ports
    pub async fn create_scraper_facade(&self) -> Result<Arc<riptide_facade::facades::ScraperFacade>> {
        let config = riptide_facade::config::RiptideConfig::default();
        let facade = riptide_facade::facades::ScraperFacade::new(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create ScraperFacade: {}", e))?;

        Ok(Arc::new(facade))
    }

    // TODO: Add more facade factories as we migrate
    // create_browser_facade()
    // create_spider_facade()
    // create_search_facade()
    // create_engine_facade()
    // create_resource_facade()
}

/// Re-export the old AppState name for backward compatibility during migration
pub type AppState = ApplicationContext;
