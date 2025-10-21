use crate::errors::ApiResult;
use riptide_browser::dynamic::DynamicRenderResult;
use riptide_types::{ExtractedDoc as CoreExtractedDoc, OutputFormat};
use tracing::{debug, info, warn};

/// Extract content using ExtractionFacade (replaces direct WASM extractor usage)
///
/// This function has been migrated to use the ExtractionFacade pattern for better
/// consistency across the codebase. It now uses the facade's extract_html method
/// with appropriate options based on the output format.
pub(super) async fn extract_with_extraction_facade(
    facade: &riptide_facade::facades::ExtractionFacade,
    html: &str,
    url: &str,
    output_format: &OutputFormat,
) -> Result<CoreExtractedDoc, Box<dyn std::error::Error + Send + Sync>> {
    // Validate inputs before processing
    if html.trim().is_empty() {
        return Err("Empty HTML content provided".into());
    }

    if url.trim().is_empty() {
        return Err("Empty URL provided".into());
    }

    // Basic URL validation
    if let Err(e) = url::Url::parse(url) {
        return Err(format!("Invalid URL format: {}", e).into());
    }

    // Validate HTML size (prevent excessive memory usage)
    if html.as_bytes().len() > 50 * 1024 * 1024 {
        // 50MB limit
        return Err("HTML content too large (>50MB)".into());
    }

    // Configure extraction options based on output format
    let options = riptide_facade::facades::HtmlExtractionOptions {
        as_markdown: matches!(output_format, OutputFormat::Markdown),
        clean: true,
        include_metadata: true,
        extract_links: true,
        extract_images: true,
        custom_selectors: None,
    };

    // Use facade for extraction with fallback strategy chain for best quality
    let strategies = vec![
        riptide_facade::facades::ExtractionStrategy::Wasm,
        riptide_facade::facades::ExtractionStrategy::HtmlCss,
        riptide_facade::facades::ExtractionStrategy::Fallback,
    ];

    let extracted = facade
        .extract_with_fallback(html, url, &strategies)
        .await
        .map_err(|e| {
            Box::new(std::io::Error::other(format!(
                "Extraction failed for URL '{}': {}",
                url, e
            ))) as Box<dyn std::error::Error + Send + Sync>
        })?;

    // Convert facade result to CoreExtractedDoc
    let core_doc = CoreExtractedDoc {
        url: extracted.url.clone(),
        title: extracted.title.clone(),
        text: extracted.text.clone(),
        quality_score: Some((extracted.confidence * 100.0).min(100.0) as u8),
        links: extracted.links.clone(),
        byline: extracted.metadata.get("author").cloned(),
        published_iso: extracted.metadata.get("published_date").cloned(),
        markdown: extracted.markdown.clone(),
        media: extracted.images.clone(),
        language: extracted.metadata.get("language").cloned(),
        reading_time: None,
        word_count: Some(extracted.text.split_whitespace().count() as u32),
        categories: Vec::new(),
        site_name: extracted.metadata.get("site_name").cloned(),
        description: extracted.metadata.get("description").cloned(),
    };

    Ok(core_doc)
}

/// Extract content from render result using ExtractionFacade
pub(super) async fn extract_content(
    facade: &riptide_facade::facades::ExtractionFacade,
    render_result: &Option<DynamicRenderResult>,
    output_format: &OutputFormat,
    url: &str,
) -> ApiResult<Option<CoreExtractedDoc>> {
    if let Some(result) = render_result {
        if !result.success {
            return Ok(None);
        }

        // Use ExtractionFacade to process the HTML
        match extract_with_extraction_facade(facade, &result.html, url, output_format).await {
            Ok(doc) => {
                // Log extraction completion
                info!(
                    url = %doc.url,
                    title = ?doc.title,
                    word_count = ?doc.word_count,
                    quality_score = ?doc.quality_score,
                    links_count = doc.links.len(),
                    "Extraction completed successfully via ExtractionFacade"
                );

                debug!(
                    url = %doc.url,
                    title = ?doc.title,
                    word_count = ?doc.word_count,
                    quality_score = ?doc.quality_score,
                    "Extracted content details"
                );

                Ok(Some(doc))
            }
            Err(e) => {
                warn!(
                    error = %e,
                    url = %url,
                    "Extraction failed, falling back to empty result"
                );

                // Return None rather than failing the entire request
                // This allows the render response to still be useful
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_with_extraction_facade_validation() {
        // Create a test facade
        let config = riptide_facade::RiptideConfig::default();
        let facade = match riptide_facade::facades::ExtractionFacade::new(config).await {
            Ok(f) => f,
            Err(_) => {
                println!("Skipping test - facade initialization failed");
                return;
            }
        };

        // Test 1: Empty HTML validation
        let result = extract_with_extraction_facade(
            &facade,
            "",
            "https://example.com",
            &OutputFormat::Document,
        )
        .await;
        assert!(result.is_err(), "Should reject empty HTML");

        // Test 2: Empty URL validation
        let result = extract_with_extraction_facade(
            &facade,
            "<html><body>Test</body></html>",
            "",
            &OutputFormat::Document,
        )
        .await;
        assert!(result.is_err(), "Should reject empty URL");

        // Test 3: Invalid URL validation
        let result = extract_with_extraction_facade(
            &facade,
            "<html><body>Test</body></html>",
            "not-a-url",
            &OutputFormat::Document,
        )
        .await;
        assert!(result.is_err(), "Should reject invalid URL");
    }
}
