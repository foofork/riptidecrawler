# WASM Component Model Migration Plan

## Executive Summary

This document outlines the migration strategy from WASI command interface to WASM Component Model for RipTide's content extraction system. The migration will enhance performance, security, and interoperability while enabling proper trek-rs integration.

## Current State Analysis

### Existing Implementation

The current WASM extractor is partially implemented with Component Model support:

**✅ What's Working:**
- Basic WIT interface definition (`extractor.wit`)
- Component Model host-side integration (`component.rs`)
- wasip2 target compilation capability
- Type-safe bindings with `wit-bindgen`

**❌ What's Missing:**
- trek-rs integration (currently commented out)
- Advanced error handling in WIT interface
- Performance optimizations
- Proper guest-side Component Model implementation
- Build pipeline automation

### Technical Debt

1. **Trek-rs Dependency Conflict**: Version mismatch and disabled integration
2. **Fallback Implementation**: Current guest uses simple string manipulation
3. **Limited Error Handling**: WIT interface lacks typed error responses
4. **Missing Performance Features**: No instance pooling or memory reuse

## Component Model Architecture Design

### Enhanced WIT Interface

```wit
// Enhanced extractor.wit with typed interfaces and error handling
package riptide:extractor@0.2.0;

/// Content extraction modes
variant extraction-mode {
    /// Extract article content with readability algorithms
    article,
    /// Extract full content including sidebars and navigation
    full,
    /// Extract only metadata (title, description, etc.)
    metadata,
    /// Custom extraction with provided CSS selectors
    custom(list<string>),
}

/// Structured extraction result with rich metadata
record extracted-content {
    /// Source URL
    url: string,
    /// Extracted title
    title: option<string>,
    /// Author/byline information
    byline: option<string>,
    /// Publication date in ISO format
    published-iso: option<string>,
    /// Content in Markdown format
    markdown: string,
    /// Plain text content
    text: string,
    /// Extracted links
    links: list<string>,
    /// Media URLs (images, videos)
    media: list<string>,
    /// Content language detection
    language: option<string>,
    /// Reading time estimate (minutes)
    reading-time: option<u32>,
    /// Content quality score (0-100)
    quality-score: option<u8>,
}

/// Extraction errors
variant extraction-error {
    /// Invalid HTML input
    invalid-html(string),
    /// Network-related errors
    network-error(string),
    /// Parsing failures
    parse-error(string),
    /// Resource limits exceeded
    resource-limit(string),
    /// Internal processing error
    internal-error(string),
}

/// Main extraction world
world extractor {
    /// Extract content from HTML with enhanced error handling
    export extract: func(
        html: string,
        url: string,
        mode: extraction-mode
    ) -> result<extracted-content, extraction-error>;

    /// Health check for component status
    export health-check: func() -> string;

    /// Get component version and capabilities
    export get-info: func() -> string;
}
```

### Host-Side Integration Architecture

