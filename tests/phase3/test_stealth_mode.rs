use riptide_core::stealth::{
    StealthConfig, StealthController, StealthPreset, RotationStrategy, load_user_agents_from_file,
    LocaleStrategy
};
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[cfg(test)]
mod stealth_tests {
    use super::*;

    #[test]
    fn test_stealth_preset_configuration() {
        // Test None preset
        let none_config = StealthConfig::from_preset(StealthPreset::None);
        assert_eq!(none_config.preset, StealthPreset::None);
        assert!(!none_config.fingerprinting.cdp_stealth.disable_automation_controlled);
        assert!(!none_config.fingerprinting.cdp_stealth.override_webdriver);

        // Test Low preset
        let low_config = StealthConfig::from_preset(StealthPreset::Low);
        assert_eq!(low_config.preset, StealthPreset::Low);
        assert!(low_config.fingerprinting.cdp_stealth.disable_automation_controlled);
        assert!(low_config.fingerprinting.cdp_stealth.override_webdriver);
        assert_eq!(low_config.user_agent.strategy, RotationStrategy::Sequential);

        // Test Medium preset (default)
        let medium_config = StealthConfig::from_preset(StealthPreset::Medium);
        assert_eq!(medium_config.preset, StealthPreset::Medium);
        assert!(medium_config.fingerprinting.cdp_stealth.disable_automation_controlled);

        // Test High preset
        let high_config = StealthConfig::from_preset(StealthPreset::High);
        assert_eq!(high_config.preset, StealthPreset::High);
        assert_eq!(high_config.user_agent.strategy, RotationStrategy::Random);
        assert_eq!(high_config.request_randomization.timing_jitter.jitter_percentage, 0.4);
        assert_eq!(high_config.fingerprinting.webgl.noise_level, 0.2);
    }

    #[test]
    fn test_user_agent_rotation_strategies() {
        // Test Sequential rotation
        let mut config = StealthConfig::default();
        config.user_agent.strategy = RotationStrategy::Sequential;
        config.user_agent.agents = vec![
            "Agent1".to_string(),
            "Agent2".to_string(),
            "Agent3".to_string(),
        ];

        let mut controller = StealthController::new(config);

        // Sequential should increment through agents
        let ua1 = controller.next_user_agent().to_string();
        let ua2 = controller.next_user_agent().to_string();
        let ua3 = controller.next_user_agent().to_string();
        let ua4 = controller.next_user_agent().to_string(); // Should wrap around

        assert_eq!(ua1, "Agent2"); // First call increments index
        assert_eq!(ua2, "Agent3");
        assert_eq!(ua3, "Agent1");
        assert_eq!(ua4, "Agent2");

        // Test Sticky rotation
        let mut sticky_config = StealthConfig::default();
        sticky_config.user_agent.strategy = RotationStrategy::Sticky;
        sticky_config.user_agent.agents = vec![
            "StickyAgent1".to_string(),
            "StickyAgent2".to_string(),
        ];

        let mut sticky_controller = StealthController::new(sticky_config);

        let sticky_ua1 = sticky_controller.next_user_agent().to_string();
        let sticky_ua2 = sticky_controller.next_user_agent().to_string();

        assert_eq!(sticky_ua1, sticky_ua2); // Should be the same for sticky strategy
    }

    #[test]
    fn test_cdp_flags_generation() {
        let none_controller = StealthController::from_preset(StealthPreset::None);
        let low_controller = StealthController::from_preset(StealthPreset::Low);
        let medium_controller = StealthController::from_preset(StealthPreset::Medium);
        let high_controller = StealthController::from_preset(StealthPreset::High);

        let none_flags = none_controller.get_cdp_flags();
        let low_flags = low_controller.get_cdp_flags();
        let medium_flags = medium_controller.get_cdp_flags();
        let high_flags = high_controller.get_cdp_flags();

        // None should have no flags
        assert!(none_flags.is_empty());

        // Low should have basic flags
        assert!(!low_flags.is_empty());
        assert!(low_flags.contains(&"--disable-blink-features=AutomationControlled".to_string()));
        assert!(low_flags.contains(&"--no-first-run".to_string()));

        // Medium should have additional flags
        assert!(medium_flags.len() > low_flags.len());
        assert!(medium_flags.contains(&"--disable-web-security".to_string()));

        // High should have the most flags
        assert!(high_flags.len() > medium_flags.len());
        assert!(high_flags.contains(&"--disable-extensions".to_string()));
        assert!(high_flags.contains(&"--mute-audio".to_string()));
    }

