# riptide-pdf

PDF processing capabilities for the RipTide web scraping framework, providing native PDF text extraction, image extraction, and content analysis using pdfium-render.

## Overview

`riptide-pdf` is a high-performance PDF processing library designed for the RipTide ecosystem. It provides comprehensive PDF extraction capabilities with built-in memory management, progress tracking, and streaming support for large documents.

The crate uses `pdfium-render` (when the `pdf` feature is enabled) to provide native PDF processing without external dependencies, making it ideal for production environments requiring robust document processing.

## PDF Processing Capabilities

### Text Extraction
- **Page-by-page text extraction** with layout preservation
- **Configurable formatting** (preserve or normalize whitespace)
- **Font size filtering** to exclude small print
- **Text coordinate extraction** for layout analysis
- **Block-level text grouping** for structural analysis

### Image Extraction
- **Embedded image extraction** with format detection
- **Dimension filtering** to exclude small decorative images
- **Position tracking** for layout reconstruction
- **Base64 encoding** for easy transport
- **Format support**: PNG, JPEG, GIF, BMP, TIFF

### Metadata Extraction
- Document title, author, subject, keywords
- Creator and producer applications
- Creation and modification dates
- PDF version information
- Page count and document permissions
- Custom metadata key-value pairs

### Advanced Features
- **Table detection** (experimental)
- **Link extraction** from document annotations
- **Quality scoring** for content assessment
- **Reading time estimation**
- **Memory spike detection** and prevention
- **Streaming support** for large files

## PdfProcessor Trait

The core abstraction is the `PdfProcessor` trait, which provides a consistent interface for PDF processing:

```rust
#[async_trait]
pub trait PdfProcessor: Send + Sync {
    /// Process a PDF from bytes
    async fn process_pdf(
        &self,
        data: &[u8],
        config: &PdfConfig
    ) -> PdfResult<PdfProcessingResult>;

    /// Process a PDF with progress tracking
    async fn process_pdf_with_progress(
        &self,
        data: &[u8],
        config: &PdfConfig,
        progress_callback: Option<ProgressCallback>,
    ) -> PdfResult<PdfProcessingResult>;

    /// Detect if OCR is needed for this PDF
    async fn detect_ocr_need(&self, data: &[u8]) -> PdfResult<bool>;

    /// Check if the processor is available
    fn is_available(&self) -> bool;

    /// Get processor capabilities
    fn capabilities(&self) -> PdfCapabilities;
}
```

### Implementations

- **PdfiumProcessor**: Full-featured implementation using pdfium-render (requires `pdf` feature)
- **DefaultPdfProcessor**: Fallback implementation that returns errors when `pdf` feature is disabled

## Usage Examples

### Basic Text Extraction

```rust
use riptide_pdf::{create_pdf_processor, PdfConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create processor
    let processor = create_pdf_processor();

    // Load PDF file
    let pdf_bytes = std::fs::read("document.pdf")?;

    // Configure extraction
    let config = PdfConfig {
        extract_text: true,
        extract_images: false,
        extract_metadata: true,
        ..Default::default()
    };

    // Process PDF
    let result = processor.process_pdf(&pdf_bytes, &config).await?;

    // Access extracted content
    if let Some(text) = result.text {
        println!("Extracted {} characters", text.len());
        println!("Pages: {}", result.metadata.page_count);
        println!("Processing time: {} ms", result.stats.processing_time_ms);
    }

    Ok(())
}
```

### Streaming with Progress Tracking

```rust
use riptide_pdf::{create_pdf_processor, PdfConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = create_pdf_processor();
    let pdf_bytes = std::fs::read("large_document.pdf")?;

    // Configure with progress tracking enabled
    let config = PdfConfig {
        enable_progress_tracking: true,
        ..Default::default()
    };

    // Create progress callback
    let progress_callback = Box::new(|current: u32, total: u32| {
        let percentage = (current as f32 / total as f32) * 100.0;
        println!("Processing: {:.1}% ({}/{})", percentage, current, total);
    });

    // Process with progress updates
    let result = processor
        .process_pdf_with_progress(&pdf_bytes, &config, Some(progress_callback))
        .await?;

    println!("Completed! Processed {} pages", result.metadata.page_count);

    Ok(())
}
```

### Image and Table Extraction

```rust
use riptide_pdf::{create_pdf_processor, PdfConfig, ImageExtractionSettings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = create_pdf_processor();
    let pdf_bytes = std::fs::read("report.pdf")?;

    // Configure image extraction
    let config = PdfConfig {
        extract_text: true,
        extract_images: true,
        image_settings: ImageExtractionSettings {
            max_images: 100,
            min_dimensions: (100, 100), // Skip small images
            include_positions: true,
            base64_encode: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let result = processor.process_pdf(&pdf_bytes, &config).await?;

    // Process extracted images
    for image in &result.images {
        println!(
            "Image {} on page {}: {}x{}",
            image.index, image.page, image.width, image.height
        );

        if let Some(position) = &image.position {
            println!("  Position: ({}, {})", position.x, position.y);
        }
    }

    println!("Extracted {} images", result.images.len());

    Ok(())
}
```

