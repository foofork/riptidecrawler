use crate::client::ApiClient;
use crate::output;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Validates that a session ID is non-empty and contains only alphanumeric characters and hyphens
fn validate_session_id(session_id: &str) -> Result<()> {
    if session_id.is_empty() {
        bail!("Session ID cannot be empty");
    }

    if !session_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        bail!("Session ID must contain only alphanumeric characters, hyphens, and underscores");
    }

    Ok(())
}

/// Validates that a URL has a proper scheme and format
fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        bail!("URL cannot be empty");
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        bail!("URL must start with http:// or https://");
    }

    // Basic URL validation - check for at least a domain
    let url_without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap();

    if url_without_scheme.is_empty() || !url_without_scheme.contains('.') {
        bail!("Invalid URL format - must include a domain");
    }

    Ok(())
}

/// Checks HTTP response status and parses error body if request failed
async fn check_response_status(response: reqwest::Response) -> Result<reqwest::Response> {
    let status = response.status();

    if !status.is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read error response".to_string());

        bail!(
            "API request failed with status {}: {}",
            status,
            error_body
        );
    }

    Ok(response)
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SessionApiCommands {
    /// Create new session
    Create {
        /// Session name
        #[arg(long, short = 'n')]
        name: Option<String>,

        /// Session TTL in seconds
        #[arg(long, default_value = "3600")]
        ttl: u64,
    },

    /// List all sessions
    List,

    /// Get session details
    Get {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,
    },

    /// Delete session
    Delete {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,
    },

    /// Add URL to session
    Add {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,

        /// URL to add
        #[arg(value_name = "URL")]
        url: String,
    },

    /// Extract content from session URLs
    Extract {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,

        /// Extraction strategy
        #[arg(long, value_parser = ["auto", "css", "wasm", "llm", "multi"], default_value = "multi")]
        strategy: String,
    },

    /// Get session extraction results
    Results {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,
    },

    /// Export session data
    Export {
        /// Session ID
        #[arg(value_name = "SESSION_ID")]
        session_id: String,

        /// Export format
        #[arg(long, value_parser = ["json", "csv", "ndjson"], default_value = "json")]
        format: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    ttl: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionResponse {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    created_at: String,
    ttl: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionListResponse {
    sessions: Vec<SessionSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionSummary {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    created_at: String,
    url_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionDetails {
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    created_at: String,
    ttl: u64,
    urls: Vec<String>,
    results_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddUrlRequest {
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExtractRequest {
    strategy: String,
}

pub async fn execute(
    client: ApiClient,
    command: SessionApiCommands,
    output_format: &str,
) -> Result<()> {
    match command {
        SessionApiCommands::Create { name, ttl } => {
            let request = CreateSessionRequest { name, ttl };

            let response = client
                .post_json("/sessions", serde_json::to_value(&request)?)
                .await
                .context("Failed to create session")?;

            let response: SessionResponse = response.json().await?;

            match output_format {
                "json" => output::print_json(&response),
                _ => {
                    println!("✓ Session created");
                    println!();
                    println!("ID: {}", response.session_id);
                    if let Some(n) = response.name {
                        println!("Name: {}", n);
                    }
                    println!("TTL: {} seconds", response.ttl);
                }
            }
        }

        SessionApiCommands::List => {
            let response = client
                .get("/sessions")
                .await
                .context("Failed to list sessions")?;

            let response: SessionListResponse = response.json().await?;

            match output_format {
                "json" => output::print_json(&response.sessions),
                _ => {
                    if response.sessions.is_empty() {
                        println!("No active sessions");
                        return Ok(());
                    }

                    println!("✓ Found {} session(s)", response.sessions.len());
                    println!();

                    let mut table =
                        output::create_table(vec!["Session ID", "Name", "Created", "URLs"]);

                    for session in response.sessions {
                        table.add_row(vec![
                            &session.session_id,
                            &session.name.unwrap_or_else(|| "-".to_string()),
                            &session.created_at,
                            &session.url_count.to_string(),
                        ]);
                    }

                    println!("{}", table);
                }
            }
        }

        SessionApiCommands::Get { session_id } => {
            validate_session_id(&session_id)?;

            let response = client
                .get(&format!("/sessions/{}", session_id))
                .await
                .context("Failed to get session")?;

            let response = check_response_status(response).await?;
            let response: SessionDetails = response.json().await?;

            match output_format {
                "json" => output::print_json(&response),
                _ => {
                    println!("Session Details");
                    println!();
                    println!("ID: {}", response.session_id);
                    if let Some(name) = response.name {
                        println!("Name: {}", name);
                    }
                    println!("Created: {}", response.created_at);
                    println!("TTL: {} seconds", response.ttl);
                    println!("URLs: {}", response.urls.len());
                    println!("Results: {}", response.results_count);

                    if !response.urls.is_empty() {
                        println!();
                        println!("URLs:");
                        for url in response.urls {
                            println!("  - {}", url);
                        }
                    }
                }
            }
        }

        SessionApiCommands::Delete { session_id } => {
            validate_session_id(&session_id)?;

            let response = client
                .delete(&format!("/sessions/{}", session_id))
                .await
                .context("Failed to delete session")?;

            let response = check_response_status(response).await?;
            let _ = response.text().await?;

            match output_format {
                "json" => output::print_json(&serde_json::json!({"deleted": session_id})),
                _ => println!("✓ Session deleted: {}", session_id),
            }
        }

        SessionApiCommands::Add { session_id, url } => {
            validate_session_id(&session_id)?;
            validate_url(&url)?;

            let request = AddUrlRequest { url: url.clone() };

            let response = client
                .post_json(
                    &format!("/sessions/{}/urls", session_id),
                    serde_json::to_value(&request)?,
                )
                .await
                .context("Failed to add URL to session")?;

            let response = check_response_status(response).await?;
            let _ = response.text().await?;

            match output_format {
                "json" => output::print_json(&serde_json::json!({
                    "session_id": session_id,
                    "url": url
                })),
                _ => println!("✓ Added URL to session: {}", url),
            }
        }

        SessionApiCommands::Extract {
            session_id,
            strategy,
        } => {
            validate_session_id(&session_id)?;

            let request = ExtractRequest {
                strategy: strategy.clone(),
            };

            let response = client
                .post_json(
                    &format!("/sessions/{}/extract", session_id),
                    serde_json::to_value(&request)?,
                )
                .await
                .context("Failed to extract session content")?;

            let response = check_response_status(response).await?;
            let _ = response.text().await?;

            match output_format {
                "json" => output::print_json(&serde_json::json!({
                    "session_id": session_id,
                    "strategy": strategy,
                    "status": "extracting"
                })),
                _ => println!(
                    "✓ Extraction started for session using {} strategy",
                    strategy
                ),
            }
        }

        SessionApiCommands::Results { session_id } => {
            validate_session_id(&session_id)?;

            let response = client
                .get(&format!("/sessions/{}/results", session_id))
                .await
                .context("Failed to get session results")?;

            let response = check_response_status(response).await?;
            let response: serde_json::Value = response.json().await?;

            match output_format {
                "json" => output::print_json(&response),
                _ => {
                    if let Some(results) = response.get("results").and_then(|r| r.as_array()) {
                        println!("✓ Session Results ({} results)", results.len());
                        println!();

                        for (i, result) in results.iter().enumerate() {
                            println!("Result {}:", i + 1);
                            if let Some(url) = result.get("url").and_then(|u| u.as_str()) {
                                println!("  URL: {}", url);
                            }
                            if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
                                println!("  Status: {}", status);
                            }
                            if let Some(content) = result.get("content").and_then(|c| c.as_str()) {
                                let preview = if content.len() > 100 {
                                    format!("{}...", &content[..100])
                                } else {
                                    content.to_string()
                                };
                                println!("  Content: {}", preview);
                            }
                            println!();
                        }
                    } else {
                        println!("No results available");
                    }
                }
            }
        }

        SessionApiCommands::Export { session_id, format } => {
            validate_session_id(&session_id)?;

            let response = client
                .get(&format!(
                    "/sessions/{}/export?format={}",
                    session_id, format
                ))
                .await
                .context("Failed to export session")?;

            let response = check_response_status(response).await?;
            let response: serde_json::Value = response.json().await?;

            match output_format {
                "json" => output::print_json(&response),
                _ => {
                    // For non-JSON output formats, print the data directly
                    println!("{}", serde_json::to_string_pretty(&response)?);
                }
            }
        }
    }

    Ok(())
}
