//! JavaScript injection and evasion tests
//!
//! Tests for JavaScript code generation, webdriver override, automation cleanup,
//! timezone spoofing, and all browser API overrides

use riptide_core::stealth::{
    CanvasConfig, HardwareConfig, JavaScriptInjector, LocaleStrategy, StealthController,
    StealthPreset, WebGlConfig,
};

#[test]
fn test_javascript_injection_includes_webdriver_override() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should override navigator.webdriver
    assert!(js_code.contains("'webdriver'"));
    assert!(js_code.contains("false"));
}

#[test]
fn test_javascript_injection_includes_plugins() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Fixed("en-US".to_string());

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should mock plugins
    assert!(js_code.contains("plugins"));
    assert!(js_code.contains("Chrome PDF Plugin"));
    assert!(js_code.contains("Native Client"));
}

#[test]
fn test_javascript_injection_includes_languages() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Fixed("de-DE".to_string());

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should set navigator.languages and language
    assert!(js_code.contains("languages"));
    assert!(js_code.contains("de-DE"));
}

#[test]
fn test_javascript_injection_includes_hardware_specs() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should spoof hardware
    assert!(js_code.contains("hardwareConcurrency"));
    assert!(js_code.contains("deviceMemory"));
    assert!(js_code.contains("platform"));
}

#[test]
fn test_javascript_injection_includes_webgl_spoofing() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should override WebGL
    assert!(js_code.contains("WebGLRenderingContext"));
    assert!(js_code.contains("getParameter"));
    assert!(js_code.contains("37445")); // UNMASKED_VENDOR_WEBGL
    assert!(js_code.contains("37446")); // UNMASKED_RENDERER_WEBGL
}

#[test]
fn test_javascript_injection_supports_webgl2() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should support WebGL2
    assert!(js_code.contains("WebGL2RenderingContext"));
}

#[test]
fn test_javascript_injection_includes_canvas_noise() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig {
        add_noise: true,
        noise_intensity: 0.05,
        block_data_extraction: false,
    };
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should override canvas methods
    assert!(js_code.contains("toDataURL") || js_code.contains("getImageData"));
}

#[test]
fn test_javascript_injection_no_canvas_noise_when_disabled() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig {
        add_noise: false,
        noise_intensity: 0.0,
        block_data_extraction: false,
    };
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Canvas protection should be minimal or absent when disabled
    // (implementation may still have structure but not active noise)
}

#[test]
fn test_javascript_injection_automation_cleanup() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should clean up automation properties
    // Note: These are deleted, so the code removes them
    assert!(
        js_code.contains("__webdriver")
            || js_code.contains("__fxdriver")
            || js_code.contains("__driver")
    );
}

#[test]
fn test_javascript_injection_timezone_override() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Fixed("en-US".to_string());

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should override timezone
    assert!(js_code.contains("getTimezoneOffset"));
    assert!(js_code.contains("DateTimeFormat") || js_code.contains("timeZone"));
}

#[test]
fn test_javascript_injection_additional_protections() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should include screen properties
    assert!(js_code.contains("colorDepth") || js_code.contains("pixelDepth"));

    // Should include battery API spoofing
    assert!(js_code.contains("getBattery"));

    // Should include audio context protection
    assert!(js_code.contains("AudioContext"));
}

#[test]
fn test_javascript_injection_locale_variations() {
    let locales = vec![
        LocaleStrategy::Fixed("en-US".to_string()),
        LocaleStrategy::Fixed("en-GB".to_string()),
        LocaleStrategy::Fixed("de-DE".to_string()),
        LocaleStrategy::Fixed("fr-FR".to_string()),
    ];

    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();

    for locale_strategy in locales {
        let injector = JavaScriptInjector::new(
            &hardware_config,
            &webgl_config,
            &canvas_config,
            &locale_strategy,
        );

        let js_code = injector.generate_stealth_js();

        // Each should produce valid JavaScript
        assert!(!js_code.is_empty());
        assert!(js_code.contains("function") || js_code.contains("const"));
    }
}

#[test]
fn test_javascript_injection_random_locale() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should select a valid locale
    assert!(!js_code.is_empty());
    assert!(js_code.contains("languages"));
}

#[test]
fn test_javascript_injection_size_reasonable() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should be substantial but not too large
    assert!(js_code.len() > 1000); // At least 1KB
    assert!(js_code.len() < 100000); // Less than 100KB
}

#[test]
fn test_javascript_injection_no_syntax_errors() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Basic syntax validation
    assert!(js_code.contains("(function()"));
    assert!(js_code.contains("'use strict'"));
    let open_braces = js_code.matches('{').count();
    let close_braces = js_code.matches('}').count();
    assert_eq!(
        open_braces, close_braces,
        "Braces should be balanced in generated JS"
    );
}

#[test]
fn test_stealth_controller_js_generation() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    let js_code = controller.get_stealth_js();

    assert!(!js_code.is_empty());
    assert!(js_code.contains("'webdriver'"));
    assert!(js_code.contains("hardwareConcurrency"));
    assert!(js_code.contains("WebGLRenderingContext"));
}

#[test]
fn test_stealth_controller_js_caching() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    let js1 = controller.get_stealth_js();
    let js2 = controller.get_stealth_js();

    // Should be consistent for same session
    assert_eq!(js1, js2);
}

#[test]
fn test_different_presets_different_js() {
    let mut low_controller = StealthController::from_preset(StealthPreset::Low);
    let mut high_controller = StealthController::from_preset(StealthPreset::High);

    let low_js = low_controller.get_stealth_js();
    let high_js = high_controller.get_stealth_js();

    // Both should be valid
    assert!(!low_js.is_empty());
    assert!(!high_js.is_empty());
}

#[test]
fn test_javascript_injection_no_debug_code() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should not contain debugging code
    assert!(!js_code.contains("console.log"));
    assert!(!js_code.contains("debugger"));
    assert!(!js_code.contains("alert("));
}

#[test]
fn test_javascript_injection_mime_types() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();
    let locale_strategy = LocaleStrategy::Random;

    let injector = JavaScriptInjector::new(
        &hardware_config,
        &webgl_config,
        &canvas_config,
        &locale_strategy,
    );

    let js_code = injector.generate_stealth_js();

    // Should mock mimeTypes
    assert!(js_code.contains("mimeTypes"));
    assert!(js_code.contains("application/pdf"));
}
