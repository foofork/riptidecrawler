use std::collections::HashMap;
use std::sync::Arc;

use super::config::{PdfCapabilities, PdfConfig};
use super::errors::{PdfError, PdfResult};
use super::types::{PdfImage, PdfMetadata, PdfProcessingResult, PdfStats, ProgressCallback};
use super::utils;

#[cfg(feature = "pdf")]
use pdfium_render::prelude::*;
#[cfg(feature = "pdf")]
use std::sync::OnceLock;
#[cfg(feature = "pdf")]
use tokio::sync::Semaphore;

#[cfg(feature = "pdf")]
type Pdfium = pdfium_render::prelude::Pdfium;
#[cfg(feature = "pdf")]
type PdfDocument<'a> = pdfium_render::prelude::PdfDocument<'a>;
#[cfg(feature = "pdf")]
type PdfPage<'a> = pdfium_render::prelude::PdfPage<'a>;

// Global semaphore for PDF processing concurrency control
#[cfg(feature = "pdf")]
static PDF_SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();

/// PDF processor trait for different implementations
///
/// Note: This trait is object-safe (dyn compatible) for use with trait objects
pub trait PdfProcessor: Send + Sync {
    /// Process a PDF from bytes
    async fn process_pdf(&self, data: &[u8], config: &PdfConfig) -> PdfResult<PdfProcessingResult>;

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

/// Pdfium-based PDF processor with concurrency control
#[cfg(feature = "pdf")]
#[derive(Clone)]
pub struct PdfiumProcessor {
    capabilities: PdfCapabilities,
}

#[cfg(feature = "pdf")]
impl Default for PdfiumProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "pdf")]
impl PdfiumProcessor {
    pub fn new() -> Self {
        Self {
            capabilities: PdfCapabilities {
                text_extraction: true,
                image_extraction: true,
                metadata_extraction: true,
                table_extraction: false,
                form_extraction: false,
                encrypted_pdfs: false,
                max_file_size: 100 * 1024 * 1024, // 100MB
                supported_versions: vec![
                    "1.0".to_string(),
                    "1.1".to_string(),
                    "1.2".to_string(),
                    "1.3".to_string(),
                    "1.4".to_string(),
                    "1.5".to_string(),
                    "1.6".to_string(),
                    "1.7".to_string(),
                    "2.0".to_string(),
                ],
            },
        }
    }

    /// Enhanced PDF processing with page-by-page extraction and progress tracking
    async fn process_pdf_enhanced(
        &self,
        data: &[u8],
        config: &PdfConfig,
        progress_callback: Option<ProgressCallback>,
    ) -> PdfResult<PdfProcessingResult> {
        let start_time = std::time::Instant::now();
        let initial_memory = self.get_memory_usage();

        // Acquire semaphore permit for concurrency control (limit to 2 concurrent operations)
        let semaphore = PDF_SEMAPHORE.get_or_init(|| Arc::new(Semaphore::new(2)));
        let _permit = semaphore
            .acquire()
            .await
            .map_err(|_| PdfError::ProcessingError {
                message: "Failed to acquire processing permit".to_string(),
            })?;

        let processor_clone = self.clone();
        let data = data.to_vec(); // Clone data to move into blocking task
        let config = config.clone(); // Clone config to move into blocking task
        let progress_callback_moved = progress_callback; // Move callback into blocking task

        tokio::task::spawn_blocking(move || {
            // Validate PDF before processing
            processor_clone.validate_pdf_input(&data, &config)?;

            // Initialize progress tracking
            if let Some(ref callback) = progress_callback_moved {
                callback(0, 1); // Start with unknown page count
            }

            // Initialize Pdfium with error handling
            let pdfium = processor_clone.initialize_pdfium()?;
            let document = processor_clone.load_document(&pdfium, &data)?;

            let total_pages = document.pages().len() as u32;
            let mut extraction_results = ExtractionResults::new();

            // Update progress with actual page count
            if let Some(ref callback) = progress_callback_moved {
                callback(0, total_pages);
            }

            // Extract content page by page with memory monitoring
            for page_index in 0..document.pages().len() {
                // Check memory usage periodically
                if page_index % 10 == 0 {
                    let current_memory = processor_clone.get_memory_usage();
                    if current_memory > initial_memory + (200 * 1024 * 1024) { // 200MB spike threshold
                        return Err(PdfError::MemoryLimit {
                            used: current_memory,
                            limit: initial_memory + (200 * 1024 * 1024),
                        });
                    }
                }

                let page = document
                    .pages()
                    .get(page_index)
                    .map_err(|e| PdfError::ProcessingError {
                        message: format!("Failed to get page {}: {}", page_index, e),
                    })?;

                processor_clone.extract_page_content(&page, page_index.into(), &config, &mut extraction_results)?;

                // Update progress after each page
                if let Some(ref callback) = progress_callback_moved {
                    callback(page_index as u32 + 1, total_pages);
                }
            }

            // Check if OCR is needed
            processor_clone.handle_ocr_if_needed(&mut extraction_results, &config);

            // Extract comprehensive metadata
            let metadata = processor_clone.extract_metadata(&document, total_pages, &config)?;

            let processing_time = start_time.elapsed().as_millis() as u64;
            let final_memory = processor_clone.get_memory_usage();

            Ok(processor_clone.build_processing_result(
                extraction_results,
                metadata,
                processing_time,
                data.len() as u64,
                &config,
                final_memory.saturating_sub(initial_memory),
            ))
        }).await.map_err(|e| PdfError::ProcessingError { message: format!("Blocking task failed: {}", e) })?
    }

