# Thin Handler Refactoring Design

**Date:** 2025-11-06
**Status:** Design Phase
**Objective:** Refactor 6 handlers from 100+ lines to 20-30 lines by moving business logic to facades

---

## Executive Summary

**Current Problem:**
- Handlers contain 80-100+ lines of business logic
- HTTP client calls, parsing, error handling in handlers (API layer)
- Violates clean Rust layering architecture
- Makes testing difficult

**Target Architecture:**
- Handlers: 20-30 lines (validate → call facade → map response)
- Facades: All business logic, HTTP calls, orchestration
- AppState: Initialize facades with dependencies

**Impact:**
- **Lines to Move:** ~500 lines of business logic from handlers → facades
- **Handlers Affected:** 6 handlers (extract, search, spider, pdf, crawl, upload_pdf)
- **New Facade Methods:** 8-10 methods across existing facades

---

## Current State Analysis

### Handler Breakdown

#### 1. Extract Handler (`extract.rs`) - 112 lines
**Current Size:** 80 lines (lines 20-80)
**Target Size:** 25 lines

**Business Logic in Handler (SHOULD BE IN FACADE):**
```rust
Lines 40-72: HTTP client operations
- state.http_client.get(&payload.url).send().await
- response.status().is_success() checks
- response.text().await parsing
- Complex error handling with custom messages

Lines 27-30: URL validation
- url::Url::parse() validation
- Custom ApiError::invalid_url() handling
```

**What Should Stay in Handler:**
- Request deserialization (line 22: `Json(payload)`)
- Metrics/tracing instrumentation (lines 19, 24, 32-37)
- Response mapping to HTTP format

**Business Logic Count:** ~50 lines to move to facade

---

#### 2. Search Handler (`search.rs`) - 106 lines
**Current Size:** 52 lines (lines 25-52)
**Target Size:** 22 lines

**Business Logic in Handler (SHOULD BE IN FACADE):**
```rust
Lines 29-32: Query validation
- params.q.trim().is_empty() check
- Custom ApiError::validation() handling

Lines 35: Limit clamping logic
- params.limit.clamp(1, 50) - business rule

Lines 46-50: Placeholder for SearchFacade call
- Currently returns SERVICE_UNAVAILABLE
- Should call SearchFacade with validated params
```

**What Should Stay in Handler:**
- Query parameter extraction (line 25: `Query(params)`)
- Metrics/tracing instrumentation (lines 24, 26, 37-44)
- Response mapping to HTTP format

**Business Logic Count:** ~15 lines to move to facade

---

#### 3. Spider Handlers (`spider.rs`) - 109 lines
**Current Size:** 87 lines (lines 68-87 + status + control)
**Target Size:** 30 lines (3 handlers × 10 lines each)

**Business Logic in Handler (SHOULD BE IN FACADE):**
```rust
Lines 68-87: spider_crawl handler
- Query parameter extraction and validation
- SpiderCrawlQuery deserialization
- Currently returns ApiError::internal (facade unavailable)

Lines 90-98: spider_status handler
- Status retrieval logic (currently stubbed)

Lines 101-109: spider_control handler
- Control operations (start/stop/reset) logic (currently stubbed)
```

**What Should Stay in Handler:**
- Request deserialization
- Metrics/tracing instrumentation (lines 56-66)
- Response mapping to HTTP format

**Business Logic Count:** ~25 lines to move to facade

---

#### 4. PDF Handler (`pdf.rs`) - 642 lines
**Current Size:** 155 lines (process_pdf) + 235 lines (process_pdf_stream) + 163 lines (upload_pdf)
**Target Size:** 30 lines (process_pdf) + 25 lines (stream) + 30 lines (upload)

**Business Logic in Handler (SHOULD BE IN FACADE):**

**process_pdf (lines 75-155):**
```rust
Lines 82-89: Base64 decoding
- BASE64_STANDARD.decode() operation
- Custom error handling

Lines 100-104: File size validation
- 50MB limit check - business rule

Lines 107-149: Resource acquisition
- ResourceManager complex pattern matching
- Multiple error cases (Timeout, ResourceExhausted, MemoryPressure, RateLimited)
- Guard lifecycle management

Lines 152-154: PDF processing placeholder
- Currently returns ApiError::internal (facade unavailable)
```

**process_pdf_stream (lines 161-234):**
```rust
Lines 168-189: Same as process_pdf (duplication!)
- Base64 decoding
- File size validation

Lines 193-209: PDF integration setup
- create_pdf_integration_for_pipeline()
- create_progress_channel()
- should_process_as_pdf() validation
- tokio::spawn for async processing

Lines 212-234: Stream construction
- create_enhanced_progress_stream() with metrics
- StreamingResponseBuilder complex setup
```

**upload_pdf (lines 386-549):**
```rust
Lines 398-484: Multipart field processing
- Complex state machine for field extraction
- Content type validation
- PDF magic bytes validation (lines 428-433)
- Multiple field types (file, filename, url, stream_progress)

Lines 501-543: Resource acquisition (DUPLICATE of process_pdf!)

Lines 546-548: PDF processing placeholder
```

**create_enhanced_progress_stream (lines 236-355):**
- 120 lines of stream processing logic
- Progress metrics calculation
- State machine for progress updates
- Should be in facade/service layer

**What Should Stay in Handler:**
- Request deserialization
- Metrics/tracing instrumentation
- Response mapping to HTTP format

**Business Logic Count:** ~450 lines to move to facade (largest refactor)

---

