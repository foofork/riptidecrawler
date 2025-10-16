use riptide_stealth::{
    load_user_agents_from_file, BrowserType, CanvasConfig, FingerprintingConfig, HardwareConfig,
    JavaScriptInjector, LocaleStrategy, RequestRandomization, RotationStrategy, StealthConfig,
    StealthController, StealthPreset, TimingConfig, UserAgentConfig, UserAgentManager, WebGlConfig,
};
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Integration tests for stealth mode component lifecycle
/// Tests the complete stealth system including user agent rotation,
/// fingerprinting evasion, JavaScript injection, and timing randomization.

#[tokio::test]
async fn test_stealth_controller_complete_lifecycle() {
    let mut controller = StealthController::from_preset(StealthPreset::High);

    // Test initial state
    assert!(controller.is_stealth_enabled());
    assert_eq!(controller.get_request_count(), 0);
    assert_eq!(controller.get_preset(), &StealthPreset::High);

    // Test user agent rotation through multiple requests
    let mut user_agents = HashSet::new();
    for _ in 0..10 {
        let ua = controller.next_user_agent().to_string();
        user_agents.insert(ua);
        controller.mark_request_sent();
    }

    // Should have some variety in user agents
    assert!(user_agents.len() > 1);
    assert_eq!(controller.get_request_count(), 10);

    // Test header generation consistency
    let headers1 = controller.generate_headers();
    let headers2 = controller.generate_headers();

    // Should always have basic headers (HeaderConsistencyManager uses lowercase)
    assert!(headers1.contains_key("accept") || headers1.contains_key("Accept"));
    assert!(headers1.contains_key("accept-language") || headers1.contains_key("Accept-Language"));
    assert!(headers1.contains_key("accept-encoding") || headers1.contains_key("Accept-Encoding"));
    assert!(headers2.contains_key("accept") || headers2.contains_key("Accept"));

    // Test viewport generation
    let mut viewports = HashSet::new();
    for _ in 0..10 {
        let viewport = controller.random_viewport();
        viewports.insert(viewport);
        assert!(viewport.0 > 0 && viewport.1 > 0);
    }

    // Should have some variety in viewports
    assert!(viewports.len() > 1);

    // Test timing delays
    let delays: Vec<Duration> = (0..5).map(|_| controller.calculate_delay()).collect();

    for delay in &delays {
        assert!(delay.as_millis() > 0);
        assert!(delay.as_millis() < 5000); // Reasonable upper bound
    }

    // Test session reset
    controller.reset_session();
    assert_eq!(controller.get_request_count(), 0);
}

