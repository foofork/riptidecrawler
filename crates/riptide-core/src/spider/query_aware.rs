use crate::spider::types::{CrawlRequest, CrawlResult};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use url::Url;

/// Configuration for query-aware spider functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAwareConfig {
    /// Enable query-aware foraging (opt-in)
    pub query_foraging: bool,
    /// Target query or keywords for relevance scoring
    pub target_query: Option<String>,
    /// Weight for BM25 scoring component (α)
    pub bm25_weight: f64,
    /// Weight for URL signals component (β)
    pub url_signals_weight: f64,
    /// Weight for domain diversity component (γ)
    pub domain_diversity_weight: f64,
    /// Weight for content similarity component (δ)
    pub content_similarity_weight: f64,
    /// Minimum relevance score threshold for early stopping
    pub min_relevance_threshold: f64,
    /// Window size for calculating relevance trends
    pub relevance_window_size: usize,
    /// BM25 parameters
    pub bm25_k1: f64,
    pub bm25_b: f64,
}

impl Default for QueryAwareConfig {
    fn default() -> Self {
        Self {
            query_foraging: false, // Opt-in
            target_query: None,
            bm25_weight: 0.4,      // α
            url_signals_weight: 0.2, // β
            domain_diversity_weight: 0.2, // γ
            content_similarity_weight: 0.2, // δ
            min_relevance_threshold: 0.3,
            relevance_window_size: 10,
            bm25_k1: 1.2,
            bm25_b: 0.75,
        }
    }
}

/// BM25 scoring implementation for text relevance
#[derive(Debug, Clone)]
pub struct BM25Scorer {
    /// Document frequency for each term
    term_doc_freq: HashMap<String, usize>,
    /// Total number of documents
    total_docs: usize,
    /// Average document length
    avg_doc_length: f64,
    /// BM25 parameters
    k1: f64,
    b: f64,
    /// Query terms
    query_terms: Vec<String>,
}

impl BM25Scorer {
    /// Create a new BM25 scorer with query terms
    pub fn new(query: &str, k1: f64, b: f64) -> Self {
        let query_terms = tokenize(query);
        Self {
            term_doc_freq: HashMap::new(),
            total_docs: 0,
            avg_doc_length: 0.0,
            k1,
            b,
            query_terms,
        }
    }

    /// Update corpus statistics with a new document
    pub fn update_corpus(&mut self, document: &str) {
        let doc_terms: HashSet<String> = tokenize(document).into_iter().collect();

        for term in doc_terms {
            *self.term_doc_freq.entry(term).or_insert(0) += 1;
        }

        self.total_docs += 1;

        // Update average document length
        let doc_length = tokenize(document).len() as f64;
        self.avg_doc_length = ((self.avg_doc_length * (self.total_docs - 1) as f64) + doc_length) / self.total_docs as f64;
    }

    /// Calculate BM25 score for a document
    pub fn score(&self, document: &str) -> f64 {
        if self.query_terms.is_empty() || self.total_docs == 0 {
            return 0.0;
        }

        let doc_terms = tokenize(document);
        let doc_length = doc_terms.len() as f64;

        // Count term frequencies in document
        let mut term_freq = HashMap::new();
        for term in doc_terms {
            *term_freq.entry(term).or_insert(0) += 1;
        }

        let mut score = 0.0;

        for query_term in &self.query_terms {
            let tf = *term_freq.get(query_term).unwrap_or(&0) as f64;
            let df = *self.term_doc_freq.get(query_term).unwrap_or(&0) as f64;

            if tf > 0.0 && df > 0.0 {
                // IDF calculation
                let idf = ((self.total_docs as f64 - df + 0.5) / (df + 0.5)).ln();

                // BM25 formula
                let numerator = tf * (self.k1 + 1.0);
                let denominator = tf + self.k1 * (1.0 - self.b + self.b * (doc_length / self.avg_doc_length));

                score += idf * (numerator / denominator);
            }
        }

        score
    }
}

/// URL signal analysis for path and depth relevance
#[derive(Debug, Clone)]
pub struct UrlSignalAnalyzer {
    /// Query terms for path analysis
    query_terms: Vec<String>,
}

impl UrlSignalAnalyzer {
    pub fn new(query: Option<&str>) -> Self {
        let query_terms = query.map(tokenize).unwrap_or_default();
        Self { query_terms }
    }

    /// Calculate URL signals score based on depth and path relevance
    pub fn score(&self, url: &Url, depth: usize) -> f64 {
        let depth_score = self.calculate_depth_score(depth);
        let path_score = self.calculate_path_relevance(url);

        // Combine depth and path scores (equal weight)
        (depth_score + path_score) / 2.0
    }

