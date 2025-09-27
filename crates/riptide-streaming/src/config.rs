//! Configuration management for RipTide streaming
//!
//! This module provides configuration management with support for
//! multiple sources (files, environment variables, CLI arguments).

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use config::{Config, ConfigError, Environment, File};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiptideConfig {
    pub api: ApiConfig,
    pub streaming: StreamingConfig,
    pub reports: ReportsConfig,
    pub cli: CliConfig,
    pub logging: LoggingConfig,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub rate_limit: RateLimitConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    pub buffer_size: usize,
    pub max_concurrent_streams: usize,
    pub progress_update_interval_ms: u64,
    pub ndjson_enabled: bool,
    pub websocket_enabled: bool,
    pub backpressure: BackpressureConfig,
}

/// Backpressure configuration (re-exported from backpressure module)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub max_in_flight: usize,
    pub max_memory_bytes: u64,
    pub max_total_items: usize,
    pub activation_threshold: f64,
    pub recovery_threshold: f64,
    pub check_interval_ms: u64,
    pub adaptive: bool,
}

/// Reports configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportsConfig {
    pub output_directory: PathBuf,
    pub template_directory: Option<PathBuf>,
    pub default_format: String,
    pub include_charts: bool,
    pub include_raw_data: bool,
    pub chart_width: u32,
    pub chart_height: u32,
    pub theme: String,
}

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub default_output_format: String,
    pub color_enabled: bool,
    pub progress_bars: bool,
    pub verbose: bool,
    pub quiet: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_logging: bool,
    pub log_file: Option<PathBuf>,
    pub structured_logging: bool,
}

