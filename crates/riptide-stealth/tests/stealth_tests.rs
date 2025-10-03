//! Real-world stealth and anti-detection tests

use anyhow::Result;
use riptide_stealth::*;
use std::collections::HashMap;
use std::time::Duration;

#[cfg(test)]
mod fingerprint_tests {
    use super::*;
    use riptide_stealth::fingerprint::{BrowserFingerprint, FingerprintGenerator};

    #[test]
    fn test_unique_fingerprint_generation() {
        let generator = FingerprintGenerator::new();

        let fp1 = generator.generate();
        let fp2 = generator.generate();

        // Each fingerprint should be unique
        assert_ne!(fp1.canvas_hash, fp2.canvas_hash);
        assert_ne!(fp1.webgl_hash, fp2.webgl_hash);
        assert_ne!(fp1.audio_hash, fp2.audio_hash);
    }

    #[test]
    fn test_realistic_fingerprint_values() {
        let generator = FingerprintGenerator::new();
        let fp = generator.generate();

        // Check realistic screen resolutions
        let valid_resolutions = vec![
            (1920, 1080),
            (1366, 768),
            (1440, 900),
            (2560, 1440),
            (3840, 2160),
            (1280, 720),
        ];
        assert!(valid_resolutions.contains(&(fp.screen_width, fp.screen_height)));

        // Check realistic timezone offsets
        assert!(fp.timezone_offset >= -720 && fp.timezone_offset <= 840);

        // Check plugin consistency
        if fp.plugins.contains("Chrome PDF Viewer") {
            assert!(fp.user_agent.contains("Chrome"));
        }

        // Check WebGL vendor/renderer pairs
        if fp.webgl_vendor.contains("Intel") {
            assert!(fp.webgl_renderer.contains("Intel") || fp.webgl_renderer.contains("Iris"));
        }
    }

    #[test]
    fn test_fingerprint_persistence() {
        let generator = FingerprintGenerator::new();

        // Generate persistent fingerprint for a session
        let session_fp = generator.generate_persistent("test_session");
        let session_fp2 = generator.generate_persistent("test_session");

        // Same session should have consistent core attributes
        assert_eq!(session_fp.screen_width, session_fp2.screen_width);
        assert_eq!(session_fp.platform, session_fp2.platform);
        assert_eq!(session_fp.language, session_fp2.language);

        // But some attributes should still vary for realism
        assert_ne!(session_fp.canvas_hash, session_fp2.canvas_hash);
    }
}

#[cfg(test)]
mod user_agent_tests {
    use super::*;
    use riptide_stealth::user_agent::{UserAgentConfig, UserAgentRotator};

    #[test]
    fn test_user_agent_rotation() {
        let config = UserAgentConfig {
            browsers: vec!["Chrome", "Firefox", "Safari", "Edge"],
            platforms: vec!["Windows", "Mac", "Linux"],
            mobile_percentage: 0.3,
            ..Default::default()
        };

        let rotator = UserAgentRotator::new(config);
        let mut user_agents = HashMap::new();

        // Generate multiple user agents
        for _ in 0..100 {
            let ua = rotator.next();
            *user_agents.entry(ua.clone()).or_insert(0) += 1;
        }

        // Should have variety
        assert!(user_agents.len() > 10);

        // Should include mobile agents (roughly 30%)
        let mobile_count = user_agents
            .keys()
            .filter(|ua| ua.contains("Mobile") || ua.contains("Android") || ua.contains("iPhone"))
            .count();
        assert!(mobile_count > 20 && mobile_count < 40);
    }

    #[test]
    fn test_user_agent_validity() {
        let rotator = UserAgentRotator::default();

        for _ in 0..50 {
            let ua = rotator.next();

            // Check basic structure
            assert!(ua.contains("Mozilla/"));
            assert!(ua.contains("(") && ua.contains(")"));

            // Check version numbers are realistic
            if ua.contains("Chrome") {
                // Chrome version should be 90+
                let version = extract_chrome_version(&ua);
                assert!(version >= 90 && version <= 130);
            }

            if ua.contains("Firefox") {
                // Firefox version should be 90+
                let version = extract_firefox_version(&ua);
                assert!(version >= 90 && version <= 130);
            }
        }
    }

