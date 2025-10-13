use anyhow::Result;
use std::path::PathBuf;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

// Generate typed bindings from the WIT world
bindgen!({
    world: "extractor",
    path: "../../wasm/riptide-extractor-wasm/wit/extractor.wit",
});

// Host state that implements WasiView
struct Host {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for Host {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Get the path to the WASM component binary
fn wasm_path() -> PathBuf {
    std::env::var_os("RIPTIDE_WASM_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // Default to the componentized artifact
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm")
        })
}

/// Helper function to extract content using the WASM component
pub fn extract_content(html: &str, url: &str, mode: &str) -> Result<ExtractedContent> {
    let mut cfg = Config::new();
    cfg.wasm_component_model(true);
    let engine = Engine::new(&cfg)?;
    let component = Component::from_file(&engine, wasm_path())?;

    // Create linker with WASI Preview 2 support
    let mut linker: Linker<Host> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Create WASI context with builder
    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_env().build();

    let host = Host {
        wasi,
        table: ResourceTable::new(),
    };
    let mut store = Store::new(&engine, host);

    let extractor = Extractor::instantiate(&mut store, &component, &linker)?;

    // Convert mode string to extraction mode enum
    let extraction_mode = match mode {
        "article" => ExtractionMode::Article,
        "full" => ExtractionMode::Full,
        "metadata" => ExtractionMode::Metadata,
        _ => ExtractionMode::Article, // default
    };

    match extractor.call_extract(&mut store, html, url, &extraction_mode)? {
        Ok(content) => Ok(content),
        Err(err) => anyhow::bail!("Extraction failed: {:?}", err),
    }
}

/// Check if the WASM component exists and is loadable
pub fn component_available() -> bool {
    wasm_path().exists()
}
