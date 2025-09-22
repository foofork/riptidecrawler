// Enhanced guest-side implementation for WASM Component Model
// This file shows how to implement the enhanced WIT interface

use trek_rs::{Article, Extractor as TrekExtractor, ExtractorConfig};
use serde_json;
use std::collections::HashMap;

// Generate bindings from the enhanced WIT file
wit_bindgen::generate!({
    world: "extractor",
    path: "wit",
    with: {
        "riptide:extractor/extractor": generate,
    },
});

// Export the component implementation
export!(Component);

/// Component state for tracking metrics and caches
struct Component {
    extraction_count: u64,
    memory_usage: u64,
    start_time: std::time::Instant,
    extractor_cache: HashMap<String, TrekExtractor>,
}

impl Component {
    fn new() -> Self {
        Self {
            extraction_count: 0,
            memory_usage: 0,
            start_time: std::time::Instant::now(),
            extractor_cache: HashMap::new(),
        }
    }

    fn get_or_create_extractor(&mut self, mode: &ExtractionMode) -> TrekExtractor {
        let config = match mode {
            ExtractionMode::Article => ExtractorConfig::new()
                .with_readability(true)
                .with_metadata(true)
                .with_links(true),
            ExtractionMode::Full => ExtractorConfig::new()
                .with_full_content(true)
                .with_navigation(true)
                .with_sidebars(true),
            ExtractionMode::Metadata => ExtractorConfig::new()
                .with_metadata_only(true)
                .with_structured_data(true),
            ExtractionMode::Custom(selectors) => ExtractorConfig::new()
                .with_custom_selectors(selectors.clone()),
        };

        TrekExtractor::with_config(config)
    }
}

// Global component instance (Component Model handles this safely)
static mut COMPONENT: Option<Component> = None;

fn get_component() -> &'static mut Component {
    unsafe {
        if COMPONENT.is_none() {
            COMPONENT = Some(Component::new());
        }
        COMPONENT.as_mut().unwrap()
    }
}

impl Guest for Component {
    fn extract(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<ExtractedContent, ExtractionError> {
        let component = get_component();
        let start_time = std::time::Instant::now();

        // Input validation
        if html.is_empty() {
            return Err(ExtractionError::InvalidHtml(
                "HTML content cannot be empty".to_string(),
            ));
        }

        if url.is_empty() {
            return Err(ExtractionError::InvalidHtml(
                "URL cannot be empty".to_string(),
            ));
        }

        // Validate URL format
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ExtractionError::InvalidHtml(
                "URL must start with http:// or https://".to_string(),
            ));
        }

        // Get appropriate extractor for the mode
        let mut extractor = component.get_or_create_extractor(&mode);

        // Perform extraction using trek-rs
        let article = match extractor
            .url(&url)
            .html(&html)
            .extract()
        {
            Ok(article) => article,
            Err(trek_error) => {
                return Err(ExtractionError::ExtractorError(
                    format!("Trek extraction failed: {}", trek_error),
                ));
            }
        };

        // Convert trek-rs result to Component Model format
        let content = convert_article_to_content(article, &url, &mode);

        // Update metrics
        component.extraction_count += 1;
        let processing_time = start_time.elapsed();

        Ok(content)
    }

    fn extract_with_stats(
        html: String,
        url: String,
        mode: ExtractionMode,
    ) -> Result<(ExtractedContent, ExtractionStats), ExtractionError> {
        let start_time = std::time::Instant::now();
        let start_memory = get_memory_usage();

        // Perform extraction
        let content = Self::extract(html.clone(), url, mode)?;

        // Calculate statistics
        let processing_time = start_time.elapsed();
        let memory_used = get_memory_usage().saturating_sub(start_memory);

        let stats = ExtractionStats {
            processing_time_ms: processing_time.as_millis() as u64,
            memory_used,
            nodes_processed: estimate_node_count(&html),
            links_found: content.links.len() as u32,
            images_found: content.media.len() as u32,
        };

        Ok((content, stats))
    }

    fn validate_html(html: String) -> Result<bool, ExtractionError> {
        if html.is_empty() {
            return Ok(false);
        }

        // Basic HTML validation
        let has_html_tag = html.contains("<html") || html.contains("<!DOCTYPE");
        let has_content = html.len() > 100; // Minimum reasonable content length
        let balanced_tags = count_opening_tags(&html) <= count_closing_tags(&html) + 10; // Allow some tolerance

        Ok(has_html_tag && has_content && balanced_tags)
    }

    fn health_check() -> HealthStatus {
        let component = get_component();

        HealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            trek_version: trek_rs::version().to_string(),
            capabilities: vec![
                "article".to_string(),
                "full".to_string(),
                "metadata".to_string(),
                "custom".to_string(),
            ],
            memory_usage: Some(get_memory_usage()),
            extraction_count: Some(component.extraction_count),
        }
    }

    fn get_info() -> ComponentInfo {
        ComponentInfo {
            name: "riptide-extractor-wasm".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            component_model_version: "0.2.0".to_string(),
            features: vec![
                "trek-rs".to_string(),
                "simd".to_string(),
                "advanced-parsing".to_string(),
                "error-handling".to_string(),
                "metrics".to_string(),
            ],
            supported_modes: vec![
                "article".to_string(),
                "full".to_string(),
                "metadata".to_string(),
                "custom".to_string(),
            ],
            build_timestamp: Some(env!("BUILD_TIMESTAMP").to_string()),
            git_commit: option_env!("GIT_COMMIT").map(|s| s.to_string()),
        }
    }

    fn reset_state() -> Result<String, ExtractionError> {
        let component = get_component();

        // Clear caches
        component.extractor_cache.clear();

        // Reset counters (keep some for debugging)
        let old_count = component.extraction_count;
        component.extraction_count = 0;
        component.memory_usage = 0;

        Ok(format!(
            "State reset successfully. Previous extraction count: {}",
            old_count
        ))
    }

    fn get_modes() -> Vec<String> {
        vec![
            "article: Extract article content using readability algorithms".to_string(),
            "full: Extract complete page content including sidebars".to_string(),
            "metadata: Extract only page metadata and structured data".to_string(),
            "custom: Extract content using provided CSS selectors".to_string(),
        ]
    }
}

