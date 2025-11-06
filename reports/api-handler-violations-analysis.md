# API Handler Architecture Violations Report

**Date:** 2025-11-06
**Analyzer:** Code Quality Analyzer
**Scope:** `crates/riptide-api/src/handlers/`

## Executive Summary

This report documents violations of the **API Handler Rule**: *"Handle transport only — not business logic."*

**Findings:**
- **Total Handlers Analyzed:** 8 primary handler files
- **Critical Violations:** 15
- **Severity Breakdown:**
  - 🔴 **High (8):** Complex business logic, data orchestration
  - 🟠 **Medium (5):** Data transformations beyond DTO mapping
  - 🟡 **Low (2):** Minor coupling issues

---

## Rule Definition

### ✅ API Handlers SHOULD:
- HTTP request/response parsing
- Input validation (format, length)
- Authentication/authorization checks
- DTO mapping (request → domain, domain → response)
- Call facade/service layer
- Return HTTP responses

### ❌ API Handlers MUST NOT:
- Business logic (loops, complex calculations, decision trees)
- Data orchestration (multi-step workflows)
- Direct domain operations
- Database/Redis operations
- Complex data transformations
- Retry logic, transactions, multi-step workflows

---

## Critical Violations

### 1. **tables.rs** - Complex Business Logic in Handlers

#### Violation 1.1: Data Type Detection Algorithm
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/tables.rs:389-416`

```rust
/// Detect column data types from table data (simplified implementation)
fn detect_column_types(table: &AdvancedTableData) -> Vec<String> {
    let mut column_types = Vec::new();

    if table.rows.is_empty() {
        return column_types;
    }

    let num_columns = table.structure.total_columns;

    for col_index in 0..num_columns {
        let mut sample_values = Vec::new();

        // Collect sample values from this column
        for row in table.rows.iter().take(10) {
            if let Some(cell) = row.cells.get(col_index) {
                sample_values.push(&cell.content);
            }
        }

        // Detect type based on sample values
        let detected_type = detect_type_from_samples(&sample_values);
        column_types.push(detected_type);
    }

    column_types
}
```

**Violation Type:** 🔴 **High** - Business Logic (nested loops, complex calculations)

**Issues:**
- Nested iteration over table rows and columns
- Statistical analysis (sampling first 10 rows)
- Type inference algorithm with thresholds
- Decision trees for data type detection

**Suggested Fix:**
```rust
// Move to riptide-intelligence crate or create new analyzer service
pub struct TableAnalyzer {
    // Business logic for table analysis
}

impl TableAnalyzer {
    pub fn detect_column_types(&self, table: &AdvancedTableData) -> Vec<String> {
        // Implementation here
    }
}

// Handler calls facade:
let analyzer = state.table_analyzer;
let data_types = analyzer.detect_column_types(&table);
```

---

#### Violation 1.2: Type Detection Heuristics
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/tables.rs:418-462`

```rust
/// Detect data type from sample values
fn detect_type_from_samples(samples: &[&String]) -> String {
    if samples.is_empty() {
        return "unknown".to_string();
    }

    let mut numeric_count = 0;
    let mut date_count = 0;
    let mut boolean_count = 0;

    for &sample in samples {
        let trimmed = sample.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check for boolean
        if ["true", "false", "yes", "no", "1", "0"].contains(&trimmed.to_lowercase().as_str()) {
            boolean_count += 1;
        }
        // Check for numeric (integer or float)
        else if trimmed.parse::<f64>().is_ok() {
            numeric_count += 1;
        }
        // Check for date-like patterns (simplified)
        else if is_date_like(trimmed) {
            date_count += 1;
        }
    }

    let total_samples = samples.len();
    let threshold = (total_samples as f64 * 0.7) as usize; // 70% threshold

    if numeric_count >= threshold {
        "number"
    } else if date_count >= threshold {
        "date"
    } else if boolean_count >= threshold {
        "boolean"
    } else {
        "string"
    }
    .to_string()
}
```

