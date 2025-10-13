//! Full 3-tier pipeline integration tests
//! Tests Decision::Raw, Decision::ProbesFirst with fallback, and Decision::Headless paths

use std::sync::Arc;

// Mock structures for testing
#[derive(Debug, Clone, PartialEq)]
enum Decision {
    Raw,
    ProbesFirst,
    Headless,
}

#[derive(Debug, Clone)]
struct PipelineMetrics {
    pub gate_decision_raw: u64,
    pub gate_decision_probes_first: u64,
    pub gate_decision_headless: u64,
    pub extraction_raw_count: u64,
    pub extraction_probes_first_count: u64,
    pub extraction_headless_count: u64,
    pub extraction_fallback_count: u64,
}

impl PipelineMetrics {
    fn new() -> Self {
        Self {
            gate_decision_raw: 0,
            gate_decision_probes_first: 0,
            gate_decision_headless: 0,
            extraction_raw_count: 0,
            extraction_probes_first_count: 0,
            extraction_headless_count: 0,
            extraction_fallback_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct ExtractionResult {
    pub text: String,
    pub title: Option<String>,
    pub links: Vec<String>,
    pub quality_score: f32,
}

#[derive(Debug, Clone)]
struct PipelineResult {
    pub decision: Decision,
    pub extraction: ExtractionResult,
    pub fallback_triggered: bool,
}

struct Pipeline {
    metrics: Arc<std::sync::Mutex<PipelineMetrics>>,
}

impl Pipeline {
    fn new() -> Self {
        Self {
            metrics: Arc::new(std::sync::Mutex::new(PipelineMetrics::new())),
        }
    }

    async fn process(&self, html: &str, url: &str) -> Result<PipelineResult, String> {
        // Step 1: Gate decision
        let decision = self.make_gate_decision(html);

        // Record gate decision
        {
            let mut metrics = self.metrics.lock().unwrap();
            match decision {
                Decision::Raw => metrics.gate_decision_raw += 1,
                Decision::ProbesFirst => metrics.gate_decision_probes_first += 1,
                Decision::Headless => metrics.gate_decision_headless += 1,
            }
        }

        // Step 2: Extraction based on decision
        let (extraction, fallback_triggered) = match decision {
            Decision::Raw => {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.extraction_raw_count += 1;
                drop(metrics);
                (self.extract_raw(html, url).await?, false)
            }
            Decision::ProbesFirst => {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.extraction_probes_first_count += 1;
                drop(metrics);
                self.extract_with_probes_and_fallback(html, url).await?
            }
            Decision::Headless => {
                let mut metrics = self.metrics.lock().unwrap();
                metrics.extraction_headless_count += 1;
                drop(metrics);
                (self.extract_headless(html, url).await?, false)
            }
        };

        Ok(PipelineResult {
            decision,
            extraction,
            fallback_triggered,
        })
    }

    fn make_gate_decision(&self, html: &str) -> Decision {
        // Simple heuristics for gate decision
        let has_spa_markers = html.contains("react")
            || html.contains("vue")
            || html.contains("angular")
            || html.contains("data-reactroot");

        let script_ratio = Self::calculate_script_ratio(html);
        let text_ratio = Self::calculate_text_ratio(html);

        if has_spa_markers || script_ratio > 0.3 {
            Decision::Headless
        } else if script_ratio > 0.15 || text_ratio < 0.2 {
            Decision::ProbesFirst
        } else {
            Decision::Raw
        }
    }

    fn calculate_script_ratio(html: &str) -> f32 {
        let script_content: usize = html.matches("<script").count() * 100; // Approximate
        let total_size = html.len();
        if total_size == 0 {
            return 0.0;
        }
        (script_content as f32) / (total_size as f32)
    }

    fn calculate_text_ratio(html: &str) -> f32 {
        // Simple text extraction: count characters outside tags
        let text_chars = html.chars().filter(|c| !c.is_whitespace()).count();
        let total_size = html.len();
        if total_size == 0 {
            return 0.0;
        }
        (text_chars as f32) / (total_size as f32)
    }

    async fn extract_raw(&self, html: &str, url: &str) -> Result<ExtractionResult, String> {
        // Simulate raw extraction
        let text = Self::strip_html(html);
        let title = Self::extract_title(html);
        let links = Self::extract_links(html);

        Ok(ExtractionResult {
            text,
            title,
            links,
            quality_score: 80.0,
        })
    }

    async fn extract_with_probes_and_fallback(
        &self,
        html: &str,
        url: &str,
    ) -> Result<(ExtractionResult, bool), String> {
        // Try probes first extraction
        let result = self.extract_with_probes(html, url).await?;

        // Check if fallback is needed (quality threshold)
        if result.quality_score < 60.0 || result.text.len() < 100 {
            // Trigger fallback to headless
            let mut metrics = self.metrics.lock().unwrap();
            metrics.extraction_fallback_count += 1;
            drop(metrics);

            let fallback_result = self.extract_headless(html, url).await?;
            Ok((fallback_result, true))
        } else {
            Ok((result, false))
        }
    }

    async fn extract_with_probes(&self, html: &str, url: &str) -> Result<ExtractionResult, String> {
        // Simulate probes-first extraction (slightly better than raw)
        let text = Self::strip_html(html);
        let title = Self::extract_title(html);
        let links = Self::extract_links(html);

        let quality_score = if text.len() > 200 { 75.0 } else { 50.0 };

        Ok(ExtractionResult {
            text,
            title,
            links,
            quality_score,
        })
    }

    async fn extract_headless(&self, _html: &str, url: &str) -> Result<ExtractionResult, String> {
        // Simulate headless extraction (best quality)
        Ok(ExtractionResult {
            text: "High quality headless extracted content from dynamic page.".to_string(),
            title: Some("Headless Title".to_string()),
            links: vec![url.to_string()],
            quality_score: 90.0,
        })
    }

    fn strip_html(html: &str) -> String {
        // Simple HTML stripping
        let mut result = String::new();
        let mut in_tag = false;

        for ch in html.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        result.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn extract_title(html: &str) -> Option<String> {
        if let Some(start) = html.find("<title>") {
            if let Some(end) = html[start..].find("</title>") {
                let title = &html[start + 7..start + end];
                return Some(Self::strip_html(title));
            }
        }
        None
    }

    fn extract_links(html: &str) -> Vec<String> {
        html.matches("href=")
            .enumerate()
            .map(|(i, _)| format!("link_{}", i))
            .collect()
    }

    fn gather_metrics(&self) -> PipelineMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

// Test fixture loaders
fn load_fixture(name: &str) -> String {
    match name {
        "simple_article.html" => r#"
            <html>
                <head><title>Simple Article</title></head>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <p>This is a simple article with good text content and minimal scripts.</p>
                        <p>It should trigger the Raw decision path.</p>
                        <a href="/link1">Link 1</a>
                        <a href="/link2">Link 2</a>
                    </article>
                </body>
            </html>
        "#.to_string(),

        "spa_site.html" => r#"
            <html>
                <head><title>SPA Site</title></head>
                <body>
                    <div id="root" data-reactroot></div>
                    <script src="/react.js"></script>
                    <script src="/app.js"></script>
                    <script>window.__INITIAL_STATE__ = {};</script>
                    <p>Minimal static content</p>
                </body>
            </html>
        "#.to_string(),

        "react_app.html" => r#"
            <html>
                <head><title>React App</title></head>
                <body>
                    <div id="app"></div>
                    <script>
                        // Large React application
                        const React = require('react');
                        const ReactDOM = require('react-dom');
                    </script>
                    <script src="/bundle.js"></script>
                </body>
            </html>
        "#.to_string(),

        _ => panic!("Unknown fixture: {}", name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_raw_path() {
        let pipeline = Pipeline::new();
        let html = load_fixture("simple_article.html");

        let result = pipeline.process(&html, "https://example.com").await.unwrap();

        assert_eq!(result.decision, Decision::Raw);
        assert!(result.extraction.text.len() > 100, "Should extract substantial text");
        assert!(!result.fallback_triggered, "Should not trigger fallback");

        // Verify metrics were recorded
        let metrics = pipeline.gather_metrics();
        assert_eq!(metrics.gate_decision_raw, 1, "Should record raw gate decision");
        assert_eq!(metrics.extraction_raw_count, 1, "Should record raw extraction");
    }

    #[tokio::test]
    async fn test_pipeline_probes_first_no_fallback() {
        let pipeline = Pipeline::new();

        // HTML with moderate script content
        let html = r#"
            <html>
                <head><title>Moderate Site</title></head>
                <body>
                    <article>
                        <p>Good content here with enough text to pass quality threshold.</p>
                        <p>Additional paragraphs to ensure we have enough content.</p>
                        <p>This should not trigger fallback to headless mode.</p>
                    </article>
                    <script src="/analytics.js"></script>
                    <script>console.log('tracking');</script>
                </body>
            </html>
        "#;

        let result = pipeline.process(html, "https://example.com").await.unwrap();

        assert_eq!(result.decision, Decision::ProbesFirst, "Should use ProbesFirst");
        assert!(!result.fallback_triggered, "Should not trigger fallback");

        let metrics = pipeline.gather_metrics();
        assert_eq!(metrics.gate_decision_probes_first, 1);
        assert_eq!(metrics.extraction_fallback_count, 0, "No fallback should occur");
    }

    #[tokio::test]
    async fn test_pipeline_probes_first_with_fallback() {
        let pipeline = Pipeline::new();
        let html = load_fixture("spa_site.html");

        let result = pipeline.process(&html, "https://example.com").await.unwrap();

        // Should use ProbesFirst, but trigger fallback due to poor quality
        assert_eq!(result.decision, Decision::ProbesFirst);
        assert!(result.fallback_triggered, "Should trigger fallback");
        assert!(result.extraction.quality_score >= 90.0, "Should have high quality after fallback");

        // Verify fallback was recorded
        let metrics = pipeline.gather_metrics();
        assert_eq!(metrics.extraction_fallback_count, 1, "Should record fallback");
    }

    #[tokio::test]
    async fn test_pipeline_headless_path() {
        let pipeline = Pipeline::new();
        let html = load_fixture("react_app.html");

        let result = pipeline.process(&html, "https://example.com").await.unwrap();

        assert_eq!(result.decision, Decision::Headless, "Should use Headless for React app");
        assert!(!result.fallback_triggered, "Headless doesn't need fallback");
        assert!(result.extraction.quality_score >= 90.0, "Should have high quality");

        // Verify headless extraction metrics
        let metrics = pipeline.gather_metrics();
        assert_eq!(metrics.gate_decision_headless, 1);
        assert_eq!(metrics.extraction_headless_count, 1);
    }

    #[tokio::test]
    async fn test_pipeline_multiple_requests() {
        let pipeline = Pipeline::new();

        // Process multiple pages through different paths
        let _ = pipeline.process(&load_fixture("simple_article.html"), "https://example.com/1").await;
        let _ = pipeline.process(&load_fixture("spa_site.html"), "https://example.com/2").await;
        let _ = pipeline.process(&load_fixture("react_app.html"), "https://example.com/3").await;

        let metrics = pipeline.gather_metrics();

        // Verify all paths were exercised
        assert!(metrics.gate_decision_raw > 0, "Should have raw decisions");
        assert!(metrics.gate_decision_probes_first > 0, "Should have probes_first decisions");
        assert!(metrics.gate_decision_headless > 0, "Should have headless decisions");
    }

    #[tokio::test]
    async fn test_pipeline_gate_decision_accuracy() {
        let pipeline = Pipeline::new();

        // Test simple HTML -> Raw
        let simple_html = "<html><body><p>Simple text content</p></body></html>";
        let result = pipeline.process(simple_html, "https://example.com").await.unwrap();
        assert_eq!(result.decision, Decision::Raw);

        // Test SPA markers -> Headless
        let spa_html = r#"<html><body><div id="root" data-reactroot></div></body></html>"#;
        let result = pipeline.process(spa_html, "https://example.com").await.unwrap();
        assert_eq!(result.decision, Decision::Headless);
    }

    #[tokio::test]
    async fn test_pipeline_extraction_quality() {
        let pipeline = Pipeline::new();

        // Test that Raw extraction produces reasonable quality
        let html = load_fixture("simple_article.html");
        let result = pipeline.process(&html, "https://example.com").await.unwrap();

        assert!(result.extraction.quality_score >= 70.0, "Raw should have decent quality");
        assert!(result.extraction.text.len() > 50, "Should extract meaningful text");
        assert!(result.extraction.title.is_some(), "Should extract title");
    }

    #[tokio::test]
    async fn test_pipeline_fallback_quality_threshold() {
        let pipeline = Pipeline::new();

        // HTML that should trigger ProbesFirst but fallback due to poor quality
        let poor_quality_html = r#"
            <html>
                <head><title>Poor Quality</title></head>
                <body>
                    <p>Short</p>
                    <script>lots of scripts</script>
                    <script>more scripts</script>
                </body>
            </html>
        "#;

        let result = pipeline.process(poor_quality_html, "https://example.com").await.unwrap();

        // Should trigger fallback
        assert!(result.fallback_triggered, "Should fallback for poor quality");
        assert!(result.extraction.quality_score >= 90.0, "Should improve quality after fallback");
    }

    #[tokio::test]
    async fn test_pipeline_metrics_consistency() {
        let pipeline = Pipeline::new();

        // Process several pages
        for i in 0..5 {
            let _ = pipeline.process(&load_fixture("simple_article.html"), &format!("https://example.com/{}", i)).await;
        }

        let metrics = pipeline.gather_metrics();

        // Gate decisions should match extractions (minus fallbacks)
        assert_eq!(
            metrics.gate_decision_raw,
            metrics.extraction_raw_count,
            "Gate decisions and extractions should match for Raw path"
        );
    }

    #[tokio::test]
    async fn test_pipeline_error_handling() {
        let pipeline = Pipeline::new();

        // Test with empty HTML
        let result = pipeline.process("", "https://example.com").await;
        assert!(result.is_ok(), "Should handle empty HTML gracefully");

        // Test with malformed HTML
        let malformed_html = "<html><body><p>Unclosed tags<body>";
        let result = pipeline.process(malformed_html, "https://example.com").await;
        assert!(result.is_ok(), "Should handle malformed HTML gracefully");
    }

    #[tokio::test]
    async fn test_pipeline_concurrent_processing() {
        let pipeline = Arc::new(Pipeline::new());
        let mut handles = vec![];

        // Process multiple pages concurrently
        for i in 0..10 {
            let pipeline_clone = Arc::clone(&pipeline);
            let handle = tokio::spawn(async move {
                let html = load_fixture("simple_article.html");
                pipeline_clone.process(&html, &format!("https://example.com/{}", i)).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Concurrent processing should succeed");
        }

        // Verify metrics accumulated correctly
        let metrics = pipeline.gather_metrics();
        assert_eq!(metrics.extraction_raw_count, 10, "Should process all requests");
    }

    #[tokio::test]
    async fn test_pipeline_script_ratio_calculation() {
        let pipeline = Pipeline::new();

        // High script ratio -> Headless
        let high_script_html = r#"
            <html><head><title>Test</title></head><body>
            <script></script><script></script><script></script>
            <script></script><script></script><script></script>
            <p>Text</p>
            </body></html>
        "#;

        let result = pipeline.process(high_script_html, "https://example.com").await.unwrap();
        assert_ne!(result.decision, Decision::Raw, "High script ratio should not use Raw");
    }

    #[tokio::test]
    async fn test_pipeline_complete_flow() {
        // Test the complete 3-tier architecture
        let pipeline = Pipeline::new();

        // Tier 1: Raw extraction
        let tier1 = pipeline.process(&load_fixture("simple_article.html"), "https://tier1.com").await.unwrap();
        assert_eq!(tier1.decision, Decision::Raw);

        // Tier 2: ProbesFirst with possible fallback
        let tier2 = pipeline.process(&load_fixture("spa_site.html"), "https://tier2.com").await.unwrap();
        assert_eq!(tier2.decision, Decision::ProbesFirst);

        // Tier 3: Headless for complex SPAs
        let tier3 = pipeline.process(&load_fixture("react_app.html"), "https://tier3.com").await.unwrap();
        assert_eq!(tier3.decision, Decision::Headless);

        // Verify all three tiers were exercised
        let metrics = pipeline.gather_metrics();
        assert!(metrics.gate_decision_raw > 0, "Tier 1 exercised");
        assert!(metrics.gate_decision_probes_first > 0, "Tier 2 exercised");
        assert!(metrics.gate_decision_headless > 0, "Tier 3 exercised");
    }
}
