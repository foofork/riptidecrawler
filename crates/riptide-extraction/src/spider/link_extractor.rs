//! HTML link extraction functionality
//!
//! Extracts URLs from HTML content using proper HTML parsing instead of regex

use crate::spider::traits::{DomSpiderConfig, ContentAnalysis, ContentType, NavigationHint};
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use std::collections::HashSet;
use url::Url;

/// HTML link extractor with advanced parsing capabilities
pub struct HtmlLinkExtractor {
    config: DomSpiderConfig,
}

impl HtmlLinkExtractor {
    /// Create a new link extractor with configuration
    pub fn new(config: DomSpiderConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(DomSpiderConfig::default())
    }

    /// Extract all links from HTML content using proper DOM parsing
    pub async fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        let document = Html::parse_document(html);
        let mut links = Vec::new();
        let mut seen_urls = HashSet::new();

        // Define selectors for different types of links
        let link_selectors = [
            ("a[href]", "href"),
            ("link[href]", "href"),
            ("area[href]", "href"),
            ("img[src]", "src"),
            ("iframe[src]", "src"),
            ("script[src]", "src"),
            ("form[action]", "action"),
        ];

        for (selector_str, attr) in &link_selectors {
            let selector = Selector::parse(selector_str)
                .map_err(|e| anyhow::anyhow!("Invalid CSS selector '{}': {}", selector_str, e))?;

            for element in document.select(&selector) {
                if let Some(url_str) = element.value().attr(attr) {
                    if self.should_extract_link(url_str) {
                        match self.resolve_url(url_str, base_url) {
                            Ok(resolved_url) => {
                                let url_string = resolved_url.to_string();
                                if seen_urls.insert(url_string) {
                                    links.push(resolved_url);

                                    // Check max links limit
                                    if let Some(max_links) = self.config.max_links_per_page {
                                        if links.len() >= max_links {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Skip invalid URLs
                                continue;
                            }
                        }
                    }
                }
            }

            // Break early if we've reached the limit
            if let Some(max_links) = self.config.max_links_per_page {
                if links.len() >= max_links {
                    break;
                }
            }
        }

        // Filter external links if configured
        if !self.config.extract_external_links {
            links.retain(|url| self.is_same_domain(url, base_url));
        }

        Ok(links)
    }

    /// Extract navigation links specifically (for better crawling strategy)
    pub async fn extract_navigation_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        let document = Html::parse_document(html);
        let mut nav_links = Vec::new();

        // Navigation-specific selectors
        let nav_selectors = [
            "nav a[href]",
            ".navigation a[href]",
            ".nav a[href]",
            ".menu a[href]",
            ".breadcrumb a[href]",
            ".pagination a[href]",
            "header a[href]",
            "footer a[href]",
        ];

        for selector_str in &nav_selectors {
            let selector = Selector::parse(selector_str)
                .map_err(|e| anyhow::anyhow!("Invalid CSS selector '{}': {}", selector_str, e))?;

            for element in document.select(&selector) {
                if let Some(href) = element.value().attr("href") {
                    if let Ok(url) = self.resolve_url(href, base_url) {
                        nav_links.push(url);
                    }
                }
            }
        }

        Ok(nav_links)
    }

    /// Extract content links (non-navigation)
    pub async fn extract_content_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        let all_links = self.extract_links(html, base_url).await?;
        let nav_links: HashSet<Url> = self.extract_navigation_links(html, base_url).await?
            .into_iter()
            .collect();

        // Return links that are not in navigation
        Ok(all_links
            .into_iter()
            .filter(|url| !nav_links.contains(url))
            .collect())
    }

    /// Analyze link structure for content optimization
    pub async fn analyze_link_structure(&self, html: &str, base_url: &Url) -> Result<ContentAnalysis> {
        let document = Html::parse_document(html);
        let all_links = self.extract_links(html, base_url).await?;
        let nav_links = self.extract_navigation_links(html, base_url).await?;

        // Calculate link density
        let text_content = self.extract_text_content(&document);
        let text_length = text_content.len();
        let link_density = if text_length > 0 {
            all_links.len() as f64 / text_length as f64 * 1000.0 // Links per 1000 chars
        } else {
            0.0
        };

        // Calculate internal vs external link ratio
        let internal_links = all_links.iter()
            .filter(|url| self.is_same_domain(url, base_url))
            .count();
        let internal_link_ratio = if !all_links.is_empty() {
            internal_links as f64 / all_links.len() as f64
        } else {
            0.0
        };

        // Detect content type based on link patterns
        let content_type = self.detect_content_type(&document, &all_links, &nav_links);

        // Extract navigation hints
        let navigation_hints = self.extract_navigation_hints(&document)?;

        // Calculate unique text characters
        let unique_text_chars = self.count_unique_chars(&text_content);

        // Calculate content quality score
        let quality_score = self.calculate_quality_score(
            &all_links,
            &nav_links,
            text_length,
            unique_text_chars,
        );

        Ok(ContentAnalysis {
            content_type,
            link_density,
            quality_score,
            unique_text_chars,
            internal_link_ratio,
            navigation_hints,
            freshness_indicators: self.extract_freshness_indicators(&document),
        })
    }

