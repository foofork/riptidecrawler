//! Browser fingerprinting countermeasures
//!
//! This module contains all fingerprinting-related configuration
//! and techniques to avoid browser fingerprinting detection.

use serde::{Deserialize, Serialize};

/// Browser fingerprinting countermeasures
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    /// WebRTC fingerprinting countermeasures
    pub webrtc: WebRtcConfig,

    /// Hardware fingerprinting countermeasures
    pub hardware: HardwareConfig,

    /// Font fingerprinting countermeasures
    pub fonts: FontConfig,
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

impl WebGlConfig {
    /// Get randomized WebGL vendor and renderer strings
    pub fn get_random_webgl_specs(&self) -> (String, String) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if !self.randomize_vendor {
            return (
                "Intel Inc.".to_string(),
                "Intel Iris OpenGL Engine".to_string(),
            );
        }

        let gpu_configs = [
            ("Intel Inc.", "Intel Iris OpenGL Engine"),
            ("Intel Inc.", "Intel UHD Graphics 630"),
            ("NVIDIA Corporation", "NVIDIA GeForce GTX 1060/PCIe/SSE2"),
            ("NVIDIA Corporation", "NVIDIA GeForce RTX 3060/PCIe/SSE2"),
            ("ATI Technologies Inc.", "AMD Radeon RX 580"),
            ("ATI Technologies Inc.", "AMD Radeon RX 6600 XT"),
        ];

        let index = rng.gen_range(0..gpu_configs.len());
        let (vendor, renderer) = gpu_configs[index];
        (vendor.to_string(), renderer.to_string())
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

    /// Noise intensity for audio fingerprinting (0.0 to 1.0)
    pub noise_intensity: f32,

    /// Spoof audio hardware properties
    pub spoof_hardware: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            add_noise: true,
            block_extraction: false,
            noise_intensity: 0.001,
            spoof_hardware: true,
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

/// WebRTC fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcConfig {
    /// Block WebRTC IP leaks
    pub block_ip_leak: bool,

    /// Spoof media devices
    pub spoof_media_devices: bool,

    /// Disable RTC data channels
    pub disable_data_channels: bool,
}

impl Default for WebRtcConfig {
    fn default() -> Self {
        Self {
            block_ip_leak: true,
            spoof_media_devices: true,
            disable_data_channels: false,
        }
    }
}

/// Hardware fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// Spoof CPU core count
    pub spoof_cpu_cores: bool,

    /// Spoof device memory
    pub spoof_device_memory: bool,

    /// Spoof battery information
    pub spoof_battery: bool,

    /// Available CPU core options
    pub cpu_core_options: Vec<u32>,

    /// Available memory options (in GB)
    pub memory_options: Vec<u32>,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            spoof_cpu_cores: true,
            spoof_device_memory: true,
            spoof_battery: true,
            cpu_core_options: vec![2, 4, 6, 8, 12, 16],
            memory_options: vec![2, 4, 8, 16],
        }
    }
}

impl HardwareConfig {
    /// Generate randomized hardware specifications
    pub fn get_random_hardware_specs(&self) -> (u32, u32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let cpu_cores = if self.spoof_cpu_cores {
            let index = rng.gen_range(0..self.cpu_core_options.len());
            self.cpu_core_options[index]
        } else {
            4 // Default
        };

        let device_memory = if self.spoof_device_memory {
            let index = rng.gen_range(0..self.memory_options.len());
            self.memory_options[index]
        } else {
            8 // Default
        };

        (cpu_cores, device_memory)
    }
}

/// Font fingerprinting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Limit available fonts
    pub limit_fonts: bool,

    /// Standard font list to report
    pub standard_fonts: Vec<String>,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            limit_fonts: true,
            standard_fonts: vec![
                "Arial".to_string(),
                "Times New Roman".to_string(),
                "Helvetica".to_string(),
                "Georgia".to_string(),
                "Verdana".to_string(),
                "Courier New".to_string(),
            ],
        }
    }
}

/// Comprehensive browser fingerprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserFingerprint {
    /// User agent string
    pub user_agent: String,

    /// Screen resolution (width, height)
    pub screen_resolution: (u32, u32),

    /// Screen color depth
    pub color_depth: u8,

    /// Timezone offset in minutes
    pub timezone_offset: i32,

    /// Timezone name (e.g., "America/New_York")
    pub timezone_name: String,

    /// WebGL vendor
    pub webgl_vendor: String,

    /// WebGL renderer
    pub webgl_renderer: String,

    /// Platform (e.g., "Win32", "MacIntel")
    pub platform: String,

    /// Language
    pub language: String,

    /// CPU cores
    pub hardware_concurrency: u32,

    /// Device memory in GB
    pub device_memory: u32,

    /// Plugin list
    pub plugins: Vec<String>,

    /// Canvas fingerprint (hash or identifier)
    pub canvas_hash: Option<String>,

    /// Audio fingerprint (hash or identifier)
    pub audio_hash: Option<String>,
}

impl BrowserFingerprint {
    /// Create a new browser fingerprint with random values
    pub fn new() -> Self {
        FingerprintGenerator::generate()
    }

    /// Create with custom user agent
    pub fn with_user_agent(user_agent: String) -> Self {
        let mut fp = Self::new();
        fp.user_agent = user_agent;
        fp
    }
}

