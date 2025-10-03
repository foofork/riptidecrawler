use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use url::Url;
use whatlang::{detect, Lang};

/// Extract all links from HTML with full attributes
pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut links = Vec::new();

    // Parse base URL for resolution
    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return links, // Return empty if base URL is invalid
    };

    // Extract links from <a href> elements
    if let Ok(selector) = Selector::parse("a[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                // Resolve relative URLs to absolute
                if let Ok(absolute_url) = base.join(href) {
                    let link_info = format_link_with_attributes(element, absolute_url.as_str());
                    links.push(link_info);
                }
            }
        }
    }

    // Extract links from area elements (image maps)
    if let Ok(selector) = Selector::parse("area[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base.join(href) {
                    links.push(absolute_url.to_string());
                }
            }
        }
    }

    // Extract canonical links
    if let Ok(selector) = Selector::parse("link[rel='canonical'][href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base.join(href) {
                    links.push(format!("canonical:{}", absolute_url));
                }
            }
        }
    }

    links
}

/// Format link with attributes (text, rel, hreflang)
fn format_link_with_attributes(element: scraper::ElementRef, url: &str) -> String {
    let text = element.text().collect::<String>().trim().to_string();
    let rel = element.value().attr("rel").unwrap_or("");
    let hreflang = element.value().attr("hreflang").unwrap_or("");

    // Format as JSON-like string for structured data
    format!(
        "{{\"url\":\"{}\",\"text\":\"{}\",\"rel\":\"{}\",\"hreflang\":\"{}\"}}",
        url,
        text.replace('"', "\\\""),
        rel,
        hreflang
    )
}

/// Extract all media URLs (images, videos, audio)
pub fn extract_media(html: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut media = Vec::new();

    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return media,
    };

    // Extract img src and srcset
    if let Ok(selector) = Selector::parse("img") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(absolute_url) = base.join(src) {
                    media.push(format!("image:{}", absolute_url));
                }
            }
            if let Some(srcset) = element.value().attr("srcset") {
                // Parse srcset format: "url 1x, url 2x" or "url 100w, url 200w"
                for src_part in srcset.split(',') {
                    let src = src_part.split_whitespace().next().unwrap_or("");
                    if !src.is_empty() {
                        if let Ok(absolute_url) = base.join(src) {
                            media.push(format!("image:{}", absolute_url));
                        }
                    }
                }
            }
        }
    }

    // Extract picture > source srcset
    if let Ok(selector) = Selector::parse("picture source[srcset]") {
        for element in document.select(&selector) {
            if let Some(srcset) = element.value().attr("srcset") {
                for src_part in srcset.split(',') {
                    let src = src_part.split_whitespace().next().unwrap_or("");
                    if !src.is_empty() {
                        if let Ok(absolute_url) = base.join(src) {
                            media.push(format!("image:{}", absolute_url));
                        }
                    }
                }
            }
        }
    }

    // Extract video sources
    if let Ok(selector) = Selector::parse("video source[src], video[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(absolute_url) = base.join(src) {
                    media.push(format!("video:{}", absolute_url));
                }
            }
        }
    }

    // Extract audio sources
    if let Ok(selector) = Selector::parse("audio source[src], audio[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src") {
                if let Ok(absolute_url) = base.join(src) {
                    media.push(format!("audio:{}", absolute_url));
                }
            }
        }
    }

    // Extract Open Graph images
    if let Ok(selector) =
        Selector::parse("meta[property='og:image'], meta[property='og:image:url']")
    {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                if let Ok(absolute_url) = base.join(content) {
                    media.push(format!("og:image:{}", absolute_url));
                }
            }
        }
    }

    // Extract favicons and touch icons
    if let Ok(selector) =
        Selector::parse("link[rel*='icon'][href], link[rel*='apple-touch-icon'][href]")
    {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Ok(absolute_url) = base.join(href) {
                    let rel = element.value().attr("rel").unwrap_or("icon");
                    media.push(format!("{}:{}", rel, absolute_url));
                }
            }
        }
    }

    media
}

