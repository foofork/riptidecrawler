//! Stealth level configuration and presets
//!
//! This module provides a comprehensive stealth level system with
//! configurable intensity levels for different anti-detection scenarios.

use crate::config::StealthPreset;
use serde::{Deserialize, Serialize};
// Removed unused imports: WebGlConfig, CanvasConfig, AudioConfig, WebRtcConfig, HardwareConfig

/// Stealth level enumeration with graduated intensity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StealthLevel {
    /// No stealth measures (fastest, detectable)
    None = 0,
    /// Basic stealth (low overhead, basic protection)
    Low = 1,
    /// Balanced stealth (moderate overhead, good protection)
    #[default]
    Medium = 2,
    /// Maximum stealth (higher overhead, best protection)
    High = 3,
}

impl From<StealthPreset> for StealthLevel {
    fn from(preset: StealthPreset) -> Self {
        match preset {
            StealthPreset::None => StealthLevel::None,
            StealthPreset::Low => StealthLevel::Low,
            StealthPreset::Medium => StealthLevel::Medium,
            StealthPreset::High => StealthLevel::High,
        }
    }
}

impl From<StealthLevel> for StealthPreset {
    fn from(level: StealthLevel) -> Self {
        match level {
            StealthLevel::None => StealthPreset::None,
            StealthLevel::Low => StealthPreset::Low,
            StealthLevel::Medium => StealthPreset::Medium,
            StealthLevel::High => StealthPreset::High,
        }
    }
}

/// Comprehensive stealth configuration based on level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthLevelConfig {
    /// Stealth intensity level
    pub level: StealthLevel,

    /// WebRTC leak prevention configuration
    pub webrtc: WebRtcLevelConfig,

    /// Canvas fingerprint protection
    pub canvas: CanvasLevelConfig,

    /// Audio context protection
    pub audio: AudioLevelConfig,

    /// WebGL spoofing configuration
    pub webgl: WebGlLevelConfig,

    /// Hardware fingerprint spoofing
    pub hardware: HardwareLevelConfig,

    /// Performance impact estimate (0.0 to 1.0)
    pub performance_impact: f32,

    /// Detection evasion score (0.0 to 1.0)
    pub evasion_score: f32,
}

impl StealthLevelConfig {
    /// Create configuration from stealth level
    pub fn from_level(level: StealthLevel) -> Self {
        match level {
            StealthLevel::None => Self::none(),
            StealthLevel::Low => Self::low(),
            StealthLevel::Medium => Self::medium(),
            StealthLevel::High => Self::high(),
        }
    }

    /// No stealth configuration
    fn none() -> Self {
        Self {
            level: StealthLevel::None,
            webrtc: WebRtcLevelConfig::none(),
            canvas: CanvasLevelConfig::none(),
            audio: AudioLevelConfig::none(),
            webgl: WebGlLevelConfig::none(),
            hardware: HardwareLevelConfig::none(),
            performance_impact: 0.0,
            evasion_score: 0.0,
        }
    }

    /// Low stealth configuration
    fn low() -> Self {
        Self {
            level: StealthLevel::Low,
            webrtc: WebRtcLevelConfig::low(),
            canvas: CanvasLevelConfig::low(),
            audio: AudioLevelConfig::low(),
            webgl: WebGlLevelConfig::low(),
            hardware: HardwareLevelConfig::low(),
            performance_impact: 0.1,
            evasion_score: 0.4,
        }
    }

    /// Medium stealth configuration
    fn medium() -> Self {
        Self {
            level: StealthLevel::Medium,
            webrtc: WebRtcLevelConfig::medium(),
            canvas: CanvasLevelConfig::medium(),
            audio: AudioLevelConfig::medium(),
            webgl: WebGlLevelConfig::medium(),
            hardware: HardwareLevelConfig::medium(),
            performance_impact: 0.3,
            evasion_score: 0.7,
        }
    }

    /// High stealth configuration
    fn high() -> Self {
        Self {
            level: StealthLevel::High,
            webrtc: WebRtcLevelConfig::high(),
            canvas: CanvasLevelConfig::high(),
            audio: AudioLevelConfig::high(),
            webgl: WebGlLevelConfig::high(),
            hardware: HardwareLevelConfig::high(),
            performance_impact: 0.5,
            evasion_score: 0.95,
        }
    }

    /// Get estimated performance impact description
    pub fn performance_description(&self) -> &str {
        match self.level {
            StealthLevel::None => "No overhead",
            StealthLevel::Low => "Minimal overhead (~5-10%)",
            StealthLevel::Medium => "Moderate overhead (~15-25%)",
            StealthLevel::High => "Significant overhead (~30-50%)",
        }
    }

    /// Get detection evasion description
    pub fn evasion_description(&self) -> &str {
        match self.level {
            StealthLevel::None => "No protection - easily detectable",
            StealthLevel::Low => "Basic protection - defeats simple detection",
            StealthLevel::Medium => "Good protection - defeats most detection",
            StealthLevel::High => "Excellent protection - defeats advanced detection",
        }
    }
}

impl Default for StealthLevelConfig {
    fn default() -> Self {
        Self::from_level(StealthLevel::default())
    }
}

/// WebRTC protection levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRtcLevelConfig {
    pub block_ip_leak: bool,
    pub block_completely: bool,
    pub spoof_media_devices: bool,
    pub block_data_channels: bool,
    pub block_stun_turn: bool,
}

