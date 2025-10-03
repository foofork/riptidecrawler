use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::config::ImageFormat;

/// Enhanced extraction result with comprehensive metadata
/// This is a local definition to avoid circular dependencies with riptide-core
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractedDoc {
    /// Source URL for context and link resolution
    pub url: String,

    /// Extracted page title
    pub title: Option<String>,

    /// Author/byline information
    pub byline: Option<String>,

    /// Publication date in ISO 8601 format
    pub published_iso: Option<String>,

    /// Content formatted as Markdown
    pub markdown: String,

    /// Plain text content with HTML tags removed
    pub text: String,

    /// List of extracted hyperlinks
    pub links: Vec<String>,

    /// List of media URLs (images, videos, audio)
    pub media: Vec<String>,

    /// Detected content language (ISO 639-1 code)
    pub language: Option<String>,

    /// Estimated reading time in minutes
    pub reading_time: Option<u32>,

    /// Content quality score (0-100, higher = better)
    pub quality_score: Option<u8>,

    /// Word count of extracted text
    pub word_count: Option<u32>,

    /// Content categories/tags if detected
    pub categories: Vec<String>,

    /// Site name/publisher if available
    pub site_name: Option<String>,

    /// Meta description from page
    pub description: Option<String>,
}

/// Result of PDF processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfProcessingResult {
    /// Whether processing was successful
    pub success: bool,

    /// Extracted text content
    pub text: Option<String>,

    /// Extracted images
    pub images: Vec<PdfImage>,

    /// Document metadata
    pub metadata: PdfMetadata,

    /// Structured content (tables, lists, etc.)
    pub structured_content: Option<StructuredContent>,

    /// Processing statistics
    pub stats: PdfStats,

    /// Error message if processing failed
    pub error: Option<String>,
}

/// Extracted image from PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfImage {
    /// Image index in document
    pub index: u32,

    /// Page number where image appears
    pub page: u32,

    /// Image data (base64 encoded if configured)
    pub data: Option<String>,

    /// Image format
    pub format: ImageFormat,

    /// Image dimensions
    pub width: u32,
    pub height: u32,

    /// Position on page (if available)
    pub position: Option<ImagePosition>,

    /// Alternative text or caption
    pub alt_text: Option<String>,
}

/// Image position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePosition {
    /// X coordinate (left edge)
    pub x: f32,

    /// Y coordinate (top edge)
    pub y: f32,

    /// Width on page
    pub width: f32,

    /// Height on page
    pub height: f32,
}

/// PDF document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    /// Document title
    pub title: Option<String>,

    /// Document author
    pub author: Option<String>,

    /// Document subject
    pub subject: Option<String>,

    /// Document keywords
    pub keywords: Option<String>,

    /// Creator application
    pub creator: Option<String>,

    /// Producer application
    pub producer: Option<String>,

    /// Creation date
    pub creation_date: Option<String>,

    /// Modification date
    pub modification_date: Option<String>,

    /// PDF version
    pub pdf_version: Option<String>,

    /// Number of pages
    pub page_count: u32,

    /// Document is encrypted
    pub encrypted: bool,

    /// Document allows copying
    pub allows_copying: bool,

    /// Document allows printing
    pub allows_printing: bool,

    /// Custom metadata
    pub custom_metadata: HashMap<String, String>,
}

/// Structured content extracted from PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredContent {
    /// Extracted tables
    pub tables: Vec<PdfTable>,

    /// Extracted lists
    pub lists: Vec<PdfList>,

    /// Document outline/bookmarks
    pub outline: Vec<OutlineItem>,

    /// Detected forms and fields
    pub forms: Vec<FormField>,
}

/// Table extracted from PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfTable {
    /// Page number
    pub page: u32,

    /// Table position
    pub position: Option<ImagePosition>,

    /// Table rows
    pub rows: Vec<Vec<String>>,

    /// Column headers (if detected)
    pub headers: Option<Vec<String>>,

    /// Table caption
    pub caption: Option<String>,
}

/// List extracted from PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfList {
    /// Page number
    pub page: u32,

    /// List items
    pub items: Vec<String>,

    /// List type (ordered, unordered)
    pub list_type: ListType,

    /// Nesting level
    pub level: u32,
}

/// List types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Ordered,
    Unordered,
    Definition,
}

/// Document outline item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    /// Outline title
    pub title: String,

    /// Target page number
    pub page: Option<u32>,

    /// Nesting level
    pub level: u32,

    /// Child items
    pub children: Vec<OutlineItem>,
}

