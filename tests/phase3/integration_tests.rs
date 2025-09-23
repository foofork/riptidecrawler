/// Integration tests for Phase 3 features
///
/// These tests verify that dynamic content handling, stealth features, and PDF processing
/// work together correctly in end-to-end scenarios.

use riptide_core::dynamic::{DynamicConfig, WaitCondition, ScrollConfig, PageAction, DynamicRenderResult};
use riptide_core::stealth::{StealthConfig, StealthController};
use riptide_core::pdf::{PdfConfig, DefaultPdfProcessor, PdfProcessor};
use riptide_core::types::{RenderMode, OutputFormat, CrawlOptions};
use std::time::Duration;
use std::collections::HashMap;

use super::test_utils::{
    create_test_dynamic_config, create_test_stealth_config, create_test_pdf_config,
    create_test_pdf_data, create_test_html, create_test_headers,
};

#[tokio::test]
async fn test_dynamic_config_with_stealth_integration() {
    let dynamic_config = DynamicConfig {
        wait_for: Some(WaitCondition::Javascript {
            script: "window.dataLoaded === true".to_string(),
            timeout: Duration::from_secs(10),
        }),
        scroll: Some(ScrollConfig {
            steps: 3,
            step_px: Some(1000),
            delay_ms: 1500,
            mode: riptide_core::dynamic::ScrollMode::Stepped,
            after_scroll_js: Some("window.lazyLoadImages()".to_string()),
            stop_condition: None,
        }),
        actions: vec![
            PageAction::Wait(WaitCondition::DomContentLoaded),
            PageAction::Click {
                selector: ".accept-cookies".to_string(),
                wait_after: Some(Duration::from_millis(1000)),
            },
            PageAction::Type {
                selector: "input[name='search']".to_string(),
                text: "test query".to_string(),
                clear_first: true,
                wait_after: Some(Duration::from_millis(500)),
            },
            PageAction::Evaluate {
                script: "document.querySelector('.content').scrollIntoView()".to_string(),
                wait_after: Some(Duration::from_millis(2000)),
            },
        ],
        capture_artifacts: true,
        timeout: Duration::from_secs(30),
        viewport: Some(riptide_core::dynamic::ViewportConfig {
            width: 1920,
            height: 1080,
            device_scale_factor: 1.0,
            is_mobile: false,
            user_agent: None, // Will be overridden by stealth config
        }),
    };

    let stealth_config = StealthConfig::default();
    let mut stealth_controller = StealthController::new(stealth_config);

    // Test that stealth measures are applied
    let user_agent = stealth_controller.next_user_agent();
    assert!(!user_agent.is_empty());
    assert!(user_agent.contains("Mozilla"));

    let headers = stealth_controller.generate_headers();
    assert!(headers.contains_key("Accept"));
    assert!(headers.contains_key("Accept-Language"));

    let delay = stealth_controller.calculate_delay();
    assert!(delay.as_millis() >= 500);
    assert!(delay.as_millis() <= 3000);

    // Test that dynamic config is properly structured
    assert!(dynamic_config.capture_artifacts);
    assert_eq!(dynamic_config.actions.len(), 4);
    assert!(dynamic_config.wait_for.is_some());
    assert!(dynamic_config.scroll.is_some());
}

#[tokio::test]
async fn test_pdf_processing_with_configuration() {
    let pdf_config = PdfConfig {
        max_size_bytes: 10 * 1024 * 1024, // 10MB
        extract_text: true,
        extract_images: false, // Disable for this test
        extract_metadata: true,
        image_settings: Default::default(),
        text_settings: riptide_core::pdf::TextExtractionSettings {
            preserve_formatting: true,
            include_coordinates: false,
            group_by_blocks: true,
            min_font_size: 8.0,
            extract_tables: false,
        },
        timeout_seconds: 30,
    };

    let processor = DefaultPdfProcessor::new();
    let pdf_data = create_test_pdf_data();

    // Test PDF processing
    let result = processor.process_pdf(&pdf_data, &pdf_config).await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.success);
    assert!(result.text.is_some());
    assert_eq!(result.metadata.page_count, 1);
    assert!(result.error.is_none());

    // Verify configuration was applied
    assert_eq!(result.images.len(), 0); // Images disabled
    assert!(result.metadata.pdf_version.is_some());
}

