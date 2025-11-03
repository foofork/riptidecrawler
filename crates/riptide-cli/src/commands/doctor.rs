/// Doctor command - System diagnostics and health checks
///
/// This command performs comprehensive health checks on all RipTide components:
/// - API Server connectivity
/// - Redis cache availability
/// - WASM extractor functionality
/// - HTTP client health
/// - Headless browser pool (if configured)
/// - Spider engine (if configured)
/// - Worker service status
///
/// Provides actionable remediation steps for any failing components.
use crate::client::ApiClient;
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::process;

#[derive(Args, Clone, Debug)]
pub struct DoctorArgs {
    /// Full diagnostic report with system metrics
    #[arg(long)]
    pub full: bool,

    /// Output detailed JSON diagnostics
    #[arg(long)]
    pub json: bool,
}

/// Health response structure matching API server format
#[derive(Serialize, Deserialize, Debug)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
    uptime: u64,
    dependencies: DependencyStatus,
    metrics: Option<SystemMetrics>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DependencyStatus {
    redis: ServiceHealth,
    extractor: ServiceHealth,
    http_client: ServiceHealth,
    headless_service: Option<ServiceHealth>,
    spider_engine: Option<ServiceHealth>,
    worker_service: Option<ServiceHealth>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceHealth {
    status: String,
    message: Option<String>,
    response_time_ms: Option<u64>,
    last_check: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemMetrics {
    memory_usage_bytes: u64,
    active_connections: u32,
    total_requests: u64,
    requests_per_second: f64,
    avg_response_time_ms: f64,
    cpu_usage_percent: Option<f32>,
    disk_usage_bytes: Option<u64>,
    file_descriptor_count: Option<u32>,
    thread_count: Option<u32>,
    load_average: Option<[f32; 3]>,
}

pub async fn execute(client: ApiClient, args: DoctorArgs, _output_format: String) -> Result<()> {
    // Fetch health check from API server
    let response = client
        .get("/healthz")
        .await
        .context("Failed to connect to RipTide API server")?;

    let status_code = response.status();
    let health: HealthResponse = response
        .json()
        .await
        .context("Failed to parse health check response")?;

    // JSON output mode
    if args.json {
        println!("{}", serde_json::to_string_pretty(&health)?);
        if !is_healthy(&health) {
            process::exit(1);
        }
        return Ok(());
    }

    // Human-readable output
    print_diagnostics(&health, args.full, status_code.as_u16());

    // Exit with error if critical components are down
    if !is_healthy(&health) {
        process::exit(1);
    }

    Ok(())
}

/// Check if system is healthy overall
fn is_healthy(health: &HealthResponse) -> bool {
    // Critical components: API server, Redis, Extractor, HTTP client
    health.status == "healthy" || health.status == "degraded"
}

/// Print human-readable diagnostics
fn print_diagnostics(health: &HealthResponse, full: bool, status_code: u16) {
    println!("\n{}", "RipTide System Diagnostics".bold().cyan());
    println!("{}", "═".repeat(50).cyan());
    println!();

    // API Server Status
    let api_status = if status_code == 200 {
        format!("{}", "✓ OK".green())
    } else {
        format!("{}", "✗ FAIL".red())
    };
    println!("{:<30} {}", "API Server".bold(), api_status);
    println!("{:<30} {}", "  Version", health.version);
    println!("{:<30} {}s", "  Uptime", health.uptime);

    println!();

    // Core Dependencies
    print_component_status("Redis", &health.dependencies.redis);
    print_component_status("WASM Extractor", &health.dependencies.extractor);
    print_component_status("HTTP Client", &health.dependencies.http_client);

    // Optional Dependencies
    if let Some(ref headless) = health.dependencies.headless_service {
        print_component_status("Headless Pool", headless);
    }

    if let Some(ref spider) = health.dependencies.spider_engine {
        print_component_status("Spider Engine", spider);
    }

    if let Some(ref worker) = health.dependencies.worker_service {
        print_component_status("Worker Service", worker);
    }

    println!();

    // System Metrics (if --full flag is used)
    if full {
        if let Some(ref metrics) = health.metrics {
            print_system_metrics(metrics);
        }
    }

    // Overall Status
    let overall_status = match health.status.as_str() {
        "healthy" => format!("{}", "✓ HEALTHY".green().bold()),
        "degraded" => format!("{}", "⚠ DEGRADED".yellow().bold()),
        _ => format!("{}", "✗ UNHEALTHY".red().bold()),
    };
    println!("{}", "─".repeat(50).cyan());
    println!("{:<30} {}", "Overall Status".bold(), overall_status);
    println!();

    // Remediation steps if unhealthy
    if health.status != "healthy" {
        print_remediation(&health.dependencies);
    }
}

/// Print individual component status
fn print_component_status(name: &str, service: &ServiceHealth) {
    let status_str = match service.status.as_str() {
        "healthy" => format!("{}", "✓ OK".green()),
        "unhealthy" => format!("{}", "✗ FAIL".red()),
        "unknown" => format!("{}", "? UNKNOWN".yellow()),
        _ => format!("{}", service.status.yellow()),
    };

    println!("{:<30} {}", name.bold(), status_str);

    if let Some(ref msg) = service.message {
        if service.status != "healthy" {
            println!("{:<30}   {}", "", msg.dimmed());
        }
    }

    if let Some(response_time) = service.response_time_ms {
        println!("{:<30}   Response: {}ms", "", response_time);
    }
}

/// Print system metrics
fn print_system_metrics(metrics: &SystemMetrics) {
    println!("{}", "System Metrics".bold().cyan());
    println!("{}", "─".repeat(50).cyan());

    // Memory
    let mem_mb = metrics.memory_usage_bytes as f64 / (1024.0 * 1024.0);
    println!("{:<30} {:.1} MB", "Memory Usage".bold(), mem_mb);

    // CPU
    if let Some(cpu) = metrics.cpu_usage_percent {
        println!("{:<30} {:.1}%", "CPU Usage".bold(), cpu);
    }

    // Load Average
    if let Some(load) = metrics.load_average {
        println!(
            "{:<30} {:.2}, {:.2}, {:.2}",
            "Load Average (1/5/15min)".bold(),
            load[0],
            load[1],
            load[2]
        );
    }

    // Connections & Requests
    println!(
        "{:<30} {}",
        "Active Connections".bold(),
        metrics.active_connections
    );
    println!("{:<30} {}", "Total Requests".bold(), metrics.total_requests);
    println!(
        "{:<30} {:.2}",
        "Requests/sec".bold(),
        metrics.requests_per_second
    );
    println!(
        "{:<30} {:.2}ms",
        "Avg Response Time".bold(),
        metrics.avg_response_time_ms
    );

    // File descriptors
    if let Some(fds) = metrics.file_descriptor_count {
        println!("{:<30} {}", "File Descriptors".bold(), fds);
    }

    // Threads
    if let Some(threads) = metrics.thread_count {
        println!("{:<30} {}", "Threads".bold(), threads);
    }

    // Disk
    if let Some(disk) = metrics.disk_usage_bytes {
        let disk_mb = disk as f64 / (1024.0 * 1024.0);
        println!("{:<30} {:.1} MB", "Disk Usage".bold(), disk_mb);
    }

    println!();
}

/// Print remediation steps for failed components
fn print_remediation(deps: &DependencyStatus) {
    let mut has_failures = false;
    let mut remediation_steps = Vec::new();

    // Check each component and collect remediation steps
    if deps.redis.status != "healthy" {
        has_failures = true;
        remediation_steps.push((
            "Redis",
            vec![
                "Check Redis service: systemctl status redis",
                "Verify Redis connection in config: cat ~/.riptide/config.yml",
                "Test Redis connectivity: redis-cli ping",
                "Restart Redis: systemctl restart redis",
            ],
        ));
    }

    if deps.extractor.status != "healthy" {
        has_failures = true;
        remediation_steps.push((
            "WASM Extractor",
            vec![
                "Check WASM module installation",
                "Verify extractor configuration",
                "Review API server logs: journalctl -u riptide-api -n 50",
            ],
        ));
    }

    if deps.http_client.status != "healthy" {
        has_failures = true;
        remediation_steps.push((
            "HTTP Client",
            vec![
                "Check network connectivity",
                "Verify proxy settings if applicable",
                "Review firewall rules",
            ],
        ));
    }

    if let Some(ref headless) = deps.headless_service {
        if headless.status != "healthy" {
            has_failures = true;
            remediation_steps.push((
                "Headless Pool",
                vec![
                    "Check headless service: systemctl status riptide-headless",
                    "Verify headless URL in config",
                    "Restart pool: riptide pool restart",
                    "Check browser availability: which chromium || which chrome",
                ],
            ));
        }
    }

    if let Some(ref spider) = deps.spider_engine {
        if spider.status != "healthy" {
            has_failures = true;
            remediation_steps.push((
                "Spider Engine",
                vec![
                    "Check spider service configuration",
                    "Verify spider engine dependencies",
                    "Review spider logs for errors",
                ],
            ));
        }
    }

    if let Some(ref worker) = deps.worker_service {
        if worker.status != "healthy" {
            has_failures = true;
            remediation_steps.push((
                "Worker Service",
                vec![
                    "Check worker service: systemctl status riptide-worker",
                    "Verify worker queue configuration",
                    "Review worker logs: journalctl -u riptide-worker -n 50",
                    "Restart worker: systemctl restart riptide-worker",
                ],
            ));
        }
    }

    if has_failures {
        println!("{}", "💡 Remediation".bold().yellow());
        println!("{}", "─".repeat(50).yellow());
        println!();

        for (i, (component, steps)) in remediation_steps.iter().enumerate() {
            println!("{}. {} Issues:", i + 1, component.bold());
            for (j, step) in steps.iter().enumerate() {
                println!("   {}) {}", ('a' as u8 + j as u8) as char, step);
            }
            println!();
        }

        println!("For more help: {}", "riptide doctor --full".cyan());
    }
}