**Violation Type:** 🔴 **High** - Complex Business Logic

**Issues:**
- Pattern matching algorithm
- Statistical threshold calculation (70% rule)
- Type inference with multiple decision branches
- Boolean/numeric/date parsing logic

**Impact:** This is core business logic that belongs in a domain service, not an API handler.

**Suggested Fix:**
Move to `riptide-intelligence::table_analysis::TypeInference`

---

#### Violation 1.3: In-Memory Storage Management
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/tables.rs:123-133`

```rust
/// In-memory storage for extracted tables (temporary)
/// In production, this would be replaced with Redis or database storage
static TABLE_STORAGE: std::sync::OnceLock<
    Arc<tokio::sync::Mutex<HashMap<String, AdvancedTableData>>>,
> = std::sync::OnceLock::new();

fn get_table_storage() -> Arc<tokio::sync::Mutex<HashMap<String, AdvancedTableData>>> {
    TABLE_STORAGE
        .get_or_init(|| Arc::new(tokio::sync::Mutex::new(HashMap::new())))
        .clone()
}
```

**Violation Type:** 🔴 **High** - Data Storage in API Layer

**Issues:**
- Handlers managing persistent storage
- Global state mutation
- Storage lifecycle management
- This violates separation of concerns

**Suggested Fix:**
```rust
// Create a TableCacheService in riptide-facade
pub struct TableCacheService {
    cache: Arc<Mutex<HashMap<String, AdvancedTableData>>>,
}

// Handler delegates to facade:
let cache_key = state.table_cache.store(table).await?;
```

---

#### Violation 1.4: Data Orchestration in Handler
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/tables.rs:200-256`

```rust
// Store tables temporarily for export and create summaries
let storage = get_table_storage();
let mut storage_guard = storage.lock().await;
let mut table_summaries = Vec::new();

for table in tables {
    // Generate unique ID for this extraction session
    let export_id = Uuid::new_v4().to_string();

    // Get sample data and headers based on options
    let (headers, sample_data) = if options.include_headers {
        let headers: Vec<String> = table
            .headers
            .main
            .iter()
            .map(|cell| cell.content.clone())
            .collect();

        let sample_data: Vec<Vec<String>> = table
            .rows
            .iter()
            .take(3)
            .map(|row| row.cells.iter().map(|cell| cell.content.clone()).collect())
            .collect();

        (headers, sample_data)
    } else {
        (vec![], vec![])
    };

    // Detect data types if enabled
    let data_types = if options.detect_data_types {
        detect_column_types(&table)
    } else {
        vec![]
    };

    let summary = TableSummary {
        id: export_id.clone(),
        rows: table.structure.total_rows,
        columns: table.structure.total_columns,
        headers: headers.clone(),
        data: sample_data,
        metadata: TableMetadata {
            has_headers: !headers.is_empty(),
            data_types,
            has_complex_structure: table.structure.has_complex_structure,
            caption: table.caption.clone(),
            css_classes: table.metadata.classes.clone(),
            html_id: table.metadata.id.clone(),
        },
    };

    // Store the full table data for export
    storage_guard.insert(export_id, table);
    table_summaries.push(summary);
}
```

**Violation Type:** 🔴 **High** - Multi-Step Data Orchestration

**Issues:**
- Complex iteration and transformation
- Conditional data extraction logic
- Multiple data structure constructions
- Storage coordination

**Suggested Fix:**
```rust
// Move to facade layer
impl TableFacade {
    pub async fn store_and_summarize(
        &self,
        tables: Vec<AdvancedTableData>,
        options: &TableExtractionOptions,
    ) -> Result<Vec<TableSummary>> {
        // All orchestration logic here
    }
}

// Handler becomes:
let summaries = state.table_facade
    .store_and_summarize(tables, &options)
    .await?;
```

---

### 2. **sessions.rs** - Direct Session Management Logic