    fn extract_chrome_version(ua: &str) -> u32 {
        ua.split("Chrome/")
            .nth(1)
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn extract_firefox_version(ua: &str) -> u32 {
        ua.split("Firefox/")
            .nth(1)
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    #[test]
    fn test_user_agent_header_consistency() {
        let rotator = UserAgentRotator::default();
        let ua = rotator.next();

        let headers = rotator.generate_consistent_headers(&ua);

        // Check header consistency
        if ua.contains("Windows") {
            assert_eq!(
                headers.get("Sec-CH-UA-Platform"),
                Some(&"\"Windows\"".to_string())
            );
        } else if ua.contains("Mac") {
            assert_eq!(
                headers.get("Sec-CH-UA-Platform"),
                Some(&"\"macOS\"".to_string())
            );
        }

        // Check Accept headers match browser
        if ua.contains("Chrome") {
            assert!(headers.get("Accept").unwrap().contains("webp"));
        }

        // Check required headers present
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("Accept-Encoding"));
    }
}

#[cfg(test)]
mod behavior_simulation_tests {
    use super::*;
    use riptide_stealth::behavior::{BehaviorSimulator, MouseMovement, ScrollPattern};

    #[tokio::test]
    async fn test_human_like_mouse_movement() {
        let simulator = BehaviorSimulator::new();

        let movements = simulator.generate_mouse_path(
            (100, 100), // start
            (500, 400), // end
            Duration::from_millis(1000),
        );

        // Should have multiple points for curve
        assert!(movements.len() > 5);

        // Should not be perfectly linear
        let mut deviations = 0;
        for i in 1..movements.len() - 1 {
            let expected_x = 100 + (400 * i / movements.len());
            let expected_y = 100 + (300 * i / movements.len());

            if (movements[i].0 as i32 - expected_x as i32).abs() > 10
                || (movements[i].1 as i32 - expected_y as i32).abs() > 10
            {
                deviations += 1;
            }
        }
        assert!(deviations > 0); // Should have some curve

        // Should respect timing
        let total_time: u64 = movements.iter().map(|m| m.2).sum();
        assert!(total_time >= 900 && total_time <= 1100);
    }

    #[tokio::test]
    async fn test_realistic_scroll_patterns() {
        let simulator = BehaviorSimulator::new();

        let pattern = simulator.generate_scroll_pattern(3000); // 3000px page

        // Should have varied scroll speeds
        let speeds: Vec<_> = pattern
            .scrolls
            .iter()
            .map(|s| s.pixels_per_second)
            .collect();
        let min_speed = speeds.iter().min().unwrap();
        let max_speed = speeds.iter().max().unwrap();
        assert!(max_speed > min_speed * 2);

        // Should have pauses for reading
        let pause_count = pattern.scrolls.iter().filter(|s| s.pause_after > 0).count();
        assert!(pause_count > 0);

        // Should cover most of the page
        let total_scroll: i32 = pattern.scrolls.iter().map(|s| s.distance).sum();
        assert!(total_scroll >= 2000);
    }

    #[tokio::test]
    async fn test_typing_simulation() {
        let simulator = BehaviorSimulator::new();

        let text = "Hello, this is a test";
        let typing = simulator.generate_typing_pattern(text);

        // Should have realistic inter-key delays
        assert!(typing.keystrokes.len() == text.len());

        let delays: Vec<_> = typing.keystrokes.iter().map(|k| k.delay_ms).collect();

        // Average typing speed (50-150ms between keys)
        let avg_delay = delays.iter().sum::<u64>() / delays.len() as u64;
        assert!(avg_delay >= 50 && avg_delay <= 150);

        // Should have variation (not robotic)
        let min_delay = delays.iter().min().unwrap();
        let max_delay = delays.iter().max().unwrap();
        assert!(max_delay > min_delay * 2);

        // Should have occasional longer pauses (thinking)
        let long_pauses = delays.iter().filter(|d| **d > 300).count();
        assert!(long_pauses > 0);
    }
}

#[cfg(test)]
mod detection_evasion_tests {
    use super::*;
    use riptide_stealth::evasion::{DetectionEvasion, EvasionConfig};