#### 5. Crawl Handler (`crawl.rs`) - 398 lines
**Current Size:** 286 lines (lines 43-286) + 109 lines (handle_spider_crawl)
**Target Size:** 35 lines (crawl) + 25 lines (handle_spider_crawl)

**Business Logic in Handler (SHOULD BE IN FACADE):**

**crawl (lines 43-286):**
```rust
Lines 50-62: Trace context extraction
- extract_trace_context() - could be middleware
- Span recording of custom attributes

Lines 71-79: Event bus emission
- BaseEvent construction
- state.event_bus.emit() calls

Lines 82: Request validation
- validate_crawl_request() - should be in facade

Lines 88-91: Spider mode routing
- Conditional logic based on options.use_spider
- Route to handle_spider_crawl()

Lines 101-162: Pipeline orchestration
- Enhanced vs standard pipeline selection
- PipelineOrchestrator.execute_batch() call
- EnhancedPipelineOrchestrator.execute_batch_enhanced() call
- Result transformation between enhanced and standard formats
- 40+ lines of result mapping

Lines 164-217: Result processing loop
- apply_content_chunking() conditional logic
- CrawlResult construction from pipeline results
- Error handling for None results

Lines 219-237: Statistics calculation
- cache_hit_rate calculation
- GateDecisionBreakdown construction
- CrawlStatistics assembly

Lines 248-257: Metrics recording
- Span recording of results
- Structured logging

Lines 267-278: Event emission (duplicate of lines 71-79)
- Complete event with metadata

Lines 281-284: HTTP metrics recording
```

**handle_spider_crawl (lines 290-397):**
```rust
Lines 295-301: URL parsing
- parse_seed_urls() utility call
- SpiderConfigBuilder construction

Lines 315-343: Spider crawl execution (unreachable code)
- Metrics recording
- spider_facade.crawl() call (removed)
- PlaceholderSpiderResult (unreachable)

Lines 345-395: Result transformation
- Spider result → CrawlResult mapping
- Statistics calculation
- Response construction
```

**What Should Stay in Handler:**
- Request deserialization
- Basic tracing (span creation)
- Response mapping to HTTP format

**Business Logic Count:** ~280 lines to move to facade

---

### Total Business Logic to Move

| Handler | Current Lines | Target Lines | Lines to Move |
|---------|---------------|--------------|---------------|
| extract.rs | 80 | 25 | 50 |
| search.rs | 52 | 22 | 15 |
| spider.rs | 87 | 30 | 25 |
| pdf.rs | 553 | 85 | 450 |
| crawl.rs | 395 | 60 | 280 |
| **TOTAL** | **1,167** | **222** | **820** |

**Summary:** Move **820 lines** of business logic from handlers to facades.

---

## Target Architecture

### Thin Handler Pattern (20-30 lines)

```rust
/// TEMPLATE: Thin handler pattern
pub async fn handler_name(
    State(state): State<AppState>,
    Json(payload): Json<RequestDto>,
) -> impl IntoResponse {
    // 1. VALIDATE INPUT (HTTP concern) - 5 lines
    //    - Basic validation (non-empty, format checks)
    //    - Return HTTP 400 errors for invalid input
    if let Err(e) = validate_input(&payload) {
        return ApiError::validation(e).into_response();
    }

    // 2. CALL FACADE (orchestration) - 3 lines
    //    - Delegate ALL business logic to facade
    //    - Facade handles: HTTP calls, parsing, retries, circuit breakers
    let result = state.facade_name
        .method_name(payload.into())
        .await;

    // 3. MAP RESPONSE (HTTP concern) - 5 lines
    //    - Convert domain result to HTTP response
    //    - Map errors to appropriate HTTP status codes
    match result {
        Ok(data) => Json(ResponseDto::from(data)).into_response(),
        Err(e) => ApiError::from(e).into_response(),
    }
}
```

---

## Facade Method Designs

### 1. ExtractionFacade Methods

#### `extract_from_url`
```rust
impl ExtractionFacade {
    /// Extract content from a URL using multi-strategy extraction
    ///
    /// This method handles:
    /// - HTTP fetching with retries
    /// - HTML parsing
    /// - Strategy selection (CSS, WASM, Readability)
    /// - Quality gates
    /// - Error recovery
    pub async fn extract_from_url(
        &self,
        url: &str,
        options: ExtractOptions,
    ) -> RiptideResult<ExtractedDoc> {
        // 1. Fetch HTML with HTTP client
        let html = self.fetch_html(url).await?;

        // 2. Apply extraction strategies
        let doc = self.extract_with_strategies(url, &html, options).await?;

        // 3. Apply quality gates
        self.apply_quality_gates(doc).await
    }

    /// Extract content from raw HTML (for PDF/multipart use cases)
    pub async fn extract_from_html(
        &self,
        url: &str,
        html: &str,
        options: ExtractOptions,
    ) -> RiptideResult<ExtractedDoc> {
        let doc = self.extract_with_strategies(url, html, options).await?;
        self.apply_quality_gates(doc).await
    }

    /// Internal: Fetch HTML with retries and circuit breaker
    async fn fetch_html(&self, url: &str) -> RiptideResult<String> {
        // HTTP client call with retry logic
        // Circuit breaker integration
        // Response parsing
        // Error handling
    }

    /// Internal: Apply extraction strategies with fallback
    async fn extract_with_strategies(
        &self,
        url: &str,
        html: &str,
        options: ExtractOptions,
    ) -> RiptideResult<ExtractedDoc> {
        // Strategy selection based on options.strategy
        // Primary strategy execution
        // Fallback to secondary strategies if quality low
        // Quality score calculation
    }

    /// Internal: Apply quality gates
    async fn apply_quality_gates(
        &self,
        mut doc: ExtractedDoc,
    ) -> RiptideResult<ExtractedDoc> {
        // Check quality score vs threshold
        // Trigger headless render if needed
        // Return enhanced doc or error
    }
}
```

