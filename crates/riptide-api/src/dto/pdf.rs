//! PDF API DTOs - Request/Response types for PDF processing
//!
//! Extracted from handlers/pdf.rs (Phase 3 Sprint 3.1)
//! Contains all 3 DTOs + options builder

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfProcessRequest {
    pub pdf_data: Option<String>,
    pub filename: Option<String>,
    pub stream_progress: Option<bool>,
    pub url: Option<String>,
    pub timeout: Option<u64>,
}

impl PdfProcessRequest {
    pub fn to_facade_options(&self) -> riptide_facade::facades::PdfProcessOptions {
        riptide_facade::facades::PdfProcessOptions {
            extract_text: true,
            extract_metadata: true,
            extract_images: false,
            include_page_numbers: true,
            filename: self.filename.clone(),
            url: self.url.clone(),
            timeout: self.timeout,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfProcessResponse {
    pub success: bool,
    pub document: Option<riptide_types::ExtractedDoc>,
    pub error: Option<String>,
    pub stats: riptide_facade::facades::ProcessingStats,
}

#[cfg(test)]
pub type ProcessingStats = riptide_facade::facades::ProcessingStats;
