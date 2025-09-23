use riptide_core::stealth::{
    StealthConfig, StealthController, UserAgentConfig, RotationStrategy, BrowserType,
    RequestRandomization, HeaderRandomization, TimingJitter, ViewportRandomization,
    LocaleRandomization, LocaleStrategy, ProxyConfig, ProxyType, ProxyEndpoint,
    ProxyRotation, ProxyAuth, FingerprintingConfig, CdpStealthConfig, WebGlConfig,
    CanvasConfig, AudioConfig, PluginConfig, TimingConfig, DomainTiming, RateLimit,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_stealth_config_creation() {
    let config = StealthConfig {
        user_agent: UserAgentConfig {
            agents: vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36".to_string(),
            ],
            strategy: RotationStrategy::Random,
            include_mobile: true,
            browser_preference: BrowserType::Chrome,
        },
        request_randomization: RequestRandomization {
            headers: HeaderRandomization {
                accept_variations: vec![
                    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
                ],
                accept_language_variations: vec![
                    "en-US,en;q=0.9".to_string(),
                ],
                accept_encoding_variations: vec![
                    "gzip, deflate, br".to_string(),
                ],
                custom_headers: HashMap::new(),
                randomize_order: true,
            },
            timing_jitter: TimingJitter {
                base_delay_ms: 1500,
                jitter_percentage: 0.3,
                min_delay_ms: 800,
                max_delay_ms: 4000,
            },
            viewport: ViewportRandomization {
                sizes: vec![(1920, 1080), (1366, 768)],
                add_variance: true,
                max_variance: 100,
            },
            locale: LocaleRandomization {
                locales: vec!["en-US".to_string(), "en-GB".to_string()],
                timezones: {
                    let mut tz = HashMap::new();
                    tz.insert("en-US".to_string(), "America/New_York".to_string());
                    tz.insert("en-GB".to_string(), "Europe/London".to_string());
                    tz
                },
                strategy: LocaleStrategy::Random,
            },
        },
        proxy: Some(ProxyConfig {
            proxy_type: ProxyType::Http,
            endpoints: vec![
                ProxyEndpoint {
                    host: "proxy1.example.com".to_string(),
                    port: 8080,
                    supports_https: true,
                    location: Some("US-East".to_string()),
                    healthy: true,
                },
                ProxyEndpoint {
                    host: "proxy2.example.com".to_string(),
                    port: 3128,
                    supports_https: false,
                    location: Some("EU-West".to_string()),
                    healthy: true,
                },
            ],
            rotation: ProxyRotation::Random,
            auth: Some(ProxyAuth {
                username: "user123".to_string(),
                password: "pass456".to_string(),
            }),
        }),
        fingerprinting: FingerprintingConfig {
            cdp_stealth: CdpStealthConfig {
                disable_automation_controlled: true,
                override_webdriver: true,
                override_permissions: true,
                override_plugins: true,
                override_chrome: true,
            },
            webgl: WebGlConfig {
                randomize_vendor: true,
                randomize_renderer: true,
                noise_level: 0.15,
            },
            canvas: CanvasConfig {
                add_noise: true,
                noise_intensity: 0.08,
                block_data_extraction: false,
            },
            audio: AudioConfig {
                add_noise: true,
                block_extraction: false,
            },
            plugins: PluginConfig {
                mock_plugins: true,
                plugin_list: vec![
                    "Chrome PDF Plugin".to_string(),
                    "Chrome PDF Viewer".to_string(),
                ],
            },
        },
        timing: TimingConfig {
            per_domain: {
                let mut domains = HashMap::new();
                domains.insert("example.com".to_string(), DomainTiming {
                    min_delay_ms: 2000,
                    max_delay_ms: 5000,
                    rpm_limit: Some(30),
                    burst_size: 3,
                });
                domains
            },
            default_timing: DomainTiming {
                min_delay_ms: 1000,
                max_delay_ms: 3000,
                rpm_limit: Some(60),
                burst_size: 5,
            },
            global_rate_limit: Some(RateLimit {
                rps: 2.0,
                burst: 10,
            }),
        },
    };

    assert_eq!(config.user_agent.agents.len(), 2);
    assert!(matches!(config.user_agent.strategy, RotationStrategy::Random));
    assert!(config.user_agent.include_mobile);
    assert!(matches!(config.user_agent.browser_preference, BrowserType::Chrome));

    assert!(config.proxy.is_some());
    if let Some(proxy) = &config.proxy {
        assert_eq!(proxy.endpoints.len(), 2);
        assert!(matches!(proxy.proxy_type, ProxyType::Http));
        assert!(proxy.auth.is_some());
    }

    assert!(config.fingerprinting.cdp_stealth.disable_automation_controlled);
    assert!(config.fingerprinting.webgl.randomize_vendor);
    assert!(config.fingerprinting.canvas.add_noise);
}