    #[test]
    fn test_user_agent_file_loading() {
        // Create a temporary file with test user agents
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "Mozilla/5.0 Test Agent 1").unwrap();
        writeln!(temp_file, "").unwrap(); // Empty line
        writeln!(temp_file, "Mozilla/5.0 Test Agent 2").unwrap();
        writeln!(temp_file, "# Another comment").unwrap();
        writeln!(temp_file, "Mozilla/5.0 Test Agent 3").unwrap();

        // Test loading via standalone function
        let agents = load_user_agents_from_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0], "Mozilla/5.0 Test Agent 1");
        assert_eq!(agents[1], "Mozilla/5.0 Test Agent 2");
        assert_eq!(agents[2], "Mozilla/5.0 Test Agent 3");

        // Test loading via config
        let mut config = StealthConfig::default();
        config.ua_file_path = Some(temp_file.path().to_str().unwrap().to_string());
        config.load_user_agents_from_file().unwrap();

        assert_eq!(config.user_agent.agents.len(), 3);
        assert_eq!(config.user_agent.agents[0], "Mozilla/5.0 Test Agent 1");
    }

    #[test]
    fn test_user_agent_file_loading_error_cases() {
        // Test non-existent file
        let result = load_user_agents_from_file("/non/existent/file.txt");
        assert!(result.is_err());

        // Test empty file
        let mut empty_file = NamedTempFile::new().unwrap();
        writeln!(empty_file, "# Only comments").unwrap();
        writeln!(empty_file, "").unwrap();

        let result = load_user_agents_from_file(empty_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_stealth_controller_fallback_user_agent() {
        // Test controller with empty user agent list
        let mut config = StealthConfig::default();
        config.user_agent.agents = vec![];

        let mut controller = StealthController::new(config);
        let fallback_ua = controller.next_user_agent();

        // Should return a fallback user agent
        assert!(!fallback_ua.is_empty());
        assert!(fallback_ua.contains("Chrome"));
    }

    #[test]
    fn test_header_generation() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let headers = controller.generate_headers();

        // Check that required headers are present
        assert!(headers.contains_key("Accept"));
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("Accept-Encoding"));

        // Check that values are not empty
        assert!(!headers.get("Accept").unwrap().is_empty());
        assert!(!headers.get("Accept-Language").unwrap().is_empty());
        assert!(!headers.get("Accept-Encoding").unwrap().is_empty());
    }

    #[test]
    fn test_viewport_randomization() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let (width1, height1) = controller.random_viewport();
        let (width2, height2) = controller.random_viewport();

        // Dimensions should be positive
        assert!(width1 > 0);
        assert!(height1 > 0);
        assert!(width2 > 0);
        assert!(height2 > 0);

        // With variance enabled, dimensions might differ
        // (though they could be the same by chance)
        assert!(width1 >= 1200); // Should be reasonable screen sizes
        assert!(height1 >= 600);
    }

    #[test]
    fn test_request_timing_jitter() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let delay1 = controller.calculate_delay();
        let delay2 = controller.calculate_delay();

        // Delays should be within expected range
        assert!(delay1.as_millis() >= 500); // min_delay_ms
        assert!(delay1.as_millis() <= 3000); // max_delay_ms
        assert!(delay2.as_millis() >= 500);
        assert!(delay2.as_millis() <= 3000);

        // Request count should increment
        assert_eq!(controller.request_count, 2);
    }

    #[test]
    fn test_stealth_config_ua_file_path_default() {
        let config = StealthConfig::default();
        assert_eq!(config.ua_file_path, Some("configs/ua_list.txt".to_string()));
        assert_eq!(config.preset, StealthPreset::Medium);
    }

    #[test]
    fn test_domain_based_rotation() {
        let mut config = StealthConfig::default();
        config.user_agent.strategy = RotationStrategy::DomainBased;
        config.user_agent.agents = vec![
            "Agent1".to_string(),
            "Agent2".to_string(),
            "Agent3".to_string(),
        ];

        let mut controller = StealthController::new(config);

        // Domain-based should be deterministic for same request count
        let ua1 = controller.next_user_agent().to_string();

        // Reset controller and try again
        let mut controller2 = StealthController::new(StealthConfig::from_preset(StealthPreset::Medium));
        controller2.config.user_agent.strategy = RotationStrategy::DomainBased;
        controller2.config.user_agent.agents = vec![
            "Agent1".to_string(),
            "Agent2".to_string(),
            "Agent3".to_string(),
        ];

        let ua2 = controller2.next_user_agent().to_string();

        // Should be deterministic based on request count
        assert_eq!(ua1, ua2);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_stealth_integration_with_real_ua_file() {
        // Test with the actual configs/ua_list.txt if it exists
        if std::path::Path::new("configs/ua_list.txt").exists() {
            let mut config = StealthConfig::default();

            // Should successfully load from the real file
            assert!(config.load_user_agents_from_file().is_ok());
            assert!(!config.user_agent.agents.is_empty());

            // Create controller and test rotation
            let mut controller = StealthController::new(config);
            let ua = controller.next_user_agent();

            assert!(!ua.is_empty());
            assert!(ua.contains("Mozilla"));
        }
    }

    #[test]
    fn test_end_to_end_stealth_workflow() {
        // Test complete stealth workflow
        let mut controller = StealthController::from_preset(StealthPreset::High);

        // Get user agent
        let user_agent = controller.next_user_agent();
        assert!(!user_agent.is_empty());

        // Get CDP flags
        let flags = controller.get_cdp_flags();
        assert!(!flags.is_empty());
        assert!(flags.contains(&"--disable-blink-features=AutomationControlled".to_string()));

        // Generate headers
        let headers = controller.generate_headers();
        assert!(headers.contains_key("Accept"));

        // Get viewport
        let (width, height) = controller.random_viewport();
        assert!(width > 0 && height > 0);

        // Calculate delay
        let delay = controller.calculate_delay();
        assert!(delay.as_millis() > 0);

        // Verify preset
        assert_eq!(*controller.get_preset(), StealthPreset::High);
    }

    #[test]
    fn test_webgl_fingerprinting_config() {
        let config = StealthConfig::from_preset(StealthPreset::High);

        // Test WebGL configuration
        assert!(config.fingerprinting.webgl.randomize_vendor);
        assert!(config.fingerprinting.webgl.randomize_renderer);
        assert_eq!(config.fingerprinting.webgl.noise_level, 0.2);
    }

    #[test]
    fn test_canvas_fingerprinting_config() {
        let config = StealthConfig::from_preset(StealthPreset::High);

        // Test Canvas configuration
        assert!(config.fingerprinting.canvas.add_noise);
        assert_eq!(config.fingerprinting.canvas.noise_intensity, 0.1);
        assert!(!config.fingerprinting.canvas.block_data_extraction); // Default is false
    }

    #[test]
    fn test_audio_fingerprinting_config() {
        let config = StealthConfig::from_preset(StealthPreset::High);

        // Test Audio configuration
        assert!(config.fingerprinting.audio.add_noise);
        assert_eq!(config.fingerprinting.audio.noise_intensity, 0.001);
        assert!(config.fingerprinting.audio.spoof_hardware);
    }

    #[test]
    fn test_webrtc_fingerprinting_config() {
        let config = StealthConfig::from_preset(StealthPreset::High);

        // Test WebRTC configuration
        assert!(config.fingerprinting.webrtc.block_ip_leak);
        assert!(config.fingerprinting.webrtc.spoof_media_devices);
        assert!(!config.fingerprinting.webrtc.disable_data_channels); // Default is false
    }

    #[test]
    fn test_hardware_fingerprinting_config() {
        let config = StealthConfig::from_preset(StealthPreset::High);

        // Test Hardware configuration
        assert!(config.fingerprinting.hardware.spoof_cpu_cores);
        assert!(config.fingerprinting.hardware.spoof_device_memory);
        assert!(config.fingerprinting.hardware.spoof_battery);
        assert!(!config.fingerprinting.hardware.cpu_core_options.is_empty());
        assert!(!config.fingerprinting.hardware.memory_options.is_empty());
    }

    #[test]
    fn test_font_fingerprinting_config() {
        let config = StealthConfig::default();

        // Test Font configuration
        assert!(config.fingerprinting.fonts.limit_fonts);
        assert!(!config.fingerprinting.fonts.standard_fonts.is_empty());
        assert!(config.fingerprinting.fonts.standard_fonts.contains(&"Arial".to_string()));
    }

    #[test]
    fn test_locale_randomization() {
        let config = StealthConfig::default();

        // Test locale configuration
        assert!(!config.request_randomization.locale.locales.is_empty());
        assert!(!config.request_randomization.locale.timezones.is_empty());

        // Verify timezone mapping exists for each locale
        for locale in &config.request_randomization.locale.locales {
            assert!(config.request_randomization.locale.timezones.contains_key(locale));
        }
    }

    #[test]
    fn test_stealth_preset_differences() {
        let none_config = StealthConfig::from_preset(StealthPreset::None);
        let low_config = StealthConfig::from_preset(StealthPreset::Low);
        let medium_config = StealthConfig::from_preset(StealthPreset::Medium);
        let high_config = StealthConfig::from_preset(StealthPreset::High);

        // None should have minimal stealth
        assert!(!none_config.fingerprinting.cdp_stealth.disable_automation_controlled);

        // Low should have basic stealth
        assert!(low_config.fingerprinting.cdp_stealth.disable_automation_controlled);
        assert_eq!(low_config.request_randomization.timing_jitter.jitter_percentage, 0.1);

        // Medium should be default (balanced)
        assert!(medium_config.fingerprinting.cdp_stealth.disable_automation_controlled);

        // High should have maximum stealth
        assert!(high_config.fingerprinting.cdp_stealth.disable_automation_controlled);
        assert_eq!(high_config.request_randomization.timing_jitter.jitter_percentage, 0.4);
        assert_eq!(high_config.fingerprinting.webgl.noise_level, 0.2);
        assert_eq!(high_config.fingerprinting.canvas.noise_intensity, 0.1);
    }

    #[test]
    fn test_stealth_controller_multiple_requests() {
        let mut controller = StealthController::from_preset(StealthPreset::High);

        // Test multiple request workflow
        for i in 0..5 {
            let user_agent = controller.next_user_agent();
            let headers = controller.generate_headers();
            let (width, height) = controller.random_viewport();
            let delay = controller.calculate_delay();

            assert!(!user_agent.is_empty());
            assert!(!headers.is_empty());
            assert!(width > 0 && height > 0);
            assert!(delay.as_millis() > 0);

            // Request count should increment
            assert_eq!(controller.request_count, (i + 1) as u64);
        }
    }

    #[test]
    fn test_comprehensive_cdp_flags() {
        let high_controller = StealthController::from_preset(StealthPreset::High);
        let flags = high_controller.get_cdp_flags();

        // Check for essential stealth flags
        assert!(flags.contains(&"--disable-blink-features=AutomationControlled".to_string()));
        assert!(flags.contains(&"--no-first-run".to_string()));
        assert!(flags.contains(&"--disable-default-apps".to_string()));
        assert!(flags.contains(&"--disable-dev-shm-usage".to_string()));
        assert!(flags.contains(&"--no-sandbox".to_string()));
        assert!(flags.contains(&"--disable-web-security".to_string()));
        assert!(flags.contains(&"--disable-extensions".to_string()));
        assert!(flags.contains(&"--mute-audio".to_string()));
    }
}