use crate::client::RipTideClient;
use crate::output;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;

#[derive(clap::Subcommand)]
pub enum JobCommands {
    /// Submit a new extraction job
    Submit {
        /// URLs to extract (comma-separated or multiple --url flags)
        #[arg(long, required = true)]
        url: Vec<String>,

        /// Extraction method (wasm, css, llm, regex, auto)
        #[arg(long, default_value = "auto")]
        method: String,

        /// Job name for identification
        #[arg(long)]
        name: Option<String>,

        /// Job priority (low, medium, high, critical)
        #[arg(long, default_value = "medium", value_parser = ["low", "medium", "high", "critical"])]
        priority: String,

        /// Output directory for results
        #[arg(long, short = 'o')]
        output_dir: Option<String>,

        /// Enable batch mode (process all URLs in parallel)
        #[arg(long)]
        batch: bool,

        /// Maximum concurrent extractions in batch mode
        #[arg(long, default_value = "5")]
        max_concurrent: u32,

        /// Job tags for categorization (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// JSON file with additional job configuration
        #[arg(long)]
        config: Option<String>,
    },

    /// List all jobs
    List {
        /// Filter by job status (pending, running, completed, failed, cancelled)
        #[arg(long, value_parser = ["pending", "running", "completed", "failed", "cancelled"])]
        status: Option<String>,

        /// Filter by priority
        #[arg(long, value_parser = ["low", "medium", "high", "critical"])]
        priority: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Maximum number of jobs to list
        #[arg(long, default_value = "50")]
        limit: u32,

        /// Show only jobs from the last N hours
        #[arg(long)]
        recent: Option<u32>,
    },

    /// Check status of a specific job
    Status {
        /// Job ID to check
        #[arg(long)]
        job_id: String,

        /// Show detailed status information
        #[arg(long)]
        detailed: bool,

        /// Watch mode - continuously update status
        #[arg(long, short = 'w')]
        watch: bool,

        /// Update interval in seconds for watch mode
        #[arg(long, default_value = "2")]
        interval: u64,
    },

    /// View logs for a job
    Logs {
        /// Job ID
        #[arg(long)]
        job_id: String,

        /// Follow log output (tail -f style)
        #[arg(long, short = 'f')]
        follow: bool,

        /// Number of log lines to show
        #[arg(long, default_value = "100")]
        lines: u32,

        /// Filter logs by level (debug, info, warn, error)
        #[arg(long, value_parser = ["debug", "info", "warn", "error"])]
        level: Option<String>,

        /// Search pattern in logs
        #[arg(long)]
        grep: Option<String>,
    },

