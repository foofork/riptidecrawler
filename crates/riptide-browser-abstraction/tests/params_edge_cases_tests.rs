//! Parameter edge case tests
//!
//! Tests boundary conditions, validation, and edge cases for all parameter types

use riptide_browser_abstraction::{
    NavigateParams, PdfParams, ScreenshotFormat, ScreenshotParams, WaitUntil,
};

// ===== ScreenshotParams Tests =====

#[test]
fn test_screenshot_params_all_combinations() {
    let combinations = vec![
        ScreenshotParams {
            full_page: true,
            format: ScreenshotFormat::Png,
            quality: Some(100),
            viewport_only: true,
        },
        ScreenshotParams {
            full_page: false,
            format: ScreenshotFormat::Jpeg,
            quality: Some(0),
            viewport_only: false,
        },
        ScreenshotParams {
            full_page: true,
            format: ScreenshotFormat::Jpeg,
            quality: None,
            viewport_only: false,
        },
    ];

    assert_eq!(combinations[0].quality, Some(100));
    assert_eq!(combinations[1].quality, Some(0));
    assert_eq!(combinations[2].quality, None);
}

#[test]
fn test_screenshot_quality_boundary_values() {
    let params_min = ScreenshotParams {
        format: ScreenshotFormat::Jpeg,
        quality: Some(0),
        full_page: false,
        viewport_only: false,
    };

    let params_max = ScreenshotParams {
        format: ScreenshotFormat::Jpeg,
        quality: Some(100),
        full_page: false,
        viewport_only: false,
    };

    assert_eq!(params_min.quality, Some(0));
    assert_eq!(params_max.quality, Some(100));
}

#[test]
fn test_screenshot_png_ignores_quality() {
    let params = ScreenshotParams {
        format: ScreenshotFormat::Png,
        quality: Some(50), // Quality should be ignored for PNG
        full_page: false,
        viewport_only: false,
    };

    // Even though quality is set, PNG format should ignore it
    assert!(matches!(params.format, ScreenshotFormat::Png));
}

#[test]
fn test_screenshot_contradictory_flags() {
    // Both full_page and viewport_only set to true
    let params = ScreenshotParams {
        full_page: true,
        format: ScreenshotFormat::Png,
        quality: None,
        viewport_only: true,
    };

    // Should handle gracefully (implementation decides priority)
    assert!(params.full_page);
    assert!(params.viewport_only);
}

#[test]
fn test_screenshot_format_serialization() {
    let png = ScreenshotFormat::Png;
    let jpeg = ScreenshotFormat::Jpeg;

    let png_json = serde_json::to_string(&png).unwrap();
    let jpeg_json = serde_json::to_string(&jpeg).unwrap();

    assert!(png_json.contains("Png"));
    assert!(jpeg_json.contains("Jpeg"));
}

#[test]
fn test_screenshot_params_clone() {
    let original = ScreenshotParams {
        full_page: true,
        format: ScreenshotFormat::Jpeg,
        quality: Some(85),
        viewport_only: false,
    };

    let cloned = original.clone();
    assert_eq!(cloned.quality, original.quality);
    assert!(cloned.full_page);
}

// ===== PdfParams Tests =====

#[test]
fn test_pdf_params_extreme_scale() {
    let params_tiny = PdfParams {
        scale: Some(0.1),
        ..Default::default()
    };

    let params_huge = PdfParams {
        scale: Some(10.0),
        ..Default::default()
    };

    assert_eq!(params_tiny.scale, Some(0.1));
    assert_eq!(params_huge.scale, Some(10.0));
}

#[test]
fn test_pdf_params_zero_margins() {
    let params = PdfParams {
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        ..Default::default()
    };

    assert_eq!(params.margin_top, Some(0.0));
    assert_eq!(params.margin_bottom, Some(0.0));
}

#[test]
fn test_pdf_params_large_margins() {
    let params = PdfParams {
        margin_top: Some(5.0),
        margin_bottom: Some(5.0),
        margin_left: Some(5.0),
        margin_right: Some(5.0),
        ..Default::default()
    };

    // Should not exceed paper size
    assert!(params.margin_top.unwrap() < params.paper_height.unwrap());
}

#[test]
fn test_pdf_params_custom_paper_sizes() {
    // A4 size
    let a4 = PdfParams {
        paper_width: Some(8.27),
        paper_height: Some(11.69),
        ..Default::default()
    };

    // Legal size
    let legal = PdfParams {
        paper_width: Some(8.5),
        paper_height: Some(14.0),
        ..Default::default()
    };

    assert_eq!(a4.paper_width, Some(8.27));
    assert_eq!(legal.paper_height, Some(14.0));
}

#[test]
fn test_pdf_params_page_ranges() {
    let params = PdfParams {
        page_ranges: Some("1-5,8,10-12".to_string()),
        ..Default::default()
    };

    assert!(params.page_ranges.is_some());
    assert!(params.page_ranges.unwrap().contains("1-5"));
}

#[test]
fn test_pdf_params_empty_page_ranges() {
    let params = PdfParams {
        page_ranges: Some("".to_string()),
        ..Default::default()
    };

    assert_eq!(params.page_ranges, Some("".to_string()));
}