#[tokio::test]
async fn test_user_agent_manager_rotation_strategies() {
    // Test Random strategy
    let random_config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![
            "Mozilla/5.0 (Chrome) Random1".to_string(),
            "Mozilla/5.0 (Chrome) Random2".to_string(),
            "Mozilla/5.0 (Chrome) Random3".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut random_manager = UserAgentManager::new(random_config);
    let mut random_agents = HashSet::new();

    for _ in 0..20 {
        let ua = random_manager.next_user_agent().to_string();
        random_agents.insert(ua);
    }

    // Random should provide variety
    assert!(random_agents.len() > 1);

    // Test Sequential strategy
    let sequential_config = UserAgentConfig {
        strategy: RotationStrategy::Sequential,
        agents: vec![
            "Mozilla/5.0 (Chrome) Seq1".to_string(),
            "Mozilla/5.0 (Chrome) Seq2".to_string(),
            "Mozilla/5.0 (Chrome) Seq3".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut sequential_manager = UserAgentManager::new(sequential_config);
    let seq_agents: Vec<String> = (0..6)
        .map(|_| sequential_manager.next_user_agent().to_string())
        .collect();

    // Sequential should cycle through agents
    assert_eq!(seq_agents[0], seq_agents[3]); // Should wrap around
    assert_eq!(seq_agents[1], seq_agents[4]);
    assert_eq!(seq_agents[2], seq_agents[5]);

    // Test Sticky strategy
    let sticky_config = UserAgentConfig {
        strategy: RotationStrategy::Sticky,
        agents: vec![
            "Mozilla/5.0 (Chrome) Sticky1".to_string(),
            "Mozilla/5.0 (Chrome) Sticky2".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut sticky_manager = UserAgentManager::new(sticky_config);
    let sticky_agents: Vec<String> = (0..5)
        .map(|_| sticky_manager.next_user_agent().to_string())
        .collect();

    // Sticky should use the same agent
    assert!(sticky_agents.iter().all(|ua| ua == &sticky_agents[0]));
}

#[tokio::test]
async fn test_browser_type_filtering() {
    let test_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edge/120.0.0.0".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    assert_eq!(manager.agent_count(), 4);

    // Filter to Chrome only
    manager.filter_by_browser_type(BrowserType::Chrome);
    assert!(manager.agent_count() > 0);
    assert!(manager.agent_count() <= 4);

    // Verify all remaining agents are Chrome
    for _ in 0..10 {
        let ua = manager.next_user_agent();
        assert!(ua.contains("Chrome"));
    }

    // Test Firefox filtering
    let config2 = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![
            "Mozilla/5.0 (Chrome) Test".to_string(),
            "Mozilla/5.0 (Firefox) Test".to_string(),
            "Mozilla/5.0 (Safari) Test".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager2 = UserAgentManager::new(config2);
    manager2.filter_by_browser_type(BrowserType::Firefox);

    let ua = manager2.next_user_agent();
    assert!(ua.contains("Firefox"));
}

#[tokio::test]
async fn test_mobile_agent_filtering() {
    let test_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
        "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15".to_string(),
        "Mozilla/5.0 (Android 12; Mobile; rv:121.0) Gecko/121.0 Firefox/121.0".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

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

#[tokio::test]
async fn test_fingerprinting_evasion_components() {
    let config = FingerprintingConfig::default();

    // Test WebGL fingerprinting evasion
    let (vendor1, renderer1) = config.webgl.get_random_webgl_specs();
    let (vendor2, renderer2) = config.webgl.get_random_webgl_specs();

    assert!(!vendor1.is_empty());
    assert!(!renderer1.is_empty());
    assert!(!vendor2.is_empty());
    assert!(!renderer2.is_empty());

    // Test Hardware fingerprinting evasion
    let (cores1, memory1) = config.hardware.get_random_hardware_specs();
    let (cores2, memory2) = config.hardware.get_random_hardware_specs();

    assert!(cores1 > 0);
    assert!(memory1 > 0);
    assert!(cores2 > 0);
    assert!(memory2 > 0);

    // Should have some variation
    let mut variations = HashSet::new();
    for _ in 0..20 {
        let (cores, memory) = config.hardware.get_random_hardware_specs();
        variations.insert((cores, memory));
    }
    assert!(variations.len() > 1);

    // Test noise levels are within bounds
    assert!(config.webgl.noise_level >= 0.0 && config.webgl.noise_level <= 1.0);
    assert!(config.canvas.noise_intensity >= 0.0 && config.canvas.noise_intensity <= 1.0);
    assert!(config.audio.noise_intensity >= 0.0 && config.audio.noise_intensity <= 1.0);
}

#[tokio::test]
async fn test_javascript_injection_generation() {
    let hardware_config = HardwareConfig::default();
    let webgl_config = WebGlConfig::default();
    let canvas_config = CanvasConfig::default();

    // Test with different locale strategies
    let strategies = vec![
        LocaleStrategy::Random,
        LocaleStrategy::Fixed("en-US".to_string()),
        LocaleStrategy::Fixed("de-DE".to_string()),
        LocaleStrategy::Fixed("fr-FR".to_string()),
    ];

    for strategy in strategies {
        let injector =
            JavaScriptInjector::new(&hardware_config, &webgl_config, &canvas_config, &strategy);

        let js_code = injector.generate_stealth_js();

        // Verify essential stealth components are included
        assert!(js_code.contains("webdriver"));
        assert!(js_code.contains("hardwareConcurrency"));
        assert!(js_code.contains("deviceMemory"));
        assert!(js_code.contains("WebGLRenderingContext"));
        assert!(js_code.contains("getParameter"));
        assert!(js_code.contains("plugins"));
        assert!(js_code.contains("languages"));

        // Verify structure and quality
        assert!(js_code.contains("function"));
        assert!(js_code.starts_with('\n'));
        assert!(js_code.len() > 1000); // Should be substantial

        // Should not contain obvious debugging or test strings
        assert!(!js_code.contains("console.log"));
        assert!(!js_code.contains("debugger"));
    }
}

#[tokio::test]
async fn test_request_randomization_features() {
    let config = RequestRandomization::default();

    // Test header variations exist and are reasonable
    assert!(!config.headers.accept_variations.is_empty());
    assert!(!config.headers.accept_language_variations.is_empty());
    assert!(!config.headers.accept_encoding_variations.is_empty());

    for accept in &config.headers.accept_variations {
        assert!(accept.contains("text/html"));
    }

    for encoding in &config.headers.accept_encoding_variations {
        assert!(encoding.contains("gzip") || encoding.contains("deflate"));
    }

    // Test timing configuration
    assert!(config.timing_jitter.base_delay_ms > 0);
    assert!(config.timing_jitter.jitter_percentage >= 0.0);
    assert!(config.timing_jitter.jitter_percentage <= 1.0);
    assert!(config.timing_jitter.min_delay_ms <= config.timing_jitter.max_delay_ms);

    // Test viewport variations
    assert!(!config.viewport.sizes.is_empty());
    for (width, height) in &config.viewport.sizes {
        assert!(*width >= 1000); // Reasonable minimum
        assert!(*height >= 600);
        assert!(*width <= 2560); // Reasonable maximum
        assert!(*height <= 1440);
    }

    // Test locale configuration
    assert!(!config.locale.locales.is_empty());
    assert!(!config.locale.timezones.is_empty());

    // Each locale should have a corresponding timezone
    for locale in &config.locale.locales {
        assert!(config.locale.timezones.contains_key(locale));
    }
}

#[tokio::test]
async fn test_stealth_preset_configurations() {
    let presets = vec![
        StealthPreset::None,
        StealthPreset::Low,
        StealthPreset::Medium,
        StealthPreset::High,
    ];

    for preset in presets {
        let config = StealthConfig::from_preset(preset.clone());

        assert_eq!(config.preset, preset);
        assert!(!config.user_agent.agents.is_empty());

        // Check that stealth level increases with preset
        match preset {
            StealthPreset::None => {
                // None should have minimal stealth
                assert!(config.get_cdp_flags().is_empty());
            }
            StealthPreset::Low => {
                // Low should have basic stealth
                assert!(!config.get_cdp_flags().is_empty());
            }
            StealthPreset::Medium | StealthPreset::High => {
                // Medium and High should have comprehensive stealth
                let flags = config.get_cdp_flags();
                assert!(!flags.is_empty());
                assert!(flags
                    .iter()
                    .any(|f| f.contains("no-sandbox") || f.contains("disable")));
            }
        }

        // All presets should have valid fingerprinting config
        assert!(config.fingerprinting.webgl.noise_level >= 0.0);
        assert!(config.fingerprinting.canvas.noise_intensity >= 0.0);
        assert!(config.fingerprinting.audio.noise_intensity >= 0.0);
    }
}

#[tokio::test]
async fn test_timing_configuration_and_delays() {
    let timing_config = TimingConfig::default();

    // Verify default timing is reasonable
    assert!(timing_config.default_timing.min_delay_ms > 0);
    assert!(timing_config.default_timing.min_delay_ms <= timing_config.default_timing.max_delay_ms);
    assert!(timing_config.default_timing.burst_size > 0);

    if let Some(rpm_limit) = timing_config.default_timing.rpm_limit {
        assert!(rpm_limit > 0);
        assert!(rpm_limit <= 3600); // Max 1 request per second
    }

    // Test delay calculation varies appropriately
    let mut controller = StealthController::from_preset(StealthPreset::Medium);
    let mut delays = Vec::new();

    for _ in 0..10 {
        let delay = controller.calculate_delay();
        delays.push(delay);
        assert!(delay.as_millis() > 0);
        assert!(delay.as_millis() < 10000); // Reasonable upper bound
    }

    // Should have some variation in delays
    let unique_delays: HashSet<_> = delays.iter().collect();
    assert!(unique_delays.len() > 1);
}

#[tokio::test]
async fn test_error_handling_and_fallbacks() {
    // Test with empty user agent list
    let empty_config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![], // Empty list
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut empty_manager = UserAgentManager::new(empty_config);
    let fallback_ua = empty_manager.next_user_agent();

    // Should provide fallback user agent
    assert!(!fallback_ua.is_empty());
    assert!(fallback_ua.contains("Mozilla"));

    // Test invalid file loading
    let file_result = load_user_agents_from_file("non_existent_file.txt");
    assert!(file_result.is_err());

    // Test browser filtering with no matches
    let chrome_only_config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec!["Mozilla/5.0 (Firefox only)".to_string()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut chrome_manager = UserAgentManager::new(chrome_only_config);
    chrome_manager.filter_by_browser_type(BrowserType::Chrome);

    // Should still provide some user agent even if no matches
    let ua = chrome_manager.next_user_agent();
    assert!(!ua.is_empty());
}

#[tokio::test]
async fn test_stealth_performance_and_memory() {
    let start = Instant::now();

    // Create multiple controllers to test memory usage
    let controllers: Vec<_> = (0..50)
        .map(|_| StealthController::from_preset(StealthPreset::High))
        .collect();

    let creation_time = start.elapsed();
    assert!(creation_time < Duration::from_millis(1000)); // Should be fast

    // Test JS generation performance
    let js_start = Instant::now();
    let mut controller = StealthController::from_preset(StealthPreset::High);

    for _ in 0..10 {
        let js = controller.get_stealth_js();
        assert!(!js.is_empty());
        assert!(js.len() > 500); // Substantial content
    }

    let js_time = js_start.elapsed();
    assert!(js_time < Duration::from_millis(500)); // Should be reasonably fast

    // Test concurrent access simulation
    // Note: Using sequential access since StealthController uses thread_rng internally
    // which is not Send. In production, each request thread would have its own controller.
    let mut controller = StealthController::from_preset(StealthPreset::Medium);
    let mut user_agents = Vec::new();
    for _ in 0..10 {
        let ua = controller.next_user_agent().to_string();
        user_agents.push(ua);
    }

    for ua in user_agents {
        assert!(!ua.is_empty());
    }

    // Verify controllers can be dropped without issues
    drop(controllers);
}

#[tokio::test]
async fn test_stealth_configuration_updates() {
    let mut controller = StealthController::from_preset(StealthPreset::Low);

    assert_eq!(controller.get_preset(), &StealthPreset::Low);
    let initial_count = controller.get_request_count();

    // Simulate some activity
    for _ in 0..5 {
        controller.next_user_agent();
        controller.mark_request_sent();
    }

    assert!(controller.get_request_count() > initial_count);

    // Update configuration
    let high_config = StealthConfig::from_preset(StealthPreset::High);
    controller.update_config(high_config);

    assert_eq!(controller.get_preset(), &StealthPreset::High);

    // Reset session
    controller.reset_session();
    assert_eq!(controller.get_request_count(), 0);

    // Verify new configuration is working
    let js_code = controller.get_stealth_js();
    assert!(!js_code.is_empty());
    assert!(js_code.contains("function"));
}
