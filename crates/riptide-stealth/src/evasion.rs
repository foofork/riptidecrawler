//! Stealth evasion techniques and coordination
//!
//! This module contains the main StealthController that coordinates
//! all stealth techniques and provides the primary interface for
//! anti-detection measures.

use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;
use tracing::warn;

use crate::behavior::BehaviorSimulator;
use crate::config::{LocaleStrategy, StealthConfig, StealthPreset};
use crate::enhancements::{
    screen_resolution::ScreenResolution, HeaderConsistencyManager, ScreenResolutionManager,
    TimezoneManager, WebRtcEnhanced,
};
use crate::javascript::JavaScriptInjector;
use crate::rate_limiter::RateLimiter;
use crate::user_agent::UserAgentManager;

/// Stealth controller for managing anti-detection measures
pub struct StealthController {
    config: StealthConfig,
    user_agent_manager: UserAgentManager,
    request_count: u64,
    last_request_time: Instant,
    js_injector: Option<JavaScriptInjector>,
    // Enhanced features for crawl4ai parity
    screen_resolution_manager: ScreenResolutionManager,
    timezone_manager: TimezoneManager,
    webrtc_enhanced: WebRtcEnhanced,
    // Advanced anti-detection features
    rate_limiter: RateLimiter,
    behavior_simulator: BehaviorSimulator,
}

impl StealthController {
    /// Create a new stealth controller
    pub fn new(mut config: StealthConfig) -> Self {
        // Load user agents from file if configured
        if let Err(e) = config.load_user_agents_from_file() {
            warn!("Failed to load user agents from file: {}", e);
        }

        let user_agent_manager = UserAgentManager::new(config.user_agent.clone());

        // Initialize enhanced features based on preset
        let webrtc_enhanced = match config.preset {
            StealthPreset::High => WebRtcEnhanced::high_security(),
            StealthPreset::Medium => WebRtcEnhanced::default(),
            StealthPreset::Low => WebRtcEnhanced {
                block_ip_leak: true,
                ..Default::default()
            },
            StealthPreset::None => WebRtcEnhanced {
                block_completely: false,
                block_ip_leak: false,
                spoof_media_devices: false,
                block_data_channels: false,
                block_stun_turn: false,
                fake_ip: None,
            },
        };

        // Initialize rate limiter with default timing from config
        let rate_limiter = RateLimiter::new(config.timing.default_timing.clone());

        Self {
            config,
            user_agent_manager,
            request_count: 0,
            last_request_time: Instant::now(),
            js_injector: None,
            screen_resolution_manager: ScreenResolutionManager::new(),
            timezone_manager: TimezoneManager::new(),
            webrtc_enhanced,
            rate_limiter,
            behavior_simulator: BehaviorSimulator::new(),
        }
    }

    /// Create from preset
    pub fn from_preset(preset: StealthPreset) -> Self {
        Self::new(StealthConfig::from_preset(preset))
    }

    /// Get the next user agent based on rotation strategy
    pub fn next_user_agent(&mut self) -> &str {
        self.user_agent_manager.increment_request_count();
        self.user_agent_manager.next_user_agent()
    }

    /// Get current user agent without rotation
    pub fn current_user_agent(&self) -> Option<&String> {
        self.user_agent_manager.current_user_agent()
    }

    /// Get CDP flags for browser configuration
    pub fn get_cdp_flags(&self) -> Vec<String> {
        self.config.get_cdp_flags()
    }

    /// Get stealth preset
    pub fn get_preset(&self) -> &StealthPreset {
        &self.config.preset
    }

    /// Generate randomized headers for the request
    pub fn generate_headers(&self) -> HashMap<String, String> {
        // Use HeaderConsistencyManager for enhanced consistency
        let default_ua = String::from("Mozilla/5.0");
        let user_agent = self.current_user_agent().unwrap_or(&default_ua);
        let mut headers = HeaderConsistencyManager::generate_consistent_headers(user_agent);

        // Add locale-specific headers
        let mut rng = rand::thread_rng();
        let (locale, _) = self.random_locale();
        HeaderConsistencyManager::add_locale_headers(
            &mut headers,
            locale,
            &self.config.request_randomization.locale.locales,
        );

        // Add randomized custom headers if configured
        for (header_name, variations) in &self.config.request_randomization.headers.custom_headers {
            if !variations.is_empty() {
                let index = rng.gen_range(0..variations.len());
                headers.insert(header_name.clone(), variations[index].clone());
            }
        }

        headers
    }

