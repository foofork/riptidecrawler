//! Language detection for HTML documents

use scraper::{Html, Selector};

pub struct LanguageDetector;

impl LanguageDetector {
    /// Detect document language
    pub fn detect(document: &Html, _html: &str) -> Option<String> {
        // Priority 1: html lang attribute
        if let Some(lang) = Self::extract_html_lang(document) {
            return Some(lang);
        }

        // Priority 2: meta content-language
        if let Some(lang) = Self::extract_meta_language(document) {
            return Some(lang);
        }

        // Priority 3: og:locale
        Self::extract_og_locale(document)
    }

    fn extract_html_lang(document: &Html) -> Option<String> {
        let selector = Selector::parse("html[lang]").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("lang")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_meta_language(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[http-equiv='content-language']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn extract_og_locale(document: &Html) -> Option<String> {
        let selector = Selector::parse("meta[property='og:locale']").ok()?;
        document
            .select(&selector)
            .next()?
            .value()
            .attr("content")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}
