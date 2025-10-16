# WASM Extractor Feature Implementations - Ready to Copy

**Date:** 2025-10-07
**Purpose:** Copy-paste ready implementations for the 4 content extraction TODOs

---

## Implementation Files Structure

```
wasm/riptide-extractor-wasm/src/
├── lib_clean.rs          # Main component (update convert_response_to_content)
├── wasm_helpers.rs       # Add new helper functions here
├── extraction.rs         # Alternative location for extraction utilities
└── common_validation.rs  # Existing validation helpers
```

---

## Feature 1: Link Extraction

### Add to `src/wasm_helpers.rs`

```rust
use scraper::{Html, Selector};
use url::Url;

/// Extract all links from HTML content
pub fn extract_links(html: &str, base_url: &str) -> Vec<Link> {
    let document = Html::parse_document(html);
    let link_selector = match Selector::parse("a[href]") {
        Ok(sel) => sel,
        Err(_) => return Vec::new(),
    };

    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return Vec::new(),
    };

    document
        .select(&link_selector)
        .filter_map(|element| {
            let href = element.value().attr("href")?;

            // Skip non-http(s) links (javascript:, mailto:, tel:, etc.)
            if href.starts_with("javascript:")
                || href.starts_with("mailto:")
                || href.starts_with("tel:")
                || href.starts_with('#')
            {
                return None;
            }

            // Resolve relative URLs to absolute
            let absolute_url = match base.join(href) {
                Ok(url) => url.to_string(),
                Err(_) => href.to_string(), // Keep as-is if resolution fails
            };

            // Extract link text
            let text: String = element.text().collect::<Vec<_>>().join(" ");
            let text = text.trim().to_string();

            // Extract rel attribute
            let rel = element.value().attr("rel").map(String::from);

            Some(Link {
                url: absolute_url,
                text: if text.is_empty() {
                    None
                } else {
                    Some(text)
                },
                rel,
            })
        })
        .collect()
}

/// Link representation
#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub text: Option<String>,
    pub rel: Option<String>,
}

#[cfg(test)]
mod link_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_links_absolute_urls() {
        let html = r#"
            <html><body>
                <a href="https://example.com/page1">Page 1</a>
                <a href="https://example.com/page2">Page 2</a>
            </body></html>
        "#;

        let links = extract_links(html, "https://base.com");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com/page1");
        assert_eq!(links[0].text, Some("Page 1".to_string()));
    }

    #[test]
    fn test_extract_links_relative_urls() {
        let html = r#"
            <html><body>
                <a href="/relative">Relative Link</a>
                <a href="another.html">Another Page</a>
            </body></html>
        "#;

        let links = extract_links(html, "https://example.com/base/");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com/relative");
        assert_eq!(links[1].url, "https://example.com/base/another.html");
    }

    #[test]
    fn test_extract_links_skip_special() {
        let html = r#"
            <html><body>
                <a href="javascript:void(0)">JS Link</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:1234567890">Phone</a>
                <a href="#anchor">Anchor</a>
                <a href="https://example.com/real">Real Link</a>
            </body></html>
        "#;

        let links = extract_links(html, "https://base.com");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com/real");
    }

    #[test]
    fn test_extract_links_with_rel() {
        let html = r#"
            <html><body>
                <a href="https://example.com" rel="nofollow">External</a>
                <a href="/internal">Internal</a>
            </body></html>
        "#;

        let links = extract_links(html, "https://base.com");
        assert_eq!(links[0].rel, Some("nofollow".to_string()));
        assert_eq!(links[1].rel, None);
    }

    #[test]
    fn test_extract_links_empty_text() {
        let html = r#"<html><body><a href="https://example.com"></a></body></html>"#;

        let links = extract_links(html, "https://base.com");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, None);
    }
}
```

---

## Feature 2: Media Extraction

### Add to `src/wasm_helpers.rs`

