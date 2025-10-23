//! Integration Tests
//!
//! End-to-end reliability tests covering gate decisions, circuit breakers,
//! and reliability patterns working together.

use riptide_reliability::gate::{decide, score, Decision, GateFeatures};
use riptide_reliability::reliability::ReliabilityConfig;
use std::time::Duration;

#[test]
fn test_gate_decision_high_quality_content() {
    let features = GateFeatures {
        html_bytes: 50000,
        visible_text_chars: 10000,
        p_count: 20,
        article_count: 1,
        has_og: true,
        script_bytes: 1000,
        spa_markers: 0,
        h1h2_count: 3,
        has_jsonld_article: true,
        domain_prior: 0.8,
    };

    let score = score(&features);
    assert!(score > 0.7, "High quality content should have high score");

    let decision = decide(&features, 0.7, 0.3);
    assert_eq!(
        decision,
        Decision::Raw,
        "High quality content should use raw extraction"
    );
}

#[test]
fn test_gate_decision_spa_content() {
    let features = GateFeatures {
        html_bytes: 10000,
        visible_text_chars: 500,
        p_count: 2,
        article_count: 0,
        has_og: false,
        script_bytes: 8000,
        spa_markers: 3, // Multiple SPA markers
        h1h2_count: 0,
        has_jsonld_article: false,
        domain_prior: 0.5,
    };

    let score = score(&features);
    assert!(score < 0.4, "SPA content should have low score");

    let decision = decide(&features, 0.7, 0.3);
    assert_eq!(
        decision,
        Decision::Headless,
        "SPA content should use headless"
    );
}

#[test]
fn test_gate_decision_mixed_content() {
    let features = GateFeatures {
        html_bytes: 30000,
        visible_text_chars: 5000,
        p_count: 10,
        article_count: 1,
        has_og: true,
        script_bytes: 3000,
        spa_markers: 1, // Some JS, but not full SPA
        h1h2_count: 2,
        has_jsonld_article: false,
        domain_prior: 0.6,
    };

    let score = score(&features);
    assert!(
        score > 0.3 && score < 0.7,
        "Mixed content should have moderate score"
    );

    let decision = decide(&features, 0.7, 0.3);
    assert_eq!(
        decision,
        Decision::ProbesFirst,
        "Mixed content should probe first"
    );
}

#[test]
fn test_gate_score_calculation() {
    // Test empty content
    let empty = GateFeatures {
        html_bytes: 0,
        visible_text_chars: 0,
        p_count: 0,
        article_count: 0,
        has_og: false,
        script_bytes: 0,
        spa_markers: 0,
        h1h2_count: 0,
        has_jsonld_article: false,
        domain_prior: 0.5,
    };
    let empty_score = score(&empty);
    assert!(
        (0.0..=1.0).contains(&empty_score),
        "Score should be normalized"
    );

    // Test maximum quality
    let perfect = GateFeatures {
        html_bytes: 10000,
        visible_text_chars: 8000,
        p_count: 50,
        article_count: 1,
        has_og: true,
        script_bytes: 100,
        spa_markers: 0,
        h1h2_count: 10,
        has_jsonld_article: true,
        domain_prior: 1.0,
    };
    let perfect_score = score(&perfect);
    assert!(
        perfect_score > 0.8,
        "Perfect content should have very high score"
    );
}

#[test]
fn test_gate_decision_for_different_content_types() {
    // Test high quality content prefers fast extraction
    let high_quality = GateFeatures {
        html_bytes: 50000,
        visible_text_chars: 10000,
        p_count: 20,
        article_count: 1,
        has_og: true,
        script_bytes: 1000,
        spa_markers: 0,
        h1h2_count: 3,
        has_jsonld_article: true,
        domain_prior: 0.8,
    };
    let decision = decide(&high_quality, 0.7, 0.3);
    assert_eq!(
        decision,
        Decision::Raw,
        "High quality should use fast extraction"
    );

    // Test SPA content needs headless
    let spa = GateFeatures {
        html_bytes: 10000,
        visible_text_chars: 500,
        p_count: 2,
        article_count: 0,
        has_og: false,
        script_bytes: 8000,
        spa_markers: 3,
        h1h2_count: 0,
        has_jsonld_article: false,
        domain_prior: 0.5,
    };
    let decision = decide(&spa, 0.7, 0.3);
    assert_eq!(decision, Decision::Headless, "SPA should need headless");
}

#[test]
fn test_reliability_config_defaults() {
    let config = ReliabilityConfig::default();

    assert_eq!(config.http_retry.max_attempts, 2, "Should have 1 retry");
    assert!(
        config.enable_graceful_degradation,
        "Graceful degradation should be enabled"
    );
    assert_eq!(
        config.headless_timeout,
        Duration::from_secs(3),
        "Headless timeout should be 3s"
    );
    assert!(
        config.fast_extraction_quality_threshold > 0.5,
        "Quality threshold should be reasonable"
    );
}

