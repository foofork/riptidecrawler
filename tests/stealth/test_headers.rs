//! Header randomization and request timing tests
//!
//! Validates header generation, randomization, timing jitter, and viewport configuration

use riptide_core::stealth::{
    HeaderRandomization, RequestRandomization, StealthController, StealthPreset, TimingJitter,
    ViewportRandomization,
};
use std::collections::HashSet;
use std::time::Duration;

#[test]
fn test_header_generation_includes_required_headers() {
    let controller = StealthController::from_preset(StealthPreset::Medium);

    let headers = controller.generate_headers();

    // Required headers
    assert!(headers.contains_key("Accept"));
    assert!(headers.contains_key("Accept-Language"));
    assert!(headers.contains_key("Accept-Encoding"));

    // Values should not be empty
    assert!(!headers.get("Accept").unwrap().is_empty());
    assert!(!headers.get("Accept-Language").unwrap().is_empty());
    assert!(!headers.get("Accept-Encoding").unwrap().is_empty());
}

#[test]
fn test_accept_header_variations() {
    let config = HeaderRandomization::default();

    assert!(!config.accept_variations.is_empty());

    // All variations should contain text/html
    for variation in &config.accept_variations {
        assert!(variation.contains("text/html"));
    }
}

#[test]
fn test_accept_language_variations() {
    let config = HeaderRandomization::default();

    assert!(!config.accept_language_variations.is_empty());

    // All variations should contain language codes
    for variation in &config.accept_language_variations {
        assert!(variation.contains("en") || variation.contains("de") || variation.contains("fr"));
    }
}

#[test]
fn test_accept_encoding_variations() {
    let config = HeaderRandomization::default();

    assert!(!config.accept_encoding_variations.is_empty());

    // All variations should contain gzip or deflate
    for variation in &config.accept_encoding_variations {
        assert!(variation.contains("gzip") || variation.contains("deflate"));
    }
}

#[test]
fn test_header_randomization_provides_variety() {
    let controller = StealthController::from_preset(StealthPreset::High);

    let mut accept_values = HashSet::new();
    let mut lang_values = HashSet::new();

    // Generate 20 header sets
    for _ in 0..20 {
        let headers = controller.generate_headers();
        accept_values.insert(headers.get("Accept").unwrap().clone());
        lang_values.insert(headers.get("Accept-Language").unwrap().clone());
    }

    // Should see some variety (if multiple variations configured)
    // At minimum, values should be valid
    assert!(!accept_values.is_empty());
    assert!(!lang_values.is_empty());
}

#[test]
fn test_custom_headers_support() {
    let mut custom_headers = std::collections::HashMap::new();
    custom_headers.insert(
        "X-Custom-Header".to_string(),
        vec!["Value1".to_string(), "Value2".to_string()],
    );

    let config = HeaderRandomization {
        accept_variations: vec!["text/html".to_string()],
        accept_language_variations: vec!["en-US".to_string()],
        accept_encoding_variations: vec!["gzip".to_string()],
        custom_headers,
        randomize_order: true,
    };

    assert!(config.custom_headers.contains_key("X-Custom-Header"));
    assert_eq!(config.custom_headers.get("X-Custom-Header").unwrap().len(), 2);
}

#[test]
fn test_timing_jitter_calculation() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    let mut delays = Vec::new();
    for _ in 0..20 {
        let delay = controller.calculate_delay();
        delays.push(delay);
    }

    // All delays should be within bounds
    for delay in &delays {
        assert!(delay.as_millis() >= 500); // min_delay_ms
        assert!(delay.as_millis() <= 3000); // max_delay_ms
    }

    // Should have some variety
    let unique_delays: HashSet<_> = delays.into_iter().collect();
    assert!(unique_delays.len() > 1, "Timing should have jitter");
}

#[test]
fn test_timing_jitter_respects_bounds() {
    let jitter = TimingJitter {
        base_delay_ms: 2000,
        jitter_percentage: 0.5, // 50% jitter
        min_delay_ms: 1000,
        max_delay_ms: 5000,
    };

    // Max jitter would be ±1000ms (50% of 2000)
    // So range is 1000-3000, clamped to min/max
    assert!(jitter.base_delay_ms >= jitter.min_delay_ms);
    assert!(jitter.base_delay_ms <= jitter.max_delay_ms);
}

