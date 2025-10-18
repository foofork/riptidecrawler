//! Enhanced browser fingerprinting with context-aware generation
//!
//! This module provides advanced fingerprinting capabilities that integrate
//! with browser contexts and CDP operations for improved stealth.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fingerprint::{
    AudioConfig, BrowserFingerprint, CanvasConfig, FingerprintGenerator, HardwareConfig,
    WebGlConfig,
};

/// Enhanced fingerprint generator with browser context awareness
pub struct EnhancedFingerprintGenerator {
    /// Base fingerprint configuration
    base_config: FingerprintConfig,
    /// Cache of generated fingerprints by session
    session_cache: HashMap<String, BrowserFingerprint>,
}

/// Configuration for fingerprint generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    /// Consistency mode - maintain fingerprint across requests
    pub maintain_consistency: bool,
    /// Session duration before fingerprint rotation (seconds)
    pub session_duration_secs: u64,
    /// Enable context-aware generation based on user agent
    pub context_aware: bool,
    /// WebGL configuration
    pub webgl: WebGlConfig,
    /// Hardware configuration
    pub hardware: HardwareConfig,
    /// Canvas configuration
    pub canvas: CanvasConfig,
    /// Audio configuration
    pub audio: AudioConfig,
}

impl Default for FingerprintConfig {
    fn default() -> Self {
        Self {
            maintain_consistency: true,
            session_duration_secs: 3600,
            context_aware: true,
            webgl: WebGlConfig::default(),
            hardware: HardwareConfig::default(),
            canvas: CanvasConfig::default(),
            audio: AudioConfig::default(),
        }
    }
}

impl EnhancedFingerprintGenerator {
    /// Create a new enhanced fingerprint generator
    pub fn new(config: FingerprintConfig) -> Self {
        Self {
            base_config: config,
            session_cache: HashMap::new(),
        }
    }

    /// Create with default configuration
    // Renamed from 'default' to avoid confusion with std::default::Default trait
    pub fn with_default_config() -> Self {
        Self::new(FingerprintConfig::default())
    }

    /// Generate a context-aware fingerprint based on user agent and browser type
    pub fn generate_contextual(
        &mut self,
        user_agent: &str,
        session_id: Option<&str>,
    ) -> BrowserFingerprint {
        // Check session cache first
        if let Some(session_id) = session_id {
            if self.base_config.maintain_consistency {
                if let Some(cached) = self.session_cache.get(session_id) {
                    return cached.clone();
                }
            }
        }

        // Generate new fingerprint with context awareness
        let mut fingerprint = FingerprintGenerator::generate();

        if self.base_config.context_aware {
            // Extract browser type and OS from user agent
            let (browser_type, os_type) = self.parse_user_agent(user_agent);

            // Adjust fingerprint based on browser/OS combination
            self.apply_browser_context(&mut fingerprint, &browser_type, &os_type);
        }

        fingerprint.user_agent = user_agent.to_string();

        // Cache if session tracking is enabled
        if let Some(session_id) = session_id {
            if self.base_config.maintain_consistency {
                self.session_cache
                    .insert(session_id.to_string(), fingerprint.clone());
            }
        }

        fingerprint
    }

    /// Generate CDP-compatible stealth parameters
    pub fn generate_cdp_params(&self, fingerprint: &BrowserFingerprint) -> CdpStealthParams {
        CdpStealthParams {
            user_agent: fingerprint.user_agent.clone(),
            platform: fingerprint.platform.clone(),
            viewport: fingerprint.screen_resolution,
            locale: fingerprint.language.clone(),
            timezone: fingerprint.timezone_name.clone(),
            webgl_vendor: fingerprint.webgl_vendor.clone(),
            webgl_renderer: fingerprint.webgl_renderer.clone(),
            hardware_concurrency: fingerprint.hardware_concurrency,
            device_memory: fingerprint.device_memory,
        }
    }

