use anyhow::{anyhow, Result};
use crate::types::ExtractedDoc;
use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::{WasiCtxBuilder};

pub struct WasmExtractor {
    engine: Engine,
    module: Module,
}

impl WasmExtractor {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, wasm_path)?;
        Ok(Self { engine, module })
    }

    pub fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        // For simplicity & robustness: run the WASM as a WASI "command" per extraction.
        // (Later you can optimize with component model/pooling.)
        let mut linker = wasmtime::Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

        let wasi = WasiCtxBuilder::new()
            .env("RIPTIDE_URL", url)?
            .env("RIPTIDE_MODE", mode)?
            .stdin(Box::new(wasmtime_wasi::pipe::MemoryInputPipe::new(html)))
            .stdout(Box::new(wasmtime_wasi::pipe::MemoryOutputPipe::new()))
            .build();

        let mut store = Store::new(&self.engine, wasi);

        let instance = linker.instantiate(&mut store, &self.module)?;
        // Expect a WASI `_start` entrypoint
        let start = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
        start.call(&mut store, ())?;

        let stdout = store.data().stdout()?;
        let contents = stdout
            .try_into_inner()
            .map_err(|_| anyhow!("Failed to read stdout from WASM"))?
            .into_inner();

        let doc: ExtractedDoc = serde_json::from_slice(&contents)
            .map_err(|e| anyhow!("WASM extractor JSON decode failed: {e}"))?;
        Ok(doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        // This will fail without a valid WASM file, but tests the constructor
        let result = WasmExtractor::new("nonexistent.wasm");
        assert!(result.is_err());
    }
}