    #[tokio::test]
    async fn test_webdriver_detection_bypass() {
        let evasion = DetectionEvasion::new(EvasionConfig::default());

        let script = evasion.generate_evasion_script();

        // Should hide webdriver property
        assert!(script.contains("delete navigator.webdriver"));
        assert!(script.contains("Object.defineProperty(navigator, 'webdriver'"));

        // Should modify permissions
        assert!(script.contains("Notification.permission"));

        // Should handle chrome properties
        assert!(script.contains("window.chrome"));
        assert!(script.contains("chrome.runtime"));
    }

    #[tokio::test]
    async fn test_headless_detection_bypass() {
        let evasion = DetectionEvasion::new(EvasionConfig {
            headless_mode: true,
            ..Default::default()
        });

        let patches = evasion.get_headless_patches();

        // Should patch missing features in headless
        assert!(patches.contains_key("navigator.languages"));
        assert!(patches.contains_key("navigator.plugins"));
        assert!(patches.contains_key("window.chrome"));

        // Should have WebGL vendor/renderer
        assert!(patches.contains_key("WebGLRenderingContext.vendor"));
        assert!(patches.contains_key("WebGLRenderingContext.renderer"));
    }

    #[tokio::test]
    async fn test_bot_detection_scores() {
        let evasion = DetectionEvasion::new(EvasionConfig::default());

        // Test against common bot detection checks
        let checks = vec![
            ("navigator.webdriver", false),           // Should be undefined/false
            ("navigator.plugins.length > 0", true),   // Should have plugins
            ("navigator.languages.length > 0", true), // Should have languages
            ("window.chrome", true),                  // Should exist for Chrome
            ("document.hidden", false),               // Should not be hidden
        ];

        for (check, expected) in checks {
            let result = evasion.evaluate_detection_check(check).await;
            assert_eq!(result, expected, "Failed check: {}", check);
        }
    }

    #[test]
    fn test_captcha_detection() {
        let detector = CaptchaDetector::new();

        let html_with_recaptcha = r#"
            <script src="https://www.google.com/recaptcha/api.js"></script>
            <div class="g-recaptcha" data-sitekey="..."></div>
        "#;

        let html_with_hcaptcha = r#"
            <script src="https://hcaptcha.com/1/api.js"></script>
            <div class="h-captcha" data-sitekey="..."></div>
        "#;

        let html_with_cloudflare = r#"
            <title>Just a moment...</title>
            <div class="cf-browser-verification"></div>
        "#;

        assert!(detector.has_captcha(html_with_recaptcha));
        assert!(detector.has_captcha(html_with_hcaptcha));
        assert!(detector.has_cloudflare_challenge(html_with_cloudflare));
    }
}

#[cfg(test)]
mod rate_limiting_tests {
    use super::*;
    use riptide_stealth::rate_limit::{RateLimitConfig, RateLimiter};
    use std::time::Instant;

    #[tokio::test]
    async fn test_rate_limiting_per_domain() {
        let config = RateLimitConfig {
            requests_per_second: 2.0,
            burst_size: 3,
            ..Default::default()
        };

        let limiter = RateLimiter::new(config);

        let start = Instant::now();

        // First 3 should go through immediately (burst)
        for i in 0..3 {
            assert!(limiter.check_rate_limit("example.com").await);
        }

        // 4th should be delayed
        limiter.check_rate_limit("example.com").await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(500)); // ~2 req/sec

        // Different domain should not be limited
        assert!(limiter.check_rate_limit("other.com").await);
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiting() {
        let mut limiter = AdaptiveRateLimiter::new();

        // Simulate server responses
        limiter
            .record_response("example.com", 200, Duration::from_millis(100))
            .await;
        limiter
            .record_response("example.com", 200, Duration::from_millis(120))
            .await;

        // Fast, successful responses - can increase rate
        let rate = limiter.get_current_rate("example.com").await;
        assert!(rate > 1.0);

        // Simulate errors
        for _ in 0..5 {
            limiter
                .record_response("example.com", 429, Duration::from_millis(50))
                .await;
        }

        // Should reduce rate after 429s
        let rate = limiter.get_current_rate("example.com").await;
        assert!(rate < 1.0);
    }
}
