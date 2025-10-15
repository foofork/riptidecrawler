use crate::commands::StealthCommands;
use crate::output;
use anyhow::{Context, Result};
use riptide_stealth::{StealthConfig, StealthController, StealthPreset};
use serde_json;
use std::fs;

pub async fn execute(command: StealthCommands) -> Result<()> {
    match command {
        StealthCommands::Configure {
            preset,
            ua_file,
            fingerprint_evasion,
            output: output_path,
        } => configure(preset, ua_file, fingerprint_evasion, output_path).await,
        StealthCommands::Test {
            url,
            preset,
            verbose,
        } => test_stealth(url, preset, verbose).await,
        StealthCommands::Info => show_info().await,
        StealthCommands::Generate { level, output } => generate_js(level, output).await,
    }
}

async fn configure(
    preset: String,
    ua_file: Option<String>,
    fingerprint_evasion: bool,
    output_path: Option<String>,
) -> Result<()> {
    output::print_info(&format!("Configuring stealth with preset: {}", preset));

    let stealth_preset = match preset.as_str() {
        "none" => StealthPreset::None,
        "low" => StealthPreset::Low,
        "medium" => StealthPreset::Medium,
        "high" => StealthPreset::High,
        _ => {
            output::print_error("Invalid preset. Use: none, low, medium, or high");
            return Ok(());
        }
    };

    let mut config = StealthConfig::from_preset(stealth_preset);

    // Apply custom settings
    if let Some(ua_file_path) = ua_file {
        config.ua_file_path = Some(ua_file_path.clone());
        output::print_info(&format!("Using user agent file: {}", ua_file_path));
    }

    if fingerprint_evasion {
        config
            .fingerprinting
            .cdp_stealth
            .disable_automation_controlled = true;
        config.fingerprinting.cdp_stealth.override_webdriver = true;
        config.fingerprinting.cdp_stealth.override_permissions = true;
        config.fingerprinting.cdp_stealth.override_plugins = true;
        config.fingerprinting.cdp_stealth.override_chrome = true;
        output::print_info("Fingerprint evasion enabled");
    }

    // Display configuration summary
    output::print_section("Stealth Configuration");
    output::print_key_value("Preset", &format!("{:?}", config.preset));
    output::print_key_value(
        "User Agent Strategy",
        &format!("{:?}", config.user_agent.strategy),
    );
    output::print_key_value(
        "Fingerprinting Enabled",
        &config
            .fingerprinting
            .cdp_stealth
            .disable_automation_controlled
            .to_string(),
    );
    output::print_key_value(
        "Timing Jitter",
        &format!(
            "{}%",
            config.request_randomization.timing_jitter.jitter_percentage * 100.0
        ),
    );

    // Save configuration if output path provided
    if let Some(path) = output_path {
        let json = serde_json::to_string_pretty(&config)
            .context("Failed to serialize stealth configuration")?;
        fs::write(&path, json).context("Failed to write configuration file")?;
        output::print_success(&format!("Configuration saved to: {}", path));
    }

    Ok(())
}

async fn test_stealth(url: String, preset: String, verbose: bool) -> Result<()> {
    output::print_info(&format!("Testing stealth configuration against: {}", url));

    let stealth_preset = match preset.as_str() {
        "none" => StealthPreset::None,
        "low" => StealthPreset::Low,
        "medium" => StealthPreset::Medium,
        "high" => StealthPreset::High,
        _ => StealthPreset::Medium,
    };

    let mut controller = StealthController::from_preset(stealth_preset);

    // Test user agent rotation
    output::print_section("User Agent Test");
    let ua = controller.next_user_agent();
    output::print_key_value("Current User Agent", ua);

    if verbose {
        output::print_info("Testing 5 user agent rotations:");
        for i in 1..=5 {
            let next_ua = controller.next_user_agent();
            println!("  {}: {}", i, next_ua);
        }
    }

    // Test headers generation
    output::print_section("Headers Test");
    let headers = controller.generate_headers();
    if verbose {
        output::print_info("Generated headers:");
        for (key, value) in &headers {
            println!("  {}: {}", key, value);
        }
    } else {
        output::print_key_value("Headers Count", &headers.len().to_string());
    }

    // Test timing randomization
    output::print_section("Timing Test");
    let delay = controller.calculate_delay();
    output::print_key_value("Request Delay", &format!("{:.2}s", delay.as_secs_f64()));

    if verbose {
        output::print_info("Testing 5 delay calculations:");
        for i in 1..=5 {
            let next_delay = controller.calculate_delay();
            println!("  {}: {:.2}s", i, next_delay.as_secs_f64());
        }
    }

    // Test CDP flags
    output::print_section("Browser Configuration");
    let cdp_flags = controller.get_cdp_flags();
    if verbose {
        output::print_info("CDP flags:");
        for flag in &cdp_flags {
            println!("  {}", flag);
        }
    } else {
        output::print_key_value("CDP Flags Count", &cdp_flags.len().to_string());
    }

    // Attempt to make a test request
    output::print_section("HTTP Test");
    match make_stealth_request(&url, &mut controller).await {
        Ok(status) => {
            output::print_success(&format!("Request successful: HTTP {}", status));
        }
        Err(e) => {
            output::print_warning(&format!("Request failed: {}", e));
        }
    }

    output::print_success("Stealth test completed");
    Ok(())
}

