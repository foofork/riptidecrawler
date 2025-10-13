use trek_rs::TrekResponse;

/// Calculate content quality score based on trek-rs TrekResponse (0-100)
pub fn calculate_quality_score(response: &TrekResponse) -> u8 {
    let mut score = 30u8; // Base score

    // Title quality (0-15 points)
    if !response.metadata.title.is_empty() {
        let title_len = response.metadata.title.len();
        if title_len > 10 && title_len < 100 {
            score += 15;
        } else if title_len > 5 {
            score += 8;
        }
    }

    // Content length (0-20 points)
    let content_len = response.content.len();
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
    if !response.metadata.author.is_empty() {
        score += 10;
    }

    // Publication date (0-10 points)
    if !response.metadata.published.is_empty() {
        score += 10;
    }

    // Word count (0-10 points)
    if response.metadata.word_count > 500 {
        score += 10;
    } else if response.metadata.word_count > 200 {
        score += 5;
    }

    // Meta tags (0-5 points)
    if !response.meta_tags.is_empty() {
        score += 5;
    }

    score.min(100)
}

/// Estimate reading time in minutes based on word count
pub fn estimate_reading_time(word_count: usize) -> Option<u32> {
    if word_count == 0 {
        return None;
    }

    // Average reading speed: 200-250 words per minute
    let reading_time = (word_count as f32 / 225.0).ceil() as u32;
    Some(reading_time.max(1))
}

/// Count words in text
#[allow(dead_code)]
pub fn count_words(text: &str) -> u32 {
    text.split_whitespace().count() as u32
}

/// Get trek-rs version
pub fn get_trek_version() -> String {
    // Use the actual version we're using
    "0.1.1".to_string()
}

/// Extract links from HTML content
#[allow(dead_code)]
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let link_selector = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mut links = Vec::new();
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Skip empty, anchor-only, javascript:, and mailto: links
            if href.is_empty()
                || href.starts_with('#')
                || href.starts_with("javascript:")
                || href.starts_with("mailto:")
            {
                continue;
            }

            // Try to resolve relative URLs
            if let Ok(base) = url::Url::parse(base_url) {
                if let Ok(resolved) = base.join(href) {
                    links.push(resolved.to_string());
                    continue;
                }
            }

            // If resolution fails, include the raw href if it looks like a URL
            if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("//")
            {
                links.push(href.to_string());
            }
        }
    }

    // Deduplicate while preserving order
    let mut seen = std::collections::HashSet::new();
    links.retain(|link| seen.insert(link.clone()));

    // Limit to reasonable number of links
    links.truncate(200);

    links
}

/// Extract media URLs (images, videos, audio) from HTML
#[allow(dead_code)]
pub fn extract_media(html: &str, base_url: &str) -> Vec<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let mut media = Vec::new();

    // Extract images from <img> tags
    if let Ok(img_selector) = Selector::parse("img[src]") {
        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                if !src.is_empty() && !src.starts_with("data:") {
                    if let Some(resolved) = resolve_media_url(src, base_url) {
                        media.push(resolved);
                    }
                }
            }
        }
    }

    // Extract videos from <video> and <source> tags
    if let Ok(video_selector) = Selector::parse("video[src], video source[src]") {
        for element in document.select(&video_selector) {
            if let Some(src) = element.value().attr("src") {
                if !src.is_empty() {
                    if let Some(resolved) = resolve_media_url(src, base_url) {
                        media.push(resolved);
                    }
                }
            }
        }
    }

    // Extract audio from <audio> and <source> tags
    if let Ok(audio_selector) = Selector::parse("audio[src], audio source[src]") {
        for element in document.select(&audio_selector) {
            if let Some(src) = element.value().attr("src") {
                if !src.is_empty() {
                    if let Some(resolved) = resolve_media_url(src, base_url) {
                        media.push(resolved);
                    }
                }
            }
        }
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    media.retain(|url| seen.insert(url.clone()));

    // Limit to reasonable number
    media.truncate(100);

    media
}

/// Helper to resolve media URLs
#[allow(dead_code)]
fn resolve_media_url(src: &str, base_url: &str) -> Option<String> {
    if src.starts_with("http://") || src.starts_with("https://") {
        return Some(src.to_string());
    }

    if let Ok(base) = url::Url::parse(base_url) {
        if let Ok(resolved) = base.join(src) {
            return Some(resolved.to_string());
        }
    }

    None
}

/// Detect content language from HTML and text
#[allow(dead_code)]
pub fn detect_language(html: &str, text: &str) -> Option<String> {
    use scraper::{Html, Selector};

    // Try HTML lang attribute first (most reliable)
    let document = Html::parse_document(html);
    if let Ok(html_selector) = Selector::parse("html[lang]") {
        if let Some(element) = document.select(&html_selector).next() {
            if let Some(lang) = element.value().attr("lang") {
                // Extract just the language code (e.g., "en" from "en-US")
                let lang_code = lang.split('-').next().unwrap_or(lang);
                if !lang_code.is_empty() && lang_code.len() <= 3 {
                    return Some(lang_code.to_lowercase());
                }
            }
        }
    }

    // Fallback to content-based detection if text is substantial
    if text.len() >= 30 {
        if let Some(info) = whatlang::detect(text) {
            return Some(info.lang().code().to_string());
        }
    }

    None
}

/// Extract categories/tags from HTML metadata
#[allow(dead_code)]
pub fn extract_categories(html: &str) -> Vec<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let mut categories = Vec::new();

    // Extract from meta keywords
    if let Ok(keywords_selector) = Selector::parse("meta[name='keywords'], meta[name='Keywords']") {
        for element in document.select(&keywords_selector) {
            if let Some(content) = element.value().attr("content") {
                for keyword in content.split(',') {
                    let trimmed = keyword.trim();
                    if !trimmed.is_empty() && trimmed.len() < 50 {
                        categories.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    // Extract from Open Graph article:tag
    if let Ok(og_selector) = Selector::parse("meta[property='article:tag']") {
        for element in document.select(&og_selector) {
            if let Some(content) = element.value().attr("content") {
                let trimmed = content.trim();
                if !trimmed.is_empty() && trimmed.len() < 50 {
                    categories.push(trimmed.to_string());
                }
            }
        }
    }

    // Extract from common tag/category elements
    let selectors = [
        ".tags a",
        ".tag",
        ".categories a",
        ".category",
        "[rel='tag']",
        ".post-tags a",
        ".post-tag",
    ];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text: String = element.text().collect();
                let trimmed = text.trim();
                if !trimmed.is_empty() && trimmed.len() < 50 {
                    categories.push(trimmed.to_string());
                }
            }
        }
    }

    // Deduplicate and clean
    let mut seen = std::collections::HashSet::new();
    categories.retain(|cat| seen.insert(cat.to_lowercase()));

    // Limit to 20 categories
    categories.truncate(20);

    categories
}