/// Detect page language using multiple methods
pub fn detect_language(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // Priority 1: <html lang> attribute
    if let Ok(selector) = Selector::parse("html[lang]") {
        if let Some(element) = document.select(&selector).next() {
            if let Some(lang) = element.value().attr("lang") {
                let normalized = normalize_lang(lang);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    // Priority 2: meta og:locale
    if let Ok(selector) = Selector::parse("meta[property='og:locale']") {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let normalized = normalize_lang(content);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    // Priority 3: JSON-LD inLanguage
    if let Ok(selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if let Some(lang) = extract_json_ld_language(&json) {
                    let normalized = normalize_lang(&lang);
                    if !normalized.is_empty() {
                        return Some(normalized);
                    }
                }
            }
        }
    }

    // Priority 4: Content-Language meta tag
    if let Ok(selector) = Selector::parse("meta[http-equiv='Content-Language']") {
        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let normalized = normalize_lang(content);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    // Priority 5: Automatic detection from text content
    let text_content = extract_text_for_detection(&document);
    if !text_content.is_empty() {
        if let Some(info) = detect(&text_content) {
            return Some(lang_to_iso_code(info.lang()));
        }
    }

    None
}

/// Extract language from JSON-LD structured data
fn extract_json_ld_language(json: &Value) -> Option<String> {
    // Check for inLanguage field
    if let Some(lang) = json.get("inLanguage") {
        if let Some(lang_str) = lang.as_str() {
            return Some(lang_str.to_string());
        }
    }

    // Check in nested objects and arrays
    match json {
        Value::Object(map) => {
            for value in map.values() {
                if let Some(lang) = extract_json_ld_language(value) {
                    return Some(lang);
                }
            }
        }
        Value::Array(arr) => {
            for value in arr {
                if let Some(lang) = extract_json_ld_language(value) {
                    return Some(lang);
                }
            }
        }
        _ => {}
    }

    None
}

/// Extract text content for language detection
fn extract_text_for_detection(document: &Html) -> String {
    let mut text = String::new();

    // Extract from common text containers
    let selectors = ["title", "h1", "h2", "h3", "p", "article", "main"];

    for selector_str in &selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector).take(10) {
                // Limit for efficiency
                let element_text = element.text().collect::<String>();
                if !element_text.trim().is_empty() {
                    text.push_str(&element_text);
                    text.push(' ');
                }
            }
        }
    }

    // Limit text length for detection efficiency
    if text.len() > 1000 {
        text.truncate(1000);
    }

    text
}

/// Normalize language codes to ISO 639-1
fn normalize_lang(lang: &str) -> String {
    let lang = lang.trim().to_lowercase();

    // Handle common formats
    if lang.contains('-') {
        // en-US -> en, zh-CN -> zh
        lang.split('-').next().unwrap_or("").to_string()
    } else if lang.contains('_') {
        // en_US -> en
        lang.split('_').next().unwrap_or("").to_string()
    } else {
        lang
    }
}

/// Convert whatlang Lang enum to ISO 639-1 code
fn lang_to_iso_code(lang: Lang) -> String {
    match lang {
        Lang::Eng => "en".to_string(),
        Lang::Rus => "ru".to_string(),
        Lang::Cmn => "zh".to_string(),
        Lang::Spa => "es".to_string(),
        Lang::Por => "pt".to_string(),
        Lang::Ita => "it".to_string(),
        Lang::Ben => "bn".to_string(),
        Lang::Fra => "fr".to_string(),
        Lang::Deu => "de".to_string(),
        Lang::Ukr => "uk".to_string(),
        Lang::Kat => "ka".to_string(),
        Lang::Ara => "ar".to_string(),
        Lang::Hin => "hi".to_string(),
        Lang::Jpn => "ja".to_string(),
        Lang::Heb => "he".to_string(),
        Lang::Yid => "yi".to_string(),
        Lang::Pol => "pl".to_string(),
        Lang::Amh => "am".to_string(),
        Lang::Jav => "jv".to_string(),
        Lang::Kor => "ko".to_string(),
        Lang::Nob => "no".to_string(),
        Lang::Dan => "da".to_string(),
        Lang::Swe => "sv".to_string(),
        Lang::Fin => "fi".to_string(),
        Lang::Tur => "tr".to_string(),
        Lang::Nld => "nl".to_string(),
        Lang::Hun => "hu".to_string(),
        Lang::Ces => "cs".to_string(),
        Lang::Ell => "el".to_string(),
        Lang::Bul => "bg".to_string(),
        Lang::Bel => "be".to_string(),
        Lang::Mar => "mr".to_string(),
        Lang::Kan => "kn".to_string(),
        Lang::Ron => "ro".to_string(),
        Lang::Slv => "sl".to_string(),
        Lang::Hrv => "hr".to_string(),
        Lang::Srp => "sr".to_string(),
        Lang::Mkd => "mk".to_string(),
        Lang::Lit => "lt".to_string(),
        Lang::Lav => "lv".to_string(),
        Lang::Est => "et".to_string(),
        Lang::Tam => "ta".to_string(),
        Lang::Vie => "vi".to_string(),
        Lang::Urd => "ur".to_string(),
        Lang::Tha => "th".to_string(),
        Lang::Guj => "gu".to_string(),
        Lang::Uzb => "uz".to_string(),
        Lang::Pan => "pa".to_string(),
        Lang::Aze => "az".to_string(),
        Lang::Ind => "id".to_string(),
        Lang::Tel => "te".to_string(),
        Lang::Pes => "fa".to_string(),
        Lang::Mal => "ml".to_string(),
        Lang::Ori => "or".to_string(),
        Lang::Mya => "my".to_string(),
        Lang::Nep => "ne".to_string(),
        Lang::Sin => "si".to_string(),
        Lang::Khm => "km".to_string(),
        Lang::Tuk => "tk".to_string(),
        Lang::Aka => "ak".to_string(),
        Lang::Zul => "zu".to_string(),
        Lang::Sna => "so".to_string(),
        Lang::Afr => "af".to_string(),
        Lang::Lat => "la".to_string(),
        Lang::Slk => "sk".to_string(),
        Lang::Cat => "ca".to_string(),
        Lang::Tgl => "tl".to_string(),
        Lang::Hye => "hy".to_string(),
        Lang::Epo => "eo".to_string(),
    }
}