async fn show_info() -> Result<()> {
    output::print_section("RipTide Stealth Information");

    println!("Available Stealth Presets:");
    println!("  • none   - No stealth measures (fastest)");
    println!("  • low    - Basic stealth with minimal overhead");
    println!("  • medium - Balanced stealth and performance (default)");
    println!("  • high   - Maximum stealth, all countermeasures enabled");
    println!();

    println!("Stealth Features:");
    println!("  • User Agent Rotation (Random, Sequential, Sticky, Domain-based)");
    println!("  • Browser Fingerprinting Countermeasures");
    println!("  • Request Timing Randomization");
    println!("  • Header Randomization and Consistency");
    println!("  • JavaScript API Overrides");
    println!("  • Behavior Simulation (mouse, scroll)");
    println!("  • Proxy Support with Rotation");
    println!();

    println!("Usage Examples:");
    println!("  # Extract with high stealth");
    println!("  riptide extract --url https://example.com --stealth-level high");
    println!();
    println!("  # Configure and save stealth settings");
    println!("  riptide stealth configure --preset high --output stealth.json");
    println!();
    println!("  # Test stealth configuration");
    println!("  riptide stealth test --url https://example.com --preset high --verbose");
    println!();
    println!("  # Generate JavaScript injection code");
    println!("  riptide stealth generate --level high --output stealth.js");

    Ok(())
}

async fn generate_js(level: String, output: Option<String>) -> Result<()> {
    output::print_info(&format!(
        "Generating stealth JavaScript for level: {}",
        level
    ));

    let stealth_preset = match level.as_str() {
        "none" => StealthPreset::None,
        "low" => StealthPreset::Low,
        "medium" => StealthPreset::Medium,
        "high" => StealthPreset::High,
        _ => StealthPreset::Medium,
    };

    let mut controller = StealthController::from_preset(stealth_preset);
    let js_code = controller.get_stealth_js();

    match output {
        Some(path) => {
            fs::write(&path, js_code).context("Failed to write JavaScript file")?;
            output::print_success(&format!("JavaScript code saved to: {}", path));
        }
        None => {
            output::print_section("Generated JavaScript Code");
            println!("{}", js_code);
        }
    }

    Ok(())
}

async fn make_stealth_request(url: &str, controller: &mut StealthController) -> Result<u16> {
    use std::time::Duration;

    let headers = controller.generate_headers();
    let user_agent = controller.next_user_agent();

    let mut client_builder = reqwest::Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(30));

    // Add custom headers
    let mut header_map = reqwest::header::HeaderMap::new();
    for (key, value) in headers {
        if let (Ok(header_name), Ok(header_value)) = (
            reqwest::header::HeaderName::from_bytes(key.as_bytes()),
            reqwest::header::HeaderValue::from_str(&value),
        ) {
            header_map.insert(header_name, header_value);
        }
    }
    client_builder = client_builder.default_headers(header_map);

    let client = client_builder.build()?;

    // Apply timing delay
    let delay = controller.calculate_delay();
    tokio::time::sleep(delay).await;

    let response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}
