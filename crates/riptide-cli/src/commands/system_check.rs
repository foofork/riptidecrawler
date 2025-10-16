use crate::client::RipTideClient;
use crate::output;
use crate::validation::{self, CheckStatus};
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct SystemCheckArgs {
    /// Run production-readiness checks (stricter)
    #[arg(long)]
    pub production: bool,

    /// Run performance baseline profiling
    #[arg(long)]
    pub profile: bool,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Skip specific checks (comma-separated: api,redis,wasm,browser,network,resources)
    #[arg(long)]
    pub skip: Option<String>,
}

pub async fn execute(client: RipTideClient, args: SystemCheckArgs) -> Result<()> {
    if args.profile {
        execute_profile(&client, &args).await
    } else if args.production {
        execute_production(&client, &args).await
    } else {
        execute_standard(&client, &args).await
    }
}

async fn execute_profile(client: &RipTideClient, args: &SystemCheckArgs) -> Result<()> {
    if args.format != "json" {
        output::print_section("Performance Baseline Profile");
        println!();
    }

    output::print_info("Running performance baseline tests...");

    let baseline = validation::run_performance_baseline(client).await?;

    if args.format == "json" {
        output::print_json(&baseline);
    } else {
        println!();
        output::print_section("Performance Baseline Results");

        if let Some(api_latency) = baseline["api_latency_ms"].as_u64() {
            output::print_key_value("API Latency", &format!("{}ms", api_latency));

            if api_latency < 50 {
                output::print_success("Excellent API response time");
            } else if api_latency < 200 {
                output::print_success("Good API response time");
            } else if api_latency < 500 {
                output::print_warning("Acceptable API response time");
            } else {
                output::print_error("Poor API response time");
            }
        }

        if let Some(timestamp) = baseline["timestamp"].as_str() {
            output::print_key_value("Timestamp", timestamp);
        }

        println!();
        output::print_info("Baseline profile complete");
    }

    Ok(())
}

async fn execute_production(client: &RipTideClient, args: &SystemCheckArgs) -> Result<()> {
    if args.format != "json" {
        output::print_section("Production Readiness Check");
        println!();
        output::print_info("Running strict production validation...");
        println!();
    }

    let report = validation::run_production_checks(client).await;

    if args.format == "json" {
        output::print_json(&report);
    } else {
        print_system_check_report(&report, true);
    }

    std::process::exit(report.exit_code());
}

async fn execute_standard(client: &RipTideClient, args: &SystemCheckArgs) -> Result<()> {
    if args.format != "json" {
        output::print_section("System Health Check");
        println!();
        output::print_info("Performing comprehensive system check...");
        println!();
    }

    let report = validation::run_comprehensive_validation(client, None).await;

    if args.format == "json" {
        output::print_json(&report);
    } else {
        print_system_check_report(&report, false);
    }

    std::process::exit(report.exit_code());
}

fn print_system_check_report(report: &validation::ValidationReport, production_mode: bool) {
    // Group checks by category
    let mut categories: std::collections::HashMap<&str, Vec<&validation::CheckResult>> =
        std::collections::HashMap::new();

    for check in &report.checks {
        let category = match check.name.as_str() {
            "API Connectivity" => "Core Services",
            "Redis" => "Core Services",
            "WASM Module" => "Extraction Engine",
            "Headless Browser" => "Extraction Engine",
            "Network Connectivity" => "Infrastructure",
            "System Resources" => "Infrastructure",
            "Filesystem Permissions" => "Infrastructure",
            "Configuration" => "Configuration",
            "Dependencies" => "Configuration",
            _ => "Other",
        };

        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(check);
    }

    // Print checks grouped by category
    let category_order = vec![
        "Core Services",
        "Extraction Engine",
        "Infrastructure",
        "Configuration",
        "Other",
    ];

    for category in category_order {
        if let Some(checks) = categories.get(category) {
            output::print_section(category);
            for check in checks {
                print_check_result(check, production_mode);
            }
            println!();
        }
    }

    // Print summary
    output::print_section("System Check Summary");

    use colored::Colorize;
    let total_line = format!(
        "Total Checks: {} | Passed: {} | Failed: {} | Warnings: {}",
        report.summary.total_checks,
        report.summary.passed.to_string().green(),
        report.summary.failed.to_string().red(),
        report.summary.warnings.to_string().yellow()
    );
    println!("{}", total_line);

    println!();

    match report.summary.overall_status {
        CheckStatus::Pass => {
            if production_mode {
                output::print_success("System is PRODUCTION READY ✓");
            } else {
                output::print_success("All critical checks passed - System is healthy!");
            }
        }
        CheckStatus::Warning => {
            output::print_warning("System operational but has warnings");
            if production_mode {
                output::print_error("NOT RECOMMENDED for production deployment");
            }
        }
        CheckStatus::Fail => {
            output::print_error("System check FAILED - Critical issues detected");
            if production_mode {
                output::print_error("System is NOT PRODUCTION READY");
            } else {
                output::print_error("System requires attention before use");
            }
        }
        CheckStatus::Skipped => {
            output::print_info("System check completed (some checks skipped)");
        }
    }

    if report.summary.failed > 0 {
        println!();
        output::print_section("Failed Checks - Action Required");
        for check in &report.checks {
            if matches!(check.status, CheckStatus::Fail) {
                if let Some(remediation) = &check.remediation {
                    println!("• {}: {}", check.name.bold(), check.message);
                    println!("  {}", "Fix:".yellow().bold());
                    for line in remediation.lines() {
                        println!("    {}", line);
                    }
                    println!();
                }
            }
        }
    }
}

fn print_check_result(result: &validation::CheckResult, production_mode: bool) {
    let (icon, color) = match result.status {
        CheckStatus::Pass => ("✓", colored::Color::Green),
        CheckStatus::Fail => ("✗", colored::Color::Red),
        CheckStatus::Warning => {
            if production_mode {
                ("✗", colored::Color::Red) // Treat warnings as failures in production mode
            } else {
                ("⚠", colored::Color::Yellow)
            }
        }
        CheckStatus::Skipped => ("○", colored::Color::Blue),
    };

    use colored::Colorize;
    print!("  {} {}: ", icon.color(color).bold(), result.name.dimmed());

    match result.status {
        CheckStatus::Pass => println!("{}", result.message.green()),
        CheckStatus::Fail => println!("{}", result.message.red()),
        CheckStatus::Warning => {
            if production_mode {
                println!("{}", result.message.red());
            } else {
                println!("{}", result.message.yellow());
            }
        }
        CheckStatus::Skipped => println!("{}", result.message.blue()),
    }
}