**Lines in Facade:** ~150 lines (vs 50 lines in handler)

---

### 2. SearchFacade Methods

#### `search`
```rust
impl SearchFacade {
    /// Search using configured providers with automatic fallback
    ///
    /// This method handles:
    /// - Query normalization
    /// - Provider selection (Serper, SearXNG, None)
    /// - Result aggregation
    /// - Error recovery and fallback
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        country: &str,
        language: &str,
        provider: Option<SearchProvider>,
    ) -> RiptideResult<SearchResults> {
        // 1. Normalize query (trim, validate)
        let normalized_query = self.normalize_query(query)?;

        // 2. Select provider (explicit or auto-select)
        let provider = provider.unwrap_or_else(|| self.select_provider());

        // 3. Execute search with provider
        let results = self.execute_search(
            &normalized_query,
            limit,
            country,
            language,
            provider,
        ).await?;

        // 4. Post-process and rank results
        Ok(self.post_process_results(results))
    }

    /// Internal: Normalize and validate query
    fn normalize_query(&self, query: &str) -> RiptideResult<String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Err(RiptideError::validation("Query cannot be empty"));
        }
        Ok(trimmed.to_string())
    }

    /// Internal: Select best provider based on availability
    fn select_provider(&self) -> SearchProvider {
        // Check provider health
        // Return best available provider
    }

    /// Internal: Execute search with retry and fallback
    async fn execute_search(
        &self,
        query: &str,
        limit: usize,
        country: &str,
        language: &str,
        provider: SearchProvider,
    ) -> RiptideResult<Vec<SearchResult>> {
        // Call provider API
        // Retry on transient failures
        // Fallback to secondary provider if needed
    }

    /// Internal: Post-process and rank results
    fn post_process_results(
        &self,
        results: Vec<SearchResult>,
    ) -> SearchResults {
        // Deduplicate results
        // Apply ranking algorithm
        // Format response
    }
}
```

**Lines in Facade:** ~100 lines (vs 15 lines in handler)

---

### 3. SpiderFacade Methods

#### `crawl`, `get_status`, `control`
```rust
impl SpiderFacade {
    /// Start a spider crawl with specified configuration
    ///
    /// This method handles:
    /// - Seed URL parsing and validation
    /// - Spider configuration
    /// - Crawl execution with budget controls
    /// - Result aggregation
    pub async fn crawl(
        &self,
        seed_urls: Vec<String>,
        options: SpiderOptions,
    ) -> RiptideResult<SpiderCrawlResult> {
        // 1. Parse and validate seed URLs
        let parsed_urls = self.parse_seed_urls(&seed_urls)?;

        // 2. Build spider configuration
        let config = self.build_spider_config(parsed_urls, options)?;

        // 3. Execute crawl with orchestrator
        let result = self.orchestrator.execute(config).await?;

        // 4. Transform result to API format
        Ok(SpiderCrawlResult::from(result))
    }

    /// Get current spider status and metrics
    pub async fn get_status(
        &self,
        session_id: &str,
    ) -> RiptideResult<SpiderStatus> {
        // Query orchestrator for session status
        // Format metrics and state
    }

    /// Control spider operations (start/stop/reset)
    pub async fn control(
        &self,
        session_id: &str,
        action: SpiderAction,
    ) -> RiptideResult<SpiderControlResult> {
        // Validate session exists
        // Execute control action (start/stop/reset)
        // Return new state
    }

    /// Internal: Parse and validate seed URLs
    fn parse_seed_urls(&self, urls: &[String]) -> RiptideResult<Vec<Url>> {
        urls.iter()
            .map(|s| Url::parse(s).map_err(|e|
                RiptideError::validation(format!("Invalid URL '{}': {}", s, e))
            ))
            .collect()
    }

    /// Internal: Build spider configuration from options
    fn build_spider_config(
        &self,
        seed_urls: Vec<Url>,
        options: SpiderOptions,
    ) -> RiptideResult<SpiderConfig> {
        // Convert SpiderOptions to SpiderConfig
        // Apply defaults
        // Validate configuration
    }
}
```

**Lines in Facade:** ~120 lines (vs 25 lines in handlers)

---

### 4. PdfFacade Methods

