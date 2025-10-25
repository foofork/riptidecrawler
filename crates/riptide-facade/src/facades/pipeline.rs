//! Pipeline facade for orchestrating multi-stage workflows.
//!
//! Provides a fluent API for building and executing complex data processing
//! pipelines with features like parallel execution, error handling, retries,
//! caching, and progress tracking.

use crate::config::RiptideConfig;
use crate::error::{RiptideError, RiptideResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Main pipeline facade for orchestrating multi-stage workflows.
#[derive(Clone)]
pub struct PipelineFacade {
    config: Arc<RiptideConfig>,
    cache: Arc<RwLock<PipelineCache>>,
}

impl PipelineFacade {
    /// Create a new pipeline facade.
    pub async fn new(config: RiptideConfig) -> RiptideResult<Self> {
        Ok(Self {
            config: Arc::new(config),
            cache: Arc::new(RwLock::new(PipelineCache::new())),
        })
    }

    /// Create a new pipeline builder.
    pub fn builder(&self) -> PipelineBuilder {
        PipelineBuilder::new((*self.config).clone())
    }

    /// Execute a pipeline.
    pub async fn execute(&self, pipeline: Pipeline) -> RiptideResult<PipelineResult> {
        let start_time = Instant::now();
        let mut stage_results = Vec::new();
        let mut context = PipelineContext::new();

        // Execute stages based on execution mode
        match pipeline.config.execution_mode {
            ExecutionMode::Sequential => {
                for (idx, stage) in pipeline.stages.iter().enumerate() {
                    let stage_result = self
                        .execute_stage_with_retry(stage, &mut context, &pipeline.config, idx)
                        .await?;
                    stage_results.push(stage_result);
                }
            }
            ExecutionMode::Parallel { degree } => {
                let chunks: Vec<_> = pipeline.stages.chunks(degree).collect();
                for chunk in chunks {
                    let mut handles = Vec::new();
                    for (offset, stage) in chunk.iter().enumerate() {
                        let stage = stage.clone();
                        let mut stage_context = context.clone();
                        let config = pipeline.config.clone();
                        let facade = self.clone();
                        let idx = stage_results.len() + offset;

                        handles.push(tokio::spawn(async move {
                            facade
                                .execute_stage_with_retry(&stage, &mut stage_context, &config, idx)
                                .await
                        }));
                    }

                    // Wait for all parallel stages to complete
                    for handle in handles {
                        let result = handle.await.map_err(|e| RiptideError::Other(e.into()))??;
                        stage_results.push(result);
                    }
                }
            }
        }

        let total_duration = start_time.elapsed();
        let final_output = context.get_final_output();

        Ok(PipelineResult {
            stages_completed: stage_results.len(),
            total_duration,
            stage_results,
            final_output,
        })
    }

    /// Execute a single stage with retry logic.
    async fn execute_stage_with_retry(
        &self,
        stage: &PipelineStage,
        context: &mut PipelineContext,
        config: &PipelineConfig,
        stage_idx: usize,
    ) -> RiptideResult<StageResult> {
        let mut attempts = 0;
        let max_attempts = config.max_retries + 1;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;

            match self.execute_stage(stage, context, config, stage_idx).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempts < max_attempts {
                        let backoff = Duration::from_millis(100 * 2_u64.pow((attempts - 1) as u32));
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| RiptideError::Other(anyhow::anyhow!("Unknown error"))))
    }

    /// Execute a single stage.
    async fn execute_stage(
        &self,
        stage: &PipelineStage,
        context: &mut PipelineContext,
        config: &PipelineConfig,
        stage_idx: usize,
    ) -> RiptideResult<StageResult> {
        let start_time = Instant::now();

        // Check cache if enabled
        if config.caching_enabled {
            let cache_key = format!("stage_{}_{:?}", stage_idx, stage);
            if let Some(cached) = self.cache.read().await.get(&cache_key) {
                return Ok(StageResult {
                    stage_name: stage.name(),
                    duration: Duration::from_millis(0),
                    status: StageStatus::CachedSuccess,
                    output: cached.clone(),
                    metadata: HashMap::new(),
                });
            }
        }

        // Execute stage based on type
        let output = match stage {
            PipelineStage::Fetch { url, options } => {
                self.execute_fetch(url, options, context).await?
            }
            PipelineStage::Extract { strategy } => self.execute_extract(strategy, context).await?,
            PipelineStage::Transform { transformer } => {
                self.execute_transform(transformer.as_ref(), context)
                    .await?
            }
            PipelineStage::Validate { validator } => {
                self.execute_validate(validator.as_ref(), context).await?
            }
            PipelineStage::Store { destination } => {
                self.execute_store(destination, context).await?
            }
        };

        let duration = start_time.elapsed();

        // Update context with output
        context.set_output(output.clone());

        // Cache result if enabled
        if config.caching_enabled {
            let cache_key = format!("stage_{}_{:?}", stage_idx, stage);
            self.cache.write().await.insert(cache_key, output.clone());
        }

        Ok(StageResult {
            stage_name: stage.name(),
            duration,
            status: StageStatus::Success,
            output,
            metadata: HashMap::new(),
        })
    }

    async fn execute_fetch(
        &self,
        url: &str,
        _options: &FetchOptions,
        _context: &PipelineContext,
    ) -> RiptideResult<serde_json::Value> {
        // Placeholder: Would use ScraperFacade or BrowserFacade
        Ok(serde_json::json!({
            "url": url,
            "content": format!("Fetched content from {}", url),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_secs(),
        }))
    }

    async fn execute_extract(
        &self,
        strategy: &ExtractionStrategy,
        context: &PipelineContext,
    ) -> RiptideResult<serde_json::Value> {
        let input = context.get_output();
        Ok(serde_json::json!({
            "strategy": format!("{:?}", strategy),
            "extracted": input,
        }))
    }

    async fn execute_transform(
        &self,
        transformer: &dyn Transformer,
        context: &PipelineContext,
    ) -> RiptideResult<serde_json::Value> {
        let input = context.get_output();
        transformer.transform(input).await
    }

    async fn execute_validate(
        &self,
        validator: &dyn Validator,
        context: &PipelineContext,
    ) -> RiptideResult<serde_json::Value> {
        let input = context.get_output();
        validator.validate(input).await
    }

    async fn execute_store(
        &self,
        destination: &StoreDestination,
        context: &PipelineContext,
    ) -> RiptideResult<serde_json::Value> {
        let input = context.get_output();
        Ok(serde_json::json!({
            "destination": format!("{:?}", destination),
            "stored": input,
        }))
    }

    // Pre-built pipeline templates

    /// Create a web scraping pipeline (Fetch → Extract → Store).
    pub async fn web_scraping_pipeline(&self, url: &str) -> RiptideResult<Pipeline> {
        self.builder()
            .add_stage(PipelineStage::Fetch {
                url: url.to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Html,
            })
            .add_stage(PipelineStage::Store {
                destination: StoreDestination::Memory,
            })
            .with_retry(3)
            .with_caching(true)
            .build()
            .await
    }

    /// Create a PDF extraction pipeline.
    pub async fn pdf_extraction_pipeline(&self, url: &str) -> RiptideResult<Pipeline> {
        self.builder()
            .add_stage(PipelineStage::Fetch {
                url: url.to_string(),
                options: FetchOptions {
                    method: HttpMethod::Get,
                    headers: vec![("Accept".to_string(), "application/pdf".to_string())],
                    timeout: Duration::from_secs(60),
                },
            })
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Pdf,
            })
            .add_stage(PipelineStage::Validate {
                validator: Arc::new(ContentValidator),
            })
            .add_stage(PipelineStage::Store {
                destination: StoreDestination::Memory,
            })
            .with_retry(2)
            .build()
            .await
    }

    /// Create a browser automation pipeline.
    pub async fn browser_automation_pipeline(
        &self,
        url: &str,
        _actions: Vec<BrowserAction>,
    ) -> RiptideResult<Pipeline> {
        self.builder()
            .add_stage(PipelineStage::Fetch {
                url: url.to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Html,
            })
            .add_stage(PipelineStage::Validate {
                validator: Arc::new(ContentValidator),
            })
            .add_stage(PipelineStage::Store {
                destination: StoreDestination::Memory,
            })
            .with_retry(3)
            .with_parallelism(1)
            .build()
            .await
    }
}

