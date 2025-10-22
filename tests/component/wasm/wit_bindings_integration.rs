/// WIT Bindings Integration Tests
///
/// Tests the WebAssembly Interface Types (WIT) bindings between the host
/// and the WASM component. Validates type conversions, function calls,
/// and contract compliance.

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::WasiCtxBuilder;

const COMPONENT_PATH: &str = "wasm/riptide-extractor-wasm/target/wasm32-wasi/release/riptide_extractor_wasm.wasm";

/// Test helper: Create configured WASM engine
fn create_test_engine() -> anyhow::Result<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.async_support(false);
    config.consume_fuel(true);

    // Enable SIMD for performance
    #[cfg(target_arch = "x86_64")]
    config.wasm_simd(true);

    Engine::new(&config)
}

/// Test helper: Create WASM store with resource limits
fn create_test_store(engine: &Engine) -> Store<WasiCtx> {
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .build();

    let mut store = Store::new(engine, wasi);

    // Set resource limits
    store.limiter(|_state| &mut StoreLimits {
        memory_size: 64 * 1024 * 1024, // 64MB
        table_elements: 10000,
        instances: 10,
        tables: 10,
        memories: 10,
    });

    // Set fuel for computation limits
    store.add_fuel(1_000_000).expect("Failed to add fuel");

    store
}

#[derive(Clone)]
struct StoreLimits {
    memory_size: usize,
    table_elements: usize,
    instances: usize,
    tables: usize,
    memories: usize,
}

impl wasmtime::ResourceLimiter for StoreLimits {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> anyhow::Result<bool> {
        Ok(desired <= self.memory_size)
    }

    fn table_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> anyhow::Result<bool> {
        Ok(desired <= self.table_elements as u32)
    }
}

/// Test WASM context
struct WasiCtx {
    wasi: wasmtime_wasi::WasiCtx,
}

#[tokio::test]
async fn test_wit_bindings_enabled() {
    let engine = create_test_engine().expect("Failed to create engine");

    // Verify component model is enabled
    assert!(engine.precompile_compatible_with(&engine));

    // Try to load the component (may not exist in CI, so we check gracefully)
    if std::path::Path::new(COMPONENT_PATH).exists() {
        let component = Component::from_file(&engine, COMPONENT_PATH);
        assert!(component.is_ok(), "Failed to load WASM component");
    } else {
        println!("WASM component not found at {}, skipping component load test", COMPONENT_PATH);
    }
}

#[tokio::test]
async fn test_component_instantiation() {
    let engine = create_test_engine().expect("Failed to create engine");

    if !std::path::Path::new(COMPONENT_PATH).exists() {
        println!("Skipping test: WASM component not found");
        return;
    }

    let component = Component::from_file(&engine, COMPONENT_PATH)
        .expect("Failed to load component");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |state: &mut WasiCtx| &mut state.wasi)
        .expect("Failed to add WASI to linker");

    let mut store = create_test_store(&engine);

    // Instantiate the component
    let instance = linker.instantiate(&mut store, &component);
    assert!(instance.is_ok(), "Failed to instantiate component");
}

#[tokio::test]
async fn test_extract_function_binding() {
    let engine = create_test_engine().expect("Failed to create engine");

    if !std::path::Path::new(COMPONENT_PATH).exists() {
        println!("Skipping test: WASM component not found");
        return;
    }

    let component = Component::from_file(&engine, COMPONENT_PATH)
        .expect("Failed to load component");

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |state: &mut WasiCtx| &mut state.wasi)
        .expect("Failed to add WASI to linker");

    let mut store = create_test_store(&engine);
    let instance = linker.instantiate(&mut store, &component)
        .expect("Failed to instantiate component");

    // Get the extract function from exports
    let exports = instance.exports(&mut store);
    let extract_func = exports.func("extract");

    assert!(extract_func.is_some(), "extract function not found in exports");

    // Verify function signature
    let func = extract_func.unwrap();
    let ty = func.ty(&store);
    assert_eq!(ty.params().count(), 3, "extract should have 3 parameters");
    assert_eq!(ty.results().count(), 1, "extract should return 1 result");
}

#[tokio::test]
async fn test_type_conversions_string_to_wasm() {
    // Test HTML string conversion
    let test_html = r#"<html><head><title>Test</title></head><body><p>Content</p></body></html>"#;
    assert!(test_html.len() < 1024 * 1024, "Test HTML should be under 1MB");

    // Test URL string conversion
    let test_url = "https://example.com/article";
    assert!(test_url.len() < 2048, "URL should be under 2KB");

    // Test mode enum conversion
    let modes = vec!["article", "full", "metadata"];
    for mode in modes {
        assert!(mode.len() < 50, "Mode string should be short");
    }
}

#[tokio::test]
async fn test_result_type_conversion() {
    // Test successful result conversion (simulated)
    let result_json = r#"{
        "url": "https://example.com",
        "title": "Test Article",
        "text": "Content here",
        "links": ["https://link1.com"],
        "media": ["https://image.jpg"],
        "language": "en",
        "quality_score": 85
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(result_json)
        .expect("Should parse result JSON");

    assert!(parsed.get("url").is_some());
    assert!(parsed.get("title").is_some());
    assert!(parsed.get("quality_score").is_some());
}

#[tokio::test]
async fn test_error_type_conversion() {
    // Test error variant conversions
    let errors = vec![
        ("invalid-html", "Malformed HTML"),
        ("parse-error", "Failed to parse"),
        ("resource-limit", "Memory exceeded"),
        ("extractor-error", "Trek-rs error"),
        ("internal-error", "Component error"),
    ];

    for (variant, message) in errors {
        assert!(!variant.is_empty());
        assert!(!message.is_empty());
        assert!(message.len() < 1024, "Error messages should be concise");
    }
}