#### `process_pdf`, `process_pdf_stream`
```rust
impl PdfFacade {
    /// Process PDF bytes synchronously
    ///
    /// This method handles:
    /// - Base64 decoding
    /// - File size validation
    /// - Resource acquisition (semaphore + memory)
    /// - PDF processing with riptide-pdf
    /// - Error recovery
    pub async fn process_pdf(
        &self,
        pdf_data: PdfInput,
        options: PdfProcessOptions,
    ) -> RiptideResult<PdfProcessResult> {
        // 1. Decode and validate PDF data
        let pdf_bytes = self.decode_pdf_data(pdf_data)?;

        // 2. Validate file size and format
        self.validate_pdf(&pdf_bytes)?;

        // 3. Acquire processing resources
        let _guard = self.acquire_resources().await?;

        // 4. Process PDF with riptide-pdf integration
        let doc = self.process_pdf_bytes(&pdf_bytes, options).await?;

        // 5. Calculate statistics
        Ok(PdfProcessResult {
            document: doc,
            stats: self.calculate_stats(&pdf_bytes),
        })
    }

    /// Process PDF with streaming progress updates
    pub async fn process_pdf_stream(
        &self,
        pdf_data: PdfInput,
        options: PdfProcessOptions,
    ) -> RiptideResult<PdfProgressStream> {
        // 1. Decode and validate PDF data
        let pdf_bytes = self.decode_pdf_data(pdf_data)?;

        // 2. Validate file size and format
        self.validate_pdf(&pdf_bytes)?;

        // 3. Create PDF integration with progress channel
        let (tx, rx) = self.create_progress_channel();

        // 4. Spawn processing task
        self.spawn_processing_task(pdf_bytes, options, tx);

        // 5. Return enhanced progress stream
        Ok(self.create_enhanced_stream(rx))
    }

    /// Process multipart PDF upload
    pub async fn process_multipart(
        &self,
        multipart: Multipart,
    ) -> RiptideResult<PdfProcessResult> {
        // 1. Extract PDF data and metadata from multipart
        let (pdf_bytes, metadata) = self.extract_multipart_data(multipart).await?;

        // 2. Delegate to process_pdf
        self.process_pdf(
            PdfInput::Bytes(pdf_bytes),
            metadata.into(),
        ).await
    }

    /// Internal: Decode PDF data (base64 or raw bytes)
    fn decode_pdf_data(&self, input: PdfInput) -> RiptideResult<Vec<u8>> {
        match input {
            PdfInput::Base64(encoded) => {
                BASE64_STANDARD.decode(encoded)
                    .map_err(|e| RiptideError::validation(format!("Invalid base64: {}", e)))
            }
            PdfInput::Bytes(bytes) => Ok(bytes),
        }
    }

    /// Internal: Validate PDF file size and format
    fn validate_pdf(&self, bytes: &[u8]) -> RiptideResult<()> {
        // Check file size (50MB limit)
        if bytes.len() > 50 * 1024 * 1024 {
            return Err(RiptideError::validation("PDF too large (max 50MB)"));
        }

        // Check PDF magic bytes
        if bytes.len() < 5 || &bytes[0..4] != b"%PDF" {
            return Err(RiptideError::validation("Not a valid PDF file"));
        }

        Ok(())
    }

    /// Internal: Acquire PDF processing resources
    async fn acquire_resources(&self) -> RiptideResult<ResourceGuard> {
        // Use ResourceManager to acquire semaphore
        // Handle all error cases (Timeout, ResourceExhausted, MemoryPressure, RateLimited)
        // Return guard for RAII cleanup
    }

    /// Internal: Process PDF bytes with riptide-pdf
    async fn process_pdf_bytes(
        &self,
        bytes: &[u8],
        options: PdfProcessOptions,
    ) -> RiptideResult<ExtractedDoc> {
        // Call riptide-pdf integration
        // Apply timeout
        // Convert to ExtractedDoc
    }

    /// Internal: Extract multipart data
    async fn extract_multipart_data(
        &self,
        mut multipart: Multipart,
    ) -> RiptideResult<(Vec<u8>, PdfMetadata)> {
        // State machine for field extraction
        // Validate content types
        // Extract file, filename, url, stream_progress fields
    }

    /// Internal: Create progress channel
    fn create_progress_channel(&self) -> (ProgressSender, ProgressReceiver) {
        // Use riptide-pdf integration
        // Create sender/receiver pair
    }

    /// Internal: Spawn processing task
    fn spawn_processing_task(
        &self,
        pdf_bytes: Vec<u8>,
        options: PdfProcessOptions,
        tx: ProgressSender,
    ) {
        // tokio::spawn async task
        // Call riptide-pdf with progress updates
        // Handle errors gracefully
    }

    /// Internal: Create enhanced progress stream
    fn create_enhanced_stream(
        &self,
        rx: ProgressReceiver,
    ) -> PdfProgressStream {
        // Wrap ProgressReceiver in enhanced stream
        // Add metrics tracking (pages/sec, overhead)
        // Add memory monitoring
    }

    /// Internal: Calculate processing statistics
    fn calculate_stats(&self, bytes: &[u8]) -> ProcessingStats {
        // Calculate processing time, file size, pages, memory
    }
}
```

**Lines in Facade:** ~300 lines (vs 450 lines in handlers)

---

### 5. CrawlFacade Methods

