//! Authentication Test Fixtures
//!
//! Provides reusable test data and utilities for authentication testing

use riptide_api::middleware::AuthConfig;

/// Test API keys for various scenarios
pub struct TestApiKeys;

impl TestApiKeys {
    /// Valid API key for basic tests
    pub fn valid() -> String {
        "test-api-key-12345".to_string()
    }

    /// Alternative valid key for multi-key tests
    pub fn valid_alt() -> String {
        "test-api-key-67890".to_string()
    }

    /// Invalid API key for negative tests
    pub fn invalid() -> String {
        "invalid-key-xxxxx".to_string()
    }

    /// Very long API key for edge case testing
    pub fn very_long() -> String {
        "a".repeat(10000)
    }

    /// API key with special characters
    pub fn with_special_chars() -> String {
        "key-!@#$%^&*()_+-=[]{}|;:',.<>?/~`".to_string()
    }

    /// API key with unicode characters
    pub fn with_unicode() -> String {
        "key-émojis-🔑-中文-العربية".to_string()
    }

    /// Empty API key
    pub fn empty() -> String {
        String::new()
    }

    /// Whitespace-only API key
    pub fn whitespace() -> String {
        "   ".to_string()
    }
}

/// SQL injection payloads for security testing
pub struct SqlInjectionPayloads;

impl SqlInjectionPayloads {
    pub fn all() -> Vec<String> {
        vec![
            "'; DROP TABLE users; --".to_string(),
            "' OR '1'='1".to_string(),
            "admin'--".to_string(),
            "1' UNION SELECT NULL--".to_string(),
            "'; DELETE FROM api_keys WHERE '1'='1".to_string(),
            "' OR 1=1--".to_string(),
            "' OR 'x'='x".to_string(),
            "1'; DROP TABLE sessions--".to_string(),
        ]
    }
}

/// Header injection payloads for security testing
pub struct HeaderInjectionPayloads;

impl HeaderInjectionPayloads {
    pub fn all() -> Vec<String> {
        vec![
            "valid-key\r\nX-Injected: malicious".to_string(),
            "valid-key\nSet-Cookie: session=hijacked".to_string(),
            "valid-key%0d%0aLocation: http://evil.com".to_string(),
            "key\r\nContent-Length: 0\r\n\r\nHTTP/1.1 200 OK".to_string(),
        ]
    }
}

/// Path traversal payloads for security testing
pub struct PathTraversalPayloads;

impl PathTraversalPayloads {
    pub fn all() -> Vec<String> {
        vec![
            "../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\system32".to_string(),
            "....//....//....//etc/passwd".to_string(),
            "../../../../../../etc/shadow".to_string(),
        ]
    }
}

/// XSS payloads for security testing
pub struct XssPayloads;

impl XssPayloads {
    pub fn all() -> Vec<String> {
        vec![
            "<script>alert('xss')</script>".to_string(),
            "<img src=x onerror=alert('xss')>".to_string(),
            "javascript:alert('xss')".to_string(),
            "<svg/onload=alert('xss')>".to_string(),
        ]
    }
}

/// Helper to create AuthConfig with test keys
pub struct AuthConfigBuilder;

impl AuthConfigBuilder {
    /// Create auth config with single valid key
    pub fn single_key() -> AuthConfig {
        AuthConfig::with_api_keys(vec![TestApiKeys::valid()])
    }

    /// Create auth config with multiple valid keys
    pub fn multiple_keys() -> AuthConfig {
        AuthConfig::with_api_keys(vec![TestApiKeys::valid(), TestApiKeys::valid_alt()])
    }

    /// Create auth config with no keys (auth disabled scenario)
    pub fn no_keys() -> AuthConfig {
        AuthConfig::with_api_keys(vec![])
    }

    /// Create auth config with special character keys
    pub fn special_keys() -> AuthConfig {
        AuthConfig::with_api_keys(vec![
            TestApiKeys::with_special_chars(),
            TestApiKeys::with_unicode(),
        ])
    }
}

/// Mock API key generator for testing
pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    /// Generate a random API key for testing
    pub fn random() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const KEY_LEN: usize = 32;

        let mut rng = rand::thread_rng();
        (0..KEY_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate multiple random API keys
    pub fn random_batch(count: usize) -> Vec<String> {
        (0..count).map(|_| Self::random()).collect()
    }

    /// Generate UUID-based API key
    pub fn uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// Rate limit test helpers
pub struct RateLimitHelpers;

impl RateLimitHelpers {
    /// Generate client ID for rate limit testing
    pub fn client_id(id: usize) -> String {
        format!("test-client-{}", id)
    }

    /// Generate multiple client IDs
    pub fn client_ids(count: usize) -> Vec<String> {
        (0..count).map(Self::client_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_generation() {
        let key = ApiKeyGenerator::random();
        assert_eq!(key.len(), 32);
        assert!(key.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_uuid_key_generation() {
        let key = ApiKeyGenerator::uuid();
        assert!(uuid::Uuid::parse_str(&key).is_ok());
    }

    #[test]
    fn test_batch_key_generation() {
        let keys = ApiKeyGenerator::random_batch(10);
        assert_eq!(keys.len(), 10);

        // Ensure all keys are unique
        let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique_keys.len(), 10);
    }

    #[test]
    fn test_sql_injection_payloads() {
        let payloads = SqlInjectionPayloads::all();
        assert!(!payloads.is_empty());
        assert!(payloads.iter().any(|p| p.contains("DROP TABLE")));
    }

    #[test]
    fn test_auth_config_builder() {
        let config = AuthConfigBuilder::single_key();
        assert!(tokio_test::block_on(config.is_valid_key(&TestApiKeys::valid())));

        let config = AuthConfigBuilder::multiple_keys();
        assert!(tokio_test::block_on(config.is_valid_key(&TestApiKeys::valid())));
        assert!(tokio_test::block_on(config.is_valid_key(&TestApiKeys::valid_alt())));
    }
}