```rust
/// Extract all media (images, videos, audio) from HTML content
pub fn extract_media(html: &str, base_url: &str) -> Vec<MediaItem> {
    let document = Html::parse_document(html);
    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return Vec::new(),
    };

    let mut media = Vec::new();

    // Extract images
    if let Ok(img_selector) = Selector::parse("img[src], img[data-src]") {
        for element in document.select(&img_selector) {
            if let Some(src) = element
                .value()
                .attr("src")
                .or_else(|| element.value().attr("data-src"))
            {
                // Skip data URIs and placeholders
                if src.starts_with("data:") || src.contains("placeholder") {
                    continue;
                }

                let absolute_url = base.join(src).ok().map(|u| u.to_string()).unwrap_or_else(|| src.to_string());

                let alt = element.value().attr("alt").map(String::from);
                let width = element.value().attr("width").and_then(|w| w.parse().ok());
                let height = element.value().attr("height").and_then(|h| h.parse().ok());

                media.push(MediaItem {
                    media_type: MediaType::Image,
                    url: absolute_url,
                    alt,
                    width,
                    height,
                    mime_type: None,
                });
            }
        }
    }

    // Extract videos
    if let Ok(video_selector) = Selector::parse("video source[src], video[src]") {
        for element in document.select(&video_selector) {
            if let Some(src) = element.value().attr("src") {
                let absolute_url = base.join(src).ok().map(|u| u.to_string()).unwrap_or_else(|| src.to_string());

                let mime_type = element.value().attr("type").map(String::from);

                media.push(MediaItem {
                    media_type: MediaType::Video,
                    url: absolute_url,
                    alt: None,
                    width: None,
                    height: None,
                    mime_type,
                });
            }
        }
    }

    // Extract audio
    if let Ok(audio_selector) = Selector::parse("audio source[src], audio[src]") {
        for element in document.select(&audio_selector) {
            if let Some(src) = element.value().attr("src") {
                let absolute_url = base.join(src).ok().map(|u| u.to_string()).unwrap_or_else(|| src.to_string());

                let mime_type = element.value().attr("type").map(String::from);

                media.push(MediaItem {
                    media_type: MediaType::Audio,
                    url: absolute_url,
                    alt: None,
                    width: None,
                    height: None,
                    mime_type,
                });
            }
        }
    }

    media
}

/// Media item representation
#[derive(Debug, Clone)]
pub struct MediaItem {
    pub media_type: MediaType,
    pub url: String,
    pub alt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    Image,
    Video,
    Audio,
}

#[cfg(test)]
mod media_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_images() {
        let html = r#"
            <html><body>
                <img src="photo.jpg" alt="Photo">
                <img src="/images/logo.png" alt="Logo" width="100" height="50">
                <img data-src="lazy.jpg" alt="Lazy">
            </body></html>
        "#;

        let media = extract_media(html, "https://example.com");
        let images: Vec<_> = media.iter().filter(|m| m.media_type == MediaType::Image).collect();

        assert_eq!(images.len(), 3);
        assert_eq!(images[0].url, "https://example.com/photo.jpg");
        assert_eq!(images[0].alt, Some("Photo".to_string()));
        assert_eq!(images[1].width, Some(100));
        assert_eq!(images[1].height, Some(50));
    }

    #[test]
    fn test_extract_video() {
        let html = r#"
            <html><body>
                <video>
                    <source src="video.mp4" type="video/mp4">
                    <source src="video.webm" type="video/webm">
                </video>
            </body></html>
        "#;

        let media = extract_media(html, "https://example.com");
        let videos: Vec<_> = media.iter().filter(|m| m.media_type == MediaType::Video).collect();

        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].url, "https://example.com/video.mp4");
        assert_eq!(videos[0].mime_type, Some("video/mp4".to_string()));
    }

    #[test]
    fn test_extract_audio() {
        let html = r#"
            <html><body>
                <audio>
                    <source src="audio.mp3" type="audio/mpeg">
                </audio>
            </body></html>
        "#;

        let media = extract_media(html, "https://example.com");
        let audio: Vec<_> = media.iter().filter(|m| m.media_type == MediaType::Audio).collect();

        assert_eq!(audio.len(), 1);
        assert_eq!(audio[0].url, "https://example.com/audio.mp3");
        assert_eq!(audio[0].mime_type, Some("audio/mpeg".to_string()));
    }

    #[test]
    fn test_skip_data_uris() {
        let html = r#"
            <html><body>
                <img src="data:image/png;base64,iVBOR...">
                <img src="real-image.jpg">
            </body></html>
        "#;

        let media = extract_media(html, "https://example.com");
        assert_eq!(media.len(), 1);
        assert_eq!(media[0].url, "https://example.com/real-image.jpg");
    }

    #[test]
    fn test_relative_media_urls() {
        let html = r#"
            <html><body>
                <img src="/images/photo.jpg">
                <img src="../assets/logo.png">
            </body></html>
        "#;

        let media = extract_media(html, "https://example.com/page/");
        assert_eq!(media.len(), 2);
        assert_eq!(media[0].url, "https://example.com/images/photo.jpg");
        assert_eq!(media[1].url, "https://example.com/assets/logo.png");
    }
}
```

