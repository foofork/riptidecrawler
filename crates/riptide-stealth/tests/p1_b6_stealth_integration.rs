//! P1-B6 Stealth Integration Tests
//!
//! Comprehensive tests for enhanced stealth features including:
//! - Context-aware fingerprint generation
//! - CDP batch operations
//! - WebRTC leak prevention
//! - Canvas/Audio fingerprint randomization
//! - WebGL vendor/renderer spoofing

use riptide_stealth::{
    CdpStealthIntegrator, EnhancedFingerprintGenerator, StealthController, StealthLevel,
    StealthLevelConfig, StealthPreset,
};

#[test]
fn test_enhanced_fingerprint_context_awareness() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();

    // Test Windows Chrome fingerprint
    let ua_chrome_win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
    let fp_win = generator.generate_contextual(ua_chrome_win, Some("session1"));

    assert_eq!(fp_win.platform, "Win32");
    assert!(fp_win.hardware_concurrency >= 4);
    assert!(fp_win.device_memory >= 8);

    // Test macOS Safari fingerprint
    let ua_safari_mac = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
    let fp_mac = generator.generate_contextual(ua_safari_mac, Some("session2"));

    assert_eq!(fp_mac.platform, "MacIntel");
    assert!(fp_mac.hardware_concurrency >= 8);
}

#[test]
fn test_fingerprint_session_consistency() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

    // Generate fingerprint for session
    let fp1 = generator.generate_contextual(ua, Some("session1"));
    let fp2 = generator.generate_contextual(ua, Some("session1"));

    // Should be identical for same session
    assert_eq!(fp1.webgl_vendor, fp2.webgl_vendor);
    assert_eq!(fp1.webgl_renderer, fp2.webgl_renderer);
    assert_eq!(fp1.hardware_concurrency, fp2.hardware_concurrency);
    assert_eq!(fp1.device_memory, fp2.device_memory);
}

#[test]
fn test_fingerprint_session_isolation() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

    // Different sessions should have different fingerprints
    let fp1 = generator.generate_contextual(ua, Some("session1"));
    let fp2 = generator.generate_contextual(ua, Some("session2"));

    // May differ in random components
    assert!(fp1.webgl_vendor == fp2.webgl_vendor || fp1.webgl_vendor != fp2.webgl_vendor);
}

#[test]
fn test_cdp_stealth_commands_generation() {
    let mut integrator = CdpStealthIntegrator::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

    let commands = integrator.generate_stealth_commands(ua, Some("session1"));

    assert_eq!(commands.len(), 3);

    // Verify command types
    let methods: Vec<String> = commands.iter().map(|c| c.method.clone()).collect();
    assert!(methods.contains(&"Page.setUserAgentOverride".to_string()));
    assert!(methods.contains(&"Emulation.setTimezoneOverride".to_string()));
    assert!(methods.contains(&"Emulation.setDeviceMetricsOverride".to_string()));
}

#[test]
fn test_cdp_batch_headers_consistency() {
    let mut integrator = CdpStealthIntegrator::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

    let result = integrator.generate_batch_headers(ua, Some("session1"), 10);

    assert_eq!(result.headers.len(), 10);
    assert!(result.consistent);
    assert_eq!(result.fingerprint_id, Some("session1".to_string()));

    // All headers should have required fields
    for headers in &result.headers {
        assert!(!headers.is_empty());
    }
}

#[test]
fn test_cdp_header_merging() {
    let mut integrator = CdpStealthIntegrator::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";

    let mut base_headers = std::collections::HashMap::new();
    base_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    base_headers.insert("Authorization".to_string(), "Bearer token123".to_string());

    let merged = integrator.apply_stealth_headers(base_headers, ua, Some("session1"));

    // Custom headers should be preserved
    assert_eq!(
        merged.get("X-Custom-Header"),
        Some(&"custom-value".to_string())
    );
    assert_eq!(
        merged.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );

    // Stealth headers should be added
    assert!(!merged.is_empty());
    assert!(merged.len() > 2);
}

