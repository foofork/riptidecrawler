//! Fingerprinting countermeasure tests
//!
//! Tests for WebGL, Canvas, Audio, Hardware, WebRTC, and Font fingerprinting evasion

use riptide_core::stealth::{
    AudioConfig, CanvasConfig, FingerprintingConfig, HardwareConfig, StealthConfig,
    StealthController, StealthPreset, WebGlConfig, WebRtcConfig,
};
use std::collections::HashSet;

#[test]
fn test_webgl_randomization_provides_variety() {
    let config = WebGlConfig::default();

    let mut vendors = HashSet::new();
    let mut renderers = HashSet::new();

    // Get 20 random WebGL specs
    for _ in 0..20 {
        let (vendor, renderer) = config.get_random_webgl_specs();
        vendors.insert(vendor);
        renderers.insert(renderer);
    }

    // Should have multiple options
    assert!(vendors.len() > 1, "WebGL vendor should vary");
    assert!(renderers.len() > 1, "WebGL renderer should vary");
}

#[test]
fn test_webgl_vendor_renderer_pairs_realistic() {
    let config = WebGlConfig::default();

    for _ in 0..50 {
        let (vendor, renderer) = config.get_random_webgl_specs();

        // Verify format
        assert!(!vendor.is_empty());
        assert!(!renderer.is_empty());

        // Verify realistic pairs
        if vendor.contains("Intel") {
            assert!(renderer.contains("Intel"));
        } else if vendor.contains("NVIDIA") {
            assert!(renderer.contains("NVIDIA") || renderer.contains("GeForce"));
        } else if vendor.contains("ATI") {
            assert!(renderer.contains("AMD") || renderer.contains("Radeon"));
        }
    }
}

#[test]
fn test_webgl_noise_level_bounds() {
    let low_config = WebGlConfig {
        randomize_vendor: true,
        randomize_renderer: true,
        noise_level: 0.1,
    };

    let high_config = WebGlConfig {
        randomize_vendor: true,
        randomize_renderer: true,
        noise_level: 0.9,
    };

    assert!(low_config.noise_level >= 0.0 && low_config.noise_level <= 1.0);
    assert!(high_config.noise_level >= 0.0 && high_config.noise_level <= 1.0);
}

#[test]
fn test_webgl_randomization_disabled() {
    let config = WebGlConfig {
        randomize_vendor: false,
        randomize_renderer: false,
        noise_level: 0.0,
    };

    let (vendor1, renderer1) = config.get_random_webgl_specs();
    let (vendor2, renderer2) = config.get_random_webgl_specs();

    // Should return consistent defaults when randomization is off
    assert_eq!(vendor1, vendor2);
    assert_eq!(renderer1, renderer2);
    assert_eq!(vendor1, "Intel Inc.");
}

#[test]
fn test_canvas_noise_configuration() {
    let no_noise = CanvasConfig {
        add_noise: false,
        noise_intensity: 0.0,
        block_data_extraction: false,
    };

    let with_noise = CanvasConfig {
        add_noise: true,
        noise_intensity: 0.05,
        block_data_extraction: false,
    };

    assert!(!no_noise.add_noise);
    assert!(with_noise.add_noise);
    assert!(with_noise.noise_intensity > 0.0);
    assert!(with_noise.noise_intensity <= 1.0);
}

#[test]
fn test_canvas_noise_intensity_bounds() {
    let configs = vec![
        CanvasConfig {
            add_noise: true,
            noise_intensity: 0.01,
            block_data_extraction: false,
        },
        CanvasConfig {
            add_noise: true,
            noise_intensity: 0.05,
            block_data_extraction: false,
        },
        CanvasConfig {
            add_noise: true,
            noise_intensity: 0.1,
            block_data_extraction: false,
        },
    ];

    for config in configs {
        assert!(config.noise_intensity >= 0.0);
        assert!(config.noise_intensity <= 1.0);
    }
}