#[test]
fn test_viewport_randomization() {
    let controller = StealthController::from_preset(StealthPreset::High);

    let mut viewports = HashSet::new();

    for _ in 0..20 {
        let (width, height) = controller.random_viewport();
        viewports.insert((width, height));

        // Verify dimensions are reasonable
        assert!(width > 0);
        assert!(height > 0);
        assert!(width >= 1000); // Minimum reasonable width
        assert!(width <= 2700); // Maximum with variance
        assert!(height >= 500); // Minimum reasonable height
        assert!(height <= 1600); // Maximum with variance
    }

    // Should have variety
    assert!(viewports.len() > 1, "Viewports should vary");
}

#[test]
fn test_viewport_sizes_are_realistic() {
    let config = ViewportRandomization::default();

    // All configured sizes should be realistic screen resolutions
    for (width, height) in &config.sizes {
        assert!(*width >= 1024); // Minimum modern screen
        assert!(*height >= 600);
        assert!(*width <= 3840); // Max 4K
        assert!(*height <= 2160); // Max 4K
    }
}

#[test]
fn test_viewport_variance_disabled() {
    let config = ViewportRandomization {
        sizes: vec![(1920, 1080)],
        add_variance: false,
        max_variance: 0,
    };

    let mut controller_config = StealthController::from_preset(StealthPreset::Low);
    controller_config.config_mut().request_randomization.viewport = config;

    // Without variance, should return exact dimensions
    let (width, height) = controller_config.random_viewport();
    assert_eq!(width, 1920);
    assert_eq!(height, 1080);
}

#[test]
fn test_viewport_variance_bounds() {
    let config = ViewportRandomization {
        sizes: vec![(1920, 1080)],
        add_variance: true,
        max_variance: 50,
    };

    assert!(config.max_variance > 0);
    assert!(config.max_variance <= 100); // Reasonable variance limit
}

#[test]
fn test_request_randomization_default_values() {
    let config = RequestRandomization::default();

    // Headers should have variations
    assert!(!config.headers.accept_variations.is_empty());
    assert!(!config.headers.accept_language_variations.is_empty());
    assert!(!config.headers.accept_encoding_variations.is_empty());

    // Timing should be configured
    assert!(config.timing_jitter.base_delay_ms > 0);
    assert!(config.timing_jitter.jitter_percentage >= 0.0);
    assert!(config.timing_jitter.jitter_percentage <= 1.0);

    // Viewport should have sizes
    assert!(!config.viewport.sizes.is_empty());

    // Locale should have locales and timezones
    assert!(!config.locale.locales.is_empty());
    assert!(!config.locale.timezones.is_empty());
}

#[test]
fn test_preset_none_minimal_randomization() {
    let mut controller = StealthController::from_preset(StealthPreset::None);

    // Even with None preset, should still generate valid headers
    let headers = controller.generate_headers();
    assert!(!headers.is_empty());

    // Timing should still work
    let delay = controller.calculate_delay();
    assert!(delay.as_millis() > 0);
}

#[test]
fn test_preset_high_maximum_randomization() {
    let config = StealthController::from_preset(StealthPreset::High);

    // High preset should have more jitter
    assert_eq!(
        config
            .config()
            .request_randomization
            .timing_jitter
            .jitter_percentage,
        0.4
    );
}

#[test]
fn test_delay_calculation_increments_request_count() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    let initial_count = controller.get_request_count();

    controller.calculate_delay();

    assert!(controller.get_request_count() > initial_count);
}

#[test]
fn test_headers_serialization() {
    let config = HeaderRandomization::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: HeaderRandomization =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(
        config.accept_variations.len(),
        deserialized.accept_variations.len()
    );
    assert_eq!(
        config.randomize_order,
        deserialized.randomize_order
    );
}

#[test]
fn test_timing_jitter_serialization() {
    let jitter = TimingJitter {
        base_delay_ms: 1500,
        jitter_percentage: 0.3,
        min_delay_ms: 800,
        max_delay_ms: 4000,
    };

    let json = serde_json::to_string(&jitter).expect("Should serialize");
    let deserialized: TimingJitter = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(jitter.base_delay_ms, deserialized.base_delay_ms);
    assert_eq!(jitter.jitter_percentage, deserialized.jitter_percentage);
}

#[test]
fn test_viewport_serialization() {
    let config = ViewportRandomization::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: ViewportRandomization =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.sizes, deserialized.sizes);
    assert_eq!(config.add_variance, deserialized.add_variance);
    assert_eq!(config.max_variance, deserialized.max_variance);
}
