use crate::component::CmExtractor;
use crate::types::ExtractedDoc;
use anyhow::Result;

pub struct WasmExtractor {
    cm_extractor: CmExtractor,
}

impl WasmExtractor {
    pub async fn new(wasm_path: &str) -> Result<Self> {
        let cm_extractor = CmExtractor::new(wasm_path).await?;
        Ok(Self { cm_extractor })
    }

    pub fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        let html_str = String::from_utf8_lossy(html);
        self.cm_extractor.extract(&html_str, url, mode)
    }
}

// Tests moved to tests/wasm_component_tests.rs using proper Component Model
