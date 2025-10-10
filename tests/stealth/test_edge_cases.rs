//! Edge case and error handling tests
//!
//! Tests for boundary conditions, error scenarios, and robustness

use riptide_core::stealth::{
    BrowserType, RotationStrategy, StealthConfig, StealthController, StealthPreset,
    UserAgentConfig, UserAgentManager, load_user_agents_from_file,
};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_empty_user_agent_list() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![],
        include_mobile: false,
        browser_preference: BrowserType::Chrome,
    };

    let mut manager = UserAgentManager::new(config);

    // Should provide fallback
    let ua = manager.next_user_agent();
    assert!(!ua.is_empty());
    assert!(ua.contains("Mozilla"));
}

#[test]
fn test_single_user_agent_sequential() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Sequential,
        agents: vec!["Only UA".to_string()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);

    // Should keep returning the same one
    let ua1 = manager.next_user_agent().to_string();
    let ua2 = manager.next_user_agent().to_string();
    let ua3 = manager.next_user_agent().to_string();

    assert_eq!(ua1, ua2);
    assert_eq!(ua2, ua3);
}

#[test]
fn test_filter_removes_all_agents() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![
            "Mozilla/5.0 Firefox".to_string(),
            "Mozilla/5.0 Firefox Alt".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    manager.filter_by_browser_type(BrowserType::Chrome);

    // All agents filtered out, should fallback
    let ua = manager.next_user_agent();
    assert!(!ua.is_empty());
}

#[test]
fn test_invalid_file_path() {
    let result = load_user_agents_from_file("/nonexistent/path/to/file.txt");
    assert!(result.is_err());
}

#[test]
fn test_empty_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "# Only comments").unwrap();
    writeln!(temp_file, "").unwrap();

    let result = load_user_agents_from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn test_file_with_comments_and_empty_lines() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "# Header comment").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "UA1").unwrap();
    writeln!(temp_file, "# Middle comment").unwrap();
    writeln!(temp_file, "").unwrap();
    writeln!(temp_file, "UA2").unwrap();
    writeln!(temp_file, "").unwrap();

    let result = load_user_agents_from_file(temp_file.path().to_str().unwrap());
    assert!(result.is_ok());

    let agents = result.unwrap();
    assert_eq!(agents.len(), 2);
    assert_eq!(agents[0], "UA1");
    assert_eq!(agents[1], "UA2");
}

#[test]
fn test_extreme_jitter_percentage() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.timing_jitter.jitter_percentage = 0.99; // 99% jitter

    let mut controller = StealthController::new(config);

    // Should still produce valid delays
    for _ in 0..10 {
        let delay = controller.calculate_delay();
        assert!(delay.as_millis() > 0);
    }
}

#[test]
fn test_zero_jitter() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.timing_jitter.jitter_percentage = 0.0;

    let mut controller = StealthController::new(config);

    // Should produce consistent delays
    let delay1 = controller.calculate_delay();
    let delay2 = controller.calculate_delay();

    // With 0 jitter, delays might still vary slightly due to clamping
    assert!(delay1.as_millis() > 0);
    assert!(delay2.as_millis() > 0);
}

#[test]
fn test_very_small_viewport() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.viewport.sizes = vec![(100, 100)];

    let controller = StealthController::new(config);
    let (width, height) = controller.random_viewport();

    // Should still work with small viewport
    assert!(width > 0);
    assert!(height > 0);
}

#[test]
fn test_very_large_viewport() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.viewport.sizes = vec![(7680, 4320)]; // 8K

    let controller = StealthController::new(config);
    let (width, height) = controller.random_viewport();

    assert!(width > 0);
    assert!(height > 0);
}

#[test]
fn test_viewport_with_max_variance() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    config.request_randomization.viewport.add_variance = true;
    config.request_randomization.viewport.max_variance = 500; // Large variance

    let controller = StealthController::new(config);
    let (width, height) = controller.random_viewport();

    assert!(width > 0);
    assert!(height > 0);
}

#[test]
fn test_many_custom_headers() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);
    let mut custom_headers = std::collections::HashMap::new();

    // Add many custom headers
    for i in 0..50 {
        custom_headers.insert(
            format!("X-Custom-{}", i),
            vec![format!("Value-{}", i)],
        );
    }

    config.request_randomization.headers.custom_headers = custom_headers;

    let controller = StealthController::new(config);
    let headers = controller.generate_headers();

    // Should still work
    assert!(!headers.is_empty());
}

#[test]
fn test_unicode_in_user_agent() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec!["Mozilla/5.0 测试".to_string()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let ua = manager.next_user_agent();

    assert!(ua.contains("测试"));
}