---

## Feature 3: Language Detection

### Option A: Lightweight (HTML attribute only)

```rust
/// Extract language from HTML lang attribute
pub fn extract_language(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Try <html lang="...">
    if let Ok(html_selector) = Selector::parse("html[lang]") {
        if let Some(element) = document.select(&html_selector).next() {
            if let Some(lang) = element.value().attr("lang") {
                let normalized = normalize_language_code(lang);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    // Try <meta http-equiv="content-language">
    if let Ok(meta_selector) = Selector::parse("meta[http-equiv='content-language']") {
        if let Some(element) = document.select(&meta_selector).next() {
            if let Some(lang) = element.value().attr("content") {
                let normalized = normalize_language_code(lang);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    None
}

/// Normalize language code to ISO 639-1 (2-letter code)
fn normalize_language_code(lang: &str) -> String {
    // Extract primary language code (e.g., "en-US" -> "en", "en_GB" -> "en")
    lang.split(&['-', '_'][..])
        .next()
        .unwrap_or(lang)
        .to_lowercase()
        .trim()
        .to_string()
}

#[cfg(test)]
mod language_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_language_from_html_tag() {
        let html = r#"<html lang="en"><body>Content</body></html>"#;
        assert_eq!(extract_language(html), Some("en".to_string()));

        let html = r#"<html lang="es-ES"><body>Contenido</body></html>"#;
        assert_eq!(extract_language(html), Some("es".to_string()));
    }

    #[test]
    fn test_extract_language_from_meta() {
        let html = r#"
            <html>
            <head><meta http-equiv="content-language" content="fr"></head>
            <body>Contenu</body>
            </html>
        "#;
        assert_eq!(extract_language(html), Some("fr".to_string()));
    }

    #[test]
    fn test_language_normalization() {
        assert_eq!(normalize_language_code("en-US"), "en");
        assert_eq!(normalize_language_code("en_GB"), "en");
        assert_eq!(normalize_language_code("zh-Hans-CN"), "zh");
        assert_eq!(normalize_language_code("EN"), "en");
    }

    #[test]
    fn test_no_language_found() {
        let html = r#"<html><body>Content</body></html>"#;
        assert_eq!(extract_language(html), None);
    }
}
```

### Option B: Content-based (requires whatlang dependency)

```toml
# Add to Cargo.toml
[dependencies]
whatlang = "0.16"
```

