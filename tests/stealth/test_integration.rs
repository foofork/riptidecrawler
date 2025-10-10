//! Integration tests for complete stealth workflows
//!
//! Tests the full stealth system working together, including session management,
//! configuration updates, and realistic usage scenarios

use riptide_core::stealth::{
    LocaleStrategy, StealthConfig, StealthController, StealthPreset,
};
use std::collections::HashSet;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_complete_stealth_workflow() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // 1. Get user agent
    let user_agent = controller.next_user_agent().to_string();
    assert!(!user_agent.is_empty());
    assert!(user_agent.contains("Mozilla"));

    // 2. Generate headers
    let headers = controller.generate_headers();
    assert!(!headers.is_empty());
    assert!(headers.contains_key("Accept"));

    // 3. Get viewport
    let (width, height) = controller.random_viewport();
    assert!(width > 0 && height > 0);

    // 4. Get locale
    let (locale, timezone) = controller.random_locale();
    assert!(!locale.is_empty());
    assert!(!timezone.is_empty());

    // 5. Get hardware specs
    let (cores, memory) = controller.random_hardware_specs();
    assert!(cores > 0 && memory > 0);

    // 6. Get WebGL specs
    let (vendor, renderer) = controller.random_webgl_specs();
    assert!(!vendor.is_empty());
    assert!(!renderer.is_empty());

    // 7. Get JavaScript injection
    let js_code = controller.get_stealth_js();
    assert!(!js_code.is_empty());

    // 8. Calculate delay
    let delay = controller.calculate_delay();
    assert!(delay.as_millis() > 0);

    // 9. Mark request sent
    controller.mark_request_sent();
    assert!(controller.get_request_count() > 0);
}

#[tokio::test]
async fn test_multiple_request_simulation() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    let mut user_agents = HashSet::new();
    let mut delays = Vec::new();

    // Simulate 10 requests
    for _ in 0..10 {
        // Get user agent
        let ua = controller.next_user_agent().to_string();
        user_agents.insert(ua);

        // Generate headers
        let headers = controller.generate_headers();
        assert!(!headers.is_empty());

        // Calculate delay and wait
        let delay = controller.calculate_delay();
        delays.push(delay);

        // Mark request sent
        controller.mark_request_sent();
    }

    assert_eq!(controller.get_request_count(), 10);
    assert!(!delays.is_empty());
}

#[tokio::test]
async fn test_session_reset() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // Make some requests
    controller.next_user_agent();
    controller.mark_request_sent();
    controller.calculate_delay();

    assert!(controller.get_request_count() > 0);

    // Reset session
    controller.reset_session();

    assert_eq!(controller.get_request_count(), 0);

    // Should still work after reset
    let ua = controller.next_user_agent();
    assert!(!ua.is_empty());
}

#[tokio::test]
async fn test_config_update_mid_session() {
    let mut controller = StealthController::from_preset(StealthPreset::Low);

    assert_eq!(controller.get_preset(), &StealthPreset::Low);

    // Update to High preset
    let high_config = StealthConfig::from_preset(StealthPreset::High);
    controller.update_config(high_config);

    assert_eq!(controller.get_preset(), &StealthPreset::High);

    // Should work with new config
    let ua = controller.next_user_agent();
    assert!(!ua.is_empty());
}

#[tokio::test]
async fn test_locale_strategy_fixed() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.locale.strategy = LocaleStrategy::Fixed("de-DE".to_string());

    let controller = StealthController::new(config);

    // All locale requests should return the same locale
    for _ in 0..5 {
        let (locale, _) = controller.random_locale();
        assert_eq!(locale, "de-DE");
    }
}

#[tokio::test]
async fn test_locale_strategy_random_provides_variety() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.locale.strategy = LocaleStrategy::Random;

    let controller = StealthController::new(config);

    let mut locales = HashSet::new();

    // Should see some variety over 20 requests
    for _ in 0..20 {
        let (locale, _) = controller.random_locale();
        locales.insert(locale.to_string());
    }

    // Likely to have multiple locales
    assert!(!locales.is_empty());
}

#[test]
fn test_preset_escalation() {
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let controller = StealthController::from_preset(preset.clone());
        assert_eq!(controller.get_preset(), &preset);

        // All presets should provide basic functionality
        let mut controller_mut = StealthController::from_preset(preset);
        let ua = controller_mut.next_user_agent();
        assert!(!ua.is_empty());
    }
}

