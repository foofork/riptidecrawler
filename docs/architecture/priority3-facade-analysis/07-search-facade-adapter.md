# SearchFacade Adapter Implementation

**Adapter**: SearchFacadeAdapter
**Implements**: `SearchProvider` trait
**Wraps**: `SearchFacade`
**Location**: `/workspaces/riptidecrawler/crates/riptide-facade/src/adapters/search_adapter.rs`

---

## Implementation

```rust
//! SearchFacade Adapter - Implements SearchProvider trait

use crate::facades::SearchFacade;
use async_trait::async_trait;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::{
    DocumentId, SearchCapabilities, SearchDocument, SearchProvider, SearchQuery, SearchResults,
};
use std::sync::Arc;

pub struct SearchFacadeAdapter {
    facade: Arc<SearchFacade>,
}

impl SearchFacadeAdapter {
    pub fn new(facade: SearchFacade) -> Arc<Self> {
        Arc::new(Self {
            facade: Arc::new(facade),
        })
    }

    pub fn from_arc(facade: Arc<SearchFacade>) -> Arc<Self> {
        Arc::new(Self { facade })
    }
}

#[async_trait]
impl SearchProvider for SearchFacadeAdapter {
    async fn search(&self, query: SearchQuery) -> RiptideResult<SearchResults> {
        let start = std::time::Instant::now();

        // Convert SearchQuery to facade method call
        let hits = if let Some(country) = &query.country {
            self.facade
                .search_with_options(
                    &query.query,
                    query.limit,
                    country,
                    query.locale.as_deref().unwrap_or("en"),
                )
                .await?
        } else {
            self.facade.search(&query.query).await?
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(SearchResults {
            hits,
            total_results: hits.len() as u64,
            query: query.query,
            search_duration_ms: duration_ms,
            metadata: Default::default(),
        })
    }

    async fn index_document(&self, _doc: SearchDocument) -> RiptideResult<DocumentId> {
        Err(riptide_types::error::RiptideError::unsupported(
            "SearchFacade does not support document indexing",
        ))
    }

    async fn delete_document(&self, _id: DocumentId) -> RiptideResult<()> {
        Err(riptide_types::error::RiptideError::unsupported(
            "SearchFacade does not support document deletion",
        ))
    }

    fn capabilities(&self) -> SearchCapabilities {
        SearchCapabilities {
            supports_indexing: false,
            supports_deletion: false,
            supports_image_search: false,
            supports_news_search: true,
            supports_time_range: false,
            supports_custom_params: false,
        }
    }

    fn provider_name(&self) -> &str {
        "search_facade"
    }

    async fn is_available(&self) -> bool {
        true
    }
}
```

---

## Usage in ApplicationContext

```rust
use riptide_facade::adapters::SearchFacadeAdapter;
use riptide_types::ports::SearchProvider;

// In ApplicationContext::new():
#[cfg(feature = "search")]
let search_provider: Option<Arc<dyn SearchProvider>> = if let Ok(facade) = SearchFacade::new(...) {
    Some(SearchFacadeAdapter::new(facade))
} else {
    None
};
```

---

**Status**: ✅ Design Complete
