use crate::client::RipTideClient;
use crate::output;
use anyhow::{Context, Result};
use chrono::Utc;
use riptide_intelligence::{
    ContentPattern, DomainAnalyzer, DomainConfig, DomainMetadata, DomainProfile, DriftAnalyzer,
    ProfileManager, ProfileRegistry, SiteBaseline, SiteStructure,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(clap::Subcommand)]
pub enum DomainCommands {
    /// Initialize a domain profile for a website
    Init {
        /// Domain URL to initialize profile for
        #[arg(long)]
        domain: String,

        /// Analyze site structure automatically
        #[arg(long)]
        analyze: bool,

        /// Output profile file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Sample URLs to analyze (comma-separated)
        #[arg(long)]
        samples: Option<String>,

        /// Number of pages to crawl for analysis
        #[arg(long, default_value = "10")]
        crawl_depth: u32,

        /// Include metadata analysis
        #[arg(long)]
        metadata: bool,
    },

    /// Set domain-specific extraction configurations
    Profile {
        /// Domain name or profile path
        #[arg(long)]
        domain: String,

        /// Stealth level preset (none, low, medium, high)
        #[arg(long, value_parser = ["none", "low", "medium", "high"])]
        stealth: Option<String>,

        /// Rate limit (requests per second)
        #[arg(long)]
        rate_limit: Option<f64>,

        /// Respect robots.txt
        #[arg(long)]
        robots_txt: bool,

        /// User agent strategy (random, sequential, sticky, domain-based)
        #[arg(long, value_parser = ["random", "sequential", "sticky", "domain-based"])]
        ua_strategy: Option<String>,

        /// Associated schema for extraction
        #[arg(long)]
        schema: Option<String>,

        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long)]
        confidence: Option<f64>,

        /// Enable JavaScript rendering
        #[arg(long)]
        javascript: bool,

        /// Request timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Custom headers (format: key=value, comma-separated)
        #[arg(long)]
        headers: Option<String>,

        /// Proxy URL
        #[arg(long)]
        proxy: Option<String>,

        /// Show profile after configuration
        #[arg(long)]
        show: bool,

        /// Save profile to file
        #[arg(long, short = 's')]
        save: bool,
    },

    /// Detect website structure changes
    Drift {
        /// Domain name or profile path
        #[arg(long)]
        domain: String,

        /// Generate detailed drift report
        #[arg(long)]
        report: bool,

        /// Output report file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Threshold for change significance (0.0-1.0)
        #[arg(long, default_value = "0.1")]
        threshold: f64,

        /// URLs to check for drift (comma-separated)
        #[arg(long)]
        urls: Option<String>,

        /// Alert on significant changes
        #[arg(long)]
        alert: bool,

        /// Compare against specific baseline version
        #[arg(long)]
        baseline: Option<String>,
    },

    /// List all domain profiles
    List {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,

        /// Filter by domain pattern
        #[arg(long)]
        filter: Option<String>,

        /// Output format (table, json, list)
        #[arg(long, default_value = "table")]
        format: String,
    },

    /// Show domain profile details
    Show {
        /// Domain name or profile path
        #[arg(long)]
        domain: String,

        /// Show version history
        #[arg(long)]
        history: bool,

        /// Show baseline structure
        #[arg(long)]
        structure: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Export domain profile
    Export {
        /// Domain name to export
        #[arg(long)]
        domain: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: String,

        /// Include version history
        #[arg(long)]
        history: bool,

        /// Export format (json, yaml)
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Import domain profile
    Import {
        /// Profile file to import
        #[arg(long)]
        file: String,

        /// Override existing profile
        #[arg(long)]
        force: bool,

        /// Validate profile before import
        #[arg(long)]
        validate: bool,
    },

    /// Remove domain profile
    Rm {
        /// Domain name to remove
        #[arg(long)]
        domain: String,

        /// Force removal without confirmation
        #[arg(long, short = 'f')]
        force: bool,

        /// Remove all versions
        #[arg(long)]
        all_versions: bool,
    },
}

pub async fn execute(
    client: RipTideClient,
    command: DomainCommands,
    _output_format: &str,
) -> Result<()> {
    match command {
        DomainCommands::Init {
            domain,
            analyze,
            output,
            samples,
            crawl_depth,
            metadata,
        } => {
            execute_init(
                client,
                domain,
                analyze,
                output,
                samples,
                crawl_depth,
                metadata,
            )
            .await
        }

        DomainCommands::Profile {
            domain,
            stealth,
            rate_limit,
            robots_txt,
            ua_strategy,
            schema,
            confidence,
            javascript,
            timeout,
            headers,
            proxy,
            show,
            save,
        } => {
            execute_profile(
                domain,
                stealth,
                rate_limit,
                robots_txt,
                ua_strategy,
                schema,
                confidence,
                javascript,
                timeout,
                headers,
                proxy,
                show,
                save,
            )
            .await
        }

        DomainCommands::Drift {
            domain,
            report,
            output,
            threshold,
            urls,
            alert,
            baseline,
        } => {
            execute_drift(
                client, domain, report, output, threshold, urls, alert, baseline,
            )
            .await
        }

        DomainCommands::List {
            verbose,
            filter,
            format,
        } => execute_list(verbose, filter, &format),

        DomainCommands::Show {
            domain,
            history,
            structure,
            format,
        } => execute_show(domain, history, structure, &format),

        DomainCommands::Export {
            domain,
            output,
            history,
            format,
        } => execute_export(domain, output, history, &format),

        DomainCommands::Import {
            file,
            force,
            validate,
        } => execute_import(file, force, validate),

        DomainCommands::Rm {
            domain,
            force,
            all_versions,
        } => execute_rm(domain, force, all_versions),
    }
}

async fn execute_init(
    client: RipTideClient,
    domain: String,
    analyze: bool,
    output_path: Option<String>,
    samples: Option<String>,
    crawl_depth: u32,
    include_metadata: bool,
) -> Result<()> {
    output::print_info(&format!("Initializing domain profile for: {}", domain));

    let mut profile = ProfileManager::create(domain.clone());

    if analyze {
        output::print_info("Analyzing site structure...");

        #[derive(Serialize)]
        struct AnalyzeRequest {
            domain: String,
            samples: Vec<String>,
            crawl_depth: u32,
            include_metadata: bool,
        }

        #[derive(Deserialize)]
        struct AnalyzeResponse {
            structure: SiteStructure,
            patterns: Vec<ContentPattern>,
            selectors: HashMap<String, Vec<String>>,
            metadata: HashMap<String, String>,
            confidence: f64,
        }

        let sample_urls = samples
            .map(|s| s.split(',').map(|u| u.trim().to_string()).collect())
            .unwrap_or_else(|| vec![domain.clone()]);

        let request = AnalyzeRequest {
            domain: domain.clone(),
            samples: sample_urls,
            crawl_depth,
            include_metadata,
        };

        let response = client.post("/api/v1/domain/analyze", &request).await?;
        let analysis: AnalyzeResponse = response.json().await?;

        profile.set_baseline(SiteBaseline {
            captured_at: Utc::now(),
            structure: analysis.structure,
            patterns: analysis.patterns,
            selectors: analysis.selectors,
            metadata: analysis.metadata,
        });

        output::print_success(&format!(
            "Site analysis complete (confidence: {:.2})",
            analysis.confidence
        ));
    }

    // Save profile
    let save_path = ProfileManager::save(&profile, output_path.as_deref())?;
    output::print_success(&format!("Domain profile saved to: {}", save_path.display()));

    // Display profile summary
    display_profile_summary(&profile);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute_profile(
    domain: String,
    stealth: Option<String>,
    rate_limit: Option<f64>,
    robots_txt: bool,
    ua_strategy: Option<String>,
    schema: Option<String>,
    confidence: Option<f64>,
    javascript: bool,
    timeout: Option<u64>,
    headers: Option<String>,
    proxy: Option<String>,
    show: bool,
    save: bool,
) -> Result<()> {
    output::print_info(&format!("Configuring profile for domain: {}", domain));

    // Load existing profile or create new
    let mut profile = ProfileManager::load_or_create(domain);

    // Update configuration using the new API
    profile.update_config(|config| {
        if let Some(level) = stealth {
            config.stealth_level = level;
        }
        if let Some(limit) = rate_limit {
            config.rate_limit = limit;
        }
        if robots_txt {
            config.respect_robots_txt = true;
        }
        if let Some(strategy) = ua_strategy {
            config.ua_strategy = strategy;
        }
        if let Some(schema_name) = schema {
            config.schema = Some(schema_name);
        }
        if let Some(conf) = confidence {
            config.confidence_threshold = conf;
        }
        if javascript {
            config.enable_javascript = true;
        }
        if let Some(timeout_secs) = timeout {
            config.request_timeout_secs = timeout_secs;
        }
        if let Some(header_str) = headers {
            let headers_map: HashMap<String, String> = header_str
                .split(',')
                .filter_map(|h| {
                    let parts: Vec<&str> = h.trim().splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect();
            config.custom_headers = headers_map;
        }
        if let Some(proxy_url) = proxy {
            config.proxy = Some(proxy_url);
        }
    });

    if save {
        let save_path = ProfileManager::save(&profile, None)?;
        output::print_success(&format!(
            "Profile updated and saved to: {}",
            save_path.display()
        ));
    } else {
        output::print_info("Profile updated (not saved to disk)");
    }

    if show {
        display_profile_config(&profile);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute_drift(
    client: RipTideClient,
    domain: String,
    generate_report: bool,
    output_path: Option<String>,
    threshold: f64,
    urls: Option<String>,
    alert: bool,
    baseline_version: Option<String>,
) -> Result<()> {
    output::print_info(&format!("Checking for drift in domain: {}", domain));

    // Load domain profile
    let profile = ProfileManager::load(&domain)?;

    if profile.baseline.is_none() {
        output::print_warning("No baseline structure found for this domain");
        output::print_info(&format!(
            "Run 'riptide domain init --domain {} --analyze' first",
            domain
        ));
        return Ok(());
    }

    #[derive(Serialize)]
    struct DriftRequest {
        domain: String,
        baseline: SiteBaseline,
        urls: Vec<String>,
        threshold: f64,
        baseline_version: Option<String>,
    }

    #[derive(Deserialize)]
    struct DriftResponse {
        drift_report: riptide_intelligence::DriftReport,
    }

    let check_urls = urls
        .map(|u| u.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| vec![profile.domain.clone()]);

    let request = DriftRequest {
        domain: domain.clone(),
        baseline: profile.baseline.clone().unwrap(),
        urls: check_urls,
        threshold,
        baseline_version,
    };

    output::print_info("Analyzing current site structure...");
    let response = client.post("/api/v1/domain/drift", &request).await?;
    let result: DriftResponse = response.json().await?;

    let report = result.drift_report;

    // Display drift report using helper function
    display_drift_report(&report, generate_report, threshold, alert, output_path)?;

    Ok(())
}

fn execute_list(verbose: bool, filter: Option<String>, format: &str) -> Result<()> {
    output::print_info("Listing domain profiles...");

    let profiles = if let Some(ref pattern) = filter {
        ProfileManager::list_filtered(pattern)?
    } else {
        ProfileManager::list_all()?
    };

    if profiles.is_empty() {
        output::print_info("No domain profiles found");
        return Ok(());
    }

    display_profile_list(&profiles, verbose, format);

    Ok(())
}

fn execute_show(
    domain: String,
    show_history: bool,
    show_structure: bool,
    format: &str,
) -> Result<()> {
    let profile = ProfileManager::load(&domain)?;

    match format {
        "json" => {
            output::print_json(&profile);
        }
        "yaml" => {
            output::print_warning("YAML output not yet implemented, showing JSON instead");
            output::print_json(&profile);
        }
        _ => {
            display_profile_details(&profile, show_history, show_structure);
        }
    }

    Ok(())
}

fn execute_export(
    domain: String,
    output_path: String,
    _include_history: bool,
    _format: &str,
) -> Result<()> {
    output::print_info(&format!("Exporting domain profile: {}", domain));
    ProfileManager::export(&domain, &output_path)?;
    output::print_success(&format!("Profile exported to: {}", output_path));
    Ok(())
}

fn execute_import(file: String, force: bool, validate: bool) -> Result<()> {
    output::print_info(&format!("Importing domain profile from: {}", file));

    let profile = ProfileManager::import(&file, force, validate)?;
    output::print_success(&format!(
        "Profile '{}' imported successfully",
        profile.domain
    ));

    Ok(())
}

fn execute_rm(domain: String, force: bool, _all_versions: bool) -> Result<()> {
    if !force {
        output::print_warning(&format!(
            "This will remove the domain profile for '{}'",
            domain
        ));
        output::print_info("Use --force to confirm deletion");
        return Ok(());
    }

    ProfileManager::delete(&domain)?;
    output::print_success(&format!("Domain profile '{}' removed", domain));

    Ok(())
}

// Helper functions for display logic
fn display_profile_summary(profile: &DomainProfile) {
    output::print_section("Domain Profile");
    output::print_key_value("Domain", &profile.domain);
    output::print_key_value("Version", &profile.version);
    output::print_key_value("Created", &profile.created_at.to_rfc3339());

    if let Some(baseline) = &profile.baseline {
        output::print_section("Baseline Structure");
        output::print_key_value(
            "Common Elements",
            &baseline.structure.common_elements.len().to_string(),
        );
        output::print_key_value("Content Patterns", &baseline.patterns.len().to_string());
        output::print_key_value("Selectors", &baseline.selectors.len().to_string());
    }
}

fn display_profile_config(profile: &DomainProfile) {
    output::print_section("Domain Configuration");
    output::print_key_value("Domain", &profile.domain);
    output::print_key_value("Stealth Level", &profile.config.stealth_level);
    output::print_key_value(
        "Rate Limit",
        &format!("{} req/s", profile.config.rate_limit),
    );
    output::print_key_value(
        "Robots.txt",
        if profile.config.respect_robots_txt {
            "Yes"
        } else {
            "No"
        },
    );
    output::print_key_value("UA Strategy", &profile.config.ua_strategy);
    if let Some(schema_name) = &profile.config.schema {
        output::print_key_value("Schema", schema_name);
    }
    output::print_key_value(
        "Confidence Threshold",
        &format!("{:.2}", profile.config.confidence_threshold),
    );
    output::print_key_value(
        "JavaScript",
        if profile.config.enable_javascript {
            "Enabled"
        } else {
            "Disabled"
        },
    );
    output::print_key_value(
        "Timeout",
        &format!("{}s", profile.config.request_timeout_secs),
    );
    if !profile.config.custom_headers.is_empty() {
        output::print_section("Custom Headers");
        for (key, value) in &profile.config.custom_headers {
            println!("  {}: {}", key, value);
        }
    }
    if let Some(proxy_url) = &profile.config.proxy {
        output::print_key_value("Proxy", proxy_url);
    }
}

fn display_drift_report(
    report: &riptide_intelligence::DriftReport,
    generate_report: bool,
    threshold: f64,
    alert: bool,
    output_path: Option<String>,
) -> Result<()> {
    output::print_section("Drift Analysis");
    output::print_key_value("Domain", &report.domain);
    output::print_key_value("Baseline Version", &report.baseline_version);
    output::print_key_value("Checked At", &report.checked_at.to_rfc3339());
    output::print_key_value(
        "Overall Drift",
        &output::format_confidence(1.0 - report.overall_drift),
    );

    output::print_section("Change Summary");
    output::print_key_value("Total Changes", &report.summary.total_changes.to_string());
    output::print_key_value("Critical", &report.summary.critical.to_string());
    output::print_key_value("Major", &report.summary.major.to_string());
    output::print_key_value("Minor", &report.summary.minor.to_string());
    output::print_key_value(
        "Structural Changes",
        &report.summary.structural_changes.to_string(),
    );
    output::print_key_value(
        "Selector Changes",
        &report.summary.selector_changes.to_string(),
    );

    if report.overall_drift > threshold {
        output::print_warning(&format!(
            "Significant drift detected ({:.2}% > {:.2}%)",
            report.overall_drift * 100.0,
            threshold * 100.0
        ));

        if generate_report {
            output::print_section("Detailed Changes");
            let mut table = output::create_table(vec!["Type", "Location", "Severity", "Impact"]);

            for change in &report.changes {
                table.add_row(vec![
                    &change.change_type,
                    &change.location,
                    &change.severity,
                    &format!("{:.2}%", change.impact * 100.0),
                ]);
            }
            println!("{table}");
        }

        if !report.recommendations.is_empty() {
            output::print_section("Recommendations");
            for rec in &report.recommendations {
                println!("  • {}", rec);
            }
        }

        if alert {
            output::print_warning("⚠️ ALERT: Significant website structure changes detected!");
        }
    } else {
        output::print_success(&format!(
            "No significant drift detected ({:.2}% < {:.2}%)",
            report.overall_drift * 100.0,
            threshold * 100.0
        ));
    }

    if let Some(report_path) = output_path {
        let report_json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&report_path, report_json)?;
        output::print_success(&format!("Drift report saved to: {}", report_path));
    }

    Ok(())
}

fn display_profile_list(profiles: &[DomainProfile], verbose: bool, format: &str) {
    match format {
        "json" => {
            output::print_json(&profiles);
        }
        "list" => {
            for profile in profiles {
                println!("{} (v{})", profile.domain, profile.version);
                if verbose {
                    println!("  Stealth: {}", profile.config.stealth_level);
                    println!("  Rate Limit: {} req/s", profile.config.rate_limit);
                    println!("  Updated: {}", profile.updated_at.to_rfc3339());
                }
                println!();
            }
            output::print_info(&format!("Total: {} profiles", profiles.len()));
        }
        _ => {
            let headers = if verbose {
                vec![
                    "Domain",
                    "Version",
                    "Stealth",
                    "Rate Limit",
                    "Schema",
                    "Updated",
                ]
            } else {
                vec!["Domain", "Version", "Stealth", "Updated"]
            };

            let mut table = output::create_table(headers);

            for profile in profiles {
                let mut row = vec![
                    profile.domain.clone(),
                    profile.version.clone(),
                    profile.config.stealth_level.clone(),
                ];

                if verbose {
                    row.push(format!("{} req/s", profile.config.rate_limit));
                    row.push(
                        profile
                            .config
                            .schema
                            .as_ref()
                            .unwrap_or(&"None".to_string())
                            .clone(),
                    );
                }

                row.push(profile.updated_at.format("%Y-%m-%d").to_string());

                let row_refs: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
                table.add_row(row_refs);
            }

            println!("{table}");
            println!();
            output::print_info(&format!("Total: {} profiles", profiles.len()));
        }
    }
}

fn display_profile_details(profile: &DomainProfile, show_history: bool, show_structure: bool) {
    output::print_section("Domain Profile");
    output::print_key_value("Domain", &profile.domain);
    output::print_key_value("Version", &profile.version);
    output::print_key_value("Created", &profile.created_at.to_rfc3339());
    output::print_key_value("Updated", &profile.updated_at.to_rfc3339());

    output::print_section("Configuration");
    output::print_key_value("Stealth Level", &profile.config.stealth_level);
    output::print_key_value(
        "Rate Limit",
        &format!("{} req/s", profile.config.rate_limit),
    );
    output::print_key_value(
        "Robots.txt",
        if profile.config.respect_robots_txt {
            "Respected"
        } else {
            "Ignored"
        },
    );
    output::print_key_value("UA Strategy", &profile.config.ua_strategy);
    output::print_key_value(
        "Confidence Threshold",
        &format!("{:.2}", profile.config.confidence_threshold),
    );
    output::print_key_value(
        "JavaScript",
        if profile.config.enable_javascript {
            "Enabled"
        } else {
            "Disabled"
        },
    );
    output::print_key_value(
        "Timeout",
        &format!("{}s", profile.config.request_timeout_secs),
    );

    if let Some(schema_name) = &profile.config.schema {
        output::print_key_value("Schema", schema_name);
    }

    if !profile.config.custom_headers.is_empty() {
        output::print_section("Custom Headers");
        for (key, value) in &profile.config.custom_headers {
            println!("  {}: {}", key, value);
        }
    }

    output::print_section("Metadata");
    output::print_key_value(
        "Total Requests",
        &profile.metadata.total_requests.to_string(),
    );
    output::print_key_value(
        "Success Rate",
        &output::format_confidence(profile.metadata.success_rate),
    );
    output::print_key_value(
        "Avg Response Time",
        &format!("{}ms", profile.metadata.avg_response_time_ms),
    );

    if !profile.metadata.tags.is_empty() {
        output::print_key_value("Tags", &profile.metadata.tags.join(", "));
    }

    if show_structure {
        if let Some(baseline) = &profile.baseline {
            output::print_section("Baseline Structure");
            output::print_key_value("Captured", &baseline.captured_at.to_rfc3339());
            output::print_key_value(
                "Common Elements",
                &baseline.structure.common_elements.len().to_string(),
            );
            output::print_key_value(
                "Content Patterns",
                &baseline.patterns.len().to_string(),
            );
            output::print_key_value("Selectors", &baseline.selectors.len().to_string());

            if !baseline.structure.url_patterns.is_empty() {
                output::print_section("URL Patterns");
                for pattern in &baseline.structure.url_patterns {
                    println!("  • {} ({})", pattern.pattern, pattern.content_type);
                }
            }
        }
    }

    if show_history {
        output::print_section("Version History");
        output::print_info("Version history tracking not yet implemented");
    }
}
