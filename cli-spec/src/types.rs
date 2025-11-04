// Type definitions for CLI spec
//
// These types mirror the YAML spec structure and provide
// type-safe access to CLI configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CliSpec {
    pub version: String,
    pub name: String,
    pub about: String,

    #[serde(default)]
    pub config: CliConfig,

    #[serde(default)]
    pub global_flags: Vec<Flag>,

    #[serde(default)]
    pub exit_codes: ExitCodes,

    #[serde(default)]
    pub commands: Vec<Command>,

    #[serde(default)]
    pub error_mapping: HashMap<String, i32>,

    #[serde(default)]
    pub output_formats: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CliConfig {
    pub precedence: Option<Vec<String>>,
    pub config_path: Option<String>,
    pub base_url: Option<BaseUrlConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BaseUrlConfig {
    pub default: String,
    pub env: String,
    pub flag: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Flag {
    pub name: String,
    pub long: String,
    pub short: Option<String>,
    pub env: Option<String>,
    pub default: Option<String>,
    pub help: String,

    #[serde(rename = "type")]
    pub flag_type: Option<String>,

    pub values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ExitCodes {
    pub success: i32,
    pub user_error: i32,
    pub server_error: i32,
    pub invalid_args: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Command {
    pub name: String,
    pub about: String,

    #[serde(default)]
    pub api: CliApiConfig,

    #[serde(default)]
    pub args: Vec<Argument>,

    #[serde(default)]
    pub flags: Vec<Flag>,

    #[serde(default)]
    pub examples: Vec<Example>,

    #[serde(default)]
    pub logic: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CliApiConfig {
    pub method: Option<String>,
    pub path: Option<String>,
    pub streaming_variant: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Argument {
    pub name: String,

    #[serde(rename = "type")]
    pub arg_type: String,

    pub required: bool,
    pub help: String,

    #[serde(default)]
    pub multiple: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Example {
    pub command: String,
    pub description: String,
}

impl CliSpec {
    /// Get a command by name
    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|cmd| cmd.name == name)
    }

    /// Get a global flag by name
    pub fn get_global_flag(&self, name: &str) -> Option<&Flag> {
        self.global_flags.iter().find(|flag| flag.name == name)
    }

    /// Check if a command has a specific flag
    pub fn command_has_flag(&self, command: &str, flag: &str) -> bool {
        self.get_command(command)
            .map(|cmd| cmd.flags.iter().any(|f| f.name == flag))
            .unwrap_or(false)
    }

    /// Get exit code for HTTP status
    pub fn get_exit_code_for_status(&self, status: u16) -> i32 {
        let key = status.to_string();
        self.error_mapping.get(&key).copied().unwrap_or_else(|| {
            if status >= 500 {
                self.exit_codes.server_error
            } else if status >= 400 {
                self.exit_codes.user_error
            } else {
                self.exit_codes.success
            }
        })
    }

    /// Get exit code for network error type
    pub fn get_exit_code_for_error(&self, error_type: &str) -> i32 {
        self.error_mapping
            .get(error_type)
            .copied()
            .unwrap_or(self.exit_codes.user_error)
    }
}

impl Command {
    /// Check if command has a specific flag
    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f.name == name)
    }

    /// Get a flag by name
    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.iter().find(|f| f.name == name)
    }

    /// Check if command supports streaming
    pub fn supports_streaming(&self) -> bool {
        self.api.streaming_variant.is_some()
    }
}