impl Default for BrowserFingerprint {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator for realistic browser fingerprints
pub struct FingerprintGenerator;

impl FingerprintGenerator {
    /// Generate a complete, realistic browser fingerprint
    pub fn generate() -> BrowserFingerprint {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Realistic screen resolutions
        let screen_resolutions = [
            (1920, 1080), // Full HD
            (1366, 768),  // Common laptop
            (1536, 864),  // 1080p scaled
            (2560, 1440), // 2K
            (1440, 900),  // MacBook Air
            (1680, 1050), // WSXGA+
            (3840, 2160), // 4K (rare)
        ];
        let screen_resolution = screen_resolutions[rng.gen_range(0..screen_resolutions.len())];

        // Realistic color depths
        let color_depth = if rng.gen_bool(0.95) { 24 } else { 32 };

        // Realistic timezones
        let timezones = [
            ("America/New_York", -300),
            ("America/Los_Angeles", -480),
            ("America/Chicago", -360),
            ("Europe/London", 0),
            ("Europe/Paris", 60),
            ("Asia/Tokyo", 540),
            ("Australia/Sydney", 660),
            ("America/Denver", -420),
        ];
        let (timezone_name, timezone_offset) = timezones[rng.gen_range(0..timezones.len())];

        // Generate realistic WebGL vendor/renderer pairs
        let webgl_configs = WebGlConfig::default();
        let (webgl_vendor, webgl_renderer) = webgl_configs.get_random_webgl_specs();

        // Determine platform from user agent
        let platform = if rng.gen_bool(0.6) {
            "Win32".to_string()
        } else if rng.gen_bool(0.7) {
            "MacIntel".to_string()
        } else {
            "Linux x86_64".to_string()
        };

        // Realistic languages
        let languages = ["en-US", "en-GB", "de-DE", "fr-FR", "es-ES", "ja-JP"];
        let language = languages[rng.gen_range(0..languages.len())].to_string();

        // Realistic hardware specs
        let hardware_config = HardwareConfig::default();
        let (hardware_concurrency, device_memory) = hardware_config.get_random_hardware_specs();

        // Realistic plugins for Chrome
        let plugins = PluginConfig::default().plugin_list;

        // Generate a realistic user agent
        let user_agent = Self::generate_realistic_user_agent(&platform);

        BrowserFingerprint {
            user_agent,
            screen_resolution,
            color_depth,
            timezone_offset,
            timezone_name: timezone_name.to_string(),
            webgl_vendor,
            webgl_renderer,
            platform,
            language,
            hardware_concurrency,
            device_memory,
            plugins,
            canvas_hash: None, // Optional: can be computed from canvas rendering
            audio_hash: None,  // Optional: can be computed from audio context
        }
    }

    /// Generate consistent headers based on fingerprint
    pub fn generate_consistent_headers(
        fingerprint: &BrowserFingerprint,
    ) -> std::collections::HashMap<String, String> {
        use crate::enhancements::HeaderConsistencyManager;

        let mut headers =
            HeaderConsistencyManager::generate_consistent_headers(&fingerprint.user_agent);

        // Add locale-specific headers
        HeaderConsistencyManager::add_locale_headers(
            &mut headers,
            &fingerprint.language,
            &vec![fingerprint.language.clone()],
        );

        headers
    }

    /// Generate a realistic user agent for the platform
    fn generate_realistic_user_agent(platform: &str) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if platform.contains("Win") {
            // Windows Chrome
            let versions = ["120.0.0.0", "121.0.0.0", "119.0.0.0"];
            let version = versions[rng.gen_range(0..versions.len())];
            format!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                version
            )
        } else if platform.contains("Mac") {
            // macOS Chrome
            let versions = ["120.0.0.0", "121.0.0.0", "119.0.0.0"];
            let version = versions[rng.gen_range(0..versions.len())];
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                version
            )
        } else {
            // Linux Chrome
            let versions = ["120.0.0.0", "121.0.0.0", "119.0.0.0"];
            let version = versions[rng.gen_range(0..versions.len())];
            format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                version
            )
        }
    }

    /// Generate a persistent fingerprint (consistent core attributes, varying details)
    pub fn generate_persistent(seed: u64) -> BrowserFingerprint {
        use rand::rngs::StdRng;
        use rand::Rng;
        use rand::SeedableRng;

        let mut rng = StdRng::seed_from_u64(seed);

        // Core attributes remain constant with the same seed
        let screen_resolutions = [(1920, 1080), (1366, 768), (2560, 1440)];
        let screen_resolution = screen_resolutions[rng.gen_range(0..screen_resolutions.len())];

        let platform = if rng.gen_bool(0.6) {
            "Win32".to_string()
        } else {
            "MacIntel".to_string()
        };

        let hardware_concurrency = [4, 8, 16][rng.gen_range(0..3)];
        let device_memory = [8, 16][rng.gen_range(0..2)];

        // Generate other attributes
        let mut fp = Self::generate();
        fp.screen_resolution = screen_resolution;
        fp.platform = platform.clone();
        fp.hardware_concurrency = hardware_concurrency;
        fp.device_memory = device_memory;
        fp.user_agent = Self::generate_realistic_user_agent(&platform);

        fp
    }
}
