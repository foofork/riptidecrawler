use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use trek_rs::{Trek, TrekOptions, TrekResponse};

mod trek_helpers;
use trek_helpers::*;

// Generate bindings from enhanced WIT file
wit_bindgen::generate!({
    world: "extractor",
    path: "wit",
});

// Export the Component Model interface
export!(Component);

/// Global extraction counter for tracking component usage
static EXTRACTION_COUNT: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));

/// Component state for caching and metrics
static COMPONENT_STATE: Lazy<std::sync::Mutex<ComponentState>> =
    Lazy::new(|| std::sync::Mutex::new(ComponentState::new()));

/// Internal component state
struct ComponentState {
    // Note: Trek doesn't implement Clone, so we create fresh instances
    memory_usage: u64,
    #[allow(dead_code)]
    start_time: std::time::Instant,
}

impl ComponentState {
    fn new() -> Self {
        Self {
            // No caching needed
            memory_usage: 0,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Create a new trek extractor for the given mode
fn create_extractor(mode: &ExtractionMode) -> Trek {
    let options = match mode {
        ExtractionMode::Article => TrekOptions {
            debug: false,
            url: None,
            output: trek_rs::types::OutputOptions {
                markdown: true,
                separate_markdown: true,
            },
            removal: trek_rs::types::RemovalOptions {
                remove_exact_selectors: true,
                remove_partial_selectors: true,
            },
        },
        ExtractionMode::Full => TrekOptions {
            debug: false,
            url: None,
            output: trek_rs::types::OutputOptions {
                markdown: false,
                separate_markdown: false,
            },
            removal: trek_rs::types::RemovalOptions {
                remove_exact_selectors: false,
                remove_partial_selectors: false,
            },
        },
        ExtractionMode::Metadata | ExtractionMode::Custom(_) => TrekOptions::default(),
    };

    Trek::new(options)
}
#[allow(dead_code)]
fn mode_to_cache_key(mode: &ExtractionMode) -> String {
    match mode {
        ExtractionMode::Article => "article".to_string(),
        ExtractionMode::Full => "full".to_string(),
        ExtractionMode::Metadata => "metadata".to_string(),
        ExtractionMode::Custom(selectors) => {
            format!("custom:{}", selectors.join(","))
        }
    }
}

/// Component build information
const COMPONENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const COMPONENT_NAME: &str = env!("CARGO_PKG_NAME");

/// Get build timestamp with fallback
fn get_build_timestamp() -> &'static str {
    option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown")
}

/// Get git commit with fallback
fn get_git_commit() -> &'static str {
    option_env!("VERGEN_GIT_SHA").unwrap_or("unknown")
}

struct Component;

impl Guest for Component {
    /// Primary extraction function with enhanced error handling and trek-rs integration
    fn extract(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        let _start_time = std::time::Instant::now();

        // Increment extraction counter
        EXTRACTION_COUNT.fetch_add(1, Ordering::Relaxed);

        // Validate HTML input
        if html.trim().is_empty() {
            return Err(ExtractionError::InvalidHtml(
                "Empty HTML content".to_string(),
            ));
        }

        // Validate URL format
        if url::Url::parse(&url).is_err() {
            return Err(ExtractionError::InvalidHtml(
                "Invalid URL format".to_string(),
            ));
        }

        // Perform extraction with trek-rs
        perform_extraction_with_trek(&html, &url, &mode)
    }

    /// Extract content with detailed performance statistics
    fn extract_with_stats(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<(ExtractedContent, ExtractionStats), ExtractionError> {
        let start_time = std::time::Instant::now();
        let initial_memory = get_memory_usage();

        let content = Self::extract(html.clone(), url, mode)?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        let memory_used = get_memory_usage().saturating_sub(initial_memory);
        let links_found = content.links.len() as u32;
        let images_found = content.media.len() as u32;

        // Count DOM nodes (approximate)
        let nodes_processed = Some(count_html_nodes(&html));

        let stats = ExtractionStats {
            processing_time_ms: processing_time,
            memory_used,
            nodes_processed,
            links_found,
            images_found,
        };

        Ok((content, stats))
    }

    /// Validate HTML content without full extraction
    fn validate_html(html: String) -> Result<bool, ExtractionError> {
        if html.trim().is_empty() {
            return Ok(false);
        }

        // Basic HTML validation
        let html_lower = html.to_lowercase();
        let has_html_tags = html_lower.contains("<html") || html_lower.contains("<!doctype");
        let has_body = html_lower.contains("<body");
        let has_content_tags = html_lower.contains("<p>")
            || html_lower.contains("<div")
            || html_lower.contains("<article")
            || html_lower.contains("<main");

        Ok(has_html_tags && (has_body || has_content_tags))
    }

    /// Health check for component monitoring
    fn health_check() -> HealthStatus {
        HealthStatus {
            status: "healthy".to_string(),
            version: COMPONENT_VERSION.to_string(),
            trek_version: get_trek_version(),
            capabilities: get_supported_modes(),
            memory_usage: Some(get_memory_usage()),
            extraction_count: Some(EXTRACTION_COUNT.load(Ordering::Relaxed)),
        }
    }

    /// Get detailed component information
    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: COMPONENT_NAME.to_string(),
            version: COMPONENT_VERSION.to_string(),
            component_model_version: "0.2.0".to_string(),
            features: vec![
                "article-extraction".to_string(),
                "full-page-extraction".to_string(),
                "metadata-extraction".to_string(),
                "custom-selectors".to_string(),
                "trek-rs-integration".to_string(),
            ],
            supported_modes: get_supported_modes(),
            build_timestamp: Some(get_build_timestamp().to_string()),
            git_commit: Some(get_git_commit().to_string()),
        }
    }

