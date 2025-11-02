//! Integration tests for API key validation (AUTH-002)
//!
//! Tests comprehensive validation of API keys to prevent weak keys
//! from compromising the authentication system.

use riptide_config::api_key_validation::validate_api_key;
use riptide_config::AuthenticationConfig;

/// Test that valid strong API keys pass validation
#[test]
fn test_valid_strong_keys() {
    // 32+ character keys with good entropy
    let valid_keys = vec![
        "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6",
        "AbCdEf123456789GhIjKl987654321MnOpQr",
        "api_prod_1234567890abcdefghijklmnopqrstuvwxyz",
        "prod_key_51234567890aBcDeFgHiJkLmNoPqRsTuVwXyZ",
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789AbCdEfGh",
    ];

    for key in valid_keys {
        assert!(
            validate_api_key(key).is_ok(),
            "Valid key should pass: {}",
            key
        );
    }
}

/// Test that keys shorter than 32 characters are rejected
#[test]
fn test_reject_short_keys() {
    let short_keys = vec![
        "abc",                            // 3 chars
        "1234567890",                     // 10 chars
        "abc123def456ghi789",             // 18 chars
        "1234567890abcdef1234567890abc",  // 30 chars
        "1234567890abcdef1234567890abcd", // 31 chars
    ];

    for key in short_keys {
        let result = validate_api_key(key);
        assert!(
            result.is_err(),
            "Short key should be rejected: {} (len={})",
            key,
            key.len()
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("too short"),
            "Error should mention length: {}",
            err
        );
    }
}

/// Test that keys with weak patterns are rejected
#[test]
fn test_reject_weak_patterns() {
    let weak_keys = vec![
        // Common weak words
        ("test1234567890123456789012345678", "test"),
        ("password123456789012345678901234", "password"),
        ("admin1234567890123456789012345678", "admin"),
        ("demo12345678901234567890123456789", "demo"),
        ("example123456789012345678901234567", "example"),
        ("sample1234567890123456789012345678", "sample"),
        ("default123456789012345678901234567", "default"),
        ("changeme12345678901234567890123456", "changeme"),
        // Weak patterns at different positions
        ("1234567890123456789012345678test", "test"),
        ("123456789test0123456789012345678", "test"),
    ];

    for (key, pattern) in weak_keys {
        let result = validate_api_key(key);
        assert!(
            result.is_err(),
            "Key with weak pattern '{}' should be rejected: {}",
            pattern,
            key
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("weak pattern"),
            "Error should mention weak pattern: {}",
            err
        );
    }
}

/// Test that weak pattern detection is case-insensitive
#[test]
fn test_weak_patterns_case_insensitive() {
    let weak_keys = vec![
        "TEST1234567890123456789012345678",
        "TeSt1234567890123456789012345678",
        "PASSWORD12345678901234567890123",
        "PaSsWoRd12345678901234567890123",
        "ADMIN1234567890123456789012345678",
        "AdMiN1234567890123456789012345678",
    ];

    for key in weak_keys {
        assert!(
            validate_api_key(key).is_err(),
            "Case variation should still be rejected: {}",
            key
        );
    }
}

/// Test that keys must contain both letters and numbers
#[test]
fn test_require_alphanumeric() {
    // Only letters (32+ chars)
    let only_letters = vec![
        "abcdefghijklmnopqrstuvwxyzabcdefgh",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGH",
        "aBcDeFgHiJkLmNoPqRsTuVwXyZaBcDeFgH",
    ];

    for key in only_letters {
        let result = validate_api_key(key);
        assert!(
            result.is_err(),
            "Keys with only letters should be rejected: {}",
            key
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("number") || err.contains("numeric"),
            "Error should mention missing numbers: {}",
            err
        );
    }

    // Only numbers (32+ chars)
    let only_numbers = vec![
        "12345678901234567890123456789012",
        "98765432109876543210987654321098",
    ];

    for key in only_numbers {
        let result = validate_api_key(key);
        assert!(
            result.is_err(),
            "Keys with only numbers should be rejected: {}",
            key
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("letter") || err.contains("alphabetic"),
            "Error should mention missing letters: {}",
            err
        );
    }
}

/// Test that special characters are allowed
#[test]
fn test_special_characters_allowed() {
    let keys_with_special = vec![
        "a1b2-c3d4_e5f6.g7h8/i9j0k1l2m3n4o5p6",
        "prod_key_1234567890abcdefghijklmnopqrstuvwxyz",
        "api_prod_1234567890AbCdEfGhIjKlMnOpQrStUvWx",
        "xyk3y.1234567890.abcdefghijklmnopqr",
    ];

    for key in keys_with_special {
        assert!(
            validate_api_key(key).is_ok(),
            "Keys with special characters should be allowed: {}",
            key
        );
    }
}

/// Test AuthenticationConfig validation during construction
#[test]
#[should_panic(expected = "Invalid API key")]
fn test_auth_config_rejects_weak_key() {
    std::env::set_var("API_KEYS", "weak");
    std::env::set_var("REQUIRE_AUTH", "true");
    let _config = AuthenticationConfig::from_env();
}

/// Test AuthenticationConfig validation with multiple keys
#[test]
#[should_panic(expected = "Invalid API key")]
fn test_auth_config_rejects_one_weak_among_valid() {
    std::env::set_var(
        "API_KEYS",
        "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6,weak,api_prod_1234567890abcdefghijklmnopqrstuvwxyz",
    );
    std::env::set_var("REQUIRE_AUTH", "true");
    let _config = AuthenticationConfig::from_env();
}

/// Test that validation is skipped when auth is disabled
#[test]
fn test_validation_skipped_when_auth_disabled() {
    std::env::set_var("API_KEYS", "weak,test,123");
    std::env::set_var("REQUIRE_AUTH", "false");
    let config = AuthenticationConfig::from_env();
    assert!(!config.require_auth);
    assert_eq!(config.api_keys.len(), 3);
}

/// Test with_api_keys builder method validation
#[test]
#[should_panic(expected = "Invalid API key")]
fn test_with_api_keys_validates() {
    let config = AuthenticationConfig::default();
    let _config = config.with_api_keys(vec!["weak".to_string()]);
}

/// Test with_api_keys allows weak keys when auth is disabled
#[test]
fn test_with_api_keys_no_validation_when_auth_disabled() {
    let config = AuthenticationConfig::default().with_require_auth(false);
    let config = config.with_api_keys(vec!["weak".to_string()]);
    assert_eq!(config.api_keys.len(), 1);
}

/// Test error messages are descriptive
#[test]
fn test_descriptive_error_messages() {
    // Test short key error
    let result = validate_api_key("short");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("5 characters"));
    assert!(err.contains("minimum 32"));

    // Test weak pattern error
    let result = validate_api_key("test1234567890123456789012345678");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("weak pattern"));
    assert!(err.contains("test"));

    // Test missing numbers error
    let result = validate_api_key("abcdefghijklmnopqrstuvwxyzabcdefgh");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("number"));
}

/// Test boundary conditions
#[test]
fn test_boundary_conditions() {
    // Exactly 32 characters with good composition
    let key_32 = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6";
    assert_eq!(key_32.len(), 32);
    assert!(validate_api_key(key_32).is_ok());

    // 33 characters
    let key_33 = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p67";
    assert_eq!(key_33.len(), 33);
    assert!(validate_api_key(key_33).is_ok());

    // 31 characters (should fail)
    let key_31 = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p";
    assert_eq!(key_31.len(), 31);
    assert!(validate_api_key(key_31).is_err());
}
