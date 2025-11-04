//! Tests for secrets redaction in Debug output
//!
//! This test suite verifies that sensitive data (API keys, passwords, tokens)
//! are properly redacted in Debug output to prevent accidental credential exposure.

use riptide_types::secrets::{redact_secret, redact_secrets, SecretString};

#[test]
fn test_secret_string_redacts_in_debug() {
    let api_key = SecretString::new("sk_test_abcdefghijklmnopqrstuvwxyz1234567890".to_string());
    let debug_output = format!("{:?}", api_key);

    // Should show first 4 chars
    assert!(debug_output.contains("sk_t"));

    // Should NOT show the rest of the secret
    assert!(!debug_output.contains("abcdefghijklmnopqrstuvwxyz"));
    assert!(!debug_output.contains("1234567890"));

    // Should contain redaction indicator
    assert!(debug_output.contains("..."));
}

#[test]
fn test_secret_string_short_values() {
    let short_secret = SecretString::new("abc".to_string());
    let debug_output = format!("{:?}", short_secret);

    // Should show all chars for short strings but still add ...
    assert!(debug_output.contains("abc"));
    assert!(debug_output.contains("..."));
}

#[test]
fn test_secret_string_empty() {
    let empty_secret = SecretString::new("".to_string());
    let debug_output = format!("{:?}", empty_secret);

    assert_eq!(debug_output, "SecretString(\"\")");
}

#[test]
fn test_secret_string_expose_secret() {
    let secret = SecretString::new("my_actual_secret_value".to_string());

    // Can still access the actual value when needed
    assert_eq!(secret.expose_secret(), "my_actual_secret_value");
}

#[test]
fn test_redact_secret_function() {
    // Test various secret formats
    assert_eq!(redact_secret("sk_test_1234567890"), "sk_t...");
    assert_eq!(redact_secret("api_key_abcdefghij"), "api_...");
    assert_eq!(redact_secret("password123456"), "pass...");
    assert_eq!(redact_secret("xyz"), "xyz...");
    assert_eq!(redact_secret(""), "");
}

#[test]
fn test_redact_secrets_multiple_keys() {
    let keys = vec![
        "key1_secret_value_long".to_string(),
        "key2_different_secret".to_string(),
        "key3_another_one".to_string(),
    ];

    let redacted = redact_secrets(&keys);

    assert_eq!(redacted.len(), 3);
    assert_eq!(redacted[0], "key1...");
    assert_eq!(redacted[1], "key2...");
    assert_eq!(redacted[2], "key3...");

    // None should contain the full secrets
    for r in &redacted {
        assert!(!r.contains("secret"));
        assert!(!r.contains("different"));
        assert!(!r.contains("another"));
    }
}

#[test]
fn test_no_secrets_in_format_output() {
    let api_key = SecretString::new("super_secret_api_key_12345678".to_string());

    // Test Debug output
    let debug_str = format!("{:?}", api_key);
    assert!(!debug_str.contains("super_secret_api_key_12345678"));
    assert!(!debug_str.contains("secret_api_key"));
    assert!(debug_str.contains("supe..."));
}

#[test]
fn test_multiple_secrets_different_lengths() {
    let secrets = vec![
        "a".to_string(),
        "ab".to_string(),
        "abc".to_string(),
        "abcd".to_string(),
        "abcde".to_string(),
        "abcdefghijklmnopqrstuvwxyz".to_string(),
    ];

    let redacted = redact_secrets(&secrets);

    // All should be redacted
    for (i, r) in redacted.iter().enumerate() {
        assert!(r.contains("..."), "Secret {} should contain ...", i);

        // First 4 chars should be preserved for longer strings
        if secrets[i].len() > 4 {
            let expected_prefix = secrets[i].chars().take(4).collect::<String>();
            assert!(
                r.starts_with(&expected_prefix),
                "Secret {} should start with {}",
                i,
                expected_prefix
            );
        }
    }
}

#[test]
fn test_secret_string_from_conversions() {
    let from_string = SecretString::from("test_key_123456".to_string());
    let from_str = SecretString::from("test_key_123456");

    assert_eq!(from_string.expose_secret(), "test_key_123456");
    assert_eq!(from_str.expose_secret(), "test_key_123456");

    // Both should redact in debug
    let debug1 = format!("{:?}", from_string);
    let debug2 = format!("{:?}", from_str);

    assert!(!debug1.contains("123456"));
    assert!(!debug2.contains("123456"));
}

#[test]
fn test_secret_string_length_methods() {
    let secret = SecretString::new("12345".to_string());

    assert_eq!(secret.len(), 5);
    assert!(!secret.is_empty());

    let empty = SecretString::new("".to_string());
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

#[test]
fn test_redaction_prevents_log_exposure() {
    // Simulating a log statement scenario
    let sensitive_key = SecretString::new("sk_live_production_key_abcdef123456".to_string());

    // What a developer might accidentally write in a log
    let log_message = format!("API Config: {:?}", sensitive_key);

    // The actual production key should NOT appear in the log
    assert!(!log_message.contains("production_key_abcdef123456"));
    assert!(!log_message.contains("sk_live_production_key_abcdef123456"));

    // But we can still identify which key it is by the prefix
    assert!(log_message.contains("sk_l"));
}

#[test]
fn test_unicode_secrets_redacted() {
    let unicode_secret = SecretString::new("🔐secret_key_with_emoji_🔑".to_string());
    let debug_output = format!("{:?}", unicode_secret);

    // Should not expose the full secret
    assert!(!debug_output.contains("secret_key_with_emoji"));

    // Should have redaction indicator
    assert!(debug_output.contains("..."));
}

#[test]
fn test_special_char_secrets() {
    let special_secret = SecretString::new("key!@#$%^&*()_+-=[]{}|;:,.<>?".to_string());
    let debug_output = format!("{:?}", special_secret);

    // Should redact
    assert!(debug_output.contains("..."));

    // Should not show full secret
    assert!(!debug_output.contains("!@#$%^&*()_+-=[]{}|;:,.<>?"));
}
