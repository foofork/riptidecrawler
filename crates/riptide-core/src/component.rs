use anyhow::Result;
use wasmtime::{component::*, Config, Engine, Store};

use crate::types::ExtractedDoc;

// Generate bindings from WIT file
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
});

pub struct CmExtractor {
    engine: Engine,
    component: Component,
    linker: Linker<()>,
}

impl CmExtractor {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);

        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, wasm_path)?;
        let linker = Linker::new(&engine);

        Ok(Self { engine, component, linker })
    }

    pub fn extract(&self, html: &str, url: &str, mode: &str) -> Result<ExtractedDoc> {
        let mut store = Store::new(&self.engine, ());
        let instance = Extractor::instantiate(&mut store, &self.component, &self.linker)?;

        let json = instance.call_extract(&mut store, html, url, mode)?;
        let doc: ExtractedDoc = serde_json::from_str(&json)?;

        Ok(doc)
    }
}