    /// Calculate delay before next request
    pub fn calculate_delay(&mut self) -> std::time::Duration {
        self.request_count += 1;
        let mut rng = rand::thread_rng();

        let base_delay = self
            .config
            .request_randomization
            .timing_jitter
            .base_delay_ms;
        let jitter_range = (base_delay as f64
            * self
                .config
                .request_randomization
                .timing_jitter
                .jitter_percentage) as u64;
        let jitter = rng.gen_range(0..=jitter_range);

        let total_delay = if rng.gen_bool(0.5) {
            base_delay + jitter
        } else {
            base_delay.saturating_sub(jitter)
        };

        let clamped_delay = total_delay
            .max(self.config.request_randomization.timing_jitter.min_delay_ms)
            .min(self.config.request_randomization.timing_jitter.max_delay_ms);

        std::time::Duration::from_millis(clamped_delay)
    }

    /// Generate a random viewport size
    pub fn random_viewport(&self) -> (u32, u32) {
        let mut rng = rand::thread_rng();
        let base_size_index =
            rng.gen_range(0..self.config.request_randomization.viewport.sizes.len());
        let (mut width, mut height) =
            self.config.request_randomization.viewport.sizes[base_size_index];

        if self.config.request_randomization.viewport.add_variance {
            let variance = self.config.request_randomization.viewport.max_variance;
            let width_variance = rng.gen_range(0..=variance);
            let height_variance = rng.gen_range(0..=variance);

            if rng.gen_bool(0.5) {
                width += width_variance;
            } else {
                width = width.saturating_sub(width_variance);
            }

            if rng.gen_bool(0.5) {
                height += height_variance;
            } else {
                height = height.saturating_sub(height_variance);
            }
        }

        (width, height)
    }

    /// Get a random locale based on strategy
    pub fn random_locale(&self) -> (&str, &str) {
        let mut rng = rand::thread_rng();

        match &self.config.request_randomization.locale.strategy {
            LocaleStrategy::Random => {
                let index =
                    rng.gen_range(0..self.config.request_randomization.locale.locales.len());
                let locale = &self.config.request_randomization.locale.locales[index];
                let timezone = self
                    .config
                    .request_randomization
                    .locale
                    .timezones
                    .get(locale)
                    .map(|tz| tz.as_str())
                    .unwrap_or("America/New_York");
                (locale, timezone)
            }
            LocaleStrategy::Fixed(locale) => {
                let timezone = self
                    .config
                    .request_randomization
                    .locale
                    .timezones
                    .get(locale)
                    .map(|tz| tz.as_str())
                    .unwrap_or("America/New_York");
                (locale, timezone)
            }
            _ => {
                // Default to en-US for Geographic and TargetBased strategies
                ("en-US", "America/New_York")
            }
        }
    }

    /// Generate randomized hardware specifications
    pub fn random_hardware_specs(&self) -> (u32, u32) {
        self.config
            .fingerprinting
            .hardware
            .get_random_hardware_specs()
    }

    /// Get randomized WebGL vendor and renderer
    pub fn random_webgl_specs(&self) -> (String, String) {
        self.config.fingerprinting.webgl.get_random_webgl_specs()
    }

    /// Generate stealth JavaScript injection code
    pub fn get_stealth_js(&mut self) -> String {
        // Create or reuse JavaScript injector
        if self.js_injector.is_none() {
            self.js_injector = Some(JavaScriptInjector::new(
                &self.config.fingerprinting.hardware,
                &self.config.fingerprinting.webgl,
                &self.config.fingerprinting.canvas,
                &self.config.request_randomization.locale.strategy,
            ));
        }

        let mut js_parts = if let Some(injector) = self.js_injector.as_ref() {
            vec![injector.generate_stealth_js()]
        } else {
            // This should never happen as we just initialized it, but handle gracefully
            vec![]
        };

        // Add enhanced WebRTC protection
        if self.config.preset != StealthPreset::None {
            js_parts.push(self.webrtc_enhanced.generate_js());
        }

        // Add timezone spoofing
        if self.config.preset != StealthPreset::None {
            let tz = self.timezone_manager.random_timezone();
            js_parts.push(self.timezone_manager.generate_js(&tz, true));
        }

        // Add screen resolution spoofing
        if self.config.preset == StealthPreset::High || self.config.preset == StealthPreset::Medium
        {
            let resolution = self.screen_resolution_manager.get_current();
            js_parts.push(self.screen_resolution_manager.generate_js(&resolution));
        }

        js_parts.join("\n\n")
    }