```rust
use whatlang::{detect, Lang};

/// Detect language from text content using whatlang
pub fn detect_language_from_content(text: &str) -> Option<String> {
    // Need sufficient text for reliable detection
    if text.len() < 50 {
        return None;
    }

    // Take first 1000 chars for detection (faster)
    let sample = if text.len() > 1000 {
        &text[..1000]
    } else {
        text
    };

    detect(sample).map(|info| {
        match info.lang() {
            Lang::Eng => "en",
            Lang::Spa => "es",
            Lang::Fra => "fr",
            Lang::Deu => "de",
            Lang::Rus => "ru",
            Lang::Jpn => "ja",
            Lang::Cmn => "zh",
            Lang::Arb => "ar",
            Lang::Por => "pt",
            Lang::Ita => "it",
            Lang::Kor => "ko",
            Lang::Hin => "hi",
            Lang::Tur => "tr",
            Lang::Pol => "pl",
            Lang::Nld => "nl",
            Lang::Swe => "sv",
            Lang::Vie => "vi",
            Lang::Tha => "th",
            _ => "unknown",
        }.to_string()
    })
}

/// Get language using both HTML attributes and content detection
pub fn get_language(html: &str, text: &str) -> Option<String> {
    // Try HTML attribute first (most reliable)
    if let Some(lang) = extract_language(html) {
        return Some(lang);
    }

    // Fallback to content-based detection
    detect_language_from_content(text)
}

#[cfg(test)]
mod language_detection_tests {
    use super::*;

    #[test]
    fn test_detect_english() {
        let text = "This is a longer English text that provides enough content for reliable language detection.";
        assert_eq!(detect_language_from_content(text), Some("en".to_string()));
    }

    #[test]
    fn test_detect_spanish() {
        let text = "Este es un texto más largo en español que proporciona suficiente contenido para una detección confiable del idioma.";
        assert_eq!(detect_language_from_content(text), Some("es".to_string()));
    }

    #[test]
    fn test_detect_insufficient_text() {
        let text = "Short";
        assert_eq!(detect_language_from_content(text), None);
    }

    #[test]
    fn test_get_language_prefers_html() {
        let html = r#"<html lang="fr"><body>This is English content but HTML says French</body></html>"#;
        let text = "This is English content but HTML says French";

        // Should return "fr" from HTML, not "en" from content
        assert_eq!(get_language(html, text), Some("fr".to_string()));
    }
}
```

---

## Feature 4: Category Extraction

### Add to `src/wasm_helpers.rs`

