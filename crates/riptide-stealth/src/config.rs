//! Configuration structures and presets for stealth mode
//!
//! This module contains all configuration-related types and functions
//! for managing stealth behavior across different scenarios.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

use crate::fingerprint::FingerprintingConfig;
use crate::user_agent::{RotationStrategy, UserAgentConfig};

/// Stealth preset levels for easy configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum StealthPreset {
    /// No stealth measures applied
    None,
    /// Basic stealth: minimal fingerprint changes
    Low,
    /// Moderate stealth: balanced detection vs performance
    #[default]
    Medium,
    /// Maximum stealth: all countermeasures enabled
    High,
}

/// Stealth configuration for anti-detection measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    /// User agent rotation settings
    pub user_agent: UserAgentConfig,

    /// Request randomization settings
    pub request_randomization: RequestRandomization,

    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,

    /// Browser fingerprinting countermeasures
    pub fingerprinting: FingerprintingConfig,

    /// Rate limiting and timing
    pub timing: TimingConfig,

    /// Stealth preset level
    pub preset: StealthPreset,

    /// Path to user agent list file
    pub ua_file_path: Option<String>,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            user_agent: UserAgentConfig::default(),
            request_randomization: RequestRandomization::default(),
            proxy: None,
            fingerprinting: FingerprintingConfig::default(),
            timing: TimingConfig::default(),
            preset: StealthPreset::default(),
            ua_file_path: Some("configs/ua_list.txt".to_string()),
        }
    }
}

impl StealthConfig {
    /// Create stealth config from preset
    pub fn from_preset(preset: StealthPreset) -> Self {
        let mut config = Self {
            preset: preset.clone(),
            ..Default::default()
        };

        match preset {
            StealthPreset::None => {
                config
                    .fingerprinting
                    .cdp_stealth
                    .disable_automation_controlled = false;
                config.fingerprinting.cdp_stealth.override_webdriver = false;
                config.fingerprinting.cdp_stealth.override_permissions = false;
                config.fingerprinting.cdp_stealth.override_plugins = false;
                config.fingerprinting.cdp_stealth.override_chrome = false;
                config.user_agent.strategy = RotationStrategy::Sticky;
            }
            StealthPreset::Low => {
                config
                    .fingerprinting
                    .cdp_stealth
                    .disable_automation_controlled = true;
                config.fingerprinting.cdp_stealth.override_webdriver = true;
                config.user_agent.strategy = RotationStrategy::Sequential;
                config.request_randomization.timing_jitter.jitter_percentage = 0.1;
            }
            StealthPreset::Medium => {
                // Use defaults (already configured for medium stealth)
            }
            StealthPreset::High => {
                config.user_agent.strategy = RotationStrategy::Random;
                config.request_randomization.timing_jitter.jitter_percentage = 0.4;
                config.fingerprinting.webgl.noise_level = 0.2;
                config.fingerprinting.canvas.noise_intensity = 0.1;
                config.fingerprinting.audio.noise_intensity = 0.001;
                config.fingerprinting.webrtc.block_ip_leak = true;
                config.fingerprinting.webrtc.spoof_media_devices = true;
                config.fingerprinting.hardware.spoof_cpu_cores = true;
                config.fingerprinting.hardware.spoof_device_memory = true;
                config.request_randomization.headers.randomize_order = true;
            }
        }

        config
    }

    /// Load user agents from file
    pub fn load_user_agents_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_path) = &self.ua_file_path {
            if Path::new(file_path).exists() {
                let content = fs::read_to_string(file_path)?;
                let agents: Vec<String> = content
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                    .map(|line| line.trim().to_string())
                    .collect();

                if !agents.is_empty() {
                    self.user_agent.agents = agents;
                    debug!(
                        "Loaded {} user agents from {}",
                        self.user_agent.agents.len(),
                        file_path
                    );
                } else {
                    warn!("No valid user agents found in file: {}", file_path);
                }
            } else {
                warn!("User agent file not found: {}, using defaults", file_path);
            }
        }
        Ok(())
    }

    /// Apply preset-specific CDP flags
    pub fn get_cdp_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();

        if self.preset == StealthPreset::None {
            return flags;
        }

        if self
            .fingerprinting
            .cdp_stealth
            .disable_automation_controlled
        {
            flags.push("--disable-blink-features=AutomationControlled".to_string());
        }

        // Base stealth flags for Low and above
        if self.preset != StealthPreset::None {
            flags.extend([
                "--no-first-run".to_string(),
                "--disable-default-apps".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--no-sandbox".to_string(),
            ]);
        }

        // Medium and High stealth flags
        if matches!(self.preset, StealthPreset::Medium | StealthPreset::High) {
            flags.extend([
                "--disable-web-security".to_string(),
                "--disable-background-timer-throttling".to_string(),
                "--disable-backgrounding-occluded-windows".to_string(),
                "--disable-renderer-backgrounding".to_string(),
            ]);
        }

        // High stealth flags
        if self.preset == StealthPreset::High {
            flags.extend([
                "--disable-features=TranslateUI".to_string(),
                "--disable-extensions".to_string(),
                "--disable-plugins".to_string(),
                "--disable-images".to_string(),
                "--mute-audio".to_string(),
            ]);
        }

        flags
    }
}

/// Request randomization settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestRandomization {
    /// Header randomization
    pub headers: HeaderRandomization,

    /// Request timing jitter
    pub timing_jitter: TimingJitter,

    /// Viewport size randomization
    pub viewport: ViewportRandomization,

    /// Language/locale randomization
    pub locale: LocaleRandomization,
}

