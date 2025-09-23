use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            user_agent: UserAgentConfig::default(),
            request_randomization: RequestRandomization::default(),
            proxy: None,
            fingerprinting: FingerprintingConfig::default(),
            timing: TimingConfig::default(),
        }
    }
}

/// User agent rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentConfig {
    /// Pool of user agents to rotate through
    pub agents: Vec<String>,

    /// Rotation strategy
    pub strategy: RotationStrategy,

    /// Whether to include mobile user agents
    pub include_mobile: bool,

    /// Browser type preference
    pub browser_preference: BrowserType,
}

impl Default for UserAgentConfig {
    fn default() -> Self {
        Self {
            agents: default_user_agents(),
            strategy: RotationStrategy::Random,
            include_mobile: false,
            browser_preference: BrowserType::Chrome,
        }
    }
}

/// User agent rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Pick randomly from the pool
    Random,

    /// Rotate in sequence
    Sequential,

    /// Use the same agent for the entire session
    Sticky,

    /// Choose based on target domain
    DomainBased,
}

/// Browser type preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Mixed,
}

/// Request randomization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for RequestRandomization {
    fn default() -> Self {
        Self {
            headers: HeaderRandomization::default(),
            timing_jitter: TimingJitter::default(),
            viewport: ViewportRandomization::default(),
            locale: LocaleRandomization::default(),
        }
    }
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

/// Browser fingerprinting countermeasures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintingConfig {
    /// Chrome DevTools Protocol stealth flags
    pub cdp_stealth: CdpStealthConfig,

    /// WebGL fingerprinting countermeasures
    pub webgl: WebGlConfig,

    /// Canvas fingerprinting countermeasures
    pub canvas: CanvasConfig,

    /// Audio fingerprinting countermeasures
    pub audio: AudioConfig,

    /// Plugin detection countermeasures
    pub plugins: PluginConfig,
}

impl Default for FingerprintingConfig {
    fn default() -> Self {
        Self {
            cdp_stealth: CdpStealthConfig::default(),
            webgl: WebGlConfig::default(),
            canvas: CanvasConfig::default(),
            audio: AudioConfig::default(),
            plugins: PluginConfig::default(),
        }
    }
}

/// Chrome DevTools Protocol stealth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpStealthConfig {
    /// Disable automation-controlled flag
    pub disable_automation_controlled: bool,

    /// Override navigator.webdriver
    pub override_webdriver: bool,

    /// Override navigator.permissions
    pub override_permissions: bool,

    /// Override navigator.plugins
    pub override_plugins: bool,

    /// Override window.chrome
    pub override_chrome: bool,
}

impl Default for CdpStealthConfig {
    fn default() -> Self {
        Self {
            disable_automation_controlled: true,
            override_webdriver: true,
            override_permissions: true,
            override_plugins: true,
            override_chrome: true,
        }
    }
}

/// WebGL fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGlConfig {
    /// Randomize WebGL vendor
    pub randomize_vendor: bool,

    /// Randomize WebGL renderer
    pub randomize_renderer: bool,

    /// Noise injection level (0.0 to 1.0)
    pub noise_level: f32,
}

impl Default for WebGlConfig {
    fn default() -> Self {
        Self {
            randomize_vendor: true,
            randomize_renderer: true,
            noise_level: 0.1,
        }
    }
}

/// Canvas fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasConfig {
    /// Add noise to canvas rendering
    pub add_noise: bool,

    /// Noise intensity (0.0 to 1.0)
    pub noise_intensity: f32,

    /// Block canvas data extraction
    pub block_data_extraction: bool,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.05,
            block_data_extraction: false,
        }
    }
}

/// Audio fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Add noise to audio context
    pub add_noise: bool,

    /// Block audio data extraction
    pub block_extraction: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            add_noise: true,
            block_extraction: false,
        }
    }
}

/// Plugin detection countermeasures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Mock common plugins
    pub mock_plugins: bool,

    /// Plugin list to mock
    pub plugin_list: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            mock_plugins: true,
            plugin_list: vec![
                "Chrome PDF Plugin".to_string(),
                "Chrome PDF Viewer".to_string(),
                "Native Client".to_string(),
            ],
        }
    }
}