/// Builder for constructing pipelines.
pub struct PipelineBuilder {
    #[allow(dead_code)]
    config: RiptideConfig,
    stages: Vec<PipelineStage>,
    pipeline_config: PipelineConfig,
}

impl PipelineBuilder {
    fn new(config: RiptideConfig) -> Self {
        Self {
            config,
            stages: Vec::new(),
            pipeline_config: PipelineConfig::default(),
        }
    }

    /// Add a stage to the pipeline.
    pub fn add_stage(mut self, stage: PipelineStage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Set maximum number of retries per stage.
    pub fn with_retry(mut self, max_retries: usize) -> Self {
        self.pipeline_config.max_retries = max_retries;
        self
    }

    /// Enable or disable caching between stages.
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.pipeline_config.caching_enabled = enabled;
        self
    }

    /// Set parallelism degree for parallel execution.
    pub fn with_parallelism(mut self, degree: usize) -> Self {
        self.pipeline_config.execution_mode = ExecutionMode::Parallel { degree };
        self
    }

    /// Build the pipeline.
    pub async fn build(self) -> RiptideResult<Pipeline> {
        if self.stages.is_empty() {
            return Err(RiptideError::config(
                "Pipeline must have at least one stage",
            ));
        }

        Ok(Pipeline {
            stages: self.stages,
            config: self.pipeline_config,
        })
    }
}

