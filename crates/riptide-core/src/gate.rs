use serde::{Deserialize, Serialize};

/// Features extracted from HTML content used for deciding the optimal extraction strategy.
///
/// This struct represents various characteristics of a web page that help determine
/// whether to use fast extraction, probing, or headless browser rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateFeatures {
    /// Total size of the HTML document in bytes
    pub html_bytes: usize,

    /// Number of visible text characters (excluding markup)
    pub visible_text_chars: usize,

    /// Number of paragraph (`<p>`) elements in the document
    pub p_count: u32,

    /// Number of semantic article elements (`<article>`, `<main>`, etc.)
    pub article_count: u32,

    /// Number of heading elements (`<h1>`, `<h2>`)
    pub h1h2_count: u32,

    /// Total size of JavaScript content in bytes
    pub script_bytes: usize,

    /// Whether the page has Open Graph metadata
    pub has_og: bool,

    /// Whether the page has JSON-LD structured data for articles
    pub has_jsonld_article: bool,

    /// Bit flags indicating SPA (Single Page Application) markers:
    /// - NEXT_DATA script tags
    /// - React hydration markers
    /// - Large root div containers
    /// - Huge JavaScript bundles
    pub spa_markers: u8,

    /// Domain-specific prior score (0.0 to 1.0) based on historical performance
    pub domain_prior: f32,
}

/// Decision made by the gate about which extraction strategy to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Use fast extraction (direct HTML parsing)
    Raw,

    /// Try fast extraction first, then fallback to headless if needed
    ProbesFirst,

    /// Use headless browser rendering immediately
    Headless,
}

/// Calculates a quality score for the HTML content based on various features.
///
/// The score represents how likely it is that fast extraction will work well.
/// Higher scores (closer to 1.0) indicate content that can be extracted quickly,
/// while lower scores indicate complex or dynamic content that may need headless rendering.
///
/// # Arguments
///
/// * `features` - The extracted features from the HTML content
///
/// # Returns
///
/// A score between 0.0 and 1.0, where:
/// - 0.8-1.0: High-quality static content, use fast extraction
/// - 0.3-0.8: Mixed content, try fast first with fallback
/// - 0.0-0.3: Complex/dynamic content, use headless rendering
///
/// # Scoring Algorithm
///
/// The algorithm considers:
/// - **Text ratio**: Higher ratio of visible text to HTML size (+)
/// - **Content structure**: More paragraphs and semantic elements (+)
/// - **Metadata presence**: Open Graph and JSON-LD structured data (+)
/// - **Script density**: High JavaScript content ratio (-)
/// - **SPA markers**: Single-page application indicators (-)
/// - **Domain prior**: Historical performance for this domain (±)
pub fn score(features: &GateFeatures) -> f32 {
    // Calculate ratios to avoid division by zero
    let text_ratio = if features.html_bytes == 0 {
        0.0
    } else {
        features.visible_text_chars as f32 / features.html_bytes as f32
    };

    let script_density = if features.html_bytes == 0 {
        0.0
    } else {
        features.script_bytes as f32 / features.html_bytes as f32
    };

    let mut score = 0.0;

    // Positive indicators (increase score)

    // High text-to-HTML ratio suggests content-rich pages
    score += (text_ratio * 1.2).clamp(0.0, 0.6);

    // More paragraphs indicate structured content (logarithmic scaling)
    score += ((features.p_count as f32 + 1.0).ln() * 0.06).clamp(0.0, 0.3);

    // Semantic article elements suggest well-structured content
    if features.article_count > 0 {
        score += 0.15;
    }

    // Open Graph metadata indicates content-focused pages
    if features.has_og {
        score += 0.08;
    }

    // JSON-LD article data suggests structured content
    if features.has_jsonld_article {
        score += 0.12;
    }

    // Negative indicators (decrease score)

    // High script density suggests dynamic/interactive content
    score -= (script_density * 0.8).clamp(0.0, 0.4);

    // Multiple SPA markers strongly indicate client-side rendering
    if features.spa_markers >= 2 {
        score -= 0.25;
    }

    // Apply domain prior (historical performance adjustment)
    let domain_adjustment = (features.domain_prior - 0.5) * 0.1;

    // Ensure score stays within valid range
    (score + domain_adjustment).clamp(0.0, 1.0)
}