```rust
/// Extract categories/tags from HTML metadata and content
pub fn extract_categories(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut categories = Vec::new();

    // 1. Extract from meta keywords
    if let Ok(keywords_selector) = Selector::parse("meta[name='keywords'], meta[name='Keywords']")
    {
        for element in document.select(&keywords_selector) {
            if let Some(content) = element.value().attr("content") {
                categories.extend(
                    content
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                );
            }
        }
    }

    // 2. Extract from Open Graph article:tag
    if let Ok(og_tag_selector) = Selector::parse("meta[property='article:tag']") {
        for element in document.select(&og_tag_selector) {
            if let Some(content) = element.value().attr("content") {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    categories.push(trimmed);
                }
            }
        }
    }

    // 3. Extract from article:section
    if let Ok(section_selector) = Selector::parse("meta[property='article:section']") {
        for element in document.select(&section_selector) {
            if let Some(content) = element.value().attr("content") {
                let trimmed = content.trim().to_string();
                if !trimmed.is_empty() {
                    categories.push(trimmed);
                }
            }
        }
    }

    // 4. Extract from common tag/category class names
    let tag_selectors = [
        ".tags a",
        ".tag a",
        ".categories a",
        ".category a",
        "[rel='tag']",
        ".post-tags a",
        ".post-categories a",
        ".article-tags a",
        ".entry-tags a",
    ];

    for selector_str in &tag_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text: String = element.text().collect::<Vec<_>>().join(" ");
                let trimmed = text.trim().to_string();

                if !trimmed.is_empty()
                    && trimmed.len() < 50
                    && !categories.contains(&trimmed)
                {
                    categories.push(trimmed);
                }
            }
        }
    }

    // Clean up and deduplicate
    categories = categories
        .into_iter()
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty() && c.len() <= 50) // Reasonable length limit
        .collect();

    categories.sort();
    categories.dedup();
    categories.truncate(20); // Limit to 20 categories

    categories
}

#[cfg(test)]
mod category_extraction_tests {
    use super::*;

    #[test]
    fn test_extract_from_meta_keywords() {
        let html = r#"
            <html><head>
                <meta name="keywords" content="technology, AI, programming, rust">
            </head><body></body></html>
        "#;

        let categories = extract_categories(html);
        assert!(categories.contains(&"technology".to_string()));
        assert!(categories.contains(&"AI".to_string()));
        assert!(categories.contains(&"programming".to_string()));
        assert!(categories.contains(&"rust".to_string()));
    }

    #[test]
    fn test_extract_from_open_graph() {
        let html = r#"
            <html><head>
                <meta property="article:tag" content="Web Development">
                <meta property="article:tag" content="JavaScript">
                <meta property="article:section" content="Technology">
            </head><body></body></html>
        "#;

        let categories = extract_categories(html);
        assert!(categories.contains(&"Web Development".to_string()));
        assert!(categories.contains(&"JavaScript".to_string()));
        assert!(categories.contains(&"Technology".to_string()));
    }

    #[test]
    fn test_extract_from_html_tags() {
        let html = r#"
            <html><body>
                <div class="tags">
                    <a href="/tag/web">Web</a>
                    <a href="/tag/design">Design</a>
                </div>
                <div class="categories">
                    <a href="/cat/tutorials">Tutorials</a>
                </div>
            </body></html>
        "#;

        let categories = extract_categories(html);
        assert!(categories.contains(&"Web".to_string()));
        assert!(categories.contains(&"Design".to_string()));
        assert!(categories.contains(&"Tutorials".to_string()));
    }

    #[test]
    fn test_deduplication() {
        let html = r#"
            <html><head>
                <meta name="keywords" content="Technology, AI">
                <meta property="article:tag" content="Technology">
            </head>
            <body>
                <div class="tags"><a>Technology</a></div>
            </body></html>
        "#;

        let categories = extract_categories(html);
        assert_eq!(
            categories.iter().filter(|c| *c == "Technology").count(),
            1
        );
    }

    #[test]
    fn test_limit_categories() {
        let mut html = String::from("<html><head>");

        // Add 30 categories
        for i in 1..=30 {
            html.push_str(&format!(
                r#"<meta property="article:tag" content="Tag{}">"#,
                i
            ));
        }

        html.push_str("</head><body></body></html>");

        let categories = extract_categories(&html);
        assert!(categories.len() <= 20);
    }

    #[test]
    fn test_filter_long_categories() {
        let html = r#"
            <html><head>
                <meta name="keywords" content="short, this is a very long category name that exceeds fifty characters and should be filtered out">
            </head></html>
        "#;

        let categories = extract_categories(html);
        assert!(categories.contains(&"short".to_string()));
        assert_eq!(categories.len(), 1); // Long one filtered out
    }
}
```

---

## Integration: Update `src/lib_clean.rs`

### Update `convert_response_to_content()` function (lines 275-302)

```rust
/// Convert wasm-rs WasmResponse to Component Model ExtractedContent
fn convert_response_to_content(
    response: WasmResponse,
    url: &str,
    _mode: &ExtractionMode,
) -> Result<ExtractedContent, ExtractionError> {
    // Calculate quality score based on content richness
    let quality_score = calculate_quality_score(&response);
    let word_count = response.metadata.word_count as u32;
    let reading_time = estimate_reading_time(response.metadata.word_count);

    // Extract enhanced content features
    let links = extract_links(&response.html, url);
    let media = extract_media(&response.html, url);
    let language = get_language(&response.html, &response.content);
    let categories = extract_categories(&response.html);

    Ok(ExtractedContent {
        url: url.to_string(),
        title: Some(response.metadata.title).filter(|s| !s.is_empty()),
        byline: Some(response.metadata.author).filter(|s| !s.is_empty()),
        published_iso: Some(response.metadata.published).filter(|s| !s.is_empty()),
        markdown: response.content_markdown.unwrap_or_default(),
        text: response.content,
        links: links.into_iter().map(|l| format!("{}: {}", l.url, l.text.unwrap_or_default())).collect(),
        media: media.into_iter().map(|m| m.url).collect(),
        language,
        reading_time,
        quality_score: Some(quality_score),
        word_count: Some(word_count),
        categories,
        site_name: Some(response.metadata.site).filter(|s| !s.is_empty()),
        description: Some(response.metadata.description).filter(|s| !s.is_empty()),
    })
}
```