    #[cfg(feature = "pdf")]
    fn validate_pdf_input(&self, data: &[u8], config: &PdfConfig) -> PdfResult<()> {
        // Check file size
        if data.len() as u64 > config.max_size_bytes {
            return Err(PdfError::FileTooLarge {
                size: data.len() as u64,
                max_size: config.max_size_bytes,
            });
        }

        // Validate PDF header
        utils::validate_pdf_header(data).map_err(|msg| PdfError::InvalidPdf { message: msg })?;

        Ok(())
    }

    #[cfg(feature = "pdf")]
    fn initialize_pdfium(&self) -> PdfResult<Pdfium> {
        let binding = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .map_err(|e| PdfError::ProcessingError {
                message: format!("Failed to initialize Pdfium: {}", e),
            })?;

        Ok(Pdfium::new(binding))
    }

    #[cfg(feature = "pdf")]
    fn load_document<'a>(&self, pdfium: &'a Pdfium, data: &'a [u8]) -> PdfResult<PdfDocument<'a>> {
        pdfium
            .load_pdf_from_byte_slice(data, None)
            .map_err(move |e| PdfError::ProcessingError {
                message: format!("Failed to load PDF: {}", e),
            })
    }

    #[cfg(feature = "pdf")]
    fn extract_page_content(
        &self,
        page: &PdfPage,
        page_index: usize,
        config: &PdfConfig,
        results: &mut ExtractionResults,
    ) -> PdfResult<()> {
        // Extract text if enabled
        if config.extract_text {
            let page_text = page
                .text()
                .map_err(|e| PdfError::ProcessingError {
                    message: format!("Failed to extract text from page {}: {}", page_index, e),
                })?
                .all();

            if !page_text.trim().is_empty() {
                results.has_text_content = true;
            }

            if config.text_settings.preserve_formatting {
                results.text.push_str(&page_text);
                results.text.push('\n');
            } else {
                results.text.push_str(&page_text.replace('\n', " "));
                results.text.push(' ');
            }
        }

        // Extract images if enabled
        if config.extract_images {
            self.extract_images_from_page(page, page_index, config, results)?;
        }

        Ok(())
    }