impl WebRtcLevelConfig {
    fn none() -> Self {
        Self {
            block_ip_leak: false,
            block_completely: false,
            spoof_media_devices: false,
            block_data_channels: false,
            block_stun_turn: false,
        }
    }

    fn low() -> Self {
        Self {
            block_ip_leak: true,
            block_completely: false,
            spoof_media_devices: false,
            block_data_channels: false,
            block_stun_turn: false,
        }
    }

    fn medium() -> Self {
        Self {
            block_ip_leak: true,
            block_completely: false,
            spoof_media_devices: true,
            block_data_channels: false,
            block_stun_turn: true,
        }
    }

    fn high() -> Self {
        Self {
            block_ip_leak: true,
            block_completely: false,
            spoof_media_devices: true,
            block_data_channels: true,
            block_stun_turn: true,
        }
    }
}

/// Canvas protection levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasLevelConfig {
    pub add_noise: bool,
    pub noise_intensity: f32,
    pub block_data_extraction: bool,
}

impl CanvasLevelConfig {
    fn none() -> Self {
        Self {
            add_noise: false,
            noise_intensity: 0.0,
            block_data_extraction: false,
        }
    }

    fn low() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.01,
            block_data_extraction: false,
        }
    }

    fn medium() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.05,
            block_data_extraction: false,
        }
    }

    fn high() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.1,
            block_data_extraction: false,
        }
    }
}

/// Audio protection levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevelConfig {
    pub add_noise: bool,
    pub noise_intensity: f32,
    pub spoof_hardware: bool,
}

impl AudioLevelConfig {
    fn none() -> Self {
        Self {
            add_noise: false,
            noise_intensity: 0.0,
            spoof_hardware: false,
        }
    }

    fn low() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.0001,
            spoof_hardware: false,
        }
    }

    fn medium() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.001,
            spoof_hardware: true,
        }
    }

    fn high() -> Self {
        Self {
            add_noise: true,
            noise_intensity: 0.002,
            spoof_hardware: true,
        }
    }
}

/// WebGL spoofing levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebGlLevelConfig {
    pub randomize_vendor: bool,
    pub randomize_renderer: bool,
    pub noise_level: f32,
}

impl WebGlLevelConfig {
    fn none() -> Self {
        Self {
            randomize_vendor: false,
            randomize_renderer: false,
            noise_level: 0.0,
        }
    }

    fn low() -> Self {
        Self {
            randomize_vendor: true,
            randomize_renderer: true,
            noise_level: 0.05,
        }
    }

    fn medium() -> Self {
        Self {
            randomize_vendor: true,
            randomize_renderer: true,
            noise_level: 0.1,
        }
    }

    fn high() -> Self {
        Self {
            randomize_vendor: true,
            randomize_renderer: true,
            noise_level: 0.2,
        }
    }
}

/// Hardware spoofing levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareLevelConfig {
    pub spoof_cpu_cores: bool,
    pub spoof_device_memory: bool,
    pub spoof_battery: bool,
}

impl HardwareLevelConfig {
    fn none() -> Self {
        Self {
            spoof_cpu_cores: false,
            spoof_device_memory: false,
            spoof_battery: false,
        }
    }

    fn low() -> Self {
        Self {
            spoof_cpu_cores: false,
            spoof_device_memory: false,
            spoof_battery: true,
        }
    }

    fn medium() -> Self {
        Self {
            spoof_cpu_cores: true,
            spoof_device_memory: true,
            spoof_battery: true,
        }
    }

    fn high() -> Self {
        Self {
            spoof_cpu_cores: true,
            spoof_device_memory: true,
            spoof_battery: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_level_default() {
        assert_eq!(StealthLevel::default(), StealthLevel::Medium);
    }

    #[test]
    fn test_level_config_creation() {
        let config = StealthLevelConfig::from_level(StealthLevel::High);
        assert_eq!(config.level, StealthLevel::High);
        assert!(config.performance_impact > 0.0);
        assert!(config.evasion_score > 0.0);
    }

    #[test]
    fn test_performance_impact() {
        let none = StealthLevelConfig::from_level(StealthLevel::None);
        let high = StealthLevelConfig::from_level(StealthLevel::High);

        assert!(none.performance_impact < high.performance_impact);
        assert!(none.evasion_score < high.evasion_score);
    }

    #[test]
    fn test_webrtc_levels() {
        let low = WebRtcLevelConfig::low();
        let high = WebRtcLevelConfig::high();

        assert!(low.block_ip_leak);
        assert!(!low.block_data_channels);
        assert!(high.block_data_channels);
    }

    #[test]
    fn test_canvas_noise_levels() {
        let low = CanvasLevelConfig::low();
        let medium = CanvasLevelConfig::medium();
        let high = CanvasLevelConfig::high();

        assert!(low.noise_intensity < medium.noise_intensity);
        assert!(medium.noise_intensity < high.noise_intensity);
    }

    #[test]
    fn test_descriptions() {
        let config = StealthLevelConfig::from_level(StealthLevel::High);
        assert!(!config.performance_description().is_empty());
        assert!(!config.evasion_description().is_empty());
    }

    #[test]
    fn test_preset_conversion() {
        let level = StealthLevel::from(StealthPreset::High);
        assert_eq!(level, StealthLevel::High);

        let preset = StealthPreset::from(StealthLevel::Medium);
        assert_eq!(preset, StealthPreset::Medium);
    }
}
