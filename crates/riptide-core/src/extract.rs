use crate::component::CmExtractor;
use crate::types::ExtractedDoc;
use anyhow::Result;

pub struct WasmExtractor {
    cm_extractor: CmExtractor,
}

impl WasmExtractor {
    pub fn new(wasm_path: &str) -> Result<Self> {
        let cm_extractor = CmExtractor::new(wasm_path)?;
        Ok(Self { cm_extractor })
    }

    pub fn extract(&self, html: &[u8], url: &str, mode: &str) -> Result<ExtractedDoc> {
        let html_str = String::from_utf8_lossy(html);
        self.cm_extractor.extract(&html_str, url, mode)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires WASM component to be built
    fn test_extractor_creation() {
        let wasm_path = "../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";
        let result = WasmExtractor::new(wasm_path);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Requires WASM component to be built
    fn test_simple_extraction() {
        let wasm_path = "../../target/wasm32-wasip2/release/riptide_extractor_wasm.wasm";
        let extractor = WasmExtractor::new(wasm_path).unwrap();
        let html = b"<html><head><title>Test</title></head><body><p>Content</p></body></html>";
        let result = extractor
            .extract(html, "https://test.com", "article")
            .unwrap();

        assert_eq!(result.url, "https://test.com");
        assert_eq!(result.title, Some("Test".to_string()));
        assert!(result.text.contains("Content"));
    }
}
