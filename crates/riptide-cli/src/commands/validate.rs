use crate::client::RipTideClient;
use crate::output;
use crate::validation::{self, CheckStatus};
use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// Run comprehensive validation with all checks
    #[arg(long)]
    pub comprehensive: bool,

    /// Check WASM setup only
    #[arg(long)]
    pub wasm: bool,

    /// Output format (text or json)
    #[arg(long, default_value = "text")]
    pub format: String,

    /// Path to WASM module (overrides environment)
    #[arg(long, env = "RIPTIDE_WASM_PATH")]
    pub wasm_path: Option<String>,

    /// Continue on failures (don't exit on first failure)
    #[arg(long)]
    pub continue_on_failure: bool,
}

pub async fn execute(client: RipTideClient, args: ValidateArgs) -> Result<()> {
    if args.wasm {
        // WASM-only check
        execute_wasm_check(&args).await
    } else if args.comprehensive {
        // Comprehensive validation
        execute_comprehensive(&client, &args).await
    } else {
        // Basic validation (backward compatible)
        execute_basic(&client, &args).await
    }
}

async fn execute_wasm_check(args: &ValidateArgs) -> Result<()> {
    output::print_info("Checking WASM setup...");

    let result = validation::check_wasm(args.wasm_path.as_deref()).await;

    if args.format == "json" {
        output::print_json(&serde_json::json!({
            "check": "wasm",
            "status": match result.status {
                CheckStatus::Pass => "pass",
                CheckStatus::Fail => "fail",
                CheckStatus::Warning => "warning",
                CheckStatus::Skipped => "skipped",
            },
            "message": result.message,
            "remediation": result.remediation,
            "details": result.details,
        }));
    } else {
        print_check_result(&result);
    }

    match result.status {
        CheckStatus::Pass => {
            std::process::exit(0);
        }
        CheckStatus::Warning => {
            std::process::exit(0);
        }
        CheckStatus::Fail => {
            std::process::exit(1);
        }
        CheckStatus::Skipped => {
            std::process::exit(0);
        }
    }
}

async fn execute_comprehensive(client: &RipTideClient, args: &ValidateArgs) -> Result<()> {
    if args.format != "json" {
        output::print_section("Comprehensive System Validation");
        println!();
    }

    let report = validation::run_comprehensive_validation(client, args.wasm_path.as_deref()).await;

    if args.format == "json" {
        output::print_json(&report);
    } else {
        print_validation_report(&report);
    }

    std::process::exit(report.exit_code());
}

async fn execute_basic(client: &RipTideClient, args: &ValidateArgs) -> Result<()> {
    output::print_info("Validating system configuration...");
    println!();

    let mut checks = Vec::new();

    // Run basic checks
    checks.push(validation::check_api_connectivity(client).await);
    checks.push(validation::check_wasm(args.wasm_path.as_deref()).await);
    checks.push(validation::check_redis(client).await);
    checks.push(validation::check_configuration().await);

    let report = validation::ValidationReport::new(checks);

    if args.format == "json" {
        output::print_json(&report);
    } else {
        print_validation_report(&report);
    }

    std::process::exit(report.exit_code());
}

fn print_check_result(result: &validation::CheckResult) {
    let (icon, color) = match result.status {
        CheckStatus::Pass => ("✓", colored::Color::Green),
        CheckStatus::Fail => ("✗", colored::Color::Red),
        CheckStatus::Warning => ("⚠", colored::Color::Yellow),
        CheckStatus::Skipped => ("○", colored::Color::Blue),
    };

    use colored::Colorize;
    println!(
        "{} {}: {}",
        icon.color(color).bold(),
        result.name.bold(),
        result.message
    );

    if let Some(remediation) = &result.remediation {
        println!("  {} {}", "→".yellow(), remediation.dimmed());
    }

    if let Some(details) = &result.details {
        if let Ok(formatted) = serde_json::to_string_pretty(details) {
            for line in formatted.lines() {
                println!("  {}", line.dimmed());
            }
        }
    }
}

fn print_validation_report(report: &validation::ValidationReport) {
    for check in &report.checks {
        print_check_result(check);
        println!();
    }

    output::print_section("Validation Summary");
    output::print_key_value("Total Checks", &report.summary.total_checks.to_string());
    output::print_key_value("Passed", &report.summary.passed.to_string());
    output::print_key_value("Failed", &report.summary.failed.to_string());
    output::print_key_value("Warnings", &report.summary.warnings.to_string());
    output::print_key_value("Skipped", &report.summary.skipped.to_string());

    println!();

    match report.summary.overall_status {
        CheckStatus::Pass => {
            output::print_success("All validation checks passed!");
        }
        CheckStatus::Warning => {
            output::print_warning("Validation completed with warnings");
        }
        CheckStatus::Fail => {
            output::print_error(&format!(
                "{} validation check(s) failed",
                report.summary.failed
            ));
        }
        CheckStatus::Skipped => {
            output::print_info("Validation completed (some checks skipped)");
        }
    }
}