    /// Reset component state and clear caches
    fn reset_state() -> Result<String, ExtractionError> {
        // Reset extraction counter
        EXTRACTION_COUNT.store(0, Ordering::Relaxed);

        // Clear caches and reset state
        if let Ok(mut state) = COMPONENT_STATE.lock() {
            let old_count = EXTRACTION_COUNT.load(Ordering::Relaxed);
            // No cache to clear
            state.memory_usage = 0;
            EXTRACTION_COUNT.store(0, Ordering::Relaxed);
            Ok(format!(
                "Component state reset successfully. Previous extraction count: {}",
                old_count
            ))
        } else {
            Err(ExtractionError::InternalError(
                "Failed to acquire state lock".to_string(),
            ))
        }
    }

    /// Get supported extraction modes
    fn get_modes() -> Vec<String> {
        get_supported_modes()
    }
}

/// Perform content extraction with trek-rs integration
fn perform_extraction_with_trek(
    html: &str,
    url: &str,
    mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    // Get or create an extractor for this mode
    let extractor = {
        match COMPONENT_STATE.lock() {
            Ok(_state) => create_extractor(mode),
            Err(_) => {
                return Err(ExtractionError::InternalError(
                    "Failed to acquire component state lock".to_string(),
                ));
            }
        }
    };

    // Perform extraction using trek-rs
    let response = match extractor.parse(html) {
        Ok(response) => response,
        Err(trek_error) => {
            return Err(ExtractionError::ExtractorError(format!(
                "Trek extraction failed: {}",
                trek_error
            )));
        }
    };

    // Convert trek-rs result to Component Model format
    convert_response_to_content(response, url, mode)
}

/// Convert trek-rs TrekResponse to Component Model ExtractedContent
fn convert_response_to_content(
    response: TrekResponse,
    url: &str,
    _mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    // Calculate quality score based on content richness
    let quality_score = calculate_quality_score(&response);
    let word_count = response.metadata.word_count as u32;
    let reading_time = estimate_reading_time(response.metadata.word_count);

    Ok(ExtractedContent {
        url: url.to_string(),
        title: Some(response.metadata.title).filter(|s| !s.is_empty()),
        byline: Some(response.metadata.author).filter(|s| !s.is_empty()),
        published_iso: Some(response.metadata.published).filter(|s| !s.is_empty()),
        markdown: response.content_markdown.unwrap_or_default(),
        text: response.content,
        links: vec![],  // TODO: Extract links from content
        media: vec![],  // TODO: Extract media URLs
        language: None, // TODO: Language detection
        reading_time,
        quality_score: Some(quality_score),
        word_count: Some(word_count),
        categories: vec![], // TODO: Category extraction
        site_name: Some(response.metadata.site).filter(|s| !s.is_empty()),
        description: Some(response.metadata.description).filter(|s| !s.is_empty()),
    })
}

/// Get supported extraction modes
fn get_supported_modes() -> Vec<String> {
    vec![
        "article - Extract article content using readability algorithms".to_string(),
        "full - Extract full page content including sidebars and navigation".to_string(),
        "metadata - Extract only metadata (title, description, structured data)".to_string(),
        "custom - Custom extraction using provided CSS selectors".to_string(),
    ]
}

/// Estimate memory usage (simplified implementation)
fn get_memory_usage() -> u64 {
    // In a real implementation, this would use platform-specific APIs
    // For now, return a placeholder value
    1024 * 1024 // 1MB placeholder
}

/// Count HTML nodes for statistics
fn count_html_nodes(html: &str) -> u32 {
    // Simple node counting by counting opening tags
    html.matches('<').count() as u32
}