#[test]
fn test_audio_fingerprinting_config() {
    let config = AudioConfig::default();

    assert!(config.add_noise);
    assert!(!config.block_extraction);
    assert!(config.noise_intensity > 0.0 && config.noise_intensity <= 1.0);
    assert!(config.spoof_hardware);
}

#[test]
fn test_audio_noise_is_subtle() {
    let config = AudioConfig::default();

    // Audio noise should be very subtle (< 0.01)
    assert!(
        config.noise_intensity < 0.01,
        "Audio noise should be subtle: {}",
        config.noise_intensity
    );
}

#[test]
fn test_hardware_specs_randomization() {
    let config = HardwareConfig::default();

    let mut core_counts = HashSet::new();
    let mut memory_sizes = HashSet::new();

    // Get 30 random hardware specs
    for _ in 0..30 {
        let (cores, memory) = config.get_random_hardware_specs();
        core_counts.insert(cores);
        memory_sizes.insert(memory);

        // Verify values are realistic
        assert!(cores >= 2 && cores <= 16);
        assert!(memory >= 2 && memory <= 16);
    }

    // Should have variety
    assert!(core_counts.len() > 1);
    assert!(memory_sizes.len() > 1);
}

#[test]
fn test_hardware_specs_realistic_values() {
    let config = HardwareConfig::default();

    let valid_cores = vec![2, 4, 6, 8, 12, 16];
    let valid_memory = vec![2, 4, 8, 16];

    for _ in 0..20 {
        let (cores, memory) = config.get_random_hardware_specs();

        assert!(
            valid_cores.contains(&cores),
            "CPU cores should be realistic: {}",
            cores
        );
        assert!(
            valid_memory.contains(&memory),
            "Memory should be realistic: {} GB",
            memory
        );
    }
}

#[test]
fn test_hardware_spoofing_disabled() {
    let config = HardwareConfig {
        spoof_cpu_cores: false,
        spoof_device_memory: false,
        spoof_battery: false,
        cpu_core_options: vec![4],
        memory_options: vec![8],
    };

    let (cores, memory) = config.get_random_hardware_specs();

    // Should return defaults when spoofing is off
    assert_eq!(cores, 4);
    assert_eq!(memory, 8);
}

#[test]
fn test_webrtc_leak_prevention() {
    let config = WebRtcConfig::default();

    assert!(config.block_ip_leak);
    assert!(config.spoof_media_devices);
    assert!(!config.disable_data_channels); // Data channels usually needed
}

#[test]
fn test_webrtc_custom_config() {
    let strict_config = WebRtcConfig {
        block_ip_leak: true,
        spoof_media_devices: true,
        disable_data_channels: true,
    };

    assert!(strict_config.block_ip_leak);
    assert!(strict_config.spoof_media_devices);
    assert!(strict_config.disable_data_channels);
}

#[test]
fn test_font_limiting() {
    let config = FingerprintingConfig::default();

    assert!(config.fonts.limit_fonts);
    assert!(!config.fonts.standard_fonts.is_empty());

    // Should have common fonts
    let fonts = &config.fonts.standard_fonts;
    assert!(fonts.contains(&"Arial".to_string()));
    assert!(fonts.contains(&"Times New Roman".to_string()));
    assert!(fonts.contains(&"Helvetica".to_string()));
}

#[test]
fn test_cdp_stealth_flags() {
    let config = FingerprintingConfig::default();

    // All CDP stealth measures should be enabled by default
    assert!(config.cdp_stealth.disable_automation_controlled);
    assert!(config.cdp_stealth.override_webdriver);
    assert!(config.cdp_stealth.override_permissions);
    assert!(config.cdp_stealth.override_plugins);
    assert!(config.cdp_stealth.override_chrome);
}

#[test]
fn test_fingerprinting_preset_none() {
    let config = StealthConfig::from_preset(StealthPreset::None);

    // None preset should have minimal fingerprinting
    assert!(!config.fingerprinting.cdp_stealth.disable_automation_controlled);
    assert!(!config.fingerprinting.cdp_stealth.override_webdriver);
}