    /// Check if stealth mode is enabled
    pub fn is_stealth_enabled(&self) -> bool {
        self.config.preset != StealthPreset::None
    }

    /// Get request count for statistics
    pub fn get_request_count(&self) -> u64 {
        self.request_count
    }

    /// Get time since last request
    pub fn time_since_last_request(&self) -> std::time::Duration {
        self.last_request_time.elapsed()
    }

    /// Mark that a request was made
    pub fn mark_request_sent(&mut self) {
        self.last_request_time = Instant::now();
        self.request_count += 1;
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: StealthConfig) {
        self.config = new_config;
        // Recreate user agent manager with new config
        self.user_agent_manager = UserAgentManager::new(self.config.user_agent.clone());
        // Clear JS injector to force regeneration with new config
        self.js_injector = None;
        // Update WebRTC configuration based on new preset
        self.webrtc_enhanced = match self.config.preset {
            StealthPreset::High => WebRtcEnhanced::high_security(),
            StealthPreset::Medium => WebRtcEnhanced::default(),
            StealthPreset::Low => WebRtcEnhanced {
                block_ip_leak: true,
                ..Default::default()
            },
            StealthPreset::None => WebRtcEnhanced {
                block_completely: false,
                block_ip_leak: false,
                spoof_media_devices: false,
                block_data_channels: false,
                block_stun_turn: false,
                fake_ip: None,
            },
        };
    }

    /// Get domain-specific timing configuration
    pub fn get_domain_timing(&self, domain: &str) -> &crate::config::DomainTiming {
        self.config
            .timing
            .per_domain
            .get(domain)
            .unwrap_or(&self.config.timing.default_timing)
    }

    /// Add user agents to the rotation pool
    pub fn add_user_agents(&mut self, agents: Vec<String>) {
        self.user_agent_manager.add_user_agents(agents);
    }

    /// Get current configuration (immutable reference)
    pub fn config(&self) -> &StealthConfig {
        &self.config
    }

    /// Get mutable configuration reference
    pub fn config_mut(&mut self) -> &mut StealthConfig {
        &mut self.config
    }

    /// Reset session data (useful for new sessions)
    pub fn reset_session(&mut self) {
        self.request_count = 0;
        self.last_request_time = Instant::now();
        self.js_injector = None;
        // Reset user agent manager session
        self.user_agent_manager = UserAgentManager::new(self.config.user_agent.clone());
        // Reset enhanced features
        self.screen_resolution_manager = ScreenResolutionManager::new();
        self.timezone_manager = TimezoneManager::new();
        // Reset advanced anti-detection features
        self.rate_limiter = RateLimiter::new(self.config.timing.default_timing.clone());
        self.behavior_simulator = BehaviorSimulator::new();
    }

    /// Get enhanced screen resolution
    pub fn get_enhanced_resolution(&mut self) -> ScreenResolution {
        self.screen_resolution_manager.generate()
    }

    /// Get enhanced timezone
    pub fn get_enhanced_timezone(
        &mut self,
    ) -> crate::enhancements::timezone_enhanced::TimezoneInfo {
        self.timezone_manager.random_timezone()
    }

    /// Get WebRTC enhanced configuration
    pub fn get_webrtc_config(&self) -> &WebRtcEnhanced {
        &self.webrtc_enhanced
    }

    /// Validate header consistency with current user agent
    pub fn validate_headers(&self, headers: &HashMap<String, String>) -> Result<(), Vec<String>> {
        let default_ua = String::from("Mozilla/5.0");
        let user_agent = self.current_user_agent().unwrap_or(&default_ua);
        HeaderConsistencyManager::validate_consistency(user_agent, headers)
    }

    /// Get rate limiter reference for advanced rate limiting control
    ///
    /// Allows direct access to rate limiting functionality including:
    /// - Checking if requests are allowed for a domain
    /// - Recording request results for adaptive throttling
    /// - Getting domain statistics
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Get mutable rate limiter reference
    pub fn rate_limiter_mut(&mut self) -> &mut RateLimiter {
        &mut self.rate_limiter
    }

