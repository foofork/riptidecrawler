//! YAML Specification Parser
//!
//! Parses cli.yaml files and validates their structure for CLI generation.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Parser-specific errors
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to read spec file: {0}")]
    ReadError(String),

    #[error("Failed to parse YAML: {0}")]
    YamlError(String),

    #[error("Invalid spec structure: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid endpoint mapping: {0}")]
    EndpointError(String),
}

/// HTTP methods supported by the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// API endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    /// HTTP method (GET, POST, etc.)
    pub method: HttpMethod,

    /// API endpoint path (e.g., "/api/v1/events")
    pub endpoint: String,

    /// Optional request body template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// Required request parameters
    #[serde(default)]
    pub requires: Vec<String>,

    /// Optional query parameters
    #[serde(default)]
    pub query_params: Vec<String>,
}

/// Command argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arg {
    /// Argument name
    pub name: String,

    /// Help text
    pub help: String,

    /// Whether the argument is required
    #[serde(default)]
    pub required: bool,

    /// Default value if not provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Value name for help display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,

    /// Whether this is a variadic argument
    #[serde(default)]
    pub multiple: bool,
}

/// Command flag definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    /// Flag name (long form)
    pub name: String,

    /// Short flag (single character)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<char>,

    /// Help text
    pub help: String,

    /// Whether the flag takes a value
    #[serde(default)]
    pub takes_value: bool,

    /// Default value if flag is provided without value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,

    /// Value name for help display
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,

    /// Possible values (for validation)
    #[serde(default)]
    pub possible_values: Vec<String>,

    /// Whether this flag is global (applies to all commands)
    #[serde(default)]
    pub global: bool,
}

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Command name
    pub name: String,

    /// Short description
    pub about: String,

    /// Long description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_about: Option<String>,

    /// API endpoint mapping
    pub api: ApiEndpoint,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<Arg>,

    /// Command-specific flags
    #[serde(default)]
    pub flags: Vec<Flag>,

    /// Subcommands
    #[serde(default)]
    pub subcommands: Vec<Command>,

    /// Whether this command requires authentication
    #[serde(default)]
    pub requires_auth: bool,

    /// Example usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Exit code definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodes {
    /// Success exit code
    #[serde(default = "default_success")]
    pub success: i32,

    /// General error exit code
    #[serde(default = "default_error")]
    pub error: i32,

    /// Network error exit code
    #[serde(default = "default_network_error")]
    pub network_error: i32,

    /// Authentication error exit code
    #[serde(default = "default_auth_error")]
    pub auth_error: i32,

    /// Validation error exit code
    #[serde(default = "default_validation_error")]
    pub validation_error: i32,

    /// Not found error exit code
    #[serde(default = "default_not_found")]
    pub not_found: i32,
}

fn default_success() -> i32 {
    0
}
fn default_error() -> i32 {
    1
}
fn default_network_error() -> i32 {
    2
}
fn default_auth_error() -> i32 {
    3
}
fn default_validation_error() -> i32 {
    4
}
fn default_not_found() -> i32 {
    5
}

impl Default for ExitCodes {
    fn default() -> Self {
        Self {
            success: 0,
            error: 1,
            network_error: 2,
            auth_error: 3,
            validation_error: 4,
            not_found: 5,
        }
    }
}

/// HTTP status code to exit code mapping
#[derive(Debug, Clone, Serialize)]
pub struct ErrorMapping {
    /// Maps HTTP status codes to CLI exit codes
    #[serde(flatten)]
    pub mappings: HashMap<u16, i32>,
}

impl<'de> Deserialize<'de> for ErrorMapping {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use std::collections::HashMap as Map;

        let map: Map<serde_yaml::Value, i32> = Map::deserialize(deserializer)?;
        let mut mappings = HashMap::new();

        for (key, value) in map {
            match key {
                serde_yaml::Value::Number(n) => {
                    if let Some(code) = n.as_u64() {
                        if code <= u16::MAX as u64 {
                            mappings.insert(code as u16, value);
                        } else {
                            return Err(D::Error::custom(format!(
                                "HTTP status code {} out of range",
                                code
                            )));
                        }
                    } else {
                        return Err(D::Error::custom("Invalid HTTP status code"));
                    }
                }
                serde_yaml::Value::String(s) => {
                    // Handle string keys like "connection_refused", "timeout", etc.
                    // These are not HTTP status codes, so we skip them for now
                    // They should be handled separately in the spec
                    if s.parse::<u16>().is_ok() {
                        return Err(D::Error::custom(format!(
                            "HTTP status code should be unquoted number, not string: {:?}",
                            s
                        )));
                    }
                    // Skip non-numeric string keys (they might be error types like "connection_refused")
                    continue;
                }
                _ => return Err(D::Error::custom("Invalid error_mapping key type")),
            }
        }