#### `crawl_batch`, `crawl_spider_mode`
```rust
impl CrawlFacade {
    /// Execute batch crawl through pipeline orchestrator
    ///
    /// This method handles:
    /// - Request validation
    /// - Pipeline selection (enhanced vs standard)
    /// - Batch execution with concurrency control
    /// - Result transformation
    /// - Statistics calculation
    pub async fn crawl_batch(
        &self,
        urls: Vec<String>,
        options: CrawlOptions,
    ) -> RiptideResult<CrawlResponse> {
        // 1. Validate request
        self.validate_crawl_request(&urls, &options)?;

        // 2. Select pipeline (enhanced vs standard)
        let pipeline = self.select_pipeline(&options);

        // 3. Execute batch crawl
        let (results, stats) = pipeline.execute_batch(&urls).await;

        // 4. Apply chunking if requested
        let results = self.apply_chunking(results, &options).await;

        // 5. Calculate statistics
        let statistics = self.calculate_statistics(&results, &stats);

        // 6. Build response
        Ok(CrawlResponse {
            total_urls: urls.len(),
            successful: stats.successful_extractions,
            failed: stats.failed_extractions,
            from_cache: stats.cache_hits,
            results,
            statistics,
        })
    }

    /// Execute spider crawl mode
    pub async fn crawl_spider_mode(
        &self,
        seed_urls: Vec<String>,
        options: CrawlOptions,
    ) -> RiptideResult<CrawlResponse> {
        // 1. Validate seed URLs
        let parsed_urls = self.parse_seed_urls(&seed_urls)?;

        // 2. Build spider configuration
        let spider_config = self.build_spider_config(parsed_urls, &options)?;

        // 3. Execute spider crawl
        let spider_result = self.spider_facade.crawl(seed_urls, spider_config).await?;

        // 4. Transform spider result to crawl response
        Ok(self.transform_spider_result(spider_result))
    }

    /// Internal: Validate crawl request
    fn validate_crawl_request(
        &self,
        urls: &[String],
        options: &CrawlOptions,
    ) -> RiptideResult<()> {
        // Validate URL count (not empty, not too many)
        // Validate options (concurrency, cache_mode)
        // Validate chunking config if provided
    }

    /// Internal: Select pipeline based on configuration
    fn select_pipeline(&self, options: &CrawlOptions) -> Pipeline {
        if self.config.enhanced_pipeline_enabled {
            Pipeline::Enhanced(EnhancedPipelineOrchestrator::new(self.state.clone(), options.clone()))
        } else {
            Pipeline::Standard(PipelineOrchestrator::new(self.state.clone(), options.clone()))
        }
    }

    /// Internal: Apply chunking to results
    async fn apply_chunking(
        &self,
        results: Vec<CrawlResult>,
        options: &CrawlOptions,
    ) -> Vec<CrawlResult> {
        if let Some(ref config) = options.chunking_config {
            // Apply chunking to each result's document
        } else {
            results
        }
    }

    /// Internal: Calculate crawl statistics
    fn calculate_statistics(
        &self,
        results: &[CrawlResult],
        stats: &PipelineStats,
    ) -> CrawlStatistics {
        // Calculate cache hit rate
        // Build gate decision breakdown
        // Build CrawlStatistics
    }

    /// Internal: Transform spider result to crawl response
    fn transform_spider_result(
        &self,
        spider_result: SpiderCrawlResult,
    ) -> CrawlResponse {
        // Convert SpiderCrawlResult to CrawlResponse format
        // Map spider pages to CrawlResult items
    }
}
```

**Lines in Facade:** ~200 lines (vs 280 lines in handler)

---

## AppState Changes

### Current State (lines 142-165 in `state.rs`)

```rust
// REMOVED: Caused circular dependency with riptide-facade
// #[cfg(feature = "extraction")]
// pub extraction_facade: Arc<ExtractionFacade>,

// #[cfg(feature = "browser")]
// pub browser_facade: Option<Arc<BrowserFacade>>,

// pub scraper_facade: Arc<ScraperFacade>,

// #[cfg(feature = "spider")]
// pub spider_facade: Option<Arc<SpiderFacade>>,

// #[cfg(feature = "search")]
// pub search_facade: Option<Arc<SearchFacade>>,
```

**Issue:** Facades were removed due to circular dependency between `riptide-api` and `riptide-facade`.

---

### Target State

**Step 1: Fix Circular Dependency**

Move facade initialization OUT of AppState construction into a separate initialization step:

```rust
// In state.rs - AppState struct
#[derive(Clone)]
pub struct AppState {
    // ... existing fields ...

    /// Extraction facade for content extraction
    pub extraction_facade: Arc<ExtractionFacade>,

    /// Search facade for web search
    pub search_facade: Arc<SearchFacade>,

    /// Spider facade for web crawling
    #[cfg(feature = "spider")]
    pub spider_facade: Option<Arc<SpiderFacade>>,

    /// PDF facade for PDF processing
    pub pdf_facade: Arc<PdfFacade>,

    /// Crawl facade for batch crawling
    pub crawl_facade: Arc<CrawlFacade>,
}
```

**Step 2: Initialize Facades After AppState Construction**

```rust
// In main.rs or AppState::new()
impl AppState {
    pub async fn new_with_facades(
        config: AppConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
    ) -> Result<Self> {
        // 1. Create base AppState WITHOUT facades
        let mut state = Self::new_base(config, metrics, health_checker).await?;

        // 2. Initialize facades with state dependencies
        state = state.with_facades().await?;

        Ok(state)
    }

    async fn new_base(
        config: AppConfig,
        metrics: Arc<RipTideMetrics>,
        health_checker: Arc<HealthChecker>,
    ) -> Result<Self> {
        // Initialize all components EXCEPT facades
        // ... existing initialization code ...
    }

    async fn with_facades(mut self) -> Result<Self> {
        // Initialize ExtractionFacade
        let extraction_facade = Arc::new(
            ExtractionFacade::new(
                self.http_client.clone(),
                self.extractor.clone(),
                self.cache.clone(),
                self.config.gate_hi_threshold,
                self.config.gate_lo_threshold,
            ).await?
        );

        // Initialize SearchFacade
        let backend = std::env::var("SEARCH_BACKEND")
            .unwrap_or_else(|_| "none".to_string())
            .parse()
            .unwrap_or(SearchBackend::None);
        let search_facade = Arc::new(SearchFacade::new(backend).await?);

        // Initialize SpiderFacade (if enabled)
        #[cfg(feature = "spider")]
        let spider_facade = if let Some(ref spider_config) = self.config.spider_config {
            Some(Arc::new(
                SpiderFacade::from_config(spider_config.clone()).await?
            ))
        } else {
            None
        };

        // Initialize PdfFacade
        let pdf_facade = Arc::new(
            PdfFacade::new(
                self.resource_manager.clone(),
                self.pdf_metrics.clone(),
            ).await?
        );

        // Initialize CrawlFacade
        let crawl_facade = Arc::new(
            CrawlFacade::new(
                self.clone(), // Needs full AppState for pipeline access
                spider_facade.clone(),
            ).await?
        );

        // Assign facades to state
        self.extraction_facade = extraction_facade;
        self.search_facade = search_facade;
        #[cfg(feature = "spider")]
        self.spider_facade = spider_facade;
        self.pdf_facade = pdf_facade;
        self.crawl_facade = crawl_facade;

        Ok(self)
    }
}
```

