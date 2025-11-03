//! Integration of confidence scoring with HTML extraction strategies

use crate::confidence::{ConfidenceScore, ConfidenceScorer};
use riptide_types::ExtractedContent;

/// WASM extractor confidence scorer
pub struct WasmConfidenceScorer {
    base_confidence: f64,
}

impl WasmConfidenceScorer {
    pub fn new() -> Self {
        Self {
            base_confidence: 0.8, // WASM is our primary, high-quality extractor
        }
    }

    /// Compute confidence from HTML content analysis
    pub fn analyze_html(&self, html: &str) -> ConfidenceScore {
        let mut score = ConfidenceScore::new(self.base_confidence, "wasm");

        // Component: Structured content detection
        let structure_score = self.detect_structure(html);
        score.add_component("structure", structure_score);

        // Component: Content indicators
        let content_score = self.detect_content_quality(html);
        score.add_component("content_quality", content_score);

        // Component: Semantic tags
        let semantic_score = self.detect_semantic_tags(html);
        score.add_component("semantic_tags", semantic_score);

        // Component: Metadata presence
        let metadata_score = self.detect_metadata(html);
        score.add_component("metadata", metadata_score);

        score
    }

    fn detect_structure(&self, html: &str) -> f64 {
        let mut score: f64 = 0.0;

        if html.contains("<article") {
            score += 0.4;
        }
        if html.contains("<main") {
            score += 0.3;
        }
        if html.contains("<header") && html.contains("<footer") {
            score += 0.2;
        }
        if html.contains("<section") {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn detect_content_quality(&self, html: &str) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Length indicators
        if html.len() > 10000 {
            score += 0.2;
        } else if html.len() > 5000 {
            score += 0.1;
        }

        // Content class indicators
        if html.contains("class=\"content\"") || html.contains("class=\"post\"") {
            score += 0.2;
        }

        // Paragraph density
        let p_count = html.matches("<p").count();
        if p_count > 5 {
            score += 0.1;
        }

        score.min(1.0)
    }

    fn detect_semantic_tags(&self, html: &str) -> f64 {
        let mut score: f64 = 0.0;

        let semantic_tags = [
            "<article", "<section", "<nav", "<aside", "<header", "<footer", "<main", "<time",
        ];

        for tag in &semantic_tags {
            if html.contains(tag) {
                score += 0.125; // Each tag adds value
            }
        }

        score.min(1.0)
    }

    fn detect_metadata(&self, html: &str) -> f64 {
        let mut score: f64 = 0.0;

        let metadata_indicators = [
            "property=\"og:title\"",
            "property=\"og:description\"",
            "name=\"description\"",
            "name=\"author\"",
            "<meta name=",
        ];

        for indicator in &metadata_indicators {
            if html.contains(indicator) {
                score += 0.2;
            }
        }

        score.min(1.0)
    }
}

impl Default for WasmConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer for WasmConfidenceScorer {
    fn compute_confidence<T>(&self, _doc: &T) -> ConfidenceScore {
        // Generic implementation - would need HTML access for real analysis
        ConfidenceScore::new(self.base_confidence, "wasm")
    }
}

/// CSS selector-based confidence scorer
pub struct CssConfidenceScorer {
    base_confidence: f64,
}

impl CssConfidenceScorer {
    pub fn new() -> Self {
        Self {
            base_confidence: 0.7, // CSS extraction is reliable but may miss context
        }
    }

    pub fn analyze_html(&self, html: &str) -> ConfidenceScore {
        let mut score = ConfidenceScore::new(self.base_confidence, "css");

        // Check for common CSS patterns
        let selector_score = self.detect_selectable_content(html);
        score.add_component("selector_quality", selector_score);

        // Check for structured content
        let structure_score = self.detect_css_structure(html);
        score.add_component("css_structure", structure_score);

        score
    }

    fn detect_selectable_content(&self, html: &str) -> f64 {
        let mut score: f64 = 0.5;

        // Check for common content classes
        let content_classes = [
            "class=\"content\"",
            "class=\"main-content\"",
            "class=\"post-content\"",
            "class=\"article-body\"",
            "id=\"content\"",
        ];

        for class in &content_classes {
            if html.contains(class) {
                score += 0.1;
            }
        }

        score.min(1.0)
    }