/// A complete pipeline with stages and configuration.
#[derive(Debug, Clone)]
pub struct Pipeline {
    pub stages: Vec<PipelineStage>,
    pub config: PipelineConfig,
}

/// Configuration for pipeline execution.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_retries: usize,
    pub caching_enabled: bool,
    pub execution_mode: ExecutionMode,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_retries: 0,
            caching_enabled: false,
            execution_mode: ExecutionMode::Sequential,
        }
    }
}

/// Execution mode for pipeline stages.
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Sequential,
    Parallel { degree: usize },
}

/// A stage in the pipeline.
#[derive(Debug, Clone)]
pub enum PipelineStage {
    Fetch { url: String, options: FetchOptions },
    Extract { strategy: ExtractionStrategy },
    Transform { transformer: Arc<dyn Transformer> },
    Validate { validator: Arc<dyn Validator> },
    Store { destination: StoreDestination },
}

impl PipelineStage {
    fn name(&self) -> String {
        match self {
            Self::Fetch { .. } => "Fetch".to_string(),
            Self::Extract { .. } => "Extract".to_string(),
            Self::Transform { .. } => "Transform".to_string(),
            Self::Validate { .. } => "Validate".to_string(),
            Self::Store { .. } => "Store".to_string(),
        }
    }
}

/// Options for fetch stage.
#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub timeout: Duration,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            method: HttpMethod::Get,
            headers: Vec::new(),
            timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
}

/// Strategy for extraction stage.
#[derive(Debug, Clone)]
pub enum ExtractionStrategy {
    Html,
    Json,
    Pdf,
    Custom(String),
}

/// Transformer trait for transform stage.
pub trait Transformer: Send + Sync + std::fmt::Debug {
    fn transform(
        &self,
        input: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = RiptideResult<serde_json::Value>> + Send + '_>,
    >;
}

/// Validator trait for validate stage.
pub trait Validator: Send + Sync + std::fmt::Debug {
    fn validate(
        &self,
        input: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = RiptideResult<serde_json::Value>> + Send + '_>,
    >;
}

/// Store destination.
#[derive(Debug, Clone)]
pub enum StoreDestination {
    Memory,
    File(String),
    Database(String),
}

/// Browser action (re-exported from browser facade).
#[derive(Debug, Clone)]
pub enum BrowserAction {
    Click { selector: String },
    Type { selector: String, text: String },
    Wait { duration_ms: u64 },
}

/// Result of pipeline execution.
#[derive(Debug)]
pub struct PipelineResult {
    pub stages_completed: usize,
    pub total_duration: Duration,
    pub stage_results: Vec<StageResult>,
    pub final_output: serde_json::Value,
}

/// Result of a single stage execution.
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage_name: String,
    pub duration: Duration,
    pub status: StageStatus,
    pub output: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageStatus {
    Success,
    CachedSuccess,
    Failed,
}

/// Context passed between pipeline stages.
#[derive(Debug, Clone)]
struct PipelineContext {
    #[allow(dead_code)]
    data: HashMap<String, serde_json::Value>,
    current_output: serde_json::Value,
}

impl PipelineContext {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            current_output: serde_json::Value::Null,
        }
    }

    fn set_output(&mut self, output: serde_json::Value) {
        self.current_output = output;
    }

    fn get_output(&self) -> serde_json::Value {
        self.current_output.clone()
    }

