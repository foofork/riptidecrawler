//! Header consistency management
//!
//! This module ensures that HTTP headers are consistent with the user agent
//! and other browser characteristics to avoid fingerprinting detection.

use std::collections::HashMap;

/// Header consistency manager
pub struct HeaderConsistencyManager;

impl HeaderConsistencyManager {
    /// Generate consistent headers based on user agent
    pub fn generate_consistent_headers(user_agent: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        // Detect browser type
        let is_chrome = user_agent.contains("Chrome") && !user_agent.contains("Edg");
        let is_firefox = user_agent.contains("Firefox");
        let is_safari = user_agent.contains("Safari") && !user_agent.contains("Chrome");
        let is_edge = user_agent.contains("Edg");

        // Detect platform
        let is_windows = user_agent.contains("Windows");
        let is_mac = user_agent.contains("Mac OS X") || user_agent.contains("Macintosh");
        let is_linux = user_agent.contains("Linux") && !user_agent.contains("Android");
        let is_mobile = user_agent.contains("Mobile") || user_agent.contains("Android");

        // Extract Chrome version if present
        let chrome_version = if is_chrome || is_edge {
            Self::extract_version(user_agent, "Chrome/")
        } else {
            None
        };

        // sec-ch-ua headers (Chrome/Edge only)
        if is_chrome || is_edge {
            if let Some(version) = chrome_version {
                let major_version = version.split('.').next().unwrap_or("120");

                if is_edge {
                    headers.insert(
                        "sec-ch-ua".to_string(),
                        format!(
                            r#""Microsoft Edge";v="{}", "Chromium";v="{}", "Not=A?Brand";v="99""#,
                            major_version, major_version
                        ),
                    );
                    headers.insert("sec-ch-ua-full-version".to_string(), version.clone());
                    headers.insert(
                        "sec-ch-ua-full-version-list".to_string(),
                        format!(r#""Microsoft Edge";v="{}""#, version),
                    );
                } else {
                    headers.insert(
                        "sec-ch-ua".to_string(),
                        format!(
                            r#""Google Chrome";v="{}", "Chromium";v="{}", "Not=A?Brand";v="99""#,
                            major_version, major_version
                        ),
                    );
                    headers.insert("sec-ch-ua-full-version".to_string(), version.clone());
                }

                // Mobile hint
                headers.insert(
                    "sec-ch-ua-mobile".to_string(),
                    if is_mobile { "?1" } else { "?0" }.to_string(),
                );

                // Platform hint
                let platform = if is_windows {
                    "Windows"
                } else if is_mac {
                    "macOS"
                } else if is_linux {
                    "Linux"
                } else {
                    "Unknown"
                };
                headers.insert(
                    "sec-ch-ua-platform".to_string(),
                    format!(r#""{}""#, platform),
                );
                headers.insert(
                    "sec-ch-ua-platform-version".to_string(),
                    Self::get_platform_version(platform),
                );
            }
        }

        // Platform-specific headers
        if is_windows {
            headers.insert("sec-ch-ua-bitness".to_string(), r#""64""#.to_string());
            headers.insert("sec-ch-ua-arch".to_string(), r#""x86""#.to_string());
        } else if is_mac {
            headers.insert("sec-ch-ua-arch".to_string(), r#""arm""#.to_string());
        }

        // Common headers
        headers.insert(
            "accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".to_string(),
        );

        headers.insert(
            "accept-encoding".to_string(),
            "gzip, deflate, br".to_string(),
        );

        // Language headers (default to en-US)
        headers.insert("accept-language".to_string(), "en-US,en;q=0.9".to_string());

        // Security headers
        headers.insert("sec-fetch-dest".to_string(), "document".to_string());
        headers.insert("sec-fetch-mode".to_string(), "navigate".to_string());
        headers.insert("sec-fetch-site".to_string(), "none".to_string());
        headers.insert("sec-fetch-user".to_string(), "?1".to_string());

        // Upgrade insecure requests
        headers.insert("upgrade-insecure-requests".to_string(), "1".to_string());

        // DNT (Do Not Track)
        headers.insert("dnt".to_string(), "1".to_string());

        headers
    }

    /// Generate headers consistent with locale
    pub fn add_locale_headers(
        headers: &mut HashMap<String, String>,
        locale: &str,
        locales: &[String],
    ) {
        // Generate Accept-Language header
        let mut lang_parts = vec![format!("{},", locale)];

        // Add additional locales with decreasing quality
        let mut quality = 0.9;
        for additional_locale in locales.iter().filter(|l| *l != locale).take(3) {
            lang_parts.push(format!("{};q={:.1},", additional_locale, quality));
            quality -= 0.1;
        }

        // Add base language with lowest quality
        if locale.contains('-') {
            let base_lang = locale.split('-').next().unwrap();
            lang_parts.push(format!("{};q=0.5", base_lang));
        }

        let accept_language = lang_parts.concat().trim_end_matches(',').to_string();
        headers.insert("accept-language".to_string(), accept_language);
    }

    /// Validate header consistency
    pub fn validate_consistency(
        user_agent: &str,
        headers: &HashMap<String, String>,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check sec-ch-ua consistency for Chrome/Edge
        if (user_agent.contains("Chrome") || user_agent.contains("Edg"))
            && !user_agent.contains("Firefox")
        {
            if !headers.contains_key("sec-ch-ua") {
                errors.push("Missing sec-ch-ua header for Chrome/Edge".to_string());
            }

            if !headers.contains_key("sec-ch-ua-mobile") {
                errors.push("Missing sec-ch-ua-mobile header for Chrome/Edge".to_string());
            }

            if !headers.contains_key("sec-ch-ua-platform") {
                errors.push("Missing sec-ch-ua-platform header for Chrome/Edge".to_string());
            }
        }

        // Check platform consistency
        if user_agent.contains("Windows") {
            if let Some(platform) = headers.get("sec-ch-ua-platform") {
                if !platform.contains("Windows") {
                    errors.push(format!(
                        "Platform mismatch: UA says Windows but header says {}",
                        platform
                    ));
                }
            }
        }

        if user_agent.contains("Mac OS X") || user_agent.contains("Macintosh") {
            if let Some(platform) = headers.get("sec-ch-ua-platform") {
                if !platform.contains("macOS") {
                    errors.push(format!(
                        "Platform mismatch: UA says macOS but header says {}",
                        platform
                    ));
                }
            }
        }

        // Check mobile consistency
        let is_mobile_ua = user_agent.contains("Mobile") || user_agent.contains("Android");
        if let Some(mobile_header) = headers.get("sec-ch-ua-mobile") {
            let is_mobile_header = mobile_header == "?1";
            if is_mobile_ua != is_mobile_header {
                errors.push(format!(
                    "Mobile mismatch: UA says {} but header says {}",
                    if is_mobile_ua { "mobile" } else { "desktop" },
                    if is_mobile_header {
                        "mobile"
                    } else {
                        "desktop"
                    }
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Extract version from user agent
    fn extract_version(user_agent: &str, prefix: &str) -> Option<String> {
        if let Some(start) = user_agent.find(prefix) {
            let version_start = start + prefix.len();
            let version_str = &user_agent[version_start..];

            // Find the end of the version (space or end of string)
            let version_end = version_str
                .find(' ')
                .or_else(|| version_str.find('/'))
                .unwrap_or(version_str.len());

            Some(version_str[..version_end].to_string())
        } else {
            None
        }
    }

    /// Get platform version string
    fn get_platform_version(platform: &str) -> String {
        match platform {
            "Windows" => r#""10.0.0""#.to_string(),
            "macOS" => r#""13.0.0""#.to_string(),
            "Linux" => r#""5.15.0""#.to_string(),
            _ => r#""0.0.0""#.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_headers() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        assert!(headers.contains_key("sec-ch-ua"));
        assert!(headers.contains_key("sec-ch-ua-mobile"));
        assert!(headers.contains_key("sec-ch-ua-platform"));

        // Validate
        assert!(HeaderConsistencyManager::validate_consistency(ua, &headers).is_ok());
    }

    #[test]
    fn test_firefox_headers() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        // Firefox shouldn't have sec-ch-ua headers
        assert!(!headers.contains_key("sec-ch-ua"));
    }

    #[test]
    fn test_edge_headers() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        assert!(headers.contains_key("sec-ch-ua"));
        let sec_ch_ua = headers.get("sec-ch-ua").unwrap();
        assert!(sec_ch_ua.contains("Microsoft Edge"));
    }

    #[test]
    fn test_mobile_headers() {
        let ua = "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        assert_eq!(headers.get("sec-ch-ua-mobile"), Some(&"?1".to_string()));
    }

    #[test]
    fn test_macos_headers() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        assert!(headers.get("sec-ch-ua-platform").unwrap().contains("macOS"));
    }

    #[test]
    fn test_locale_headers() {
        let mut headers = HashMap::new();
        let locales = vec![
            "en-US".to_string(),
            "en-GB".to_string(),
            "fr-FR".to_string(),
        ];

        HeaderConsistencyManager::add_locale_headers(&mut headers, "en-US", &locales);

        assert!(headers.contains_key("accept-language"));
        let accept_lang = headers.get("accept-language").unwrap();
        assert!(accept_lang.contains("en-US"));
        assert!(accept_lang.contains("en-GB"));
        assert!(accept_lang.contains("q="));
    }

    #[test]
    fn test_validation() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let headers = HeaderConsistencyManager::generate_consistent_headers(ua);

        // Should pass validation
        assert!(HeaderConsistencyManager::validate_consistency(ua, &headers).is_ok());

        // Test with inconsistent headers
        let mut bad_headers = headers.clone();
        bad_headers.insert("sec-ch-ua-platform".to_string(), r#""Linux""#.to_string());

        assert!(HeaderConsistencyManager::validate_consistency(ua, &bad_headers).is_err());
    }

    #[test]
    fn test_version_extraction() {
        let ua = "Chrome/120.0.6099.109 Safari/537.36";
        let version = HeaderConsistencyManager::extract_version(ua, "Chrome/");
        assert_eq!(version, Some("120.0.6099.109".to_string()));
    }
}
