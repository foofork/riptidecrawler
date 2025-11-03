// CLI Spec Validation Tests
// Tests the YAML spec structure and parser against CLI refactoring requirements
//
// Requirements tested:
// 1. YAML spec loads successfully
// 2. All 7 commands are present (extract, spider, search, render, doctor, config, session)
// 3. API endpoints are correctly mapped
// 4. Global flags are defined
// 5. Exit codes are valid
// 6. Error mapping is complete (4xx→1, 5xx→2)
// 7. Extraction strategy rules:
//    - extract has strategy/quality/selector flags
//    - spider/search do NOT have strategy flags
// 8. Streaming endpoints identified correctly
// 9. Examples are present for each command

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Spec Data Structures
// ============================================================================

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
    pub api: ApiConfig,

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
pub struct ApiConfig {
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
    pub multiple: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Example {
    pub command: String,
    pub description: String,
}

// ============================================================================
// Test Utilities
// ============================================================================

fn load_spec() -> Result<CliSpec, Box<dyn std::error::Error>> {
    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec.yaml");

    let contents = fs::read_to_string(&spec_path).map_err(|e| {
        format!(
            "Failed to read spec.yaml: {}. Expected at {:?}",
            e, spec_path
        )
    })?;

    let spec: CliSpec =
        serde_yaml::from_str(&contents).map_err(|e| format!("Failed to parse spec.yaml: {}", e))?;

    Ok(spec)
}

fn get_command<'a>(spec: &'a CliSpec, name: &str) -> Option<&'a Command> {
    spec.commands.iter().find(|cmd| cmd.name == name)
}

fn has_flag(flags: &[Flag], name: &str) -> bool {
    flags.iter().any(|f| f.name == name)
}

// ============================================================================
// Core Spec Tests
// ============================================================================

#[test]
fn test_spec_loads() {
    let result = load_spec();
    assert!(
        result.is_ok(),
        "Failed to load spec.yaml: {:?}",
        result.err()
    );
}

#[test]
fn test_spec_version_present() {
    let spec = load_spec().expect("Failed to load spec");
    assert!(!spec.version.is_empty(), "Spec version must be present");
    assert!(
        spec.version.starts_with("1."),
        "Spec version should be 1.x for v1.0 release, got: {}",
        spec.version
    );
}

#[test]
fn test_spec_metadata() {
    let spec = load_spec().expect("Failed to load spec");

    assert_eq!(spec.name, "riptide", "CLI name must be 'riptide'");
    assert!(!spec.about.is_empty(), "CLI description must be present");
}

// ============================================================================
// Command Presence Tests
// ============================================================================