#[test]
fn test_fingerprinting_preset_low() {
    let config = StealthConfig::from_preset(StealthPreset::Low);

    // Low preset should have basic fingerprinting
    assert!(config.fingerprinting.cdp_stealth.disable_automation_controlled);
    assert!(config.fingerprinting.cdp_stealth.override_webdriver);
    assert_eq!(config.fingerprinting.webgl.noise_level, 0.05);
}

#[test]
fn test_fingerprinting_preset_medium() {
    let config = StealthConfig::from_preset(StealthPreset::Medium);

    // Medium preset should have balanced fingerprinting
    assert!(config.fingerprinting.cdp_stealth.disable_automation_controlled);
    assert!(config.fingerprinting.webgl.randomize_vendor);
    assert!(config.fingerprinting.canvas.add_noise);
    assert_eq!(config.fingerprinting.webgl.noise_level, 0.1);
}

#[test]
fn test_fingerprinting_preset_high() {
    let config = StealthConfig::from_preset(StealthPreset::High);

    // High preset should have maximum fingerprinting protection
    assert!(config.fingerprinting.cdp_stealth.disable_automation_controlled);
    assert!(config.fingerprinting.webgl.randomize_vendor);
    assert!(config.fingerprinting.webgl.randomize_renderer);
    assert!(config.fingerprinting.canvas.add_noise);
    assert!(config.fingerprinting.audio.add_noise);
    assert!(config.fingerprinting.webrtc.block_ip_leak);
    assert_eq!(config.fingerprinting.webgl.noise_level, 0.2);
    assert_eq!(config.fingerprinting.canvas.noise_intensity, 0.1);
}

#[test]
fn test_stealth_controller_random_webgl() {
    let controller = StealthController::from_preset(StealthPreset::High);

    let mut specs = HashSet::new();
    for _ in 0..15 {
        let (vendor, renderer) = controller.random_webgl_specs();
        specs.insert(format!("{}/{}", vendor, renderer));
    }

    assert!(specs.len() > 1, "WebGL specs should vary");
}

#[test]
fn test_stealth_controller_random_hardware() {
    let controller = StealthController::from_preset(StealthPreset::High);

    let mut specs = HashSet::new();
    for _ in 0..15 {
        let (cores, memory) = controller.random_hardware_specs();
        specs.insert((cores, memory));
    }

    assert!(specs.len() > 1, "Hardware specs should vary");
}

#[test]
fn test_fingerprinting_serialization() {
    let config = FingerprintingConfig::default();

    // Test that config can be serialized and deserialized
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: FingerprintingConfig =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(
        config.cdp_stealth.disable_automation_controlled,
        deserialized.cdp_stealth.disable_automation_controlled
    );
    assert_eq!(
        config.webgl.randomize_vendor,
        deserialized.webgl.randomize_vendor
    );
}

#[test]
fn test_comprehensive_fingerprinting_coverage() {
    let config = FingerprintingConfig::default();

    // Verify all major fingerprinting vectors are covered
    // CDP
    assert!(config.cdp_stealth.disable_automation_controlled);

    // WebGL
    assert!(config.webgl.randomize_vendor);

    // Canvas
    assert!(config.canvas.add_noise);

    // Audio
    assert!(config.audio.add_noise);

    // Plugins
    assert!(config.plugins.mock_plugins);

    // WebRTC
    assert!(config.webrtc.block_ip_leak);

    // Hardware
    assert!(config.hardware.spoof_cpu_cores);

    // Fonts
    assert!(config.fonts.limit_fonts);
}

#[test]
fn test_plugin_list_realistic() {
    let config = FingerprintingConfig::default();

    assert!(!config.plugins.plugin_list.is_empty());

    // Should include common Chrome plugins
    let plugins = &config.plugins.plugin_list;
    assert!(plugins
        .iter()
        .any(|p| p.contains("PDF") || p.contains("Native Client")));
}