    /// Calculate depth score (lower depth = higher score)
    fn calculate_depth_score(&self, depth: usize) -> f64 {
        // Exponential decay with depth
        (-0.3 * depth as f64).exp()
    }

    /// Calculate path relevance based on query terms in URL path
    fn calculate_path_relevance(&self, url: &Url) -> f64 {
        if self.query_terms.is_empty() {
            return 0.5; // Neutral score when no query
        }

        let path = url.path().to_lowercase();
        let path_segments: Vec<&str> = path.split('/').collect();

        let mut relevance_score = 0.0;
        let mut total_terms = 0;

        for query_term in &self.query_terms {
            total_terms += 1;

            // Check if term appears in path
            if path.contains(query_term) {
                relevance_score += 1.0;

                // Bonus for term in domain/subdomain
                if let Some(host) = url.host_str() {
                    if host.to_lowercase().contains(query_term) {
                        relevance_score += 0.5;
                    }
                }

                // Bonus for term in early path segments
                for (i, segment) in path_segments.iter().enumerate() {
                    if segment.contains(query_term) {
                        let position_bonus = 1.0 / (i + 1) as f64;
                        relevance_score += position_bonus * 0.3;
                    }
                }
            }
        }

        if total_terms > 0 {
            relevance_score / total_terms as f64
        } else {
            0.5
        }
    }
}

/// Domain diversity scoring to encourage broad crawling
#[derive(Debug, Clone)]
pub struct DomainDiversityAnalyzer {
    /// Domains already crawled with their page counts
    domain_counts: HashMap<String, usize>,
    /// Total pages crawled
    total_pages: usize,
}

impl DomainDiversityAnalyzer {
    pub fn new() -> Self {
        Self {
            domain_counts: HashMap::new(),
            total_pages: 0,
        }
    }

    /// Record a crawled page from a domain
    pub fn record_page(&mut self, domain: &str) {
        *self.domain_counts.entry(domain.to_string()).or_insert(0) += 1;
        self.total_pages += 1;
    }

    /// Calculate diversity score for a new domain
    pub fn score(&self, domain: &str) -> f64 {
        let domain_count = *self.domain_counts.get(domain).unwrap_or(&0);

        if self.total_pages == 0 {
            return 1.0; // First page gets full score
        }

        // Calculate current domain's share of total pages
        let domain_share = domain_count as f64 / self.total_pages as f64;

        // Encourage diversity by giving higher scores to less-crawled domains
        // Use sigmoid function to smooth the penalty
        let diversity_score = 1.0 / (1.0 + (domain_share * 10.0).exp());

        // Bonus for completely new domains
        if domain_count == 0 {
            diversity_score + 0.2
        } else {
            diversity_score
        }
    }

    /// Get current domain statistics
    pub fn get_stats(&self) -> (usize, usize) {
        (self.domain_counts.len(), self.total_pages)
    }
}

/// Content similarity analyzer for query relevance
#[derive(Debug, Clone)]
pub struct ContentSimilarityAnalyzer {
    query_terms: Vec<String>,
}

impl ContentSimilarityAnalyzer {
    pub fn new(query: Option<&str>) -> Self {
        let query_terms = query.map(tokenize).unwrap_or_default();
        Self { query_terms }
    }

    /// Calculate content similarity score using simple term overlap
    pub fn score(&self, content: &str) -> f64 {
        if self.query_terms.is_empty() {
            return 0.5; // Neutral score when no query
        }

        let content_terms: HashSet<String> = tokenize(content).into_iter().collect();
        let query_terms_set: HashSet<String> = self.query_terms.iter().cloned().collect();

        // Calculate Jaccard similarity
        let intersection: HashSet<_> = content_terms.intersection(&query_terms_set).collect();
        let union: HashSet<_> = content_terms.union(&query_terms_set).collect();

        if union.is_empty() {
            0.0
        } else {
            intersection.len() as f64 / union.len() as f64
        }
    }
}

/// Main query-aware scoring engine
#[derive(Debug)]
pub struct QueryAwareScorer {
    config: QueryAwareConfig,
    bm25_scorer: BM25Scorer,
    url_analyzer: UrlSignalAnalyzer,
    domain_analyzer: DomainDiversityAnalyzer,
    content_analyzer: ContentSimilarityAnalyzer,
    recent_scores: Vec<f64>,
}