/// Convert trek-rs Article to Component Model ExtractedContent
fn convert_article_to_content(
    article: Article,
    url: &str,
    mode: &ExtractionMode,
) -> ExtractedContent {
    // Calculate quality score based on content richness
    let quality_score = calculate_quality_score(&article);

    ExtractedContent {
        url: url.to_string(),
        title: article.title,
        byline: article.byline,
        published_iso: article.published_time.map(|t| t.to_rfc3339()),
        markdown: article.content_markdown.unwrap_or_default(),
        text: article.content_text.clone(),
        links: article.links.unwrap_or_default(),
        media: article.images.unwrap_or_default(),
        language: article.language,
        reading_time: estimate_reading_time(&article.content_text),
        quality_score: Some(quality_score),
        word_count: Some(count_words(&article.content_text)),
        categories: article.tags.unwrap_or_default(),
        site_name: article.site_name,
        description: article.description,
    }
}

/// Calculate content quality score (0-100)
fn calculate_quality_score(article: &Article) -> u8 {
    let mut score = 30u8; // Base score

    // Title quality (0-15 points)
    if let Some(title) = &article.title {
        if title.len() > 10 && title.len() < 100 {
            score += 15;
        } else if title.len() > 5 {
            score += 8;
        }
    }

    // Content length (0-20 points)
    let content_len = article.content_text.len();
    if content_len > 2000 {
        score += 20;
    } else if content_len > 1000 {
        score += 15;
    } else if content_len > 500 {
        score += 10;
    } else if content_len > 200 {
        score += 5;
    }

    // Author/byline (0-10 points)
    if article.byline.is_some() {
        score += 10;
    }

    // Publication date (0-10 points)
    if article.published_time.is_some() {
        score += 10;
    }

    // Media content (0-10 points)
    if let Some(images) = &article.images {
        if images.len() > 3 {
            score += 10;
        } else if images.len() > 0 {
            score += 5;
        }
    }

    // Links (0-5 points)
    if let Some(links) = &article.links {
        if links.len() > 5 {
            score += 5;
        } else if links.len() > 0 {
            score += 2;
        }
    }

    score.min(100)
}

/// Estimate reading time in minutes based on word count
fn estimate_reading_time(text: &str) -> Option<u32> {
    let word_count = count_words(text);
    if word_count == 0 {
        return None;
    }

    // Average reading speed: 200-250 words per minute
    let reading_time = (word_count as f32 / 225.0).ceil() as u32;
    Some(reading_time.max(1))
}

/// Count words in text
fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// Estimate DOM node count from HTML
fn estimate_node_count(html: &str) -> Option<u32> {
    let opening_tags = count_opening_tags(html);
    Some(opening_tags)
}

/// Count opening HTML tags
fn count_opening_tags(html: &str) -> u32 {
    html.matches('<').count() as u32
}

/// Count closing HTML tags
fn count_closing_tags(html: &str) -> u32 {
    html.matches("</").count() as u32
}

/// Get current memory usage (placeholder implementation)
fn get_memory_usage() -> u64 {
    // In a real implementation, this would query WASM memory usage
    // For now, return a reasonable estimate
    1024 * 1024 // 1MB placeholder
}

/// Build-time environment variable injection
/// Add this to build.rs:
/// ```rust
/// use std::process::Command;
///
/// fn main() {
///     // Set build timestamp
///     println!("cargo:rustc-env=BUILD_TIMESTAMP={}",
///         chrono::Utc::now().to_rfc3339());
///
///     // Set git commit if available
///     if let Ok(output) = Command::new("git")
///         .args(&["rev-parse", "--short", "HEAD"])
///         .output()
///     {
///         let git_hash = String::from_utf8(output.stdout).unwrap_or_default();
///         println!("cargo:rustc-env=GIT_COMMIT={}", git_hash.trim());
///     }
/// }
/// ```