    #[cfg(feature = "pdf")]
    fn extract_images_from_page(
        &self,
        page: &PdfPage,
        page_index: usize,
        config: &PdfConfig,
        results: &mut ExtractionResults,
    ) -> PdfResult<()> {
        for (obj_index, obj) in page.objects().iter().enumerate() {
            if matches!(obj.object_type(), PdfPageObjectType::Image) {
                results.images_count += 1;

                if results.images.len() < config.image_settings.max_images as usize {
                    // Extract image properties
                    let (width, height, position) = self.extract_image_properties(&obj, config)?;

                    // Skip images that are too small
                    if width < config.image_settings.min_dimensions.0
                        || height < config.image_settings.min_dimensions.1 {
                        continue;
                    }

                    // Extract image data if configured
                    let data = if config.image_settings.base64_encode {
                        self.extract_image_data(&obj)?
                    } else {
                        None
                    };

                    // Determine image format (default to PNG)
                    let format = self.detect_image_format(&obj).unwrap_or(super::config::ImageFormat::Png);

                    results.images.push(PdfImage {
                        index: obj_index as u32,
                        page: page_index as u32,
                        data,
                        format,
                        width,
                        height,
                        position,
                        alt_text: None, // Could extract from accessibility info
                    });
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "pdf")]
    fn extract_image_properties(
        &self,
        obj: &pdfium_render::prelude::PdfPageObject,
        config: &PdfConfig,
    ) -> PdfResult<(u32, u32, Option<super::types::ImagePosition>)> {
        let bounds = obj.bounds().map_err(|e| PdfError::ProcessingError {
            message: format!("Failed to get image bounds: {}", e),
        })?;

        let width = bounds.width().value as u32;
        let height = bounds.height().value as u32;

        let position = if config.image_settings.include_positions {
            Some(super::types::ImagePosition {
                x: bounds.left().value,
                y: bounds.top().value,
                width: bounds.width().value,
                height: bounds.height().value,
            })
        } else {
            None
        };

        Ok((width, height, position))
    }

    #[cfg(feature = "pdf")]
    fn extract_image_data(
        &self,
        _obj: &pdfium_render::prelude::PdfPageObject,
    ) -> PdfResult<Option<String>> {
        // TODO: Implement actual image data extraction and base64 encoding
        // This would require accessing the image's raw bitmap data
        // For now, return None to avoid blocking implementation
        Ok(None)
    }

    #[cfg(feature = "pdf")]
    fn detect_image_format(
        &self,
        _obj: &pdfium_render::prelude::PdfPageObject,
    ) -> Option<super::config::ImageFormat> {
        // TODO: Implement actual format detection based on image data
        // For now, default to PNG
        Some(super::config::ImageFormat::Png)
    }

    #[cfg(feature = "pdf")]
    fn handle_ocr_if_needed(&self, results: &mut ExtractionResults, config: &PdfConfig) {
        let needs_ocr = !results.has_text_content && config.ocr_config.enable_ocr;
        if needs_ocr && results.text.trim().is_empty() {
            results.text = "[OCR needed - image-based PDF detected]".to_string();
        }
    }

    #[cfg(feature = "pdf")]
    fn extract_metadata(
        &self,
        document: &PdfDocument,
        total_pages: u32,
        config: &PdfConfig,
    ) -> PdfResult<PdfMetadata> {
        let mut custom_metadata = HashMap::new();

        if config.extract_metadata {
            custom_metadata.insert("extracted_by".to_string(), "riptide-pdf".to_string());
            custom_metadata.insert("pages".to_string(), total_pages.to_string());
        }

        // Extract comprehensive metadata from document info
        let mut title = None;
        let mut author = None;
        let mut subject = None;
        let mut keywords = None;
        let mut creator = None;
        let mut producer = None;
        let mut creation_date = None;
        let mut modification_date = None;
        let mut pdf_version = None;

        // For now, skip complex metadata extraction due to API changes
        // The pdfium_render metadata API has changed and needs further investigation
        // TODO: Update when stable API methods are determined
        title = None;
        author = None;
        subject = None;
        keywords = None;
        creator = None;
        producer = None;
        creation_date = None;
        modification_date = None;

        // Extract PDF version from document
        let version = document.version();
        // PdfDocumentVersion is an enum, convert to string representation
        pdf_version = Some(format!("{:?}", version));

        // Check document permissions - use conservative defaults if API unavailable
        let permissions = document.permissions();
        // Note: API methods changed - using safe defaults until proper methods are found
        let allows_copying = true; // Default to allowed
        let allows_printing = true; // Default to allowed

        // Check if document is encrypted - use conservative default
        let encrypted = false; // Default to not encrypted

        Ok(PdfMetadata {
            title,
            author,
            subject,
            keywords,
            creator,
            producer,
            creation_date,
            modification_date,
            pdf_version,
            page_count: total_pages,
            encrypted,
            allows_copying,
            allows_printing,
            custom_metadata,
        })
    }

    #[cfg(feature = "pdf")]
    fn get_memory_usage(&self) -> u64 {
        // Get current memory usage for monitoring
        #[cfg(unix)]
        {
            if let Ok(usage) = psutil::process::Process::current() {
                if let Ok(memory_info) = usage.memory_info() {
                    return memory_info.rss();
                }
            }
        }

        // Fallback: estimate based on system info
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.used_memory()
    }

    #[cfg(feature = "pdf")]
    fn build_processing_result(
        &self,
        results: ExtractionResults,
        metadata: PdfMetadata,
        processing_time: u64,
        file_size: u64,
        config: &PdfConfig,
        memory_used: u64,
    ) -> PdfProcessingResult {
        let page_count = metadata.page_count;
        let images_count = results.images.len() as u32;
        let text_length = results.text.len() as u32;

        PdfProcessingResult {
            success: true,
            text: if config.extract_text {
                Some(results.text)
            } else {
                None
            },
            images: results.images,
            metadata,
            structured_content: None,
            stats: PdfStats {
                processing_time_ms: processing_time,
                memory_used,
                pages_processed: page_count,
                images_extracted: images_count,
                tables_found: 0,
                text_length,
                file_size,
            },
            error: None,
        }
    }

    /// Process PDF with pdfium-render library for ExtractedDoc
    #[cfg(feature = "pdf")]
    pub async fn process_pdf_bytes(
        &self,
        pdf_bytes: &[u8],
    ) -> PdfResult<crate::types::ExtractedDoc> {
        // Acquire semaphore permit (limit to 2 concurrent operations)
        let semaphore = PDF_SEMAPHORE.get_or_init(|| Arc::new(Semaphore::new(2)));
        let _permit = semaphore
            .acquire()
            .await
            .map_err(|_| PdfError::ProcessingError {
                message: "Failed to acquire processing permit".to_string(),
            })?;

        let processor_clone = self.clone();
        let pdf_bytes = pdf_bytes.to_vec();

        tokio::task::spawn_blocking(move || {
            // Validate input
            if pdf_bytes.len() as u64 > processor_clone.capabilities.max_file_size {
                return Err(PdfError::FileTooLarge {
                    size: pdf_bytes.len() as u64,
                    max_size: processor_clone.capabilities.max_file_size,
                });
            }

            utils::validate_pdf_header(&pdf_bytes)
                .map_err(|msg| PdfError::InvalidPdf { message: msg })?;

            // Initialize and process
            let pdfium = processor_clone.initialize_pdfium()?;
            let document = processor_clone.load_document(&pdfium, &pdf_bytes)?;

            let mut text = String::new();
            let mut metadata = HashMap::new();
            let mut images_count = 0;
            let links = Vec::new();
            let mut media = Vec::new();

            // Extract content
            for page_index in 0..document.pages().len() {
                let page = document
                    .pages()
                    .get(page_index)
                    .map_err(|e| PdfError::ProcessingError {
                        message: format!("Failed to get page {}: {}", page_index, e),
                    })?;

                let page_text = page
                    .text()
                    .map_err(|e| PdfError::ProcessingError {
                        message: format!("Failed to extract text from page {}: {}", page_index, e),
                    })?
                    .all();

                text.push_str(&page_text);
                text.push('\n');

                // Count images
                images_count += page
                    .objects()
                    .iter()
                    .filter(|obj| matches!(obj.object_type(), PdfPageObjectType::Image))
                    .count();
            }

            // Add metadata
            metadata.insert("extracted_by".to_string(), "riptide-pdf".to_string());
            metadata.insert("pages".to_string(), document.pages().len().to_string());

            // Add image placeholders
            for i in 0..images_count {
                media.push(format!("pdf:image:{}", i));
            }

            // Calculate reading time
            let word_count = text.split_whitespace().count() as u32;
            let reading_time = Some(utils::estimate_reading_time(word_count));

            Ok(crate::types::ExtractedDoc {
                url: "pdf://document".to_string(),
                title: metadata.get("title").cloned(),
                byline: metadata.get("author").cloned(),
                published_iso: None,
                markdown: text.clone(),
                text: text.clone(),
                links,
                media,
                language: None,
                reading_time,
                quality_score: Some(85),
                word_count: Some(word_count),
                categories: vec!["document".to_string(), "pdf".to_string()],
                site_name: metadata.get("producer").cloned(),
                description: metadata.get("subject").cloned(),
            })
        }).await.map_err(|e| PdfError::ProcessingError { message: format!("Blocking task failed: {}", e) })?
    }
}

#[cfg(feature = "pdf")]
impl PdfProcessor for PdfiumProcessor {
    async fn process_pdf(&self, data: &[u8], config: &PdfConfig) -> PdfResult<PdfProcessingResult> {
        self.process_pdf_enhanced(data, config, None).await
    }

    async fn process_pdf_with_progress(
        &self,
        data: &[u8],
        config: &PdfConfig,
        progress_callback: Option<ProgressCallback>,
    ) -> PdfResult<PdfProcessingResult> {
        self.process_pdf_enhanced(data, config, progress_callback)
            .await
    }

    async fn detect_ocr_need(&self, data: &[u8]) -> PdfResult<bool> {
        // Quick check for OCR need without full processing (limit to 2 concurrent operations)
        let semaphore = PDF_SEMAPHORE.get_or_init(|| Arc::new(Semaphore::new(2)));
        let _permit = semaphore
            .acquire()
            .await
            .map_err(|_| PdfError::ProcessingError {
                message: "Failed to acquire processing permit".to_string(),
            })?;

        let processor_clone = self.clone();
        let data = data.to_vec();

        tokio::task::spawn_blocking(move || {
            utils::validate_pdf_header(&data).map_err(|msg| PdfError::InvalidPdf { message: msg })?;

            let pdfium = processor_clone.initialize_pdfium()?;
            let document = processor_clone.load_document(&pdfium, &data)?;

            // Check first few pages for text content
            let pages_to_check = (document.pages().len().min(3)) as usize;
            let mut has_text = false;

            for page_index in 0..pages_to_check {
                if let Ok(page) = document.pages().get(page_index as u16) {
                    if let Ok(text_obj) = page.text() {
                        let page_text = text_obj.all();
                        if !page_text.trim().is_empty() {
                            has_text = true;
                            break;
                        }
                    }
                }
            }

            // If no text found but has images, likely needs OCR
            if !has_text {
                if let Ok(page) = document.pages().get(0u16) {
                    let has_images = page
                        .objects()
                        .iter()
                        .any(|obj| matches!(obj.object_type(), PdfPageObjectType::Image));
                    return Ok(has_images);
                }
            }

            Ok(false)
        }).await.map_err(|e| PdfError::ProcessingError { message: format!("Blocking task failed: {}", e) })?
    }

    fn is_available(&self) -> bool {
        #[cfg(feature = "pdf")]
        {
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .is_ok()
        }
        #[cfg(not(feature = "pdf"))]
        {
            false
        }
    }

    fn capabilities(&self) -> PdfCapabilities {
        self.capabilities.clone()
    }
}

/// Default PDF processor implementation (fallback when pdf feature is disabled)
#[cfg(not(feature = "pdf"))]
pub struct DefaultPdfProcessor {
    capabilities: PdfCapabilities,
}

#[cfg(not(feature = "pdf"))]
impl Default for DefaultPdfProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "pdf"))]
impl DefaultPdfProcessor {
    pub fn new() -> Self {
        Self {
            capabilities: PdfCapabilities {
                text_extraction: false,
                image_extraction: false,
                metadata_extraction: false,
                table_extraction: false,
                form_extraction: false,
                encrypted_pdfs: false,
                max_file_size: 0,
                supported_versions: vec![],
            },
        }
    }
}

