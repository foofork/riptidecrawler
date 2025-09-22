use anyhow::Result;
use wasmtime::{component::*, Config, Engine, Store};

use crate::types::{ComponentInfo, ExtractedDoc, ExtractionMode, ExtractionStats, HealthStatus};

// Generate bindings from enhanced WIT file
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
});

/// WebAssembly Component Model extractor for content extraction.
///
/// This extractor uses the WebAssembly Component Model (WASM-CM) to run
/// content extraction in a sandboxed environment. It provides better
/// performance and security compared to traditional WASI approaches.
///
/// # Architecture
///
/// The component model allows for:
/// - Type-safe interfaces defined in WIT (WebAssembly Interface Types)
/// - Better resource management and sandboxing
/// - Compositional design with multiple components
/// - Language-agnostic interfaces
///
/// # Performance
///
/// Component model extractors typically offer:
/// - Faster instantiation than WASI commands
/// - Better memory reuse across extractions
/// - More efficient host-guest communication
/// - Reduced overhead for repeated operations
pub struct CmExtractor {
    /// WebAssembly engine configured for component model
    engine: Engine,

    /// Pre-compiled WebAssembly component
    component: Component,

    /// Linker for importing host functions
    linker: Linker<()>,
}

impl CmExtractor {
    /// Creates a new component model extractor from a WASM file.
    ///
    /// # Arguments
    ///
    /// * `wasm_path` - Path to the WebAssembly component file
    ///
    /// # Returns
    ///
    /// A new `CmExtractor` instance ready for content extraction.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WASM file cannot be read or is invalid
    /// - The component doesn't implement the expected interface
    /// - The WebAssembly engine cannot be initialized
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_core::component::CmExtractor;
    ///
    /// let extractor = CmExtractor::new("./extractor.wasm")?;
    /// ```
    pub fn new(wasm_path: &str) -> Result<Self> {
        // Configure Wasmtime for component model support
        let mut config = Config::new();
        config.wasm_component_model(true);

        // Enable additional features for better performance
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);

        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, wasm_path)?;
        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            component,
            linker,
        })
    }

    /// Extracts content from HTML using the WebAssembly component.
    ///
    /// This method instantiates the component, calls the extraction function,
    /// and returns the structured content data.
    ///
    /// # Arguments
    ///
    /// * `html` - Raw HTML content to extract from
    /// * `url` - Source URL for context and link resolution
    /// * `mode` - Extraction mode ("article", "full", "metadata")
    ///
    /// # Returns
    ///
    /// An `ExtractedDoc` containing the structured content data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Component instantiation fails
    /// - The extraction function throws an exception
    /// - The returned JSON cannot be parsed
    /// - Memory limits are exceeded
    ///
    /// # Performance Notes
    ///
    /// Each call to `extract` creates a new component instance. For high-throughput
    /// scenarios, consider using an instance pool or the reusable extraction methods.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use riptide_core::component::CmExtractor;
    ///
    /// let extractor = CmExtractor::new("./extractor.wasm")?;
    /// let html = "<html><body><h1>Title</h1><p>Content</p></body></html>";
    /// let doc = extractor.extract(html, "https://example.com", "article")?;
    ///
    /// println!("Title: {:?}", doc.title);
    /// println!("Text: {}", doc.text);
    /// ```
    /// Extract content using the legacy string-based mode for backward compatibility
    pub fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedDoc> {
        // Convert legacy mode string to new ExtractionMode enum
        let extraction_mode = match mode {
            "article" => ExtractionMode::Article,
            "full" => ExtractionMode::Full,
            "metadata" => ExtractionMode::Metadata,
            _ => ExtractionMode::Article, // Default fallback
        };

        self.extract_typed(html, url, extraction_mode)
    }

    /// Extract content using the enhanced typed interface
    pub fn extract_typed(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Create a new store for this extraction operation
        let mut store = Store::new(&self.engine, ());

        // Instantiate the component with the configured linker
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Convert our ExtractionMode to the WIT extraction-mode
        let wit_mode = match mode {
            ExtractionMode::Article => {
                exports::riptide::extractor::extract::ExtractionMode::Article
            }
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => {
                exports::riptide::extractor::extract::ExtractionMode::Metadata
            }
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extract::ExtractionMode::Custom(selectors)
            }
        };

        // Call the enhanced extraction function
        let result = bindings
            .interface0
            .call_extract(&mut store, html, url, &wit_mode)?;

        match result {
            Ok(extracted_content) => {
                // Convert from Component Model types to our internal types
                Ok(ExtractedDoc {
                    url: extracted_content.url,
                    title: extracted_content.title,
                    byline: extracted_content.byline,
                    published_iso: extracted_content.published_iso,
                    markdown: extracted_content.markdown,
                    text: extracted_content.text,
                    links: extracted_content.links,
                    media: extracted_content.media,
                    language: extracted_content.language,
                    reading_time: extracted_content.reading_time,
                    quality_score: extracted_content.quality_score,
                    word_count: extracted_content.word_count,
                    categories: extracted_content.categories,
                    site_name: extracted_content.site_name,
                    description: extracted_content.description,
                })
            }
            Err(extraction_error) => {
                // Convert Component Model error to anyhow::Error
                let error_msg = match extraction_error {
                    exports::riptide::extractor::extract::ExtractionError::InvalidHtml(msg) => {
                        format!("Invalid HTML: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::NetworkError(msg) => {
                        format!("Network error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ParseError(msg) => {
                        format!("Parse error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ResourceLimit(msg) => {
                        format!("Resource limit: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::ExtractorError(msg) => {
                        format!("Extractor error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::InternalError(msg) => {
                        format!("Internal error: {}", msg)
                    }
                    exports::riptide::extractor::extract::ExtractionError::UnsupportedMode(msg) => {
                        format!("Unsupported mode: {}", msg)
                    }
                };
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Extract content with detailed performance statistics
    pub fn extract_with_stats(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<(ExtractedDoc, ExtractionStats)> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Convert our ExtractionMode to the WIT extraction-mode
        let wit_mode = match mode {
            ExtractionMode::Article => {
                exports::riptide::extractor::extract::ExtractionMode::Article
            }
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => {
                exports::riptide::extractor::extract::ExtractionMode::Metadata
            }
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extract::ExtractionMode::Custom(selectors)
            }
        };

        let result = bindings
            .interface0
            .call_extract_with_stats(&mut store, html, url, &wit_mode)?;

        match result {
            Ok((extracted_content, stats)) => {
                let doc = ExtractedDoc {
                    url: extracted_content.url,
                    title: extracted_content.title,
                    byline: extracted_content.byline,
                    published_iso: extracted_content.published_iso,
                    markdown: extracted_content.markdown,
                    text: extracted_content.text,
                    links: extracted_content.links,
                    media: extracted_content.media,
                    language: extracted_content.language,
                    reading_time: extracted_content.reading_time,
                    quality_score: extracted_content.quality_score,
                    word_count: extracted_content.word_count,
                    categories: extracted_content.categories,
                    site_name: extracted_content.site_name,
                    description: extracted_content.description,
                };

                let extraction_stats = ExtractionStats {
                    processing_time_ms: stats.processing_time_ms,
                    memory_used: stats.memory_used,
                    nodes_processed: stats.nodes_processed,
                    links_found: stats.links_found,
                    images_found: stats.images_found,
                };

                Ok((doc, extraction_stats))
            }
            Err(extraction_error) => {
                let error_msg = format!("Extraction failed: {:?}", extraction_error);
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Validate HTML content without full extraction
    pub fn validate_html(&self, html: &str) -> Result<bool> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        match bindings.interface0.call_validate_html(&mut store, html)? {
            Ok(is_valid) => Ok(is_valid),
            Err(err) => Err(anyhow::anyhow!("Validation error: {:?}", err)),
        }
    }

    /// Get component health status
    pub fn health_check(&self) -> Result<HealthStatus> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        let health = bindings.interface0.call_health_check(&mut store)?;
        Ok(HealthStatus {
            status: health.status,
            version: health.version,
            trek_version: health.trek_version,
            capabilities: health.capabilities,
            memory_usage: health.memory_usage,
            extraction_count: health.extraction_count,
        })
    }

    /// Get detailed component information
    pub fn get_info(&self) -> Result<ComponentInfo> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        let info = bindings.interface0.call_get_info(&mut store)?;
        Ok(ComponentInfo {
            name: info.name,
            version: info.version,
            component_model_version: info.component_model_version,
            features: info.features,
            supported_modes: info.supported_modes,
            build_timestamp: info.build_timestamp,
            git_commit: info.git_commit,
        })
    }

    /// Reset component state and clear caches
    pub fn reset_state(&self) -> Result<String> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        match bindings.interface0.call_reset_state(&mut store)? {
            Ok(message) => Ok(message),
            Err(err) => Err(anyhow::anyhow!("Reset failed: {:?}", err)),
        }
    }

    /// Get supported extraction modes
    pub fn get_modes(&self) -> Result<Vec<String>> {
        let mut store = Store::new(&self.engine, ());
        let bindings = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        Ok(bindings.interface0.call_get_modes(&mut store)?)
    }
}
