//! Secure secrets handling with automatic redaction in Debug output
//!
//! This module provides utilities for handling sensitive data like API keys,
//! passwords, and tokens. All secrets are automatically redacted in Debug
//! output to prevent accidental credential exposure in logs.
//!
//! # Examples
//!
//! ```
//! use riptide_types::secrets::SecretString;
//!
//! // Create a secret - it will never be exposed in Debug output
//! let api_key = SecretString::new("sk_test_abcdefghijklmnopqrstuvwxyz".to_string());
//!
//! // Debug output shows only first 4 characters
//! println!("{:?}", api_key); // Output: SecretString("sk_t...")
//!
//! // Access the actual value when needed
//! let key_value = api_key.expose_secret();
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// A string that contains sensitive data and is automatically redacted in Debug output.
///
/// When displayed via Debug, only the first 4 characters are shown, followed by "...".
/// This prevents accidental credential exposure in logs while still providing enough
/// information to identify different secrets.
#[derive(Clone, Serialize, Deserialize)]
pub struct SecretString(String);

impl SecretString {
    /// Create a new secret string
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Expose the secret value (use with caution)
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// Get the length of the secret without exposing it
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the secret is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redacted = redact_secret(&self.0);
        write!(f, "SecretString(\"{}\")", redacted)
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Redact a secret string, showing only the first 4 characters
///
/// # Examples
///
/// ```
/// use riptide_types::secrets::redact_secret;
///
/// assert_eq!(redact_secret("sk_test_abcdefghijklmnopqrstuvwxyz"), "sk_t...");
/// assert_eq!(redact_secret("short"), "shor...");
/// assert_eq!(redact_secret(""), "");
/// ```
pub fn redact_secret(secret: &str) -> String {
    if secret.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 4 {
        format!("{}...", secret)
    } else {
        format!("{}...", chars[..4].iter().collect::<String>())
    }
}

/// Redact a list of secrets for Debug output
///
/// # Examples
///
/// ```
/// use riptide_types::secrets::redact_secrets;
///
/// let keys = vec!["key1".to_string(), "key2".to_string()];
/// let redacted = redact_secrets(&keys);
/// assert_eq!(redacted, vec!["key1...", "key2..."]);
/// ```
pub fn redact_secrets(secrets: &[String]) -> Vec<String> {
    secrets.iter().map(|s| redact_secret(s)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_creation() {
        let secret = SecretString::new("my_secret_key_12345".to_string());
        assert_eq!(secret.expose_secret(), "my_secret_key_12345");
        assert_eq!(secret.len(), 19);
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_secret_string_debug() {
        let secret = SecretString::new("sk_test_abcdefghijklmnopqrstuvwxyz".to_string());
        let debug_output = format!("{:?}", secret);
        assert_eq!(debug_output, "SecretString(\"sk_t...\")");
        assert!(!debug_output.contains("abcdefghijklmnopqrstuvwxyz"));
    }

    #[test]
    fn test_secret_string_from_string() {
        let secret = SecretString::from("test_key".to_string());
        assert_eq!(secret.expose_secret(), "test_key");
    }

    #[test]
    fn test_secret_string_from_str() {
        let secret = SecretString::from("test_key");
        assert_eq!(secret.expose_secret(), "test_key");
    }

    #[test]
    fn test_redact_secret() {
        assert_eq!(
            redact_secret("sk_test_abcdefghijklmnopqrstuvwxyz"),
            "sk_t..."
        );
        assert_eq!(redact_secret("short"), "shor...");
        assert_eq!(redact_secret("abc"), "abc...");
        assert_eq!(redact_secret(""), "");
        assert_eq!(redact_secret("a"), "a...");
    }

    #[test]
    fn test_redact_secrets() {
        let secrets = vec![
            "key1_very_long_secret".to_string(),
            "key2_another_secret".to_string(),
            "key3".to_string(),
        ];
        let redacted = redact_secrets(&secrets);
        assert_eq!(redacted, vec!["key1...", "key2...", "key3..."]);
    }

    #[test]
    fn test_secret_not_in_debug_output() {
        let secret = SecretString::new("super_secret_password_12345".to_string());
        let debug_str = format!("{:?}", secret);

        // Should show first 4 chars
        assert!(debug_str.contains("supe"));

        // Should NOT show the rest
        assert!(!debug_str.contains("secret_password_12345"));
        assert!(!debug_str.contains("super_secret_password_12345"));
    }

    #[test]
    fn test_empty_secret() {
        let secret = SecretString::new("".to_string());
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
        assert_eq!(format!("{:?}", secret), "SecretString(\"\")");
    }
}