    /// Get behavior simulator reference for human-like interactions
    ///
    /// Allows generation of:
    /// - Human-like mouse movements
    /// - Smooth scrolling actions
    /// - Natural reading pauses
    /// - Realistic click delays
    pub fn behavior_simulator(&self) -> &BehaviorSimulator {
        &self.behavior_simulator
    }

    /// Get mutable behavior simulator reference
    pub fn behavior_simulator_mut(&mut self) -> &mut BehaviorSimulator {
        &mut self.behavior_simulator
    }

    /// Check rate limit for a domain before making a request
    ///
    /// This integrates rate limiting into the request flow.
    /// Returns Ok(delay) if allowed, Err(retry_after) if rate limited.
    pub async fn check_rate_limit_for_domain(
        &self,
        domain: &str,
    ) -> Result<std::time::Duration, std::time::Duration> {
        let timing = self.get_domain_timing(domain);
        self.rate_limiter
            .check_rate_limit(domain, Some(timing))
            .await
    }

    /// Record the result of a request for adaptive rate limiting
    ///
    /// Call this after each request completes to enable adaptive throttling.
    /// The rate limiter will automatically adjust delays based on success/failure patterns.
    ///
    /// # Arguments
    /// * `domain` - The domain the request was made to
    /// * `success` - Whether the request succeeded
    /// * `status_code` - HTTP status code (used to detect rate limiting: 429, 503)
    pub fn record_request_result(&self, domain: &str, success: bool, status_code: Option<u16>) {
        let is_rate_limit_error = status_code
            .map(|code| code == 429 || code == 503)
            .unwrap_or(false);
        self.rate_limiter
            .record_request_result(domain, success, is_rate_limit_error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_controller_creation() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);
        assert!(controller.is_stealth_enabled());
    }

    #[test]
    fn test_stealth_controller_from_preset() {
        let controller = StealthController::from_preset(StealthPreset::High);
        assert_eq!(controller.get_preset(), &StealthPreset::High);
        assert!(controller.is_stealth_enabled());

        let controller_none = StealthController::from_preset(StealthPreset::None);
        assert_eq!(controller_none.get_preset(), &StealthPreset::None);
        assert!(!controller_none.is_stealth_enabled());
    }

    #[test]
    fn test_user_agent_rotation() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let ua1 = controller.next_user_agent().to_string();
        let ua2 = controller.next_user_agent().to_string();

        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());
        // For random strategy, they might be the same, but both should be valid
    }

    #[test]
    fn test_header_generation() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let headers = controller.generate_headers();
        // HeaderConsistencyManager uses lowercase keys
        assert!(headers.contains_key("accept") || headers.contains_key("Accept"));
        assert!(headers.contains_key("accept-language") || headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("accept-encoding") || headers.contains_key("Accept-Encoding"));
        // Should also have enhanced headers
        assert!(!headers.is_empty());
    }

    #[test]
    fn test_viewport_randomization() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let (width, height) = controller.random_viewport();
        assert!(width > 0);
        assert!(height > 0);
        assert!(width >= 1280 - 50); // Considering variance
        assert!(height >= 720 - 50); // Considering variance
    }

    #[test]
    fn test_delay_calculation() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let delay = controller.calculate_delay();
        assert!(delay.as_millis() >= 500); // Min delay
        assert!(delay.as_millis() <= 3000); // Max delay
    }

    #[test]
    fn test_javascript_generation() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let js_code = controller.get_stealth_js();
        assert!(!js_code.is_empty());
        assert!(js_code.contains("'webdriver'"));
        assert!(js_code.contains("hardwareConcurrency"));
    }

    #[test]
    fn test_request_tracking() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        assert_eq!(controller.get_request_count(), 0);

        controller.mark_request_sent();
        assert_eq!(controller.get_request_count(), 1);

        // Test that time tracking works
        assert!(controller.time_since_last_request().as_millis() < 100);
    }

    #[test]
    fn test_session_reset() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        controller.mark_request_sent();
        assert_eq!(controller.get_request_count(), 1);

        controller.reset_session();
        assert_eq!(controller.get_request_count(), 0);
    }

    #[test]
    fn test_config_update() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let new_config = StealthConfig::from_preset(StealthPreset::High);
        controller.update_config(new_config);

        assert_eq!(controller.get_preset(), &StealthPreset::High);
    }
}