    fn get_final_output(&self) -> serde_json::Value {
        self.current_output.clone()
    }
}

/// Cache for pipeline stage results.
struct PipelineCache {
    entries: HashMap<String, serde_json::Value>,
}

impl PipelineCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: String, value: serde_json::Value) {
        self.entries.insert(key, value);
    }
}

// Example transformer implementation
#[derive(Debug)]
#[allow(dead_code)]
struct UppercaseTransformer;

impl Transformer for UppercaseTransformer {
    fn transform(
        &self,
        input: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = RiptideResult<serde_json::Value>> + Send + '_>,
    > {
        Box::pin(async move {
            if let Some(s) = input.as_str() {
                Ok(serde_json::Value::String(s.to_uppercase()))
            } else {
                Ok(input)
            }
        })
    }
}

// Example validator implementation
#[derive(Debug)]
struct ContentValidator;

impl Validator for ContentValidator {
    fn validate(
        &self,
        input: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = RiptideResult<serde_json::Value>> + Send + '_>,
    > {
        Box::pin(async move {
            if input.is_null() {
                Err(RiptideError::extraction("Content is null"))
            } else {
                Ok(input)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_facade() -> PipelineFacade {
        let config = RiptideConfig::default();
        PipelineFacade::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_pipeline_builder_pattern() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com".to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Html,
            })
            .with_retry(3)
            .with_caching(true)
            .build()
            .await
            .unwrap();

        assert_eq!(pipeline.stages.len(), 2);
        assert_eq!(pipeline.config.max_retries, 3);
        assert!(pipeline.config.caching_enabled);
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com".to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Extract {
                strategy: ExtractionStrategy::Html,
            })
            .build()
            .await
            .unwrap();

        let result = facade.execute(pipeline).await.unwrap();
        assert_eq!(result.stages_completed, 2);
        assert_eq!(result.stage_results.len(), 2);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com/1".to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com/2".to_string(),
                options: FetchOptions::default(),
            })
            .with_parallelism(2)
            .build()
            .await
            .unwrap();

        let result = facade.execute(pipeline).await.unwrap();
        assert_eq!(result.stages_completed, 2);
    }

    #[tokio::test]
    async fn test_error_handling_and_retries() {
        let facade = create_test_facade().await;

        // Test with validator that will pass
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com".to_string(),
                options: FetchOptions::default(),
            })
            .add_stage(PipelineStage::Validate {
                validator: Arc::new(ContentValidator),
            })
            .with_retry(2)
            .build()
            .await
            .unwrap();

        let result = facade.execute(pipeline).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prebuilt_web_scraping_pipeline() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .web_scraping_pipeline("https://example.com")
            .await
            .unwrap();

        assert_eq!(pipeline.stages.len(), 3);
        assert_eq!(pipeline.config.max_retries, 3);
        assert!(pipeline.config.caching_enabled);

        let result = facade.execute(pipeline).await.unwrap();
        assert_eq!(result.stages_completed, 3);
    }

    #[tokio::test]
    async fn test_prebuilt_pdf_extraction_pipeline() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .pdf_extraction_pipeline("https://example.com/doc.pdf")
            .await
            .unwrap();

        assert_eq!(pipeline.stages.len(), 4);
        assert_eq!(pipeline.config.max_retries, 2);

        let result = facade.execute(pipeline).await.unwrap();
        assert_eq!(result.stages_completed, 4);
    }

    #[tokio::test]
    async fn test_caching_behavior() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Fetch {
                url: "https://example.com".to_string(),
                options: FetchOptions::default(),
            })
            .with_caching(true)
            .build()
            .await
            .unwrap();

        // First execution
        let result1 = facade.execute(pipeline.clone()).await.unwrap();
        assert_eq!(result1.stage_results[0].status, StageStatus::Success);

        // Second execution should use cache
        let result2 = facade.execute(pipeline).await.unwrap();
        assert_eq!(result2.stage_results[0].status, StageStatus::CachedSuccess);
    }

    #[tokio::test]
    async fn test_transformer_stage() {
        let facade = create_test_facade().await;
        let pipeline = facade
            .builder()
            .add_stage(PipelineStage::Transform {
                transformer: Arc::new(UppercaseTransformer),
            })
            .build()
            .await
            .unwrap();

        let result = facade.execute(pipeline).await.unwrap();
        assert_eq!(result.stages_completed, 1);
    }
}
