use riptide_api::validation::{validate_crawl_request, validate_deepsearch_request};
use riptide_api::models::{CrawlBody, DeepSearchBody};
use riptide_api::errors::ApiError;
use riptide_core::types::CrawlOptions;

#[cfg(test)]
mod crawl_validation_tests {
    use super::*;

    #[test]
    fn test_valid_single_url() {
        let body = CrawlBody {
            urls: vec!["https://example.com".to_string()],
            options: None,
        };

        assert!(validate_crawl_request(&body).is_ok());
    }

    #[test]
    fn test_valid_multiple_urls() {
        let body = CrawlBody {
            urls: vec![
                "https://example.com".to_string(),
                "http://test.org".to_string(),
                "https://www.rust-lang.org/learn".to_string(),
            ],
            options: None,
        };

        assert!(validate_crawl_request(&body).is_ok());
    }

    #[test]
    fn test_empty_urls_rejected() {
        let body = CrawlBody {
            urls: vec![],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("At least one URL is required"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_too_many_urls_rejected() {
        let body = CrawlBody {
            urls: vec!["https://example.com".to_string(); 101], // MAX is 100
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("Too many URLs"));
                assert!(message.contains("101"));
                assert!(message.contains("100"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_invalid_url_scheme_rejected() {
        let body = CrawlBody {
            urls: vec!["ftp://example.com".to_string()],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::InvalidUrl { url, message } => {
                assert_eq!(url, "ftp://example.com");
                assert!(message.contains("unsupported scheme"));
                assert!(message.contains("ftp"));
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_malformed_url_rejected() {
        let body = CrawlBody {
            urls: vec!["not-a-url".to_string()],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::InvalidUrl { .. } => {}, // Expected
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_localhost_urls_rejected() {
        let test_cases = vec![
            "http://localhost:8080",
            "https://127.0.0.1:3000",
            "http://::1/test",
            "https://localhost/path",
        ];

        for url in test_cases {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "URL should be rejected: {}", url);

            match result.unwrap_err() {
                ApiError::InvalidUrl { message, .. } => {
                    assert!(message.contains("private/localhost"));
                }
                _ => panic!("Expected InvalidUrl for {}", url),
            }
        }
    }

    #[test]
    fn test_private_ip_addresses_rejected() {
        let test_cases = vec![
            "http://10.0.0.1",
            "https://172.16.5.4",
            "http://192.168.1.100",
            "https://169.254.1.1", // link-local
        ];

        for url in test_cases {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "Private IP should be rejected: {}", url);

            match result.unwrap_err() {
                ApiError::InvalidUrl { message, .. } => {
                    assert!(message.contains("private/localhost"));
                }
                _ => panic!("Expected InvalidUrl for {}", url),
            }
        }
    }

    #[test]
    fn test_public_ip_addresses_allowed() {
        let test_cases = vec![
            "http://8.8.8.8",
            "https://1.1.1.1",
            "http://208.67.222.222",
        ];

        for url in test_cases {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_ok(), "Public IP should be allowed: {}", url);
        }
    }

    #[test]
    fn test_excessively_long_url_rejected() {
        let long_url = format!("https://example.com/{}", "a".repeat(2100)); // Over 2048 limit
        let body = CrawlBody {
            urls: vec![long_url.clone()],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::InvalidUrl { message, .. } => {
                assert!(message.contains("too long"));
                assert!(message.contains("2048"));
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_suspicious_file_extensions_rejected() {
        let suspicious_urls = vec![
            "https://example.com/malware.exe",
            "http://test.org/script.bat",
            "https://site.com/file.scr",
            "http://domain.com/virus.vbs",
        ];

        for url in suspicious_urls {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "Suspicious URL should be rejected: {}", url);

            match result.unwrap_err() {
                ApiError::InvalidUrl { message, .. } => {
                    assert!(message.contains("suspicious file extension"));
                }
                _ => panic!("Expected InvalidUrl for {}", url),
            }
        }
    }

    #[test]
    fn test_excessive_url_encoding_rejected() {
        let encoded_url = "https://example.com/".to_string() + &"%20".repeat(25); // Over 20 limit
        let body = CrawlBody {
            urls: vec![encoded_url.clone()],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::InvalidUrl { message, .. } => {
                assert!(message.contains("excessive URL encoding"));
            }
            _ => panic!("Expected InvalidUrl"),
        }
    }

    #[test]
    fn test_local_domain_names_rejected() {
        let local_domains = vec![
            "http://hostname.local",
            "https://device.localhost",
        ];

        for url in local_domains {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "Local domain should be rejected: {}", url);
        }
    }

    #[test]
    fn test_mixed_valid_and_invalid_urls() {
        let body = CrawlBody {
            urls: vec![
                "https://example.com".to_string(),      // Valid
                "http://localhost:8080".to_string(),    // Invalid - localhost
                "https://test.org".to_string(),         // Valid
            ],
            options: None,
        };

        let result = validate_crawl_request(&body);
        assert!(result.is_err(), "Should fail on first invalid URL");
    }
}

#[cfg(test)]
mod deepsearch_validation_tests {
    use super::*;

    #[test]
    fn test_valid_deepsearch_request() {
        let body = DeepSearchBody {
            query: "rust programming language".to_string(),
            limit: Some(10),
            country: Some("US".to_string()),
            locale: Some("en".to_string()),
            include_content: Some(true),
            crawl_options: None,
        };

        assert!(validate_deepsearch_request(&body).is_ok());
    }

    #[test]
    fn test_minimal_valid_deepsearch() {
        let body = DeepSearchBody {
            query: "test".to_string(),
            limit: None,
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        assert!(validate_deepsearch_request(&body).is_ok());
    }

    #[test]
    fn test_empty_query_rejected() {
        let body = DeepSearchBody {
            query: "".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("cannot be empty"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_whitespace_only_query_rejected() {
        let body = DeepSearchBody {
            query: "   \t\n  ".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("cannot be empty"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_excessively_long_query_rejected() {
        let body = DeepSearchBody {
            query: "a".repeat(501), // Over 500 limit
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("Query too long"));
                assert!(message.contains("501"));
                assert!(message.contains("500"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_zero_limit_rejected() {
        let body = DeepSearchBody {
            query: "test query".to_string(),
            limit: Some(0),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("must be greater than 0"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_excessive_limit_rejected() {
        let body = DeepSearchBody {
            query: "test query".to_string(),
            limit: Some(51), // Over 50 limit
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("limit too high"));
                assert!(message.contains("51"));
                assert!(message.contains("50"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_sql_injection_patterns_rejected() {
        let sql_patterns = vec![
            "test query' union select * from users--",
            "search'; drop table data; --",
            "query union select password from accounts",
            "test' or '1'='1' --",
        ];

        for query in sql_patterns {
            let body = DeepSearchBody {
                query: query.to_string(),
                limit: Some(10),
                country: None,
                locale: None,
                include_content: None,
                crawl_options: None,
            };

            let result = validate_deepsearch_request(&body);
            assert!(result.is_err(), "SQL injection should be detected: {}", query);

            match result.unwrap_err() {
                ApiError::ValidationError { message } => {
                    assert!(message.contains("suspicious SQL patterns"));
                }
                _ => panic!("Expected ValidationError for SQL injection"),
            }
        }
    }

    #[test]
    fn test_script_injection_patterns_rejected() {
        let script_patterns = vec![
            "test <script>alert('xss')</script>",
            "search javascript:alert(1)",
            "query data:text/html,<script>alert('xss')</script>",
        ];

        for query in script_patterns {
            let body = DeepSearchBody {
                query: query.to_string(),
                limit: Some(10),
                country: None,
                locale: None,
                include_content: None,
                crawl_options: None,
            };

            let result = validate_deepsearch_request(&body);
            assert!(result.is_err(), "Script injection should be detected: {}", query);

            match result.unwrap_err() {
                ApiError::ValidationError { message } => {
                    assert!(message.contains("suspicious script patterns"));
                }
                _ => panic!("Expected ValidationError for script injection"),
            }
        }
    }

    #[test]
    fn test_control_characters_rejected() {
        let query_with_controls = format!("test{}\x00query\x08", '\x01');
        let body = DeepSearchBody {
            query: query_with_controls,
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        let result = validate_deepsearch_request(&body);
        assert!(result.is_err());

        match result.unwrap_err() {
            ApiError::ValidationError { message } => {
                assert!(message.contains("invalid control characters"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_allowed_control_characters() {
        let body = DeepSearchBody {
            query: "test\tquery\nwith allowed chars".to_string(),
            limit: Some(10),
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        assert!(validate_deepsearch_request(&body).is_ok());
    }

    #[test]
    fn test_boundary_limits() {
        // Test exactly at the limit
        let body = DeepSearchBody {
            query: "a".repeat(500), // Exactly at limit
            limit: Some(50), // Exactly at limit
            country: None,
            locale: None,
            include_content: None,
            crawl_options: None,
        };

        assert!(validate_deepsearch_request(&body).is_ok());
    }
}

// Property-based tests for validation edge cases
#[cfg(test)]
mod validation_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_url_validation_with_random_domains(
            domain in "[a-zA-Z0-9-]{1,63}\\.[a-zA-Z]{2,6}"
        ) {
            let url = format!("https://{}", domain);
            let body = CrawlBody {
                urls: vec![url.clone()],
                options: None,
            };

            // Valid domain structure should pass basic validation
            // (may still fail on other checks like private IPs)
            let result = validate_crawl_request(&body);

            // If it fails, it should be for a specific security reason, not parsing
            if result.is_err() {
                match result.unwrap_err() {
                    ApiError::InvalidUrl { message, .. } => {
                        // Should be a security-related rejection, not a parsing error
                        prop_assert!(
                            message.contains("private") ||
                            message.contains("localhost") ||
                            message.contains("suspicious") ||
                            message.contains("encoding")
                        );
                    }
                    _ => {
                        prop_assert!(false, "Unexpected error type for domain: {}", domain);
                    }
                }
            }
        }

        #[test]
        fn test_query_validation_with_safe_strings(
            query in "[a-zA-Z0-9 .,!?-]{1,100}"
        ) {
            let body = DeepSearchBody {
                query: query.clone(),
                limit: Some(10),
                country: None,
                locale: None,
                include_content: None,
                crawl_options: None,
            };

            // Safe alphanumeric queries should always pass
            let result = validate_deepsearch_request(&body);
            prop_assert!(result.is_ok(), "Safe query should pass: {}", query);
        }

        #[test]
        fn test_url_count_limits(count in 1usize..100usize) {
            let urls = vec!["https://example.com".to_string(); count];
            let body = CrawlBody {
                urls,
                options: None,
            };

            // Should pass for counts within limit
            let result = validate_crawl_request(&body);
            prop_assert!(result.is_ok(), "URL count {} should be valid", count);
        }

        #[test]
        fn test_search_limit_bounds(limit in 1u32..50u32) {
            let body = DeepSearchBody {
                query: "test query".to_string(),
                limit: Some(limit),
                country: None,
                locale: None,
                include_content: None,
                crawl_options: None,
            };

            // All limits within bounds should pass
            let result = validate_deepsearch_request(&body);
            prop_assert!(result.is_ok(), "Limit {} should be valid", limit);
        }
    }
}

// Edge case tests for security validation
#[cfg(test)]
mod security_validation_tests {
    use super::*;

    #[test]
    fn test_ipv6_private_addresses() {
        let ipv6_addresses = vec![
            "http://[::1]:8080",           // loopback
            "https://[fe80::1]",           // link-local
            "http://[fd00::1]",            // unique local
        ];

        for url in ipv6_addresses {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "IPv6 private address should be rejected: {}", url);
        }
    }

    #[test]
    fn test_edge_case_url_schemes() {
        let schemes = vec![
            "javascript:",
            "data:",
            "file:",
            "chrome:",
            "about:",
        ];

        for scheme in schemes {
            let url = format!("{}//test", scheme);
            let body = CrawlBody {
                urls: vec![url.clone()],
                options: None,
            };

            let result = validate_crawl_request(&body);
            assert!(result.is_err(), "Non-HTTP scheme should be rejected: {}", url);
        }
    }

    #[test]
    fn test_url_with_auth_components() {
        let urls_with_auth = vec![
            "https://user:pass@example.com",
            "http://admin@test.org",
        ];

        for url in urls_with_auth {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            // These should be allowed (credentials in URL are valid HTTP)
            let result = validate_crawl_request(&body);
            assert!(result.is_ok(), "URL with auth should be allowed: {}", url);
        }
    }

    #[test]
    fn test_unicode_domains() {
        let unicode_urls = vec![
            "https://例え.テスト",
            "http://испытание.рф",
        ];

        for url in unicode_urls {
            let body = CrawlBody {
                urls: vec![url.to_string()],
                options: None,
            };

            // These may pass or fail depending on URL parsing,
            // but should not cause panics
            let _result = validate_crawl_request(&body);
        }
    }

    #[test]
    fn test_extremely_deep_paths() {
        let deep_path = "/".to_string() + &"path/".repeat(100);
        let url = format!("https://example.com{}", deep_path);

        let body = CrawlBody {
            urls: vec![url],
            options: None,
        };

        // Should pass if under length limit
        let _result = validate_crawl_request(&body);
    }
}