**Step 3: Update main.rs**

```rust
// In crates/riptide-api/src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // ... initialization ...

    // OLD:
    // let state = AppState::new(config, metrics, health_checker).await?;

    // NEW:
    let state = AppState::new_with_facades(config, metrics, health_checker).await?;

    // ... rest of main ...
}
```

---

## Handler Refactoring Plan

### Phase 1: Extract Handler (Simplest)

**Target:** 25 lines (from 80 lines)

**Steps:**
1. Create `ExtractionFacade::extract_from_url()` method
2. Move HTTP client logic to facade (lines 40-72)
3. Move URL validation to facade (lines 27-30)
4. Update handler to:
   ```rust
   pub async fn extract(
       State(state): State<AppState>,
       Json(payload): Json<ExtractRequest>,
   ) -> impl IntoResponse {
       // Validate URL format (HTTP concern)
       if let Err(e) = url::Url::parse(&payload.url) {
           return ApiError::invalid_url(&payload.url, e.to_string()).into_response();
       }

       // Call facade with all business logic
       let result = state.extraction_facade
           .extract_from_url(&payload.url, payload.options)
           .await;

       // Map result to HTTP response
       match result {
           Ok(doc) => Json(ExtractResponse { document: doc }).into_response(),
           Err(e) => ApiError::from(e).into_response(),
       }
   }
   ```
5. Test handler with existing tests
6. Verify metrics and tracing still work

**Success Criteria:**
- Handler is 25 lines
- All tests pass
- Metrics/tracing unchanged

---

### Phase 2: Search Handler

**Target:** 22 lines (from 52 lines)

**Steps:**
1. Create `SearchFacade::search()` method
2. Move query validation to facade (lines 29-32)
3. Move limit clamping to facade (line 35)
4. Update handler to:
   ```rust
   pub async fn search(
       State(state): State<AppState>,
       Query(params): Query<SearchQuery>,
   ) -> Response {
       // Call facade with all business logic
       let result = state.search_facade
           .search(
               &params.q,
               params.limit,
               &params.country,
               &params.language,
               params.provider,
           )
           .await;

       // Map result to HTTP response
       match result {
           Ok(results) => Json(SearchResponse::from(results)).into_response(),
           Err(e) => ApiError::from(e).into_response(),
       }
   }
   ```
5. Test handler
6. Verify provider fallback works

**Success Criteria:**
- Handler is 22 lines
- All tests pass
- Provider fallback works

---

### Phase 3: Spider Handlers

**Target:** 30 lines total (3 handlers × 10 lines)

**Steps:**
1. Create `SpiderFacade::crawl()`, `get_status()`, `control()` methods
2. Move URL parsing to facade
3. Move spider config building to facade
4. Update handlers:
   ```rust
   pub async fn spider_crawl(
       State(state): State<AppState>,
       Query(query): Query<SpiderCrawlQuery>,
       Json(body): Json<SpiderCrawlBody>,
   ) -> Result<impl IntoResponse, ApiError> {
       let result = state.spider_facade
           .crawl(body.seed_urls, body.into())
           .await?;
       Ok(Json(result))
   }

   pub async fn spider_status(
       State(state): State<AppState>,
       Json(body): Json<SpiderStatusRequest>,
   ) -> Result<impl IntoResponse, ApiError> {
       let status = state.spider_facade
           .get_status(&body.session_id)
           .await?;
       Ok(Json(status))
   }

   pub async fn spider_control(
       State(state): State<AppState>,
       Json(body): Json<SpiderControlRequest>,
   ) -> Result<impl IntoResponse, ApiError> {
       let result = state.spider_facade
           .control(&body.session_id, body.action)
           .await?;
       Ok(Json(result))
   }
   ```
5. Test all three handlers
6. Verify session management works

**Success Criteria:**
- Each handler is ~10 lines
- All tests pass
- Session management works

---

### Phase 4: PDF Handlers (Most Complex)

**Target:** 85 lines total (30 + 25 + 30)

