use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use trek_rs::{Trek, TrekOptions, TrekResponse};

mod trek_helpers;
use trek_helpers::*;

mod common_validation;
use common_validation::*;

mod extraction;
use extraction::*;

// Generate bindings from enhanced WIT file
wit_bindgen::generate!({
    world: "extractor",
    path: "wit",
});

// WIT-generated types are automatically public and available

// Export the Component Model interface
export!(Component);

// Note: WIT-generated types are available after build

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

#[derive(Clone, Default)]
pub struct Component;

impl Component {
    /// Create a new component instance
    pub fn new() -> Self {
        Self
    }

    /// Extract content using the Guest trait implementation
    pub fn extract(
        &self,
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        self.extract_internal(html, url, mode)
    }

    /// Extract content with detailed statistics
    pub fn extract_with_stats(
        &self,
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<(ExtractedContent, ExtractionStats), ExtractionError> {
        self.extract_with_stats_internal(html, url, mode)
    }

    /// Validate HTML content without full extraction
    pub fn validate_html(&self, html: String) -> Result<bool, ExtractionError> {
        self.validate_html_internal(html)
    }

    /// Health check for component monitoring
    pub fn health_check(&self) -> HealthStatus {
        self.health_check_internal()
    }

    /// Get detailed component information
    pub fn get_info(&self) -> ComponentInfo {
        self.get_info_internal()
    }

    /// Reset component state and clear caches
    pub fn reset_state(&self) -> Result<String, ExtractionError> {
        self.reset_state_internal()
    }

    /// Get supported extraction modes
    pub fn get_modes(&self) -> Vec<String> {
        self.get_modes_internal()
    }
}

// Note: Validation logic moved to common_validation module

impl Component {
    /// Internal extraction function
    fn extract_internal(
        &self,
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        // Timing measurement removed - not used in production
        // Could be re-added with feature flag for profiling if needed

        // Increment extraction counter
        EXTRACTION_COUNT.fetch_add(1, Ordering::Relaxed);

        // Use common validation logic
        validate_extraction_input(&html, &url)?;
        validate_content_size(html.len())?;
        validate_extraction_mode(&mode)?;

        // Perform enhanced extraction with all features
        perform_enhanced_extraction(&html, &url, &mode)
    }

    /// Extract content with detailed performance statistics
    fn extract_with_stats_internal(
        &self,
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<(ExtractedContent, ExtractionStats), ExtractionError> {
        let start_time = std::time::Instant::now();
        let initial_memory = get_memory_usage();

        let content = self.extract_internal(html.clone(), url, mode)?;

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
    fn validate_html_internal(&self, html: String) -> Result<bool, ExtractionError> {
        // Use common validation logic
        match validate_html_structure(&html) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Health check for component monitoring
    fn health_check_internal(&self) -> HealthStatus {
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
    fn get_info_internal(&self) -> ComponentInfo {
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
                "links-extraction".to_string(),
                "media-extraction".to_string(),
                "language-detection".to_string(),
                "category-extraction".to_string(),
                "url-resolution".to_string(),
            ],
            supported_modes: get_supported_modes(),
            build_timestamp: Some(get_build_timestamp().to_string()),
            git_commit: Some(get_git_commit().to_string()),
        }
    }

    /// Reset component state and clear caches
    fn reset_state_internal(&self) -> Result<String, ExtractionError> {
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
    fn get_modes_internal(&self) -> Vec<String> {
        get_supported_modes()
    }
}

impl Guest for Component {
    /// Primary extraction function with enhanced error handling and trek-rs integration
    fn extract(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        let component = Component::new();
        component.extract_internal(html, url, mode)
    }

    /// Extract content with detailed performance statistics
    fn extract_with_stats(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<(ExtractedContent, ExtractionStats), ExtractionError> {
        let component = Component::new();
        component.extract_with_stats_internal(html, url, mode)
    }

    /// Validate HTML content without full extraction
    fn validate_html(html: String) -> Result<bool, ExtractionError> {
        let component = Component::new();
        component.validate_html_internal(html)
    }

    /// Health check for component monitoring
    fn health_check() -> HealthStatus {
        let component = Component::new();
        component.health_check_internal()
    }

    /// Get detailed component information
    fn get_info() -> ComponentInfo {
        let component = Component::new();
        component.get_info_internal()
    }

    /// Reset component state and clear caches
    fn reset_state() -> Result<String, ExtractionError> {
        let component = Component::new();
        component.reset_state_internal()
    }

    /// Get supported extraction modes
    fn get_modes() -> Vec<String> {
        let component = Component::new();
        component.get_modes_internal()
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

    // For full extraction, we need the original HTML to extract additional features
    // Trek-rs provides the processed content, but we need to re-parse for links/media
    // Note: In a real implementation, we would need access to the original HTML
    // For now, we'll extract what we can from the processed content

    Ok(ExtractedContent {
        url: url.to_string(),
        title: Some(response.metadata.title).filter(|s| !s.is_empty()),
        byline: Some(response.metadata.author).filter(|s| !s.is_empty()),
        published_iso: Some(response.metadata.published).filter(|s| !s.is_empty()),
        markdown: response.content_markdown.unwrap_or_default(),
        text: response.content,
        links: vec![],  // Will be populated by enhanced extraction
        media: vec![],  // Will be populated by enhanced extraction
        language: None, // Will be populated by enhanced extraction
        reading_time,
        quality_score: Some(quality_score),
        word_count: Some(word_count),
        categories: vec![], // Will be populated by enhanced extraction
        site_name: Some(response.metadata.site).filter(|s| !s.is_empty()),
        description: Some(response.metadata.description).filter(|s| !s.is_empty()),
    })
}

/// Enhanced extraction function that combines trek-rs with our custom extractors
fn perform_enhanced_extraction(
    html: &str,
    url: &str,
    mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    // First, get the base extraction from trek-rs
    let mut content = perform_extraction_with_trek(html, url, mode)?;

    // Then enhance with our custom extractors
    content.links = extract_links(html, url);
    content.media = extract_media(html, url);
    content.language = detect_language(html);
    content.categories = extract_categories(html);

    // Recalculate quality score based on enhanced data
    content.quality_score = Some(calculate_enhanced_quality_score(&content));

    Ok(content)
}

/// Calculate enhanced quality score based on all extracted features
fn calculate_enhanced_quality_score(content: &ExtractedContent) -> u8 {
    let mut score = content.quality_score.unwrap_or(30);

    // Bonus points for rich link extraction
    let link_count = content.links.len();
    if link_count > 10 {
        score = score.saturating_add(10);
    } else if link_count > 5 {
        score = score.saturating_add(5);
    }

    // Bonus points for media content
    let media_count = content.media.len();
    if media_count > 5 {
        score = score.saturating_add(10);
    } else if media_count > 0 {
        score = score.saturating_add(5);
    }

    // Bonus for language detection
    if content.language.is_some() {
        score = score.saturating_add(5);
    }

    // Bonus for categories
    if !content.categories.is_empty() {
        score = score.saturating_add(5);
    }

    score.min(100)
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