#[tokio::test]
async fn test_stealth_controller_user_agent_rotation() {
    let config = StealthConfig::default();
    let mut controller = StealthController::new(config);

    // Test different rotation strategies
    for _ in 0..10 {
        let user_agent = controller.next_user_agent();
        assert!(!user_agent.is_empty());
        assert!(user_agent.contains("Mozilla"));
    }
}

#[tokio::test]
async fn test_stealth_controller_header_generation() {
    let config = StealthConfig::default();
    let controller = StealthController::new(config);

    let headers = controller.generate_headers();

    assert!(headers.contains_key("Accept"));
    assert!(headers.contains_key("Accept-Language"));
    assert!(headers.contains_key("Accept-Encoding"));

    // Verify header values are from the configured variations
    let accept = headers.get("Accept").unwrap();
    assert!(accept.contains("text/html"));

    let accept_lang = headers.get("Accept-Language").unwrap();
    assert!(accept_lang.contains("en"));

    let accept_enc = headers.get("Accept-Encoding").unwrap();
    assert!(accept_enc.contains("gzip"));
}

#[tokio::test]
async fn test_stealth_controller_delay_calculation() {
    let config = StealthConfig::default();
    let mut controller = StealthController::new(config);

    // Test multiple delay calculations
    for _ in 0..20 {
        let delay = controller.calculate_delay();
        let delay_ms = delay.as_millis() as u64;

        // Should be within configured bounds
        assert!(delay_ms >= 500); // min_delay_ms
        assert!(delay_ms <= 3000); // max_delay_ms
    }
}

#[tokio::test]
async fn test_stealth_controller_viewport_randomization() {
    let config = StealthConfig::default();
    let controller = StealthController::new(config);

    // Test multiple viewport generations
    for _ in 0..20 {
        let (width, height) = controller.random_viewport();

        assert!(width > 0);
        assert!(height > 0);

        // Should be reasonable viewport sizes
        assert!(width >= 1000); // Smallest configured size
        assert!(width <= 2000); // Largest configured size + variance
        assert!(height >= 700);
        assert!(height <= 1200);
    }
}

#[tokio::test]
async fn test_rotation_strategies() {
    let strategies = vec![
        RotationStrategy::Random,
        RotationStrategy::Sequential,
        RotationStrategy::Sticky,
        RotationStrategy::DomainBased,
    ];

    for strategy in strategies {
        let config = StealthConfig {
            user_agent: UserAgentConfig {
                agents: vec![
                    "Agent1".to_string(),
                    "Agent2".to_string(),
                    "Agent3".to_string(),
                ],
                strategy,
                include_mobile: false,
                browser_preference: BrowserType::Chrome,
            },
            ..StealthConfig::default()
        };

        let mut controller = StealthController::new(config);

        // Test that user agent rotation works for each strategy
        let ua1 = controller.next_user_agent().to_string();
        let ua2 = controller.next_user_agent().to_string();
        let ua3 = controller.next_user_agent().to_string();

        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());
        assert!(!ua3.is_empty());

        // For sticky strategy, all should be the same
        if matches!(strategy, RotationStrategy::Sticky) {
            assert_eq!(ua1, ua2);
            assert_eq!(ua2, ua3);
        }
    }
}

#[tokio::test]
async fn test_browser_type_preferences() {
    let browser_types = vec![
        BrowserType::Chrome,
        BrowserType::Firefox,
        BrowserType::Safari,
        BrowserType::Edge,
        BrowserType::Mixed,
    ];

    for browser_type in browser_types {
        let config = StealthConfig {
            user_agent: UserAgentConfig {
                agents: vec!["Test Agent".to_string()],
                strategy: RotationStrategy::Sticky,
                include_mobile: false,
                browser_preference: browser_type,
            },
            ..StealthConfig::default()
        };

        // Test serialization
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: StealthConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&config.user_agent.browser_preference) ==
                std::mem::discriminant(&deserialized.user_agent.browser_preference));
    }
}

