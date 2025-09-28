//! Integration tests for riptide-stealth crate
//!
//! These tests verify that the public API works correctly
//! and that all stealth functionality is properly exported.

use riptide_stealth::*;

#[test]
fn test_stealth_controller_api() {
    // Test creating controllers with different presets
    let mut high_stealth = create_high_stealth_controller();
    let mut medium_stealth = create_medium_stealth_controller();
    let mut low_stealth = create_low_stealth_controller();
    let mut no_stealth = create_no_stealth_controller();

    // Test that each preset works
    assert_eq!(high_stealth.get_preset(), &StealthPreset::High);
    assert_eq!(medium_stealth.get_preset(), &StealthPreset::Medium);
    assert_eq!(low_stealth.get_preset(), &StealthPreset::Low);
    assert_eq!(no_stealth.get_preset(), &StealthPreset::None);

    // Test user agent rotation
    let ua1 = high_stealth.next_user_agent();
    let ua2 = medium_stealth.next_user_agent();
    let ua3 = low_stealth.next_user_agent();
    let ua4 = no_stealth.next_user_agent();

    assert!(!ua1.is_empty());
    assert!(!ua2.is_empty());
    assert!(!ua3.is_empty());
    assert!(!ua4.is_empty());

    // Test header generation
    let headers = high_stealth.generate_headers();
    assert!(headers.len() >= 3);
    assert!(headers.contains_key("Accept"));
    assert!(headers.contains_key("Accept-Language"));
    assert!(headers.contains_key("Accept-Encoding"));

    // Test JavaScript generation
    let js_code = high_stealth.get_stealth_js();
    assert!(!js_code.is_empty());
    assert!(js_code.contains("webdriver"));

    // Test stealth detection
    assert!(high_stealth.is_stealth_enabled());
    assert!(medium_stealth.is_stealth_enabled());
    assert!(low_stealth.is_stealth_enabled());
    assert!(!no_stealth.is_stealth_enabled());
}

#[test]
fn test_config_types() {
    // Test that all config types are properly exported
    let _stealth_config = StealthConfig::default();
    let _user_agent_config = UserAgentConfig::default();
    let _fingerprinting_config = FingerprintingConfig::default();
    let _request_randomization = RequestRandomization::default();

    // Test enums
    let _rotation_strategy = RotationStrategy::Random;
    let _browser_type = BrowserType::Chrome;
    let _locale_strategy = LocaleStrategy::Random;
    let _stealth_preset = StealthPreset::Medium;
}

#[test]
fn test_user_agent_manager() {
    let config = UserAgentConfig::default();
    let mut manager = UserAgentManager::new(config);

    // Test basic functionality
    assert!(manager.agent_count() > 0);

    let ua1 = manager.next_user_agent().to_string();
    let ua2 = manager.next_user_agent().to_string();

    assert!(!ua1.is_empty());
    assert!(!ua2.is_empty());
}

#[test]
fn test_javascript_injector() {
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
    assert!(!js_code.is_empty());
    assert!(js_code.contains("webdriver"));
    assert!(js_code.contains("hardwareConcurrency"));
}

#[test]
fn test_crate_info() {
    // Test that version and crate name are exported
    assert!(!VERSION.is_empty());
    assert_eq!(CRATE_NAME, "riptide-stealth");
}

#[test]
fn test_load_user_agents_function() {
    // Test that the function is exported and handles errors correctly
    let result = load_user_agents_from_file("nonexistent_file.txt");
    assert!(result.is_err());
}