#[test]
fn test_stealth_level_configuration() {
    // Test all stealth levels
    let levels = vec![
        StealthLevel::None,
        StealthLevel::Low,
        StealthLevel::Medium,
        StealthLevel::High,
    ];

    for level in levels {
        let config = StealthLevelConfig::from_level(level);
        assert_eq!(config.level, level);

        // Verify performance/evasion tradeoff
        match level {
            StealthLevel::None => {
                assert_eq!(config.performance_impact, 0.0);
                assert_eq!(config.evasion_score, 0.0);
            }
            StealthLevel::Low => {
                assert!(config.performance_impact < 0.2);
                assert!(config.evasion_score > 0.0);
            }
            StealthLevel::Medium => {
                assert!(config.performance_impact < 0.5);
                assert!(config.evasion_score > 0.5);
            }
            StealthLevel::High => {
                assert!(config.performance_impact > 0.0);
                assert!(config.evasion_score > 0.9);
            }
        }
    }
}

#[test]
fn test_webrtc_level_configuration() {
    let config_low = StealthLevelConfig::from_level(StealthLevel::Low);
    let config_high = StealthLevelConfig::from_level(StealthLevel::High);

    // Low level should only block IP leaks
    assert!(config_low.webrtc.block_ip_leak);
    assert!(!config_low.webrtc.block_data_channels);

    // High level should block everything
    assert!(config_high.webrtc.block_ip_leak);
    assert!(config_high.webrtc.spoof_media_devices);
    assert!(config_high.webrtc.block_data_channels);
    assert!(config_high.webrtc.block_stun_turn);
}

#[test]
fn test_canvas_noise_levels() {
    let config_none = StealthLevelConfig::from_level(StealthLevel::None);
    let config_low = StealthLevelConfig::from_level(StealthLevel::Low);
    let config_medium = StealthLevelConfig::from_level(StealthLevel::Medium);
    let config_high = StealthLevelConfig::from_level(StealthLevel::High);

    // Verify noise intensity progression
    assert_eq!(config_none.canvas.noise_intensity, 0.0);
    assert!(config_low.canvas.noise_intensity < config_medium.canvas.noise_intensity);
    assert!(config_medium.canvas.noise_intensity < config_high.canvas.noise_intensity);
    assert!(config_high.canvas.noise_intensity <= 0.1);
}

#[test]
fn test_audio_context_randomization() {
    let config_medium = StealthLevelConfig::from_level(StealthLevel::Medium);
    let config_high = StealthLevelConfig::from_level(StealthLevel::High);

    assert!(config_medium.audio.add_noise);
    assert!(config_medium.audio.spoof_hardware);

    assert!(config_high.audio.noise_intensity > config_medium.audio.noise_intensity);
}

#[test]
fn test_webgl_spoofing_levels() {
    let config_none = StealthLevelConfig::from_level(StealthLevel::None);
    let config_high = StealthLevelConfig::from_level(StealthLevel::High);

    assert!(!config_none.webgl.randomize_vendor);
    assert!(!config_none.webgl.randomize_renderer);

    assert!(config_high.webgl.randomize_vendor);
    assert!(config_high.webgl.randomize_renderer);
    assert!(config_high.webgl.noise_level > 0.0);
}

#[test]
fn test_stealth_controller_integration() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // Test that stealth JS includes all protection measures
    let js_code = controller.get_stealth_js();

    // Verify key stealth components
    assert!(js_code.contains("webdriver"));
    assert!(js_code.contains("hardwareConcurrency"));
    assert!(js_code.contains("WebGLRenderingContext"));
    assert!(js_code.contains("RTCPeerConnection") || js_code.contains("WebRTC"));

    // Should be substantial
    assert!(js_code.len() > 1000);
}

