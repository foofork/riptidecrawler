//! Trait behavior and contract tests
//!
//! Tests for trait implementations, Send+Sync bounds, and trait object behavior

use riptide_browser_abstraction::{EngineType, NavigateParams, PdfParams, ScreenshotParams};
use std::fmt::Debug;

// Test that EngineType implements required traits
#[test]
fn test_engine_type_traits() {
    fn assert_debug<T: Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_copy<T: Copy>() {}
    fn assert_eq<T: PartialEq>() {}
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_debug::<EngineType>();
    assert_clone::<EngineType>();
    assert_copy::<EngineType>();
    assert_eq::<EngineType>();
    assert_send::<EngineType>();
    assert_sync::<EngineType>();
}

#[test]
fn test_screenshot_params_traits() {
    fn assert_debug<T: Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_debug::<ScreenshotParams>();
    assert_clone::<ScreenshotParams>();
    assert_send::<ScreenshotParams>();
    assert_sync::<ScreenshotParams>();
}

#[test]
fn test_pdf_params_traits() {
    fn assert_debug<T: Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_debug::<PdfParams>();
    assert_clone::<PdfParams>();
    assert_send::<PdfParams>();
    assert_sync::<PdfParams>();
}

#[test]
fn test_navigate_params_traits() {
    fn assert_debug<T: Debug>() {}
    fn assert_clone<T: Clone>() {}
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_debug::<NavigateParams>();
    assert_clone::<NavigateParams>();
    assert_send::<NavigateParams>();
    assert_sync::<NavigateParams>();
}

#[test]
fn test_params_default_implementations() {
    // All params should have sensible defaults
    let screenshot = ScreenshotParams::default();
    let pdf = PdfParams::default();
    let navigate = NavigateParams::default();

    // Screenshot defaults
    assert!(!screenshot.full_page);
    assert_eq!(screenshot.quality, Some(80));

    // PDF defaults
    assert!(pdf.print_background);
    assert_eq!(pdf.scale, Some(1.0));
    assert_eq!(pdf.paper_width, Some(8.5));
    assert_eq!(pdf.paper_height, Some(11.0));

    // Navigate defaults
    assert_eq!(navigate.timeout_ms, 30000);
    assert!(navigate.referer.is_none());
}

#[test]
fn test_engine_type_partial_eq_reflexive() {
    let engine = EngineType::Chromiumoxide;
    assert_eq!(engine, engine);
}

#[test]
fn test_engine_type_partial_eq_symmetric() {
    let engine1 = EngineType::SpiderChrome;
    let engine2 = EngineType::SpiderChrome;

    assert_eq!(engine1, engine2);
    assert_eq!(engine2, engine1);
}

#[test]
fn test_engine_type_partial_eq_transitive() {
    let engine1 = EngineType::Chromiumoxide;
    let engine2 = EngineType::Chromiumoxide;
    let engine3 = EngineType::Chromiumoxide;

    assert_eq!(engine1, engine2);
    assert_eq!(engine2, engine3);
    assert_eq!(engine1, engine3);
}

#[test]
fn test_params_in_vec() {
    // Test that params can be stored in collections
    let screenshots = vec![
        ScreenshotParams::default(),
        ScreenshotParams {
            full_page: true,
            ..Default::default()
        },
    ];

    assert_eq!(screenshots.len(), 2);
}

#[test]
fn test_params_in_option() {
    // Test that params work with Option
    let maybe_screenshot: Option<ScreenshotParams> = Some(ScreenshotParams::default());
    let maybe_pdf: Option<PdfParams> = None;

    assert!(maybe_screenshot.is_some());
    assert!(maybe_pdf.is_none());
}

#[test]
fn test_params_in_result() {
    // Test that params work with Result
    let screenshot_result: Result<ScreenshotParams, String> = Ok(ScreenshotParams::default());
    let pdf_result: Result<PdfParams, String> = Err("error".to_string());

    assert!(screenshot_result.is_ok());
    assert!(pdf_result.is_err());
}

#[test]
fn test_engine_type_match_exhaustive() {
    let engine = EngineType::Chromiumoxide;

    // Should compile and work with exhaustive match
    let name = match engine {
        EngineType::Chromiumoxide => "chromiumoxide",
        EngineType::SpiderChrome => "spider-chrome",
    };

    assert_eq!(name, "chromiumoxide");
}

#[test]
fn test_params_builder_pattern() {
    // Test that params can be built incrementally
    let mut params = PdfParams::default();
    params.landscape = true;
    params.scale = Some(1.5);

    assert!(params.landscape);
    assert_eq!(params.scale, Some(1.5));
}

#[test]
fn test_params_struct_update_syntax() {
    // Test struct update syntax
    let base = ScreenshotParams::default();
    let custom = ScreenshotParams {
        full_page: true,
        ..base
    };

    assert!(custom.full_page);
    assert_eq!(custom.quality, Some(80)); // From base
}

#[test]
fn test_debug_output_quality() {
    use riptide_browser_abstraction::ScreenshotFormat;

    // Debug output should be useful
    let params = ScreenshotParams {
        full_page: true,
        format: ScreenshotFormat::Jpeg,
        quality: Some(90),
        viewport_only: false,
    };

    let debug = format!("{:?}", params);
    assert!(debug.contains("full_page"));
    assert!(debug.contains("true"));
    assert!(debug.contains("90"));
}

#[test]
fn test_clone_independence() {
    // Cloned params should be independent
    let original = PdfParams {
        landscape: false,
        ..Default::default()
    };

    let mut cloned = original.clone();
    cloned.landscape = true;

    assert!(!original.landscape);
    assert!(cloned.landscape);
}
