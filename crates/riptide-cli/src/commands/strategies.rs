/// Strategies command - Manage and use extraction strategies
///
/// This command provides information about available extraction strategies
/// and allows crawling URLs with specific strategies.
use crate::client::ApiClient;
use crate::output::{self, OutputFormat};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

/// Strategy subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum StrategyCommands {
    /// List available extraction strategies
    List,
    /// Get detailed info about a strategy
    Info {
        /// Strategy name (native/wasm/headless/auto)
        name: String,
    },
    /// Use a strategy to crawl a URL
    Crawl {
        /// URL to crawl
        url: String,
        /// Strategy to use (native/wasm/headless/auto)
        #[arg(short, long)]
        strategy: String,
        /// Output format (json/table/text)
        #[arg(short = 'f', long, default_value = "json")]
        format: String,
    },
}

/// Arguments for the strategies command
#[derive(Args, Clone, Debug)]
pub struct StrategyArgs {
    #[command(subcommand)]
    pub command: StrategyCommands,
}

/// Response from the strategies API endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResponse {
    pub recommended_strategy: String,
    pub confidence_score: f64,
    pub reasoning: String,
    pub alternatives: Vec<AlternativeStrategy>,
    pub processing_time_ms: u64,
}

/// Alternative strategy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeStrategy {
    pub strategy: String,
    pub score: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

/// Request for strategy-based crawl
#[derive(Serialize, Debug)]
struct CrawlRequest {
    url: String,
    force_strategy: Option<String>,
    enable_probe: Option<bool>,
}

/// Execute the strategies command
pub async fn execute(client: ApiClient, args: StrategyArgs, output_format: String) -> Result<()> {
    match args.command {
        StrategyCommands::List => execute_list(client, output_format).await,
        StrategyCommands::Info { name } => execute_info(client, name, output_format).await,
        StrategyCommands::Crawl {
            url,
            strategy,
            format,
        } => execute_crawl(client, url, strategy, format).await,
    }
}

/// List all available strategies
async fn execute_list(client: ApiClient, output_format: String) -> Result<()> {
    output::print_info("Fetching available strategies...");

    // GET /strategies/info endpoint
    let response = client
        .get("/strategies/info")
        .await
        .context("Failed to fetch strategies from API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("Unknown error"));
        anyhow::bail!("API returned error {}: {}", status, error_text);
    }

    let strategy_response = response
        .json::<StrategyResponse>()
        .await
        .context("Failed to parse strategies response")?;

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_strategy_list(&strategy_response, format)?;

    Ok(())
}

/// Get detailed info about a specific strategy
async fn execute_info(client: ApiClient, name: String, output_format: String) -> Result<()> {
    output::print_info(&format!("Fetching info for strategy '{}'...", name));

    // GET /strategies/info endpoint
    let response = client
        .get("/strategies/info")
        .await
        .context("Failed to fetch strategies from API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("Unknown error"));
        anyhow::bail!("API returned error {}: {}", status, error_text);
    }

    let strategy_response = response
        .json::<StrategyResponse>()
        .await
        .context("Failed to parse strategies response")?;

    // Find the specific strategy
    let strategy = strategy_response
        .alternatives
        .iter()
        .find(|s| s.strategy.to_lowercase() == name.to_lowercase())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Strategy '{}' not found. Available strategies: {}",
                name,
                strategy_response
                    .alternatives
                    .iter()
                    .map(|s| s.strategy.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_strategy_info(strategy, format)?;

    Ok(())
}

/// Crawl a URL using a specific strategy
async fn execute_crawl(
    client: ApiClient,
    url: String,
    strategy: String,
    output_format: String,
) -> Result<()> {
    output::print_info(&format!(
        "Crawling '{}' with '{}' strategy...",
        url, strategy
    ));

    // Build request payload
    let request = CrawlRequest {
        url: url.clone(),
        force_strategy: Some(strategy.clone()),
        enable_probe: Some(false),
    };

    // POST /strategies/crawl endpoint
    let response = client
        .post::<CrawlRequest, StrategyResponse>("/strategies/crawl", &request)
        .await
        .context("Failed to crawl URL with strategy")?;

    // Format and print output
    let format = OutputFormat::parse(&output_format)?;
    print_crawl_result(&response, format)?;

    Ok(())
}

/// Print strategy list in the specified format
fn print_strategy_list(response: &StrategyResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            print_strategies_table(response)?;
        }
        OutputFormat::Text => {
            print_strategies_text(response)?;
        }
        OutputFormat::Stream => {
            // For strategies, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print strategies as a formatted table
fn print_strategies_table(response: &StrategyResponse) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::{Cell, Color, Table};

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Strategy", "Score", "Pros", "Cons"]);

    for strategy in &response.alternatives {
        let score_cell = {
            let score = strategy.score;
            let color = if score >= 0.8 {
                Color::Green
            } else if score >= 0.5 {
                Color::Yellow
            } else {
                Color::Red
            };
            Cell::new(format!("{:.2}", score)).fg(color)
        };

        let pros = strategy.pros.join("\n");
        let cons = strategy.cons.join("\n");

        table.add_row(vec![
            Cell::new(&strategy.strategy),
            score_cell,
            Cell::new(&pros),
            Cell::new(&cons),
        ]);
    }

    println!("{}", table);
    println!("\nConfiguration:");
    println!("  Recommended: {}", response.recommended_strategy);
    println!("  Confidence: {:.2}", response.confidence_score);
    println!("  Reasoning: {}", response.reasoning);

    Ok(())
}

/// Print strategies as formatted text
fn print_strategies_text(response: &StrategyResponse) -> Result<()> {
    println!("Available Strategies:\n");

    for (i, strategy) in response.alternatives.iter().enumerate() {
        println!(
            "{}. {} (Score: {:.2})",
            i + 1,
            strategy.strategy,
            strategy.score
        );

        if !strategy.pros.is_empty() {
            println!("   Pros:");
            for pro in &strategy.pros {
                println!("     + {}", pro);
            }
        }

        if !strategy.cons.is_empty() {
            println!("   Cons:");
            for con in &strategy.cons {
                println!("     - {}", con);
            }
        }

        println!();
    }

    println!("Configuration:");
    println!("  Recommended: {}", response.recommended_strategy);
    println!("  Confidence: {:.2}", response.confidence_score);
    println!("  Reasoning: {}", response.reasoning);

    Ok(())
}

/// Print detailed info about a specific strategy
fn print_strategy_info(strategy: &AlternativeStrategy, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(strategy)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            use comfy_table::modifiers::UTF8_ROUND_CORNERS;
            use comfy_table::presets::UTF8_FULL;
            use comfy_table::{Cell, Color, Table};

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["Property", "Value"]);

            table.add_row(vec![Cell::new("Strategy"), Cell::new(&strategy.strategy)]);

            let score_cell = {
                let score = strategy.score;
                let color = if score >= 0.8 {
                    Color::Green
                } else if score >= 0.5 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                Cell::new(format!("{:.2}", score)).fg(color)
            };

            table.add_row(vec![Cell::new("Score"), score_cell]);

            table.add_row(vec![Cell::new("Pros"), Cell::new(strategy.pros.join("\n"))]);

            table.add_row(vec![Cell::new("Cons"), Cell::new(strategy.cons.join("\n"))]);

            println!("{}", table);
        }
        OutputFormat::Text => {
            println!("Strategy: {}", strategy.strategy);
            println!("Score: {:.2}", strategy.score);

            if !strategy.pros.is_empty() {
                println!("\nPros:");
                for pro in &strategy.pros {
                    println!("  + {}", pro);
                }
            }

            if !strategy.cons.is_empty() {
                println!("\nCons:");
                for con in &strategy.cons {
                    println!("  - {}", con);
                }
            }
        }
        OutputFormat::Stream => {
            // For single strategy info, stream format is same as JSON
            let json = serde_json::to_string_pretty(strategy)?;
            println!("{}", json);
        }
    }
    Ok(())
}

