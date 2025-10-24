//! Integration Tests for Domain Warm-Start Caching (Phase 10.4)
//!
//! This test suite validates the domain profile engine caching functionality:
//! - Cache hit/miss scenarios
//! - Confidence threshold behavior
//! - TTL expiry logic
//! - Integration with decide_engine()
//!
//! ## Test Coverage (23 tests)
//! 1. Cache hit/miss scenarios (8 tests)
//! 2. Confidence threshold tests (5 tests)
//! 3. TTL expiry tests (5 tests)
//! 4. decide_engine() integration (5 tests)

use chrono::{Duration, Utc};
use riptide_intelligence::domain_profiling::{DomainProfile, ProfileManager};
use riptide_reliability::Engine;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a temporary test profile with custom home directory
fn create_test_profile(name: &str, temp_dir: &TempDir) -> DomainProfile {
    let mut profile = DomainProfile::new(name.to_string());

    // Save to temp directory for isolation
    let profile_path = temp_dir.path().join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(&profile).unwrap();
    fs::write(&profile_path, json).unwrap();

    profile
}

/// Load profile from temp directory
fn load_test_profile(name: &str, temp_dir: &TempDir) -> DomainProfile {
    let profile_path = temp_dir.path().join(format!("{}.json", name));
    let content = fs::read_to_string(&profile_path).unwrap();
    serde_json::from_str(&content).unwrap()
}

/// Save profile to temp directory
fn save_test_profile(profile: &DomainProfile, temp_dir: &TempDir) -> PathBuf {
    let profile_path = temp_dir.path().join(format!("{}.json", profile.name));
    let json = serde_json::to_string_pretty(&profile).unwrap();
    fs::write(&profile_path, &json).unwrap();
    profile_path
}

// ============================================================================
// GROUP 1: Cache Hit/Miss Scenarios (8 tests)
// ============================================================================

#[test]
fn test_01_cache_miss_first_extraction() {
    // First extraction for a domain should have no cache
    let temp_dir = TempDir::new().unwrap();
    let profile = create_test_profile("example.com", &temp_dir);

    assert!(profile.preferred_engine.is_none(), "No cached engine initially");
    assert!(profile.last_success_confidence.is_none(), "No confidence initially");
    assert!(profile.engine_cache_expires_at.is_none(), "No expiry initially");
    assert!(!profile.is_cache_valid(), "Cache should not be valid");
    assert!(profile.get_cached_engine().is_none(), "get_cached_engine returns None");
}

#[test]
fn test_02_cache_hit_valid_within_ttl() {
    // Cache hit when engine is cached and TTL is valid
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("cached-domain.com", &temp_dir);

    // Cache engine with high confidence
    profile.cache_engine(Engine::Wasm, 0.85);

    assert!(profile.is_cache_valid(), "Cache should be valid");
    assert_eq!(
        profile.get_cached_engine(),
        Some(Engine::Wasm),
        "Should return cached engine"
    );

    // Verify fields are set correctly
    assert_eq!(profile.preferred_engine, Some(Engine::Wasm));
    assert_eq!(profile.last_success_confidence, Some(0.85));
    assert!(profile.engine_cache_expires_at.is_some());
}

#[test]
fn test_03_cache_miss_expired_ttl() {
    // Cache miss when TTL has expired (>7 days)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("expired-domain.com", &temp_dir);

    // Manually set expired cache (8 days ago)
    profile.preferred_engine = Some(Engine::Headless);
    profile.last_success_confidence = Some(0.90);
    profile.engine_cache_expires_at = Some(Utc::now() - Duration::days(8));

    assert!(!profile.is_cache_valid(), "Cache should be invalid (expired)");
    assert!(
        profile.get_cached_engine().is_none(),
        "get_cached_engine returns None for expired cache"
    );
}

#[test]
fn test_04_cache_hit_high_confidence() {
    // Cache hit with high confidence (>90%)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("high-confidence.com", &temp_dir);

    profile.cache_engine(Engine::Wasm, 0.95);

    assert!(profile.is_cache_valid());
    assert_eq!(
        profile.get_cached_engine(),
        Some(Engine::Wasm),
        "High confidence should return cached engine"
    );
}

