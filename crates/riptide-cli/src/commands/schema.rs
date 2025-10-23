use crate::client::RipTideClient;
use crate::output;
use anyhow::Result;
use riptide_extraction::schema::{
    ExtractionSchema, SchemaComparator, SchemaLearnRequest, SchemaLearnResponse, SchemaTestRequest,
    SchemaTestResponse,
};
use std::fs;
use std::path::Path;

#[derive(clap::Subcommand)]
pub enum SchemaCommands {
    /// Learn extraction schema from a URL
    Learn {
        /// URL to analyze and learn schema from
        #[arg(long)]
        url: String,

        /// Goal-based learning type (article, product, listing, form, etc.)
        #[arg(long, default_value = "article")]
        goal: String,

        /// Output schema file path
        #[arg(long, short = 'o', default_value = "schema.json")]
        output: String,

        /// Minimum confidence threshold (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        confidence: f64,

        /// Generate selectors for specific fields (comma-separated)
        #[arg(long)]
        fields: Option<String>,

        /// Enable verbose learning output
        #[arg(long)]
        verbose: bool,
    },

    /// Test schema against URLs
    Test {
        /// Schema file to test
        #[arg(long, short = 's')]
        schema: String,

        /// URLs to test against (comma-separated)
        #[arg(long)]
        urls: String,

        /// Generate detailed validation report
        #[arg(long)]
        report: bool,

        /// Output report file path
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,
    },

    /// Compare two schemas
    Diff {
        /// First schema file
        #[arg(long)]
        schema1: String,

        /// Second schema file
        #[arg(long)]
        schema2: String,

        /// Output format (text, json, table)
        #[arg(long, default_value = "text")]
        format: String,

        /// Show only differences
        #[arg(long)]
        only_diff: bool,
    },

    /// Push schema to registry
    Push {
        /// Schema file to push
        #[arg(long, short = 's')]
        schema: String,

        /// Schema name in registry
        #[arg(long)]
        name: String,

        /// Schema version
        #[arg(long, default_value = "1.0.0")]
        version: String,

        /// Schema description
        #[arg(long)]
        description: Option<String>,

        /// Tags for schema (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Make schema public
        #[arg(long)]
        public: bool,
    },

    /// List available schemas
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by goal type
        #[arg(long)]
        goal: Option<String>,

        /// Show only public schemas
        #[arg(long)]
        public_only: bool,

        /// Output format (table, json, list)
        #[arg(long, default_value = "table")]
        format: String,

