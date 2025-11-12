# EngineFacade Adapter Implementation

**Adapter**: EngineFacadeAdapter
**Implements**: `EngineSelection` trait
**Wraps**: `EngineFacade`
**Location**: `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/engine_adapter.rs`

---

## Implementation

```rust
//! EngineFacade Adapter - Implements EngineSelection trait

use crate::facades::{EngineFacade, EngineSelectionCriteria};
use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::{
    ContentAnalysis, EngineChoice, EngineInfo, EngineSelection, EngineSelectionConfig,
    EngineSelectionFlags, EngineSelectionRequest, EngineSelectionStats, EngineType,
};
use std::sync::Arc;

pub struct EngineFacadeAdapter {
    facade: Arc<EngineFacade>,
}

impl EngineFacadeAdapter {
    pub fn new(facade: EngineFacade) -> Arc<Self> {
        Arc::new(Self {
            facade: Arc::new(facade),
        })
    }

    pub fn from_arc(facade: Arc<EngineFacade>) -> Arc<Self> {
        Arc::new(Self { facade })
    }
}

#[async_trait]
impl EngineSelection for EngineFacadeAdapter {
    async fn select_engine(&self, request: EngineSelectionRequest) -> RiptideResult<EngineChoice> {
        // Convert port types to facade types
        let criteria = EngineSelectionCriteria {
            html: request.html,
            url: request.url,
            flags: convert_flags(request.flags),
        };

        let config = self.facade.select_engine(criteria).await?;

        // Convert facade types to port types
        Ok(EngineChoice {
            engine: convert_engine_type(config.engine),
            confidence: config.confidence / 100.0, // Convert 0-100 to 0.0-1.0
            reasons: config.reasons,
            analysis: ContentAnalysis {
                text_density: 0.5, // Would need to extract from facade
                has_javascript: true,
                has_spa_framework: false,
                has_dynamic_content: false,
                has_placeholders: false,
                html_size: criteria.html.len(),
                script_count: 0,
                interactive_elements: 0,
            },
            flags: request.flags,
        })
    }

    async fn available_engines(&self) -> Vec<EngineInfo> {
        self.facade
            .get_capabilities()
            .await
            .into_iter()
            .map(|cap| EngineInfo {
                engine_type: convert_engine_type(cap.engine),
                name: cap.name,
                description: cap.description,
                features: cap.features,
                performance: convert_performance(cap.performance),
                cost_profile: convert_cost(cap.cost),
            })
            .collect()
    }

    async fn validate_compatibility(&self, _url: &str, _engine: EngineType) -> bool {
        true // EngineFacade doesn't expose this
    }

    async fn selection_stats(&self) -> EngineSelectionStats {
        if let Ok(stats) = self.facade.get_stats().await {
            convert_stats(stats)
        } else {
            EngineSelectionStats::default()
        }
    }

    async fn configure(&self, config: EngineSelectionConfig) -> RiptideResult<()> {
        self.facade
            .enable_probe_first(config.probe_first_enabled)
            .await
    }
}

// Helper conversion functions
fn convert_flags(flags: EngineSelectionFlags) -> riptide_reliability::engine_selection::EngineSelectionFlags {
    riptide_reliability::engine_selection::EngineSelectionFlags {
        use_visible_text_density: flags.use_visible_text_density,
        detect_placeholders: flags.detect_placeholders,
        probe_first_spa: flags.probe_first_spa,
    }
}

fn convert_engine_type(engine: riptide_reliability::engine_selection::Engine) -> EngineType {
    match engine {
        riptide_reliability::engine_selection::Engine::Browser => EngineType::Browser,
        riptide_reliability::engine_selection::Engine::Scraper => EngineType::Scraper,
    }
}
```

---

## Usage in ApplicationContext

```rust
use riptide_facade::adapters::EngineFacadeAdapter;
use riptide_types::ports::EngineSelection;

// In ApplicationContext::new():
let engine_facade = EngineFacade::new(cache_storage);
let engine_selector: Arc<dyn EngineSelection> = EngineFacadeAdapter::new(engine_facade);
```

---

**Status**: ✅ Design Complete