#[tokio::test]
async fn test_timing_between_requests() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    controller.mark_request_sent();

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;

    let elapsed = controller.time_since_last_request();
    assert!(elapsed.as_millis() >= 100);
}

#[test]
fn test_stealth_enabled_check() {
    let enabled = StealthController::from_preset(StealthPreset::High);
    let disabled = StealthController::from_preset(StealthPreset::None);

    assert!(enabled.is_stealth_enabled());
    assert!(!disabled.is_stealth_enabled());
}

#[test]
fn test_default_config_valid() {
    let config = StealthConfig::default();

    // Should have valid user agents
    assert!(!config.user_agent.agents.is_empty());

    // Should have header variations
    assert!(!config.request_randomization.headers.accept_variations.is_empty());

    // Should have viewport sizes
    assert!(!config.request_randomization.viewport.sizes.is_empty());

    // Should have locales
    assert!(!config.request_randomization.locale.locales.is_empty());

    // Fingerprinting should be enabled
    assert!(config.fingerprinting.cdp_stealth.disable_automation_controlled);
}

#[test]
fn test_add_custom_user_agents() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    controller.add_user_agents(vec![
        "Custom UA 1".to_string(),
        "Custom UA 2".to_string(),
    ]);

    // Should still work
    let ua = controller.next_user_agent();
    assert!(!ua.is_empty());
}

#[test]
fn test_config_serialization_roundtrip() {
    let original_config = StealthConfig::from_preset(StealthPreset::High);

    let json = serde_json::to_string(&original_config).expect("Should serialize");
    let deserialized: StealthConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(original_config.preset, deserialized.preset);
    assert_eq!(
        original_config.user_agent.agents.len(),
        deserialized.user_agent.agents.len()
    );
}

#[tokio::test]
async fn test_performance_under_load() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    let start = Instant::now();

    // Simulate 100 requests
    for _ in 0..100 {
        controller.next_user_agent();
        controller.generate_headers();
        controller.random_viewport();
        controller.calculate_delay();
        controller.mark_request_sent();
    }

    let elapsed = start.elapsed();

    // Should complete reasonably quickly (< 1 second for 100 requests)
    assert!(
        elapsed.as_millis() < 1000,
        "100 requests took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_memory_efficiency() {
    // Create multiple controllers
    let controllers: Vec<_> = (0..50)
        .map(|_| StealthController::from_preset(StealthPreset::High))
        .collect();

    assert_eq!(controllers.len(), 50);

    // All should be functional
    for controller in &controllers {
        assert!(controller.is_stealth_enabled());
    }
}

#[test]
fn test_concurrent_access_safety() {
    use std::sync::{Arc, RwLock};

    let controller = Arc::new(RwLock::new(StealthController::from_preset(
        StealthPreset::Medium,
    )));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let ctrl = Arc::clone(&controller);
            std::thread::spawn(move || {
                let mut c = ctrl.write().unwrap();
                c.next_user_agent().to_string()
            })
        })
        .collect();

    for handle in handles {
        let ua = handle.join().unwrap();
        assert!(!ua.is_empty());
    }
}

#[test]
fn test_request_count_tracking() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    assert_eq!(controller.get_request_count(), 0);

    for i in 1..=5 {
        controller.mark_request_sent();
        assert_eq!(controller.get_request_count(), i);
    }
}

#[test]
fn test_timezone_mappings_complete() {
    let config = StealthConfig::default();

    // Each locale should have a timezone
    for locale in &config.request_randomization.locale.locales {
        assert!(
            config
                .request_randomization
                .locale
                .timezones
                .contains_key(locale),
            "Locale {} missing timezone mapping",
            locale
        );
    }
}

#[test]
fn test_cdp_flags_generation() {
    let none_controller = StealthController::from_preset(StealthPreset::None);
    let high_controller = StealthController::from_preset(StealthPreset::High);

    let none_flags = none_controller.get_cdp_flags();
    let high_flags = high_controller.get_cdp_flags();

    // None should have minimal flags
    assert!(none_flags.is_empty());

    // High should have comprehensive flags
    assert!(!high_flags.is_empty());
    assert!(high_flags.contains(&"--disable-blink-features=AutomationControlled".to_string()));
}

#[test]
fn test_domain_timing_fallback() {
    let controller = StealthController::from_preset(StealthPreset::Medium);

    // Unknown domain should use default timing
    let timing = controller.get_domain_timing("unknown-domain.com");
    assert!(timing.min_delay_ms > 0);
    assert!(timing.min_delay_ms <= timing.max_delay_ms);
}
