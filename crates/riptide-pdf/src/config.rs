use serde::{Deserialize, Serialize};

/// PDF processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfConfig {
    /// Maximum PDF size to process (in bytes)
    pub max_size_bytes: u64,

    /// Whether to extract text content
    pub extract_text: bool,

    /// Whether to extract images
    pub extract_images: bool,

    /// Whether to extract metadata
    pub extract_metadata: bool,

    /// Image extraction settings
    pub image_settings: ImageExtractionSettings,

    /// Text extraction settings
    pub text_settings: TextExtractionSettings,

    /// Timeout for PDF processing
    pub timeout_seconds: u64,

    /// OCR configuration for image-based PDFs
    pub ocr_config: OcrConfig,

    /// Enable page-by-page processing with progress tracking
    pub enable_progress_tracking: bool,

    /// Memory management settings
    pub memory_settings: MemorySettings,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            extract_text: true,
            extract_images: false,
            extract_metadata: true,
            image_settings: ImageExtractionSettings::default(),
            text_settings: TextExtractionSettings::default(),
            timeout_seconds: 30,
            ocr_config: OcrConfig::default(),
            enable_progress_tracking: false,
            memory_settings: MemorySettings::default(),
        }
    }
}

/// Image extraction settings for PDFs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageExtractionSettings {
    /// Maximum number of images to extract
    pub max_images: u32,

    /// Minimum image dimensions (width x height)
    pub min_dimensions: (u32, u32),

    /// Supported image formats to extract
    pub formats: Vec<ImageFormat>,

    /// Whether to include image position data
    pub include_positions: bool,

    /// Whether to encode images as base64
    pub base64_encode: bool,
}

impl Default for ImageExtractionSettings {
    fn default() -> Self {
        Self {
            max_images: 50,
            min_dimensions: (50, 50),
            formats: vec![ImageFormat::Png, ImageFormat::Jpeg],
            include_positions: true,
            base64_encode: true,
        }
    }
}

/// Supported image formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Tiff,
}

/// Text extraction settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionSettings {
    /// Whether to preserve formatting (newlines, spacing)
    pub preserve_formatting: bool,

    /// Whether to extract text coordinates
    pub include_coordinates: bool,

    /// Whether to group text by blocks/paragraphs
    pub group_by_blocks: bool,

    /// Minimum font size to extract
    pub min_font_size: f32,

    /// Whether to extract tables as structured data
    pub extract_tables: bool,
}

impl Default for TextExtractionSettings {
    fn default() -> Self {
        Self {
            preserve_formatting: true,
            include_coordinates: false,
            group_by_blocks: true,
            min_font_size: 6.0,
            extract_tables: false,
        }
    }
}

/// OCR detection and fallback support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrConfig {
    /// Enable OCR fallback for image-based PDFs
    pub enable_ocr: bool,

    /// OCR confidence threshold (0.0 to 1.0)
    pub confidence_threshold: f32,

    /// Languages to detect (ISO 639-1 codes)
    pub languages: Vec<String>,

    /// OCR engine preference
    pub engine: OcrEngine,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            enable_ocr: false,
            confidence_threshold: 0.7,
            languages: vec!["eng".to_string()],
            engine: OcrEngine::Tesseract,
        }
    }
}

/// OCR engine options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OcrEngine {
    Tesseract,
    Paddle,
    EasyOcr,
}

/// PDF processor capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfCapabilities {
    /// Supports text extraction
    pub text_extraction: bool,

    /// Supports image extraction
    pub image_extraction: bool,

    /// Supports metadata extraction
    pub metadata_extraction: bool,

    /// Supports table extraction
    pub table_extraction: bool,

    /// Supports form field extraction
    pub form_extraction: bool,

    /// Supports encrypted PDFs
    pub encrypted_pdfs: bool,

    /// Maximum file size supported
    pub max_file_size: u64,

    /// Supported PDF versions
    pub supported_versions: Vec<String>,
}

impl PdfCapabilities {
    /// Get current PDF processing capabilities based on available features
    pub fn current() -> Self {
        Self {
            text_extraction: true,
            image_extraction: cfg!(feature = "pdf"),
            metadata_extraction: true,
            table_extraction: cfg!(feature = "pdf"),
            form_extraction: cfg!(feature = "pdf"),
            encrypted_pdfs: cfg!(feature = "pdf"),
            max_file_size: if cfg!(feature = "pdf") {
                100 * 1024 * 1024
            } else {
                10 * 1024 * 1024
            }, // 100MB vs 10MB
            supported_versions: if cfg!(feature = "pdf") {
                vec![
                    "1.0".to_string(),
                    "1.1".to_string(),
                    "1.2".to_string(),
                    "1.3".to_string(),
                    "1.4".to_string(),
                    "1.5".to_string(),
                    "1.6".to_string(),
                    "1.7".to_string(),
                    "2.0".to_string(),
                ]
            } else {
                vec!["1.4".to_string()] // Minimal support
            },
        }
    }

    /// Check if text extraction is supported
    pub fn can_extract_text(&self) -> bool {
        self.text_extraction
    }

    /// Check if image extraction is supported
    pub fn can_extract_images(&self) -> bool {
        self.image_extraction
    }

    /// Check if metadata extraction is supported
    pub fn can_extract_metadata(&self) -> bool {
        self.metadata_extraction
    }
}

/// Memory management settings for PDF processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Maximum memory spike allowed per worker (in bytes)
    pub max_memory_spike_bytes: u64,

    /// Memory check interval (number of pages)
    pub memory_check_interval: usize,

    /// Cleanup interval (number of pages)
    pub cleanup_interval: usize,

    /// Memory pressure threshold (0.0 to 1.0)
    pub memory_pressure_threshold: f64,

    /// Maximum concurrent PDF processing operations
    pub max_concurrent_operations: usize,

    /// Enable aggressive memory cleanup
    pub aggressive_cleanup: bool,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            max_memory_spike_bytes: 200 * 1024 * 1024, // 200MB
            memory_check_interval: 5,
            cleanup_interval: 20,
            memory_pressure_threshold: 0.8, // 80%
            max_concurrent_operations: 2,
            aggressive_cleanup: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_config_default() {
        let config = PdfConfig::default();
        assert!(config.extract_text);
        assert!(config.extract_metadata);
        assert!(!config.extract_images);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_image_settings_default() {
        let settings = ImageExtractionSettings::default();
        assert_eq!(settings.max_images, 50);
        assert_eq!(settings.min_dimensions, (50, 50));
        assert!(settings.include_positions);
        assert!(settings.base64_encode);
    }

    #[test]
    fn test_text_settings_default() {
        let settings = TextExtractionSettings::default();
        assert!(settings.preserve_formatting);
        assert!(!settings.include_coordinates);
        assert!(settings.group_by_blocks);
        assert_eq!(settings.min_font_size, 6.0);
        assert!(!settings.extract_tables);
    }

    #[test]
    fn test_ocr_config_default() {
        let config = OcrConfig::default();
        assert!(!config.enable_ocr);
        assert_eq!(config.confidence_threshold, 0.7);
        assert_eq!(config.languages, vec!["eng".to_string()]);
        assert!(matches!(config.engine, OcrEngine::Tesseract));
    }
}