    /// Generate batch CDP headers for multiple requests
    pub fn generate_batch_headers(
        &self,
        fingerprint: &BrowserFingerprint,
        count: usize,
    ) -> Vec<HashMap<String, String>> {
        let mut headers_batch = Vec::with_capacity(count);

        for _ in 0..count {
            let headers = FingerprintGenerator::generate_consistent_headers(fingerprint);
            headers_batch.push(headers);
        }

        headers_batch
    }

    /// Parse user agent to extract browser and OS information
    fn parse_user_agent(&self, user_agent: &str) -> (String, String) {
        let browser_type = if user_agent.contains("Chrome") && !user_agent.contains("Chromium") {
            "Chrome".to_string()
        } else if user_agent.contains("Firefox") {
            "Firefox".to_string()
        } else if user_agent.contains("Safari") && !user_agent.contains("Chrome") {
            "Safari".to_string()
        } else {
            "Chrome".to_string() // Default
        };

        let os_type = if user_agent.contains("Windows") {
            "Windows".to_string()
        } else if user_agent.contains("Macintosh") || user_agent.contains("Mac OS") {
            "macOS".to_string()
        } else if user_agent.contains("Linux") {
            "Linux".to_string()
        } else {
            "Windows".to_string() // Default
        };

        (browser_type, os_type)
    }

    /// Apply browser-specific context to fingerprint
    fn apply_browser_context(
        &self,
        fingerprint: &mut BrowserFingerprint,
        _browser_type: &str,
        os_type: &str,
    ) {
        // Adjust platform based on OS
        fingerprint.platform = match os_type {
            "Windows" => "Win32".to_string(),
            "macOS" => "MacIntel".to_string(),
            "Linux" => "Linux x86_64".to_string(),
            _ => "Win32".to_string(),
        };

        // Adjust hardware specs based on OS (realistic configurations)
        let mut rng = rand::thread_rng();
        match os_type {
            "macOS" => {
                // MacBooks typically have 8-16 cores, 8-32GB RAM
                fingerprint.hardware_concurrency = [8, 10, 12, 16][rng.gen_range(0..4)];
                fingerprint.device_memory = [8, 16, 32][rng.gen_range(0..3)];
            }
            "Windows" => {
                // Windows PCs vary widely
                fingerprint.hardware_concurrency = [4, 6, 8, 12, 16][rng.gen_range(0..5)];
                fingerprint.device_memory = [8, 16, 32][rng.gen_range(0..3)];
            }
            "Linux" => {
                // Linux users tend to have powerful machines
                fingerprint.hardware_concurrency = [8, 12, 16, 24][rng.gen_range(0..4)];
                fingerprint.device_memory = [16, 32, 64][rng.gen_range(0..3)];
            }
            _ => {}
        }

        // Adjust WebGL based on OS
        match os_type {
            "macOS" => {
                let gpus = [
                    ("Apple", "Apple M1"),
                    ("Apple", "Apple M2"),
                    ("Apple", "Apple M3"),
                    ("AMD", "AMD Radeon Pro 5500M"),
                ];
                let (vendor, renderer) = gpus[rng.gen_range(0..gpus.len())];
                fingerprint.webgl_vendor = vendor.to_string();
                fingerprint.webgl_renderer = renderer.to_string();
            }
            "Windows" => {
                let gpus = [
                    ("NVIDIA Corporation", "NVIDIA GeForce RTX 3060"),
                    ("NVIDIA Corporation", "NVIDIA GeForce RTX 4060"),
                    ("AMD", "AMD Radeon RX 6600 XT"),
                    ("Intel Inc.", "Intel UHD Graphics 630"),
                ];
                let (vendor, renderer) = gpus[rng.gen_range(0..gpus.len())];
                fingerprint.webgl_vendor = vendor.to_string();
                fingerprint.webgl_renderer = renderer.to_string();
            }
            _ => {}
        }
    }

    /// Clear session cache for memory management
    pub fn clear_cache(&mut self) {
        self.session_cache.clear();
    }

    /// Remove specific session from cache
    pub fn remove_session(&mut self, session_id: &str) {
        self.session_cache.remove(session_id);
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.session_cache.len()
    }
}

/// CDP stealth parameters for browser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpStealthParams {
    pub user_agent: String,
    pub platform: String,
    pub viewport: (u32, u32),
    pub locale: String,
    pub timezone: String,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
    pub hardware_concurrency: u32,
    pub device_memory: u32,
}

