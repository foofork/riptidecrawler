use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};

mod extraction_helpers;
use extraction_helpers::*;

mod common_validation;
use common_validation::*;

// Extraction module with comprehensive link, media, language, and category extraction
mod extraction;

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
    memory_usage: u64,
    #[allow(dead_code)]
    start_time: std::time::Instant,
}

impl ComponentState {
    fn new() -> Self {
        Self {
            memory_usage: 0,
            start_time: std::time::Instant::now(),
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
            extractor_version: get_extractor_version(),
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
                "scraper-based-extraction".to_string(),
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

/// Perform content extraction using scraper (WASI-compatible, no browser APIs)
fn perform_extraction_with_scraper(
    html: &str,
    url: &str,
    mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    use scraper::Html;

    let document = Html::parse_document(html);

    // Extract title
    let title = extract_title(&document);

    // Extract metadata
    let byline = extract_meta_content(&document, &["author", "article:author"]);
    let published = extract_meta_content(&document, &["article:published_time", "datePublished"]);
    let site_name = extract_meta_content(&document, &["og:site_name", "twitter:site"]);
    let description = extract_meta_content(&document, &["description", "og:description"]);

    // Extract main content based on mode
    let text = match mode {
        ExtractionMode::Article => extract_article_content(&document),
        ExtractionMode::Full => extract_full_content(&document),
        ExtractionMode::Metadata => String::new(),
        ExtractionMode::Custom(selectors) => extract_custom_content(&document, selectors),
    };

    // Calculate word count and reading time
    let word_count = count_words(&text) as u32;
    let reading_time = estimate_reading_time(word_count as usize);

    // Calculate quality score
    let quality_score = calculate_basic_quality_score(
        title.as_ref().map(|t| t.len()).unwrap_or(0),
        text.len(),
        byline.is_some(),
        published.is_some(),
        word_count as usize,
    );

    Ok(ExtractedContent {
        url: url.to_string(),
        title,
        byline,
        published_iso: published,
        markdown: String::new(), // Markdown conversion can be added later if needed
        text,
        links: vec![],  // Will be populated by enhanced extraction
        media: vec![],  // Will be populated by enhanced extraction
        language: None, // Will be populated by enhanced extraction
        reading_time,
        quality_score: Some(quality_score),
        word_count: Some(word_count),
        categories: vec![], // Will be populated by enhanced extraction
        site_name,
        description,
    })
}

/// Extract title from HTML document
fn extract_title(document: &scraper::Html) -> Option<String> {
    use scraper::Selector;

    // Try <title> tag first
    if let Ok(selector) = Selector::parse("title") {
        if let Some(element) = document.select(&selector).next() {
            let title: String = element.text().collect();
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // Try Open Graph title
    if let Ok(selector) = Selector::parse("meta[property='og:title']") {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }

    // Try h1 as fallback
    if let Ok(selector) = Selector::parse("h1") {
        if let Some(element) = document.select(&selector).next() {
            let title: String = element.text().collect();
            let trimmed = title.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Extract meta content by property names
fn extract_meta_content(document: &scraper::Html, properties: &[&str]) -> Option<String> {
    use scraper::Selector;

    for property in properties {
        // Try meta[property]
        let property_selector = format!("meta[property='{}']", property);
        if let Ok(selector) = Selector::parse(&property_selector) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }; // Semicolon to drop temporary

        // Try meta[name]
        let name_selector = format!("meta[name='{}']", property);
        if let Ok(selector) = Selector::parse(&name_selector) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(content) = element.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }; // Semicolon to drop temporary
    }

    None
}

/// Extract article content using common article selectors
fn extract_article_content(document: &scraper::Html) -> String {
    use scraper::Selector;

    // Try common article content selectors
    let selectors = [
        "article",
        "main",
        "[role='main']",
        ".article-content",
        ".post-content",
        ".entry-content",
        "#content",
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text: String = element.text().collect();
                let trimmed = text.trim();
                if trimmed.len() > 200 {
                    return trimmed.to_string();
                }
            }
        }
    }

    // Fallback to body
    extract_full_content(document)
}

/// Extract full page content
fn extract_full_content(document: &scraper::Html) -> String {
    use scraper::Selector;

    if let Ok(selector) = Selector::parse("body") {
        if let Some(body) = document.select(&selector).next() {
            let text: String = body.text().collect();
            return text.trim().to_string();
        }
    }

    String::new()
}

/// Extract content using custom CSS selectors
fn extract_custom_content(document: &scraper::Html, selectors: &[String]) -> String {
    use scraper::Selector;

    let mut content = Vec::new();

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text: String = element.text().collect();
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    content.push(trimmed.to_string());
                }
            }
        }
    }

    content.join("\n\n")
}

/// Enhanced extraction function that combines scraper with our custom extractors
fn perform_enhanced_extraction(
    html: &str,
    url: &str,
    mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    // First, get the base extraction from scraper
    let mut content = perform_extraction_with_scraper(html, url, mode)?;

    // Then enhance with comprehensive extractors from extraction module
    content.links = extraction::extract_links(html, url);
    content.media = extraction::extract_media(html, url);
    content.language = extraction::detect_language(html);
    content.categories = extraction::extract_categories(html);

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