/// Timing configuration for request pacing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Per-domain timing settings
    pub per_domain: HashMap<String, DomainTiming>,

    /// Default timing for unknown domains
    pub default_timing: DomainTiming,

    /// Global rate limiting
    pub global_rate_limit: Option<RateLimit>,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            per_domain: HashMap::new(),
            default_timing: DomainTiming::default(),
            global_rate_limit: None,
        }
    }
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

/// Stealth controller for managing anti-detection measures
pub struct StealthController {
    config: StealthConfig,
    current_user_agent: Option<String>,
    request_count: u64,
    last_request_time: std::time::Instant,
}

impl StealthController {
    /// Create a new stealth controller
    pub fn new(config: StealthConfig) -> Self {
        Self {
            config,
            current_user_agent: None,
            request_count: 0,
            last_request_time: std::time::Instant::now(),
        }
    }

    /// Get the next user agent based on rotation strategy
    pub fn next_user_agent(&mut self) -> &str {
        match self.config.user_agent.strategy {
            RotationStrategy::Random => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..self.config.user_agent.agents.len());
                self.current_user_agent = Some(self.config.user_agent.agents[index].clone());
            }
            RotationStrategy::Sequential => {
                let index = (self.request_count as usize) % self.config.user_agent.agents.len();
                self.current_user_agent = Some(self.config.user_agent.agents[index].clone());
            }
            RotationStrategy::Sticky => {
                if self.current_user_agent.is_none() {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..self.config.user_agent.agents.len());
                    self.current_user_agent = Some(self.config.user_agent.agents[index].clone());
                }
            }
            RotationStrategy::DomainBased => {
                // TODO: Implement domain-based selection
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..self.config.user_agent.agents.len());
                self.current_user_agent = Some(self.config.user_agent.agents[index].clone());
            }
        }

        self.current_user_agent.as_ref().unwrap()
    }

    /// Generate randomized headers for the request
    pub fn generate_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        let mut rng = rand::thread_rng();

        // Accept header
        let accept_index = rng.gen_range(
            0..self
                .config
                .request_randomization
                .headers
                .accept_variations
                .len(),
        );
        headers.insert(
            "Accept".to_string(),
            self.config.request_randomization.headers.accept_variations[accept_index].clone(),
        );

        // Accept-Language header
        let lang_index = rng.gen_range(
            0..self
                .config
                .request_randomization
                .headers
                .accept_language_variations
                .len(),
        );
        headers.insert(
            "Accept-Language".to_string(),
            self.config
                .request_randomization
                .headers
                .accept_language_variations[lang_index]
                .clone(),
        );

        // Accept-Encoding header
        let enc_index = rng.gen_range(
            0..self
                .config
                .request_randomization
                .headers
                .accept_encoding_variations
                .len(),
        );
        headers.insert(
            "Accept-Encoding".to_string(),
            self.config
                .request_randomization
                .headers
                .accept_encoding_variations[enc_index]
                .clone(),
        );

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
}

/// Default user agents for rotation
fn default_user_agents() -> Vec<String> {
    vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_config_default() {
        let config = StealthConfig::default();
        assert!(!config.user_agent.agents.is_empty());
        assert!(matches!(
            config.user_agent.strategy,
            RotationStrategy::Random
        ));
    }

    #[test]
    fn test_stealth_controller_user_agent_rotation() {
        let config = StealthConfig::default();
        let mut controller = StealthController::new(config);

        let ua1 = controller.next_user_agent().to_string();
        let ua2 = controller.next_user_agent().to_string();

        // For random strategy, UAs might be the same, but for sequential they should differ
        assert!(!ua1.is_empty());
        assert!(!ua2.is_empty());
    }

    #[test]
    fn test_header_generation() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let headers = controller.generate_headers();
        assert!(headers.contains_key("Accept"));
        assert!(headers.contains_key("Accept-Language"));
        assert!(headers.contains_key("Accept-Encoding"));
    }

    #[test]
    fn test_viewport_randomization() {
        let config = StealthConfig::default();
        let controller = StealthController::new(config);

        let (width, height) = controller.random_viewport();
        assert!(width > 0);
        assert!(height > 0);
    }
}
