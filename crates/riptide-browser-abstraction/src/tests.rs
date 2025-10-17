//! Unit tests for browser abstraction

use crate::*;

#[test]
fn test_engine_type_serialization() {
    let chrome = EngineType::Chromiumoxide;
    assert_eq!(format!("{:?}", chrome), "Chromiumoxide");

    // Spider-chrome disabled due to type conflicts - see ADR-006
    // #[cfg(feature = "spider")]
    // {
    //     let spider = EngineType::SpiderChrome;
    //     assert_eq!(format!("{:?}", spider), "SpiderChrome");
    // }
}

#[test]
fn test_screenshot_params_default() {
    let params = ScreenshotParams::default();
    assert!(!params.full_page);
    assert_eq!(params.quality, Some(80));
    assert!(!params.viewport_only);
}

#[test]
fn test_pdf_params_default() {
    let params = PdfParams::default();
    assert!(params.print_background);
    assert_eq!(params.scale, 1.0);
    assert!(!params.landscape);
    assert_eq!(params.paper_width, Some(8.5));
    assert_eq!(params.paper_height, Some(11.0));
}

    #[test]
    fn test_navigate_params_default() {
        let params = NavigateParams::default();
        assert_eq!(params.timeout_ms, 30000);
        assert!(params.referer.is_none());
    }

    #[test]
    fn test_screenshot_format_variants() {
        let png = ScreenshotFormat::Png;
        let jpeg = ScreenshotFormat::Jpeg;

        assert_eq!(format!("{:?}", png), "Png");
        assert_eq!(format!("{:?}", jpeg), "Jpeg");
    }

    #[test]
    fn test_wait_until_variants() {
        use crate::params::WaitUntil;

        let load = WaitUntil::Load;
        let dom = WaitUntil::DOMContentLoaded;
        let idle = WaitUntil::NetworkIdle;

        assert_eq!(format!("{:?}", load), "Load");
        assert_eq!(format!("{:?}", dom), "DOMContentLoaded");
        assert_eq!(format!("{:?}", idle), "NetworkIdle");
    }

    #[test]
    fn test_error_types() {
        let err = AbstractionError::PageCreation("test".to_string());
        assert_eq!(err.to_string(), "Failed to create page: test");

        let err = AbstractionError::Navigation("test".to_string());
        assert_eq!(err.to_string(), "Failed to navigate: test");

        let err = AbstractionError::Unsupported("test".to_string());
        assert_eq!(err.to_string(), "Operation not supported: test");
    }

    #[test]
    fn test_custom_screenshot_params() {
        let params = ScreenshotParams {
            full_page: true,
            format: ScreenshotFormat::Jpeg,
            quality: Some(90),
            viewport_only: true,
        };

        assert!(params.full_page);
        assert_eq!(params.quality, Some(90));
        assert!(params.viewport_only);
    }

    #[test]
fn test_custom_pdf_params() {
    let params = PdfParams {
        print_background: false,
        scale: 1.5,
        landscape: true,
        paper_width: Some(11.0),
        paper_height: Some(17.0),
    };

    assert!(!params.print_background);
    assert_eq!(params.scale, 1.5);
    assert!(params.landscape);
}
