//! Browser fingerprinting countermeasures
//!
//! This module contains all fingerprinting-related configuration
//! and techniques to avoid browser fingerprinting detection.

use serde::{Deserialize, Serialize};

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

    /// WebRTC fingerprinting countermeasures
    pub webrtc: WebRtcConfig,

    /// Hardware fingerprinting countermeasures
    pub hardware: HardwareConfig,

    /// Font fingerprinting countermeasures
    pub fonts: FontConfig,
}

impl Default for FingerprintingConfig {
    fn default() -> Self {
        Self {
            cdp_stealth: CdpStealthConfig::default(),
            webgl: WebGlConfig::default(),
            canvas: CanvasConfig::default(),
            audio: AudioConfig::default(),
            plugins: PluginConfig::default(),
            webrtc: WebRtcConfig::default(),
            hardware: HardwareConfig::default(),
            fonts: FontConfig::default(),
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