#[test]
fn test_all_commands_present() {
    let spec = load_spec().expect("Failed to load spec");

    let required_commands = vec![
        "extract", "spider", "search", "render", "doctor", "config", "session",
    ];

    for cmd_name in required_commands {
        let cmd = get_command(&spec, cmd_name);
        assert!(
            cmd.is_some(),
            "Required command '{}' not found in spec. Found commands: {:?}",
            cmd_name,
            spec.commands.iter().map(|c| &c.name).collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_command_count() {
    let spec = load_spec().expect("Failed to load spec");

    assert_eq!(
        spec.commands.len(),
        7,
        "Expected exactly 7 commands for v1.0, found {}. Commands: {:?}",
        spec.commands.len(),
        spec.commands.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
}

#[test]
fn test_all_commands_have_descriptions() {
    let spec = load_spec().expect("Failed to load spec");

    for cmd in &spec.commands {
        assert!(
            !cmd.about.is_empty(),
            "Command '{}' must have a description",
            cmd.name
        );
    }
}

// ============================================================================
// API Endpoint Mapping Tests
// ============================================================================

#[test]
fn test_api_endpoint_mapping() {
    let spec = load_spec().expect("Failed to load spec");

    let expected_mappings = vec![
        ("extract", "POST", "/extract"),
        ("spider", "POST", "/spider/crawl"),
        ("search", "POST", "/deepsearch"),
        ("render", "POST", "/render"),
        ("doctor", "GET", "/healthz"),
    ];

    for (cmd_name, expected_method, expected_path) in expected_mappings {
        let cmd = get_command(&spec, cmd_name)
            .unwrap_or_else(|| panic!("Command '{}' not found", cmd_name));

        // Config and session may not have API mappings (local/multiple endpoints)
        if cmd_name == "config" || cmd_name == "session" {
            continue;
        }

        assert_eq!(
            cmd.api.method.as_deref(),
            Some(expected_method),
            "Command '{}' should use {} method",
            cmd_name,
            expected_method
        );

        assert_eq!(
            cmd.api.path.as_deref(),
            Some(expected_path),
            "Command '{}' should map to {} endpoint",
            cmd_name,
            expected_path
        );
    }
}

#[test]
fn test_streaming_endpoints_identified() {
    let spec = load_spec().expect("Failed to load spec");

    // Commands that should have streaming variants
    let streaming_commands = vec![("search", "/deepsearch/stream")];

    for (cmd_name, expected_stream_path) in streaming_commands {
        let cmd = get_command(&spec, cmd_name)
            .unwrap_or_else(|| panic!("Command '{}' not found", cmd_name));

        assert_eq!(
            cmd.api.streaming_variant.as_deref(),
            Some(expected_stream_path),
            "Command '{}' should have streaming variant at {}",
            cmd_name,
            expected_stream_path
        );
    }
}

#[test]
fn test_no_unexpected_streaming_endpoints() {
    let spec = load_spec().expect("Failed to load spec");

    // Commands that should NOT have streaming (spider, render, doctor)
    let non_streaming = vec!["spider", "render", "doctor"];

    for cmd_name in non_streaming {
        let cmd = get_command(&spec, cmd_name)
            .unwrap_or_else(|| panic!("Command '{}' not found", cmd_name));

        assert!(
            cmd.api.streaming_variant.is_none(),
            "Command '{}' should NOT have streaming variant",
            cmd_name
        );
    }
}

// ============================================================================
// Extraction Strategy Tests
// ============================================================================

#[test]
fn test_extract_has_strategy_flags() {
    let spec = load_spec().expect("Failed to load spec");
    let extract = get_command(&spec, "extract").expect("extract command not found");

    // Extract command MUST have strategy control
    let required_flags = vec!["strategy", "quality-threshold", "timeout"];

    for flag_name in required_flags {
        assert!(
            has_flag(&extract.flags, flag_name),
            "extract command must have '{}' flag for extraction control",
            flag_name
        );
    }
}

#[test]
fn test_extract_has_selector_flag() {
    let spec = load_spec().expect("Failed to load spec");
    let extract = get_command(&spec, "extract").expect("extract command not found");

    // CSS selector support
    assert!(
        has_flag(&extract.flags, "selector"),
        "extract command must have 'selector' flag for CSS extraction"
    );
}

#[test]
fn test_extract_strategy_values() {
    let spec = load_spec().expect("Failed to load spec");
    let extract = get_command(&spec, "extract").expect("extract command not found");

    let strategy_flag = extract
        .flags
        .iter()
        .find(|f| f.name == "strategy")
        .expect("strategy flag not found");

    // Verify strategy has expected values
    assert!(
        strategy_flag.values.is_some(),
        "strategy flag must define allowed values"
    );

    let values = strategy_flag.values.as_ref().unwrap();
    let expected_strategies = vec!["auto", "css", "wasm", "llm", "multi"];

    for strategy in expected_strategies {
        assert!(
            values.contains(&strategy.to_string()),
            "strategy flag must support '{}' strategy",
            strategy
        );
    }
}

#[test]
fn test_spider_no_strategy_flags() {
    let spec = load_spec().expect("Failed to load spec");
    let spider = get_command(&spec, "spider").expect("spider command not found");

    // Spider should NOT have extraction strategy flags
    let forbidden_flags = vec!["strategy", "quality-threshold", "selector"];

    for flag_name in forbidden_flags {
        assert!(
            !has_flag(&spider.flags, flag_name),
            "spider command must NOT have '{}' flag (API doesn't support extraction control)",
            flag_name
        );
    }
}

#[test]
fn test_search_no_strategy_flags() {
    let spec = load_spec().expect("Failed to load spec");
    let search = get_command(&spec, "search").expect("search command not found");

    // Search should NOT have extraction strategy flags
    let forbidden_flags = vec!["strategy", "quality-threshold", "selector"];

    for flag_name in forbidden_flags {
        assert!(
            !has_flag(&search.flags, flag_name),
            "search command must NOT have '{}' flag (API uses automatic extraction)",
            flag_name
        );
    }
}

#[test]
fn test_render_no_strategy_flags() {
    let spec = load_spec().expect("Failed to load spec");
    let render = get_command(&spec, "render").expect("render command not found");

    // Render should NOT have extraction strategy flags (it's for rendering, not extraction)
    let forbidden_flags = vec!["strategy", "quality-threshold", "selector"];

    for flag_name in forbidden_flags {
        assert!(
            !has_flag(&render.flags, flag_name),
            "render command must NOT have '{}' flag (render is not for extraction)",
            flag_name
        );
    }
}

// ============================================================================
// Global Flags Tests
// ============================================================================

#[test]
fn test_global_flags_defined() {
    let spec = load_spec().expect("Failed to load spec");

    assert!(
        !spec.global_flags.is_empty(),
        "Global flags must be defined"
    );
}

#[test]
fn test_required_global_flags() {
    let spec = load_spec().expect("Failed to load spec");

    let required_global_flags = vec![
        "url",     // API server URL
        "api-key", // Authentication
        "output",  // Output format
        "quiet",   // Suppress progress
        "verbose", // Verbose output
    ];

    for flag_name in required_global_flags {
        assert!(
            has_flag(&spec.global_flags, flag_name),
            "Global flag '{}' must be defined",
            flag_name
        );
    }
}

#[test]
fn test_output_flag_values() {
    let spec = load_spec().expect("Failed to load spec");

    let output_flag = spec
        .global_flags
        .iter()
        .find(|f| f.name == "output")
        .expect("output flag not found");

    assert!(
        output_flag.values.is_some(),
        "output flag must define allowed values"
    );

    let values = output_flag.values.as_ref().unwrap();
    let expected_formats = vec!["json", "table", "text", "ndjson"];

    for format in expected_formats {
        assert!(
            values.contains(&format.to_string()),
            "output flag must support '{}' format",
            format
        );
    }
}

#[test]
fn test_url_flag_has_env_var() {
    let spec = load_spec().expect("Failed to load spec");

    let url_flag = spec
        .global_flags
        .iter()
        .find(|f| f.name == "url")
        .expect("url flag not found");

    assert_eq!(
        url_flag.env.as_deref(),
        Some("RIPTIDE_BASE_URL"),
        "url flag must support RIPTIDE_BASE_URL environment variable"
    );
}

#[test]
fn test_url_flag_default() {
    let spec = load_spec().expect("Failed to load spec");

    let url_flag = spec
        .global_flags
        .iter()
        .find(|f| f.name == "url")
        .expect("url flag not found");

    assert_eq!(
        url_flag.default.as_deref(),
        Some("http://localhost:8080"),
        "url flag default must be http://localhost:8080"
    );
}

// ============================================================================
// Exit Code Tests
// ============================================================================

#[test]
fn test_exit_codes_valid() {
    let spec = load_spec().expect("Failed to load spec");

    assert_eq!(spec.exit_codes.success, 0, "Success exit code must be 0");
    assert_eq!(
        spec.exit_codes.user_error, 1,
        "User error exit code must be 1"
    );
    assert_eq!(
        spec.exit_codes.server_error, 2,
        "Server error exit code must be 2"
    );
    assert_eq!(
        spec.exit_codes.invalid_args, 3,
        "Invalid args exit code must be 3"
    );
}

// ============================================================================
// Error Mapping Tests
// ============================================================================

#[test]
fn test_error_mapping_complete() {
    let spec = load_spec().expect("Failed to load spec");

    assert!(
        !spec.error_mapping.is_empty(),
        "Error mapping must be defined"
    );
}

#[test]
fn test_4xx_errors_map_to_user_error() {
    let spec = load_spec().expect("Failed to load spec");

    let client_errors = vec!["400", "401", "403", "404", "429"];

    for status in client_errors {
        let exit_code = spec.error_mapping.get(status);
        assert_eq!(
            exit_code,
            Some(&1),
            "HTTP {} should map to exit code 1 (user error), got {:?}",
            status,
            exit_code
        );
    }
}

#[test]
fn test_5xx_errors_map_to_server_error() {
    let spec = load_spec().expect("Failed to load spec");

    let server_errors = vec!["500", "502", "503", "504"];

    for status in server_errors {
        let exit_code = spec.error_mapping.get(status);
        assert_eq!(
            exit_code,
            Some(&2),
            "HTTP {} should map to exit code 2 (server error), got {:?}",
            status,
            exit_code
        );
    }
}

#[test]
fn test_network_errors_mapped() {
    let spec = load_spec().expect("Failed to load spec");

    let network_errors = vec!["connection_refused", "timeout", "dns_failed"];

    for error_type in network_errors {
        assert!(
            spec.error_mapping.contains_key(error_type),
            "Network error '{}' must be mapped to exit code",
            error_type
        );

        let exit_code = spec.error_mapping.get(error_type);
        assert_eq!(
            exit_code,
            Some(&1),
            "Network error '{}' should map to exit code 1 (user error)",
            error_type
        );
    }
}

// ============================================================================
// Command Examples Tests
// ============================================================================

#[test]
fn test_examples_present_for_each_command() {
    let spec = load_spec().expect("Failed to load spec");

    for cmd in &spec.commands {
        assert!(
            !cmd.examples.is_empty(),
            "Command '{}' must have at least one example",
            cmd.name
        );
    }
}

#[test]
fn test_examples_have_descriptions() {
    let spec = load_spec().expect("Failed to load spec");

    for cmd in &spec.commands {
        for (idx, example) in cmd.examples.iter().enumerate() {
            assert!(
                !example.command.is_empty(),
                "Command '{}' example {} must have command text",
                cmd.name,
                idx
            );
            assert!(
                !example.description.is_empty(),
                "Command '{}' example {} must have description",
                cmd.name,
                idx
            );
        }
    }
}

#[test]
fn test_extract_examples_show_strategy_usage() {
    let spec = load_spec().expect("Failed to load spec");
    let extract = get_command(&spec, "extract").expect("extract command not found");

    // At least one example should demonstrate strategy usage
    let has_strategy_example = extract
        .examples
        .iter()
        .any(|ex| ex.command.contains("--strategy"));

    assert!(
        has_strategy_example,
        "extract command examples should demonstrate --strategy flag usage"
    );
}

#[test]
fn test_spider_examples_show_depth_usage() {
    let spec = load_spec().expect("Failed to load spec");
    let spider = get_command(&spec, "spider").expect("spider command not found");

    // At least one example should demonstrate depth usage
    let has_depth_example = spider
        .examples
        .iter()
        .any(|ex| ex.command.contains("--depth"));

    assert!(
        has_depth_example,
        "spider command examples should demonstrate --depth flag usage"
    );
}

// ============================================================================
// Command-Specific Argument Tests
// ============================================================================

#[test]
fn test_extract_has_url_argument() {
    let spec = load_spec().expect("Failed to load spec");
    let extract = get_command(&spec, "extract").expect("extract command not found");

    let has_url = extract.args.iter().any(|arg| arg.name == "url");
    assert!(has_url, "extract command must have 'url' argument");
}

#[test]
fn test_spider_has_url_argument() {
    let spec = load_spec().expect("Failed to load spec");
    let spider = get_command(&spec, "spider").expect("spider command not found");

    let has_url = spider.args.iter().any(|arg| arg.name == "url");
    assert!(has_url, "spider command must have 'url' argument");
}

#[test]
fn test_search_has_query_argument() {
    let spec = load_spec().expect("Failed to load spec");
    let search = get_command(&spec, "search").expect("search command not found");

    let has_query = search.args.iter().any(|arg| arg.name == "query");
    assert!(has_query, "search command must have 'query' argument");
}

// ============================================================================
// Doctor Command Tests
// ============================================================================

#[test]
fn test_doctor_has_full_flag() {
    let spec = load_spec().expect("Failed to load spec");
    let doctor = get_command(&spec, "doctor").expect("doctor command not found");

    assert!(
        has_flag(&doctor.flags, "full"),
        "doctor command must have 'full' flag for comprehensive diagnostics"
    );
}

#[test]
fn test_doctor_has_diagnostic_logic() {
    let spec = load_spec().expect("Failed to load spec");
    let doctor = get_command(&spec, "doctor").expect("doctor command not found");

    assert!(
        doctor.logic.is_some(),
        "doctor command should define diagnostic logic"
    );
}

// ============================================================================
// Session Command Tests
// ============================================================================

#[test]
fn test_session_command_present() {
    let spec = load_spec().expect("Failed to load spec");

    let session = get_command(&spec, "session");
    assert!(
        session.is_some(),
        "session command must be present for authenticated crawling"
    );
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_config_precedence_defined() {
    let spec = load_spec().expect("Failed to load spec");

    assert!(
        spec.config.precedence.is_some(),
        "Configuration precedence must be defined"
    );

    let precedence = spec.config.precedence.as_ref().unwrap();
    assert_eq!(
        precedence.as_slice(),
        &["flags", "env", "config_file"],
        "Precedence should be: flags > env > config_file"
    );
}

#[test]
fn test_config_path_defined() {
    let spec = load_spec().expect("Failed to load spec");

    assert!(
        spec.config.config_path.is_some(),
        "Default config file path must be defined"
    );

    let path = spec.config.config_path.as_ref().unwrap();
    assert!(
        path.contains(".config/riptide"),
        "Config path should be in ~/.config/riptide/"
    );
}

// ============================================================================
// Spec Consistency Tests
// ============================================================================

#[test]
fn test_no_duplicate_command_names() {
    let spec = load_spec().expect("Failed to load spec");

    let mut seen = std::collections::HashSet::new();
    for cmd in &spec.commands {
        assert!(
            seen.insert(&cmd.name),
            "Duplicate command name found: '{}'",
            cmd.name
        );
    }
}

#[test]
fn test_no_duplicate_flag_names_in_commands() {
    let spec = load_spec().expect("Failed to load spec");

    for cmd in &spec.commands {
        let mut seen = std::collections::HashSet::new();
        for flag in &cmd.flags {
            assert!(
                seen.insert(&flag.name),
                "Duplicate flag '{}' in command '{}'",
                flag.name,
                cmd.name
            );
        }
    }
}

#[test]
fn test_all_flags_have_help_text() {
    let spec = load_spec().expect("Failed to load spec");

    // Check global flags
    for flag in &spec.global_flags {
        assert!(
            !flag.help.is_empty(),
            "Global flag '{}' must have help text",
            flag.name
        );
    }

    // Check command flags
    for cmd in &spec.commands {
        for flag in &cmd.flags {
            assert!(
                !flag.help.is_empty(),
                "Flag '{}' in command '{}' must have help text",
                flag.name,
                cmd.name
            );
        }
    }
}

// ============================================================================
// Spec Completeness Tests
// ============================================================================

#[test]
fn test_spec_is_complete_for_v1() {
    let spec = load_spec().expect("Failed to load spec");

    // Verify all critical sections are present
    assert!(!spec.commands.is_empty(), "Commands must be defined");
    assert!(
        !spec.global_flags.is_empty(),
        "Global flags must be defined"
    );
    assert!(
        !spec.error_mapping.is_empty(),
        "Error mapping must be defined"
    );
    assert_eq!(spec.exit_codes.success, 0, "Exit codes must be defined");

    // Verify all 7 required commands
    let required = vec![
        "extract", "spider", "search", "render", "doctor", "config", "session",
    ];
    for cmd_name in required {
        assert!(
            get_command(&spec, cmd_name).is_some(),
            "Required command '{}' missing",
            cmd_name
        );
    }
}

#[test]
fn test_spec_matches_refactoring_plan() {
    let spec = load_spec().expect("Failed to load spec");

    // From CLI-REFACTORING-PLAN.md requirements:
    // 1. Exactly 7 commands for v1.0
    assert_eq!(spec.commands.len(), 7, "v1.0 must have exactly 7 commands");

    // 2. extract is PRIMARY command with full control
    let extract = get_command(&spec, "extract").expect("extract must exist");
    assert!(
        has_flag(&extract.flags, "strategy"),
        "extract must have strategy flag"
    );

    // 3. spider/search do NOT have extraction control
    let spider = get_command(&spec, "spider").expect("spider must exist");
    assert!(
        !has_flag(&spider.flags, "strategy"),
        "spider must NOT have strategy flag"
    );

    let search = get_command(&spec, "search").expect("search must exist");
    assert!(
        !has_flag(&search.flags, "strategy"),
        "search must NOT have strategy flag"
    );

    // 4. Exit codes match specification
    assert_eq!(spec.exit_codes.success, 0);
    assert_eq!(spec.exit_codes.user_error, 1);
    assert_eq!(spec.exit_codes.server_error, 2);
    assert_eq!(spec.exit_codes.invalid_args, 3);
}