#[tokio::test]
async fn test_proxy_configuration() {
    let proxy_types = vec![
        ProxyType::Http,
        ProxyType::Https,
        ProxyType::Socks4,
        ProxyType::Socks5,
    ];

    let proxy_rotations = vec![
        ProxyRotation::Random,
        ProxyRotation::RoundRobin,
        ProxyRotation::HealthBased,
        ProxyRotation::Geographic,
    ];

    for proxy_type in proxy_types {
        for rotation in &proxy_rotations {
            let config = ProxyConfig {
                proxy_type: proxy_type.clone(),
                endpoints: vec![
                    ProxyEndpoint {
                        host: "proxy.example.com".to_string(),
                        port: 8080,
                        supports_https: true,
                        location: Some("US".to_string()),
                        healthy: true,
                    },
                ],
                rotation: rotation.clone(),
                auth: None,
            };

            // Test serialization
            let json = serde_json::to_string(&config).expect("Should serialize");
            let deserialized: ProxyConfig = serde_json::from_str(&json).expect("Should deserialize");

            assert!(std::mem::discriminant(&config.proxy_type) ==
                    std::mem::discriminant(&deserialized.proxy_type));
            assert!(std::mem::discriminant(&config.rotation) ==
                    std::mem::discriminant(&deserialized.rotation));
        }
    }
}

#[tokio::test]
async fn test_fingerprinting_config() {
    let config = FingerprintingConfig {
        cdp_stealth: CdpStealthConfig {
            disable_automation_controlled: true,
            override_webdriver: true,
            override_permissions: false,
            override_plugins: true,
            override_chrome: false,
        },
        webgl: WebGlConfig {
            randomize_vendor: true,
            randomize_renderer: false,
            noise_level: 0.25,
        },
        canvas: CanvasConfig {
            add_noise: false,
            noise_intensity: 0.1,
            block_data_extraction: true,
        },
        audio: AudioConfig {
            add_noise: true,
            block_extraction: true,
        },
        plugins: PluginConfig {
            mock_plugins: false,
            plugin_list: vec!["Custom Plugin".to_string()],
        },
    };

    assert!(config.cdp_stealth.disable_automation_controlled);
    assert!(!config.cdp_stealth.override_permissions);
    assert!(config.webgl.randomize_vendor);
    assert!(!config.webgl.randomize_renderer);
    assert!(!config.canvas.add_noise);
    assert!(config.canvas.block_data_extraction);
    assert!(config.audio.add_noise);
    assert!(config.audio.block_extraction);
    assert!(!config.plugins.mock_plugins);
    assert_eq!(config.plugins.plugin_list.len(), 1);

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: FingerprintingConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.cdp_stealth.disable_automation_controlled,
               deserialized.cdp_stealth.disable_automation_controlled);
    assert_eq!(config.webgl.noise_level, deserialized.webgl.noise_level);
    assert_eq!(config.canvas.noise_intensity, deserialized.canvas.noise_intensity);
}

#[tokio::test]
async fn test_timing_configuration() {
    let mut per_domain = HashMap::new();
    per_domain.insert("slow-site.com".to_string(), DomainTiming {
        min_delay_ms: 3000,
        max_delay_ms: 8000,
        rpm_limit: Some(20),
        burst_size: 2,
    });
    per_domain.insert("fast-site.com".to_string(), DomainTiming {
        min_delay_ms: 500,
        max_delay_ms: 1500,
        rpm_limit: Some(120),
        burst_size: 10,
    });

    let config = TimingConfig {
        per_domain,
        default_timing: DomainTiming {
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            rpm_limit: Some(60),
            burst_size: 5,
        },
        global_rate_limit: Some(RateLimit {
            rps: 5.0,
            burst: 20,
        }),
    };

    assert_eq!(config.per_domain.len(), 2);
    assert!(config.global_rate_limit.is_some());

    let slow_site = config.per_domain.get("slow-site.com").unwrap();
    assert_eq!(slow_site.min_delay_ms, 3000);
    assert_eq!(slow_site.max_delay_ms, 8000);
    assert_eq!(slow_site.rpm_limit, Some(20));

    let fast_site = config.per_domain.get("fast-site.com").unwrap();
    assert_eq!(fast_site.min_delay_ms, 500);
    assert_eq!(fast_site.rpm_limit, Some(120));

    if let Some(global_limit) = &config.global_rate_limit {
        assert_eq!(global_limit.rps, 5.0);
        assert_eq!(global_limit.burst, 20);
    }
}