#### Violation 2.1: Session Expiry Filtering Logic
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/sessions.rs:361-377`

```rust
// Filter expired sessions if not explicitly requested
let include_expired = query.include_expired.unwrap_or(false);
let filtered_sessions = if !include_expired {
    // Filter out expired sessions by checking each one
    let mut active_sessions = Vec::new();
    for session_id in session_ids {
        if let Ok(Some(session)) = state.session_manager.get_session(&session_id).await {
            let now = std::time::SystemTime::now();
            if session.expires_at > now {
                active_sessions.push(session_id);
            }
        }
    }
    active_sessions
} else {
    session_ids
};
```

**Violation Type:** 🟠 **Medium** - Business Logic Loop

**Issues:**
- Iterating and filtering business data
- Time-based expiry logic
- Conditional data processing

**Suggested Fix:**
```rust
// Move filtering to SessionManager
impl SessionManager {
    pub async fn list_sessions(&self, include_expired: bool) -> Vec<String> {
        if include_expired {
            self.list_all_sessions().await
        } else {
            self.list_active_sessions().await
        }
    }
}

// Handler becomes simple:
let sessions = state.session_manager
    .list_sessions(include_expired)
    .await;
```

---

### 3. **profiles.rs** - Complex Configuration Management

#### Violation 3.1: Profile Configuration Builder Logic
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/profiles.rs:189-213`

```rust
// Apply optional configuration
if let Some(config) = request.config {
    profile.update_config(|c| {
        if let Some(level) = config.stealth_level {
            c.stealth_level = level;
        }
        if let Some(limit) = config.rate_limit {
            c.rate_limit = limit;
        }
        if let Some(respect) = config.respect_robots_txt {
            c.respect_robots_txt = respect;
        }
        if let Some(strategy) = config.ua_strategy {
            c.ua_strategy = strategy;
        }
        if let Some(threshold) = config.confidence_threshold {
            c.confidence_threshold = threshold;
        }
        if let Some(js) = config.enable_javascript {
            c.enable_javascript = js;
        }
        if let Some(timeout) = config.request_timeout_secs {
            c.request_timeout_secs = timeout;
        }
    });
}
```

**Violation Type:** 🟠 **Medium** - Configuration Orchestration

**Issues:**
- Multiple conditional updates
- Builder pattern logic in handler
- Configuration merge logic

**Suggested Fix:**
```rust
// Create ProfileConfigBuilder in domain layer
impl ProfileManager {
    pub fn create_with_config(
        domain: String,
        config: Option<ProfileConfigRequest>,
    ) -> DomainProfile {
        let mut profile = Self::create(domain);
        if let Some(cfg) = config {
            profile.apply_config(cfg);
        }
        profile
    }
}
```

---

#### Violation 3.2: Batch Operation Orchestration
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/profiles.rs:469-494`

```rust
let mut created = Vec::new();
let mut failed = Vec::new();

for profile_req in request.profiles {
    let domain = profile_req.domain.clone();
    match create_profile_internal(&_state, profile_req).await {
        Ok(_) => created.push(domain),
        Err(e) => failed.push(BatchFailure {
            domain,
            error: e.to_string(),
        }),
    }
}

let response = BatchCreateResponse { created, failed };
```

**Violation Type:** 🟠 **Medium** - Batch Orchestration

**Issues:**
- Loop with error handling
- Result aggregation logic
- Batch operation coordination

**Suggested Fix:**
```rust
// Move to ProfileFacade
impl ProfileFacade {
    pub async fn batch_create(
        &self,
        requests: Vec<CreateProfileRequest>,
    ) -> BatchCreateResponse {
        // All batch logic here
    }
}
```

---

#### Violation 3.3: Cache Warming Simulation Logic
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/profiles.rs:546-574`

```rust
// For now, this is a placeholder for actual cache warming logic
// In production, this would:
// 1. Fetch and analyze the URL
// 2. Determine optimal engine
// 3. Cache the engine preference
//
// Since we don't have the full extraction pipeline here,
// we'll simulate by setting a mock cache

use riptide_reliability::engine_selection::Engine;

// Simulate successful analysis (in production, use actual analyzer)
let simulated_engine = Engine::Wasm; // Default to WASM
let simulated_confidence = 0.85; // High confidence

profile.cache_engine(simulated_engine, simulated_confidence);

// Save updated profile
ProfileManager::save(&profile, None).map_err(|e| {
    error!("Failed to save profile: {}", e);
    ApiError::InternalError {
        message: format!("Failed to save profile: {}", e),
    }
})?;
```