    /// Check if a link should be extracted based on configuration
    fn should_extract_link(&self, url_str: &str) -> bool {
        // Check ignore patterns
        for pattern in &self.config.ignore_link_patterns {
            if url_str.starts_with(pattern.trim_start_matches('^')) {
                return false;
            }
        }

        // Basic validation
        !url_str.trim().is_empty()
    }

    /// Resolve relative URLs to absolute URLs
    fn resolve_url(&self, url_str: &str, base_url: &Url) -> Result<Url> {
        if url_str.starts_with("http://") || url_str.starts_with("https://") {
            Url::parse(url_str).context("Failed to parse absolute URL")
        } else {
            base_url.join(url_str).context("Failed to resolve relative URL")
        }
    }

    /// Check if two URLs are from the same domain
    fn is_same_domain(&self, url: &Url, base_url: &Url) -> bool {
        url.host_str() == base_url.host_str()
    }

    /// Extract plain text content from document
    fn extract_text_content(&self, document: &Html) -> String {
        let text_selector = Selector::parse("body")
            .expect("hardcoded CSS selector should be valid");
        document
            .select(&text_selector)
            .next()
            .map(|body| body.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_default()
    }

    /// Detect content type based on link and content patterns
    fn detect_content_type(&self, document: &Html, all_links: &[Url], nav_links: &[Url]) -> ContentType {
        // Check for product indicators
        if self.has_product_indicators(document) {
            return ContentType::Product;
        }

        // Check for article indicators
        if self.has_article_indicators(document) {
            return ContentType::Article;
        }

        // Check for category/listing indicators
        if self.has_category_indicators(document, all_links) {
            return ContentType::Category;
        }

        // Check for form indicators
        if self.has_form_indicators(document) {
            return ContentType::Form;
        }

        // Check for navigation pages (high nav link ratio)
        let nav_ratio = if !all_links.is_empty() {
            nav_links.len() as f64 / all_links.len() as f64
        } else {
            0.0
        };

        if nav_ratio > 0.7 {
            return ContentType::Navigation;
        }

        ContentType::Unknown
    }

    /// Check for product page indicators
    fn has_product_indicators(&self, document: &Html) -> bool {
        let indicators = [
            ".product",
            ".item",
            ".price",
            ".add-to-cart",
            ".buy-now",
            "[itemtype*=\"Product\"]",
        ];

        indicators.iter().any(|selector| {
            Selector::parse(selector)
                .map(|sel| document.select(&sel).next().is_some())
                .unwrap_or(false)
        })
    }

    /// Check for article indicators
    fn has_article_indicators(&self, document: &Html) -> bool {
        let indicators = [
            "article",
            ".article",
            ".post",
            ".blog-post",
            "[itemtype*=\"Article\"]",
            "time[datetime]",
            ".author",
        ];

        indicators.iter().any(|selector| {
            Selector::parse(selector)
                .map(|sel| document.select(&sel).next().is_some())
                .unwrap_or(false)
        })
    }

    /// Check for category/listing indicators
    fn has_category_indicators(&self, document: &Html, all_links: &[Url]) -> bool {
        let has_listing_elements = [
            ".category",
            ".listing",
            ".products",
            ".items",
            ".grid",
            ".list",
        ].iter().any(|selector| {
            Selector::parse(selector)
                .map(|sel| document.select(&sel).next().is_some())
                .unwrap_or(false)
        });

        // Category pages typically have many similar links
        let has_many_links = all_links.len() > 20;

        has_listing_elements || has_many_links
    }

    /// Check for form indicators
    fn has_form_indicators(&self, document: &Html) -> bool {
        let form_selector = Selector::parse("form")
            .expect("hardcoded CSS selector should be valid");
        document.select(&form_selector).count() > 0
    }

    /// Extract navigation hints from the document
    fn extract_navigation_hints(&self, document: &Html) -> Result<Vec<NavigationHint>> {
        let mut hints = Vec::new();

        // Extract breadcrumbs
        if let Some(breadcrumb) = self.extract_breadcrumb(document) {
            hints.push(breadcrumb);
        }

        // Extract pagination
        if let Some(pagination) = self.extract_pagination(document) {
            hints.push(pagination);
        }

        // Extract site navigation
        if let Some(nav) = self.extract_site_navigation(document) {
            hints.push(nav);
        }

        Ok(hints)
    }

    /// Extract breadcrumb navigation
    fn extract_breadcrumb(&self, document: &Html) -> Option<NavigationHint> {
        let breadcrumb_selectors = [
            ".breadcrumb",
            ".breadcrumbs",
            "[itemtype*=\"BreadcrumbList\"]",
            "nav[aria-label*=\"breadcrumb\" i]",
        ];

        for selector_str in &breadcrumb_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(breadcrumb_elem) = document.select(&selector).next() {
                    let path: Vec<String> = breadcrumb_elem
                        .text()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !path.is_empty() {
                        return Some(NavigationHint::Breadcrumb { path });
                    }
                }
            }
        }

