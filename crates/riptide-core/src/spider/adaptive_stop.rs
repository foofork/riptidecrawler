use crate::spider::types::{ContentWindow, CrawlResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Configuration for adaptive stopping algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStopConfig {
    /// Size of sliding window for content analysis
    pub window_size: usize,
    /// Minimum unique text characters gain threshold
    pub min_gain_threshold: f64,
    /// Number of consecutive low-gain iterations before stopping
    pub patience: usize,
    /// Minimum pages to crawl before considering stopping
    pub min_pages_before_stop: usize,
    /// Enable content quality scoring
    pub enable_quality_scoring: bool,
    /// Threshold for content quality score
    pub quality_threshold: f64,
    /// Weight for text content in scoring
    pub text_content_weight: f64,
    /// Weight for link richness in scoring
    pub link_richness_weight: f64,
    /// Weight for content size in scoring
    pub content_size_weight: f64,
    /// Enable adaptive threshold adjustment
    pub enable_adaptive_threshold: bool,
    /// Site type hints for threshold adjustment
    pub site_type_hints: SiteTypeHints,
    /// Maximum time to analyze content
    pub max_analysis_time: Duration,
}

impl Default for AdaptiveStopConfig {
    fn default() -> Self {
        Self {
            window_size: 10,
            min_gain_threshold: 100.0, // Minimum 100 unique chars per page
            patience: 5,
            min_pages_before_stop: 20,
            enable_quality_scoring: true,
            quality_threshold: 0.5,
            text_content_weight: 0.6,
            link_richness_weight: 0.3,
            content_size_weight: 0.1,
            enable_adaptive_threshold: true,
            site_type_hints: SiteTypeHints::default(),
            max_analysis_time: Duration::from_millis(100),
        }
    }
}

/// Site type hints for threshold adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteTypeHints {
    /// News sites typically have higher content variation
    pub news_site_multiplier: f64,
    /// E-commerce sites may have repetitive content
    pub ecommerce_multiplier: f64,
    /// Blog sites typically have varied content
    pub blog_multiplier: f64,
    /// Documentation sites may have structured content
    pub documentation_multiplier: f64,
    /// Social media sites have high content variation
    pub social_media_multiplier: f64,
    /// Default multiplier for unknown sites
    pub default_multiplier: f64,
}

impl Default for SiteTypeHints {
    fn default() -> Self {
        Self {
            news_site_multiplier: 1.5,
            ecommerce_multiplier: 0.7,
            blog_multiplier: 1.2,
            documentation_multiplier: 0.9,
            social_media_multiplier: 1.8,
            default_multiplier: 1.0,
        }
    }
}

/// Detected site type based on content analysis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SiteType {
    News,
    ECommerce,
    Blog,
    Documentation,
    SocialMedia,
    Unknown,
}

impl SiteType {
    /// Get the threshold multiplier for this site type
    pub fn threshold_multiplier(&self, hints: &SiteTypeHints) -> f64 {
        match self {
            SiteType::News => hints.news_site_multiplier,
            SiteType::ECommerce => hints.ecommerce_multiplier,
            SiteType::Blog => hints.blog_multiplier,
            SiteType::Documentation => hints.documentation_multiplier,
            SiteType::SocialMedia => hints.social_media_multiplier,
            SiteType::Unknown => hints.default_multiplier,
        }
    }
}

/// Content analysis metrics
#[derive(Debug, Clone)]
pub struct ContentMetrics {
    /// Unique text characters in the content
    pub unique_text_chars: usize,
    /// Total content size
    pub content_size: usize,
    /// Number of extracted links
    pub link_count: usize,
    /// Content quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Processing time for analysis
    pub analysis_time: Duration,
}

impl ContentMetrics {
    /// Calculate content gain score
    pub fn gain_score(&self, weights: &AdaptiveStopConfig) -> f64 {
        let text_score = self.unique_text_chars as f64 * weights.text_content_weight;
        let link_score = self.link_count as f64 * weights.link_richness_weight;
        let size_score = (self.content_size as f64).ln().max(0.0_f64) * weights.content_size_weight;

        text_score + link_score + size_score
    }
}

/// Adaptive stop decision information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopDecision {
    /// Whether to stop crawling
    pub should_stop: bool,
    /// Reason for the decision
    pub reason: String,
    /// Current gain average
    pub current_gain_average: f64,
    /// Threshold used for decision
    pub threshold_used: f64,
    /// Consecutive low-gain count
    pub consecutive_low_gain: usize,
    /// Total pages analyzed
    pub pages_analyzed: usize,
    /// Site type detected
    pub detected_site_type: SiteType,
}

