#![allow(clippy::all, dead_code, unused)]

//! Tests for configuration and directory management
//!
//! Coverage includes:
//! - Directory path resolution
//! - Environment variable overrides
//! - Directory creation
//! - Configuration helpers

use riptide_cli::config::{self, env, *};
use std::env as std_env;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_output_directory_default() {
    std_env::remove_var("RIPTIDE_OUTPUT_DIR");

    let output_dir = get_output_directory();
    let output_str = output_dir.to_string_lossy();

    assert!(
        output_str.contains("riptide") || output_str.contains("riptide-output"),
        "Output directory should contain 'riptide': {}",
        output_str
    );
}

#[test]
fn test_output_directory_env_override() {
    let custom_path = "/tmp/test-riptide-output";
    std_env::set_var("RIPTIDE_OUTPUT_DIR", custom_path);

    let output_dir = get_output_directory();
    assert_eq!(output_dir, PathBuf::from(custom_path));

    std_env::remove_var("RIPTIDE_OUTPUT_DIR");
}

#[test]
fn test_screenshots_directory() {
    std_env::remove_var("RIPTIDE_OUTPUT_DIR");

    let base = get_output_directory();
    let screenshots = get_screenshots_directory();

    assert_eq!(screenshots, base.join("screenshots"));
}

#[test]
fn test_html_directory() {
    let base = get_output_directory();
    let html = get_html_directory();

    assert_eq!(html, base.join("html"));
}

#[test]
fn test_pdf_directory() {
    let base = get_output_directory();
    let pdf = get_pdf_directory();

    assert_eq!(pdf, base.join("pdf"));
}

#[test]
fn test_dom_directory() {
    let base = get_output_directory();
    let dom = get_dom_directory();

    assert_eq!(dom, base.join("dom"));
}

#[test]
fn test_har_directory() {
    let base = get_output_directory();
    let har = get_har_directory();

    assert_eq!(har, base.join("har"));
}

#[test]
fn test_reports_directory() {
    let base = get_output_directory();
    let reports = get_reports_directory();

    assert_eq!(reports, base.join("reports"));
}

#[test]
fn test_crawl_directory() {
    let base = get_output_directory();
    let crawl = get_crawl_directory();

    assert_eq!(crawl, base.join("crawl"));
}

#[test]
fn test_sessions_directory() {
    let base = get_output_directory();
    let sessions = get_sessions_directory();

    assert_eq!(sessions, base.join("sessions"));
}

#[test]
fn test_cache_directory_default() {
    std_env::remove_var("RIPTIDE_CACHE_DIR");

    let cache_dir = get_cache_directory();
    let cache_str = cache_dir.to_string_lossy();

    assert!(
        cache_str.contains("riptide"),
        "Cache directory should contain 'riptide': {}",
        cache_str
    );
}

#[test]
fn test_cache_directory_env_override() {
    let custom_cache = "/tmp/test-riptide-cache";
    std_env::set_var("RIPTIDE_CACHE_DIR", custom_cache);

    let cache_dir = get_cache_directory();
    assert_eq!(cache_dir, PathBuf::from(custom_cache));

    std_env::remove_var("RIPTIDE_CACHE_DIR");
}

#[test]
fn test_logs_directory_default() {
    std_env::remove_var("RIPTIDE_LOGS_DIR");

    let logs_dir = get_logs_directory();
    let logs_str = logs_dir.to_string_lossy();

    assert!(
        logs_str.contains("riptide") || logs_str.contains("riptide-logs"),
        "Logs directory should contain 'riptide': {}",
        logs_str
    );
}

#[test]
fn test_logs_directory_env_override() {
    let custom_logs = "/tmp/test-riptide-logs";
    std_env::set_var("RIPTIDE_LOGS_DIR", custom_logs);

    let logs_dir = get_logs_directory();
    assert_eq!(logs_dir, PathBuf::from(custom_logs));

    std_env::remove_var("RIPTIDE_LOGS_DIR");
}

#[test]
fn test_ensure_directory_exists_new() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("new-directory");

    assert!(!test_path.exists());

    ensure_directory_exists(&test_path).unwrap();
    assert!(test_path.exists());
    assert!(test_path.is_dir());
}

#[test]
fn test_ensure_directory_exists_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let test_path = temp_dir.path().join("existing-directory");

    std::fs::create_dir(&test_path).unwrap();
    assert!(test_path.exists());

    // Should not error on existing directory
    ensure_directory_exists(&test_path).unwrap();
    assert!(test_path.exists());
}

#[test]
fn test_ensure_directory_nested() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("level1").join("level2").join("level3");

    assert!(!nested_path.exists());

    ensure_directory_exists(&nested_path).unwrap();
    assert!(nested_path.exists());
}

