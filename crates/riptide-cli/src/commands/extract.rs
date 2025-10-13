use crate::client::RipTideClient;
use crate::commands::ExtractArgs;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

// Local extraction support
use riptide_html::wasm_extraction::WasmExtractor;

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

    // Use local extraction if --local flag is set
    if args.local {
        return execute_local_extraction(args, output_format).await;
    }

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

/// Execute local WASM extraction without API server
async fn execute_local_extraction(args: ExtractArgs, output_format: &str) -> Result<()> {
    use std::time::Instant;

    // Fetch HTML content
    output::print_info("Fetching HTML content...");
    let client = reqwest::Client::builder()
        .user_agent("RipTide/1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let start = Instant::now();
    let response = client.get(&args.url).send().await?;
    let html = response.text().await?;
    let fetch_time = start.elapsed();

    // Perform local extraction
    output::print_info("Performing local WASM extraction...");
    let extraction_start = Instant::now();

    // Determine WASM path
    let wasm_path = std::env::var("RIPTIDE_WASM_PATH").unwrap_or_else(|_| {
        let manifest_dir = std::env!("CARGO_MANIFEST_DIR");
        format!(
            "{}/../../target/wasm32-wasip2/release/riptide-extractor-wasm.component.wasm",
            manifest_dir
        )
    });

    // Create extractor and extract content
    let extractor = WasmExtractor::new(&wasm_path).await?;

    let mode = if args.metadata {
        "metadata"
    } else if args.method == "full" {
        "full"
    } else {
        "article"
    };

    let result = extractor.extract(html.as_bytes(), &args.url, mode)?;
    let extraction_time = extraction_start.elapsed();

    // Calculate word count and confidence
    let word_count = result.text.split_whitespace().count();
    let confidence = result.quality_score.unwrap_or(0) as f64 / 100.0;

    // Create response structure
    let extract_result = ExtractResponse {
        content: result.text.clone(),
        confidence: Some(confidence),
        method_used: Some("local-wasm".to_string()),
        extraction_time_ms: Some(extraction_time.as_millis() as u64),
        metadata: Some(serde_json::json!({
            "title": result.title,
            "byline": result.byline,
            "published": result.published_iso,
            "site_name": result.site_name,
            "description": result.description,
            "word_count": word_count,
            "reading_time": result.reading_time,
            "quality_score": result.quality_score,
            "links_count": result.links.len(),
            "media_count": result.media.len(),
            "language": result.language,
            "categories": result.categories,
            "fetch_time_ms": fetch_time.as_millis(),
        })),
    };

    // Output results
    match output_format {
        "json" => {
            output::print_json(&extract_result);
        }
        "text" => {
            output::print_success("Content extracted successfully (local mode)");
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
            table.add_row(vec!["Mode", "Local WASM"]);

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