#[tokio::test]
async fn test_crawl_options_with_phase3_features() {
    let crawl_options = CrawlOptions {
        concurrency: 8,
        cache_mode: "read_through".to_string(),
        dynamic_wait_for: Some("window.appReady".to_string()),
        scroll_steps: 5,
        token_chunk_max: 2000,
        token_overlap: 200,
        dynamic_config: Some(create_test_dynamic_config()),
        stealth_config: Some(create_test_stealth_config()),
        pdf_config: Some(create_test_pdf_config()),
        render_mode: RenderMode::Adaptive,
        output_format: OutputFormat::Document,
    };

    // Test that all configurations are properly set
    assert_eq!(crawl_options.concurrency, 8);
    assert!(crawl_options.dynamic_config.is_some());
    assert!(crawl_options.stealth_config.is_some());
    assert!(crawl_options.pdf_config.is_some());
    assert!(matches!(crawl_options.render_mode, RenderMode::Adaptive));
    assert!(matches!(crawl_options.output_format, OutputFormat::Document));

    // Test serialization of complex structure
    let json = serde_json::to_string(&crawl_options).expect("Should serialize");
    let deserialized: CrawlOptions = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(crawl_options.concurrency, deserialized.concurrency);
    assert!(deserialized.dynamic_config.is_some());
    assert!(deserialized.stealth_config.is_some());
    assert!(deserialized.pdf_config.is_some());
}

#[tokio::test]
async fn test_render_mode_selection() {
    let test_cases = vec![
        ("https://example.com/article.html", RenderMode::Adaptive, "Should default to adaptive"),
        ("https://example.com/spa-app/", RenderMode::Dynamic, "SPA should use dynamic"),
        ("https://example.com/document.pdf", RenderMode::Pdf, "PDF should use PDF mode"),
        ("https://example.com/simple.html", RenderMode::Static, "Simple page can use static"),
    ];

    for (url, expected_mode, description) in test_cases {
        // Simulate mode selection logic
        let selected_mode = if url.ends_with(".pdf") {
            RenderMode::Pdf
        } else if url.contains("spa-") {
            RenderMode::Dynamic
        } else if url.contains("simple") {
            RenderMode::Static
        } else {
            RenderMode::Adaptive
        };

        assert!(
            std::mem::discriminant(&selected_mode) == std::mem::discriminant(&expected_mode),
            "{}: Expected {:?}, got {:?}",
            description,
            expected_mode,
            selected_mode
        );
    }
}

#[tokio::test]
async fn test_output_format_selection() {
    let formats = vec![
        OutputFormat::Document,
        OutputFormat::NdJson,
        OutputFormat::Chunked,
        OutputFormat::Text,
        OutputFormat::Markdown,
    ];

    for format in formats {
        let crawl_options = CrawlOptions {
            output_format: format.clone(),
            ..Default::default()
        };

        // Test serialization
        let json = serde_json::to_string(&crawl_options).expect("Should serialize");
        let deserialized: CrawlOptions = serde_json::from_str(&json).expect("Should deserialize");

        assert!(
            std::mem::discriminant(&crawl_options.output_format) ==
            std::mem::discriminant(&deserialized.output_format)
        );
    }
}