/// Header randomization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRandomization {
    /// Accept header variations
    pub accept_variations: Vec<String>,

    /// Accept-Language variations
    pub accept_language_variations: Vec<String>,

    /// Accept-Encoding variations
    pub accept_encoding_variations: Vec<String>,

    /// Additional headers to randomize
    pub custom_headers: HashMap<String, Vec<String>>,

    /// Whether to randomize header order
    pub randomize_order: bool,
}

impl Default for HeaderRandomization {
    fn default() -> Self {
        Self {
            accept_variations: vec![
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".to_string(),
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8".to_string(),
            ],
            accept_language_variations: vec![
                "en-US,en;q=0.9".to_string(),
                "en-US,en;q=0.8".to_string(),
                "en-US,en;q=0.9,es;q=0.8".to_string(),
            ],
            accept_encoding_variations: vec![
                "gzip, deflate, br".to_string(),
                "gzip, deflate".to_string(),
                "gzip, deflate, br, zstd".to_string(),
            ],
            custom_headers: HashMap::new(),
            randomize_order: true,
        }
    }
}

/// Timing jitter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingJitter {
    /// Base delay between requests (in milliseconds)
    pub base_delay_ms: u64,

    /// Maximum jitter percentage (0.0 to 1.0)
    pub jitter_percentage: f64,

    /// Minimum delay regardless of jitter
    pub min_delay_ms: u64,

    /// Maximum delay regardless of jitter
    pub max_delay_ms: u64,
}

impl Default for TimingJitter {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000,
            jitter_percentage: 0.2,
            min_delay_ms: 500,
            max_delay_ms: 3000,
        }
    }
}

/// Viewport randomization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportRandomization {
    /// Common viewport sizes to choose from
    pub sizes: Vec<(u32, u32)>,

    /// Whether to add small random variations
    pub add_variance: bool,

    /// Maximum variance in pixels
    pub max_variance: u32,
}

impl Default for ViewportRandomization {
    fn default() -> Self {
        Self {
            sizes: vec![
                (1920, 1080),
                (1366, 768),
                (1536, 864),
                (1440, 900),
                (1280, 720),
                (1600, 900),
            ],
            add_variance: true,
            max_variance: 50,
        }
    }
}

/// Locale randomization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleRandomization {
    /// Available locales to choose from
    pub locales: Vec<String>,

    /// Corresponding timezone for each locale
    pub timezones: HashMap<String, String>,

    /// Strategy for locale selection
    pub strategy: LocaleStrategy,
}

impl Default for LocaleRandomization {
    fn default() -> Self {
        let mut timezones = HashMap::new();
        timezones.insert("en-US".to_string(), "America/New_York".to_string());
        timezones.insert("en-GB".to_string(), "Europe/London".to_string());
        timezones.insert("de-DE".to_string(), "Europe/Berlin".to_string());
        timezones.insert("fr-FR".to_string(), "Europe/Paris".to_string());

        Self {
            locales: vec![
                "en-US".to_string(),
                "en-GB".to_string(),
                "de-DE".to_string(),
                "fr-FR".to_string(),
            ],
            timezones,
            strategy: LocaleStrategy::Random,
        }
    }
}

/// Locale selection strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocaleStrategy {
    Random,
    Geographic,
    TargetBased,
    Fixed(String),
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy type
    pub proxy_type: ProxyType,

    /// Proxy endpoints
    pub endpoints: Vec<ProxyEndpoint>,

    /// Rotation strategy for multiple proxies
    pub rotation: ProxyRotation,

    /// Authentication if required
    pub auth: Option<ProxyAuth>,
}

/// Proxy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

/// Proxy endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEndpoint {
    /// Proxy host
    pub host: String,

    /// Proxy port
    pub port: u16,

    /// Whether this proxy supports HTTPS
    pub supports_https: bool,

    /// Geographic location hint
    pub location: Option<String>,

    /// Health status
    pub healthy: bool,
}

/// Proxy rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyRotation {
    Random,
    RoundRobin,
    HealthBased,
    Geographic,
}

/// Proxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    /// Username
    pub username: String,

    /// Password
    pub password: String,
}

/// Timing configuration for request pacing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimingConfig {
    /// Per-domain timing settings
    pub per_domain: HashMap<String, DomainTiming>,

    /// Default timing for unknown domains
    pub default_timing: DomainTiming,

    /// Global rate limiting
    pub global_rate_limit: Option<RateLimit>,
}

/// Per-domain timing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTiming {
    /// Minimum delay between requests
    pub min_delay_ms: u64,

    /// Maximum delay between requests
    pub max_delay_ms: u64,

    /// Requests per minute limit
    pub rpm_limit: Option<u32>,

    /// Burst allowance
    pub burst_size: u32,
}

impl Default for DomainTiming {
    fn default() -> Self {
        Self {
            min_delay_ms: 1000,
            max_delay_ms: 3000,
            rpm_limit: Some(60),
            burst_size: 5,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per second
    pub rps: f64,

    /// Burst capacity
    pub burst: u32,
}

/// Load user agents from external file
pub fn load_user_agents_from_file(
    file_path: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        return Err(format!("User agent file not found: {}", file_path).into());
    }

    let content = fs::read_to_string(file_path)?;
    let agents: Vec<String> = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();

    if agents.is_empty() {
        return Err("No valid user agents found in file".into());
    }

    Ok(agents)
}
