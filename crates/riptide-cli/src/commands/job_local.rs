/// Local job management commands (no API server required)
use crate::job::{JobManager, JobPriority, JobStatus, LogLevel};
use crate::output;
use anyhow::Result;
use clap::Subcommand;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Subcommand)]
pub enum JobLocalCommands {
    /// Submit a new extraction job
    Submit {
        /// URLs to extract (comma-separated or multiple --url flags)
        #[arg(long, required = true)]
        url: Vec<String>,

        /// Extraction strategy (auto, wasm, css, llm, regex, etc.)
        #[arg(long, default_value = "auto")]
        strategy: String,

        /// Job name for identification
        #[arg(long)]
        name: Option<String>,

        /// Job priority (low, medium, high, critical)
        #[arg(long, default_value = "medium", value_parser = ["low", "medium", "high", "critical"])]
        priority: String,

        /// Job tags for categorization (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Enable streaming output
        #[arg(long)]
        stream: bool,
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
        limit: usize,
    },

    /// Check status of a specific job
    Status {
        /// Job ID to check (can use short form, e.g., first 8 chars)
        #[arg(long)]
        id: String,

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
        id: String,

        /// Follow log output (tail -f style)
        #[arg(long, short = 'f')]
        follow: bool,

        /// Number of log lines to show
        #[arg(long, default_value = "100")]
        lines: usize,

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
        id: String,
    },

