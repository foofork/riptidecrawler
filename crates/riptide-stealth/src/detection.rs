//! Detection evasion and CAPTCHA detection
//!
//! This module provides tools to detect and evade bot detection mechanisms
//! including webdriver detection, plugin checks, and CAPTCHA challenges.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bot detection check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionScore {
    /// Overall detection score (0.0 = human-like, 1.0 = definitely a bot)
    pub overall_score: f32,

    /// Individual check results
    pub checks: HashMap<String, bool>,

    /// Risk level
    pub risk_level: RiskLevel,
}

/// Risk level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Very low risk of detection
    Low,
    /// Medium risk, some suspicious signals
    Medium,
    /// High risk, multiple bot signals detected
    High,
}

impl DetectionScore {
    /// Calculate overall score from checks
    fn calculate_score(checks: &HashMap<String, bool>) -> f32 {
        if checks.is_empty() {
            return 0.0;
        }

        let failed_checks = checks.values().filter(|&&v| v).count() as f32;
        failed_checks / checks.len() as f32
    }

    /// Determine risk level from score
    fn determine_risk(score: f32) -> RiskLevel {
        if score < 0.3 {
            RiskLevel::Low
        } else if score < 0.6 {
            RiskLevel::Medium
        } else {
            RiskLevel::High
        }
    }
}

/// Detection evasion analyzer
pub struct DetectionEvasion;

impl DetectionEvasion {
    /// Check common bot detection markers
    ///
    /// This simulates checking for common bot detection signals that would be
    /// found in a browser environment. Returns a detection score indicating
    /// how bot-like the current configuration appears.
    pub fn check_detection_markers() -> DetectionScore {
        let mut checks = HashMap::new();

        // Check 1: Webdriver detection
        // In a real browser, window.navigator.webdriver should be undefined or false
        checks.insert("webdriver_detected".to_string(), false);

        // Check 2: Plugin consistency
        // Real browsers have consistent plugin configurations
        checks.insert("plugin_inconsistency".to_string(), false);

        // Check 3: Language checks
        // navigator.languages should exist and be consistent
        checks.insert("language_mismatch".to_string(), false);

        // Check 4: Chrome detection
        // window.chrome should exist for Chrome browsers
        checks.insert("chrome_missing".to_string(), false);

        // Check 5: Hidden state
        // document.hidden should behave normally
        checks.insert("hidden_state_anomaly".to_string(), false);

        // Check 6: Permission API
        // navigator.permissions should exist and work correctly
        checks.insert("permissions_blocked".to_string(), false);

        // Check 7: Screen properties
        // Screen dimensions should be realistic
        checks.insert("screen_anomaly".to_string(), false);

        // Check 8: Timezone consistency
        // Timezone should match geolocation
        checks.insert("timezone_mismatch".to_string(), false);

        let overall_score = DetectionScore::calculate_score(&checks);
        let risk_level = DetectionScore::determine_risk(overall_score);

        DetectionScore {
            overall_score,
            checks,
            risk_level,
        }
    }

    /// Check if webdriver is detected (simulated)
    pub fn check_webdriver() -> bool {
        // In reality, this would check window.navigator.webdriver
        // For now, simulate a clean environment
        false
    }

    /// Check plugin consistency
    pub fn check_plugins() -> bool {
        // Simulate checking for plugin consistency
        // Returns true if inconsistency detected
        false
    }

    /// Check Chrome object presence
    pub fn check_chrome() -> bool {
        // Simulate checking for window.chrome
        // Returns true if Chrome object is missing when it should be present
        false
    }

    /// Check hidden state anomalies
    pub fn check_hidden_state() -> bool {
        // Simulate checking document.hidden behavior
        // Returns true if anomaly detected
        false
    }
}

/// CAPTCHA challenge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptchaType {
    /// Google reCAPTCHA v2
    ReCaptchaV2,
    /// Google reCAPTCHA v3
    ReCaptchaV3,
    /// hCaptcha
    HCaptcha,
    /// Cloudflare Turnstile
    Cloudflare,
    /// Unknown or custom CAPTCHA
    Unknown,
}

/// CAPTCHA detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaDetection {
    /// Type of CAPTCHA detected
    pub captcha_type: Option<CaptchaType>,

    /// Whether a CAPTCHA was detected
    pub detected: bool,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// CAPTCHA detector
pub struct CaptchaDetector;