/// Adaptive stopping engine
pub struct AdaptiveStopEngine {
    config: AdaptiveStopConfig,

    // Content analysis window
    content_window: Arc<RwLock<ContentWindow>>,

    // Quality tracking
    quality_scores: Arc<RwLock<VecDeque<f64>>>,

    // Stop condition tracking
    consecutive_low_gain: Arc<RwLock<usize>>,
    pages_analyzed: Arc<RwLock<usize>>,

    // Site analysis
    detected_site_type: Arc<RwLock<SiteType>>,
    site_analysis_samples: Arc<RwLock<Vec<ContentMetrics>>>,

    // Performance tracking
    analysis_times: Arc<RwLock<VecDeque<Duration>>>,
}

impl AdaptiveStopEngine {
    pub fn new(config: AdaptiveStopConfig) -> Self {
        Self {
            content_window: Arc::new(RwLock::new(ContentWindow::new(config.window_size))),
            quality_scores: Arc::new(RwLock::new(VecDeque::new())),
            consecutive_low_gain: Arc::new(RwLock::new(0)),
            pages_analyzed: Arc::new(RwLock::new(0)),
            detected_site_type: Arc::new(RwLock::new(SiteType::Unknown)),
            site_analysis_samples: Arc::new(RwLock::new(Vec::new())),
            analysis_times: Arc::new(RwLock::new(VecDeque::new())),
            config,
        }
    }

    /// Analyze a crawl result and update internal state
    pub async fn analyze_result(&self, result: &CrawlResult) -> Result<ContentMetrics> {
        let start_time = Instant::now();

        if !result.success {
            // Don't analyze failed results
            return Ok(ContentMetrics {
                unique_text_chars: 0,
                content_size: 0,
                link_count: 0,
                quality_score: 0.0,
                analysis_time: start_time.elapsed(),
            });
        }

        // Calculate basic metrics
        let unique_text_chars = result.unique_text_chars();
        let content_size = result.content_size;
        let link_count = result.extracted_urls.len();

        // Calculate quality score
        let quality_score = if self.config.enable_quality_scoring {
            self.calculate_quality_score(result).await
        } else {
            1.0
        };

        let metrics = ContentMetrics {
            unique_text_chars,
            content_size,
            link_count,
            quality_score,
            analysis_time: start_time.elapsed(),
        };

        // Update content window
        {
            let mut window = self.content_window.write().await;
            window.add_measurement(unique_text_chars);
        }

        // Update quality tracking
        if self.config.enable_quality_scoring {
            let mut quality_scores = self.quality_scores.write().await;
            quality_scores.push_back(quality_score);
            if quality_scores.len() > self.config.window_size {
                quality_scores.pop_front();
            }
        }

        // Update site analysis
        {
            let mut samples = self.site_analysis_samples.write().await;
            samples.push(metrics.clone());
            if samples.len() > 50 {
                samples.remove(0); // Keep recent samples
            }

            // Update site type detection
            if samples.len() >= 10 {
                let new_site_type = self.detect_site_type(&samples);
                *self.detected_site_type.write().await = new_site_type;
            }
        }

        // Update pages analyzed
        {
            let mut pages = self.pages_analyzed.write().await;
            *pages += 1;
        }

        // Track analysis performance
        {
            let mut times = self.analysis_times.write().await;
            times.push_back(metrics.analysis_time);
            if times.len() > 100 {
                times.pop_front();
            }
        }

        debug!(
            unique_chars = unique_text_chars,
            content_size = content_size,
            link_count = link_count,
            quality_score = quality_score,
            analysis_time_ms = metrics.analysis_time.as_millis(),
            "Content analyzed"
        );

        Ok(metrics)
    }