    /// Cancel a running job
    Cancel {
        /// Job ID to cancel
        #[arg(long)]
        job_id: String,

        /// Force cancellation without cleanup
        #[arg(long)]
        force: bool,

        /// Cancel multiple jobs by tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Get job results
    Results {
        /// Job ID
        #[arg(long)]
        job_id: String,

        /// Output format (json, text, markdown)
        #[arg(long, default_value = "text")]
        format: String,

        /// Output directory to save results
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// Include raw HTML in results
        #[arg(long)]
        include_html: bool,
    },

    /// Retry a failed job
    Retry {
        /// Job ID to retry
        #[arg(long)]
        job_id: String,

        /// Maximum retry attempts
        #[arg(long, default_value = "3")]
        max_retries: u32,
    },

    /// Show job statistics
    Stats {
        /// Time range (1h, 24h, 7d, 30d, all)
        #[arg(long, default_value = "24h")]
        range: String,

        /// Group by field (status, priority, method)
        #[arg(long, value_parser = ["status", "priority", "method"])]
        group_by: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: Option<String>,
    pub status: JobStatus,
    pub priority: JobPriority,
    pub urls: Vec<String>,
    pub method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: JobProgress,
    pub tags: Vec<String>,
    pub error: Option<String>,
    pub results_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobProgress {
    pub total: u32,
    pub completed: u32,
    pub failed: u32,
    pub percentage: f32,
    pub current_url: Option<String>,
}

#[derive(Serialize)]
struct SubmitJobRequest {
    urls: Vec<String>,
    method: String,
    name: Option<String>,
    priority: String,
    batch: bool,
    max_concurrent: u32,
    tags: Vec<String>,
    config: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Serialize)]
struct SubmitJobResponse {
    job_id: String,
    status: String,
}

#[derive(Deserialize, Serialize)]
struct JobListResponse {
    jobs: Vec<Job>,
    total: u32,
}

#[derive(Deserialize, Serialize)]
struct JobLogsResponse {
    logs: Vec<LogEntry>,
    has_more: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: String,
    message: String,
}

#[derive(Deserialize, Serialize)]
struct JobStatsResponse {
    total_jobs: u32,
    by_status: HashMap<String, u32>,
    by_priority: HashMap<String, u32>,
    by_method: HashMap<String, u32>,
    avg_duration_seconds: f64,
    success_rate: f64,
}

pub async fn execute(
    client: RipTideClient,
    command: JobCommands,
    output_format: &str,
) -> Result<()> {
    match command {
        JobCommands::Submit {
            url,
            method,
            name,
            priority,
            output_dir,
            batch,
            max_concurrent,
            tags,
            config,
        } => {
            submit_job(
                client,
                url,
                method,
                name,
                priority,
                output_dir,
                batch,
                max_concurrent,
                tags,
                config,
                output_format,
            )
            .await
        }
        JobCommands::List {
            status,
            priority,
            tag,
            limit,
            recent,
        } => list_jobs(client, status, priority, tag, limit, recent, output_format).await,
        JobCommands::Status {
            job_id,
            detailed,
            watch,
            interval,
        } => job_status(client, job_id, detailed, watch, interval, output_format).await,
        JobCommands::Logs {
            job_id,
            follow,
            lines,
            level,
            grep,
        } => job_logs(client, job_id, follow, lines, level, grep).await,
        JobCommands::Cancel { job_id, force, tag } => {
            cancel_job(client, job_id, force, tag, output_format).await
        }
        JobCommands::Results {
            job_id,
            format,
            output,
            include_html,
        } => job_results(client, job_id, format, output, include_html).await,
        JobCommands::Retry {
            job_id,
            max_retries,
        } => retry_job(client, job_id, max_retries, output_format).await,
        JobCommands::Stats { range, group_by } => {
            job_stats(client, range, group_by, output_format).await
        }
    }
}

async fn submit_job(
    client: RipTideClient,
    urls: Vec<String>,
    method: String,
    name: Option<String>,
    priority: String,
    output_dir: Option<String>,
    batch: bool,
    max_concurrent: u32,
    tags: Option<String>,
    config_file: Option<String>,
    output_format: &str,
) -> Result<()> {
    // Parse tags
    let tags_vec = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    // Load config if provided
    let config = if let Some(path) = config_file {
        let content =
            fs::read_to_string(&path).context(format!("Failed to read config file: {}", path))?;
        Some(serde_json::from_str(&content).context("Invalid JSON in config file")?)
    } else {
        None
    };

    output::print_info(&format!("Submitting job with {} URL(s)...", urls.len()));

    let request = SubmitJobRequest {
        urls: urls.clone(),
        method,
        name: name.clone(),
        priority: priority.clone(),
        batch,
        max_concurrent,
        tags: tags_vec,
        config,
    };

    let response = client.post("/api/v1/jobs/submit", &request).await?;
    let result: SubmitJobResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        _ => {
            output::print_success(&format!("Job submitted successfully: {}", result.job_id));
            output::print_info(&format!("Status: {}", result.status));
            if let Some(dir) = output_dir {
                output::print_info(&format!("Results will be saved to: {}", dir));
            }
            output::print_info(&format!(
                "\nView status: riptide job status --job-id {}",
                result.job_id
            ));
            output::print_info(&format!(
                "View logs: riptide job logs --job-id {}",
                result.job_id
            ));
        }
    }

    Ok(())
}

async fn list_jobs(
    client: RipTideClient,
    status: Option<String>,
    priority: Option<String>,
    tag: Option<String>,
    limit: u32,
    recent: Option<u32>,
    output_format: &str,
) -> Result<()> {
    let mut query_params = vec![("limit", limit.to_string())];

    if let Some(s) = status {
        query_params.push(("status", s));
    }
    if let Some(p) = priority {
        query_params.push(("priority", p));
    }
    if let Some(t) = tag {
        query_params.push(("tag", t));
    }
    if let Some(r) = recent {
        query_params.push(("recent_hours", r.to_string()));
    }

    let query_string = query_params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let response = client
        .get(&format!("/api/v1/jobs?{}", query_string))
        .await?;
    let result: JobListResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        _ => {
            output::print_success(&format!("Found {} job(s)", result.total));

            if !result.jobs.is_empty() {
                let mut table = output::create_table(vec![
                    "ID", "Name", "Status", "Priority", "Progress", "Created",
                ]);

                for job in &result.jobs {
                    let name = job.name.as_deref().unwrap_or("-");
                    let progress = format!(
                        "{}/{} ({}%)",
                        job.progress.completed, job.progress.total, job.progress.percentage as u32
                    );
                    let created = job.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

                    table.add_row(vec![
                        &job.id[..8],
                        name,
                        &format!("{:?}", job.status),
                        &format!("{:?}", job.priority),
                        &progress,
                        &created,
                    ]);
                }
                println!("{table}");
            }
        }
    }