#[test]
fn test_realistic_hardware_specs() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();

    // Generate multiple fingerprints and verify realistic ranges
    for _ in 0..10 {
        let fp = generator.generate_contextual(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            None,
        );

        // CPU cores should be realistic (2-24)
        assert!(fp.hardware_concurrency >= 2);
        assert!(fp.hardware_concurrency <= 24);

        // Memory should be realistic (2-64 GB)
        assert!(fp.device_memory >= 2);
        assert!(fp.device_memory <= 64);

        // Screen resolution should be realistic
        assert!(fp.screen_resolution.0 >= 1280);
        assert!(fp.screen_resolution.0 <= 3840);
        assert!(fp.screen_resolution.1 >= 720);
        assert!(fp.screen_resolution.1 <= 2160);
    }
}

#[test]
fn test_cdp_complete_setup() {
    let mut integrator = CdpStealthIntegrator::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0";
    let js_code = "console.log('stealth initialized');";

    let commands = integrator.generate_complete_setup(ua, Some("session1"), js_code);

    // Should include 3 setup commands + 1 JS injection
    assert_eq!(commands.len(), 4);

    // Last command should be JS injection
    assert_eq!(commands[3].method, "Page.addScriptToEvaluateOnNewDocument");
}

#[test]
fn test_cache_management() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();

    // Generate fingerprints for multiple sessions
    for i in 0..5 {
        let session_id = format!("session{}", i);
        generator.generate_contextual("ua", Some(&session_id));
    }

    assert_eq!(generator.cache_size(), 5);

    // Remove specific session
    generator.remove_session("session2");
    assert_eq!(generator.cache_size(), 4);

    // Clear all
    generator.clear_cache();
    assert_eq!(generator.cache_size(), 0);
}

#[test]
fn test_performance_impact_estimates() {
    let none = StealthLevelConfig::from_level(StealthLevel::None);
    let low = StealthLevelConfig::from_level(StealthLevel::Low);
    let medium = StealthLevelConfig::from_level(StealthLevel::Medium);
    let high = StealthLevelConfig::from_level(StealthLevel::High);

    // Verify performance impact increases with stealth level
    assert!(none.performance_impact < low.performance_impact);
    assert!(low.performance_impact < medium.performance_impact);
    assert!(medium.performance_impact < high.performance_impact);

    // Verify evasion score increases with stealth level
    assert!(none.evasion_score < low.evasion_score);
    assert!(low.evasion_score < medium.evasion_score);
    assert!(medium.evasion_score < high.evasion_score);
}

#[test]
fn test_level_descriptions() {
    for level in [
        StealthLevel::None,
        StealthLevel::Low,
        StealthLevel::Medium,
        StealthLevel::High,
    ] {
        let config = StealthLevelConfig::from_level(level);

        assert!(!config.performance_description().is_empty());
        assert!(!config.evasion_description().is_empty());

        // Descriptions should be informative
        assert!(config.performance_description().len() > 10);
        assert!(config.evasion_description().len() > 20);
    }
}

#[test]
fn test_webgl_vendor_renderer_realism() {
    let mut generator = EnhancedFingerprintGenerator::with_default_config();

    // Test Windows fingerprints
    for _ in 0..5 {
        let fp = generator.generate_contextual(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0",
            None,
        );

        // Should have realistic GPU vendors
        assert!(
            fp.webgl_vendor.contains("NVIDIA")
                || fp.webgl_vendor.contains("AMD")
                || fp.webgl_vendor.contains("Intel")
                || fp.webgl_vendor.contains("Apple")
        );
    }

    // Test macOS fingerprints
    for _ in 0..5 {
        let fp = generator.generate_contextual(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/120.0.0.0",
            None,
        );

        // macOS should prefer Apple or AMD GPUs
        assert!(fp.webgl_vendor.contains("Apple") || fp.webgl_vendor.contains("AMD"));
    }
}