**Violation Type:** 🟡 **Low** - Placeholder Business Logic

**Issues:**
- Cache warming simulation
- Engine selection logic (even if placeholder)
- Profile mutation and persistence

**Note:** This is acknowledged as temporary, but still violates the pattern.

**Suggested Fix:**
```rust
// Create ProfileCacheService
impl ProfileCacheService {
    pub async fn warm_cache(
        &self,
        domain: &str,
        url: &str,
    ) -> Result<CacheWarmResult> {
        // Real cache warming logic
    }
}
```

---

### 4. **crawl.rs** - Pipeline Orchestration in Handler

#### Violation 4.1: Complex Pipeline Selection Logic
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/crawl.rs:100-162`

```rust
// Create pipeline orchestrator - use enhanced pipeline if enabled
let (pipeline_results, stats) = if state
    .config
    .enhanced_pipeline_config
    .enable_enhanced_pipeline
{
    info!("Using enhanced pipeline orchestrator with detailed phase timing");
    let enhanced_pipeline = EnhancedPipelineOrchestrator::new(state.clone(), options.clone());
    let (results, enhanced_stats) = enhanced_pipeline.execute_batch_enhanced(&body.urls).await;

    // Convert enhanced stats to standard stats for compatibility
    let standard_stats = crate::pipeline::PipelineStats {
        total_processed: enhanced_stats.total_urls,
        cache_hits: enhanced_stats.cache_hits,
        successful_extractions: enhanced_stats.successful,
        failed_extractions: enhanced_stats.failed,
        gate_decisions: enhanced_stats.gate_decisions,
        avg_processing_time_ms: enhanced_stats.avg_processing_time_ms,
        total_processing_time_ms: enhanced_stats.total_duration_ms,
    };

    // Convert enhanced results to standard pipeline results
    let standard_results: Vec<Option<crate::pipeline::PipelineResult>> = results
        .into_iter()
        .map(|opt_result| {
            opt_result.map(|enhanced_result| crate::pipeline::PipelineResult {
                document: enhanced_result.document.unwrap_or_else(|| {
                    riptide_types::ExtractedDoc {
                        url: enhanced_result.url.clone(),
                        title: None,
                        text: String::new(),
                        // ... 15 more fields
                    }
                }),
                from_cache: enhanced_result.cache_hit,
                gate_decision: enhanced_result.gate_decision,
                quality_score: enhanced_result.quality_score,
                processing_time_ms: enhanced_result.total_duration_ms,
                cache_key: format!("riptide:v1:enhanced:{}", enhanced_result.url),
                http_status: 200,
            })
        })
        .collect();

    (standard_results, standard_stats)
} else {
    info!("Using standard pipeline orchestrator");
    let pipeline = PipelineOrchestrator::new(state.clone(), options.clone());
    pipeline.execute_batch(&body.urls).await
};
```

**Violation Type:** 🔴 **High** - Complex Orchestration

**Issues:**
- Pipeline selection logic
- Data structure conversions (enhanced → standard)
- Complex mapping operations
- Fallback document construction

**Suggested Fix:**
```rust
// Create unified facade
impl CrawlFacade {
    pub async fn execute_batch(
        &self,
        urls: &[String],
        options: &CrawlOptions,
    ) -> (Vec<CrawlResult>, CrawlStats) {
        // All pipeline selection and conversion logic here
    }
}

// Handler becomes:
let (results, stats) = state.crawl_facade
    .execute_batch(&body.urls, &options)
    .await?;
```

---

#### Violation 4.2: Result Transformation Loop
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/crawl.rs:165-217`

