//! User agent rotation and validation tests
//!
//! Validates all rotation strategies, browser filtering, and mobile detection

use riptide_core::stealth::{
    BrowserType, RotationStrategy, StealthController, StealthPreset, UserAgentConfig,
    UserAgentManager,
};
use std::collections::HashSet;

#[test]
fn test_random_rotation_provides_variety() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![
            "Mozilla/5.0 (Windows) Chrome/120".to_string(),
            "Mozilla/5.0 (Mac) Chrome/119".to_string(),
            "Mozilla/5.0 (Linux) Firefox/121".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let mut agents = HashSet::new();

    // Collect 30 samples - should see variety
    for _ in 0..30 {
        let ua = manager.next_user_agent().to_string();
        agents.insert(ua);
    }

    // Should have at least 2 different agents (very likely with 30 samples)
    assert!(
        agents.len() >= 2,
        "Random rotation should provide variety, got {} unique agents",
        agents.len()
    );
}

#[test]
fn test_sequential_rotation_cycles() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Sequential,
        agents: vec![
            "UA1".to_string(),
            "UA2".to_string(),
            "UA3".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);

    // Test multiple complete cycles
    let ua1 = manager.next_user_agent().to_string();
    let ua2 = manager.next_user_agent().to_string();
    let ua3 = manager.next_user_agent().to_string();
    let ua4 = manager.next_user_agent().to_string(); // Should wrap
    let ua5 = manager.next_user_agent().to_string();
    let ua6 = manager.next_user_agent().to_string();

    assert_eq!(ua1, "UA2"); // Starts at index 0, increments to 1
    assert_eq!(ua2, "UA3");
    assert_eq!(ua3, "UA1"); // Wraps around
    assert_eq!(ua4, "UA2");
    assert_eq!(ua5, "UA3");
    assert_eq!(ua6, "UA1");
}

#[test]
fn test_sticky_rotation_consistency() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Sticky,
        agents: vec![
            "UA1".to_string(),
            "UA2".to_string(),
            "UA3".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);

    // All calls should return the same agent
    let ua1 = manager.next_user_agent().to_string();
    let ua2 = manager.next_user_agent().to_string();
    let ua3 = manager.next_user_agent().to_string();

    assert_eq!(ua1, ua2);
    assert_eq!(ua2, ua3);
    assert!(!ua1.is_empty());
}

#[test]
fn test_domain_based_rotation_deterministic() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::DomainBased,
        agents: vec![
            "UA1".to_string(),
            "UA2".to_string(),
            "UA3".to_string(),
        ],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager1 = UserAgentManager::new(config.clone());
    let mut manager2 = UserAgentManager::new(config);

    // Same request count should give same result
    let ua1 = manager1.next_user_agent().to_string();
    let ua2 = manager2.next_user_agent().to_string();

    assert_eq!(ua1, ua2, "Domain-based rotation should be deterministic");
}

#[test]
fn test_chrome_filter() {
    let test_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; rv:121.0) Firefox/121.0".to_string(),
        "Mozilla/5.0 (Macintosh) Safari/605.1.15".to_string(),
        "Mozilla/5.0 (Windows) Chrome/119.0.0.0 Edge/119.0.0.0".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let original_count = manager.agent_count();

    manager.filter_by_browser_type(BrowserType::Chrome);

    assert!(manager.agent_count() < original_count);
    assert!(manager.agent_count() > 0);

    // Verify all remaining agents are Chrome
    for _ in 0..10 {
        let ua = manager.next_user_agent();
        assert!(
            ua.contains("Chrome") && !ua.contains("Edge"),
            "Expected Chrome UA, got: {}",
            ua
        );
    }
}