    /// Check if crawling should stop based on adaptive criteria
    pub async fn should_stop(&self) -> Result<StopDecision> {
        let pages_analyzed = *self.pages_analyzed.read().await;

        // Don't stop if we haven't analyzed enough pages
        if pages_analyzed < self.config.min_pages_before_stop {
            return Ok(StopDecision {
                should_stop: false,
                reason: format!(
                    "Not enough pages analyzed ({} < {})",
                    pages_analyzed, self.config.min_pages_before_stop
                ),
                current_gain_average: f64::INFINITY,
                threshold_used: self.config.min_gain_threshold,
                consecutive_low_gain: 0,
                pages_analyzed,
                detected_site_type: *self.detected_site_type.read().await,
            });
        }

        // Get current gain average
        let window = self.content_window.read().await;
        if !window.has_sufficient_data() {
            return Ok(StopDecision {
                should_stop: false,
                reason: "Insufficient data in content window".to_string(),
                current_gain_average: f64::INFINITY,
                threshold_used: self.config.min_gain_threshold,
                consecutive_low_gain: 0,
                pages_analyzed,
                detected_site_type: *self.detected_site_type.read().await,
            });
        }

        let current_gain_average = window.average_gain();

        // Calculate adaptive threshold
        let threshold = if self.config.enable_adaptive_threshold {
            self.calculate_adaptive_threshold().await
        } else {
            self.config.min_gain_threshold
        };

        let consecutive_low_gain = *self.consecutive_low_gain.read().await;
        let detected_site_type = *self.detected_site_type.read().await;

        // Check if gain is below threshold
        if current_gain_average < threshold {
            let mut consecutive = self.consecutive_low_gain.write().await;
            *consecutive += 1;

            debug!(
                gain = current_gain_average,
                threshold = threshold,
                consecutive = *consecutive,
                patience = self.config.patience,
                "Low gain detected"
            );

            // Check if we've been patient enough
            if *consecutive >= self.config.patience {
                return Ok(StopDecision {
                    should_stop: true,
                    reason: format!(
                        "Low content gain for {} consecutive iterations (gain: {:.2} < threshold: {:.2})",
                        *consecutive, current_gain_average, threshold
                    ),
                    current_gain_average,
                    threshold_used: threshold,
                    consecutive_low_gain: *consecutive,
                    pages_analyzed,
                    detected_site_type,
                });
            }
        } else {
            // Reset consecutive counter on good gain
            *self.consecutive_low_gain.write().await = 0;
        }

        // Check quality threshold if enabled
        if self.config.enable_quality_scoring {
            let quality_scores = self.quality_scores.read().await;
            if quality_scores.len() >= 5 {
                let avg_quality: f64 = quality_scores.iter().sum::<f64>() / quality_scores.len() as f64;
                if avg_quality < self.config.quality_threshold {
                    return Ok(StopDecision {
                        should_stop: true,
                        reason: format!(
                            "Low content quality average: {:.3} < {:.3}",
                            avg_quality, self.config.quality_threshold
                        ),
                        current_gain_average,
                        threshold_used: threshold,
                        consecutive_low_gain,
                        pages_analyzed,
                        detected_site_type,
                    });
                }
            }
        }

        Ok(StopDecision {
            should_stop: false,
            reason: "Continue crawling".to_string(),
            current_gain_average,
            threshold_used: threshold,
            consecutive_low_gain,
            pages_analyzed,
            detected_site_type,
        })
    }

    /// Calculate quality score for content
    async fn calculate_quality_score(&self, result: &CrawlResult) -> f64 {
        let mut score = 0.0_f64;

        // Text content quality
        if let Some(text) = &result.text_content {
            let word_count = text.split_whitespace().count();
            let sentence_count = text.matches('.').count() + text.matches('!').count() + text.matches('?').count();

            // Basic readability heuristics
            if word_count > 100 {
                score += 0.3;
            }
            if sentence_count > 5 {
                score += 0.2;
            }
            if text.len() > 500 {
                score += 0.2;
            }

            // Check for structured content
            if text.contains("</") || text.contains("<!--") {
                score -= 0.1; // Penalize HTML artifacts
            }
        }

        // Link quality
        let internal_links = result.extracted_urls.iter()
            .filter(|url| {
                result.request.url.host_str() == url.host_str()
            })
            .count();

        if internal_links > 0 {
            score += 0.2;
        }
        if result.extracted_urls.len() > 5 {
            score += 0.1;
        }

        // Content size quality
        if result.content_size > 1024 {
            score += 0.2;
        }

        score.clamp(0.0_f64, 1.0_f64)
    }

    /// Calculate adaptive threshold based on site type and performance
    async fn calculate_adaptive_threshold(&self) -> f64 {
        let detected_site_type = *self.detected_site_type.read().await;
        let base_threshold = self.config.min_gain_threshold;

        // Adjust based on site type
        let site_multiplier = detected_site_type.threshold_multiplier(&self.config.site_type_hints);
        let adjusted_threshold = base_threshold * site_multiplier;

        // Adjust based on recent performance
        let samples = self.site_analysis_samples.read().await;
        if samples.len() >= 5 {
            let recent_samples = &samples[samples.len().saturating_sub(5)..];
            let avg_unique_chars: f64 = recent_samples.iter()
                .map(|s| s.unique_text_chars as f64)
                .sum::<f64>() / recent_samples.len() as f64;

            // If recent content is consistently low, lower threshold
            if avg_unique_chars < adjusted_threshold * 0.5 {
                return adjusted_threshold * 0.7;
            }
            // If recent content is high, maintain or raise threshold
            if avg_unique_chars > adjusted_threshold * 2.0 {
                return adjusted_threshold * 1.2;
            }
        }

        debug!(
            base_threshold = base_threshold,
            site_type = ?detected_site_type,
            site_multiplier = site_multiplier,
            final_threshold = adjusted_threshold,
            "Calculated adaptive threshold"
        );

        adjusted_threshold
    }

