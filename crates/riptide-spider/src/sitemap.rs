use crate::types::{CrawlRequest, Priority, SitemapConfig};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, info, warn};
use url::Url;
use xml::reader::{EventReader, XmlEvent};

/// Entry from a sitemap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapEntry {
    pub url: Url,
    pub last_modified: Option<DateTime<Utc>>,
    pub change_frequency: Option<String>,
    pub priority: Option<f64>,
}

/// Sitemap parser for discovering URLs
#[derive(Debug)]
pub struct SitemapParser {
    config: SitemapConfig,
    client: Client,
    cache: HashSet<String>,
}

impl SitemapParser {
    pub fn new(config: SitemapConfig) -> Self {
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config,
            client,
            cache: HashSet::new(),
        }
    }

    /// Parse sitemap from URL and return entries
    pub async fn parse_sitemap(&self, sitemap_url: &str) -> Result<Vec<SitemapEntry>> {
        debug!("Parsing sitemap: {}", sitemap_url);

        let response = self
            .client
            .get(sitemap_url)
            .send()
            .await
            .context("Failed to fetch sitemap")?;

        let content = response
            .text()
            .await
            .context("Failed to read sitemap content")?;

        self.parse_sitemap_content(&content).await
    }

    /// Parse sitemap content from string
    pub async fn parse_sitemap_content(&self, content: &str) -> Result<Vec<SitemapEntry>> {
        let mut entries = Vec::new();
        let parser = EventReader::from_str(content);

        let mut current_url: Option<String> = None;
        let mut current_lastmod: Option<DateTime<Utc>> = None;
        let mut current_changefreq: Option<String> = None;
        let mut current_priority: Option<f64> = None;

        for event in parser {
            match event {
                Ok(XmlEvent::StartElement { name, .. }) => {
                    if name.local_name.as_str() == "url" {
                        // Reset current entry
                        current_url = None;
                        current_lastmod = None;
                        current_changefreq = None;
                        current_priority = None;
                    }
                }
                Ok(XmlEvent::Characters(data)) => {
                    // This is a simplified parser - in reality we'd need to track element context
                    if data.starts_with("http") {
                        current_url = Some(data);
                    }
                }
                Ok(XmlEvent::EndElement { name }) => {
                    if name.local_name == "url" {
                        if let Some(url_str) = &current_url {
                            if let Ok(url) = Url::parse(url_str) {
                                // Apply filters
                                if let Some(max_urls) = self.config.max_urls {
                                    if entries.len() >= max_urls {
                                        break;
                                    }
                                }

                                if let Some(priority) = current_priority {
                                    if priority < self.config.min_priority {
                                        continue;
                                    }
                                }

                                entries.push(SitemapEntry {
                                    url,
                                    last_modified: current_lastmod,
                                    change_frequency: current_changefreq.clone(),
                                    priority: current_priority,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        info!("Parsed {} entries from sitemap", entries.len());
        Ok(entries)
    }

    /// Discover sitemaps for a domain
    pub async fn discover_sitemaps(&self, base_url: &Url) -> Result<Vec<String>> {
        let mut sitemaps = Vec::new();

        // Add custom sitemaps
        for sitemap in &self.config.custom_sitemaps {
            sitemaps.push(sitemap.clone());
        }

        if self.config.enable_discovery {
            // Try common sitemap locations
            let common_locations = vec![
                format!("{}/sitemap.xml", base_url.origin().ascii_serialization()),
                format!(
                    "{}/sitemap_index.xml",
                    base_url.origin().ascii_serialization()
                ),
                format!("{}/sitemaps.xml", base_url.origin().ascii_serialization()),
            ];

            for location in common_locations {
                if self.check_sitemap_exists(&location).await {
                    sitemaps.push(location);
                    if !self.config.follow_sitemap_index {
                        break; // Only get the first one found
                    }
                }
            }

            // Check robots.txt for sitemap entries
            match self.parse_robots_txt_sitemaps(base_url).await {
                Ok(robots_sitemaps) => {
                    for sitemap_url in robots_sitemaps {
                        if !sitemaps.contains(&sitemap_url) {
                            sitemaps.push(sitemap_url);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to parse robots.txt for sitemaps: {}", e);
                }
            }
        }

        Ok(sitemaps)
    }

    /// Parse robots.txt for sitemap directives
    /// Returns sitemap URLs found in robots.txt per RFC 9309
    async fn parse_robots_txt_sitemaps(&self, base_url: &Url) -> Result<Vec<String>> {
        let robots_url = format!("{}/robots.txt", base_url.origin().ascii_serialization());
        debug!("Checking robots.txt at: {}", robots_url);

        let response = self
            .client
            .get(&robots_url)
            .send()
            .await
            .context("Failed to fetch robots.txt")?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let content = response
            .text()
            .await
            .context("Failed to read robots.txt content")?;

        let mut sitemaps = Vec::new();

        // Parse robots.txt for "Sitemap:" directives (case-insensitive per RFC 9309)
        for line in content.lines() {
            let trimmed = line.trim();

            // Check for sitemap directive (case-insensitive)
            if let Some(sitemap_value) = trimmed
                .strip_prefix("Sitemap:")
                .or_else(|| trimmed.strip_prefix("sitemap:"))
                .or_else(|| trimmed.strip_prefix("SITEMAP:"))
            {
                let sitemap_url = sitemap_value.trim();

                // Validate URL format
                if let Ok(parsed_url) = Url::parse(sitemap_url) {
                    if parsed_url.scheme() == "http" || parsed_url.scheme() == "https" {
                        debug!("Found sitemap in robots.txt: {}", sitemap_url);
                        sitemaps.push(sitemap_url.to_string());
                    }
                }
            }
        }

        info!("Found {} sitemap(s) in robots.txt", sitemaps.len());
        Ok(sitemaps)
    }

    /// Check if a sitemap URL exists
    async fn check_sitemap_exists(&self, url: &str) -> bool {
        match self.client.head(url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Parse all discovered sitemaps and return unique URLs
    pub async fn parse_all_sitemaps(&self, base_url: &Url) -> Result<Vec<SitemapEntry>> {
        let sitemap_urls = self.discover_sitemaps(base_url).await?;
        let mut all_entries = Vec::new();
        let mut seen_urls = HashSet::new();

        for sitemap_url in sitemap_urls {
            match self.parse_sitemap(&sitemap_url).await {
                Ok(entries) => {
                    for entry in entries {
                        let url_str = entry.url.to_string();
                        if seen_urls.insert(url_str) {
                            all_entries.push(entry);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse sitemap {}: {}", sitemap_url, e);
                }
            }
        }

        info!("Discovered {} unique URLs from sitemaps", all_entries.len());
        Ok(all_entries)
    }

    /// Discover sitemaps and parse them, returning all entries
    pub async fn discover_and_parse(&mut self, base_url: &Url) -> Result<Vec<SitemapEntry>> {
        debug!("Discovering and parsing sitemaps for: {}", base_url);

        let sitemap_urls = self.discover_sitemaps(base_url).await?;
        let mut all_entries = Vec::new();

        for sitemap_url in sitemap_urls {
            // Check cache to avoid re-parsing the same sitemap
            if self.cache.contains(&sitemap_url) {
                debug!("Skipping cached sitemap: {}", sitemap_url);
                continue;
            }

            match self.parse_sitemap(&sitemap_url).await {
                Ok(entries) => {
                    debug!(
                        "Parsed {} entries from sitemap: {}",
                        entries.len(),
                        sitemap_url
                    );
                    all_entries.extend(entries);
                    // Add to cache after successful parsing
                    self.cache.insert(sitemap_url);
                }
                Err(e) => {
                    warn!("Failed to parse sitemap {}: {}", sitemap_url, e);
                }
            }
        }

        // Remove duplicates based on URL
        let mut seen_urls = HashSet::new();
        all_entries.retain(|entry| {
            let url_str = entry.url.to_string();
            seen_urls.insert(url_str)
        });

        info!(
            "Discovered and parsed {} unique URLs from sitemaps",
            all_entries.len()
        );
        Ok(all_entries)
    }

    /// Convert sitemap entries to crawl requests with appropriate priority and metadata
    pub fn urls_to_crawl_requests(&self, entries: Vec<SitemapEntry>) -> Vec<CrawlRequest> {
        debug!(
            "Converting {} sitemap entries to crawl requests",
            entries.len()
        );

        entries
            .into_iter()
            .map(|entry| {
                // Determine priority based on sitemap priority
                let priority = match entry.priority {
                    Some(p) if p >= 0.8 => Priority::Critical,
                    Some(p) if p >= 0.6 => Priority::High,
                    Some(p) if p >= 0.3 => Priority::Medium,
                    _ => Priority::Low,
                };

                // Create crawl request with metadata from sitemap
                let mut request = CrawlRequest::new(entry.url.clone())
                    .with_priority(priority)
                    .with_depth(0); // Sitemap URLs start at depth 0

                // Add sitemap-specific metadata
                if let Some(last_mod) = entry.last_modified {
                    request =
                        request.with_metadata("sitemap_lastmod".to_string(), last_mod.to_rfc3339());
                }

                if let Some(change_freq) = entry.change_frequency {
                    request = request.with_metadata("sitemap_changefreq".to_string(), change_freq);
                }

                if let Some(priority_val) = entry.priority {
                    request = request
                        .with_metadata("sitemap_priority".to_string(), priority_val.to_string());
                }

                // Add score based on priority for best-first strategy
                if let Some(priority_val) = entry.priority {
                    request = request.with_score(priority_val);
                }

                request.with_metadata("source".to_string(), "sitemap".to_string())
            })
            .collect()
    }

    /// Clear the internal cache of processed sitemaps
    pub fn clear_cache(&mut self) {
        debug!("Clearing sitemap cache ({} entries)", self.cache.len());
        self.cache.clear();
    }
}
