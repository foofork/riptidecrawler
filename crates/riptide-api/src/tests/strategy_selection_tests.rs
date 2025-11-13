//! Tests for strategy selection mechanism and raw HTML access
//!
//! These tests verify that:
//! 1. Strategy parsing works correctly for all supported strategies
//! 2. Raw HTML is included only when requested
//! 3. Backward compatibility is maintained
//! 4. Feature guards work correctly for WASM

use crate::handlers::extract::*;
use riptide_types::{ExtractOptions, ExtractRequest};

#[test]
fn test_parse_strategy_native() {
    let strategy = parse_extraction_strategy("native");
    assert!(strategy.is_some());
    assert_eq!(
        strategy.unwrap(),
        riptide_facade::facades::ExtractionStrategy::HtmlCss
    );
}

#[test]
fn test_parse_strategy_css() {
    let strategy = parse_extraction_strategy("css");
    assert!(strategy.is_some());
    assert_eq!(
        strategy.unwrap(),
        riptide_facade::facades::ExtractionStrategy::HtmlCss
    );
}

#[test]
fn test_parse_strategy_wasm() {
    let strategy = parse_extraction_strategy("wasm");
    assert!(strategy.is_some());
    assert_eq!(
        strategy.unwrap(),
        riptide_facade::facades::ExtractionStrategy::Wasm
    );
}

#[test]
fn test_parse_strategy_auto() {
    let strategy = parse_extraction_strategy("auto");
    assert!(
        strategy.is_none(),
        "auto should return None to use UnifiedExtractor"
    );
}

#[test]
fn test_parse_strategy_multi() {
    let strategy = parse_extraction_strategy("multi");
    assert!(
        strategy.is_none(),
        "multi should return None to use UnifiedExtractor"
    );
}

#[test]
fn test_parse_strategy_markdown() {
    let strategy = parse_extraction_strategy("markdown");
    assert!(
        strategy.is_none(),
        "markdown is handled via as_markdown flag"
    );
}

#[test]
fn test_parse_strategy_unknown() {
    let strategy = parse_extraction_strategy("unknown-strategy");
    assert!(
        strategy.is_none(),
        "unknown strategies should default to None"
    );
}

#[test]
fn test_parse_strategy_case_insensitive() {
    let strategy1 = parse_extraction_strategy("NATIVE");
    let strategy2 = parse_extraction_strategy("Native");
    let strategy3 = parse_extraction_strategy("native");

    assert!(strategy1.is_some());
    assert!(strategy2.is_some());
    assert!(strategy3.is_some());
    assert_eq!(strategy1, strategy2);
    assert_eq!(strategy2, strategy3);
}

#[test]
fn test_extract_request_with_include_html_true() {
    let json = r#"{
        "url": "https://example.com",
        "options": {
            "include_html": true
        }
    }"#;
    let req: ExtractRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.options.include_html, true);
}

#[test]
fn test_extract_request_with_include_html_false() {
    let json = r#"{
        "url": "https://example.com",
        "options": {
            "include_html": false
        }
    }"#;
    let req: ExtractRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.options.include_html, false);
}

#[test]
fn test_extract_request_include_html_defaults_to_false() {
    let json = r#"{
        "url": "https://example.com"
    }"#;
    let req: ExtractRequest = serde_json::from_str(json).unwrap();
    assert_eq!(
        req.options.include_html, false,
        "include_html should default to false"
    );
}

#[test]
fn test_extract_request_with_strategy_and_html() {
    let json = r#"{
        "url": "https://example.com",
        "options": {
            "strategy": "css",
            "include_html": true,
            "quality_threshold": 0.8
        }
    }"#;
    let req: ExtractRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.options.strategy, "css");
    assert_eq!(req.options.include_html, true);
    assert_eq!(req.options.quality_threshold, 0.8);
}

#[test]
fn test_default_extract_options() {
    let options = ExtractOptions::default();
    assert_eq!(options.strategy, "multi");
    assert_eq!(options.quality_threshold, 0.7);
    assert_eq!(options.timeout_ms, 30000);
    assert_eq!(options.include_html, false);
}

#[test]
fn test_backward_compatibility_without_include_html() {
    // Old requests without include_html field should still work
    let json = r#"{
        "url": "https://example.com",
        "mode": "article",
        "options": {
            "strategy": "multi",
            "quality_threshold": 0.7
        }
    }"#;
    let req: ExtractRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.options.include_html, false);
}

#[test]
fn test_all_supported_strategies() {
    let strategies = vec![
        ("native", true),
        ("css", true),
        ("wasm", true),
        ("auto", false),
        ("multi", false),
        ("markdown", false),
    ];

    for (strategy_str, should_return_some) in strategies {
        let result = parse_extraction_strategy(strategy_str);
        if should_return_some {
            assert!(
                result.is_some(),
                "Strategy '{}' should return Some",
                strategy_str
            );
        } else {
            assert!(
                result.is_none(),
                "Strategy '{}' should return None",
                strategy_str
            );
        }
    }
}