#[tokio::test]
async fn test_complex_wait_condition_combinations() {
    let complex_conditions = vec![
        // Single conditions
        WaitCondition::DomContentLoaded,
        WaitCondition::Load,
        WaitCondition::Selector {
            selector: ".content-ready".to_string(),
            timeout: Duration::from_secs(5),
        },
        WaitCondition::Javascript {
            script: "window.apiLoaded && window.uiReady".to_string(),
            timeout: Duration::from_secs(10),
        },
        WaitCondition::NetworkIdle {
            timeout: Duration::from_secs(15),
            idle_time: Duration::from_secs(2),
        },
        // Complex nested condition
        WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::Selector {
                selector: ".api-data-loaded".to_string(),
                timeout: Duration::from_secs(8),
            },
            WaitCondition::Javascript {
                script: "document.querySelectorAll('.lazy-image').length === document.querySelectorAll('.lazy-image.loaded').length".to_string(),
                timeout: Duration::from_secs(12),
            },
        ]),
    ];

    for condition in complex_conditions {
        let dynamic_config = DynamicConfig {
            wait_for: Some(condition.clone()),
            scroll: None,
            actions: Vec::new(),
            capture_artifacts: false,
            timeout: Duration::from_secs(30),
            viewport: None,
        };

        // Test serialization of complex nested structures
        let json = serde_json::to_string(&dynamic_config).expect("Should serialize");
        let deserialized: DynamicConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert!(deserialized.wait_for.is_some());

        // Verify condition types match
        let original = dynamic_config.wait_for.as_ref().unwrap();
        let deserialized_condition = deserialized.wait_for.as_ref().unwrap();

        assert!(
            std::mem::discriminant(original) == std::mem::discriminant(deserialized_condition),
            "Condition types should match after serialization"
        );

        // For Multiple conditions, verify length
        if let (WaitCondition::Multiple(orig), WaitCondition::Multiple(deser)) =
            (original, deserialized_condition) {
            assert_eq!(orig.len(), deser.len(), "Multiple condition lengths should match");
        }
    }
}