/// Extract categories from various sources
pub fn extract_categories(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut categories = Vec::new();

    // Extract from JSON-LD articleSection
    if let Ok(selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                extract_json_ld_categories(&json, &mut categories);
            }
        }
    }

    // Extract from breadcrumb schemas
    extract_breadcrumb_categories(&document, &mut categories);

    // Extract from meta category tags
    if let Ok(selector) = Selector::parse("meta[name='category'], meta[name='categories'], meta[property='article:section'], meta[property='article:tag']") {
        for element in document.select(&selector) {
            if let Some(content) = element.value().attr("content") {
                for category in content.split(',') {
                    let trimmed = category.trim();
                    if !trimmed.is_empty() && !categories.contains(&trimmed.to_string()) {
                        categories.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    // Extract from Open Graph article tags
    if let Ok(selector) = Selector::parse("meta[property^='article:']") {
        for element in document.select(&selector) {
            if let Some(property) = element.value().attr("property") {
                if property.contains("tag") || property.contains("section") {
                    if let Some(content) = element.value().attr("content") {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() && !categories.contains(&trimmed.to_string()) {
                            categories.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    // Extract from class names that suggest categories
    if let Ok(selector) = Selector::parse("[class*='category'], [class*='tag'], [class*='topic']") {
        for element in document.select(&selector).take(10) {
            // Limit for performance
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() && text.len() < 50 && !categories.contains(&text) {
                // Only add if it looks like a category (short text)
                if is_likely_category(&text) {
                    categories.push(text);
                }
            }
        }
    }

    // Deduplicate and clean up
    categories.sort();
    categories.dedup();
    categories.truncate(20); // Limit to prevent excessive data

    categories
}

/// Extract categories from JSON-LD structured data
fn extract_json_ld_categories(json: &Value, categories: &mut Vec<String>) {
    match json {
        Value::Object(map) => {
            // Check for articleSection
            if let Some(section) = map.get("articleSection") {
                match section {
                    Value::String(s) => {
                        if !categories.contains(s) {
                            categories.push(s.clone());
                        }
                    }
                    Value::Array(arr) => {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                if !categories.contains(&s.to_string()) {
                                    categories.push(s.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check for keywords
            if let Some(keywords) = map.get("keywords") {
                match keywords {
                    Value::String(s) => {
                        for keyword in s.split(',') {
                            let trimmed = keyword.trim().to_string();
                            if !trimmed.is_empty() && !categories.contains(&trimmed) {
                                categories.push(trimmed);
                            }
                        }
                    }
                    Value::Array(arr) => {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                if !categories.contains(&s.to_string()) {
                                    categories.push(s.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Recursively check nested objects
            for value in map.values() {
                extract_json_ld_categories(value, categories);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                extract_json_ld_categories(item, categories);
            }
        }
        _ => {}
    }
}

/// Extract categories from breadcrumb navigation
fn extract_breadcrumb_categories(document: &Html, categories: &mut Vec<String>) {
    // JSON-LD BreadcrumbList
    if let Ok(selector) = Selector::parse("script[type='application/ld+json']") {
        for element in document.select(&selector) {
            let text = element.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                if let Some(type_val) = json.get("@type") {
                    if type_val == "BreadcrumbList" {
                        if let Some(Value::Array(arr)) = json.get("itemListElement") {
                            for item in arr {
                                if let Some(name) = item.get("name") {
                                    if let Some(name_str) = name.as_str() {
                                        let trimmed = name_str.trim().to_string();
                                        if !trimmed.is_empty() && !categories.contains(&trimmed) {
                                            categories.push(trimmed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // HTML breadcrumb elements
    let breadcrumb_selectors = [
        "nav[aria-label*='breadcrumb'] a",
        ".breadcrumb a",
        ".breadcrumbs a",
        "[role='navigation'] a",
    ];

    for selector_str in &breadcrumb_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<String>().trim().to_string();
                if !text.is_empty()
                    && text.len() < 100
                    && !categories.contains(&text)
                    && is_likely_category(&text)
                {
                    categories.push(text);
                }
            }
        }
    }
}

/// Check if text looks like a category (heuristic)
fn is_likely_category(text: &str) -> bool {
    let text = text.trim();

    // Basic checks
    if text.len() < 2 || text.len() > 50 {
        return false;
    }

    // Skip common non-category words
    let skip_words = [
        "home",
        "index",
        "main",
        "page",
        "click",
        "here",
        "read more",
        "continue",
    ];
    let lower_text = text.to_lowercase();

    for skip in &skip_words {
        if lower_text.contains(skip) {
            return false;
        }
    }

    // Skip if it looks like a sentence (has common sentence patterns)
    if text.chars().filter(|&c| c == ' ').count() > 3 {
        return false;
    }

    // Skip if it contains numbers in a non-category way
    let re = Regex::new(r"\d{4}|\d+\.\d+|page\s+\d+").unwrap();
    if re.is_match(&lower_text) {
        return false;
    }

    true
}