#[tokio::test]
async fn test_locale_strategies() {
    let strategies = vec![
        LocaleStrategy::Random,
        LocaleStrategy::Geographic,
        LocaleStrategy::TargetBased,
        LocaleStrategy::Fixed("en-US".to_string()),
    ];

    for strategy in strategies {
        let config = LocaleRandomization {
            locales: vec!["en-US".to_string(), "de-DE".to_string()],
            timezones: HashMap::new(),
            strategy,
        };

        // Test serialization
        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: LocaleRandomization = serde_json::from_str(&json).expect("Should deserialize");

        assert!(std::mem::discriminant(&config.strategy) ==
                std::mem::discriminant(&deserialized.strategy));
    }
}

#[tokio::test]
async fn test_header_randomization() {
    let mut custom_headers = HashMap::new();
    custom_headers.insert("X-Forwarded-For".to_string(), vec![
        "192.168.1.1".to_string(),
        "10.0.0.1".to_string(),
    ]);
    custom_headers.insert("X-Real-IP".to_string(), vec![
        "203.0.113.1".to_string(),
    ]);

    let config = HeaderRandomization {
        accept_variations: vec![
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string(),
        ],
        accept_language_variations: vec![
            "en-US,en;q=0.9".to_string(),
            "en-US,en;q=0.8,de;q=0.6".to_string(),
        ],
        accept_encoding_variations: vec![
            "gzip, deflate".to_string(),
            "gzip, deflate, br".to_string(),
        ],
        custom_headers,
        randomize_order: true,
    };

    assert_eq!(config.accept_variations.len(), 2);
    assert_eq!(config.accept_language_variations.len(), 2);
    assert_eq!(config.accept_encoding_variations.len(), 2);
    assert_eq!(config.custom_headers.len(), 2);
    assert!(config.randomize_order);

    let x_forwarded_for = config.custom_headers.get("X-Forwarded-For").unwrap();
    assert_eq!(x_forwarded_for.len(), 2);
    assert!(x_forwarded_for.contains(&"192.168.1.1".to_string()));

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: HeaderRandomization = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.accept_variations, deserialized.accept_variations);
    assert_eq!(config.custom_headers, deserialized.custom_headers);
}

#[tokio::test]
async fn test_timing_jitter() {
    let config = TimingJitter {
        base_delay_ms: 2000,
        jitter_percentage: 0.4, // 40% jitter
        min_delay_ms: 1000,
        max_delay_ms: 5000,
    };

    // Test that jitter calculation would be within bounds
    let max_jitter = (config.base_delay_ms as f64 * config.jitter_percentage) as u64;
    assert_eq!(max_jitter, 800); // 40% of 2000ms

    let min_expected = config.base_delay_ms - max_jitter;
    let max_expected = config.base_delay_ms + max_jitter;

    // These would be clamped by min/max delays
    let final_min = min_expected.max(config.min_delay_ms);
    let final_max = max_expected.min(config.max_delay_ms);

    assert_eq!(final_min, 1200); // max(1200, 1000) = 1200
    assert_eq!(final_max, 2800); // min(2800, 5000) = 2800

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: TimingJitter = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.base_delay_ms, deserialized.base_delay_ms);
    assert_eq!(config.jitter_percentage, deserialized.jitter_percentage);
}

#[tokio::test]
async fn test_viewport_randomization() {
    let config = ViewportRandomization {
        sizes: vec![
            (1920, 1080),
            (1366, 768),
            (1536, 864),
            (390, 844), // iPhone
        ],
        add_variance: true,
        max_variance: 50,
    };

    assert_eq!(config.sizes.len(), 4);
    assert!(config.add_variance);
    assert_eq!(config.max_variance, 50);

    // Verify viewport sizes are reasonable
    for (width, height) in &config.sizes {
        assert!(*width > 0);
        assert!(*height > 0);
        assert!(*width >= 300); // Minimum reasonable width
        assert!(*height >= 200); // Minimum reasonable height
    }

    // Test serialization
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: ViewportRandomization = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.sizes, deserialized.sizes);
    assert_eq!(config.add_variance, deserialized.add_variance);
    assert_eq!(config.max_variance, deserialized.max_variance);
}