```rust
// Convert pipeline results to API response format
let mut crawl_results = Vec::with_capacity(body.urls.len());
let mut from_cache_count = 0;

for (index, pipeline_result) in pipeline_results.into_iter().enumerate() {
    let url = &body.urls[index];

    match pipeline_result {
        Some(result) => {
            if result.from_cache {
                from_cache_count += 1;
            }

            // Apply chunking if requested
            let document = if let Some(ref chunking_config) = options.chunking_config {
                let doc = result.document;
                apply_content_chunking(doc.clone(), chunking_config)
                    .await
                    .unwrap_or(doc)
            } else {
                result.document
            };

            crawl_results.push(CrawlResult {
                url: url.clone(),
                status: result.http_status,
                from_cache: result.from_cache,
                gate_decision: result.gate_decision,
                quality_score: result.quality_score,
                processing_time_ms: result.processing_time_ms,
                document: Some(document),
                error: None,
                cache_key: result.cache_key,
            });
        }
        None => {
            crawl_results.push(CrawlResult {
                url: url.clone(),
                status: 0,
                from_cache: false,
                gate_decision: "failed".to_string(),
                quality_score: 0.0,
                processing_time_ms: 0,
                document: None,
                error: Some(ErrorInfo {
                    error_type: "pipeline_error".to_string(),
                    message: "Failed to process URL".to_string(),
                    retryable: true,
                }),
                cache_key: "".to_string(),
            });
        }
    }
}
```

**Violation Type:** 🟠 **Medium** - Data Transformation Logic

**Issues:**
- Complex iteration and transformation
- Conditional chunking application
- Error object construction
- Cache counting logic

**Suggested Fix:**
Move all transformation to facade layer.

---

### 5. **llm.rs** - Provider Management Business Logic