        Ok(ErrorMapping { mappings })
    }
}

impl ErrorMapping {
    /// Get exit code for HTTP status code
    pub fn get_exit_code(&self, status_code: u16) -> Option<i32> {
        self.mappings.get(&status_code).copied()
    }

    /// Get exit code or default error code
    pub fn get_exit_code_or_default(&self, status_code: u16, default: i32) -> i32 {
        self.get_exit_code(status_code).unwrap_or(default)
    }
}

/// API mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMapping {
    /// Base URL for the API
    pub base_url: String,

    /// API version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Default timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Whether to follow redirects
    #[serde(default = "default_true")]
    pub follow_redirects: bool,

    /// Maximum number of redirects to follow
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,
}

fn default_timeout() -> u64 {
    30
}
fn default_true() -> bool {
    true
}
fn default_max_redirects() -> u32 {
    10
}

/// Complete CLI specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSpec {
    /// Specification version
    pub version: String,

    /// CLI name
    pub name: String,

    /// CLI description
    pub about: String,

    /// CLI author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Available commands
    pub commands: Vec<Command>,

    /// Global flags
    #[serde(default)]
    pub global_flags: Vec<Flag>,

    /// Exit code definitions
    #[serde(default)]
    pub exit_codes: ExitCodes,

    /// HTTP status to exit code mapping
    #[serde(default)]
    pub error_mapping: ErrorMapping,

    /// API configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiMapping>,
}

impl Default for ErrorMapping {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert(200, 0); // OK
        mappings.insert(201, 0); // Created
        mappings.insert(204, 0); // No Content
        mappings.insert(400, 4); // Bad Request -> validation_error
        mappings.insert(401, 3); // Unauthorized -> auth_error
        mappings.insert(403, 3); // Forbidden -> auth_error
        mappings.insert(404, 5); // Not Found -> not_found
        mappings.insert(500, 1); // Internal Server Error -> error
        mappings.insert(502, 2); // Bad Gateway -> network_error
        mappings.insert(503, 2); // Service Unavailable -> network_error
        mappings.insert(504, 2); // Gateway Timeout -> network_error

        Self { mappings }
    }
}

/// YAML specification parser
pub struct SpecParser;

impl SpecParser {
    /// Parse CLI specification from YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CliSpec, ParserError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .map_err(|e| ParserError::ReadError(format!("{}: {}", path.display(), e)))?;