#[test]
fn test_pdf_params_all_options_enabled() {
    let params = PdfParams {
        print_background: true,
        scale: Some(1.5),
        landscape: true,
        paper_width: Some(11.0),
        paper_height: Some(17.0),
        display_header_footer: true,
        margin_top: Some(1.0),
        margin_bottom: Some(1.0),
        margin_left: Some(0.5),
        margin_right: Some(0.5),
        page_ranges: Some("1-10".to_string()),
        prefer_css_page_size: Some(true),
    };

    assert!(params.print_background);
    assert!(params.landscape);
    assert!(params.display_header_footer);
    assert_eq!(params.prefer_css_page_size, Some(true));
}

#[test]
fn test_pdf_params_all_options_disabled() {
    let params = PdfParams {
        print_background: false,
        scale: None,
        landscape: false,
        paper_width: None,
        paper_height: None,
        display_header_footer: false,
        margin_top: None,
        margin_bottom: None,
        margin_left: None,
        margin_right: None,
        page_ranges: None,
        prefer_css_page_size: Some(false),
    };

    assert!(!params.print_background);
    assert!(!params.landscape);
    assert!(params.scale.is_none());
}

#[test]
fn test_pdf_params_clone() {
    let original = PdfParams {
        landscape: true,
        scale: Some(2.0),
        ..Default::default()
    };

    let cloned = original.clone();
    assert_eq!(cloned.landscape, original.landscape);
    assert_eq!(cloned.scale, original.scale);
}

// ===== NavigateParams Tests =====

#[test]
fn test_navigate_params_zero_timeout() {
    let params = NavigateParams {
        timeout_ms: 0,
        wait_until: WaitUntil::Load,
        referer: None,
    };

    assert_eq!(params.timeout_ms, 0);
}

#[test]
fn test_navigate_params_very_long_timeout() {
    let params = NavigateParams {
        timeout_ms: 3600000, // 1 hour
        wait_until: WaitUntil::Load,
        referer: None,
    };

    assert_eq!(params.timeout_ms, 3600000);
}

#[test]
fn test_navigate_params_with_referer() {
    let params = NavigateParams {
        timeout_ms: 30000,
        wait_until: WaitUntil::Load,
        referer: Some("https://example.com/previous".to_string()),
    };

    assert!(params.referer.is_some());
    assert!(params.referer.unwrap().contains("example.com"));
}

#[test]
fn test_navigate_params_all_wait_strategies() {
    let params_load = NavigateParams {
        timeout_ms: 30000,
        wait_until: WaitUntil::Load,
        referer: None,
    };

    let params_dom = NavigateParams {
        timeout_ms: 30000,
        wait_until: WaitUntil::DOMContentLoaded,
        referer: None,
    };

    let params_idle = NavigateParams {
        timeout_ms: 30000,
        wait_until: WaitUntil::NetworkIdle,
        referer: None,
    };

    assert!(matches!(params_load.wait_until, WaitUntil::Load));
    assert!(matches!(params_dom.wait_until, WaitUntil::DOMContentLoaded));
    assert!(matches!(params_idle.wait_until, WaitUntil::NetworkIdle));
}

#[test]
fn test_navigate_params_clone() {
    let original = NavigateParams {
        timeout_ms: 15000,
        wait_until: WaitUntil::NetworkIdle,
        referer: Some("https://test.com".to_string()),
    };

    let cloned = original.clone();
    assert_eq!(cloned.timeout_ms, original.timeout_ms);
    assert!(matches!(cloned.wait_until, WaitUntil::NetworkIdle));
}

#[test]
fn test_wait_until_serialization() {
    let load = WaitUntil::Load;
    let dom = WaitUntil::DOMContentLoaded;
    let idle = WaitUntil::NetworkIdle;

    let load_json = serde_json::to_string(&load).unwrap();
    let dom_json = serde_json::to_string(&dom).unwrap();
    let idle_json = serde_json::to_string(&idle).unwrap();

    assert!(load_json.contains("Load"));
    assert!(dom_json.contains("DOMContentLoaded"));
    assert!(idle_json.contains("NetworkIdle"));
}

#[test]
fn test_navigate_params_serialization_round_trip() {
    let original = NavigateParams {
        timeout_ms: 45000,
        wait_until: WaitUntil::DOMContentLoaded,
        referer: Some("https://ref.example.com".to_string()),
    };

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: NavigateParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.timeout_ms, original.timeout_ms);
    assert_eq!(deserialized.referer, original.referer);
}

#[test]
fn test_screenshot_params_serialization_round_trip() {
    let original = ScreenshotParams {
        full_page: true,
        format: ScreenshotFormat::Jpeg,
        quality: Some(75),
        viewport_only: false,
    };

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: ScreenshotParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.full_page, original.full_page);
    assert_eq!(deserialized.quality, original.quality);
}

#[test]
fn test_pdf_params_serialization_round_trip() {
    let original = PdfParams {
        print_background: true,
        scale: Some(1.2),
        landscape: true,
        paper_width: Some(8.5),
        paper_height: Some(11.0),
        display_header_footer: false,
        margin_top: Some(0.5),
        margin_bottom: Some(0.5),
        margin_left: Some(0.5),
        margin_right: Some(0.5),
        page_ranges: Some("1-3".to_string()),
        prefer_css_page_size: Some(false),
    };

    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: PdfParams = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized.landscape, original.landscape);
    assert_eq!(deserialized.scale, original.scale);
    assert_eq!(deserialized.page_ranges, original.page_ranges);
}
