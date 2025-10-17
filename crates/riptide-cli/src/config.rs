//! Centralized configuration for RipTide CLI
//!
//! This module provides unified configuration for output directories,
//! environment variables, and CLI defaults.

use anyhow::Result;
use std::path::PathBuf;

/// Get the default output directory for all CLI operations
///
/// Priority order:
/// 1. RIPTIDE_OUTPUT_DIR environment variable
/// 2. Platform-specific data directory (~/Library/Application Support/riptide/output on macOS, etc.)
/// 3. Fallback to ./riptide-output in current directory
pub fn get_output_directory() -> PathBuf {
    std::env::var("RIPTIDE_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if let Some(data_dir) = dirs::data_dir() {
                data_dir.join("riptide").join("output")
            } else {
                PathBuf::from("./riptide-output")
            }
        })
}

/// Get the screenshots subdirectory
pub fn get_screenshots_directory() -> PathBuf {
    get_output_directory().join("screenshots")
}

/// Get the HTML files subdirectory
pub fn get_html_directory() -> PathBuf {
    get_output_directory().join("html")
}

/// Get the PDF files subdirectory
pub fn get_pdf_directory() -> PathBuf {
    get_output_directory().join("pdf")
}

/// Get the DOM files subdirectory
pub fn get_dom_directory() -> PathBuf {
    get_output_directory().join("dom")
}

/// Get the HAR files subdirectory
pub fn get_har_directory() -> PathBuf {
    get_output_directory().join("har")
}

/// Get the reports subdirectory
pub fn get_reports_directory() -> PathBuf {
    get_output_directory().join("reports")
}

/// Get the crawl results subdirectory
pub fn get_crawl_directory() -> PathBuf {
    get_output_directory().join("crawl")
}

/// Get the sessions subdirectory
pub fn get_sessions_directory() -> PathBuf {
    get_output_directory().join("sessions")
}

/// Get the cache subdirectory
pub fn get_cache_directory() -> PathBuf {
    std::env::var("RIPTIDE_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if let Some(cache_dir) = dirs::cache_dir() {
                cache_dir.join("riptide")
            } else {
                get_output_directory().join("cache")
            }
        })
}

/// Get the logs subdirectory
pub fn get_logs_directory() -> PathBuf {
    std::env::var("RIPTIDE_LOGS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            if let Some(data_dir) = dirs::data_dir() {
                data_dir.join("riptide").join("logs")
            } else {
                PathBuf::from("./riptide-logs")
            }
        })
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Initialize all required directories
pub fn initialize_directories() -> Result<()> {
    let directories = vec![
        get_output_directory(),
        get_screenshots_directory(),
        get_html_directory(),
        get_pdf_directory(),
        get_dom_directory(),
        get_har_directory(),
        get_reports_directory(),
        get_crawl_directory(),
        get_sessions_directory(),
        get_cache_directory(),
        get_logs_directory(),
    ];

    for dir in directories {
        ensure_directory_exists(&dir)?;
    }

    Ok(())
}

/// Configuration for environment variables
pub mod env {
    /// Get the output directory from environment
    pub fn output_dir() -> Option<String> {
        std::env::var("RIPTIDE_OUTPUT_DIR").ok()
    }

    /// Get the cache directory from environment
    pub fn cache_dir() -> Option<String> {
        std::env::var("RIPTIDE_CACHE_DIR").ok()
    }

    /// Get the logs directory from environment
    pub fn logs_dir() -> Option<String> {
        std::env::var("RIPTIDE_LOGS_DIR").ok()
    }

    /// Get the API host from environment
    pub fn api_host() -> Option<String> {
        std::env::var("RIPTIDE_API_HOST").ok()
    }

    /// Get the API port from environment
    pub fn api_port() -> Option<u16> {
        std::env::var("RIPTIDE_API_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
    }

    /// Get the log level from environment
    pub fn log_level() -> Option<String> {
        std::env::var("RIPTIDE_LOG_LEVEL")
            .ok()
            .or_else(|| std::env::var("RUST_LOG").ok())
    }

    /// Check if running in development mode
    pub fn is_development_mode() -> bool {
        std::env::var("RIPTIDE_DEV").is_ok()
            || std::env::var("DEVELOPMENT").is_ok()
            || cfg!(debug_assertions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_directory_fallback() {
        // Clear environment variable for test
        std::env::remove_var("RIPTIDE_OUTPUT_DIR");

        let output_dir = get_output_directory();
        assert!(
            output_dir.to_string_lossy().contains("riptide")
                || output_dir.to_string_lossy().contains("riptide-output")
        );
    }

    #[test]
    fn test_subdirectories() {
        let base = get_output_directory();

        assert_eq!(get_screenshots_directory(), base.join("screenshots"));
        assert_eq!(get_html_directory(), base.join("html"));
        assert_eq!(get_pdf_directory(), base.join("pdf"));
        assert_eq!(get_dom_directory(), base.join("dom"));
        assert_eq!(get_har_directory(), base.join("har"));
        assert_eq!(get_reports_directory(), base.join("reports"));
        assert_eq!(get_crawl_directory(), base.join("crawl"));
        assert_eq!(get_sessions_directory(), base.join("sessions"));
    }

    #[test]
    fn test_environment_override() {
        let custom_path = "/tmp/custom-riptide-output";
        std::env::set_var("RIPTIDE_OUTPUT_DIR", custom_path);

        let output_dir = get_output_directory();
        assert_eq!(output_dir, PathBuf::from(custom_path));

        // Cleanup
        std::env::remove_var("RIPTIDE_OUTPUT_DIR");
    }

    #[test]
    fn test_directory_creation() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let test_dir = temp.path().join("test-output");

        assert!(!test_dir.exists());
        ensure_directory_exists(&test_dir).unwrap();
        assert!(test_dir.exists());
    }

    #[test]
    fn test_env_helpers() {
        // Test that helpers don't panic
        let _ = env::output_dir();
        let _ = env::cache_dir();
        let _ = env::logs_dir();
        let _ = env::api_host();
        let _ = env::api_port();
        let _ = env::log_level();
        let _ = env::is_development_mode();
    }
}