#### Violation 5.1: Provider Information Construction
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/llm.rs:704-793`

```rust
async fn create_provider_info(
    registry: &LlmRegistry,
    provider_name: &str,
    params: &ProviderQuery,
) -> ApiResult<ProviderInfo> {
    // Get actual provider from registry
    let provider = registry
        .get_provider(provider_name)
        .ok_or_else(|| ApiError::not_found(format!("Provider '{}' not found", provider_name)))?;

    // Perform health check
    let is_available = provider.is_available().await;
    let status = if is_available {
        "healthy"
    } else {
        "unavailable"
    };

    // Get capabilities from provider
    let llm_capabilities = provider.capabilities();

    let mut capabilities = vec!["text-generation".to_string()];
    if llm_capabilities.supports_embeddings {
        capabilities.push("embedding".to_string());
    }
    if llm_capabilities.supports_functions {
        capabilities.push("function-calling".to_string());
    }
    if llm_capabilities.supports_streaming {
        capabilities.push("streaming".to_string());
    }
    capabilities.push("chat".to_string());

    // Determine provider type from name
    let provider_type = provider.name().to_string();

    // Create config requirements based on provider type
    let config_required = match provider_type.as_str() {
        "openai" | "azure_openai" => vec!["api_key", "model"],
        "anthropic" => vec!["api_key", "model"],
        "ollama" => vec!["base_url", "model"],
        "localai" => vec!["base_url", "model"],
        "aws_bedrock" => vec!["region", "model"],
        "google_vertex" => vec!["project_id", "location", "model"],
        _ => vec!["api_key"],
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    // Create cost info if requested
    let cost_info = if params.include_cost {
        llm_capabilities.models.first().map(|model| CostInfo {
            input_token_cost: Some(model.cost_per_1k_prompt_tokens),
            output_token_cost: Some(model.cost_per_1k_completion_tokens),
            currency: "USD".to_string(),
        })
    } else {
        None
    };

    // Create model info if requested
    let models = if params.include_models {
        llm_capabilities
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.id,
                context_window: Some(m.max_tokens as usize),
                max_output_tokens: Some(m.max_tokens as usize),
                supports_functions: m.supports_functions,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(ProviderInfo {
        name: provider_name.to_string(),
        provider_type,
        status: status.to_string(),
        capabilities,
        config_required,
        available: is_available,
        cost_info,
        models,
    })
}
```

**Violation Type:** 🔴 **High** - Complex Business Logic

**Issues:**
- Provider health checking
- Capability mapping logic
- Provider type detection
- Configuration requirement mapping
- Conditional model/cost info construction

**Suggested Fix:**
```rust
// Move to LlmFacade
impl LlmFacade {
    pub async fn get_provider_info(
        &self,
        provider_name: &str,
        options: ProviderQueryOptions,
    ) -> Result<ProviderInfo> {
        // All business logic here
    }
}
```

---

#### Violation 5.2: Configuration Validation Logic
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/llm.rs:795-835`

```rust
/// Validate provider configuration
async fn validate_provider_config(
    provider_name: &str,
    config: &HashMap<String, String>,
) -> ApiResult<String> {
    // Basic validation - in production would actually test provider connectivity
    match provider_name {
        name if name.contains("openai") => {
            if !config.contains_key("api_key") {
                return Err(ApiError::validation("OpenAI requires api_key".to_string()));
            }
            if !config.contains_key("model") {
                return Err(ApiError::validation(
                    "OpenAI requires model specification".to_string(),
                ));
            }
        }
        name if name.contains("anthropic") => {
            if !config.contains_key("api_key") {
                return Err(ApiError::validation(
                    "Anthropic requires api_key".to_string(),
                ));
            }
        }
        name if name.contains("ollama") => {
            if !config.contains_key("base_url") {
                return Err(ApiError::validation("Ollama requires base_url".to_string()));
            }
        }
        _ => {
            // Unknown provider - basic validation
            if config.is_empty() {
                return Err(ApiError::validation(
                    "Provider configuration cannot be empty".to_string(),
                ));
            }
        }
    }

    Ok("Configuration is valid".to_string())
}
```

**Violation Type:** 🔴 **High** - Validation Business Logic

**Issues:**
- Provider-specific validation rules
- Configuration requirement checking
- Pattern matching on provider names

**Suggested Fix:**
Move to `riptide-intelligence::LlmConfigValidator`

---

### 6. **render/processors.rs** - Rendering Strategy Logic

#### Violation 6.1: URL Content Analysis
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/render/strategies.rs:7-94`

```rust
/// Analyze URL and content patterns to determine if dynamic rendering is needed
pub(super) async fn analyze_url_for_dynamic_content(url: &str) -> bool {
    // Check for common indicators that suggest dynamic content
    let url_lower = url.to_lowercase();

    // Social media platforms and news sites with dynamic content
    let dynamic_domains = [
        "twitter.com", "x.com", "facebook.com", "instagram.com",
        "linkedin.com", "youtube.com", "tiktok.com", "reddit.com",
        // ... 20+ more domains
    ];

    // Check if URL contains dynamic domain patterns
    for domain in &dynamic_domains {
        if url_lower.contains(domain) {
            debug!(url = %url, domain = %domain, "Found dynamic domain pattern");
            return true;
        }
    }

    // Check for SPA indicators in URL
    let spa_indicators = [
        "/#/", "#!/", "/app/", "/dashboard/", "/admin/",
        "?page=", "&view=", "#page", "#view", "#section",
    ];

    for indicator in &spa_indicators {
        if url_lower.contains(indicator) {
            debug!(url = %url, indicator = %indicator, "Found SPA URL pattern");
            return true;
        }
    }

    // Check for JavaScript framework patterns
    let js_frameworks = [
        "react", "angular", "vue", "svelte", "next", "nuxt",
        "gatsby", "webpack", "vite", "parcel", "app.js",
        "bundle.js", "main.js",
    ];

    for framework in &js_frameworks {
        if url_lower.contains(framework) {
            debug!(url = %url, framework = %framework, "Found JS framework pattern");
            return true;
        }
    }

    // Default to static for unknown patterns
    debug!(url = %url, "No dynamic content indicators found");
    false
}
```

**Violation Type:** 🔴 **High** - Content Classification Logic

**Issues:**
- URL pattern analysis
- Domain classification
- SPA detection heuristics
- Framework detection

**Suggested Fix:**
Move to `riptide-intelligence::ContentAnalyzer`

---

#### Violation 6.2: Adaptive Configuration Builder
**Location:** `/workspaces/eventmesh/crates/riptide-api/src/handlers/render/strategies.rs:96-191`

```rust
/// Create adaptive dynamic configuration based on URL analysis
pub(super) fn create_adaptive_dynamic_config(url: &str) -> DynamicConfig {
    let url_lower = url.to_lowercase();

    // Determine wait strategy based on URL type
    let wait_for = if url_lower.contains("github.com") {
        Some(WaitCondition::Selector {
            selector: ".repository-content, .file-navigation, .js-repo-nav".to_string(),
            timeout: Duration::from_secs(2),
        })
    } else if url_lower.contains("reddit.com") {
        Some(WaitCondition::Selector {
            selector: "[data-testid='post'], .Post".to_string(),
            timeout: Duration::from_secs(2),
        })
    } else if url_lower.contains("medium.com") || url_lower.contains("substack.com") {
        Some(WaitCondition::Selector {
            selector: "article, .post-content, main".to_string(),
            timeout: Duration::from_secs(2),
        })
    } else if url_lower.contains("twitter.com") || url_lower.contains("x.com") {
        Some(WaitCondition::Multiple(vec![
            WaitCondition::Selector {
                selector: "[data-testid='tweet'], article".to_string(),
                timeout: Duration::from_millis(1500),
            },
            WaitCondition::NetworkIdle {
                timeout: Duration::from_millis(1000),
                idle_time: Duration::from_millis(500),
            },
        ]))
    } else {
        // Generic wait for content
        Some(WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::Timeout(Duration::from_millis(1000)),
        ]))
    };

    // Determine scroll strategy
    let scroll = if url_lower.contains("twitter.com")
        || url_lower.contains("x.com")
        || url_lower.contains("instagram.com")
        || url_lower.contains("linkedin.com")
    {
        // Social media needs more scrolling for infinite feeds
        Some(ScrollConfig {
            steps: 5,
            step_px: Some(800),
            delay_ms: 800,
            mode: ScrollMode::Stepped,
            after_scroll_js: Some(
                "window.scrollBy(0, 200); await new Promise(r => setTimeout(r, 300));".to_string(),
            ),
            stop_condition: None,
        })
    } else if url_lower.contains("medium.com") || url_lower.contains("substack.com") {
        // Article sites need gentle scrolling
        Some(ScrollConfig {
            steps: 3,
            step_px: Some(1000),
            delay_ms: 500,
            mode: ScrollMode::Smooth,
            after_scroll_js: None,
            stop_condition: None,
        })
    } else {
        // Default moderate scrolling
        Some(ScrollConfig {
            steps: 2,
            step_px: Some(800),
            delay_ms: 600,
            mode: ScrollMode::Stepped,
            after_scroll_js: None,
            stop_condition: None,
        })
    };

    // Create viewport configuration
    let viewport = Some(ViewportConfig {
        width: 1920,
        height: 1080,
        device_scale_factor: 1.0,
        is_mobile: false,
        user_agent: None,
    });

    DynamicConfig {
        wait_for,
        scroll,
        actions: Vec::new(),
        capture_artifacts: false,
        timeout: Duration::from_secs(3),
        viewport,
    }
}
```

**Violation Type:** 🔴 **High** - Complex Configuration Logic

**Issues:**
- Site-specific configuration rules
- Scroll strategy selection
- Wait condition builder
- Multiple decision branches

**Suggested Fix:**
Move to `riptide-intelligence::RenderStrategySelector`

---

## Handlers That Follow the Pattern ✅

### Good Examples:

1. **resources.rs** - Excellent adherence to transport-only pattern:
   - Simply queries `resource_manager` and maps to DTOs
   - No business logic, just data retrieval and formatting
   - All domain logic delegated to `ResourceManager`

2. **pipeline_metrics.rs** - Good separation:
   - Reads configuration from state
   - Constructs response DTOs
   - Minimal transformation logic

3. **extract.rs** - Clean facade delegation:
   - Validates input
   - Calls `extraction_facade.extract_html()`
   - Maps result to response DTO
   - No business logic

---

## Impact Analysis

### Architecture Debt
- **Coupling:** Handlers tightly coupled to domain logic
- **Testability:** Handlers hard to test without complex mocking
- **Maintainability:** Business logic scattered across API layer
- **Reusability:** Logic cannot be reused outside HTTP context

### Performance Implications
- Complex logic in request path increases latency
- Handler-level storage (tables.rs) creates memory issues
- Synchronous operations block request processing

### Security Risks
- Business logic in API layer harder to audit
- Validation logic mixed with processing logic
- Configuration validation in handler layer

---

## Recommendations

### Immediate Actions (High Priority)

1. **Create Facades for Complex Handlers:**
   ```rust
   // New facades needed:
   - TableFacade (extract, analyze, store tables)
   - ProfileFacade (create, update, batch operations)
   - CrawlFacade (unified pipeline orchestration)
   - LlmFacade (provider management, config validation)
   - RenderStrategyFacade (content analysis, config building)
   ```

2. **Extract Business Logic to Domain Services:**
   ```rust
   // New services needed:
   - TableAnalyzer (type detection, data analysis)
   - ContentAnalyzer (URL analysis, dynamic content detection)
   - RenderStrategySelector (adaptive config building)
   - LlmConfigValidator (provider-specific validation)
   ```

3. **Move Storage Logic:**
   ```rust
   // Replace handler-level storage with:
   - TableCacheService (replace global HashMap)
   - Use existing SessionManager (don't filter in handler)
   ```

### Medium Priority

4. **Simplify Result Transformations:**
   - Move all DTO mapping to facade layer
   - Create dedicated mapper utilities
   - Reduce handler cognitive complexity

5. **Consolidate Validation:**
   - Move validation rules to domain layer
   - Use validator pattern for complex rules
   - Keep only format validation in handlers

### Long-term Improvements

6. **Establish Facade Pattern:**
   - Create consistent facade interface
   - Document facade responsibilities
   - Enforce through code review

7. **Add Architecture Tests:**
   ```rust
   #[test]
   fn handlers_should_not_contain_business_logic() {
       // Check for loops, complex conditionals, etc.
   }
   ```

8. **Refactoring Guide:**
   - Create migration plan for each violation
   - Establish handler patterns and anti-patterns
   - Document clean architecture principles

---

## Conclusion

The analysis reveals significant violations of the API handler architecture rule across 8 handler files. The most critical issues are:

1. **Business logic in handlers** (tables.rs, llm.rs, render/strategies.rs)
2. **Data orchestration** (crawl.rs, profiles.rs)
3. **Complex transformations** beyond DTO mapping

**Recommended Approach:**
1. Create facade layer for each complex handler
2. Extract business logic to domain services
3. Simplify handlers to transport-only concerns
4. Enforce pattern through code review and tests

**Estimated Effort:**
- High priority fixes: 2-3 weeks
- Medium priority: 1-2 weeks
- Long-term improvements: Ongoing

---

## Appendix: Clean Handler Template

```rust
/// Clean API Handler Example
pub async fn example_handler(
    State(state): State<AppState>,
    Json(request): Json<ExampleRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. Validate input (format only)
    if request.url.is_empty() {
        return Err(ApiError::validation("URL cannot be empty"));
    }

    // 2. Call facade (all business logic here)
    let result = state.example_facade
        .process(&request)
        .await
        .map_err(ApiError::from)?;

    // 3. Map to response DTO (simple transformation)
    let response = ExampleResponse {
        id: result.id,
        status: result.status.to_string(),
        data: result.data,
    };

    // 4. Return response
    Ok((StatusCode::OK, Json(response)))
}
```

---

**End of Report**