    /// Get job results
    Results {
        /// Job ID
        #[arg(long)]
        id: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// Show job statistics
    Stats,

    /// Clean up old completed jobs
    Cleanup {
        /// Delete jobs older than N days
        #[arg(long, default_value = "30")]
        days: u32,

        /// Show what would be deleted without actually deleting
        #[arg(long)]
        dry_run: bool,
    },

    /// Show storage information
    Storage,
}

pub async fn execute(command: JobLocalCommands, output_format: &str) -> Result<()> {
    let manager = JobManager::new()?;

    match command {
        JobLocalCommands::Submit {
            url,
            strategy,
            name,
            priority,
            tags,
            stream,
        } => {
            submit_job(
                manager,
                url,
                strategy,
                name,
                priority,
                tags,
                stream,
                output_format,
            )
            .await
        }

        JobLocalCommands::List {
            status,
            priority,
            tag,
            limit,
        } => list_jobs(manager, status, priority, tag, Some(limit), output_format).await,

        JobLocalCommands::Status {
            id,
            detailed,
            watch,
            interval,
        } => job_status(manager, id, detailed, watch, interval, output_format).await,

        JobLocalCommands::Logs {
            id,
            follow,
            lines,
            level,
            grep,
        } => job_logs(manager, id, follow, Some(lines), level, grep).await,

        JobLocalCommands::Cancel { id } => cancel_job(manager, id, output_format).await,

        JobLocalCommands::Results { id, output } => {
            job_results(manager, id, output, output_format).await
        }

        JobLocalCommands::Stats => job_stats(manager, output_format).await,

        JobLocalCommands::Cleanup { days, dry_run } => {
            cleanup_jobs(manager, days, dry_run, output_format).await
        }

        JobLocalCommands::Storage => storage_info(manager, output_format).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn submit_job(
    manager: JobManager,
    urls: Vec<String>,
    strategy: String,
    name: Option<String>,
    priority_str: String,
    tags: Option<String>,
    stream: bool,
    output_format: &str,
) -> Result<()> {
    let priority = JobPriority::from(priority_str.as_str());
    let tags_vec = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    output::print_info(&format!("Submitting job with {} URL(s)...", urls.len()));

    let job_id = manager
        .submit_job(urls, strategy, name, priority, tags_vec, stream)
        .await?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({
            "job_id": job_id.as_str(),
            "status": "pending"
        })),
        _ => {
            output::print_success(&format!("Job submitted successfully: {}", job_id));
            output::print_info(&format!(
                "\nView status: riptide job status --id {}",
                job_id
            ));
            output::print_info(&format!("View logs: riptide job logs --id {}", job_id));
        }
    }

    Ok(())
}

async fn list_jobs(
    manager: JobManager,
    status: Option<String>,
    priority: Option<String>,
    tag: Option<String>,
    limit: Option<usize>,
    output_format: &str,
) -> Result<()> {
    let status_filter = status.as_ref().map(|s| match s.as_str() {
        "pending" => JobStatus::Pending,
        "running" => JobStatus::Running,
        "completed" => JobStatus::Completed,
        "failed" => JobStatus::Failed,
        "cancelled" => JobStatus::Cancelled,
        _ => JobStatus::Pending,
    });

    let priority_filter = priority.as_ref().map(|p| JobPriority::from(p.as_str()));

    let jobs = manager
        .list_jobs(status_filter, priority_filter, tag, limit)
        .await?;

    match output_format {
        "json" => output::print_json(&jobs),
        _ => {
            output::print_success(&format!("Found {} job(s)", jobs.len()));

            if !jobs.is_empty() {
                let mut table = output::create_table(vec![
                    "ID", "Name", "Status", "Priority", "Progress", "Created",
                ]);

                for job in &jobs {
                    let name = job.name.as_deref().unwrap_or("-");
                    let progress = format!(
                        "{}/{} ({:.0}%)",
                        job.progress.completed, job.progress.total, job.progress.percentage
                    );
                    let created = job.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

                    table.add_row(vec![
                        job.short_id(),
                        name,
                        &job.status.to_string(),
                        &job.priority.to_string(),
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
    manager: JobManager,
    job_id: String,
    detailed: bool,
    watch: bool,
    interval: u64,
    output_format: &str,
) -> Result<()> {
    let job_id = resolve_job_id(&manager, &job_id).await?;

    if watch {
        output::print_info("Watching job status (Ctrl+C to exit)...\n");
        loop {
            let job = manager.get_job(&job_id).await?;

            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            display_job_status(&job, detailed, output_format);

            if job.is_terminal() {
                output::print_info("\nJob completed. Exiting watch mode.");
                break;
            }

            sleep(Duration::from_secs(interval)).await;
        }
    } else {
        let job = manager.get_job(&job_id).await?;
        display_job_status(&job, detailed, output_format);
    }

    Ok(())
}

fn display_job_status(job: &crate::job::Job, detailed: bool, output_format: &str) {
    match output_format {
        "json" => output::print_json(&job),
        _ => {
            output::print_success(&format!("Job: {}", job.id));
            if let Some(name) = &job.name {
                println!("Name: {}", name);
            }
            println!("Status: {}", job.status);
            println!("Priority: {}", job.priority);
            println!("Strategy: {}", job.strategy);
            println!(
                "Progress: {}/{} ({:.0}%)",
                job.progress.completed, job.progress.total, job.progress.percentage
            );

            if let Some(current) = &job.progress.current_item {
                println!("Current: {}", current);
            }

            println!("Created: {}", job.created_at.format("%Y-%m-%d %H:%M:%S"));

            if let Some(started) = job.started_at {
                println!("Started: {}", started.format("%Y-%m-%d %H:%M:%S"));
            }

            if let Some(completed) = job.completed_at {
                println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));

                if let Some(duration) = job.duration_secs() {
                    println!("Duration: {:.2}s", duration);
                }
            }

            if !job.tags.is_empty() {
                println!("Tags: {}", job.tags.join(", "));
            }

            if let Some(error) = &job.error {
                output::print_error(&format!("Error: {}", error));
            }

            if let Some(log_path) = &job.log_path {
                println!("Logs: {}", log_path);
            }

            if let Some(results_path) = &job.results_path {
                println!("Results: {}", results_path);
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
    manager: JobManager,
    job_id: String,
    follow: bool,
    lines: Option<usize>,
    level: Option<String>,
    grep: Option<String>,
) -> Result<()> {
    let job_id = resolve_job_id(&manager, &job_id).await?;

    if follow {
        output::print_info("Following logs (Ctrl+C to exit)...\n");
        let mut last_count = 0;

        loop {
            let logs = manager.read_logs(&job_id, None, level.as_deref()).await?;

            // Only show new logs
            for entry in logs.iter().skip(last_count) {
                if let Some(ref pattern) = grep {
                    if !entry.message.contains(pattern) {
                        continue;
                    }
                }

                print_log_entry(entry);
            }

            last_count = logs.len();

            // Check if job is complete
            let job = manager.get_job(&job_id).await?;
            if job.is_terminal() {
                output::print_info("\nJob completed.");
                break;
            }

            sleep(Duration::from_secs(1)).await;
        }
    } else {
        let logs = manager.read_logs(&job_id, lines, level.as_deref()).await?;

        for entry in &logs {
            if let Some(ref pattern) = grep {
                if !entry.message.contains(pattern) {
                    continue;
                }
            }

            print_log_entry(entry);
        }
    }

    Ok(())
}

fn print_log_entry(entry: &crate::job::types::LogEntry) {
    let level_color = match entry.level {
        LogLevel::Error => "\x1b[31m", // Red
        LogLevel::Warn => "\x1b[33m",  // Yellow
        LogLevel::Info => "\x1b[32m",  // Green
        LogLevel::Debug => "\x1b[37m", // White
    };

    let url_str = if let Some(ref url) = entry.url {
        format!(" [{}]", url)
    } else {
        String::new()
    };

    println!(
        "{} {}{}\x1b[0m {}{}",
        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
        level_color,
        entry.level,
        entry.message,
        url_str
    );
}

async fn cancel_job(manager: JobManager, job_id: String, output_format: &str) -> Result<()> {
    let job_id = resolve_job_id(&manager, &job_id).await?;

    output::print_info("Cancelling job...");
    manager.cancel_job(&job_id).await?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({"status": "cancelled"})),
        _ => output::print_success("Job cancelled successfully"),
    }

    Ok(())
}

async fn job_results(
    manager: JobManager,
    job_id: String,
    output_path: Option<String>,
    output_format: &str,
) -> Result<()> {
    let job_id = resolve_job_id(&manager, &job_id).await?;

    let results = manager.load_results(&job_id).await?;

    if let Some(path) = output_path {
        let content = serde_json::to_string_pretty(&results)?;
        std::fs::write(&path, content)?;
        output::print_success(&format!("Results saved to: {}", path));
    } else {
        match output_format {
            "json" => output::print_json(&results),
            _ => println!("{:#?}", results),
        }
    }

    Ok(())
}

async fn job_stats(manager: JobManager, output_format: &str) -> Result<()> {
    let stats = manager.get_stats().await?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({
            "total_jobs": stats.total_jobs,
            "by_status": stats.by_status,
            "by_priority": stats.by_priority,
            "avg_duration_secs": stats.avg_duration_secs,
            "success_rate": stats.success_rate,
        })),
        _ => {
            output::print_success("Job Statistics");
            println!("\nTotal Jobs: {}", stats.total_jobs);
            println!("Average Duration: {:.2}s", stats.avg_duration_secs);
            println!("Success Rate: {:.1}%", stats.success_rate * 100.0);

            println!("\nBy Status:");
            let mut table = output::create_table(vec!["Status", "Count"]);
            for (status, count) in &stats.by_status {
                table.add_row(vec![status, &count.to_string()]);
            }
            println!("{table}");

            println!("\nBy Priority:");
            let mut table = output::create_table(vec!["Priority", "Count"]);
            for (priority, count) in &stats.by_priority {
                table.add_row(vec![priority, &count.to_string()]);
            }
            println!("{table}");
        }
    }

    Ok(())
}

async fn cleanup_jobs(
    manager: JobManager,
    days: u32,
    dry_run: bool,
    output_format: &str,
) -> Result<()> {
    output::print_info(&format!(
        "{}Cleaning up jobs older than {} days...",
        if dry_run { "[DRY RUN] " } else { "" },
        days
    ));

    let deleted = if dry_run {
        // Just list what would be deleted
        let all_jobs = manager.list_jobs(None, None, None, None).await?;
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);

        all_jobs
            .into_iter()
            .filter(|job| {
                job.is_terminal() && job.completed_at.map(|t| t < cutoff).unwrap_or(false)
            })
            .map(|job| job.id)
            .collect()
    } else {
        manager.cleanup_old_jobs(days).await?
    };

    match output_format {
        "json" => output::print_json(&serde_json::json!({
            "deleted_count": deleted.len(),
            "deleted_jobs": deleted,
            "dry_run": dry_run
        })),
        _ => {
            output::print_success(&format!(
                "{} {} job(s)",
                if dry_run { "Would delete" } else { "Deleted" },
                deleted.len()
            ));

            if !deleted.is_empty() && deleted.len() <= 20 {
                for job_id in &deleted {
                    println!("  - {}", job_id);
                }
            }
        }
    }

    Ok(())
}

async fn storage_info(manager: JobManager, output_format: &str) -> Result<()> {
    let stats = manager.get_storage_stats()?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({
            "total_jobs": stats.total_jobs,
            "total_size_bytes": stats.total_size_bytes,
            "total_size_human": stats.size_human(),
            "base_dir": stats.base_dir,
        })),
        _ => {
            output::print_success("Job Storage Information");
            println!("\nBase Directory: {:?}", stats.base_dir);
            println!("Total Jobs: {}", stats.total_jobs);
            println!("Total Size: {}", stats.size_human());
        }
    }

    Ok(())
}

/// Resolve a potentially short job ID to full ID
async fn resolve_job_id(manager: &JobManager, short_id: &str) -> Result<crate::job::types::JobId> {
    // If it's already a full ID, return it
    if short_id.len() > 16 {
        return Ok(short_id.into());
    }

    // Try to find matching job
    let jobs = manager.list_jobs(None, None, None, None).await?;

    for job in jobs {
        if job.id.as_str().starts_with(short_id) {
            return Ok(job.id);
        }
    }

    anyhow::bail!("Job not found: {}", short_id)
}