#[tokio::test]
async fn test_stealth_and_dynamic_coordination() {
    // Test that stealth and dynamic configs can work together
    let stealth_config = StealthConfig {
        user_agent: riptide_core::stealth::UserAgentConfig {
            agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            ],
            strategy: riptide_core::stealth::RotationStrategy::Sticky,
            include_mobile: false,
            browser_preference: riptide_core::stealth::BrowserType::Chrome,
        },
        request_randomization: riptide_core::stealth::RequestRandomization {
            timing_jitter: riptide_core::stealth::TimingJitter {
                base_delay_ms: 2000,
                jitter_percentage: 0.2,
                min_delay_ms: 1500,
                max_delay_ms: 3000,
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let dynamic_config = DynamicConfig {
        wait_for: Some(WaitCondition::Multiple(vec![
            WaitCondition::DomContentLoaded,
            WaitCondition::NetworkIdle {
                timeout: Duration::from_secs(10),
                idle_time: Duration::from_secs(1),
            },
        ])),
        actions: vec![
            PageAction::Wait(WaitCondition::Timeout(Duration::from_millis(1500))), // Matches stealth delay
            PageAction::Evaluate {
                script: "console.log('Stealth rendering active')".to_string(),
                wait_after: Some(Duration::from_millis(500)),
            },
        ],
        timeout: Duration::from_secs(45), // Longer timeout for stealth operations
        ..Default::default()
    };

    let mut stealth_controller = StealthController::new(stealth_config);

    // Test coordination
    let delay = stealth_controller.calculate_delay();
    assert!(delay.as_millis() >= 1500);
    assert!(delay.as_millis() <= 3000);

    // Verify dynamic config timeout accommodates stealth delays
    assert!(dynamic_config.timeout >= Duration::from_secs(30));

    // Test that wait conditions can be combined
    if let Some(WaitCondition::Multiple(conditions)) = &dynamic_config.wait_for {
        assert_eq!(conditions.len(), 2);
        assert!(matches!(conditions[0], WaitCondition::DomContentLoaded));
        assert!(matches!(conditions[1], WaitCondition::NetworkIdle { .. }));
    }
}

#[tokio::test]
async fn test_error_handling_integration() {
    // Test that different error types can be properly handled
    let pdf_processor = DefaultPdfProcessor::new();

    // Test invalid PDF
    let invalid_data = b"Not a PDF file";
    let result = pdf_processor.process_pdf(invalid_data, &PdfConfig::default()).await;
    assert!(result.is_err());

    // Test file too large
    let config_small_limit = PdfConfig {
        max_size_bytes: 100, // Very small limit
        ..Default::default()
    };
    let large_data = vec![0u8; 200];
    let result = pdf_processor.process_pdf(&large_data, &config_small_limit).await;
    assert!(result.is_err());

    // Test valid PDF
    let pdf_data = create_test_pdf_data();
    let result = pdf_processor.process_pdf(&pdf_data, &PdfConfig::default()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_comprehensive_crawl_options_defaults() {
    let default_options = CrawlOptions::default();

    // Test that defaults are sensible
    assert_eq!(default_options.concurrency, 16);
    assert_eq!(default_options.cache_mode, "read_through");
    assert_eq!(default_options.scroll_steps, 8);
    assert_eq!(default_options.token_chunk_max, 1200);
    assert_eq!(default_options.token_overlap, 120);

    // Phase 3 defaults
    assert!(default_options.dynamic_config.is_none());
    assert!(default_options.stealth_config.is_none());
    assert!(default_options.pdf_config.is_none());
    assert!(matches!(default_options.render_mode, RenderMode::Adaptive));
    assert!(matches!(default_options.output_format, OutputFormat::Document));
}

#[tokio::test]
async fn test_memory_and_performance_considerations() {
    // Test that configurations don't consume excessive memory
    let large_dynamic_config = DynamicConfig {
        actions: (0..100).map(|i| PageAction::Evaluate {
            script: format!("console.log('Action {}')", i),
            wait_after: Some(Duration::from_millis(10)),
        }).collect(),
        ..Default::default()
    };

    // Should still be serializable
    let json = serde_json::to_string(&large_dynamic_config).expect("Should serialize large config");
    assert!(!json.is_empty());

    // Test large stealth config
    let large_stealth_config = StealthConfig {
        user_agent: riptide_core::stealth::UserAgentConfig {
            agents: (0..50).map(|i| format!("UserAgent-{}", i)).collect(),
            ..Default::default()
        },
        ..Default::default()
    };

    let json = serde_json::to_string(&large_stealth_config).expect("Should serialize large stealth config");
    assert!(!json.is_empty());
}

#[tokio::test]
async fn test_feature_flag_compatibility() {
    // Test that features can be selectively enabled/disabled
    let minimal_config = CrawlOptions {
        concurrency: 1,
        cache_mode: "bypass".to_string(),
        dynamic_config: None, // No dynamic features
        stealth_config: None, // No stealth features
        pdf_config: None,     // No PDF features
        render_mode: RenderMode::Static, // Simple static rendering
        output_format: OutputFormat::Text, // Plain text output
        ..Default::default()
    };

    let full_config = CrawlOptions {
        concurrency: 32,
        cache_mode: "enabled".to_string(),
        dynamic_config: Some(create_test_dynamic_config()),
        stealth_config: Some(create_test_stealth_config()),
        pdf_config: Some(create_test_pdf_config()),
        render_mode: RenderMode::Dynamic,
        output_format: OutputFormat::NdJson,
        ..Default::default()
    };

    // Both should be valid
    assert!(minimal_config.dynamic_config.is_none());
    assert!(full_config.dynamic_config.is_some());

    // Test serialization compatibility
    let minimal_json = serde_json::to_string(&minimal_config).expect("Should serialize minimal");
    let full_json = serde_json::to_string(&full_config).expect("Should serialize full");

    let _: CrawlOptions = serde_json::from_str(&minimal_json).expect("Should deserialize minimal");
    let _: CrawlOptions = serde_json::from_str(&full_json).expect("Should deserialize full");
}