    Ok(())
}

async fn job_status(
    client: RipTideClient,
    job_id: String,
    detailed: bool,
    watch: bool,
    interval: u64,
    output_format: &str,
) -> Result<()> {
    if watch {
        output::print_info("Watching job status (Ctrl+C to exit)...\n");
        loop {
            let response = client.get(&format!("/api/v1/jobs/{}", job_id)).await?;
            let job: Job = response.json().await?;

            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            display_job_status(&job, detailed, output_format);

            if matches!(
                job.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            ) {
                output::print_info("\nJob completed. Exiting watch mode.");
                break;
            }

            sleep(Duration::from_secs(interval)).await;
        }
    } else {
        let response = client.get(&format!("/api/v1/jobs/{}", job_id)).await?;
        let job: Job = response.json().await?;
        display_job_status(&job, detailed, output_format);
    }

    Ok(())
}

fn display_job_status(job: &Job, detailed: bool, output_format: &str) {
    match output_format {
        "json" => output::print_json(&job),
        _ => {
            output::print_success(&format!("Job: {}", job.id));
            if let Some(name) = &job.name {
                println!("Name: {}", name);
            }
            println!("Status: {:?}", job.status);
            println!("Priority: {:?}", job.priority);
            println!("Method: {}", job.method);
            println!(
                "Progress: {}/{} ({}%)",
                job.progress.completed, job.progress.total, job.progress.percentage as u32
            );

            if let Some(current) = &job.progress.current_url {
                println!("Current URL: {}", current);
            }

            println!("Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S"));

            if let Some(started) = job.started_at {
                println!("Started: {}", started.format("%Y-%m-%d %H:%M:%S"));
            }

            if let Some(completed) = job.completed_at {
                println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));

                if let Some(started) = job.started_at {
                    let duration = completed.signed_duration_since(started);
                    println!(
                        "Duration: {:.2}s",
                        duration.num_milliseconds() as f64 / 1000.0
                    );
                }
            }

            if !job.tags.is_empty() {
                println!("Tags: {}", job.tags.join(", "));
            }

            if let Some(error) = &job.error {
                output::print_error(&format!("Error: {}", error));
            }

            if detailed {
                println!("\nURLs ({}):", job.urls.len());
                for (idx, url) in job.urls.iter().enumerate().take(10) {
                    println!("  {}. {}", idx + 1, url);
                }
                if job.urls.len() > 10 {
                    println!("  ... and {} more", job.urls.len() - 10);
                }
            }
        }
    }
}