#[test]
fn test_initialize_directories() {
    let temp_dir = TempDir::new().unwrap();
    std_env::set_var(
        "RIPTIDE_OUTPUT_DIR",
        temp_dir.path().to_string_lossy().to_string(),
    );
    std_env::set_var(
        "RIPTIDE_CACHE_DIR",
        temp_dir.path().join("cache").to_string_lossy().to_string(),
    );
    std_env::set_var(
        "RIPTIDE_LOGS_DIR",
        temp_dir.path().join("logs").to_string_lossy().to_string(),
    );

    initialize_directories().unwrap();

    // Verify all directories were created
    assert!(get_output_directory().exists());
    assert!(get_screenshots_directory().exists());
    assert!(get_html_directory().exists());
    assert!(get_pdf_directory().exists());
    assert!(get_dom_directory().exists());
    assert!(get_har_directory().exists());
    assert!(get_reports_directory().exists());
    assert!(get_crawl_directory().exists());
    assert!(get_sessions_directory().exists());
    assert!(get_cache_directory().exists());
    assert!(get_logs_directory().exists());

    std_env::remove_var("RIPTIDE_OUTPUT_DIR");
    std_env::remove_var("RIPTIDE_CACHE_DIR");
    std_env::remove_var("RIPTIDE_LOGS_DIR");
}

#[test]
fn test_env_output_dir() {
    std_env::remove_var("RIPTIDE_OUTPUT_DIR");
    assert!(env::output_dir().is_none());

    std_env::set_var("RIPTIDE_OUTPUT_DIR", "/custom/output");
    assert_eq!(env::output_dir(), Some("/custom/output".to_string()));

    std_env::remove_var("RIPTIDE_OUTPUT_DIR");
}

#[test]
fn test_env_cache_dir() {
    std_env::remove_var("RIPTIDE_CACHE_DIR");
    assert!(env::cache_dir().is_none());

    std_env::set_var("RIPTIDE_CACHE_DIR", "/custom/cache");
    assert_eq!(env::cache_dir(), Some("/custom/cache".to_string()));

    std_env::remove_var("RIPTIDE_CACHE_DIR");
}

#[test]
fn test_env_logs_dir() {
    std_env::remove_var("RIPTIDE_LOGS_DIR");
    assert!(env::logs_dir().is_none());

    std_env::set_var("RIPTIDE_LOGS_DIR", "/custom/logs");
    assert_eq!(env::logs_dir(), Some("/custom/logs".to_string()));

    std_env::remove_var("RIPTIDE_LOGS_DIR");
}

#[test]
fn test_env_api_host() {
    std_env::remove_var("RIPTIDE_API_HOST");
    assert!(env::api_host().is_none());

    std_env::set_var("RIPTIDE_API_HOST", "api.example.com");
    assert_eq!(env::api_host(), Some("api.example.com".to_string()));

    std_env::remove_var("RIPTIDE_API_HOST");
}

#[test]
fn test_env_api_port() {
    std_env::remove_var("RIPTIDE_API_PORT");
    assert!(env::api_port().is_none());

    std_env::set_var("RIPTIDE_API_PORT", "8080");
    assert_eq!(env::api_port(), Some(8080));

    std_env::remove_var("RIPTIDE_API_PORT");
}

#[test]
fn test_env_api_port_invalid() {
    std_env::set_var("RIPTIDE_API_PORT", "not-a-number");
    assert!(env::api_port().is_none());

    std_env::remove_var("RIPTIDE_API_PORT");
}

#[test]
fn test_env_log_level() {
    std_env::remove_var("RIPTIDE_LOG_LEVEL");
    std_env::remove_var("RUST_LOG");

    assert!(env::log_level().is_none());

    std_env::set_var("RIPTIDE_LOG_LEVEL", "debug");
    assert_eq!(env::log_level(), Some("debug".to_string()));

    std_env::remove_var("RIPTIDE_LOG_LEVEL");
}

#[test]
fn test_env_log_level_fallback_to_rust_log() {
    std_env::remove_var("RIPTIDE_LOG_LEVEL");
    std_env::set_var("RUST_LOG", "info");

    assert_eq!(env::log_level(), Some("info".to_string()));

    std_env::remove_var("RUST_LOG");
}

#[test]
fn test_env_is_development_mode() {
    std_env::remove_var("RIPTIDE_DEV");
    std_env::remove_var("DEVELOPMENT");

    // In debug builds, should return true
    if cfg!(debug_assertions) {
        assert!(env::is_development_mode());
    }

    std_env::set_var("RIPTIDE_DEV", "1");
    assert!(env::is_development_mode());

    std_env::remove_var("RIPTIDE_DEV");
}

#[test]
fn test_env_development_flag() {
    std_env::set_var("DEVELOPMENT", "true");
    assert!(env::is_development_mode());

    std_env::remove_var("DEVELOPMENT");
}

#[test]
fn test_multiple_env_overrides() {
    std_env::set_var("RIPTIDE_OUTPUT_DIR", "/tmp/output");
    std_env::set_var("RIPTIDE_CACHE_DIR", "/tmp/cache");
    std_env::set_var("RIPTIDE_LOGS_DIR", "/tmp/logs");

    assert_eq!(get_output_directory(), PathBuf::from("/tmp/output"));
    assert_eq!(get_cache_directory(), PathBuf::from("/tmp/cache"));
    assert_eq!(get_logs_directory(), PathBuf::from("/tmp/logs"));

    std_env::remove_var("RIPTIDE_OUTPUT_DIR");
    std_env::remove_var("RIPTIDE_CACHE_DIR");
    std_env::remove_var("RIPTIDE_LOGS_DIR");
}

#[test]
fn test_path_consistency() {
    std_env::remove_var("RIPTIDE_OUTPUT_DIR");

    let output1 = get_output_directory();
    let output2 = get_output_directory();

    assert_eq!(output1, output2, "Path should be consistent across calls");
}