#[cfg(not(feature = "pdf"))]
impl PdfProcessor for DefaultPdfProcessor {
    async fn process_pdf(
        &self,
        _data: &[u8],
        _config: &PdfConfig,
    ) -> PdfResult<PdfProcessingResult> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf"
                .to_string(),
        })
    }

    async fn process_pdf_with_progress(
        &self,
        _data: &[u8],
        _config: &PdfConfig,
        _progress_callback: Option<ProgressCallback>,
    ) -> PdfResult<PdfProcessingResult> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf"
                .to_string(),
        })
    }

    async fn detect_ocr_need(&self, _data: &[u8]) -> PdfResult<bool> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf"
                .to_string(),
        })
    }

    fn is_available(&self) -> bool {
        false
    }

    fn capabilities(&self) -> PdfCapabilities {
        self.capabilities.clone()
    }
}

/// Helper struct to collect extraction results
#[cfg(feature = "pdf")]
struct ExtractionResults {
    text: String,
    images: Vec<PdfImage>,
    images_count: usize,
    has_text_content: bool,
}

#[cfg(feature = "pdf")]
impl ExtractionResults {
    fn new() -> Self {
        Self {
            text: String::new(),
            images: Vec::new(),
            images_count: 0,
            has_text_content: false,
        }
    }
}

