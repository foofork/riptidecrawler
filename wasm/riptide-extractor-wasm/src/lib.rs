use serde::Serialize;
use std::io::{Read, Write};

#[derive(Serialize)]
struct ExtractedDocOut {
    url: String,
    title: Option<String>,
    byline: Option<String>,
    published_iso: Option<String>,
    markdown: String,
    text: String,
    links: Vec<String>,
    media: Vec<String>,
}

#[no_mangle]
pub extern "C" fn _start() {
    // Read HTML from stdin
    let mut html = Vec::new();
    std::io::stdin().read_to_end(&mut html).unwrap();

    // Read URL & mode from env
    let url = std::env::var("RIPTIDE_URL").unwrap_or_else(|_| "about:blank".into());
    let _mode = std::env::var("RIPTIDE_MODE").unwrap_or_else(|_| "article".into());

    // Use simple extraction for now
    let html_str = String::from_utf8_lossy(&html);

    // Use simple extraction for now (TODO: integrate trek-rs properly)
    let title = extract_title(&html_str);
    let content = extract_content(&html_str);

    let out = ExtractedDocOut {
        url,
        title,
        byline: None,
        published_iso: None,
        markdown: content.clone(),
        text: strip_html_tags(&content),
        links: extract_links(&html_str),
        media: extract_images(&html_str),
    };

    let json = serde_json::to_vec(&out).unwrap();
    std::io::stdout().write_all(&json).unwrap();
}

fn extract_title(html: &str) -> Option<String> {
    // Simple title extraction as fallback
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start..].find("</title>") {
            let title = &html[start + 7..start + end];
            return Some(title.to_string());
        }
    }
    None
}

fn strip_html_tags(html: &str) -> String {
    // Simple HTML tag removal
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

    result.trim().to_string()
}

fn extract_links(html: &str) -> Vec<String> {
    // Simple link extraction
    let mut links = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find("href=\"") {
        let start = pos + start + 6;
        if let Some(end) = html[start..].find('"') {
            let link = &html[start..start + end];
            if link.starts_with("http") {
                links.push(link.to_string());
            }
        }
        pos = start;
    }

    links
}

fn extract_images(html: &str) -> Vec<String> {
    // Simple image extraction
    let mut images = Vec::new();
    let mut pos = 0;

    while let Some(start) = html[pos..].find("src=\"") {
        let start = pos + start + 5;
        if let Some(end) = html[start..].find('"') {
            let src = &html[start..start + end];
            if src.starts_with("http") {
                images.push(src.to_string());
            }
        }
        pos = start;
    }

    images
}

fn extract_content(html: &str) -> String {
    // Simple content extraction - look for main content areas
    let content_tags = ["<article", "<main", "<div class=\"content", "<div id=\"content"];

    for tag in &content_tags {
        if let Some(start) = html.find(tag) {
            // Find the closing tag
            let tag_name = tag.trim_start_matches('<').split(' ').next().unwrap_or("div");
            let closing_tag = format!("</{}>", tag_name);

            if let Some(end) = html[start..].find(&closing_tag) {
                let content = &html[start..start + end + closing_tag.len()];
                return strip_html_tags(content);
            }
        }
    }

    // Fallback: extract everything in body
    if let Some(start) = html.find("<body") {
        if let Some(body_start) = html[start..].find('>') {
            let body_start = start + body_start + 1;
            if let Some(end) = html[body_start..].find("</body>") {
                let content = &html[body_start..body_start + end];
                return strip_html_tags(content);
            }
        }
    }

    // Final fallback: strip all HTML
    strip_html_tags(html)
}