```rust
// Enhanced component.rs with performance optimizations
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::{component::*, Config, Engine, Store};

use crate::types::{ExtractedDoc, ExtractionMode};

/// High-performance Component Model extractor with instance pooling
pub struct CmExtractor {
    engine: Arc<Engine>,
    component: Arc<Component>,
    linker: Arc<Linker<ExtractorState>>,
    instance_pool: Arc<Mutex<Vec<ExtractorInstance>>>,
    config: ExtractorConfig,
}

/// Per-instance state for component model execution
#[derive(Default)]
pub struct ExtractorState {
    pub memory_limit: usize,
    pub timeout_ms: u64,
    pub metrics: ExtractionMetrics,
}

/// Component instance wrapper for pooling
pub struct ExtractorInstance {
    store: Store<ExtractorState>,
    instance: Extractor,
    last_used: std::time::Instant,
}

/// Configuration for extractor behavior
#[derive(Clone)]
pub struct ExtractorConfig {
    pub max_instances: usize,
    pub instance_timeout: std::time::Duration,
    pub memory_limit: usize,
    pub enable_simd: bool,
    pub optimization_level: wasmtime::OptLevel,
}

impl CmExtractor {
    /// Create extractor with performance optimizations
    pub async fn new(wasm_path: &str, config: ExtractorConfig) -> Result<Self> {
        let mut wasmtime_config = Config::new();

        // Enable Component Model features
        wasmtime_config.wasm_component_model(true);
        wasmtime_config.cranelift_opt_level(config.optimization_level);

        // Performance features
        wasmtime_config.wasm_simd(config.enable_simd);
        wasmtime_config.wasm_bulk_memory(true);
        wasmtime_config.wasm_multi_memory(true);
        wasmtime_config.wasm_memory64(true);

        // Security and resource limits
        wasmtime_config.consume_fuel(true);
        wasmtime_config.epoch_interruption(true);

        let engine = Arc::new(Engine::new(&wasmtime_config)?);
        let component = Arc::new(Component::from_file(&engine, wasm_path)?);
        let linker = Arc::new(Self::create_linker(&engine)?);

        Ok(Self {
            engine,
            component,
            linker,
            instance_pool: Arc::new(Mutex::new(Vec::new())),
            config,
        })
    }

    /// Extract content with automatic instance management
    pub async fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        let instance = self.get_instance().await?;
        let result = self.perform_extraction(instance, html, url, mode).await;
        self.return_instance(instance).await;
        result
    }

    /// Get instance from pool or create new one
    async fn get_instance(&self) -> Result<ExtractorInstance> {
        let mut pool = self.instance_pool.lock().await;

        // Try to reuse existing instance
        if let Some(instance) = pool.pop() {
            if instance.last_used.elapsed() < self.config.instance_timeout {
                return Ok(instance);
            }
        }

        // Create new instance if pool is empty or instances are stale
        self.create_instance().await
    }

    /// Create new component instance
    async fn create_instance(&self) -> Result<ExtractorInstance> {
        let mut store = Store::new(&self.engine, ExtractorState::default());

        // Set resource limits
        store.limiter(|state| {
            wasmtime::ResourceLimiter::new()
                .memory_size(state.memory_limit)
                .table_elements(10000)
                .instances(1)
        });

        // Set fuel for execution limits
        store.add_fuel(1_000_000)?;

        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        Ok(ExtractorInstance {
            store,
            instance,
            last_used: std::time::Instant::now(),
        })
    }
}
```

### Guest-Side Trek-rs Integration

```rust
// Enhanced guest implementation with trek-rs
use trek_rs::{Article, Extractor as TrekExtractor};
use serde::Serialize;

// Generate bindings from enhanced WIT file
wit_bindgen::generate!({
    world: "extractor",
    path: "wit",
});

export!(Component);

struct Component;

impl Guest for Component {
    fn extract(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        // Validate inputs
        if html.is_empty() {
            return Err(ExtractionError::InvalidHtml("Empty HTML input".to_string()));
        }

        if url.is_empty() {
            return Err(ExtractionError::InvalidHtml("Empty URL".to_string()));
        }

        // Use trek-rs for high-quality extraction
        match TrekExtractor::new()
            .url(&url)
            .html(&html)
            .extract()
        {
            Ok(article) => {
                let content = convert_trek_to_component_model(article, &url, mode);
                Ok(content)
            }
            Err(e) => {
                Err(ExtractionError::ParseError(format!("Trek extraction failed: {}", e)))
            }
        }
    }

    fn health_check() -> String {
        serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "trek_version": trek_rs::version(),
            "capabilities": ["article", "full", "metadata", "custom"]
        }).to_string()
    }

    fn get_info() -> String {
        serde_json::json!({
            "name": "riptide-extractor-wasm",
            "version": env!("CARGO_PKG_VERSION"),
            "component_model": "0.2.0",
            "features": ["trek-rs", "simd", "advanced-parsing"],
            "supported_modes": ["article", "full", "metadata", "custom"]
        }).to_string()
    }
}

fn convert_trek_to_component_model(
    article: Article,
    url: &str,
    mode: ExtractionMode,
) -> ExtractedContent {
    ExtractedContent {
        url: url.to_string(),
        title: article.title,
        byline: article.byline,
        published_iso: article.published_time.map(|t| t.to_rfc3339()),
        markdown: article.content_markdown.unwrap_or_default(),
        text: article.content_text,
        links: article.links.unwrap_or_default(),
        media: article.images.unwrap_or_default(),
        language: article.language,
        reading_time: article.reading_time,
        quality_score: Some(calculate_quality_score(&article)),
    }
}

fn calculate_quality_score(article: &Article) -> u8 {
    let mut score = 50u8; // Base score

    // Title quality
    if article.title.as_ref().map_or(0, |t| t.len()) > 10 {
        score += 10;
    }

    // Content length
    if article.content_text.len() > 500 {
        score += 15;
    }

    // Has byline/author
    if article.byline.is_some() {
        score += 10;
    }

    // Has publication date
    if article.published_time.is_some() {
        score += 10;
    }

    // Has images
    if article.images.as_ref().map_or(0, |i| i.len()) > 0 {
        score += 5;
    }

    score.min(100)
}
```