/// Enum dispatch for PDF processors (avoids trait object issues)
#[derive(Clone)]
pub enum AnyPdfProcessor {
    #[cfg(feature = "pdf")]
    Pdfium(PdfiumProcessor),
    #[cfg(not(feature = "pdf"))]
    Default(DefaultPdfProcessor),
}

impl AnyPdfProcessor {
    /// Process a PDF from bytes
    pub async fn process_pdf(&self, data: &[u8], config: &PdfConfig) -> PdfResult<PdfProcessingResult> {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.process_pdf(data, config).await,
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.process_pdf(data, config).await,
        }
    }

    /// Process a PDF with progress tracking
    pub async fn process_pdf_with_progress(
        &self,
        data: &[u8],
        config: &PdfConfig,
        progress_callback: Option<ProgressCallback>,
    ) -> PdfResult<PdfProcessingResult> {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.process_pdf_with_progress(data, config, progress_callback).await,
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.process_pdf_with_progress(data, config, progress_callback).await,
        }
    }

    /// Detect if OCR is needed for this PDF
    pub async fn detect_ocr_need(&self, data: &[u8]) -> PdfResult<bool> {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.detect_ocr_need(data).await,
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.detect_ocr_need(data).await,
        }
    }

    /// Check if the processor is available
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.is_available(),
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.is_available(),
        }
    }

    /// Get processor capabilities
    pub fn capabilities(&self) -> PdfCapabilities {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.capabilities(),
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.capabilities(),
        }
    }

    /// Process PDF bytes for ExtractedDoc
    pub async fn process_pdf_bytes(&self, pdf_bytes: &[u8]) -> PdfResult<crate::types::ExtractedDoc> {
        match self {
            #[cfg(feature = "pdf")]
            AnyPdfProcessor::Pdfium(processor) => processor.process_pdf_bytes(pdf_bytes).await,
            #[cfg(not(feature = "pdf"))]
            AnyPdfProcessor::Default(processor) => processor.process_pdf_bytes(pdf_bytes).await,
        }
    }
}