async fn job_logs(
    client: RipTideClient,
    job_id: String,
    follow: bool,
    lines: u32,
    level: Option<String>,
    grep: Option<String>,
) -> Result<()> {
    let mut last_timestamp: Option<DateTime<Utc>> = None;

    loop {
        let mut query_params = vec![("lines", lines.to_string())];

        if let Some(lvl) = &level {
            query_params.push(("level", lvl.clone()));
        }
        if let Some(ts) = &last_timestamp {
            query_params.push(("after", ts.to_rfc3339()));
        }

        let query_string = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let response = client
            .get(&format!("/api/v1/jobs/{}/logs?{}", job_id, query_string))
            .await?;
        let result: JobLogsResponse = response.json().await?;

        for entry in &result.logs {
            if let Some(pattern) = &grep {
                if !entry.message.contains(pattern) {
                    continue;
                }
            }

            let level_color = match entry.level.as_str() {
                "ERROR" => "\x1b[31m", // Red
                "WARN" => "\x1b[33m",  // Yellow
                "INFO" => "\x1b[32m",  // Green
                _ => "\x1b[37m",       // White
            };

            println!(
                "{} {}{}\x1b[0m {}",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                level_color,
                entry.level,
                entry.message
            );

            last_timestamp = Some(entry.timestamp);
        }

        if !follow || !result.has_more {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn cancel_job(
    client: RipTideClient,
    job_id: String,
    force: bool,
    tag: Option<String>,
    output_format: &str,
) -> Result<()> {
    let endpoint = if let Some(t) = tag {
        format!("/api/v1/jobs/cancel?tag={}&force={}", t, force)
    } else {
        format!("/api/v1/jobs/{}/cancel?force={}", job_id, force)
    };

    output::print_info("Cancelling job...");

    let response = client.post(&endpoint, &()).await?;
    let result: serde_json::Value = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        _ => {
            output::print_success("Job cancelled successfully");
        }
    }

    Ok(())
}

async fn job_results(
    client: RipTideClient,
    job_id: String,
    format: String,
    output: Option<String>,
    include_html: bool,
) -> Result<()> {
    let response = client
        .get(&format!(
            "/api/v1/jobs/{}/results?include_html={}",
            job_id, include_html
        ))
        .await?;
    let result: serde_json::Value = response.json().await?;

    if let Some(output_path) = output {
        let content = match format.as_str() {
            "json" => serde_json::to_string_pretty(&result)?,
            "markdown" => {
                // Convert results to markdown format
                format!("# Job Results: {}\n\n{:#?}", job_id, result)
            }
            _ => format!("{:#?}", result),
        };

        fs::write(&output_path, content)?;
        output::print_success(&format!("Results saved to: {}", output_path));
    } else {
        match format.as_str() {
            "json" => output::print_json(&result),
            _ => println!("{:#?}", result),
        }
    }

    Ok(())
}

async fn retry_job(
    client: RipTideClient,
    job_id: String,
    max_retries: u32,
    output_format: &str,
) -> Result<()> {
    output::print_info(&format!(
        "Retrying job {} (max {} attempts)...",
        job_id, max_retries
    ));

    let response = client
        .post(
            &format!("/api/v1/jobs/{}/retry?max_retries={}", job_id, max_retries),
            &(),
        )
        .await?;
    let result: SubmitJobResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        _ => {
            output::print_success(&format!("Job retry initiated: {}", result.job_id));
        }
    }

    Ok(())
}

async fn job_stats(
    client: RipTideClient,
    range: String,
    group_by: Option<String>,
    output_format: &str,
) -> Result<()> {
    let mut query_params = vec![("range", range)];

    if let Some(group) = group_by {
        query_params.push(("group_by", group));
    }

    let query_string = query_params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");

    let response = client
        .get(&format!("/api/v1/jobs/stats?{}", query_string))
        .await?;
    let result: JobStatsResponse = response.json().await?;

    match output_format {
        "json" => output::print_json(&result),
        _ => {
            output::print_success(&format!("Job Statistics ({})", query_params[0].1));
            println!("\nTotal Jobs: {}", result.total_jobs);
            println!("Average Duration: {:.2}s", result.avg_duration_seconds);
            println!("Success Rate: {:.1}%", result.success_rate * 100.0);

            println!("\nBy Status:");
            let mut table = output::create_table(vec!["Status", "Count"]);
            for (status, count) in &result.by_status {
                table.add_row(vec![status, &count.to_string()]);
            }
            println!("{table}");

            println!("\nBy Priority:");
            let mut table = output::create_table(vec!["Priority", "Count"]);
            for (priority, count) in &result.by_priority {
                table.add_row(vec![priority, &count.to_string()]);
            }
            println!("{table}");

            println!("\nBy Method:");
            let mut table = output::create_table(vec!["Method", "Count"]);
            for (method, count) in &result.by_method {
                table.add_row(vec![method, &count.to_string()]);
            }
            println!("{table}");
        }
    }

    Ok(())
}