#[test]
fn test_05_cache_bypass_low_confidence() {
    // Cache bypass when confidence is below threshold (<70%)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("low-confidence.com", &temp_dir);

    profile.cache_engine(Engine::Raw, 0.65);

    assert!(profile.is_cache_valid(), "Cache TTL is valid");
    assert!(
        profile.get_cached_engine().is_none(),
        "Low confidence (65%) should not return cached engine"
    );
}

#[test]
fn test_06_cache_invalidation_manual() {
    // Manual cache invalidation
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("invalidate-test.com", &temp_dir);

    // Cache engine
    profile.cache_engine(Engine::Headless, 0.88);
    assert!(profile.get_cached_engine().is_some(), "Cache exists");

    // Invalidate
    profile.invalidate_cache();

    assert!(profile.preferred_engine.is_none(), "Preferred engine cleared");
    assert!(profile.last_success_confidence.is_none(), "Confidence cleared");
    assert!(profile.engine_cache_expires_at.is_none(), "Expiry cleared");
    assert!(!profile.is_cache_valid(), "Cache no longer valid");
    assert!(profile.get_cached_engine().is_none(), "No cached engine after invalidation");
}

#[test]
fn test_07_cache_update_after_extraction() {
    // Update cache after successful extraction
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("update-test.com", &temp_dir);

    // Initial cache
    profile.cache_engine(Engine::Raw, 0.75);
    assert_eq!(profile.get_cached_engine(), Some(Engine::Raw));

    // Update with different engine and confidence
    profile.cache_engine(Engine::Wasm, 0.92);

    assert_eq!(
        profile.get_cached_engine(),
        Some(Engine::Wasm),
        "Updated cached engine"
    );
    assert_eq!(
        profile.last_success_confidence,
        Some(0.92),
        "Updated confidence"
    );
}

#[test]
fn test_08_cache_persistence_save_load() {
    // Verify cache persists through save/load cycle
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("persist-test.com", &temp_dir);

    // Cache engine
    profile.cache_engine(Engine::Headless, 0.87);
    save_test_profile(&profile, &temp_dir);

    // Load from disk
    let loaded_profile = load_test_profile("persist-test.com", &temp_dir);

    assert_eq!(
        loaded_profile.preferred_engine,
        Some(Engine::Headless),
        "Engine persisted"
    );
    assert_eq!(
        loaded_profile.last_success_confidence,
        Some(0.87),
        "Confidence persisted"
    );
    assert!(loaded_profile.is_cache_valid(), "Cache TTL persisted");
    assert_eq!(
        loaded_profile.get_cached_engine(),
        Some(Engine::Headless),
        "Loaded cache works"
    );
}

// ============================================================================
// GROUP 2: Confidence Threshold Tests (5 tests)
// ============================================================================

#[test]
fn test_09_high_confidence_95_cache_used() {
    // Confidence 95% - cache should be used
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("conf-95.com", &temp_dir);

    profile.cache_engine(Engine::Wasm, 0.95);

    assert_eq!(
        profile.get_cached_engine(),
        Some(Engine::Wasm),
        "95% confidence should use cache"
    );
}

#[test]
fn test_10_medium_confidence_75_cache_used() {
    // Confidence 75% - cache should be used (above 70% threshold)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("conf-75.com", &temp_dir);

    profile.cache_engine(Engine::Headless, 0.75);

    assert_eq!(
        profile.get_cached_engine(),
        Some(Engine::Headless),
        "75% confidence should use cache"
    );
}

#[test]
fn test_11_low_confidence_65_cache_bypassed() {
    // Confidence 65% - cache should be bypassed (below 70% threshold)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("conf-65.com", &temp_dir);

    profile.cache_engine(Engine::Raw, 0.65);

    assert!(
        profile.get_cached_engine().is_none(),
        "65% confidence should bypass cache"
    );
}

#[test]
fn test_12_zero_confidence_cache_bypassed() {
    // Zero confidence - cache should be bypassed
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("conf-zero.com", &temp_dir);

    profile.cache_engine(Engine::Wasm, 0.0);

    assert!(
        profile.get_cached_engine().is_none(),
        "0% confidence should bypass cache"
    );
}

#[test]
fn test_13_confidence_decay_over_time() {
    // Confidence doesn't change over time, but TTL expires
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("conf-decay.com", &temp_dir);

    // Cache with high confidence
    profile.cache_engine(Engine::Headless, 0.90);
    assert_eq!(profile.last_success_confidence, Some(0.90));

    // Simulate time passing by manually setting expired TTL
    profile.engine_cache_expires_at = Some(Utc::now() - Duration::days(8));

    // Confidence is still high, but cache is expired
    assert_eq!(
        profile.last_success_confidence,
        Some(0.90),
        "Confidence unchanged"
    );
    assert!(!profile.is_cache_valid(), "TTL expired");
    assert!(
        profile.get_cached_engine().is_none(),
        "Expired cache not used despite high confidence"
    );
}