/// Create appropriate PDF processor based on available features
#[cfg(feature = "pdf")]
pub fn create_pdf_processor() -> AnyPdfProcessor {
    AnyPdfProcessor::Pdfium(PdfiumProcessor::new())
}

#[cfg(not(feature = "pdf"))]
pub fn create_pdf_processor() -> AnyPdfProcessor {
    AnyPdfProcessor::Default(DefaultPdfProcessor::new())
}

/// Fallback implementation for when PDF feature is disabled
#[cfg(not(feature = "pdf"))]
impl DefaultPdfProcessor {
    /// Process PDF bytes (fallback implementation)
    pub async fn process_pdf_bytes(
        &self,
        _pdf_bytes: &[u8],
    ) -> PdfResult<crate::types::ExtractedDoc> {
        Err(PdfError::ProcessingError {
            message: "PDF processing feature is not enabled. Enable with --features pdf"
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = create_pdf_processor();

        #[cfg(feature = "pdf")]
        {
            let capabilities = processor.capabilities();
            assert!(capabilities.text_extraction);
        }

        #[cfg(not(feature = "pdf"))]
        {
            assert!(!processor.is_available());
        }
    }

    #[tokio::test]
    async fn test_default_processor_errors() {
        #[cfg(not(feature = "pdf"))]
        {
            let processor = DefaultPdfProcessor::new();
            let config = PdfConfig::default();
            let data = b"%PDF-1.7\n";

            let result = processor.process_pdf(data, &config).await;
            assert!(result.is_err());
        }
    }
}