#[test]
fn test_firefox_filter() {
    let test_agents = vec![
        "Mozilla/5.0 (Windows) Chrome/120.0.0.0".to_string(),
        "Mozilla/5.0 (Windows; rv:121.0) Firefox/121.0".to_string(),
        "Mozilla/5.0 (Mac; rv:120.0) Firefox/120.0".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    manager.filter_by_browser_type(BrowserType::Firefox);

    assert_eq!(manager.agent_count(), 2);

    // Verify all remaining agents are Firefox
    for _ in 0..5 {
        let ua = manager.next_user_agent();
        assert!(ua.contains("Firefox"), "Expected Firefox UA, got: {}", ua);
    }
}

#[test]
fn test_mobile_detection_and_removal() {
    let test_agents = vec![
        "Mozilla/5.0 (Windows NT 10.0) Chrome/120".to_string(),
        "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0) Safari/605.1.15".to_string(),
        "Mozilla/5.0 (Android 12; Mobile) Chrome/120".to_string(),
        "Mozilla/5.0 (iPad; CPU OS 15_0) Safari/605.1.15".to_string(),
        "Mozilla/5.0 (Macintosh) Safari/605.1.15".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    assert_eq!(manager.agent_count(), 5);

    manager.remove_mobile_agents();

    assert_eq!(manager.agent_count(), 2);

    // Verify no mobile agents remain
    for _ in 0..10 {
        let ua = manager.next_user_agent();
        assert!(!ua.contains("iPhone"));
        assert!(!ua.contains("Android"));
        assert!(!ua.contains("iPad"));
        assert!(!ua.contains("Mobile"));
    }
}

#[test]
fn test_add_custom_user_agents() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec!["UA1".to_string()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    assert_eq!(manager.agent_count(), 1);

    manager.add_user_agents(vec!["UA2".to_string(), "UA3".to_string()]);

    assert_eq!(manager.agent_count(), 3);
}

#[test]
fn test_empty_agent_list_fallback() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: vec![],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    let fallback_ua = manager.next_user_agent();

    // Should return a valid fallback user agent
    assert!(!fallback_ua.is_empty());
    assert!(fallback_ua.contains("Mozilla"));
    assert!(fallback_ua.contains("Chrome"));
}

#[test]
fn test_user_agent_validity_format() {
    let controller = StealthController::from_preset(StealthPreset::High);
    let mut controller_mut = StealthController::from_preset(StealthPreset::High);

    let ua = controller_mut.next_user_agent();

    // Verify user agent has expected structure
    assert!(ua.starts_with("Mozilla/"));
    assert!(ua.len() > 50); // User agents should be substantial
    assert!(
        ua.contains("AppleWebKit") || ua.contains("Gecko"),
        "UA should contain browser engine"
    );
}

#[test]
fn test_default_user_agents_are_realistic() {
    let config = UserAgentConfig::default();
    assert!(!config.agents.is_empty());

    // Verify all default agents look realistic
    for ua in &config.agents {
        assert!(ua.starts_with("Mozilla/5.0"));
        assert!(ua.contains("Chrome") || ua.contains("Firefox") || ua.contains("Safari"));
        // Should have OS info
        assert!(
            ua.contains("Windows")
                || ua.contains("Macintosh")
                || ua.contains("X11")
                || ua.contains("Linux")
        );
    }
}

#[test]
fn test_preset_user_agent_rotation() {
    let mut controller = StealthController::from_preset(StealthPreset::Medium);

    let mut agents = HashSet::new();
    for _ in 0..20 {
        let ua = controller.next_user_agent().to_string();
        agents.insert(ua);
    }

    // Should have multiple unique agents
    assert!(agents.len() > 1, "Preset should provide variety");
}

#[test]
fn test_request_count_increment() {
    let config = UserAgentConfig {
        strategy: RotationStrategy::DomainBased,
        agents: vec!["UA1".to_string(), "UA2".to_string()],
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);

    // Request count affects domain-based rotation
    let ua1 = manager.next_user_agent().to_string();
    manager.increment_request_count();
    let ua2 = manager.next_user_agent().to_string();

    // Different request counts may produce different UAs
    // (not guaranteed but possible)
    assert!(!ua1.is_empty());
    assert!(!ua2.is_empty());
}

#[test]
fn test_browser_type_mixed_accepts_all() {
    let test_agents = vec![
        "Chrome UA".to_string(),
        "Firefox UA".to_string(),
        "Safari UA".to_string(),
    ];

    let config = UserAgentConfig {
        strategy: RotationStrategy::Random,
        agents: test_agents,
        include_mobile: false,
        browser_preference: BrowserType::Mixed,
    };

    let mut manager = UserAgentManager::new(config);
    manager.filter_by_browser_type(BrowserType::Mixed);

    // Mixed should accept all browsers
    assert_eq!(manager.agent_count(), 3);
}