impl CdpStealthParams {
    /// Convert to CDP command parameters for Page.setUserAgentOverride
    pub fn to_user_agent_override(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "userAgent".to_string(),
            serde_json::Value::String(self.user_agent.clone()),
        );
        params.insert(
            "platform".to_string(),
            serde_json::Value::String(self.platform.clone()),
        );
        params.insert(
            "acceptLanguage".to_string(),
            serde_json::Value::String(self.locale.clone()),
        );
        params
    }

    /// Convert to CDP command parameters for Emulation.setTimezoneOverride
    pub fn to_timezone_override(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "timezoneId".to_string(),
            serde_json::Value::String(self.timezone.clone()),
        );
        params
    }

    /// Convert to CDP command parameters for Emulation.setDeviceMetricsOverride
    pub fn to_device_metrics_override(&self) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "width".to_string(),
            serde_json::Value::Number(self.viewport.0.into()),
        );
        params.insert(
            "height".to_string(),
            serde_json::Value::Number(self.viewport.1.into()),
        );
        params.insert(
            "deviceScaleFactor".to_string(),
            serde_json::Value::Number(1.into()),
        );
        params.insert("mobile".to_string(), serde_json::Value::Bool(false));
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_generator_creation() {
        let generator = EnhancedFingerprintGenerator::with_default_config();
        assert_eq!(generator.cache_size(), 0);
    }

    #[test]
    fn test_contextual_generation() {
        let mut generator = EnhancedFingerprintGenerator::with_default_config();

        let ua_chrome_win = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let fingerprint = generator.generate_contextual(ua_chrome_win, Some("session1"));

        assert_eq!(fingerprint.platform, "Win32");
        assert_eq!(generator.cache_size(), 1);
    }

    #[test]
    fn test_session_consistency() {
        let mut generator = EnhancedFingerprintGenerator::with_default_config();

        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36";
        let fp1 = generator.generate_contextual(ua, Some("session1"));
        let fp2 = generator.generate_contextual(ua, Some("session1"));

        // Should return same fingerprint for same session
        assert_eq!(fp1.webgl_vendor, fp2.webgl_vendor);
        assert_eq!(fp1.hardware_concurrency, fp2.hardware_concurrency);
    }

    #[test]
    fn test_user_agent_parsing() {
        let generator = EnhancedFingerprintGenerator::with_default_config();

        let (browser, os) = generator.parse_user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        );
        assert_eq!(browser, "Chrome");
        assert_eq!(os, "Windows");

        let (browser, os) = generator.parse_user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"
        );
        assert_eq!(browser, "Safari");
        assert_eq!(os, "macOS");
    }

    #[test]
    fn test_cdp_params_generation() {
        let generator = EnhancedFingerprintGenerator::with_default_config();
        let fingerprint = FingerprintGenerator::generate();
        let cdp_params = generator.generate_cdp_params(&fingerprint);

        assert!(!cdp_params.user_agent.is_empty());
        assert!(!cdp_params.platform.is_empty());
        assert!(cdp_params.viewport.0 > 0);
        assert!(cdp_params.viewport.1 > 0);
    }

    #[test]
    fn test_batch_headers_generation() {
        let generator = EnhancedFingerprintGenerator::with_default_config();
        let fingerprint = FingerprintGenerator::generate();
        let headers_batch = generator.generate_batch_headers(&fingerprint, 5);

        assert_eq!(headers_batch.len(), 5);
        for headers in headers_batch {
            assert!(!headers.is_empty());
        }
    }

    #[test]
    fn test_cache_management() {
        let mut generator = EnhancedFingerprintGenerator::with_default_config();

        generator.generate_contextual("ua1", Some("session1"));
        generator.generate_contextual("ua2", Some("session2"));
        assert_eq!(generator.cache_size(), 2);

        generator.remove_session("session1");
        assert_eq!(generator.cache_size(), 1);

        generator.clear_cache();
        assert_eq!(generator.cache_size(), 0);
    }
}