impl QueryAwareScorer {
    /// Create a new query-aware scorer
    pub fn new(config: QueryAwareConfig) -> Self {
        let query = config.target_query.as_deref();

        let bm25_scorer = BM25Scorer::new(
            query.unwrap_or(""),
            config.bm25_k1,
            config.bm25_b,
        );

        let url_analyzer = UrlSignalAnalyzer::new(query);
        let domain_analyzer = DomainDiversityAnalyzer::new();
        let content_analyzer = ContentSimilarityAnalyzer::new(query);

        Self {
            config,
            bm25_scorer,
            url_analyzer,
            domain_analyzer,
            content_analyzer,
            recent_scores: Vec::new(),
        }
    }

    /// Update the scorer with a crawled result
    pub fn update_with_result(&mut self, result: &CrawlResult) {
        if let Some(content) = &result.text_content {
            // Update BM25 corpus
            self.bm25_scorer.update_corpus(content);
        }

        // Update domain diversity
        if let Some(domain) = result.request.url.host_str() {
            self.domain_analyzer.record_page(domain);
        }
    }

    /// Calculate comprehensive relevance score for a crawl request
    pub fn score_request(&mut self, request: &CrawlRequest, content: Option<&str>) -> f64 {
        if !self.config.query_foraging {
            return 1.0; // Return neutral score when feature is disabled
        }

        let mut total_score = 0.0;

        // BM25 score (α component)
        if let Some(content) = content {
            let bm25_score = self.bm25_scorer.score(content);
            total_score += self.config.bm25_weight * bm25_score;
        }

        // URL signals score (β component)
        let url_score = self.url_analyzer.score(&request.url, request.depth);
        total_score += self.config.url_signals_weight * url_score;

        // Domain diversity score (γ component)
        let diversity_score = if let Some(domain) = request.url.host_str() {
            self.domain_analyzer.score(domain)
        } else {
            0.5
        };
        total_score += self.config.domain_diversity_weight * diversity_score;

        // Content similarity score (δ component)
        if let Some(content) = content {
            let similarity_score = self.content_analyzer.score(content);
            total_score += self.config.content_similarity_weight * similarity_score;
        }

        // Store recent score for trend analysis
        self.recent_scores.push(total_score);
        if self.recent_scores.len() > self.config.relevance_window_size {
            self.recent_scores.remove(0);
        }

        total_score
    }

    /// Check if crawling should stop due to low relevance
    pub fn should_stop_early(&self) -> (bool, String) {
        if !self.config.query_foraging || self.recent_scores.len() < self.config.relevance_window_size {
            return (false, String::new());
        }

        let avg_score = self.recent_scores.iter().sum::<f64>() / self.recent_scores.len() as f64;

        if avg_score < self.config.min_relevance_threshold {
            let reason = format!(
                "Low relevance detected: average score {:.3} below threshold {:.3}",
                avg_score, self.config.min_relevance_threshold
            );
            (true, reason)
        } else {
            (false, String::new())
        }
    }

    /// Get current scoring statistics
    pub fn get_stats(&self) -> QueryAwareStats {
        let (unique_domains, total_pages) = self.domain_analyzer.get_stats();
        let avg_recent_score = if !self.recent_scores.is_empty() {
            self.recent_scores.iter().sum::<f64>() / self.recent_scores.len() as f64
        } else {
            0.0
        };

        QueryAwareStats {
            enabled: self.config.query_foraging,
            unique_domains,
            total_pages,
            avg_recent_relevance: avg_recent_score,
            corpus_size: self.bm25_scorer.total_docs,
            recent_scores: self.recent_scores.clone(),
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: QueryAwareConfig) {
        self.config = config;
    }
}

/// Statistics for query-aware crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAwareStats {
    pub enabled: bool,
    pub unique_domains: usize,
    pub total_pages: usize,
    pub avg_recent_relevance: f64,
    pub corpus_size: usize,
    pub recent_scores: Vec<f64>,
}

