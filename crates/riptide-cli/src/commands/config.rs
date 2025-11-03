/// Config command - Local YAML-based configuration management
use anyhow::{anyhow, Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration structure for RipTide CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub crawl: CrawlConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_api_url")]
    pub url: String,
    pub key: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_cache_mode")]
    pub cache_mode: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: default_api_url(),
            key: None,
            timeout: default_timeout(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
        }
    }
}

impl Default for CrawlConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            cache_mode: default_cache_mode(),
        }
    }
}

fn default_api_url() -> String {
    "http://localhost:8080".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_output_format() -> String {
    "text".to_string()
}

fn default_concurrency() -> usize {
    5
}

fn default_cache_mode() -> String {
    "auto".to_string()
}

#[derive(Args, Clone, Debug)]
pub struct ConfigArgs {
    /// Configuration subcommand
    #[arg(value_enum)]
    pub action: ConfigAction,

    /// Configuration key (for get/set)
    pub key: Option<String>,

    /// Configuration value (for set)
    pub value: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum ConfigAction {
    /// Get a configuration value
    Get,
    /// Set a configuration value
    Set,
    /// List all configuration values
    List,
    /// Reset configuration to defaults
    Reset,
    /// Show configuration file path
    Path,
}

/// Get the config file path (~/.config/riptide/config.yaml)
pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not determine config directory"))?
        .join("riptide");

    Ok(config_dir.join("config.yaml"))
}

/// Load configuration from file, or return default if file doesn't exist
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

    let config: Config = serde_yaml::from_str(&content).context("Failed to parse config file")?;

    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;

    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let yaml = serde_yaml::to_string(config).context("Failed to serialize config")?;

    fs::write(&config_path, yaml).context("Failed to write config file")?;

    Ok(())
}

/// Get a nested configuration value by key path (e.g., "api.url")
fn get_config_value(config: &Config, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["api", "url"] => Ok(config.api.url.clone()),
        ["api", "key"] => Ok(config
            .api
            .key
            .clone()
            .unwrap_or_else(|| "not set".to_string())),
        ["api", "timeout"] => Ok(config.api.timeout.to_string()),
        ["output", "format"] => Ok(config.output.format.clone()),
        ["crawl", "concurrency"] => Ok(config.crawl.concurrency.to_string()),
        ["crawl", "cache_mode"] => Ok(config.crawl.cache_mode.clone()),
        _ => Err(anyhow!("Unknown configuration key: {}", key)),
    }
}

/// Set a nested configuration value by key path
fn set_config_value(config: &mut Config, key: &str, value: &str) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["api", "url"] => {
            config.api.url = value.to_string();
            Ok(())
        }
        ["api", "key"] => {
            config.api.key = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            Ok(())
        }
        ["api", "timeout"] => {
            config.api.timeout = value.parse().context("Timeout must be a number")?;
            Ok(())
        }
        ["output", "format"] => {
            if !["text", "json", "table"].contains(&value) {
                return Err(anyhow!("Format must be one of: text, json, table"));
            }
            config.output.format = value.to_string();
            Ok(())
        }
        ["crawl", "concurrency"] => {
            config.crawl.concurrency = value.parse().context("Concurrency must be a number")?;
            Ok(())
        }
        ["crawl", "cache_mode"] => {
            if !["auto", "read", "write", "off"].contains(&value) {
                return Err(anyhow!("Cache mode must be one of: auto, read, write, off"));
            }
            config.crawl.cache_mode = value.to_string();
            Ok(())
        }
        _ => Err(anyhow!("Unknown configuration key: {}", key)),
    }
}

pub async fn execute(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::Get => {
            let key = args
                .key
                .ok_or_else(|| anyhow!("Key is required for 'get' command"))?;
            let config = load_config()?;
            let value = get_config_value(&config, &key)?;
            println!("{}", value);
        }

        ConfigAction::Set => {
            let key = args
                .key
                .ok_or_else(|| anyhow!("Key is required for 'set' command"))?;
            let value = args
                .value
                .ok_or_else(|| anyhow!("Value is required for 'set' command"))?;

            let mut config = load_config()?;
            set_config_value(&mut config, &key, &value)?;
            save_config(&config)?;

            println!("✓ Set {} = {}", key, value);
        }

        ConfigAction::List => {
            let config = load_config()?;
            let yaml = serde_yaml::to_string(&config).context("Failed to serialize config")?;
            println!("{}", yaml);
        }

        ConfigAction::Reset => {
            let config_path = get_config_path()?;

            if config_path.exists() {
                fs::remove_file(&config_path).context("Failed to remove config file")?;
                println!("✓ Configuration reset to defaults");
            } else {
                println!("Configuration file does not exist");
            }
        }

        ConfigAction::Path => {
            let config_path = get_config_path()?;
            println!("{}", config_path.display());
        }
    }

    Ok(())
}