**Steps:**
1. Create `PdfFacade::process_pdf()`, `process_pdf_stream()`, `process_multipart()` methods
2. Move base64 decoding to facade
3. Move file validation to facade
4. Move resource acquisition to facade
5. Move multipart parsing to facade
6. Move progress stream creation to facade
7. Update handlers:
   ```rust
   pub async fn process_pdf(
       State(state): State<AppState>,
       Json(request): Json<PdfProcessRequest>,
   ) -> Result<Json<PdfProcessResponse>, ApiError> {
       let pdf_input = PdfInput::Base64(
           request.pdf_data.ok_or_else(|| ApiError::validation("PDF data required"))?
       );

       let result = state.pdf_facade
           .process_pdf(pdf_input, request.into())
           .await?;

       Ok(Json(PdfProcessResponse::from(result)))
   }

   pub async fn process_pdf_stream(
       State(state): State<AppState>,
       Json(request): Json<PdfProcessRequest>,
   ) -> Result<Response, ApiError> {
       let pdf_input = PdfInput::Base64(
           request.pdf_data.ok_or_else(|| ApiError::validation("PDF data required"))?
       );

       let stream = state.pdf_facade
           .process_pdf_stream(pdf_input, request.into())
           .await?;

       Ok(StreamingResponseBuilder::new(StreamingResponseType::Ndjson)
           .build(stream))
   }

   pub async fn upload_pdf(
       State(state): State<AppState>,
       multipart: Multipart,
   ) -> Result<Json<PdfProcessResponse>, ApiError> {
       let result = state.pdf_facade
           .process_multipart(multipart)
           .await?;

       Ok(Json(PdfProcessResponse::from(result)))
   }
   ```
8. Test all three handlers
9. Verify streaming still works
10. Verify multipart upload still works

**Success Criteria:**
- process_pdf: 30 lines
- process_pdf_stream: 25 lines
- upload_pdf: 30 lines
- All tests pass
- Streaming works
- Multipart works

---

### Phase 5: Crawl Handler (Largest Refactor)

**Target:** 60 lines total (35 + 25)

**Steps:**
1. Create `CrawlFacade::crawl_batch()`, `crawl_spider_mode()` methods
2. Move pipeline selection to facade
3. Move result transformation to facade
4. Move statistics calculation to facade
5. Move event emission to facade (or keep in handler?)
6. Update handlers:
   ```rust
   pub async fn crawl(
       State(state): State<AppState>,
       headers: HeaderMap,
       Json(body): Json<CrawlBody>,
   ) -> Result<Json<CrawlResponse>, ApiError> {
       // Extract trace context (middleware-level concern)
       if let Some(_ctx) = extract_trace_context(&headers) {
           // Context propagation
       }

       // Route to spider mode if requested
       if body.options.as_ref().and_then(|o| o.use_spider).unwrap_or(false) {
           return crawl_spider_mode(state, body).await;
       }

       // Call facade for batch crawl
       let response = state.crawl_facade
           .crawl_batch(body.urls, body.options.unwrap_or_default())
           .await?;

       Ok(Json(response))
   }

   async fn crawl_spider_mode(
       state: AppState,
       body: CrawlBody,
   ) -> Result<Json<CrawlResponse>, ApiError> {
       let response = state.crawl_facade
           .crawl_spider_mode(body.urls, body.options.unwrap_or_default())
           .await?;

       Ok(Json(response))
   }
   ```
7. Test both handlers
8. Verify enhanced pipeline still works
9. Verify spider mode still works

**Success Criteria:**
- crawl: 35 lines
- crawl_spider_mode: 25 lines
- All tests pass
- Enhanced pipeline works
- Spider mode works

---

## Testing Strategy

### Unit Tests (Facade Level)