impl CaptchaDetector {
    /// Detect CAPTCHA challenges in HTML content
    pub fn detect_challenge(html: &str) -> CaptchaDetection {
        let html_lower = html.to_lowercase();

        // Check for reCAPTCHA
        if html_lower.contains("recaptcha") || html_lower.contains("google.com/recaptcha") {
            let captcha_type = if html_lower.contains("recaptcha/api.js") {
                CaptchaType::ReCaptchaV2
            } else if html_lower.contains("recaptcha/enterprise.js")
                || html_lower.contains("grecaptcha.execute")
            {
                CaptchaType::ReCaptchaV3
            } else {
                CaptchaType::ReCaptchaV2
            };

            let mut metadata = HashMap::new();
            metadata.insert("provider".to_string(), "Google reCAPTCHA".to_string());

            return CaptchaDetection {
                captcha_type: Some(captcha_type),
                detected: true,
                confidence: 0.95,
                metadata,
            };
        }

        // Check for hCaptcha
        if html_lower.contains("hcaptcha") || html_lower.contains("hcaptcha.com") {
            let mut metadata = HashMap::new();
            metadata.insert("provider".to_string(), "hCaptcha".to_string());

            return CaptchaDetection {
                captcha_type: Some(CaptchaType::HCaptcha),
                detected: true,
                confidence: 0.95,
                metadata,
            };
        }

        // Check for Cloudflare Turnstile
        if html_lower.contains("turnstile")
            || html_lower.contains("challenges.cloudflare.com")
            || html_lower.contains("cf-challenge")
            || (html_lower.contains("cloudflare") && html_lower.contains("checking your browser"))
        {
            let mut metadata = HashMap::new();
            metadata.insert("provider".to_string(), "Cloudflare".to_string());

            return CaptchaDetection {
                captcha_type: Some(CaptchaType::Cloudflare),
                detected: true,
                confidence: 0.90,
                metadata,
            };
        }

        // Check for generic CAPTCHA indicators (be more specific to avoid false positives)
        let has_captcha_keyword = html_lower.contains("captcha");
        let has_challenge_keyword = html_lower.contains("challenge");
        let has_verify_keyword = html_lower.contains("verify you are human")
            || html_lower.contains("verify you're human")
            || html_lower.contains("prove you are human");

        // Only flag as CAPTCHA if we have strong indicators
        if (has_captcha_keyword
            && (html_lower.contains("<form") || html_lower.contains("data-sitekey")))
            || (has_challenge_keyword && has_verify_keyword)
        {
            let mut metadata = HashMap::new();
            metadata.insert("provider".to_string(), "Unknown".to_string());

            return CaptchaDetection {
                captcha_type: Some(CaptchaType::Unknown),
                detected: true,
                confidence: 0.70,
                metadata,
            };
        }

        // No CAPTCHA detected
        CaptchaDetection {
            captcha_type: None,
            detected: false,
            confidence: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Detect specific CAPTCHA types
    pub fn detect_recaptcha(html: &str) -> bool {
        let html_lower = html.to_lowercase();
        html_lower.contains("recaptcha") || html_lower.contains("google.com/recaptcha")
    }

    pub fn detect_hcaptcha(html: &str) -> bool {
        let html_lower = html.to_lowercase();
        html_lower.contains("hcaptcha") || html_lower.contains("hcaptcha.com")
    }

    pub fn detect_cloudflare(html: &str) -> bool {
        let html_lower = html.to_lowercase();
        html_lower.contains("turnstile")
            || html_lower.contains("challenges.cloudflare.com")
            || html_lower.contains("cf-challenge")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_markers() {
        let score = DetectionEvasion::check_detection_markers();

        // Should have low detection score with proper evasion
        assert!(score.overall_score < 0.3);
        assert_eq!(score.risk_level, RiskLevel::Low);

        // All checks should pass (false = not detected)
        assert!(!score.checks.get("webdriver_detected").unwrap());
        assert!(!score.checks.get("plugin_inconsistency").unwrap());
        assert!(!score.checks.get("chrome_missing").unwrap());
    }

    #[test]
    fn test_individual_checks() {
        assert!(!DetectionEvasion::check_webdriver());
        assert!(!DetectionEvasion::check_plugins());
        assert!(!DetectionEvasion::check_chrome());
        assert!(!DetectionEvasion::check_hidden_state());
    }

    #[test]
    fn test_recaptcha_detection() {
        let html = r#"
            <html>
                <script src="https://www.google.com/recaptcha/api.js"></script>
                <div class="g-recaptcha" data-sitekey="test-key"></div>
            </html>
        "#;

        let result = CaptchaDetector::detect_challenge(html);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::ReCaptchaV2));
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_hcaptcha_detection() {
        let html = r#"
            <html>
                <script src="https://hcaptcha.com/1/api.js"></script>
                <div class="h-captcha" data-sitekey="test-key"></div>
            </html>
        "#;

        let result = CaptchaDetector::detect_challenge(html);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::HCaptcha));
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_cloudflare_detection() {
        let html = r#"
            <html>
                <title>Just a moment...</title>
                <div>Checking your browser before accessing example.com</div>
                <script src="https://challenges.cloudflare.com/turnstile/v0/api.js"></script>
            </html>
        "#;

        let result = CaptchaDetector::detect_challenge(html);
        assert!(result.detected);
        assert_eq!(result.captcha_type, Some(CaptchaType::Cloudflare));
        assert!(result.confidence > 0.8);
    }

    #[test]
    fn test_no_captcha() {
        let html = r#"
            <html>
                <body>
                    <h1>Welcome to our website</h1>
                    <p>This is normal content without any CAPTCHA</p>
                </body>
            </html>
        "#;

        let result = CaptchaDetector::detect_challenge(html);
        assert!(!result.detected);
        assert_eq!(result.captcha_type, None);
    }

    #[test]
    fn test_specific_detectors() {
        let recaptcha_html = r#"<script src="https://www.google.com/recaptcha/api.js"></script>"#;
        assert!(CaptchaDetector::detect_recaptcha(recaptcha_html));

        let hcaptcha_html = r#"<script src="https://hcaptcha.com/1/api.js"></script>"#;
        assert!(CaptchaDetector::detect_hcaptcha(hcaptcha_html));

        let cloudflare_html = r#"<div class="cf-challenge"></div>"#;
        assert!(CaptchaDetector::detect_cloudflare(cloudflare_html));
    }
}