    /// Detect site type based on content patterns
    fn detect_site_type(&self, samples: &[ContentMetrics]) -> SiteType {
        if samples.is_empty() {
            return SiteType::Unknown;
        }

        let avg_unique_chars: f64 = samples.iter()
            .map(|s| s.unique_text_chars as f64)
            .sum::<f64>() / samples.len() as f64;

        let avg_links: f64 = samples.iter()
            .map(|s| s.link_count as f64)
            .sum::<f64>() / samples.len() as f64;

        let avg_quality: f64 = samples.iter()
            .map(|s| s.quality_score)
            .sum::<f64>() / samples.len() as f64;

        // Simple heuristic-based classification
        if avg_unique_chars > 5000.0 && avg_quality > 0.7 {
            if avg_links > 20.0 {
                SiteType::News
            } else {
                SiteType::Blog
            }
        } else if avg_links > 50.0 && avg_unique_chars > 2000.0 {
            SiteType::SocialMedia
        } else if avg_links < 5.0 && avg_unique_chars > 3000.0 {
            SiteType::Documentation
        } else if avg_links > 10.0 && avg_unique_chars < 2000.0 {
            SiteType::ECommerce
        } else {
            SiteType::Unknown
        }
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> AdaptiveStopStats {
        let window = self.content_window.read().await;
        let quality_scores = self.quality_scores.read().await;
        let analysis_times = self.analysis_times.read().await;

        let avg_analysis_time = if analysis_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = analysis_times.iter().sum();
            total / analysis_times.len() as u32
        };

        AdaptiveStopStats {
            pages_analyzed: *self.pages_analyzed.read().await,
            consecutive_low_gain: *self.consecutive_low_gain.read().await,
            current_gain_average: window.average_gain(),
            detected_site_type: *self.detected_site_type.read().await,
            avg_quality_score: if quality_scores.is_empty() {
                0.0
            } else {
                quality_scores.iter().sum::<f64>() / quality_scores.len() as f64
            },
            avg_analysis_time,
            window_full: window.full,
        }
    }

    /// Reset all tracking state
    pub async fn reset(&self) {
        *self.content_window.write().await = ContentWindow::new(self.config.window_size);
        self.quality_scores.write().await.clear();
        *self.consecutive_low_gain.write().await = 0;
        *self.pages_analyzed.write().await = 0;
        *self.detected_site_type.write().await = SiteType::Unknown;
        self.site_analysis_samples.write().await.clear();
        self.analysis_times.write().await.clear();

        info!("Adaptive stop engine reset");
    }

    /// Get configuration
    pub fn get_config(&self) -> &AdaptiveStopConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AdaptiveStopConfig) {
        self.config = config;
        info!("Adaptive stop configuration updated");
    }
}