### Add imports to top of `src/lib_clean.rs`

```rust
mod wasm_helpers;
use wasm_helpers::{
    extract_links,
    extract_media,
    extract_language,
    detect_language_from_content,
    get_language,
    extract_categories,
    Link,
    MediaItem,
    MediaType,
};
```

---

## Dependencies: Update `Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...

# For language detection (optional but recommended)
whatlang = "0.16"

# Already have these:
scraper = "0.20"  # For HTML parsing
url = "2.5"       # For URL resolution
```

---

## Testing: Add integration test

### Add to `tests/integration/mod.rs` at end of file

```rust
#[test]
fn test_enhanced_content_extraction_features() {
    println!("\n🆕 Testing Enhanced Content Extraction Features...");

    let component = Component;

    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta name="keywords" content="technology, web development, rust">
            <meta property="article:tag" content="Programming">
            <title>Test Article</title>
        </head>
        <body>
            <article>
                <h1>Test Article Title</h1>
                <p>This is a test article with enough content for language detection.</p>
                <p>It contains <a href="https://example.com/link1">external links</a> and
                   <a href="/internal">internal links</a>.</p>
                <img src="photo.jpg" alt="Test Photo">
                <video><source src="video.mp4" type="video/mp4"></video>
            </article>
            <div class="tags">
                <a href="/tag/tutorial">Tutorial</a>
                <a href="/tag/guide">Guide</a>
            </div>
        </body>
        </html>
    "#;

    let result = component.extract(
        html.to_string(),
        "https://test.example.com/article".to_string(),
        ExtractionMode::Article,
    );

    assert!(result.is_ok(), "Enhanced extraction should succeed");

    let content = result.unwrap();

    // Test link extraction
    assert!(
        !content.links.is_empty(),
        "Should extract links (found: {})",
        content.links.len()
    );
    println!("  ✅ Extracted {} links", content.links.len());

    // Test media extraction
    assert!(
        !content.media.is_empty(),
        "Should extract media (found: {})",
        content.media.len()
    );
    println!("  ✅ Extracted {} media items", content.media.len());

    // Test language detection
    assert!(
        content.language.is_some(),
        "Should detect language"
    );
    println!("  ✅ Detected language: {:?}", content.language);

    // Test category extraction
    assert!(
        !content.categories.is_empty(),
        "Should extract categories (found: {})",
        content.categories.len()
    );
    println!("  ✅ Extracted {} categories", content.categories.len());

    // Verify specific categories
    assert!(
        content.categories.iter().any(|c| c.contains("technology") || c.contains("Programming")),
        "Should contain expected categories"
    );

    println!("  🎉 All enhanced features working!");
}
```

---

## Verification Commands

```bash
# 1. Build with new features
cd /workspaces/eventmesh/wasm/riptide-extractor-wasm
cargo build --release

# 2. Run unit tests
cargo test --lib wasm_helpers

# 3. Run integration tests
cargo test --test test_runner test_enhanced_content_extraction_features

# 4. Run full suite
cargo test --package riptide-extractor-wasm

# 5. Check performance
cargo test --release --test test_runner regression_test_performance_baseline
```

---

## Performance Expectations

| Feature | Time Impact | Memory Impact |
|---------|-------------|---------------|
| Link Extraction | +0.5-2ms | +1-10 KB |
| Media Extraction | +0.5-2ms | +1-4 KB |
| Language Detection (HTML) | +0.1ms | +2 bytes |
| Language Detection (content) | +1-5ms | +2 bytes |
| Category Extraction | +0.5-1ms | +500 bytes |
| **Total** | **+3-10ms** | **+5-15 KB** |

All features should keep total extraction time under 50ms target.

---

## Ready to Implement?

1. Copy each feature section to the appropriate file
2. Add imports to `lib_clean.rs`
3. Update `convert_response_to_content()`
4. Add `whatlang` to `Cargo.toml` (if using content-based language detection)
5. Run tests to verify

**Each feature is independent and can be implemented separately!**

Good luck! 🚀