        None
    }

    /// Extract pagination information
    fn extract_pagination(&self, document: &Html) -> Option<NavigationHint> {
        let pagination_selectors = [
            ".pagination",
            ".paging",
            ".page-numbers",
            "[aria-label*=\"pagination\" i]",
        ];

        for selector_str in &pagination_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if document.select(&selector).next().is_some() {
                    // Try to extract current page and total
                    let current = self.extract_current_page(document).unwrap_or(1);
                    let total = self.extract_total_pages(document);

                    return Some(NavigationHint::Pagination { current, total });
                }
            }
        }

        None
    }

    /// Extract site navigation
    fn extract_site_navigation(&self, document: &Html) -> Option<NavigationHint> {
        let nav_selectors = ["nav", ".navigation", ".menu"];

        for selector_str in &nav_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(nav_elem) = document.select(&selector).next() {
                    let link_selector = Selector::parse("a")
                        .expect("hardcoded CSS selector should be valid");
                    let items: Vec<String> = nav_elem
                        .select(&link_selector)
                        .filter_map(|a| a.text().next())
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !items.is_empty() {
                        return Some(NavigationHint::SiteNavigation { items });
                    }
                }
            }
        }

        None
    }

    /// Extract current page number from pagination
    fn extract_current_page(&self, document: &Html) -> Option<u32> {
        let current_selectors = [
            ".current",
            ".active",
            "[aria-current=\"page\"]",
        ];

        for selector_str in &current_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(elem) = document.select(&selector).next() {
                    if let Some(text) = elem.text().next() {
                        if let Ok(page) = text.trim().parse::<u32>() {
                            return Some(page);
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract total pages from pagination
    fn extract_total_pages(&self, document: &Html) -> Option<u32> {
        // Look for "Page X of Y" patterns
        let text = document.html();
        for line in text.lines() {
            if let Some(captures) = regex::Regex::new(r"(?i)page\s+\d+\s+of\s+(\d+)")
                .ok()
                .and_then(|re| re.captures(line))
            {
                if let Some(total_str) = captures.get(1) {
                    if let Ok(total) = total_str.as_str().parse::<u32>() {
                        return Some(total);
                    }
                }
            }
        }

        None
    }

    /// Extract freshness indicators
    fn extract_freshness_indicators(&self, document: &Html) -> Vec<String> {
        let mut indicators = Vec::new();

        // Look for date/time elements
        let time_selector = Selector::parse("time")
            .expect("hardcoded CSS selector should be valid");
        for time_elem in document.select(&time_selector) {
            if let Some(datetime) = time_elem.value().attr("datetime") {
                indicators.push(format!("datetime:{}", datetime));
            }
        }

        // Look for common date patterns in text
        let date_classes = [".date", ".published", ".updated", ".timestamp"];
        for class in &date_classes {
            if let Ok(selector) = Selector::parse(class) {
                for elem in document.select(&selector) {
                    if let Some(text) = elem.text().next() {
                        indicators.push(format!("{}:{}", class, text.trim()));
                    }
                }
            }
        }

        indicators
    }

    /// Count unique characters in text
    fn count_unique_chars(&self, text: &str) -> usize {
        let mut chars: Vec<char> = text.chars().collect();
        chars.sort_unstable();
        chars.dedup();
        chars.len()
    }

    /// Calculate content quality score
    fn calculate_quality_score(
        &self,
        all_links: &[Url],
        nav_links: &[Url],
        text_length: usize,
        unique_chars: usize,
    ) -> f64 {
        let mut score = 0.0;

        // Text quality (0-0.4)
        if text_length > 0 {
            let char_diversity = unique_chars as f64 / text_length as f64;
            score += (char_diversity * 0.4).min(0.4);
        }

        // Link quality (0-0.3)
        if !all_links.is_empty() {
            let nav_ratio = nav_links.len() as f64 / all_links.len() as f64;
            let content_link_ratio = 1.0 - nav_ratio;
            score += (content_link_ratio * 0.3).min(0.3);
        }

        // Content length bonus (0-0.3)
        let length_score = if text_length > 2000 {
            0.3
        } else if text_length > 500 {
            (text_length as f64 / 2000.0) * 0.3
        } else {
            0.0
        };
        score += length_score;

        score.min(1.0)
    }
}