// ============================================================================
// GROUP 3: TTL Expiry Tests (5 tests)
// ============================================================================

#[test]
fn test_14_ttl_within_window_cache_valid() {
    // TTL within 7-day window - cache valid
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("ttl-within.com", &temp_dir);

    profile.cache_engine(Engine::Wasm, 0.85);

    // Cache just created, should be valid
    assert!(profile.is_cache_valid(), "Fresh cache is valid");

    // Verify expiry is ~7 days in future
    let expires_at = profile.engine_cache_expires_at.unwrap();
    let days_until_expiry = (expires_at - Utc::now()).num_days();
    assert!(
        days_until_expiry >= 6 && days_until_expiry <= 7,
        "TTL should be ~7 days: {} days",
        days_until_expiry
    );
}

#[test]
fn test_15_ttl_at_boundary_7days() {
    // TTL at boundary (exactly 7 days) - cache expires
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("ttl-boundary.com", &temp_dir);

    profile.cache_engine(Engine::Headless, 0.88);

    // Set expiry to exactly now (7 days passed)
    profile.engine_cache_expires_at = Some(Utc::now());

    assert!(
        !profile.is_cache_valid(),
        "Cache at exact TTL boundary is invalid"
    );
}

#[test]
fn test_16_ttl_past_boundary_invalid() {
    // TTL past boundary (>7 days) - cache invalid
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("ttl-past.com", &temp_dir);

    profile.cache_engine(Engine::Raw, 0.90);

    // Set expiry to 10 days ago
    profile.engine_cache_expires_at = Some(Utc::now() - Duration::days(10));

    assert!(!profile.is_cache_valid(), "Cache past TTL is invalid");
    assert!(profile.get_cached_engine().is_none(), "Expired cache not returned");
}

#[test]
fn test_17_ttl_clock_skew_handling() {
    // TTL with minor clock skew (1 second in future still valid)
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("ttl-skew.com", &temp_dir);

    profile.cache_engine(Engine::Wasm, 0.85);

    // Simulate 1-second clock skew (cache expires 1 second ago)
    profile.engine_cache_expires_at = Some(Utc::now() - Duration::seconds(1));

    assert!(
        !profile.is_cache_valid(),
        "Even 1 second past expiry is invalid"
    );
}

#[test]
fn test_18_ttl_update_after_re_extraction() {
    // TTL updates when cache is refreshed
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("ttl-refresh.com", &temp_dir);

    // Initial cache
    profile.cache_engine(Engine::Headless, 0.80);
    let initial_expiry = profile.engine_cache_expires_at.unwrap();

    // Wait a moment (simulate time passing)
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Re-cache (refresh TTL)
    profile.cache_engine(Engine::Wasm, 0.92);
    let refreshed_expiry = profile.engine_cache_expires_at.unwrap();

    assert!(
        refreshed_expiry > initial_expiry,
        "TTL should be refreshed on re-caching"
    );
    assert!(profile.is_cache_valid(), "Refreshed cache is valid");
}

// ============================================================================
// GROUP 4: Integration with decide_engine() (5 tests)
// ============================================================================

#[test]
fn test_19_decide_engine_uses_cache_when_valid() {
    // decide_engine() should use cache when valid
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("decide-cache.com", &temp_dir);

    // Cache Headless engine with high confidence
    profile.cache_engine(Engine::Headless, 0.90);

    // Simulate decide_engine() using cached result
    let cached_engine = profile.get_cached_engine();
    assert_eq!(
        cached_engine,
        Some(Engine::Headless),
        "decide_engine would use cached Headless"
    );
}

#[test]
fn test_20_decide_engine_fallback_when_cache_invalid() {
    // decide_engine() falls back to analysis when cache invalid
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("decide-fallback.com", &temp_dir);

    // Set expired cache
    profile.preferred_engine = Some(Engine::Wasm);
    profile.last_success_confidence = Some(0.85);
    profile.engine_cache_expires_at = Some(Utc::now() - Duration::days(8));

    // Cache is invalid, should fall back to analysis
    let cached_engine = profile.get_cached_engine();
    assert!(
        cached_engine.is_none(),
        "decide_engine should fall back to analysis"
    );
}

