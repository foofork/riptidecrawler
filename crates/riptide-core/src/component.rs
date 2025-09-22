use anyhow::Result;
use wasmtime::{component::*, Config, Engine, Store};

use crate::types::ExtractedDoc;

// Generate bindings from WIT file
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
    pub fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedDoc> {
        // Create a new store for this extraction operation
        let mut store = Store::new(&self.engine, ());

        // Instantiate the component with the configured linker
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        // Call the extraction function defined in the WIT interface
        let json = instance.call_extract(&mut store, html, url, mode)?;

        // Parse the JSON response into our structured type
        let doc: ExtractedDoc = serde_json::from_str(&json)?;

        Ok(doc)
    }
}