## Migration Strategy

### Phase 1: Foundation Setup (Week 1-2)

1. **Dependency Resolution**
   - Pin trek-rs to version 0.2.1
   - Update Cargo.toml configurations
   - Verify Component Model compatibility

2. **Enhanced WIT Interface**
   - Implement new WIT interface with typed errors
   - Add extraction modes and result types
   - Include health check and info endpoints

3. **Build Pipeline**
   - Create `.cargo/config.toml` for wasip2 target
   - Set up automated Component Model builds
   - Configure CI/CD for WASM compilation

### Phase 2: Core Implementation (Week 3-4)

1. **Guest-Side Integration**
   - Integrate trek-rs in WASM component
   - Implement enhanced error handling
   - Add content quality scoring

2. **Host-Side Optimizations**
   - Implement instance pooling
   - Add resource management
   - Configure performance optimizations

3. **Testing Infrastructure**
   - Component Model integration tests
   - Performance benchmarks
   - Error handling validation

### Phase 3: Performance & Production (Week 5-6)

1. **Performance Tuning**
   - SIMD optimizations
   - Memory usage optimization
   - Concurrency improvements

2. **Monitoring & Observability**
   - Metrics collection
   - Performance monitoring
   - Error tracking

3. **Documentation & Examples**
   - API documentation
   - Usage examples
   - Migration guides

## Implementation Details

### Build Configuration

Create `.cargo/config.toml`:
```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "wasmtime run --wasm component-model"

[env]
RUSTFLAGS = "-C target-feature=+simd128 -C opt-level=s"
```

### Cargo.toml Updates

For WASM component:
```toml
[package]
name = "riptide-extractor-wasm"
version = "0.2.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.30"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
trek-rs = "0.2.1"  # Re-enabled with proper version

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"
```

For host crate:
```toml
[dependencies]
wasmtime = { version = "26", features = [
    "cache",
    "component-model",
    "cranelift",
    "pooling-allocator"
] }
wasmtime-wasi = "26"
wit-bindgen = "0.30"
tokio = { version = "1", features = ["sync", "time"] }
```

### Performance Optimizations

1. **Instance Pooling**: Reuse Component instances to avoid initialization overhead
2. **Memory Management**: Pre-allocate memory pools for large documents
3. **SIMD Instructions**: Enable vectorized operations for text processing
4. **Async Operations**: Non-blocking extraction with proper resource limits

### Error Handling Strategy

1. **Typed Errors**: Use WIT variants for structured error responses
2. **Error Recovery**: Implement fallback strategies for parsing failures
3. **Resource Limits**: Prevent memory exhaustion and infinite loops
4. **Timeout Management**: Cancel long-running extractions

## Testing Strategy

### Unit Tests
- Component instantiation
- Basic extraction functionality
- Error handling scenarios
- Resource limit enforcement

### Integration Tests
- End-to-end extraction workflows
- Performance benchmarks
- Memory usage validation
- Concurrent extraction testing

### Performance Tests
- Throughput measurements
- Memory usage profiling
- Instance pool efficiency
- SIMD optimization validation

## Security Considerations

1. **Sandboxing**: Component Model provides better isolation than WASI
2. **Resource Limits**: Memory, CPU, and time limits prevent abuse
3. **Input Validation**: Strict validation of HTML and URL inputs
4. **Fuel Limits**: Prevent infinite loops and excessive computation

## Success Metrics

1. **Performance**: 2x improvement in extraction speed
2. **Memory**: 50% reduction in memory usage per extraction
3. **Reliability**: 99.9% success rate for valid inputs
4. **Concurrency**: Support for 100+ concurrent extractions

## Risk Mitigation

1. **Compatibility**: Maintain backward compatibility during migration
2. **Fallback**: Keep existing implementation as fallback
3. **Monitoring**: Comprehensive metrics and alerting
4. **Rollback**: Quick rollback capability if issues arise

## Timeline

- **Week 1-2**: Foundation setup and WIT interface design
- **Week 3-4**: Core implementation with trek-rs integration
- **Week 5-6**: Performance optimization and production readiness
- **Week 7**: Testing, documentation, and deployment

## Conclusion

This migration to WASM Component Model will provide significant improvements in performance, security, and maintainability. The typed interfaces and proper trek-rs integration will enable more robust content extraction capabilities while maintaining the sandboxed security benefits of WebAssembly.

The phased approach ensures minimal disruption to existing functionality while delivering measurable improvements at each stage.