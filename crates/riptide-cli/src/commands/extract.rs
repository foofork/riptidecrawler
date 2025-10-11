use crate::client::RipTideClient;
use crate::commands::ExtractArgs;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize)]
struct ExtractRequest {
    url: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strategy: Option<String>,
    include_confidence: bool,
}

#[derive(Deserialize, Serialize)]
struct ExtractResponse {
    content: String,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    method_used: Option<String>,
    #[serde(default)]
    extraction_time_ms: Option<u64>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

pub async fn execute(client: RipTideClient, args: ExtractArgs, output_format: &str) -> Result<()> {
    output::print_info(&format!("Extracting content from: {}", args.url));

    let request = ExtractRequest {
        url: args.url.clone(),
        method: args.method.clone(),
        selector: args.selector,
        pattern: args.pattern,
        strategy: args.strategy,
        include_confidence: args.show_confidence,
    };

    let response = client.post("/api/v1/extract", &request).await?;
    let extract_result: ExtractResponse = response.json().await?;

    match output_format {
        "json" => {
            output::print_json(&extract_result);
        }
        "text" => {
            output::print_success("Content extracted successfully");
            println!();

            if args.show_confidence {
                if let Some(confidence) = extract_result.confidence {
                    output::print_key_value("Confidence", &output::format_confidence(confidence));
                }
            }

            if let Some(method) = extract_result.method_used {
                output::print_key_value("Method", &method);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                output::print_key_value("Extraction Time", &format!("{}ms", time));
            }

            if args.metadata {
                if let Some(metadata) = extract_result.metadata {
                    output::print_section("Metadata");
                    println!("{}", serde_json::to_string_pretty(&metadata)?);
                }
            }

            output::print_section("Extracted Content");
            println!("{}", extract_result.content);

            // Save to file if specified
            if let Some(file_path) = args.file {
                fs::write(&file_path, &extract_result.content)?;
                output::print_success(&format!("Content saved to: {}", file_path));
            }
        }
        "table" => {
            let mut table = output::create_table(vec!["Field", "Value"]);
            table.add_row(vec!["URL", &args.url]);

            if let Some(confidence) = extract_result.confidence {
                table.add_row(vec!["Confidence", &output::format_confidence(confidence)]);
            }

            if let Some(method) = extract_result.method_used {
                table.add_row(vec!["Method", &method]);
            }

            if let Some(time) = extract_result.extraction_time_ms {
                table.add_row(vec!["Time", &format!("{}ms", time)]);
            }

            table.add_row(vec![
                "Content Length",
                &format!("{} chars", extract_result.content.len()),
            ]);

            println!("{table}");
        }
        _ => {
            output::print_warning(&format!("Unknown output format: {}", output_format));
            output::print_json(&extract_result);
        }
    }

    Ok(())
}