#[test]
fn test_reliability_config_from_env() {
    // Set environment variables
    std::env::set_var("RELIABILITY_MAX_RETRIES", "5");
    std::env::set_var("RELIABILITY_TIMEOUT_SECS", "15");
    std::env::set_var("RELIABILITY_GRACEFUL_DEGRADATION", "false");
    std::env::set_var("RELIABILITY_QUALITY_THRESHOLD", "0.75");

    let config = ReliabilityConfig::from_env();

    assert_eq!(config.http_retry.max_attempts, 5);
    assert_eq!(config.headless_timeout, Duration::from_secs(15));
    assert!(!config.enable_graceful_degradation);
    assert_eq!(config.fast_extraction_quality_threshold, 0.75);

    // Clean up
    std::env::remove_var("RELIABILITY_MAX_RETRIES");
    std::env::remove_var("RELIABILITY_TIMEOUT_SECS");
    std::env::remove_var("RELIABILITY_GRACEFUL_DEGRADATION");
    std::env::remove_var("RELIABILITY_QUALITY_THRESHOLD");
}

#[test]
fn test_reliability_config_retry_settings() {
    let config = ReliabilityConfig::default();

    // Test HTTP retry configuration
    assert!(
        config.http_retry.initial_delay > Duration::from_millis(0),
        "Should have positive initial delay"
    );
    assert!(
        config.http_retry.max_delay > config.http_retry.initial_delay,
        "Max delay should be greater than initial delay"
    );
    assert!(
        config.http_retry.backoff_multiplier > 1.0,
        "Backoff multiplier should be > 1.0"
    );
    assert!(config.http_retry.jitter, "Jitter should be enabled");

    // Test circuit breaker configuration
    assert!(
        config.headless_circuit_breaker.failure_threshold > 0,
        "Should have failure threshold"
    );
    assert!(
        config.headless_circuit_breaker.open_cooldown_ms > 0,
        "Should have cooldown period"
    );
}

#[test]
fn test_gate_decision_thresholds() {
    let features = GateFeatures {
        html_bytes: 30000,
        visible_text_chars: 5000,
        p_count: 10,
        article_count: 1,
        has_og: true,
        script_bytes: 3000,
        spa_markers: 1,
        h1h2_count: 2,
        has_jsonld_article: false,
        domain_prior: 0.6,
    };

    // Test with different thresholds
    let _score = score(&features);

    // High threshold for raw should be more selective
    let decision_high = decide(&features, 0.9, 0.3);
    assert_ne!(
        decision_high,
        Decision::Raw,
        "High threshold should not choose raw for moderate content"
    );

    // Low threshold for headless should avoid headless more often
    let decision_low = decide(&features, 0.7, 0.1);
    assert_ne!(
        decision_low,
        Decision::Headless,
        "Low headless threshold should avoid headless for moderate content"
    );
}

#[test]
fn test_domain_prior_influence() {
    let base_features = GateFeatures {
        html_bytes: 30000,
        visible_text_chars: 5000,
        p_count: 10,
        article_count: 1,
        has_og: false,
        script_bytes: 3000,
        spa_markers: 1,
        h1h2_count: 2,
        has_jsonld_article: false,
        domain_prior: 0.3, // Low prior
    };

    let high_prior_features = GateFeatures {
        domain_prior: 0.9, // High prior
        ..base_features
    };

    let score_low = score(&base_features);
    let score_high = score(&high_prior_features);

    assert!(
        score_high > score_low,
        "High domain prior should improve score"
    );
}

#[test]
fn test_complex_scenario_with_multiple_factors() {
    // Scenario: News article with some JavaScript but good structure
    let news_article = GateFeatures {
        html_bytes: 75000,
        visible_text_chars: 15000, // Good text ratio
        p_count: 30,               // Many paragraphs
        article_count: 1,          // Article tag present
        has_og: true,              // Good metadata
        script_bytes: 5000,        // Some JS but not dominant
        spa_markers: 0,            // Not an SPA
        h1h2_count: 5,             // Good heading structure
        has_jsonld_article: true,  // Structured data
        domain_prior: 0.75,        // Known good domain
    };

    let score = score(&news_article);
    assert!(score > 0.65, "News article should have good score");

    let decision = decide(&news_article, 0.7, 0.3);
    assert!(
        decision == Decision::Raw || decision == Decision::ProbesFirst,
        "News article should prefer fast extraction"
    );
}

#[test]
fn test_script_heavy_content() {
    let script_heavy = GateFeatures {
        html_bytes: 20000,
        visible_text_chars: 1000, // Low text ratio
        p_count: 3,
        article_count: 0,
        has_og: false,
        script_bytes: 18000, // 90% JavaScript
        spa_markers: 2,
        h1h2_count: 0,
        has_jsonld_article: false,
        domain_prior: 0.5,
    };

    let score = score(&script_heavy);
    assert!(score < 0.3, "Script-heavy content should have low score");

    let decision = decide(&script_heavy, 0.7, 0.3);
    assert_eq!(
        decision,
        Decision::Headless,
        "Script-heavy content should use headless"
    );
}