/// Print crawl result in the specified format
fn print_crawl_result(response: &StrategyResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
        OutputFormat::Table => {
            use comfy_table::modifiers::UTF8_ROUND_CORNERS;
            use comfy_table::presets::UTF8_FULL;
            use comfy_table::{Cell, Color, Table};

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec!["Property", "Value"]);

            table.add_row(vec![
                Cell::new("Strategy Used"),
                Cell::new(&response.recommended_strategy),
            ]);

            let confidence_cell = {
                let score = response.confidence_score;
                let color = if score >= 0.8 {
                    Color::Green
                } else if score >= 0.5 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                Cell::new(format!("{:.2}", score)).fg(color)
            };

            table.add_row(vec![Cell::new("Confidence"), confidence_cell]);

            table.add_row(vec![Cell::new("Reasoning"), Cell::new(&response.reasoning)]);

            table.add_row(vec![
                Cell::new("Processing Time"),
                Cell::new(format!("{}ms", response.processing_time_ms)),
            ]);

            println!("{}", table);
        }
        OutputFormat::Text => {
            println!("Crawl Result:");
            println!("  Strategy Used: {}", response.recommended_strategy);
            println!("  Confidence: {:.2}", response.confidence_score);
            println!("  Reasoning: {}", response.reasoning);
            println!("  Processing Time: {}ms", response.processing_time_ms);
        }
        OutputFormat::Stream => {
            // For crawl result, stream format is same as JSON
            let json = serde_json::to_string_pretty(response)?;
            println!("{}", json);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_response_deserialization() {
        let json = r#"{
            "recommended_strategy": "auto",
            "confidence_score": 0.95,
            "reasoning": "Test reasoning",
            "alternatives": [
                {
                    "strategy": "native",
                    "score": 1.0,
                    "pros": ["Fast"],
                    "cons": ["No JS"]
                }
            ],
            "processing_time_ms": 10
        }"#;

        let response: StrategyResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.recommended_strategy, "auto");
        assert_eq!(response.confidence_score, 0.95);
        assert_eq!(response.alternatives.len(), 1);
        assert_eq!(response.alternatives[0].strategy, "native");
    }

    #[test]
    fn test_crawl_request_serialization() {
        let request = CrawlRequest {
            url: "https://example.com".to_string(),
            force_strategy: Some("native".to_string()),
            enable_probe: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("native"));
        assert!(json.contains("\"enable_probe\":false"));
    }

    #[test]
    fn test_alternative_strategy_deserialization() {
        let json = r#"{
            "strategy": "wasm",
            "score": 0.8,
            "pros": ["JS support", "SPA compatible"],
            "cons": ["Higher overhead"]
        }"#;

        let strategy: AlternativeStrategy = serde_json::from_str(json).unwrap();
        assert_eq!(strategy.strategy, "wasm");
        assert_eq!(strategy.score, 0.8);
        assert_eq!(strategy.pros.len(), 2);
        assert_eq!(strategy.cons.len(), 1);
    }
}