/// Statistics for adaptive stopping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveStopStats {
    pub pages_analyzed: usize,
    pub consecutive_low_gain: usize,
    pub current_gain_average: f64,
    pub detected_site_type: SiteType,
    pub avg_quality_score: f64,
    pub avg_analysis_time: Duration,
    pub window_full: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spider::types::CrawlRequest;
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_basic_content_analysis() {
        let config = AdaptiveStopConfig::default();
        let engine = AdaptiveStopEngine::new(config);

        let url = Url::from_str("https://example.com/").expect("Valid URL");
        let request = CrawlRequest::new(url);

        let mut result = CrawlResult::success(request);
        result.text_content = Some("This is a test content with many unique characters and words.".to_string());
        result.content_size = 1024;
        result.extracted_urls = vec![
            Url::from_str("https://example.com/page1").expect("Valid URL"),
            Url::from_str("https://example.com/page2").expect("Valid URL"),
        ];

        let metrics = engine.analyze_result(&result).await.expect("Analysis should work");

        assert!(metrics.unique_text_chars > 0);
        assert_eq!(metrics.content_size, 1024);
        assert_eq!(metrics.link_count, 2);
        assert!(metrics.quality_score > 0.0);
    }

    #[tokio::test]
    async fn test_adaptive_stopping_decision() {
        let mut config = AdaptiveStopConfig::default();
        config.min_pages_before_stop = 5;
        config.patience = 3;
        config.min_gain_threshold = 10.0;

        let engine = AdaptiveStopEngine::new(config);

        // Analyze several low-quality results
        for i in 0..10 {
            let url = Url::from_str(&format!("https://example.com/page{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);

            let mut result = CrawlResult::success(request);
            result.text_content = Some("short".to_string()); // Very low content
            result.content_size = 100;

            engine.analyze_result(&result).await.expect("Analysis should work");
        }

        let decision = engine.should_stop().await.expect("Decision should work");
        assert!(decision.should_stop);
        assert!(decision.reason.contains("Low content gain"));
    }

    #[tokio::test]
    async fn test_site_type_detection() {
        let config = AdaptiveStopConfig::default();
        let engine = AdaptiveStopEngine::new(config);

        // Simulate news site content (high content, many links)
        for i in 0..15 {
            let url = Url::from_str(&format!("https://news.example.com/article{}", i)).expect("Valid URL");
            let request = CrawlRequest::new(url);

            let mut result = CrawlResult::success(request);
            result.text_content = Some("This is a comprehensive news article with lots of detailed information about current events and breaking news that provides in-depth analysis and commentary on the latest developments in politics, technology, and society.".to_string());
            result.content_size = 5000;
            result.extracted_urls = (0..25).map(|j| {
                Url::from_str(&format!("https://news.example.com/related{}", j)).expect("Valid URL")
            }).collect();

            engine.analyze_result(&result).await.expect("Analysis should work");
        }

        let stats = engine.get_stats().await;
        // Should detect as news site (high content + many links)
        assert!(matches!(stats.detected_site_type, SiteType::News | SiteType::Blog));
    }

    #[tokio::test]
    async fn test_quality_scoring() {
        let config = AdaptiveStopConfig::default();
        let engine = AdaptiveStopEngine::new(config);

        // High quality content
        let url = Url::from_str("https://example.com/").expect("Valid URL");
        let request = CrawlRequest::new(url);

        let mut high_quality_result = CrawlResult::success(request.clone());
        high_quality_result.text_content = Some(
            "This is a high-quality article with substantial content. It contains multiple sentences and paragraphs. The content is well-structured and informative. It provides valuable information to readers and maintains good readability throughout the entire document.".to_string()
        );
        high_quality_result.content_size = 2048;
        high_quality_result.extracted_urls = vec![
            Url::from_str("https://example.com/related1").expect("Valid URL"),
            Url::from_str("https://example.com/related2").expect("Valid URL"),
            Url::from_str("https://example.com/related3").expect("Valid URL"),
        ];

        let high_metrics = engine.analyze_result(&high_quality_result).await.expect("Should work");

        // Low quality content
        let mut low_quality_result = CrawlResult::success(request);
        low_quality_result.text_content = Some("short".to_string());
        low_quality_result.content_size = 50;

        let low_metrics = engine.analyze_result(&low_quality_result).await.expect("Should work");

        assert!(high_metrics.quality_score > low_metrics.quality_score);
    }

    #[tokio::test]
    async fn test_adaptive_threshold_calculation() {
        let mut config = AdaptiveStopConfig::default();
        config.enable_adaptive_threshold = true;

        let engine = AdaptiveStopEngine::new(config);

        // Set site type to news (should increase threshold)
        *engine.detected_site_type.write().await = SiteType::News;

        let threshold = engine.calculate_adaptive_threshold().await;
        assert!(threshold > engine.config.min_gain_threshold);
    }

    #[test]
    fn test_content_window() {
        let mut window = ContentWindow::new(3);

        assert!(!window.has_sufficient_data());

        window.add_measurement(100);
        window.add_measurement(150);
        window.add_measurement(200);

        assert!(window.has_sufficient_data());

        let gain = window.average_gain();
        assert!(gain > 0.0); // Should show positive gain

        // Add measurement that shows no gain
        window.add_measurement(200);
        let new_gain = window.average_gain();
        assert!(new_gain < gain); // Should show reduced gain
    }

    #[test]
    fn test_site_type_multipliers() {
        let hints = SiteTypeHints::default();

        assert_eq!(SiteType::News.threshold_multiplier(&hints), hints.news_site_multiplier);
        assert_eq!(SiteType::ECommerce.threshold_multiplier(&hints), hints.ecommerce_multiplier);
        assert_eq!(SiteType::Unknown.threshold_multiplier(&hints), hints.default_multiplier);
    }
}