/// Configuration builder for creating configurations from multiple sources
pub struct ConfigBuilder {
    config: Config,
    config_file: Option<PathBuf>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            config_file: None,
        }
    }

    /// Add configuration from a file
    pub fn add_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        let path = path.into();
        if path.exists() {
            self.config = self.config.clone()
                .merge(File::from(path.clone()))
                .unwrap_or_else(|_| self.config);
            self.config_file = Some(path);
        }
        self
    }

    /// Add configuration from environment variables
    pub fn add_env(mut self, prefix: &str) -> Self {
        self.config = self.config.clone()
            .merge(Environment::with_prefix(prefix).separator("_"))
            .unwrap_or_else(|_| self.config);
        self
    }

    /// Add configuration from a HashMap
    pub fn add_map(mut self, map: HashMap<String, String>) -> Self {
        for (key, value) in map {
            self.config = self.config.clone()
                .set(&key, value)
                .unwrap_or_else(|_| self.config);
        }
        self
    }

    /// Build the final configuration
    pub fn build(self) -> Result<RiptideConfig, ConfigError> {
        let mut config = self.config;
        
        // Set defaults if not provided
        config = config.set_default("api.host", "localhost")?;
        config = config.set_default("api.port", 8080)?;
        config = config.set_default("api.timeout_seconds", 30)?;
        config = config.set_default("api.max_retries", 3)?;
        config = config.set_default("api.rate_limit.requests_per_minute", 60)?;
        config = config.set_default("api.rate_limit.burst_size", 10)?;
        config = config.set_default("api.rate_limit.enabled", true)?;
        
        config = config.set_default("streaming.buffer_size", 1000)?;
        config = config.set_default("streaming.max_concurrent_streams", 10)?;
        config = config.set_default("streaming.progress_update_interval_ms", 1000)?;
        config = config.set_default("streaming.ndjson_enabled", true)?;
        config = config.set_default("streaming.websocket_enabled", true)?;
        
        config = config.set_default("streaming.backpressure.max_in_flight", 1000)?;
        config = config.set_default("streaming.backpressure.max_memory_bytes", 104857600)?; // 100MB
        config = config.set_default("streaming.backpressure.max_total_items", 10000)?;
        config = config.set_default("streaming.backpressure.activation_threshold", 0.8)?;
        config = config.set_default("streaming.backpressure.recovery_threshold", 0.6)?;
        config = config.set_default("streaming.backpressure.check_interval_ms", 500)?;
        config = config.set_default("streaming.backpressure.adaptive", true)?;
        
        let output_dir = get_default_output_directory();
        config = config.set_default("reports.output_directory", output_dir.to_string_lossy().as_ref())?;
        config = config.set_default("reports.default_format", "html")?;
        config = config.set_default("reports.include_charts", true)?;
        config = config.set_default("reports.include_raw_data", false)?;
        config = config.set_default("reports.chart_width", 800)?;
        config = config.set_default("reports.chart_height", 400)?;
        config = config.set_default("reports.theme", "modern")?;
        
        config = config.set_default("cli.default_output_format", "json")?;
        config = config.set_default("cli.color_enabled", true)?;
        config = config.set_default("cli.progress_bars", true)?;
        config = config.set_default("cli.verbose", false)?;
        config = config.set_default("cli.quiet", false)?;
        
        config = config.set_default("logging.level", "info")?;
        config = config.set_default("logging.format", "pretty")?;
        config = config.set_default("logging.file_logging", false)?;
        config = config.set_default("logging.structured_logging", false)?;
        
        config.try_deserialize()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration manager for loading and saving configurations
pub struct ConfigManager {
    config: RiptideConfig,
    config_file: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new configuration manager with default configuration
    pub fn new() -> Result<Self> {
        let config = ConfigBuilder::new()
            .add_env("RIPTIDE")
            .build()?;
        
        Ok(Self {
            config,
            config_file: None,
        })
    }

    /// Load configuration from multiple sources
    pub fn load() -> Result<Self> {
        let config_file = get_config_file_path();
        
        let config = ConfigBuilder::new()
            .add_file(&config_file)
            .add_env("RIPTIDE")
            .build()?;
        
        Ok(Self {
            config,
            config_file: Some(config_file),
        })
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let path = path.into();
        
        let config = ConfigBuilder::new()
            .add_file(&path)
            .add_env("RIPTIDE")
            .build()?;
        
        Ok(Self {
            config,
            config_file: Some(path),
        })
    }

    /// Get the current configuration
    pub fn config(&self) -> &RiptideConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut RiptideConfig {
        &mut self.config
    }

    /// Save the configuration to file
    pub fn save(&self) -> Result<()> {
        let config_file = self.config_file.as_ref()
            .unwrap_or(&get_config_file_path());
        
        // Ensure the config directory exists
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let toml_content = toml::to_string_pretty(&self.config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write(config_file, toml_content)?;
        Ok(())
    }

    /// Save the configuration to a specific file
    pub fn save_to_file<P: Into<PathBuf>>(&self, path: P) -> Result<()> {
        let path = path.into();
        
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let toml_content = toml::to_string_pretty(&self.config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write(&path, toml_content)?;
        Ok(())
    }

    /// Reset to default configuration
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.config = ConfigBuilder::new().build()?;
        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate API configuration
        if self.config.api.port == 0 {
            return Err(anyhow::anyhow!("API port cannot be 0"));
        }
        
        if self.config.api.timeout_seconds == 0 {
            return Err(anyhow::anyhow!("API timeout must be greater than 0"));
        }
        
        // Validate streaming configuration
        if self.config.streaming.buffer_size == 0 {
            return Err(anyhow::anyhow!("Streaming buffer size must be greater than 0"));
        }
        
        if self.config.streaming.max_concurrent_streams == 0 {
            return Err(anyhow::anyhow!("Max concurrent streams must be greater than 0"));
        }
        
        // Validate backpressure configuration
        let bp = &self.config.streaming.backpressure;
        if bp.activation_threshold < 0.0 || bp.activation_threshold > 1.0 {
            return Err(anyhow::anyhow!("Activation threshold must be between 0.0 and 1.0"));
        }
        
        if bp.recovery_threshold < 0.0 || bp.recovery_threshold > 1.0 {
            return Err(anyhow::anyhow!("Recovery threshold must be between 0.0 and 1.0"));
        }
        
        if bp.activation_threshold <= bp.recovery_threshold {
            return Err(anyhow::anyhow!("Activation threshold must be greater than recovery threshold"));
        }
        
        // Validate reports configuration
        if !self.config.reports.output_directory.exists() {
            if let Err(e) = std::fs::create_dir_all(&self.config.reports.output_directory) {
                return Err(anyhow::anyhow!("Cannot create output directory: {}", e));
            }
        }
        
        Ok(())
    }

    /// Get configuration summary as a formatted string
    pub fn summary(&self) -> String {
        format!(
            "RipTide Configuration Summary:
\
             API: {}:{}
\
             Streaming: {} buffer, {} max streams
\
             Reports: {} ({})
\
             Logging: {}",
            self.config.api.host,
            self.config.api.port,
            self.config.streaming.buffer_size,
            self.config.streaming.max_concurrent_streams,
            self.config.reports.output_directory.display(),
            self.config.reports.default_format,
            self.config.logging.level
        )
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default configuration")
    }
}

/// Get the default configuration file path
pub fn get_config_file_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "riptide", "riptide") {
        let config_dir = proj_dirs.config_dir();
        config_dir.join("config.toml")
    } else {
        PathBuf::from("riptide-config.toml")
    }
}