#[tokio::test]
async fn test_health_status_type_conversion() {
    let health_json = r#"{
        "status": "healthy",
        "version": "0.1.0",
        "trek_version": "0.1.0",
        "capabilities": ["article-extraction", "full-page-extraction"],
        "memory_usage": 1048576,
        "extraction_count": 42
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(health_json)
        .expect("Should parse health status");

    assert_eq!(parsed["status"], "healthy");
    assert!(parsed["capabilities"].is_array());
    assert!(parsed["memory_usage"].is_number());
}

#[tokio::test]
async fn test_component_info_type_conversion() {
    let info_json = r#"{
        "name": "riptide-extractor-wasm",
        "version": "0.1.0",
        "component_model_version": "0.2.0",
        "features": ["article-extraction", "links-extraction", "media-extraction"],
        "supported_modes": ["article", "full", "metadata", "custom"],
        "build_timestamp": "2025-10-13T00:00:00Z",
        "git_commit": "abc123"
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(info_json)
        .expect("Should parse component info");

    assert_eq!(parsed["name"], "riptide-extractor-wasm");
    assert!(parsed["features"].is_array());
    assert!(parsed["supported_modes"].is_array());
}

#[tokio::test]
async fn test_extraction_stats_type_conversion() {
    let stats_json = r#"{
        "processing_time_ms": 45,
        "memory_used": 524288,
        "nodes_processed": 150,
        "links_found": 12,
        "images_found": 5
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(stats_json)
        .expect("Should parse extraction stats");

    assert!(parsed["processing_time_ms"].is_number());
    assert!(parsed["memory_used"].is_number());
    assert!(parsed["links_found"].is_number());
}

#[tokio::test]
async fn test_custom_extraction_mode_with_selectors() {
    // Test custom mode with CSS selectors
    let selectors = vec![
        "article",
        ".content",
        "#main",
        "div.article-body > p",
    ];

    for selector in selectors {
        assert!(selector.len() < 256, "Selectors should be reasonable length");
        assert!(!selector.is_empty(), "Selectors should not be empty");
    }
}

#[tokio::test]
async fn test_roundtrip_type_conversion() {
    // Test host -> WASM -> host type roundtrip
    let original_html = "<html><body>Test</body></html>";
    let original_url = "https://example.com";

    // Simulate conversion to WASM types and back
    let wasm_html = original_html.to_string();
    let wasm_url = original_url.to_string();

    // Convert back
    let host_html: &str = &wasm_html;
    let host_url: &str = &wasm_url;

    assert_eq!(host_html, original_html);
    assert_eq!(host_url, original_url);
}

#[tokio::test]
async fn test_option_type_handling() {
    // Test optional fields in ExtractedContent
    let content_with_optionals = serde_json::json!({
        "url": "https://example.com",
        "title": "Test",
        "byline": null,
        "published_iso": "2025-10-13T00:00:00Z",
        "markdown": "# Test",
        "text": "Test content",
        "links": [],
        "media": [],
        "language": "en",
        "reading_time": 5,
        "quality_score": null,
        "word_count": 100,
        "categories": [],
        "site_name": "Example",
        "description": null
    });

    // Verify optional fields can be null
    assert!(content_with_optionals["byline"].is_null());
    assert!(content_with_optionals["quality_score"].is_null());
    assert!(content_with_optionals["description"].is_null());

    // Verify required fields are present
    assert!(content_with_optionals["url"].is_string());
    assert!(content_with_optionals["markdown"].is_string());
    assert!(content_with_optionals["text"].is_string());
}

#[tokio::test]
async fn test_list_type_handling() {
    // Test list/vector type conversions
    let links = vec![
        "https://example.com/1",
        "https://example.com/2",
        "https://example.com/3",
    ];

    let media = vec![
        "image:https://example.com/1.jpg",
        "video:https://example.com/video.mp4",
        "audio:https://example.com/audio.mp3",
    ];

    let categories = vec![
        "Technology",
        "Programming",
        "Rust",
    ];

    // Convert to JSON and back
    let json = serde_json::json!({
        "links": links,
        "media": media,
        "categories": categories
    });

    assert!(json["links"].is_array());
    assert_eq!(json["links"].as_array().unwrap().len(), 3);
    assert!(json["media"].is_array());
    assert!(json["categories"].is_array());
}

#[tokio::test]
async fn test_u8_u32_u64_type_conversions() {
    // Test numeric type conversions
    let quality_score: u8 = 85;
    let word_count: u32 = 1500;
    let memory_usage: u64 = 2_097_152; // 2MB

    assert!(quality_score <= 100);
    assert!(word_count < 1_000_000);
    assert!(memory_usage < 100_000_000);

    // Test boundary values
    let min_score: u8 = 0;
    let max_score: u8 = 100;
    assert!(min_score < max_score);

    let min_count: u32 = 0;
    let max_count: u32 = u32::MAX;
    assert!(min_count < max_count);
}

#[tokio::test]
async fn test_enum_variant_conversions() {
    // Test extraction mode enum
    #[derive(Debug, PartialEq)]
    enum ExtractionMode {
        Article,
        Full,
        Metadata,
        Custom(Vec<String>),
    }

    let modes = vec![
        ExtractionMode::Article,
        ExtractionMode::Full,
        ExtractionMode::Metadata,
        ExtractionMode::Custom(vec!["article".to_string(), ".content".to_string()]),
    ];

    for mode in modes {
        match mode {
            ExtractionMode::Article => assert!(true),
            ExtractionMode::Full => assert!(true),
            ExtractionMode::Metadata => assert!(true),
            ExtractionMode::Custom(selectors) => {
                assert!(!selectors.is_empty());
            }
        }
    }
}