        Self::parse_str(&contents)
    }

    /// Parse CLI specification from YAML string
    pub fn parse_str(yaml: &str) -> Result<CliSpec, ParserError> {
        let spec: CliSpec =
            serde_yaml::from_str(yaml).map_err(|e| ParserError::YamlError(e.to_string()))?;

        Self::validate(&spec)?;
        Ok(spec)
    }

    /// Validate the parsed specification
    fn validate(spec: &CliSpec) -> Result<(), ParserError> {
        // Validate version is present
        if spec.version.is_empty() {
            return Err(ParserError::MissingField("version".to_string()));
        }

        // Validate name is present
        if spec.name.is_empty() {
            return Err(ParserError::MissingField("name".to_string()));
        }

        // Validate commands
        if spec.commands.is_empty() {
            return Err(ParserError::ValidationError(
                "At least one command must be defined".to_string(),
            ));
        }

        // Validate each command
        for cmd in &spec.commands {
            Self::validate_command(cmd)?;
        }

        // Validate global flags don't conflict
        let mut flag_names = std::collections::HashSet::new();
        for flag in &spec.global_flags {
            if !flag_names.insert(&flag.name) {
                return Err(ParserError::ValidationError(format!(
                    "Duplicate global flag: {}",
                    flag.name
                )));
            }
        }

        Ok(())
    }

    /// Validate a single command
    fn validate_command(cmd: &Command) -> Result<(), ParserError> {
        // Validate command name
        if cmd.name.is_empty() {
            return Err(ParserError::ValidationError(
                "Command name cannot be empty".to_string(),
            ));
        }

        // Validate API endpoint
        if cmd.api.endpoint.is_empty() {
            return Err(ParserError::EndpointError(format!(
                "Command '{}' has empty endpoint",
                cmd.name
            )));
        }

        // Validate endpoint path starts with /
        if !cmd.api.endpoint.starts_with('/') {
            return Err(ParserError::EndpointError(format!(
                "Endpoint '{}' must start with '/'",
                cmd.api.endpoint
            )));
        }

        // Validate required parameters are in args or flags
        let arg_names: std::collections::HashSet<_> =
            cmd.args.iter().map(|a| a.name.as_str()).collect();
        let flag_names: std::collections::HashSet<_> =
            cmd.flags.iter().map(|f| f.name.as_str()).collect();

        for required in &cmd.api.requires {
            if !arg_names.contains(required.as_str()) && !flag_names.contains(required.as_str()) {
                return Err(ParserError::ValidationError(format!(
                    "Required parameter '{}' not found in args or flags for command '{}'",
                    required, cmd.name
                )));
            }
        }

        // Validate subcommands recursively
        for subcmd in &cmd.subcommands {
            Self::validate_command(subcmd)?;
        }

        Ok(())
    }

    /// Find a command by name (including subcommands)
    pub fn find_command<'a>(spec: &'a CliSpec, name: &str) -> Option<&'a Command> {
        Self::find_command_recursive(&spec.commands, name)
    }

    fn find_command_recursive<'a>(commands: &'a [Command], name: &str) -> Option<&'a Command> {
        for cmd in commands {
            if cmd.name == name {
                return Some(cmd);
            }
            if let Some(found) = Self::find_command_recursive(&cmd.subcommands, name) {
                return Some(found);
            }
        }
        None
    }

    /// Get all command names (including subcommands)
    pub fn all_command_names(spec: &CliSpec) -> Vec<String> {
        let mut names = Vec::new();
        Self::collect_command_names(&spec.commands, &mut names, None);
        names
    }

    fn collect_command_names(commands: &[Command], names: &mut Vec<String>, prefix: Option<&str>) {
        for cmd in commands {
            let full_name = if let Some(p) = prefix {
                format!("{} {}", p, cmd.name)
            } else {
                cmd.name.clone()
            };

            names.push(full_name.clone());

            if !cmd.subcommands.is_empty() {
                Self::collect_command_names(&cmd.subcommands, names, Some(&full_name));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_spec() -> String {
        r#"
version: "1.0.0"
name: "eventctl"
about: "Event management CLI"
author: "EventMesh Team"

api:
  base_url: "http://localhost:8080"
  version: "v1"
  timeout: 30

commands:
  - name: "events"
    about: "Manage events"
    api:
      method: GET
      endpoint: "/api/v1/events"
      query_params: ["limit", "offset"]
    flags:
      - name: "limit"
        short: "l"
        help: "Maximum number of events"
        takes_value: true
        value_name: "NUM"
      - name: "json"
        short: "j"
        help: "Output as JSON"
        takes_value: false

  - name: "event"
    about: "Event operations"
    api:
      method: GET
      endpoint: "/api/v1/event/{id}"
      requires: ["id"]
    args:
      - name: "id"
        help: "Event ID"
        required: true
        value_name: "EVENT_ID"
    subcommands:
      - name: "create"
        about: "Create a new event"
        api:
          method: POST
          endpoint: "/api/v1/events"
          body: '{"topic": "{{topic}}", "data": "{{data}}"}'
          requires: ["topic", "data"]
        args:
          - name: "topic"
            help: "Event topic"
            required: true
          - name: "data"
            help: "Event data"
            required: true

global_flags:
  - name: "verbose"
    short: "v"
    help: "Verbose output"
    takes_value: false
    global: true

exit_codes:
  success: 0
  error: 1
  network_error: 2
  auth_error: 3
  validation_error: 4
  not_found: 5

error_mapping:
  200: 0
  201: 0
  400: 4
  401: 3
  404: 5
  500: 1
"#
        .to_string()
    }

    #[test]
    fn test_parse_valid_spec() {
        let yaml = create_test_spec();
        let spec = SpecParser::parse_str(&yaml).expect("Failed to parse spec");

        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.name, "eventctl");
        assert_eq!(spec.commands.len(), 2);
        assert_eq!(spec.global_flags.len(), 1);
    }

    #[test]
    fn test_parse_from_file() {
        let yaml = create_test_spec();
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file
            .write_all(yaml.as_bytes())
            .expect("Failed to write");

        let spec = SpecParser::from_file(temp_file.path()).expect("Failed to parse file");
        assert_eq!(spec.version, "1.0.0");
    }

    #[test]
    fn test_validate_missing_version() {
        let yaml = r#"
name: "test"
about: "test"
commands: []
"#;
        let result = SpecParser::parse_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_commands() {
        let yaml = r#"
version: "1.0.0"
name: "test"
about: "test"
commands: []
"#;
        let result = SpecParser::parse_str(yaml);
        assert!(result.is_err());
        if let Err(ParserError::ValidationError(msg)) = result {
            assert!(msg.contains("At least one command"));
        }
    }

    #[test]
    fn test_validate_invalid_endpoint() {
        let yaml = r#"
version: "1.0.0"
name: "test"
about: "test"
commands:
  - name: "test"
    about: "test"
    api:
      method: GET
      endpoint: "invalid"
"#;
        let result = SpecParser::parse_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_command() {
        let yaml = create_test_spec();
        let spec = SpecParser::parse_str(&yaml).expect("Failed to parse");

        let cmd = SpecParser::find_command(&spec, "events").expect("Command not found");
        assert_eq!(cmd.name, "events");

        let subcmd = SpecParser::find_command(&spec, "create").expect("Subcommand not found");
        assert_eq!(subcmd.name, "create");
    }

    #[test]
    fn test_all_command_names() {
        let yaml = create_test_spec();
        let spec = SpecParser::parse_str(&yaml).expect("Failed to parse");

        let names = SpecParser::all_command_names(&spec);
        assert!(names.contains(&"events".to_string()));
        assert!(names.contains(&"event".to_string()));
        assert!(names.contains(&"event create".to_string()));
    }

    #[test]
    fn test_error_mapping() {
        let yaml = create_test_spec();
        let spec = SpecParser::parse_str(&yaml).expect("Failed to parse");

        assert_eq!(spec.error_mapping.get_exit_code(200), Some(0));
        assert_eq!(spec.error_mapping.get_exit_code(404), Some(5));
        assert_eq!(spec.error_mapping.get_exit_code_or_default(999, 1), 1);
    }

    #[test]
    fn test_http_methods() {
        let yaml = r#"
version: "1.0.0"
name: "test"
about: "test"
commands:
  - name: "get"
    about: "GET test"
    api:
      method: GET
      endpoint: "/test"
  - name: "post"
    about: "POST test"
    api:
      method: POST
      endpoint: "/test"
  - name: "put"
    about: "PUT test"
    api:
      method: PUT
      endpoint: "/test"
  - name: "delete"
    about: "DELETE test"
    api:
      method: DELETE
      endpoint: "/test"
  - name: "patch"
    about: "PATCH test"
    api:
      method: PATCH
      endpoint: "/test"
"#;
        let spec = SpecParser::parse_str(yaml).expect("Failed to parse");

        assert_eq!(spec.commands[0].api.method, HttpMethod::Get);
        assert_eq!(spec.commands[1].api.method, HttpMethod::Post);
        assert_eq!(spec.commands[2].api.method, HttpMethod::Put);
        assert_eq!(spec.commands[3].api.method, HttpMethod::Delete);
        assert_eq!(spec.commands[4].api.method, HttpMethod::Patch);
    }

    #[test]
    fn test_validate_required_params() {
        let yaml = r#"
version: "1.0.0"
name: "test"
about: "test"
commands:
  - name: "test"
    about: "test"
    api:
      method: POST
      endpoint: "/test"
      requires: ["missing_param"]
"#;
        let result = SpecParser::parse_str(yaml);
        assert!(result.is_err());
        if let Err(ParserError::ValidationError(msg)) = result {
            assert!(msg.contains("missing_param"));
        }
    }

    #[test]
    fn test_exit_codes_defaults() {
        let codes = ExitCodes::default();
        assert_eq!(codes.success, 0);
        assert_eq!(codes.error, 1);
        assert_eq!(codes.network_error, 2);
        assert_eq!(codes.auth_error, 3);
        assert_eq!(codes.validation_error, 4);
        assert_eq!(codes.not_found, 5);
    }

    #[test]
    fn test_api_mapping() {
        let yaml = create_test_spec();
        let spec = SpecParser::parse_str(&yaml).expect("Failed to parse");

        let api = spec.api.as_ref().expect("API config missing");
        assert_eq!(api.base_url, "http://localhost:8080");
        assert_eq!(api.version.as_ref().unwrap(), "v1");
        assert_eq!(api.timeout, 30);
    }
}