        /// Maximum number of results
        #[arg(long, default_value = "20")]
        limit: u32,
    },

    /// Show schema details
    Show {
        /// Schema name or file path
        #[arg(long)]
        schema: String,

        /// Show selector details
        #[arg(long)]
        selectors: bool,

        /// Show validation rules
        #[arg(long)]
        validation: bool,

        /// Show example usage
        #[arg(long)]
        example: bool,

        /// Output format (text, json, yaml)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Remove schema from registry
    Rm {
        /// Schema name to remove
        #[arg(long)]
        name: String,

        /// Schema version (removes all versions if not specified)
        #[arg(long)]
        version: Option<String>,

        /// Force removal without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
}

pub async fn execute(
    client: RipTideClient,
    command: SchemaCommands,
    _output_format: &str,
) -> Result<()> {
    match command {
        SchemaCommands::Learn {
            url,
            goal,
            output,
            confidence,
            fields,
            verbose,
        } => execute_learn(client, url, goal, output, confidence, fields, verbose).await,

        SchemaCommands::Test {
            schema,
            urls,
            report,
            output,
            fail_fast,
        } => execute_test(client, schema, urls, report, output, fail_fast).await,

        SchemaCommands::Diff {
            schema1,
            schema2,
            format,
            only_diff,
        } => execute_diff(schema1, schema2, &format, only_diff),

        SchemaCommands::Push {
            schema,
            name,
            version,
            description,
            tags,
            public,
        } => execute_push(client, schema, name, version, description, tags, public).await,

        SchemaCommands::List {
            tag,
            goal,
            public_only,
            format,
            limit,
        } => execute_list(client, tag, goal, public_only, &format, limit).await,

        SchemaCommands::Show {
            schema,
            selectors,
            validation,
            example,
            format,
        } => execute_show(client, schema, selectors, validation, example, &format).await,

        SchemaCommands::Rm {
            name,
            version,
            force,
        } => execute_rm(client, name, version, force).await,
    }
}

async fn execute_learn(
    client: RipTideClient,
    url: String,
    goal: String,
    output_path: String,
    confidence: f64,
    fields: Option<String>,
    verbose: bool,
) -> Result<()> {
    output::print_info(&format!("Learning extraction schema from: {}", url));
    output::print_info(&format!("Goal type: {}", goal));
    output::print_info(&format!("Confidence threshold: {:.2}", confidence));

    let fields_vec = fields.map(|f| f.split(',').map(|s| s.trim().to_string()).collect());

    let request = SchemaLearnRequest {
        url: url.clone(),
        goal: goal.clone(),
        confidence_threshold: confidence,
        fields: fields_vec,
        verbose,
    };

    let response = client.post("/api/v1/schema/learn", &request).await?;
    let result: SchemaLearnResponse = response.json().await?;

    // Save schema to file
    let schema_json = serde_json::to_string_pretty(&result.schema)?;
    fs::write(&output_path, schema_json)?;

    output::print_success(&format!("Schema learned and saved to: {}", output_path));
    println!();

    // Display analysis
    output::print_section("Schema Analysis");
    output::print_key_value(
        "Overall Confidence",
        &output::format_confidence(result.analysis.confidence),
    );
    output::print_key_value(
        "Fields Detected",
        &result.analysis.fields_detected.to_string(),
    );
    output::print_key_value(
        "Selectors Generated",
        &result.analysis.selectors_generated.to_string(),
    );

    if !result.analysis.patterns_found.is_empty() {
        output::print_section("Patterns Found");
        for pattern in &result.analysis.patterns_found {
            println!("  • {}", pattern);
        }
    }

    if !result.analysis.warnings.is_empty() {
        output::print_section("Warnings");
        for warning in &result.analysis.warnings {
            output::print_warning(&format!("  • {}", warning));
        }
    }

    if !result.suggestions.is_empty() {
        output::print_section("Suggestions");
        for suggestion in &result.suggestions {
            println!("  • {}", suggestion);
        }
    }

    // Display field summary
    if verbose {
        output::print_section("Fields");
        for (field_name, field) in &result.schema.fields {
            println!("  • {} ({})", field_name, field.field_type);
            if let Some(desc) = &field.description {
                println!("    Description: {}", desc);
            }
            println!("    Required: {}", field.required);
        }
    }

    Ok(())
}

async fn execute_test(
    client: RipTideClient,
    schema_path: String,
    urls: String,
    report: bool,
    output_path: Option<String>,
    fail_fast: bool,
) -> Result<()> {
    output::print_info(&format!("Testing schema: {}", schema_path));

    // Load schema from file
    let schema_content = fs::read_to_string(&schema_path)?;
    let schema: ExtractionSchema = serde_json::from_str(&schema_content)?;

    // Parse URLs
    let url_list: Vec<String> = urls.split(',').map(|s| s.trim().to_string()).collect();
    output::print_info(&format!("Testing against {} URLs", url_list.len()));

    let request = SchemaTestRequest {
        schema,
        urls: url_list,
        fail_fast,
    };

    let response = client.post("/api/v1/schema/test", &request).await?;
    let result: SchemaTestResponse = response.json().await?;

    // Display results
    output::print_section("Test Results");
    output::print_key_value("Total Tests", &result.total_tests.to_string());
    output::print_key_value(
        "Passed",
        &format!(
            "{} ({}%)",
            result.passed,
            (result.success_rate * 100.0) as u32
        ),
    );
    output::print_key_value("Failed", &result.failed.to_string());
    output::print_key_value(
        "Success Rate",
        &output::format_confidence(result.success_rate),
    );
    println!();

    // Show summary statistics
    output::print_section("Summary Statistics");
    output::print_key_value(
        "Average Confidence",
        &output::format_confidence(result.summary.avg_confidence),
    );
    output::print_key_value(
        "Average Extraction Time",
        &format!("{}ms", result.summary.avg_extraction_time_ms),
    );

    if !result.summary.most_common_errors.is_empty() {
        output::print_section("Most Common Errors");
        for error in &result.summary.most_common_errors {
            println!("  • {}", error);
        }
    }

    // Display individual test results if verbose or if report is requested
    if report {
        output::print_section("Detailed Results");
        let mut table = output::create_table(vec!["URL", "Status", "Confidence", "Fields", "Time"]);

        for test_result in &result.results {
            let status = if test_result.success {
                "✓ PASS"
            } else {
                "✗ FAIL"
            };

            table.add_row(vec![
                &test_result.url,
                status,
                &output::format_confidence(test_result.confidence),
                &test_result.fields_extracted.to_string(),
                &format!("{}ms", test_result.extraction_time_ms),
            ]);
        }
        println!("{table}");

        // Show field success rates
        if !result.summary.field_success_rates.is_empty() {
            output::print_section("Field Success Rates");
            let mut field_table = output::create_table(vec!["Field", "Success Rate"]);
            for (field, rate) in &result.summary.field_success_rates {
                field_table.add_row(vec![field, &output::format_confidence(*rate)]);
            }
            println!("{field_table}");
        }
    }

    // Save report if output path specified
    if let Some(report_path) = output_path {
        let report_json = serde_json::to_string_pretty(&result)?;
        fs::write(&report_path, report_json)?;
        output::print_success(&format!("Report saved to: {}", report_path));
    }

    Ok(())
}

fn execute_diff(
    schema1_path: String,
    schema2_path: String,
    format: &str,
    _only_diff: bool,
) -> Result<()> {
    output::print_info(&format!(
        "Comparing schemas: {} vs {}",
        schema1_path, schema2_path
    ));

    // Load both schemas
    let schema1_content = fs::read_to_string(&schema1_path)?;
    let schema1: ExtractionSchema = serde_json::from_str(&schema1_content)?;

    let schema2_content = fs::read_to_string(&schema2_path)?;
    let schema2: ExtractionSchema = serde_json::from_str(&schema2_content)?;

    // Use SchemaComparator from extraction crate
    let comparator = SchemaComparator::new();
    let comparison = comparator.compare(&schema1, &schema2)?;

    match format {
        "json" => {
            let json = comparator.generate_json_report(&comparison)?;
            println!("{}", json);
        }
        "table" => {
            let table_data = comparator.generate_table_report(&comparison);
            let mut table = output::create_table(vec!["Category", "Value"]);
            for row in table_data.iter().skip(1) {
                // Skip header
                let row_vec: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
                table.add_row(row_vec);
            }
            println!("{table}");
        }
        _ => {
            // Text format
            let report = comparator.generate_text_report(&comparison);
            println!("{}", report);
        }
    }

    Ok(())
}

async fn execute_push(
    client: RipTideClient,
    schema_path: String,
    name: String,
    version: String,
    description: Option<String>,
    tags: Option<String>,
    public: bool,
) -> Result<()> {
    output::print_info(&format!("Pushing schema: {}", name));

    // Load schema from file
    let schema_content = fs::read_to_string(&schema_path)?;
    let mut schema: ExtractionSchema = serde_json::from_str(&schema_content)?;

    // Update schema metadata
    schema.name = name.clone();
    schema.version = version.clone();
    if let Some(desc) = description {
        schema.description = Some(desc);
    }
    if let Some(tag_str) = tags {
        schema.metadata.tags = tag_str.split(',').map(|s| s.trim().to_string()).collect();
    }
    schema.metadata.is_public = public;

    let response = client.post("/api/v1/schema/push", &schema).await?;

    if response.status().is_success() {
        output::print_success(&format!(
            "Schema '{}' v{} pushed to registry",
            name, version
        ));
        if public {
            output::print_info("Schema is now public");
        }
    } else {
        anyhow::bail!("Failed to push schema: {}", response.status());
    }

    Ok(())
}

async fn execute_list(
    client: RipTideClient,
    tag: Option<String>,
    goal: Option<String>,
    public_only: bool,
    format: &str,
    limit: u32,
) -> Result<()> {
    use riptide_extraction::schema::registry::{ListRequest, ListResponse};

    output::print_info("Listing schemas from registry");

    let request = ListRequest {
        tag,
        goal,
        public_only,
        limit,
    };

    let response = client.post("/api/v1/schema/list", &request).await?;
    let result: ListResponse = response.json().await?;

    match format {
        "json" => {
            output::print_json(&result);
        }
        "list" => {
            for schema in &result.schemas {
                println!("{} v{} ({})", schema.name, schema.version, schema.goal);
                if let Some(desc) = &schema.description {
                    println!("  {}", desc);
                }
                println!();
            }
            output::print_info(&format!("Total: {} schemas", result.total));
        }
        _ => {
            // Table format
            let mut table = output::create_table(vec![
                "Name",
                "Version",
                "Goal",
                "Public",
                "Usage",
                "Success Rate",
            ]);

            for schema in &result.schemas {
                let public_str = if schema.is_public { "Yes" } else { "No" };
                let success_str = schema
                    .success_rate
                    .map(output::format_confidence)
                    .unwrap_or_else(|| "N/A".to_string());

                table.add_row(vec![
                    &schema.name,
                    &schema.version,
                    &schema.goal,
                    public_str,
                    &schema.usage_count.to_string(),
                    &success_str,
                ]);
            }

            println!("{table}");
            println!();
            output::print_info(&format!(
                "Showing {} of {} schemas",
                result.schemas.len(),
                result.total
            ));
        }
    }

    Ok(())
}

async fn execute_show(
    client: RipTideClient,
    schema_identifier: String,
    show_selectors: bool,
    show_validation: bool,
    show_example: bool,
    format: &str,
) -> Result<()> {
    use serde::Serialize;

    // Check if it's a file path or registry name
    let schema: ExtractionSchema = if Path::new(&schema_identifier).exists() {
        output::print_info(&format!("Loading schema from file: {}", schema_identifier));
        let content = fs::read_to_string(&schema_identifier)?;
        serde_json::from_str(&content)?
    } else {
        output::print_info(&format!(
            "Fetching schema from registry: {}",
            schema_identifier
        ));
        #[derive(Serialize)]
        struct ShowRequest {
            name: String,
        }
        let request = ShowRequest {
            name: schema_identifier,
        };
        let response = client.post("/api/v1/schema/show", &request).await?;
        response.json().await?
    };

    match format {
        "json" => {
            output::print_json(&schema);
        }
        "yaml" => {
            // For now, output as JSON with warning
            output::print_warning("YAML output not yet implemented, showing JSON instead");
            output::print_json(&schema);
        }
        _ => {
            // Text format
            output::print_section("Schema Information");
            output::print_key_value("Name", &schema.name);
            output::print_key_value("Version", &schema.version);
            output::print_key_value("Goal", &schema.goal);
            if let Some(desc) = &schema.description {
                output::print_key_value("Description", desc);
            }
            output::print_key_value("Public", &schema.metadata.is_public.to_string());
            output::print_key_value("Usage Count", &schema.metadata.usage_count.to_string());
            if let Some(rate) = schema.metadata.success_rate {
                output::print_key_value("Success Rate", &output::format_confidence(rate));
            }

            if !schema.metadata.tags.is_empty() {
                output::print_key_value("Tags", &schema.metadata.tags.join(", "));
            }

            // Show fields
            output::print_section("Fields");
            for (field_name, field) in &schema.fields {
                println!(
                    "  • {} ({}){}",
                    field_name,
                    field.field_type,
                    if field.required { " [required]" } else { "" }
                );
                if let Some(desc) = &field.description {
                    println!("    {}", desc);
                }
            }

            // Show selectors if requested
            if show_selectors {
                output::print_section("Selectors");
                for (field, rules) in &schema.selectors {
                    println!("  {}:", field);
                    for rule in rules {
                        println!("    • {} ({})", rule.selector, rule.selector_type);
                        println!(
                            "      Priority: {}, Confidence: {:.2}",
                            rule.priority, rule.confidence
                        );
                        if let Some(fallback) = &rule.fallback {
                            println!("      Fallback: {}", fallback);
                        }
                    }
                }
            }

            // Show validation rules if requested
            if show_validation {
                if let Some(validation) = &schema.validation {
                    output::print_section("Validation Rules");
                    if let Some(min_fields) = validation.min_fields {
                        output::print_key_value("Minimum Fields", &min_fields.to_string());
                    }
                    if let Some(required) = &validation.required_fields {
                        output::print_key_value("Required Fields", &required.join(", "));
                    }
                    if let Some(min_conf) = validation.min_confidence {
                        output::print_key_value("Minimum Confidence", &min_conf.to_string());
                    }
                }
            }

            // Show example usage if requested
            if show_example {
                output::print_section("Example Usage");
                println!("  riptide extract --url <URL> --schema {}", schema.name);
                println!(
                    "  riptide schema test --schema {} --urls <URLs>",
                    schema.name
                );
            }
        }
    }

    Ok(())
}

async fn execute_rm(
    client: RipTideClient,
    name: String,
    version: Option<String>,
    force: bool,
) -> Result<()> {
    use serde::Serialize;

    if !force {
        output::print_warning("This will remove the schema from the registry.");
        output::print_info("Use --force to confirm deletion");
        return Ok(());
    }

    #[derive(Serialize)]
    struct RmRequest {
        name: String,
        version: Option<String>,
    }

    let request = RmRequest {
        name: name.clone(),
        version: version.clone(),
    };

    let response = client.post("/api/v1/schema/remove", &request).await?;

    if response.status().is_success() {
        if let Some(ver) = version {
            output::print_success(&format!("Schema '{}' v{} removed from registry", name, ver));
        } else {
            output::print_success(&format!(
                "All versions of schema '{}' removed from registry",
                name
            ));
        }
    } else {
        anyhow::bail!("Failed to remove schema: {}", response.status());
    }

    Ok(())
}