/// Determines if a URL or content type should skip headless rendering for PDF content.
///
/// PDFs should be processed directly with the PDF pipeline rather than through
/// headless browser rendering for better performance and accuracy.
///
/// # Arguments
///
/// * `url` - The URL being processed
/// * `content_type` - Optional HTTP Content-Type header value
///
/// # Returns
///
/// `true` if headless rendering should be used, `false` if PDF processing should be used instead
///
/// # Examples
///
/// ```rust
/// use riptide_core::gate::should_use_headless;
///
/// // PDF URLs should skip headless
/// assert!(!should_use_headless("https://example.com/document.pdf", None));
/// assert!(!should_use_headless("https://example.com/file", Some("application/pdf")));
///
/// // Regular web content should use headless if needed
/// assert!(should_use_headless("https://example.com/article", Some("text/html")));
/// ```
pub fn should_use_headless(url: &str, content_type: Option<&str>) -> bool {
    // Skip headless for PDFs - use direct PDF processing instead
    let url_lower = url.to_lowercase();
    if url_lower.ends_with(".pdf") {
        return false;
    }

    if let Some(ct) = content_type {
        if ct.contains("application/pdf") {
            return false;
        }
    }

    // For all other content, allow headless rendering
    true
}

/// Decides which extraction strategy to use based on content features and thresholds.
///
/// This function implements a three-tier decision system:
/// 1. **Fast extraction** for high-quality static content
/// 2. **Probing strategy** for mixed content (try fast, fallback to headless)
/// 3. **Headless rendering** for complex or dynamic content
///
/// # Arguments
///
/// * `features` - The extracted features from the HTML content
/// * `hi` - High threshold for fast extraction (typically 0.7)
/// * `lo` - Low threshold for headless rendering (typically 0.3)
///
/// # Returns
///
/// A `Decision` indicating the optimal extraction strategy.
///
/// # Decision Logic
///
/// - If score >= `hi`: Use fast extraction (`Decision::Raw`)
/// - If score <= `lo` OR heavy SPA markers: Use headless rendering (`Decision::Headless`)
/// - Otherwise: Try fast first with headless fallback (`Decision::ProbesFirst`)
///
/// # Examples
///
/// ```rust
/// use riptide_core::gate::{GateFeatures, decide, Decision};
///
/// // High-quality article
/// let article_features = GateFeatures {
///     html_bytes: 10000,
///     visible_text_chars: 5000,
///     p_count: 15,
///     article_count: 1,
///     has_og: true,
///     script_bytes: 500,
///     spa_markers: 0,
///     // ... other fields
/// };
/// assert_eq!(decide(&article_features, 0.7, 0.3), Decision::Raw);
///
/// // SPA application
/// let spa_features = GateFeatures {
///     html_bytes: 8000,
///     visible_text_chars: 200,
///     script_bytes: 6000,
///     spa_markers: 3,
///     // ... other fields
/// };
/// assert_eq!(decide(&spa_features, 0.7, 0.3), Decision::Headless);
/// ```
pub fn decide(features: &GateFeatures, hi: f32, lo: f32) -> Decision {
    let content_score = score(features);

    if content_score >= hi {
        // High-quality content: use fast extraction
        Decision::Raw
    } else if content_score <= lo || features.spa_markers >= 3 {
        // Low-quality or heavily SPA content: use headless rendering
        Decision::Headless
    } else {
        // Mixed content: try fast first, fallback to headless if needed
        Decision::ProbesFirst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_simple_article() {
        let features = GateFeatures {
            html_bytes: 10000,
            visible_text_chars: 5000,
            p_count: 10,
            article_count: 1,
            h1h2_count: 3,
            script_bytes: 500,
            has_og: true,
            has_jsonld_article: true,
            spa_markers: 0,
            domain_prior: 0.7,
        };
        let s = score(&features);
        assert!(s > 0.5);
    }

    #[test]
    fn test_decide_spa() {
        let features = GateFeatures {
            html_bytes: 10000,
            visible_text_chars: 500,
            p_count: 2,
            article_count: 0,
            h1h2_count: 1,
            script_bytes: 8000,
            has_og: false,
            has_jsonld_article: false,
            spa_markers: 3,
            domain_prior: 0.5,
        };
        assert_eq!(decide(&features, 0.7, 0.3), Decision::Headless);
    }

    #[test]
    fn test_should_use_headless_pdf_urls() {
        // PDF file extensions should skip headless
        assert!(!should_use_headless(
            "https://example.com/document.pdf",
            None
        ));
        assert!(!should_use_headless("https://example.com/file.PDF", None));
        assert!(!should_use_headless("/local/path/document.pdf", None));

        // PDF content type should skip headless
        assert!(!should_use_headless(
            "https://example.com/document",
            Some("application/pdf")
        ));
        assert!(!should_use_headless(
            "https://example.com/file",
            Some("application/pdf; charset=utf-8")
        ));

        // Regular web content should allow headless
        assert!(should_use_headless(
            "https://example.com/article",
            Some("text/html")
        ));
        assert!(should_use_headless("https://example.com/page", None));
        assert!(should_use_headless(
            "https://example.com/app.js",
            Some("application/javascript")
        ));

        // Edge cases
        assert!(should_use_headless("https://example.com/pdf-viewer", None));
        assert!(should_use_headless(
            "https://example.com/file.pdf.html",
            Some("text/html")
        ));
    }
}
