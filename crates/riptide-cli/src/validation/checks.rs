use super::types::*;
use crate::client::RipTideClient;
use anyhow::Result;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

/// Check WASM module availability and initialization
pub async fn check_wasm(wasm_path: Option<&str>) -> CheckResult {
    // First, try to find the WASM module
    let wasm_module_path = if let Some(path) = wasm_path {
        PathBuf::from(path)
    } else if let Ok(path) = std::env::var("RIPTIDE_WASM_PATH") {
        PathBuf::from(path)
    } else {
        // Check common locations
        let common_paths = [
            "wasm/riptide-extractor-wasm/pkg/riptide_extractor_wasm_bg.wasm",
            "../wasm/riptide-extractor-wasm/pkg/riptide_extractor_wasm_bg.wasm",
            "./riptide_extractor_wasm_bg.wasm",
        ];

        match common_paths.iter().find(|p| PathBuf::from(p).exists()) {
            Some(path) => PathBuf::from(path),
            None => {
                return CheckResult::fail(
                    "WASM Module",
                    "WASM module not found",
                    "Set RIPTIDE_WASM_PATH environment variable or build WASM module with:\n\
                     cd wasm/riptide-extractor-wasm && wasm-pack build --target web",
                );
            }
        }
    };

    if !wasm_module_path.exists() {
        return CheckResult::fail(
            "WASM Module",
            format!("WASM module not found at: {}", wasm_module_path.display()),
            "Build the WASM module with:\n\
             cd wasm/riptide-extractor-wasm && wasm-pack build --target web",
        );
    }

    // Check if the file is readable
    match std::fs::metadata(&wasm_module_path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size == 0 {
                return CheckResult::fail(
                    "WASM Module",
                    "WASM module file is empty",
                    "Rebuild the WASM module",
                );
            }

            CheckResult::pass(
                "WASM Module",
                format!(
                    "WASM module available at {} ({} bytes)",
                    wasm_module_path.display(),
                    size
                ),
            )
            .with_details(json!({
                "path": wasm_module_path.display().to_string(),
                "size_bytes": size,
            }))
        }
        Err(e) => CheckResult::fail(
            "WASM Module",
            format!("Cannot access WASM module: {}", e),
            "Check file permissions or rebuild the WASM module",
        ),
    }
}

/// Check headless browser availability
pub async fn check_headless_browser() -> CheckResult {
    // Check if Chrome/Chromium is available
    let browsers = vec!["google-chrome", "chromium", "chromium-browser", "chrome"];

    for browser in browsers {
        if let Ok(output) = Command::new(browser).arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return CheckResult::pass(
                    "Headless Browser",
                    format!("Browser available: {}", version),
                )
                .with_details(json!({
                    "browser": browser,
                    "version": version,
                }));
            }
        }
    }

    CheckResult::fail(
        "Headless Browser",
        "No compatible browser found",
        "Install Chrome or Chromium:\n\
         Ubuntu/Debian: sudo apt-get install chromium-browser\n\
         macOS: brew install chromium\n\
         Or set CHROME_PATH environment variable to custom location",
    )
}

/// Check Redis connectivity
pub async fn check_redis(client: &RipTideClient) -> CheckResult {
    match client.get("/api/health/detailed").await {
        Ok(response) => {
            if let Ok(health) = response.json::<serde_json::Value>().await {
                let redis_status = health["redis"].as_str().unwrap_or("unknown");

                if redis_status == "connected" {
                    CheckResult::pass("Redis", "Redis connection established")
                } else if redis_status == "not_configured" {
                    CheckResult::warning(
                        "Redis",
                        "Redis not configured (optional for local operation)",
                    )
                } else {
                    CheckResult::fail(
                        "Redis",
                        format!("Redis status: {}", redis_status),
                        "Check Redis configuration and ensure Redis server is running:\n\
                         docker run -d -p 6379:6379 redis:latest\n\
                         or install locally: sudo apt-get install redis-server",
                    )
                }
            } else {
                CheckResult::warning("Redis", "Could not determine Redis status")
            }
        }
        Err(e) => CheckResult::fail(
            "Redis",
            format!("Health check failed: {}", e),
            "Ensure the RipTide API server is running and accessible",
        ),
    }
}

/// Check API connectivity
pub async fn check_api_connectivity(client: &RipTideClient) -> CheckResult {
    match client.get("/healthz").await {
        Ok(_) => CheckResult::pass("API Connectivity", "API server is reachable"),
        Err(e) => CheckResult::fail(
            "API Connectivity",
            format!("Cannot reach API server: {}", e),
            "Ensure RipTide API server is running:\n\
             cargo run --bin riptide-api\n\
             or check RIPTIDE_API_URL environment variable",
        ),
    }
}

/// Check file system permissions
pub async fn check_filesystem_permissions() -> CheckResult {
    // Check cache directory
    let cache_dir = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("riptide")
    } else {
        PathBuf::from("/tmp/riptide-cache")
    };

    // Try to create the directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        return CheckResult::fail(
            "Filesystem Permissions",
            format!("Cannot create cache directory: {}", e),
            format!(
                "Ensure write permissions for: {}\n\
                 Or set RIPTIDE_CACHE_DIR to a writable location",
                cache_dir.display()
            ),
        );
    }

    // Try to write a test file
    let test_file = cache_dir.join(".test-write");
    match std::fs::write(&test_file, b"test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            CheckResult::pass(
                "Filesystem Permissions",
                format!("Cache directory writable: {}", cache_dir.display()),
            )
        }
        Err(e) => CheckResult::fail(
            "Filesystem Permissions",
            format!("Cannot write to cache directory: {}", e),
            format!("Check permissions for: {}", cache_dir.display()),
        ),
    }
}