/// Simple tokenization function
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|word| word.len() > 2) // Filter out very short words
        .map(|word| {
            // Remove punctuation
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_tokenize() {
        let text = "Hello, World! This is a test.";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["hello", "world", "this", "test"]);
    }

    #[test]
    fn test_bm25_scorer_creation() {
        let scorer = BM25Scorer::new("machine learning", 1.2, 0.75);
        assert_eq!(scorer.query_terms, vec!["machine", "learning"]);
        assert_eq!(scorer.k1, 1.2);
        assert_eq!(scorer.b, 0.75);
    }

    #[test]
    fn test_bm25_scoring() {
        let mut scorer = BM25Scorer::new("machine learning", 1.2, 0.75);

        // Add some documents to corpus
        scorer.update_corpus("Machine learning is a subset of artificial intelligence");
        scorer.update_corpus("Deep learning is a type of machine learning");
        scorer.update_corpus("Natural language processing uses machine learning");

        // Score a relevant document
        let score1 = scorer.score("Machine learning algorithms are powerful");
        let score2 = scorer.score("This document has no relevant content");

        assert!(score1 > score2);
        assert!(score1 > 0.0);
    }

    #[test]
    fn test_url_signal_analyzer() {
        let analyzer = UrlSignalAnalyzer::new(Some("machine learning"));

        let url1 = Url::from_str("https://example.com/machine-learning/tutorial").unwrap();
        let url2 = Url::from_str("https://example.com/random/page").unwrap();

        let score1 = analyzer.score(&url1, 1);
        let score2 = analyzer.score(&url2, 1);

        assert!(score1 > score2);
    }

    #[test]
    fn test_depth_scoring() {
        let analyzer = UrlSignalAnalyzer::new(None);
        let url = Url::from_str("https://example.com/page").unwrap();

        let shallow_score = analyzer.calculate_depth_score(1);
        let deep_score = analyzer.calculate_depth_score(5);

        assert!(shallow_score > deep_score);
    }

    #[test]
    fn test_domain_diversity_analyzer() {
        let mut analyzer = DomainDiversityAnalyzer::new();

        // First page from new domain should get high score
        let score1 = analyzer.score("example.com");
        assert!(score1 > 0.8);

        // Record some pages
        analyzer.record_page("example.com");
        analyzer.record_page("example.com");
        analyzer.record_page("other.com");

        // Same domain should get lower score now
        let score2 = analyzer.score("example.com");
        // New domain should get higher score
        let score3 = analyzer.score("newdomain.com");

        assert!(score3 > score2);
    }

    #[test]
    fn test_content_similarity_analyzer() {
        let analyzer = ContentSimilarityAnalyzer::new(Some("machine learning"));

        let relevant_content = "This article discusses machine learning algorithms and their applications";
        let irrelevant_content = "This is about cooking recipes and food preparation";

        let score1 = analyzer.score(relevant_content);
        let score2 = analyzer.score(irrelevant_content);

        assert!(score1 > score2);
    }

    #[test]
    fn test_query_aware_scorer_creation() {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            ..Default::default()
        };

        let scorer = QueryAwareScorer::new(config);
        assert!(scorer.config.query_foraging);
    }

    #[test]
    fn test_query_aware_scoring_disabled() {
        let config = QueryAwareConfig {
            query_foraging: false,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);
        let url = Url::from_str("https://example.com/test").unwrap();
        let request = CrawlRequest::new(url);

        let score = scorer.score_request(&request, Some("test content"));
        assert_eq!(score, 1.0); // Should return neutral score when disabled
    }

    #[test]
    fn test_early_stopping() {
        let config = QueryAwareConfig {
            query_foraging: true,
            min_relevance_threshold: 0.5,
            relevance_window_size: 3,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Add low scores
        scorer.recent_scores = vec![0.2, 0.3, 0.1];

        let (should_stop, reason) = scorer.should_stop_early();
        assert!(should_stop);
        assert!(!reason.is_empty());
    }

    #[test]
    fn test_comprehensive_scoring() {
        let config = QueryAwareConfig {
            query_foraging: true,
            target_query: Some("machine learning".to_string()),
            bm25_weight: 0.4,
            url_signals_weight: 0.2,
            domain_diversity_weight: 0.2,
            content_similarity_weight: 0.2,
            ..Default::default()
        };

        let mut scorer = QueryAwareScorer::new(config);

        // Create a request that should score well
        let url = Url::from_str("https://ml.example.com/machine-learning/intro").unwrap();
        let request = CrawlRequest::new(url).with_depth(1);
        let content = "This introduction to machine learning covers basic algorithms";

        let score = scorer.score_request(&request, Some(content));

        // Should get a reasonable score for relevant content
        assert!(score > 0.0);
        assert!(score <= 4.0); // Maximum possible score given weights
    }

    #[test]
    fn test_weight_validation() {
        let config = QueryAwareConfig::default();
        let total_weight = config.bm25_weight + config.url_signals_weight +
                          config.domain_diversity_weight + config.content_similarity_weight;

        // Weights should sum to 1.0 for proper normalization
        assert!((total_weight - 1.0).abs() < 0.001);
    }
}