/// Form field from PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Field name
    pub name: String,

    /// Field type
    pub field_type: FieldType,

    /// Field value
    pub value: Option<String>,

    /// Field position
    pub position: Option<ImagePosition>,

    /// Whether field is required
    pub required: bool,
}

/// Form field types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Checkbox,
    Radio,
    ComboBox,
    ListBox,
    Button,
    Signature,
}

/// PDF processing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfStats {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Memory usage in bytes
    pub memory_used: u64,

    /// Number of pages processed
    pub pages_processed: u32,

    /// Number of images extracted
    pub images_extracted: u32,

    /// Number of tables found
    pub tables_found: u32,

    /// Total text length
    pub text_length: u32,

    /// File size in bytes
    pub file_size: u64,
}

/// Page processing progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingProgress {
    /// Current page being processed
    pub current_page: u32,

    /// Total pages in document
    pub total_pages: u32,

    /// Progress percentage (0.0 to 100.0)
    pub percentage: f32,

    /// Estimated time remaining in milliseconds
    pub estimated_remaining_ms: Option<u64>,

    /// Current processing stage
    pub stage: ProcessingStage,
}

/// Processing stages for progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStage {
    Loading,
    ExtractingMetadata,
    ExtractingText(u32),   // page number
    ExtractingImages(u32), // page number
    ExtractingTables(u32), // page number
    OcrProcessing(u32),    // page number
    Finalizing,
    Complete,
}

/// Progress callback type for page-by-page processing
pub type ProgressCallback = Box<dyn Fn(u32, u32) + Send + Sync>;

/// Enhanced progress callback with detailed progress information
pub type DetailedProgressCallback = Box<dyn Fn(ProcessingProgress) + Send + Sync>;

/// Progress update events for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressUpdate {
    /// Processing started with document info
    Started {
        total_pages: u32,
        file_size: u64,
        timestamp: String,
    },
    /// Page processing progress
    Progress(ProcessingProgress),
    /// Processing stage changed
    StageChanged {
        stage: ProcessingStage,
        timestamp: String,
    },
    /// Processing completed successfully
    Completed {
        result: Box<PdfProcessingResult>,
        timestamp: String,
    },
    /// Processing failed with error
    Failed { error: String, timestamp: String },
    /// Keep-alive message
    KeepAlive { timestamp: String },
}

impl Default for ProgressUpdate {
    fn default() -> Self {
        ProgressUpdate::KeepAlive {
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Progress channel for async communication
pub type ProgressSender = tokio::sync::mpsc::UnboundedSender<ProgressUpdate>;
pub type ProgressReceiver = tokio::sync::mpsc::UnboundedReceiver<ProgressUpdate>;

/// Create a progress channel pair
pub fn create_progress_channel() -> (ProgressSender, ProgressReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processing_result_creation() {
        let result = PdfProcessingResult {
            success: true,
            text: Some("Sample text".to_string()),
            images: vec![],
            metadata: PdfMetadata {
                title: Some("Test Document".to_string()),
                author: None,
                subject: None,
                keywords: None,
                creator: None,
                producer: None,
                creation_date: None,
                modification_date: None,
                pdf_version: Some("1.7".to_string()),
                page_count: 1,
                encrypted: false,
                allows_copying: true,
                allows_printing: true,
                custom_metadata: HashMap::new(),
            },
            structured_content: None,
            stats: PdfStats {
                processing_time_ms: 100,
                memory_used: 1024,
                pages_processed: 1,
                images_extracted: 0,
                tables_found: 0,
                text_length: 11,
                file_size: 2048,
            },
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.text, Some("Sample text".to_string()));
        assert_eq!(result.stats.processing_time_ms, 100);
    }

    #[test]
    fn test_pdf_image_creation() {
        let image = PdfImage {
            index: 0,
            page: 1,
            data: None,
            format: ImageFormat::Png,
            width: 100,
            height: 200,
            position: Some(ImagePosition {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 200.0,
            }),
            alt_text: Some("Test image".to_string()),
        };

        assert_eq!(image.index, 0);
        assert_eq!(image.page, 1);
        assert_eq!(image.width, 100);
        assert_eq!(image.height, 200);
        assert!(image.position.is_some());
    }

    #[test]
    fn test_processing_progress() {
        let progress = ProcessingProgress {
            current_page: 2,
            total_pages: 10,
            percentage: 20.0,
            estimated_remaining_ms: Some(8000),
            stage: ProcessingStage::ExtractingText(2),
        };

        assert_eq!(progress.current_page, 2);
        assert_eq!(progress.total_pages, 10);
        assert_eq!(progress.percentage, 20.0);
        assert!(matches!(progress.stage, ProcessingStage::ExtractingText(2)));
    }
}