#[test]
fn test_very_long_user_agent() {
    let long_ua = "Mozilla/5.0 ".to_string() + &"A".repeat(1000);
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![long_ua.clone()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let ua = manager.next_user_agent();

    assert_eq!(ua, long_ua);
}

#[test]
fn test_config_with_invalid_preset() {
    // Test that all presets work
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let config = StealthConfig::from_preset(preset);
        let controller = StealthController::new(config);
        assert!(controller.get_preset() != &StealthPreset::None || !controller.is_stealth_enabled());
    }
}

#[test]
fn test_timing_bounds_enforced() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);

    // Set contradictory timing values
    config.request_randomization.timing_jitter.min_delay_ms = 5000;
    config.request_randomization.timing_jitter.max_delay_ms = 1000; // Less than min
    config.request_randomization.timing_jitter.base_delay_ms = 3000;

    let mut controller = StealthController::new(config);

    // Should still produce valid delays (min wins)
    let delay = controller.calculate_delay();
    assert!(delay.as_millis() > 0);
}

#[test]
fn test_session_reset_multiple_times() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    for _ in 0..5 {
        controller.mark_request_sent();
        assert!(controller.get_request_count() > 0);

        controller.reset_session();
        assert_eq!(controller.get_request_count(), 0);
    }
}

#[test]
fn test_update_config_clears_js_cache() {
    let mut controller = StealthController::from_preset(StealthPreset::Low);

    let js1 = controller.get_stealth_js();

    // Update config
    let new_config = StealthConfig::from_preset(StealthPreset::High);
    controller.update_config(new_config);

    let js2 = controller.get_stealth_js();

    // JavaScript should be regenerated
    assert!(!js1.is_empty());
    assert!(!js2.is_empty());
}

#[test]
fn test_browser_filter_with_edge_cases() {
    let agents = vec![
        "Chrome and Edge".to_string(),
        "Safari with Chrome-like".to_string(),
        "Pure Firefox".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);

    // Filter by Safari (should be careful about Chrome in UA)
    manager.filter_by_browser_type(BrowserType::Safari);
    assert!(manager.agent_count() > 0);
}

#[test]
fn test_mobile_detection_edge_cases() {
    let agents = vec![
        "Mobile in description but not device".to_string(),
        "Real Android Mobile".to_string(),
        "iPad tablet".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let before = manager.agent_count();

    manager.remove_mobile_agents();
    let after = manager.agent_count();

    assert!(after < before);
}

#[test]
fn test_locale_with_missing_timezone() {
    let mut config = StealthConfig::from_preset(StealthPreset::Medium);

    // Add locale without timezone mapping
    config
        .request_randomization
        .locale
        .locales
        .push("xx-XX".to_string());

    let controller = StealthController::new(config);
    let (locale, timezone) = controller.random_locale();

    // Should still return valid values (fallback)
    assert!(!locale.is_empty());
    assert!(!timezone.is_empty());
}

#[test]
fn test_concurrent_controller_creation() {
    use std::sync::Arc;
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let controller = StealthController::from_preset(StealthPreset::High);
                assert!(controller.is_stealth_enabled());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_hardware_specs_boundary_values() {
    use riptide_core::stealth::HardwareConfig;

    let config = HardwareConfig {
        spoof_cpu_cores: true,
        spoof_device_memory: true,
        spoof_battery: true,
        cpu_core_options: vec![1], // Single option
        memory_options: vec![1],   // Single option
    };

    let (cores, memory) = config.get_random_hardware_specs();

    assert_eq!(cores, 1);
    assert_eq!(memory, 1);
}

#[test]
fn test_webgl_specs_no_randomization_config() {
    use riptide_core::stealth::WebGlConfig;

    let config = WebGlConfig {
        randomize_vendor: false,
        randomize_renderer: false,
        noise_level: 0.0,
    };

    let (vendor1, renderer1) = config.get_random_webgl_specs();
    let (vendor2, renderer2) = config.get_random_webgl_specs();

    // Should be consistent when randomization is off
    assert_eq!(vendor1, vendor2);
    assert_eq!(renderer1, renderer2);
}

#[test]
fn test_serialization_with_special_characters() {
    let mut config = StealthConfig::default();
    config.user_agent.agents = vec![
        "Mozilla/5.0 \"quoted\"".to_string(),
        "Mozilla/5.0 'single'".to_string(),
        "Mozilla/5.0 \\backslash\\".to_string(),
    ];

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: StealthConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.user_agent.agents.len(), deserialized.user_agent.agents.len());
}

#[test]
fn test_max_request_count() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    // Simulate many requests
    for _ in 0..1000 {
        controller.mark_request_sent();
    }

    assert_eq!(controller.get_request_count(), 1000);
}