**Test each facade method independently:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extraction_facade_extract_from_url() {
        let facade = ExtractionFacade::new_test().await;
        let result = facade.extract_from_url("https://example.com", ExtractOptions::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extraction_facade_handles_fetch_errors() {
        let facade = ExtractionFacade::new_test().await;
        let result = facade.extract_from_url("https://invalid-domain-12345.com", ExtractOptions::default()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_facade_validates_query() {
        let facade = SearchFacade::new_test().await;
        let result = facade.search("", 10, "us", "en", None).await;
        assert!(matches!(result, Err(RiptideError::Validation(_))));
    }

    #[tokio::test]
    async fn test_pdf_facade_validates_file_size() {
        let facade = PdfFacade::new_test().await;
        let large_data = vec![0u8; 51 * 1024 * 1024]; // 51MB
        let result = facade.process_pdf(PdfInput::Bytes(large_data), PdfProcessOptions::default()).await;
        assert!(matches!(result, Err(RiptideError::Validation(_))));
    }
}
```

---

### Integration Tests (Handler Level)

**Test thin handlers with mocked facades:**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum_test_helpers::TestClient;

    #[tokio::test]
    async fn test_extract_handler_success() {
        let app = create_test_app().await;
        let client = TestClient::new(app);

        let payload = json!({
            "url": "https://example.com",
            "mode": "standard"
        });

        let response = client.post("/extract")
            .json(&payload)
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_extract_handler_invalid_url() {
        let app = create_test_app().await;
        let client = TestClient::new(app);

        let payload = json!({
            "url": "not-a-url",
            "mode": "standard"
        });

        let response = client.post("/extract")
            .json(&payload)
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
```

---

### End-to-End Tests

**Test full request flow:**

```rust
#[tokio::test]
async fn test_e2e_extract_flow() {
    // 1. Start test server
    let server = spawn_test_server().await;

    // 2. Make real HTTP request
    let response = reqwest::Client::new()
        .post(format!("{}/extract", server.addr()))
        .json(&json!({
            "url": "https://example.com",
            "mode": "standard"
        }))
        .send()
        .await
        .unwrap();

    // 3. Verify response
    assert_eq!(response.status(), StatusCode::OK);
    let body: ExtractResponse = response.json().await.unwrap();
    assert!(body.document.title.is_some());
}
```

---

## Rollback Plan

### If Issues Arise During Refactoring

**Option 1: Revert Specific Handler**

```bash
# Revert single handler if tests fail
git checkout HEAD -- crates/riptide-api/src/handlers/extract.rs
cargo test -p riptide-api --test extract_tests
```

**Option 2: Revert Entire Phase**

```bash
# Revert entire phase if multiple handlers broken
git revert <phase-commit-sha>
cargo test -p riptide-api
```

**Option 3: Feature Flag Fallback**

```rust
// Add feature flag for old handler implementation
#[cfg(feature = "thin-handlers")]
pub async fn extract_new(/* thin handler */) -> impl IntoResponse { ... }

#[cfg(not(feature = "thin-handlers"))]
pub async fn extract(/* old handler */) -> impl IntoResponse { ... }
```

**Option 4: Gradual Migration**

```rust
// Keep both handlers, route based on header
pub async fn extract(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ExtractRequest>,
) -> impl IntoResponse {
    if headers.get("X-Use-Thin-Handler").is_some() {
        extract_thin(state, payload).await
    } else {
        extract_legacy(state, payload).await
    }
}
```

---

## Migration Checklist

### Pre-Refactoring

- [ ] Read all 6 handlers and document business logic
- [ ] Design facade method signatures
- [ ] Design AppState initialization changes
- [ ] Set up test infrastructure
- [ ] Create rollback plan

### Phase 1: Extract Handler

- [ ] Create `ExtractionFacade::extract_from_url()` method
- [ ] Move HTTP logic to facade
- [ ] Move URL validation to facade
- [ ] Update handler to thin pattern
- [ ] Run tests: `cargo test -p riptide-api --test extract_tests`
- [ ] Verify metrics/tracing
- [ ] Commit: `git commit -m "refactor: Thin extract handler (Phase 1)"`

### Phase 2: Search Handler

- [ ] Create `SearchFacade::search()` method
- [ ] Move query validation to facade
- [ ] Move limit clamping to facade
- [ ] Update handler to thin pattern
- [ ] Run tests: `cargo test -p riptide-api --test search_tests`
- [ ] Verify provider fallback
- [ ] Commit: `git commit -m "refactor: Thin search handler (Phase 2)"`

### Phase 3: Spider Handlers

- [ ] Create `SpiderFacade::crawl()`, `get_status()`, `control()` methods
- [ ] Move URL parsing to facade
- [ ] Move spider config to facade
- [ ] Update all 3 handlers to thin pattern
- [ ] Run tests: `cargo test -p riptide-api --test spider_tests`
- [ ] Verify session management
- [ ] Commit: `git commit -m "refactor: Thin spider handlers (Phase 3)"`

### Phase 4: PDF Handlers

- [ ] Create `PdfFacade::process_pdf()`, `process_pdf_stream()`, `process_multipart()` methods
- [ ] Move base64 decoding to facade
- [ ] Move file validation to facade
- [ ] Move resource acquisition to facade
- [ ] Move multipart parsing to facade
- [ ] Move progress stream to facade
- [ ] Update all 3 handlers to thin pattern
- [ ] Run tests: `cargo test -p riptide-api --test pdf_tests`
- [ ] Verify streaming works
- [ ] Verify multipart works
- [ ] Commit: `git commit -m "refactor: Thin PDF handlers (Phase 4)"`

### Phase 5: Crawl Handler

- [ ] Create `CrawlFacade::crawl_batch()`, `crawl_spider_mode()` methods
- [ ] Move pipeline selection to facade
- [ ] Move result transformation to facade
- [ ] Move statistics calculation to facade
- [ ] Update both handlers to thin pattern
- [ ] Run tests: `cargo test -p riptide-api --test crawl_tests`
- [ ] Verify enhanced pipeline
- [ ] Verify spider mode
- [ ] Commit: `git commit -m "refactor: Thin crawl handlers (Phase 5)"`

### Post-Refactoring

- [ ] Run full test suite: `cargo test --workspace`
- [ ] Run clippy: `cargo clippy --all -- -D warnings`
- [ ] Verify metrics still work
- [ ] Verify tracing still works
- [ ] Update documentation
- [ ] Create PR with all phases

---

## Success Metrics

### Code Quality

- [x] **Handler Size:** All handlers 20-30 lines (target: 222 lines total, down from 1,167 lines)
- [ ] **Business Logic:** 0 HTTP client calls in handlers
- [ ] **Error Handling:** Centralized in facades
- [ ] **Testing:** 90%+ test coverage on facades

### Functional Correctness

- [ ] **All Tests Pass:** `cargo test --workspace`
- [ ] **No Warnings:** `cargo clippy --all -- -D warnings`
- [ ] **Metrics Preserved:** Prometheus metrics unchanged
- [ ] **Tracing Preserved:** OpenTelemetry spans unchanged

### Architecture

- [ ] **Clean Layering:** Handlers → Facades → Services → Infra
- [ ] **No Circular Dependencies:** riptide-api → riptide-facade (one-way)
- [ ] **SOLID Principles:** Single responsibility, Open/closed, Dependency inversion

---

## Summary

**Problem:** Handlers contain 820 lines of business logic that should be in facades.

**Solution:** Refactor 6 handlers to thin pattern (20-30 lines each) by:
1. Moving HTTP client logic to facades
2. Moving validation logic to facades
3. Moving orchestration logic to facades
4. Keeping only HTTP concerns in handlers

**Result:**
- **Before:** 1,167 lines in handlers
- **After:** 222 lines in handlers
- **Moved:** 820 lines to facades
- **Improvement:** 81% reduction in handler complexity

**Next Steps:**
1. Review this design document
2. Execute Phase 1 (Extract Handler)
3. Validate with tests
4. Continue with Phases 2-5

---

**End of Design Document**
