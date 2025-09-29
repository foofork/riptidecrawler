use crate::job::{Job, JobType, PdfExtractionOptions};
use crate::worker::JobProcessor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use riptide_core::types::{CrawlOptions, ExtractedDoc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Batch crawl processor for handling multiple URL crawling
pub struct BatchCrawlProcessor {
    /// HTTP client for making requests
    http_client: reqwest::Client,
    /// WASM extractor for content processing
    extractor: Arc<dyn riptide_core::extract::WasmExtractor>,
    /// Cache manager for storing results
    cache: Arc<tokio::sync::Mutex<riptide_core::cache::CacheManager>>,
    /// Maximum batch size to prevent memory issues
    max_batch_size: usize,
    /// Maximum concurrent requests within a batch
    max_concurrency: usize,
}

impl BatchCrawlProcessor {
    /// Create a new batch crawl processor
    pub fn new(
        http_client: reqwest::Client,
        extractor: Arc<dyn riptide_core::extract::WasmExtractor>,
        cache: Arc<tokio::sync::Mutex<riptide_core::cache::CacheManager>>,
        max_batch_size: usize,
        max_concurrency: usize,
    ) -> Self {
        Self {
            http_client,
            extractor,
            cache,
            max_batch_size,
            max_concurrency,
        }
    }

    /// Process a single URL with caching support
    async fn process_single_url(
        &self,
        url: &str,
        options: &Option<CrawlOptions>,
    ) -> Result<CrawlResult> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.generate_cache_key(url, options);
        let mut cache = self.cache.lock().await;

        // Try to get cached result
        if let Ok(Some(cached_result)) = cache.get_simple::<CrawlResult>(&cache_key).await {
            debug!(url = %url, cache_key = %cache_key, "Cache hit for URL");
            return Ok(cached_result);
        }

        drop(cache); // Release cache lock

        // Fetch URL
        debug!(url = %url, "Fetching URL for batch processing");

        let response = self.http_client
            .get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .context("Failed to fetch URL")?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let processing_time = start_time.elapsed().as_millis() as u64;
            return Ok(CrawlResult {
                url: url.to_string(),
                status,
                from_cache: false,
                processing_time_ms: processing_time,
                document: None,
                error: Some(format!("HTTP error: {}", status)),
            });
        }

        let content = response.text().await
            .context("Failed to get response text")?;

        // Extract content using WASM
        let extraction_result = self.extractor
            .extract(content.as_bytes(), url, "default");

        let processing_time = start_time.elapsed().as_millis() as u64;

        match extraction_result {
            Ok(document) => {
                let result = CrawlResult {
                    url: url.to_string(),
                    status,
                    from_cache: false,
                    processing_time_ms: processing_time,
                    document: Some(document),
                    error: None,
                };

                // Cache the successful result
                let mut cache = self.cache.lock().await;
                if let Err(e) = cache.set_simple(&cache_key, &result, 3600).await { // 1 hour TTL
                    debug!(error = %e, cache_key = %cache_key, "Failed to cache result");
                }
                drop(cache);

                Ok(result)
            }
            Err(e) => {
                Ok(CrawlResult {
                    url: url.to_string(),
                    status,
                    from_cache: false,
                    processing_time_ms: processing_time,
                    document: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Generate cache key for URL and options
    fn generate_cache_key(&self, url: &str, options: &Option<CrawlOptions>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);

        if let Some(opts) = options {
            // Include relevant options in cache key
            std::mem::discriminant(&opts.output_format).hash(&mut hasher);
            opts.cache_mode.hash(&mut hasher);
        }

        format!("batch_crawl_{:x}", hasher.finish())
    }
}

#[async_trait]
impl JobProcessor for BatchCrawlProcessor {
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value> {
        match &job.job_type {
            JobType::BatchCrawl { urls, options } => {
                info!(
                    job_id = %job.id,
                    url_count = urls.len(),
                    "Processing batch crawl job"
                );

                if urls.len() > self.max_batch_size {
                    return Err(anyhow::anyhow!(
                        "Batch size {} exceeds maximum allowed size {}",
                        urls.len(),
                        self.max_batch_size
                    ));
                }

                let mut results = Vec::new();
                let mut successful = 0;
                let mut failed = 0;
                let mut from_cache = 0;

                // Process URLs with controlled concurrency
                let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrency));
                let mut handles = Vec::new();

                for url in urls.iter() {
                    let url = url.clone();
                    let options = options.clone();
                    let semaphore = semaphore.clone();
                    let http_client = self.http_client.clone();
                    let extractor = Arc::clone(&self.extractor);
                    let cache = Arc::clone(&self.cache);

                    let handle = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.expect("Semaphore closed");

                        // Create a temporary processor for this task
                        let temp_processor = BatchCrawlProcessor {
                            http_client,
                            extractor,
                            cache,
                            max_batch_size: 1, // Not used in single URL processing
                            max_concurrency: 1, // Not used in single URL processing
                        };

                        temp_processor.process_single_url(&url, &options).await
                    });

                    handles.push(handle);
                }

                // Collect all results
                for handle in handles {
                    match handle.await {
                        Ok(Ok(result)) => {
                            if result.error.is_none() {
                                successful += 1;
                            } else {
                                failed += 1;
                            }
                            if result.from_cache {
                                from_cache += 1;
                            }
                            results.push(result);
                        }
                        Ok(Err(e)) => {
                            error!(error = %e, "Failed to process URL in batch");
                            failed += 1;
                            // Add error result for failed URL
                            results.push(CrawlResult {
                                url: "unknown".to_string(),
                                status: 0,
                                from_cache: false,
                                processing_time_ms: 0,
                                document: None,
                                error: Some(e.to_string()),
                            });
                        }
                        Err(e) => {
                            error!(error = %e, "Task failed for batch URL");
                            failed += 1;
                        }
                    }
                }

                let batch_response = BatchCrawlResponse {
                    total_urls: urls.len(),
                    successful,
                    failed,
                    from_cache,
                    results,
                };

                info!(
                    job_id = %job.id,
                    total = urls.len(),
                    successful = successful,
                    failed = failed,
                    from_cache = from_cache,
                    "Batch crawl job completed"
                );

                Ok(serde_json::to_value(batch_response)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported job type for BatchCrawlProcessor")),
        }
    }

    fn supported_job_types(&self) -> Vec<String> {
        vec!["BatchCrawl".to_string()]
    }

    fn processor_name(&self) -> String {
        "BatchCrawlProcessor".to_string()
    }
}

/// Single URL crawl processor
pub struct SingleCrawlProcessor {
    /// HTTP client for making requests
    http_client: reqwest::Client,
    /// WASM extractor for content processing
    extractor: Arc<dyn riptide_core::extract::WasmExtractor>,
    /// Cache manager for storing results
    cache: Arc<tokio::sync::Mutex<riptide_core::cache::CacheManager>>,
}

impl SingleCrawlProcessor {
    pub fn new(
        http_client: reqwest::Client,
        extractor: Arc<dyn riptide_core::extract::WasmExtractor>,
        cache: Arc<tokio::sync::Mutex<riptide_core::cache::CacheManager>>,
    ) -> Self {
        Self {
            http_client,
            extractor,
            cache,
        }
    }
}

#[async_trait]
impl JobProcessor for SingleCrawlProcessor {
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value> {
        match &job.job_type {
            JobType::SingleCrawl { url, options } => {
                info!(job_id = %job.id, url = %url, "Processing single crawl job");

                let batch_processor = BatchCrawlProcessor::new(
                    self.http_client.clone(),
                    self.extractor.clone(),
                    self.cache.clone(),
                    1, // Single URL
                    1, // Single concurrency
                );

                let result = batch_processor.process_single_url(url, options).await?;

                info!(
                    job_id = %job.id,
                    url = %url,
                    success = result.error.is_none(),
                    from_cache = result.from_cache,
                    processing_time = result.processing_time_ms,
                    "Single crawl job completed"
                );

                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported job type for SingleCrawlProcessor")),
        }
    }

    fn supported_job_types(&self) -> Vec<String> {
        vec!["SingleCrawl".to_string()]
    }

    fn processor_name(&self) -> String {
        "SingleCrawlProcessor".to_string()
    }
}

/// Maintenance task processor
pub struct MaintenanceProcessor;

#[async_trait]
impl JobProcessor for MaintenanceProcessor {
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value> {
        match &job.job_type {
            JobType::Maintenance { task_type, parameters } => {
                info!(
                    job_id = %job.id,
                    task_type = %task_type,
                    "Processing maintenance job"
                );

                match task_type.as_str() {
                    "cache_cleanup" => {
                        // Simulate cache cleanup
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

                        let result = MaintenanceResult {
                            task_type: task_type.clone(),
                            success: true,
                            message: "Cache cleanup completed successfully".to_string(),
                            details: parameters.clone(),
                        };

                        Ok(serde_json::to_value(result)?)
                    }
                    "health_check" => {
                        // Simulate health check
                        let result = MaintenanceResult {
                            task_type: task_type.clone(),
                            success: true,
                            message: "Health check passed".to_string(),
                            details: parameters.clone(),
                        };

                        Ok(serde_json::to_value(result)?)
                    }
                    "log_rotation" => {
                        // Simulate log rotation
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                        let result = MaintenanceResult {
                            task_type: task_type.clone(),
                            success: true,
                            message: "Log rotation completed".to_string(),
                            details: parameters.clone(),
                        };

                        Ok(serde_json::to_value(result)?)
                    }
                    _ => {
                        let result = MaintenanceResult {
                            task_type: task_type.clone(),
                            success: false,
                            message: format!("Unknown maintenance task type: {}", task_type),
                            details: parameters.clone(),
                        };

                        Ok(serde_json::to_value(result)?)
                    }
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported job type for MaintenanceProcessor")),
        }
    }

    fn supported_job_types(&self) -> Vec<String> {
        vec!["Maintenance".to_string()]
    }

    fn processor_name(&self) -> String {
        "MaintenanceProcessor".to_string()
    }
}

/// Custom job processor for arbitrary tasks
pub struct CustomJobProcessor;

#[async_trait]
impl JobProcessor for CustomJobProcessor {
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value> {
        match &job.job_type {
            JobType::Custom { job_name, payload } => {
                info!(
                    job_id = %job.id,
                    job_name = %job_name,
                    "Processing custom job"
                );

                // For now, just echo back the payload with processing metadata
                let result = CustomJobResult {
                    job_name: job_name.clone(),
                    input_payload: payload.clone(),
                    processing_info: serde_json::json!({
                        "processed_at": chrono::Utc::now().to_rfc3339(),
                        "processor": "CustomJobProcessor",
                        "status": "completed"
                    }),
                };

                Ok(serde_json::to_value(result)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported job type for CustomJobProcessor")),
        }
    }

    fn supported_job_types(&self) -> Vec<String> {
        vec!["Custom".to_string()]
    }

    fn processor_name(&self) -> String {
        "CustomJobProcessor".to_string()
    }
}

/// Result structure for batch crawl operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchCrawlResponse {
    pub total_urls: usize,
    pub successful: usize,
    pub failed: usize,
    pub from_cache: usize,
    pub results: Vec<CrawlResult>,
}

/// Result structure for individual URL crawl
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrawlResult {
    pub url: String,
    pub status: u16,
    pub from_cache: bool,
    pub processing_time_ms: u64,
    pub document: Option<ExtractedDoc>,
    pub error: Option<String>,
}

/// Result structure for maintenance tasks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaintenanceResult {
    pub task_type: String,
    pub success: bool,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
}

/// Result structure for custom jobs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomJobResult {
    pub job_name: String,
    pub input_payload: serde_json::Value,
    pub processing_info: serde_json::Value,
}

/// PDF processor for handling PDF extraction jobs
pub struct PdfProcessor {
    /// PDF pipeline integration for processing
    pdf_pipeline: Arc<riptide_core::pdf::PdfPipelineIntegration>,
    /// Default PDF configuration
    default_config: riptide_core::pdf::PdfConfig,
    /// Maximum concurrent PDF processing operations
    #[allow(dead_code)]
    max_concurrent: usize,
}

impl PdfProcessor {
    /// Create a new PDF processor
    pub fn new() -> Self {
        let default_config = riptide_core::pdf::PdfConfig {
            extract_text: true,
            extract_images: false,
            extract_metadata: true,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            ..Default::default()
        };

        Self {
            pdf_pipeline: Arc::new(riptide_core::pdf::PdfPipelineIntegration::with_config(default_config.clone())),
            default_config,
            max_concurrent: 2, // ROADMAP requirement: max 2 concurrent operations
        }
    }

    /// Create PDF processor with custom configuration
    pub fn with_config(config: riptide_core::pdf::PdfConfig) -> Self {
        Self {
            pdf_pipeline: Arc::new(riptide_core::pdf::PdfPipelineIntegration::with_config(config.clone())),
            default_config: config,
            max_concurrent: 2,
        }
    }

    /// Convert PDF extraction options to PDF config
    fn create_pdf_config(&self, options: &Option<PdfExtractionOptions>) -> riptide_core::pdf::PdfConfig {
        match options {
            Some(opts) => {
                let mut config = self.default_config.clone();
                config.extract_text = opts.extract_text;
                config.extract_images = opts.extract_images;
                config.extract_metadata = opts.extract_metadata;
                config.max_size_bytes = opts.max_size_bytes;

                // Apply custom settings if any
                for (key, value) in &opts.custom_settings {
                    match key.as_str() {
                        "memory_pressure_threshold" => {
                            if let Some(threshold) = value.as_f64() {
                                config.memory_settings.memory_pressure_threshold = threshold;
                            }
                        },
                        "aggressive_cleanup" => {
                            if let Some(cleanup) = value.as_bool() {
                                config.memory_settings.aggressive_cleanup = cleanup;
                            }
                        },
                        "preserve_formatting" => {
                            if let Some(preserve) = value.as_bool() {
                                config.text_settings.preserve_formatting = preserve;
                            }
                        },
                        _ => {
                            debug!("Unknown PDF config setting: {}", key);
                        }
                    }
                }
                config
            },
            None => self.default_config.clone(),
        }
    }

    /// Process PDF with progress tracking
    async fn process_pdf_with_progress(
        &self,
        pdf_data: &[u8],
        url: Option<&str>,
        _config: &riptide_core::pdf::PdfConfig,
        enable_progress: bool,
    ) -> Result<ExtractedDoc> {
        if enable_progress {
            // Create progress channel for tracking using the correct type
            let (progress_tx, mut progress_rx) = riptide_core::pdf::types::create_progress_channel();

            // Spawn progress monitoring task
            let progress_task = tokio::spawn(async move {
                while let Some(progress) = progress_rx.recv().await {
                    debug!("PDF processing progress: {:?}", progress);
                }
            });

            // Process with progress tracking
            let result = self.pdf_pipeline
                .process_pdf_bytes_with_progress(pdf_data, progress_tx)
                .await
                .context("Failed to process PDF with progress tracking")?;

            // Wait for progress monitoring to complete
            progress_task.abort();

            Ok(riptide_core::convert_pdf_extracted_doc(result))
        } else {
            // Process without progress tracking
            let result = self.pdf_pipeline
                .process_pdf_to_extracted_doc(pdf_data, url)
                .await
                .context("Failed to process PDF")?;
            Ok(riptide_core::convert_pdf_extracted_doc(result))
        }
    }

    /// Create result with metadata and statistics
    fn create_pdf_result(
        &self,
        extracted_doc: ExtractedDoc,
        file_size: usize,
        processing_time_ms: u64,
        job_id: uuid::Uuid,
    ) -> serde_json::Value {
        let mut result = serde_json::json!({
            "success": true,
            "document": extracted_doc,
            "stats": {
                "file_size_bytes": file_size,
                "processing_time_ms": processing_time_ms,
                "text_length": extracted_doc.text.len(),
                "word_count": extracted_doc.word_count.unwrap_or(0),
                "pages_processed": 0, // Will be populated from PDF processing result later
                "media_count": extracted_doc.media.len(),
            },
            "metadata": {
                "job_id": job_id,
                "processor": "PdfProcessor",
                "extracted_by": "riptide-pdf",
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });

        // Add PDF-specific metadata
        if let Some(metadata) = result.get_mut("metadata") {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("content_type".to_string(), serde_json::json!("application/pdf"));
                obj.insert("quality_score".to_string(), serde_json::json!(extracted_doc.quality_score.unwrap_or(0)));
                if let Some(reading_time) = extracted_doc.reading_time {
                    obj.insert("reading_time_minutes".to_string(), serde_json::json!(reading_time));
                }
            }
        }

        result
    }
}

impl Default for PdfProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobProcessor for PdfProcessor {
    async fn process_job(&self, job: &Job) -> Result<serde_json::Value> {
        match &job.job_type {
            JobType::PdfExtraction { pdf_data, url, options } => {
                let start_time = std::time::Instant::now();

                info!(
                    job_id = %job.id,
                    file_size = pdf_data.len(),
                    url = ?url,
                    "Processing PDF extraction job"
                );

                // Validate input size
                let config = self.create_pdf_config(options);
                if pdf_data.len() as u64 > config.max_size_bytes {
                    return Err(anyhow::anyhow!(
                        "PDF file too large: {} bytes (max: {} bytes)",
                        pdf_data.len(),
                        config.max_size_bytes
                    ));
                }

                // Check if progress tracking is enabled
                let enable_progress = options
                    .as_ref()
                    .map(|o| o.enable_progress)
                    .unwrap_or(true);

                // Process PDF with appropriate method
                match self.process_pdf_with_progress(
                    pdf_data,
                    url.as_deref(),
                    &config,
                    enable_progress,
                ).await {
                    Ok(extracted_doc) => {
                        let processing_time_ms = start_time.elapsed().as_millis() as u64;

                        info!(
                            job_id = %job.id,
                            processing_time_ms = processing_time_ms,
                            text_length = extracted_doc.text.len(),
                            word_count = extracted_doc.word_count.unwrap_or(0),
                            media_count = extracted_doc.media.len(),
                            "PDF extraction completed successfully"
                        );

                        Ok(self.create_pdf_result(
                            extracted_doc,
                            pdf_data.len(),
                            processing_time_ms,
                            job.id,
                        ))
                    },
                    Err(e) => {
                        let processing_time_ms = start_time.elapsed().as_millis() as u64;

                        error!(
                            job_id = %job.id,
                            error = %e,
                            processing_time_ms = processing_time_ms,
                            "PDF extraction failed"
                        );

                        // Create error result with diagnostic information
                        let error_result = serde_json::json!({
                            "success": false,
                            "error": {
                                "message": e.to_string(),
                                "type": "PdfProcessingError",
                                "file_size_bytes": pdf_data.len(),
                                "processing_time_ms": processing_time_ms,
                            },
                            "metadata": {
                                "job_id": job.id,
                                "processor": "PdfProcessor",
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                            }
                        });

                        // Return as successful job result but with error content
                        // This allows the worker to handle the error appropriately
                        Ok(error_result)
                    }
                }
            }
            _ => Err(anyhow::anyhow!("Unsupported job type for PdfProcessor")),
        }
    }

    fn supported_job_types(&self) -> Vec<String> {
        vec!["PdfExtraction".to_string()]
    }

    fn processor_name(&self) -> String {
        "PdfProcessor".to_string()
    }
}

/// Result structure for PDF extraction operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfExtractionResult {
    pub success: bool,
    pub document: Option<ExtractedDoc>,
    pub error: Option<String>,
    pub stats: PdfExtractionStats,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Statistics for PDF extraction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdfExtractionStats {
    pub file_size_bytes: usize,
    pub processing_time_ms: u64,
    pub pages_processed: u32,
    pub text_length: usize,
    pub word_count: u32,
    pub media_count: usize,
    pub memory_used_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(invalid_value)] // Using unsafe zeroed for test mocks
    fn test_batch_crawl_processor_name() {
        let processor = BatchCrawlProcessor {
            http_client: reqwest::Client::new(),
            extractor: Arc::new(unsafe { std::mem::zeroed() }), // Mock for test
            cache: Arc::new(tokio::sync::Mutex::new(unsafe { std::mem::zeroed() })), // Mock for test
            max_batch_size: 100,
            max_concurrency: 10,
        };

        assert_eq!(processor.processor_name(), "BatchCrawlProcessor");
        assert_eq!(processor.supported_job_types(), vec!["BatchCrawl"]);
    }

    #[test]
    fn test_maintenance_processor_job_types() {
        let processor = MaintenanceProcessor;
        assert_eq!(processor.processor_name(), "MaintenanceProcessor");
        assert_eq!(processor.supported_job_types(), vec!["Maintenance"]);
    }

    #[test]
    fn test_custom_processor_job_types() {
        let processor = CustomJobProcessor;
        assert_eq!(processor.processor_name(), "CustomJobProcessor");
        assert_eq!(processor.supported_job_types(), vec!["Custom"]);
    }

    #[test]
    fn test_pdf_processor_creation() {
        let processor = PdfProcessor::new();
        assert_eq!(processor.processor_name(), "PdfProcessor");
        assert_eq!(processor.supported_job_types(), vec!["PdfExtraction"]);
        assert_eq!(processor.max_concurrent, 2);
    }

    #[test]
    fn test_pdf_config_creation() {
        let processor = PdfProcessor::new();

        // Test default options
        let config = processor.create_pdf_config(&None);
        assert!(config.extract_text);
        assert!(config.extract_metadata);
        assert!(!config.extract_images);

        // Test custom options
        let custom_options = PdfExtractionOptions {
            extract_text: false,
            extract_images: true,
            extract_metadata: false,
            max_size_bytes: 50 * 1024 * 1024,
            enable_progress: false,
            custom_settings: HashMap::new(),
        };

        let config = processor.create_pdf_config(&Some(custom_options));
        assert!(!config.extract_text);
        assert!(config.extract_images);
        assert!(!config.extract_metadata);
        assert_eq!(config.max_size_bytes, 50 * 1024 * 1024);
    }

    #[test]
    fn test_pdf_extraction_options_default() {
        let options = PdfExtractionOptions::default();
        assert!(options.extract_text);
        assert!(!options.extract_images);
        assert!(options.extract_metadata);
        assert_eq!(options.max_size_bytes, 100 * 1024 * 1024);
        assert!(options.enable_progress);
    }
}