//! Comprehensive test suite for stealth module
//!
//! This module contains integration tests and comprehensive test coverage
//! for all stealth functionality to ensure reliability and correctness.

use crate::stealth::*;

#[cfg(test)]
mod stealth_integration_tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_stealth_config_presets() {
        // Test each preset creates valid configuration
        let presets = vec![
            StealthPreset::None,
            StealthPreset::Low,
            StealthPreset::Medium,
            StealthPreset::High,
        ];

        for preset in presets {
            let config = StealthConfig::from_preset(preset.clone());
            assert_eq!(config.preset, preset);

            // Ensure user agents are not empty
            assert!(!config.user_agent.agents.is_empty());

            // Ensure fingerprinting config is valid
            assert!(config.fingerprinting.webgl.noise_level >= 0.0);
            assert!(config.fingerprinting.webgl.noise_level <= 1.0);
            assert!(config.fingerprinting.canvas.noise_intensity >= 0.0);
            assert!(config.fingerprinting.canvas.noise_intensity <= 1.0);
        }
    }

    #[test]
    fn test_stealth_controller_full_workflow() {
        let mut controller = StealthController::from_preset(StealthPreset::High);

        // Test initial state
        assert!(controller.is_stealth_enabled());
        assert_eq!(controller.get_request_count(), 0);

        // Test user agent rotation
        let ua1 = controller.next_user_agent().to_string();
        let ua2 = controller.next_user_agent().to_string();
        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());

        // Test header generation
        let headers = controller.generate_headers();
        assert!(headers.len() >= 3); // At least Accept, Accept-Language, Accept-Encoding

        // Test viewport generation
        let (width, height) = controller.random_viewport();
        assert!(width > 0 && height > 0);

        // Test delay calculation
        let delay = controller.calculate_delay();
        assert!(delay.as_millis() > 0);

        // Test JavaScript generation
        let js_code = controller.get_stealth_js();
        assert!(!js_code.is_empty());
        assert!(js_code.contains("function"));

        // Test request tracking
        controller.mark_request_sent();
        assert!(controller.get_request_count() > 0);
    }

    #[test]
    fn test_user_agent_manager_strategies() {
        // Test Random strategy
        let mut config = UserAgentConfig::default();
        config.strategy = RotationStrategy::Random;
        config.agents = vec!["UA1".to_string(), "UA2".to_string(), "UA3".to_string()];

        let mut manager = UserAgentManager::new(config.clone());
        let agents_seen = (0..10)
            .map(|_| manager.next_user_agent().to_string())
            .collect::<HashSet<_>>();

        // Should see some variety with random strategy
        assert!(agents_seen.len() <= 3);
        assert!(agents_seen.len() >= 1);

        // Test Sequential strategy
        config.strategy = RotationStrategy::Sequential;
        let mut manager = UserAgentManager::new(config.clone());

        let ua1 = manager.next_user_agent().to_string();
        let ua2 = manager.next_user_agent().to_string();
        let ua3 = manager.next_user_agent().to_string();
        let ua4 = manager.next_user_agent().to_string(); // Should wrap around

        assert_ne!(ua1, ua2);
        assert_ne!(ua2, ua3);
        assert_eq!(ua1, ua4); // Should wrap around to first

        // Test Sticky strategy
        config.strategy = RotationStrategy::Sticky;
        let mut manager = UserAgentManager::new(config);

        let ua1 = manager.next_user_agent().to_string();
        let ua2 = manager.next_user_agent().to_string();
        let ua3 = manager.next_user_agent().to_string();

        assert_eq!(ua1, ua2);
        assert_eq!(ua2, ua3);
    }

    #[test]
    fn test_fingerprinting_configs() {
        let config = FingerprintingConfig::default();

        // Test WebGL config
        let (vendor, renderer) = config.webgl.get_random_webgl_specs();
        assert!(!vendor.is_empty());
        assert!(!renderer.is_empty());

        // Test Hardware config
        let (cpu_cores, memory) = config.hardware.get_random_hardware_specs();
        assert!(cpu_cores > 0);
        assert!(memory > 0);
        assert!(config.hardware.cpu_core_options.contains(&cpu_cores));
        assert!(config.hardware.memory_options.contains(&memory));

        // Test Canvas config
        assert!(config.canvas.noise_intensity >= 0.0);
        assert!(config.canvas.noise_intensity <= 1.0);

        // Test Audio config
        assert!(config.audio.noise_intensity >= 0.0);
        assert!(config.audio.noise_intensity <= 1.0);
    }

    #[test]
    fn test_javascript_injector_comprehensive() {
        let hardware_config = HardwareConfig::default();
        let webgl_config = WebGlConfig::default();
        let canvas_config = CanvasConfig::default();

        // Test with different locale strategies
        let strategies = vec![
            LocaleStrategy::Random,
            LocaleStrategy::Fixed("en-US".to_string()),
            LocaleStrategy::Fixed("de-DE".to_string()),
        ];

        for strategy in strategies {
            let injector =
                JavaScriptInjector::new(&hardware_config, &webgl_config, &canvas_config, &strategy);

            let js_code = injector.generate_stealth_js();

            // Verify essential components
            assert!(js_code.contains("'webdriver'"));
            assert!(js_code.contains("hardwareConcurrency"));
            assert!(js_code.contains("deviceMemory"));
            assert!(js_code.contains("WebGLRenderingContext"));
            assert!(js_code.contains("getParameter"));
            assert!(js_code.contains("plugins"));
            assert!(js_code.contains("languages"));

            // Verify structure
            assert!(js_code.contains("function"));
            assert!(js_code.starts_with('\n'));
            assert!(js_code.len() > 1000); // Should be substantial
        }
    }

    #[test]
    fn test_request_randomization() {
        let config = RequestRandomization::default();

        // Test header variations are not empty
        assert!(!config.headers.accept_variations.is_empty());
        assert!(!config.headers.accept_language_variations.is_empty());
        assert!(!config.headers.accept_encoding_variations.is_empty());

        // Test timing jitter is reasonable
        assert!(config.timing_jitter.base_delay_ms > 0);
        assert!(config.timing_jitter.jitter_percentage >= 0.0);
        assert!(config.timing_jitter.jitter_percentage <= 1.0);
        assert!(config.timing_jitter.min_delay_ms <= config.timing_jitter.max_delay_ms);

        // Test viewport sizes are reasonable
        assert!(!config.viewport.sizes.is_empty());
        for (width, height) in &config.viewport.sizes {
            assert!(*width > 0);
            assert!(*height > 0);
            assert!(*width >= 1000); // Reasonable minimum
            assert!(*height >= 600); // Reasonable minimum
        }

        // Test locale configuration
        assert!(!config.locale.locales.is_empty());
        assert!(!config.locale.timezones.is_empty());

        // Ensure each locale has a timezone
        for locale in &config.locale.locales {
            assert!(config.locale.timezones.contains_key(locale));
        }
    }

    #[test]
    fn test_timing_configuration() {
        let config = TimingConfig::default();

        // Test default timing is reasonable
        assert!(config.default_timing.min_delay_ms > 0);
        assert!(config.default_timing.min_delay_ms <= config.default_timing.max_delay_ms);
        assert!(config.default_timing.burst_size > 0);

        if let Some(rpm_limit) = config.default_timing.rpm_limit {
            assert!(rpm_limit > 0);
            assert!(rpm_limit <= 3600); // Max 1 request per second seems reasonable
        }
    }

    #[test]
    fn test_error_handling() {
        // Test invalid user agent file path
        let result = load_user_agents_from_file("nonexistent_file.txt");
        assert!(result.is_err());

        // Test empty user agent list handling
        let mut config = UserAgentConfig::default();
        config.agents.clear();

        let mut manager = UserAgentManager::new(config);
        let ua = manager.next_user_agent();
        // Should fall back to default user agent
        assert!(!ua.is_empty());
        assert!(ua.contains("Mozilla"));
    }

    #[test]
    fn test_stealth_controller_configuration_updates() {
        let mut controller = StealthController::from_preset(StealthPreset::Low);
        assert_eq!(controller.get_preset(), &StealthPreset::Low);

        let new_config = StealthConfig::from_preset(StealthPreset::High);
        controller.update_config(new_config);
        assert_eq!(controller.get_preset(), &StealthPreset::High);

        // Test session reset
        controller.mark_request_sent();
        assert!(controller.get_request_count() > 0);

        controller.reset_session();
        assert_eq!(controller.get_request_count(), 0);
    }

    #[test]
    fn test_browser_type_filtering() {
        let test_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edge/120.0.0.0".to_string(),
        ];

        let mut config = UserAgentConfig::default();
        config.agents = test_agents;

        let mut manager = UserAgentManager::new(config);

        // Test Chrome filtering
        manager.filter_by_browser_type(BrowserType::Chrome);
        assert!(manager.agent_count() > 0);

        // All remaining should be Chrome
        for _ in 0..10 {
            let ua = manager.next_user_agent();
            assert!(ua.contains("Chrome"));
            assert!(!ua.contains("Firefox"));
        }
    }

    #[test]
    fn test_mobile_agent_filtering() {
        let test_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
            "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15"
                .to_string(),
            "Mozilla/5.0 (Android 12; Mobile; rv:121.0) Gecko/121.0 Firefox/121.0".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
        ];

        let mut config = UserAgentConfig::default();
        config.agents = test_agents;

        let mut manager = UserAgentManager::new(config);
        let original_count = manager.agent_count();

        manager.remove_mobile_agents();
        assert!(manager.agent_count() < original_count);

        // Verify no mobile agents remain
        for _ in 0..10 {
            let ua = manager.next_user_agent();
            assert!(!ua.contains("iPhone"));
            assert!(!ua.contains("Android"));
            assert!(!ua.contains("Mobile"));
        }
    }

    #[test]
    fn test_cdp_flags_generation() {
        let presets = vec![
            StealthPreset::None,
            StealthPreset::Low,
            StealthPreset::Medium,
            StealthPreset::High,
        ];

        for preset in presets {
            let config = StealthConfig::from_preset(preset.clone());
            let flags = config.get_cdp_flags();

            match preset {
                StealthPreset::None => {
                    assert!(flags.is_empty());
                }
                StealthPreset::Low => {
                    assert!(!flags.is_empty());
                    assert!(flags.iter().any(|f| f.contains("AutomationControlled")));
                }
                StealthPreset::Medium | StealthPreset::High => {
                    assert!(!flags.is_empty());
                    assert!(flags.iter().any(|f| f.contains("no-sandbox")));
                    assert!(flags.iter().any(|f| f.contains("disable-web-security")));
                }
            }
        }
    }

    #[test]
    fn test_performance_and_memory_usage() {
        // Test that creating many controllers doesn't consume excessive memory
        let controllers: Vec<_> = (0..100)
            .map(|_| StealthController::from_preset(StealthPreset::Medium))
            .collect();

        assert_eq!(controllers.len(), 100);

        // Test that generating many JS injections is reasonably fast
        let start = std::time::Instant::now();
        let mut controller = StealthController::from_preset(StealthPreset::High);

        for _ in 0..10 {
            let js = controller.get_stealth_js();
            assert!(!js.is_empty());
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000); // Should complete in under 1 second
    }
}
