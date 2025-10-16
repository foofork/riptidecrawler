use crate::client::RipTideClient;
use crate::commands::SearchArgs;
use crate::output;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<SearchResult>,
    total_results: u32,
    search_time_ms: u64,
}

#[derive(Deserialize, Serialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
    #[serde(default)]
    relevance_score: Option<f64>,
}

pub async fn execute(client: RipTideClient, args: SearchArgs, output_format: &str) -> Result<()> {
    output::print_info(&format!("Searching for: {}", args.query));

    let mut url = format!(
        "/api/v1/search?q={}&limit={}",
        urlencoding::encode(&args.query),
        args.limit
    );

    if let Some(domain) = args.domain {
        url.push_str(&format!("&domain={}", urlencoding::encode(&domain)));
    }

    let response = client.get(&url).await?;
    let result: SearchResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        "table" => {
            output::print_success(&format!(
                "Found {} results in {}ms",
                result.total_results, result.search_time_ms
            ));

            if !result.results.is_empty() {
                let mut table = output::create_table(vec!["Title", "URL", "Relevance"]);
                for search_result in &result.results {
                    let relevance = search_result
                        .relevance_score
                        .map(output::format_confidence)
                        .unwrap_or_else(|| "N/A".to_string());
                    table.add_row(vec![&search_result.title, &search_result.url, &relevance]);
                }
                println!("{table}");
            }
        }
        _ => {
            output::print_success(&format!(
                "Found {} results in {}ms",
                result.total_results, result.search_time_ms
            ));
            println!();

            for (idx, search_result) in result.results.iter().enumerate() {
                output::print_section(&format!("Result #{}", idx + 1));
                output::print_key_value("Title", &search_result.title);
                output::print_key_value("URL", &search_result.url);
                output::print_key_value("Snippet", &search_result.snippet);

                if let Some(score) = search_result.relevance_score {
                    output::print_key_value("Relevance", &output::format_confidence(score));
                }
                println!();
            }
        }
    }

    Ok(())
}
