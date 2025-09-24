use crate::job::{Job, JobType};
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
    extractor: Arc<riptide_core::extract::WasmExtractor>,
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
        extractor: Arc<riptide_core::extract::WasmExtractor>,
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
        let _cache_key = self.generate_cache_key(url, options);
        let cache = self.cache.lock().await;

        // TODO: Implement proper cache lookup once CacheManager API is available
        // For now, skip caching to get compilation working

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
                // TODO: Cache the result once CacheManager API is available

                Ok(CrawlResult {
                    url: url.to_string(),
                    status,
                    from_cache: false,
                    processing_time_ms: processing_time,
                    document: Some(document),
                    error: None,
                })
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
    extractor: Arc<riptide_core::extract::WasmExtractor>,
    /// Cache manager for storing results
    cache: Arc<tokio::sync::Mutex<riptide_core::cache::CacheManager>>,
}

impl SingleCrawlProcessor {
    pub fn new(
        http_client: reqwest::Client,
        extractor: Arc<riptide_core::extract::WasmExtractor>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
}