### OCR Detection

```rust
use riptide_pdf::create_pdf_processor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = create_pdf_processor();
    let pdf_bytes = std::fs::read("scanned_document.pdf")?;

    // Check if document needs OCR
    let needs_ocr = processor.detect_ocr_need(&pdf_bytes).await?;

    if needs_ocr {
        println!("This appears to be a scanned/image-based PDF");
        println!("OCR processing would be recommended");
    } else {
        println!("Document contains extractable text");
    }

    Ok(())
}
```

### Integration with RipTide Core

```rust
use riptide_pdf::{detect_and_process_pdf, PdfConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content_bytes = download_content("https://example.com/doc.pdf").await?;

    // Automatic PDF detection and processing
    if let Some(extracted_doc) = detect_and_process_pdf(
        &content_bytes,
        "https://example.com/doc.pdf",
        &PdfConfig::default()
    ).await? {
        println!("Title: {:?}", extracted_doc.title);
        println!("Text: {}", extracted_doc.text);
        println!("Quality: {:?}", extracted_doc.quality_score);
        println!("Reading time: {:?} min", extracted_doc.reading_time);
    }

    Ok(())
}
```

## Configuration

### PdfConfig

The main configuration structure:

```rust
pub struct PdfConfig {
    /// Maximum PDF size to process (default: 100MB)
    pub max_size_bytes: u64,

    /// Extract text content (default: true)
    pub extract_text: bool,

    /// Extract images (default: false)
    pub extract_images: bool,

    /// Extract metadata (default: true)
    pub extract_metadata: bool,

    /// Image extraction settings
    pub image_settings: ImageExtractionSettings,

    /// Text extraction settings
    pub text_settings: TextExtractionSettings,

    /// Processing timeout (default: 30s)
    pub timeout_seconds: u64,

    /// OCR configuration
    pub ocr_config: OcrConfig,

    /// Enable progress tracking (default: false)
    pub enable_progress_tracking: bool,

    /// Memory management settings
    pub memory_settings: MemorySettings,
}
```

### Memory Settings

Control memory usage and concurrency:

```rust
pub struct MemorySettings {
    /// Maximum memory spike per worker (default: 200MB)
    pub max_memory_spike_bytes: u64,

    /// Check memory every N pages (default: 5)
    pub memory_check_interval: usize,

    /// Cleanup every N pages (default: 20)
    pub cleanup_interval: usize,

    /// Memory pressure threshold 0.0-1.0 (default: 0.8)
    pub memory_pressure_threshold: f64,

    /// Max concurrent operations (default: 2)
    pub max_concurrent_operations: usize,

    /// Enable aggressive cleanup (default: true)
    pub aggressive_cleanup: bool,
}
```

### Text Settings

Configure text extraction behavior:

```rust
pub struct TextExtractionSettings {
    /// Preserve formatting/whitespace (default: true)
    pub preserve_formatting: bool,

    /// Include text coordinates (default: false)
    pub include_coordinates: bool,

    /// Group text by blocks (default: true)
    pub group_by_blocks: bool,

    /// Minimum font size to extract (default: 6.0)
    pub min_font_size: f32,

    /// Extract tables (default: false)
    pub extract_tables: bool,
}
```

### Image Settings

Configure image extraction:

```rust
pub struct ImageExtractionSettings {
    /// Maximum images to extract (default: 50)
    pub max_images: u32,

    /// Minimum dimensions (width, height) (default: 50x50)
    pub min_dimensions: (u32, u32),

    /// Supported formats (default: PNG, JPEG)
    pub formats: Vec<ImageFormat>,

    /// Include position data (default: true)
    pub include_positions: bool,

    /// Base64 encode images (default: true)
    pub base64_encode: bool,
}
```

## Feature Flags

### `pdf` (default)

Enables full PDF processing capabilities using pdfium-render:

```toml
[dependencies]
riptide-pdf = { version = "0.1", features = ["pdf"] }
```

This feature provides:
- Native PDF rendering and text extraction
- Image extraction from PDF objects
- Full metadata extraction
- High-performance processing
- Support for PDF versions 1.0-2.0

**Without this feature**, the crate provides:
- Basic PDF detection
- Error messages guiding users to enable the feature
- Type definitions and configuration structures

### `benchmarks`

Enables performance benchmarking tools:

```toml
[dependencies]
riptide-pdf = { version = "0.1", features = ["benchmarks"] }
```

This feature provides:
- Memory usage benchmarking
- Performance profiling utilities
- Throughput measurement tools

## Performance Considerations

### Memory Management

The crate implements sophisticated memory management:

1. **Concurrency Limiting**: Maximum 2 concurrent PDF operations by default
2. **Memory Spike Detection**: Monitors RSS memory and prevents >200MB spikes
3. **Automatic Cleanup**: Periodic memory cleanup during processing
4. **Aggressive Mode**: Optional aggressive cleanup for memory-constrained environments

### Best Practices

**Large Documents**:
```rust
let config = PdfConfig {
    enable_progress_tracking: true,
    memory_settings: MemorySettings {
        memory_check_interval: 3,  // Check more frequently
        cleanup_interval: 10,       // Clean up more often
        aggressive_cleanup: true,
        ..Default::default()
    },
    ..Default::default()
};
```