    fn detect_css_structure(&self, html: &str) -> f64 {
        let mut score: f64 = 0.0;

        if html.contains("class=") {
            score += 0.3;
        }
        if html.contains("id=") {
            score += 0.3;
        }
        if html.contains("<div") {
            score += 0.2;
        }
        if html.contains("<span") {
            score += 0.2;
        }

        score.min(1.0)
    }
}

impl Default for CssConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer for CssConfidenceScorer {
    fn compute_confidence<T>(&self, _doc: &T) -> ConfidenceScore {
        ConfidenceScore::new(self.base_confidence, "css")
    }
}

/// Regex-based confidence scorer
pub struct RegexConfidenceScorer;

impl RegexConfidenceScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score_from_match(matched: bool) -> ConfidenceScore {
        // Regex provides binary confidence
        ConfidenceScore::from_boolean(matched, "regex")
    }

    pub fn score_from_match_quality(
        pattern_matches: usize,
        total_patterns: usize,
    ) -> ConfidenceScore {
        // Compute ratio of matched patterns
        let ratio = if total_patterns > 0 {
            // Safe conversion: practical pattern counts will fit in f64
            #[allow(clippy::cast_precision_loss)]
            let matches = pattern_matches as f64;
            #[allow(clippy::cast_precision_loss)]
            let total = total_patterns as f64;
            matches / total
        } else {
            0.0
        };

        ConfidenceScore::new(ratio, "regex")
    }
}

impl Default for RegexConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfidenceScorer for RegexConfidenceScorer {
    fn compute_confidence<T>(&self, _doc: &T) -> ConfidenceScore {
        ConfidenceScore::from_boolean(true, "regex")
    }
}

/// Helper function to map old quality_score (u8) to new confidence score
pub fn quality_score_to_confidence(quality: Option<u8>) -> ConfidenceScore {
    match quality {
        Some(q) => {
            // Old quality_score was 0-10, normalize to 0.0-1.0
            // Safe conversion: u8 always fits in f64
            let normalized = f64::from(q) / 10.0;
            ConfidenceScore::new(normalized, "legacy_quality")
        }
        None => ConfidenceScore::new(0.5, "unknown"), // Default medium confidence
    }
}

/// Update ExtractedContent with confidence score
pub fn add_confidence_to_extraction(
    mut content: ExtractedContent,
    confidence: ConfidenceScore,
) -> ExtractedContent {
    // Store confidence value in extraction_confidence field
    content.extraction_confidence = confidence.value();
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_confidence_scoring() {
        let html = r#"
            <html>
                <head>
                    <meta property="og:title" content="Test">
                    <meta name="description" content="Test description">
                </head>
                <body>
                    <main>
                        <article>
                            <header><h1>Title</h1></header>
                            <p>Content paragraph 1</p>
                            <p>Content paragraph 2</p>
                            <p>Content paragraph 3</p>
                        </article>
                    </main>
                </body>
            </html>
        "#;

        let scorer = WasmConfidenceScorer::new();
        let score = scorer.analyze_html(html);

        // With good structure, metadata, and semantic tags, score should be decent
        // Component average: structure (0.7) + content (0.5) + semantic (0.5) + metadata (0.4) = 0.525 avg
        assert!(
            score.value() >= 0.5,
            "Expected score >= 0.5, got {}",
            score.value()
        );
        assert!(
            score.value() <= 0.6,
            "Expected score <= 0.6, got {}",
            score.value()
        );
        assert_eq!(score.method(), "wasm");
        assert_eq!(score.components().len(), 4);
    }

    #[test]
    fn test_css_confidence_scoring() {
        let html = r#"
            <div class="content">
                <div class="post-content">
                    <p>Some content</p>
                </div>
            </div>
        "#;

        let scorer = CssConfidenceScorer::new();
        let score = scorer.analyze_html(html);

        // CSS extraction has reasonable confidence with content classes
        assert!(
            score.value() >= 0.6,
            "Expected score >= 0.6, got {}",
            score.value()
        );
        assert_eq!(score.method(), "css");
    }

    #[test]
    fn test_regex_confidence_binary() {
        let matched = RegexConfidenceScorer::score_from_match(true);
        assert_eq!(matched.value(), 1.0);

        let not_matched = RegexConfidenceScorer::score_from_match(false);
        assert_eq!(not_matched.value(), 0.0);
    }

    #[test]
    fn test_regex_confidence_quality() {
        let score = RegexConfidenceScorer::score_from_match_quality(7, 10);
        assert!((score.value() - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_quality_score_migration() {
        // Test old quality_score conversion
        let high_quality = quality_score_to_confidence(Some(9));
        assert!((high_quality.value() - 0.9).abs() < 0.01);

        let low_quality = quality_score_to_confidence(Some(3));
        assert!((low_quality.value() - 0.3).abs() < 0.01);

        let unknown = quality_score_to_confidence(None);
        assert_eq!(unknown.value(), 0.5);
    }
}
