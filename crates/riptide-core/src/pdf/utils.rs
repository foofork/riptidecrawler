//! Utility functions for PDF processing

/// Detect if content is a PDF based on content type and magic bytes
#[allow(dead_code)]
pub fn is_pdf_content(content_type: Option<&str>, data: &[u8]) -> bool {
    // Check content type
    if let Some(ct) = content_type {
        if ct.contains("application/pdf") {
            return true;
        }
    }

    // Check magic bytes
    data.starts_with(b"%PDF-")
}

/// Check if URL should skip headless rendering for PDF content
#[allow(dead_code)]
pub fn should_skip_headless(content_type: &str, url: &str) -> bool {
    content_type.contains("application/pdf") || url.ends_with(".pdf")
}

/// Extract PDF version from header
pub fn extract_pdf_version(data: &[u8]) -> Option<String> {
    if data.len() < 8 || !data.starts_with(b"%PDF-") {
        return None;
    }

    // PDF version is typically in format "%PDF-1.7"
    let header = std::str::from_utf8(&data[0..8]).ok()?;
    if header.len() >= 8 {
        Some(header[5..8].to_string())
    } else {
        None
    }
}

/// Estimate processing complexity based on file size
#[allow(dead_code)]
pub fn estimate_complexity(file_size: u64) -> ProcessingComplexity {
    match file_size {
        0..=1_048_575 => ProcessingComplexity::Low, // < 1MB
        1_048_576..=10_485_759 => ProcessingComplexity::Medium, // 1-10MB
        10_485_760..=52_428_800 => ProcessingComplexity::High, // 10-50MB
        _ => ProcessingComplexity::VeryHigh,        // > 50MB
    }
}

/// Validate PDF header and basic structure
pub fn validate_pdf_header(data: &[u8]) -> Result<String, String> {
    if data.len() < 8 {
        return Err("File too small to be a valid PDF".to_string());
    }

    if !data.starts_with(b"%PDF-") {
        return Err("File does not start with PDF magic bytes".to_string());
    }

    // Extract version from header
    if let Some(version) = extract_pdf_version(data) {
        // Validate version format
        if version.chars().all(|c| c.is_ascii_digit() || c == '.') && version.contains('.') {
            Ok(version)
        } else {
            Err("Invalid PDF version format".to_string())
        }
    } else {
        Err("Could not extract PDF version".to_string())
    }
}

/// Check if PDF likely contains only images (needs OCR)
#[allow(dead_code)]
pub fn likely_needs_ocr(text_content: &str, image_count: usize) -> bool {
    let text_length = text_content.trim().len();
    let has_meaningful_text = text_length > 50; // Arbitrary threshold

    // If we have images but very little text, likely needs OCR
    image_count > 0 && !has_meaningful_text
}

/// Calculate estimated reading time based on word count
pub fn estimate_reading_time(word_count: u32) -> u32 {
    // Average reading speed: 200 words per minute
    const WORDS_PER_MINUTE: u32 = 200;
    (word_count / WORDS_PER_MINUTE).max(1)
}

/// Sanitize text content by removing excessive whitespace
#[allow(dead_code)]
pub fn sanitize_text_content(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Processing complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProcessingComplexity {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl ProcessingComplexity {
    /// Get estimated processing time in seconds
    #[allow(dead_code)]
    pub fn estimated_time_seconds(&self) -> u64 {
        match self {
            ProcessingComplexity::Low => 5,
            ProcessingComplexity::Medium => 15,
            ProcessingComplexity::High => 45,
            ProcessingComplexity::VeryHigh => 120,
        }
    }

    /// Get recommended memory limit in bytes
    #[allow(dead_code)]
    pub fn memory_limit_bytes(&self) -> u64 {
        match self {
            ProcessingComplexity::Low => 50 * 1024 * 1024, // 50MB
            ProcessingComplexity::Medium => 200 * 1024 * 1024, // 200MB
            ProcessingComplexity::High => 500 * 1024 * 1024, // 500MB
            ProcessingComplexity::VeryHigh => 1024 * 1024 * 1024, // 1GB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pdf_content() {
        let pdf_data = b"%PDF-1.7\n...";
        assert!(is_pdf_content(Some("application/pdf"), pdf_data));
        assert!(is_pdf_content(None, pdf_data));
        assert!(!is_pdf_content(None, b"not a pdf"));
    }

    #[test]
    fn test_should_skip_headless() {
        assert!(should_skip_headless(
            "application/pdf",
            "http://example.com/doc.pdf"
        ));
        assert!(should_skip_headless(
            "text/html",
            "http://example.com/doc.pdf"
        ));
        assert!(!should_skip_headless(
            "text/html",
            "http://example.com/page.html"
        ));
    }

    #[test]
    fn test_extract_pdf_version() {
        let pdf_data = b"%PDF-1.7\n...";
        assert_eq!(extract_pdf_version(pdf_data), Some("1.7".to_string()));

        let invalid_data = b"not a pdf";
        assert_eq!(extract_pdf_version(invalid_data), None);

        let short_data = b"%PDF";
        assert_eq!(extract_pdf_version(short_data), None);
    }

    #[test]
    fn test_complexity_estimation() {
        assert_eq!(estimate_complexity(500_000), ProcessingComplexity::Low);
        assert_eq!(estimate_complexity(5_000_000), ProcessingComplexity::Medium);
        assert_eq!(estimate_complexity(25_000_000), ProcessingComplexity::High);
        assert_eq!(
            estimate_complexity(100_000_000),
            ProcessingComplexity::VeryHigh
        );
    }

    #[test]
    fn test_validate_pdf_header() {
        let valid_pdf = b"%PDF-1.7\n...";
        assert!(validate_pdf_header(valid_pdf).is_ok());
        assert_eq!(validate_pdf_header(valid_pdf).unwrap(), "1.7");

        let invalid_pdf = b"not a pdf";
        assert!(validate_pdf_header(invalid_pdf).is_err());

        let short_file = b"%PDF";
        assert!(validate_pdf_header(short_file).is_err());
    }

    #[test]
    fn test_likely_needs_ocr() {
        assert!(likely_needs_ocr("", 5)); // No text, has images
        assert!(!likely_needs_ocr(
            "This is a long text content with meaningful information",
            0
        )); // Has text, no images
        assert!(!likely_needs_ocr(
            "This is a long text content with meaningful information",
            2
        )); // Has text and images
        assert!(likely_needs_ocr("OCR", 3)); // Very short text, has images
    }

    #[test]
    fn test_estimate_reading_time() {
        assert_eq!(estimate_reading_time(100), 1); // Minimum 1 minute
        assert_eq!(estimate_reading_time(400), 2); // 400 words = 2 minutes
        assert_eq!(estimate_reading_time(1000), 5); // 1000 words = 5 minutes
    }

    #[test]
    fn test_sanitize_text_content() {
        let messy_text = "  Line 1  \n\n  \n  Line 2  \n   \n  Line 3  ";
        let expected = "Line 1\nLine 2\nLine 3";
        assert_eq!(sanitize_text_content(messy_text), expected);
    }

    #[test]
    fn test_processing_complexity_methods() {
        let low = ProcessingComplexity::Low;
        assert_eq!(low.estimated_time_seconds(), 5);
        assert_eq!(low.memory_limit_bytes(), 50 * 1024 * 1024);

        let high = ProcessingComplexity::VeryHigh;
        assert_eq!(high.estimated_time_seconds(), 120);
        assert_eq!(high.memory_limit_bytes(), 1024 * 1024 * 1024);
    }
}