**High-Concurrency**:
```rust
let config = PdfConfig {
    memory_settings: MemorySettings {
        max_concurrent_operations: 1,  // Reduce concurrency
        max_memory_spike_bytes: 150 * 1024 * 1024,  // Stricter limit
        ..Default::default()
    },
    ..Default::default()
};
```

**Image-Heavy Documents**:
```rust
let config = PdfConfig {
    extract_images: true,
    image_settings: ImageExtractionSettings {
        max_images: 20,              // Limit image count
        min_dimensions: (200, 200),  // Skip small images
        base64_encode: false,        // Skip encoding if not needed
        ..Default::default()
    },
    ..Default::default()
};
```

### Performance Metrics

The crate provides comprehensive performance tracking:

```rust
let result = processor.process_pdf(&pdf_bytes, &config).await?;

println!("Performance Stats:");
println!("  Processing time: {} ms", result.stats.processing_time_ms);
println!("  Memory used: {} MB", result.stats.memory_used / (1024 * 1024));
println!("  Pages processed: {}", result.stats.pages_processed);
println!("  Throughput: {:.2} pages/sec",
    result.stats.pages_processed as f64 /
    (result.stats.processing_time_ms as f64 / 1000.0)
);
```

## Testing

The crate includes comprehensive test coverage:

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test pdf_extraction_tests
cargo test --test pdf_progress_tests
cargo test --test pdf_memory_stability_test
```

### With PDF Feature

```bash
cargo test --features pdf
```

### Memory Benchmarks

```bash
cargo test --features benchmarks -- --nocapture
```

## Integration with riptide-core

The crate provides seamless integration with the RipTide core framework:

### Pipeline Integration

```rust
use riptide_pdf::create_pdf_integration_for_pipeline;

// Create pipeline integration
let pdf_integration = create_pdf_integration_for_pipeline(PdfConfig::default());

// Use in your extraction pipeline
// The integration handles PDF detection and processing automatically
```

### Content Detection

```rust
use riptide_pdf::detect_pdf_content;

// Detect if content is a PDF
let is_pdf = detect_pdf_content(&content_bytes);

if is_pdf {
    // Process as PDF
}
```

### Automatic Processing

```rust
use riptide_pdf::detect_and_process_pdf;

// One-shot PDF detection and processing
if let Some(doc) = detect_and_process_pdf(
    &bytes,
    "https://example.com/doc.pdf",
    &PdfConfig::default()
).await? {
    // Document was a PDF and is now extracted
    process_document(doc);
}
```

## Known Limitations

### Current Limitations

1. **Table Extraction**: Table detection is experimental and may not work reliably for complex layouts
2. **Form Fields**: Form field extraction is not yet implemented
3. **Encrypted PDFs**: Password-protected PDFs are not supported
4. **OCR Integration**: OCR detection is implemented, but actual OCR processing requires external tools
5. **Metadata API**: Some metadata fields may be empty due to pdfium-render API changes
6. **Image Data**: Raw image data extraction is limited by pdfium-render's current API stability

### Platform Limitations

- **Pdfium Library**: Requires pdfium library to be available on the system
  - Linux: Usually available via system packages
  - macOS: May require manual installation
  - Windows: Requires pdfium.dll in PATH

### Workarounds

**Missing Pdfium**:
```rust
if !processor.is_available() {
    eprintln!("Pdfium not available, using fallback");
    // Use alternative processing or skip PDF
}
```

**Large Memory Requirements**:
```rust
// Use stricter memory settings
let config = PdfConfig {
    memory_settings: MemorySettings {
        max_memory_spike_bytes: 100 * 1024 * 1024,  // 100MB limit
        max_concurrent_operations: 1,
        ..Default::default()
    },
    ..Default::default()
};
```

**Timeout Issues**:
```rust
// Increase timeout for very large documents
let config = PdfConfig {
    timeout_seconds: 120,  // 2 minutes
    ..Default::default()
};
```

## Error Handling

The crate provides comprehensive error types:

```rust
use riptide_pdf::{PdfError, PdfResult};

match processor.process_pdf(&bytes, &config).await {
    Ok(result) => {
        println!("Success: {} pages", result.metadata.page_count);
    }
    Err(PdfError::FileTooLarge { size, max_size }) => {
        eprintln!("File too large: {} bytes (max: {})", size, max_size);
    }
    Err(PdfError::InvalidPdf { message }) => {
        eprintln!("Invalid PDF: {}", message);
    }
    Err(PdfError::MemoryLimit { used, limit }) => {
        eprintln!("Memory limit exceeded: {} > {}", used, limit);
    }
    Err(PdfError::ProcessingError { message }) => {
        eprintln!("Processing error: {}", message);
    }
    Err(PdfError::Timeout { seconds }) => {
        eprintln!("Timeout after {} seconds", seconds);
    }
}
```

## License

Apache-2.0

## Repository

https://github.com/your-org/eventmesh

## Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## Version

Current version: 0.1.0