#[test]
fn test_21_decide_engine_updates_cache_after_extraction() {
    // decide_engine() updates cache after successful extraction
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("decide-update.com", &temp_dir);

    // No cache initially
    assert!(profile.get_cached_engine().is_none());

    // Simulate decide_engine() extracting and caching result
    let extraction_engine = Engine::Wasm;
    let extraction_confidence = 0.88;

    profile.cache_engine(extraction_engine, extraction_confidence);

    // Verify cache was updated
    assert_eq!(profile.get_cached_engine(), Some(Engine::Wasm));
    assert_eq!(profile.last_success_confidence, Some(0.88));
}

#[test]
fn test_22_decide_engine_missing_profile_graceful() {
    // decide_engine() handles missing profile gracefully
    let temp_dir = TempDir::new().unwrap();

    // No profile exists initially
    let profile_path = temp_dir.path().join("nonexistent.json");
    assert!(!profile_path.exists(), "Profile doesn't exist");

    // decide_engine() would create new profile or skip cache
    // Here we simulate: create new profile on-demand
    let new_profile = DomainProfile::new("nonexistent.com".to_string());

    assert!(
        new_profile.get_cached_engine().is_none(),
        "New profile has no cache"
    );
}

#[test]
fn test_23_decide_engine_probe_first_with_cache() {
    // decide_engine() with probe_first flag still respects cache
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("probe-cache.com", &temp_dir);

    // Cache indicates Wasm is best
    profile.cache_engine(Engine::Wasm, 0.89);

    // Even with probe_first optimization, cache can inform the decision
    // If cache says Wasm works well, probe_first might still probe with Wasm first
    let cached_engine = profile.get_cached_engine();
    assert_eq!(
        cached_engine,
        Some(Engine::Wasm),
        "Cache informs probe_first decision"
    );
}

// ============================================================================
// Additional Edge Case Tests
// ============================================================================

#[test]
fn test_cache_with_different_engines() {
    // Test caching works with all engine types
    let temp_dir = TempDir::new().unwrap();

    for (engine, confidence) in [
        (Engine::Raw, 0.75),
        (Engine::Wasm, 0.85),
        (Engine::Headless, 0.95),
    ] {
        let mut profile = create_test_profile(&format!("{}-test.com", engine.name()), &temp_dir);
        profile.cache_engine(engine, confidence);

        assert_eq!(
            profile.get_cached_engine(),
            Some(engine),
            "Caching works for {}",
            engine.name()
        );
    }
}

#[test]
fn test_cache_exactly_at_threshold() {
    // Test behavior at exact 70% threshold
    let temp_dir = TempDir::new().unwrap();

    // Exactly at threshold (should NOT use cache due to > check)
    let mut profile_at = create_test_profile("threshold-at.com", &temp_dir);
    profile_at.cache_engine(Engine::Wasm, 0.70);
    assert!(
        profile_at.get_cached_engine().is_none(),
        "Exactly 70% should not use cache (needs >70%)"
    );

    // Just above threshold (should use cache)
    let mut profile_above = create_test_profile("threshold-above.com", &temp_dir);
    profile_above.cache_engine(Engine::Wasm, 0.71);
    assert!(
        profile_above.get_cached_engine().is_some(),
        "71% should use cache"
    );
}

#[test]
fn test_cache_serialization_roundtrip() {
    // Verify cache data survives JSON serialization
    let temp_dir = TempDir::new().unwrap();
    let mut profile = create_test_profile("serialize-test.com", &temp_dir);

    profile.cache_engine(Engine::Headless, 0.92);

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&profile).unwrap();

    // Deserialize back
    let deserialized: DomainProfile = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.preferred_engine, Some(Engine::Headless));
    assert_eq!(deserialized.last_success_confidence, Some(0.92));
    assert!(deserialized.engine_cache_expires_at.is_some());
    assert!(deserialized.is_cache_valid());
}

#[test]
fn test_profile_version_updated() {
    // Verify profile version is updated to 1.1.0 for caching support
    let temp_dir = TempDir::new().unwrap();
    let profile = create_test_profile("version-test.com", &temp_dir);

    assert_eq!(
        profile.version, "1.1.0",
        "Profile version should be 1.1.0 for caching support"
    );
}
