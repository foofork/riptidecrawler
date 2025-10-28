//! Media extraction from HTML documents

use scraper::{Html, Selector};
use url::Url;

pub struct MediaExtractor;

impl MediaExtractor {
    /// Extract media (images, videos) from document
    pub fn extract(document: &Html, base_url: &str) -> Vec<String> {
        let mut media = Vec::new();
        let base = Url::parse(base_url).ok();

        // Extract images
        if let Ok(selector) = Selector::parse("img[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    let resolved = if let Some(ref base) = base {
                        base.join(src).ok().map(|u| u.to_string())
                    } else {
                        Some(src.to_string())
                    };

                    if let Some(url) = resolved {
                        if Self::is_valid_media_url(&url) {
                            media.push(url);
                        }
                    }
                }
            }
        }

        // Extract video sources
        if let Ok(selector) = Selector::parse("video source[src], video[src]") {
            for element in document.select(&selector) {
                if let Some(src) = element.value().attr("src") {
                    let resolved = if let Some(ref base) = base {
                        base.join(src).ok().map(|u| u.to_string())
                    } else {
                        Some(src.to_string())
                    };

                    if let Some(url) = resolved {
                        if Self::is_valid_media_url(&url) {
                            media.push(url);
                        }
                    }
                }
            }
        }

        media
    }

    fn is_valid_media_url(url: &str) -> bool {
        // Must start with http:// or https:// or be a data URL
        if url.starts_with("data:") {
            return true;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return false;
        }

        // Check for common media extensions
        let media_extensions = [
            ".jpg", ".jpeg", ".png", ".gif", ".webp", ".svg", ".mp4", ".webm", ".ogg", ".mov",
        ];

        media_extensions
            .iter()
            .any(|ext| url.to_lowercase().contains(ext))
    }
}