/// Check network connectivity
pub async fn check_network() -> CheckResult {
    let test_urls = vec![
        "https://www.google.com",
        "https://www.cloudflare.com",
        "https://httpbin.org",
    ];

    for url in test_urls {
        if let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            if let Ok(_) = client.head(url).send().await {
                return CheckResult::pass("Network Connectivity", "Internet connection available");
            }
        }
    }

    CheckResult::fail(
        "Network Connectivity",
        "Cannot establish internet connection",
        "Check network connection and firewall settings\n\
         Ensure outbound HTTPS (443) is allowed",
    )
}

/// Check CPU and system resources
pub async fn check_system_resources() -> CheckResult {
    // Get CPU info
    let cpu_count = num_cpus::get();

    // Get available memory (basic check)
    let available_memory_mb = if let Ok(output) = Command::new("free").arg("-m").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .nth(1)
            .and_then(|line| {
                line.split_whitespace()
                    .nth(6)
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .unwrap_or(0)
    } else {
        0
    };

    let mut warnings = Vec::new();

    if cpu_count < 2 {
        warnings.push("Low CPU count (recommended: 2+ cores)".to_string());
    }

    if available_memory_mb < 512 {
        warnings.push("Low available memory (recommended: 512MB+)".to_string());
    }

    let message = if warnings.is_empty() {
        format!(
            "System resources adequate ({} CPUs, {}MB available)",
            cpu_count, available_memory_mb
        )
    } else {
        format!(
            "{} CPUs, {}MB available. Warnings: {}",
            cpu_count,
            available_memory_mb,
            warnings.join(", ")
        )
    };

    let result = if warnings.is_empty() {
        CheckResult::pass("System Resources", message)
    } else {
        CheckResult::warning("System Resources", message)
    };

    result.with_details(json!({
        "cpu_count": cpu_count,
        "available_memory_mb": available_memory_mb,
        "warnings": warnings,
    }))
}

/// Check configuration validity
pub async fn check_configuration() -> CheckResult {
    let mut config_issues = Vec::new();

    // Check critical environment variables
    if std::env::var("RIPTIDE_API_URL").is_err() {
        config_issues.push("RIPTIDE_API_URL not set (using default)");
    }

    if std::env::var("RUST_LOG").is_err() {
        config_issues.push("RUST_LOG not set (using default log level)");
    }

    if config_issues.is_empty() {
        CheckResult::pass("Configuration", "Configuration is valid")
    } else {
        CheckResult::warning(
            "Configuration",
            format!("Minor configuration issues: {}", config_issues.join(", ")),
        )
    }
}

/// Check dependencies
pub async fn check_dependencies() -> CheckResult {
    let mut missing_deps = Vec::new();

    // Check for wasm-pack (for WASM builds)
    if Command::new("wasm-pack").arg("--version").output().is_err() {
        missing_deps.push("wasm-pack (optional, for WASM development)");
    }

    if missing_deps.is_empty() {
        CheckResult::pass("Dependencies", "All required dependencies available")
    } else {
        CheckResult::warning(
            "Dependencies",
            format!("Optional dependencies missing: {}", missing_deps.join(", ")),
        )
    }
}

/// Comprehensive validation - runs all checks
pub async fn run_comprehensive_validation(
    client: &RipTideClient,
    wasm_path: Option<&str>,
) -> ValidationReport {
    let mut checks = Vec::new();

    // Core checks (always run)
    checks.push(check_api_connectivity(client).await);
    checks.push(check_wasm(wasm_path).await);
    checks.push(check_filesystem_permissions().await);
    checks.push(check_configuration().await);

    // Optional checks (warn if fail)
    checks.push(check_redis(client).await);
    checks.push(check_headless_browser().await);
    checks.push(check_network().await);
    checks.push(check_system_resources().await);
    checks.push(check_dependencies().await);

    ValidationReport::new(checks)
}

/// Production-specific checks
pub async fn run_production_checks(client: &RipTideClient) -> ValidationReport {
    let mut checks = Vec::new();

    // All production checks must pass
    checks.push(check_api_connectivity(client).await);
    checks.push(check_redis(client).await);
    checks.push(check_wasm(None).await);
    checks.push(check_headless_browser().await);
    checks.push(check_network().await);
    checks.push(check_system_resources().await);
    checks.push(check_filesystem_permissions().await);

    ValidationReport::new(checks)
}

/// Performance baseline profiling
pub async fn run_performance_baseline(client: &RipTideClient) -> Result<serde_json::Value> {
    let start = std::time::Instant::now();

    // Simple API response time test
    let api_latency = if let Ok(_) = client.get("/healthz").await {
        start.elapsed().as_millis()
    } else {
        0
    };

    // WASM load time (basic check)
    let wasm_load_time = 0; // Would need actual WASM initialization

    Ok(json!({
        "api_latency_ms": api_latency,
        "wasm_load_time_ms": wasm_load_time,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