/// Get the default output directory
pub fn get_default_output_directory() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "riptide", "riptide") {
        proj_dirs.data_dir().join("reports")
    } else {
        PathBuf::from("reports")
    }
}

/// Get the default log directory
pub fn get_default_log_directory() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "riptide", "riptide") {
        proj_dirs.data_dir().join("logs")
    } else {
        PathBuf::from("logs")
    }
}

/// Environment variable helpers
pub mod env {
    use std::env;
    
    pub fn get_api_host() -> Option<String> {
        env::var("RIPTIDE_API_HOST").ok()
    }
    
    pub fn get_api_port() -> Option<u16> {
        env::var("RIPTIDE_API_PORT").ok()
            .and_then(|s| s.parse().ok())
    }
    
    pub fn get_log_level() -> Option<String> {
        env::var("RIPTIDE_LOG_LEVEL").ok()
            .or_else(|| env::var("RUST_LOG").ok())
    }
    
    pub fn get_config_file() -> Option<String> {
        env::var("RIPTIDE_CONFIG_FILE").ok()
    }
    
    pub fn is_development_mode() -> bool {
        env::var("RIPTIDE_DEV").is_ok() || 
        env::var("DEVELOPMENT").is_ok() ||
        cfg!(debug_assertions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_config_builder_defaults() {
        let config = ConfigBuilder::new().build().unwrap();
        
        assert_eq!(config.api.host, "localhost");
        assert_eq!(config.api.port, 8080);
        assert!(config.streaming.ndjson_enabled);
        assert!(config.reports.include_charts);
    }
    
    #[test]
    fn test_config_builder_with_map() {
        let mut map = HashMap::new();
        map.insert("api.port".to_string(), "9000".to_string());
        map.insert("api.host".to_string(), "0.0.0.0".to_string());
        
        let config = ConfigBuilder::new()
            .add_map(map)
            .build()
            .unwrap();
        
        assert_eq!(config.api.host, "0.0.0.0");
        assert_eq!(config.api.port, 9000);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = ConfigBuilder::new().build().unwrap();
        let manager = ConfigManager { config: config.clone(), config_file: None };
        
        // Valid configuration should pass
        assert!(manager.validate().is_ok());
        
        // Invalid port should fail
        config.api.port = 0;
        let manager = ConfigManager { config: config.clone(), config_file: None };
        assert!(manager.validate().is_err());
        
        // Invalid threshold should fail
        config.api.port = 8080;
        config.streaming.backpressure.activation_threshold = 1.5;
        let manager = ConfigManager { config, config_file: None };
        assert!(manager.validate().is_err());
    }
    
    #[test]
    fn test_config_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("test-config.toml");
        
        // Create and save a configuration
        let mut manager = ConfigManager::new().unwrap();
        manager.config_mut().api.port = 9999;
        manager.save_to_file(&config_file).unwrap();
        
        // Load the configuration from file
        let loaded_manager = ConfigManager::load_from_file(&config_file).unwrap();
        assert_eq!(loaded_manager.config().api.port, 9999);
    }
    
    #[test]
    fn test_environment_helpers() {
        // These tests would need to set environment variables
        // For now, just test that they don't panic
        let _ = env::get_api_host();
        let _ = env::get_api_port();
        let _ = env::get_log_level();
        let _ = env::is_development_mode();
    }
    
    #[test]
    fn test_config_summary() {
        let manager = ConfigManager::new().unwrap();
        let summary = manager.summary();
        
        assert!(summary.contains("RipTide Configuration Summary"));
        assert!(summary.contains("localhost:8080"));
        assert!(summary.contains("info"));
    }
    
    #[test]
    fn test_default_paths() {
        let config_path = get_config_file_path();
        assert!(config_path.to_string_lossy().ends_with("config.toml"));
        
        let output_dir = get_default_output_directory();
        assert!(output_dir.to_string_lossy().ends_with("reports"));
        
        let log_dir = get_default_log_directory();
        assert!(log_dir.to_string_lossy().ends_with("logs"));
    }
}
