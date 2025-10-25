#![allow(dead_code)]

use super::types::{Job, JobId, LogEntry};
use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Job storage manager for persisting jobs to disk
pub struct JobStorage {
    /// Base directory for job storage (~/.riptide/jobs/)
    base_dir: PathBuf,
}

impl JobStorage {
    /// Create a new job storage instance
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let base_dir = home.join(".riptide").join("jobs");

        // Create base directory if it doesn't exist
        fs::create_dir_all(&base_dir)
            .context(format!("Failed to create jobs directory: {:?}", base_dir))?;

        Ok(Self { base_dir })
    }

    /// Get the directory path for a specific job
    fn job_dir(&self, job_id: &JobId) -> PathBuf {
        self.base_dir.join(job_id.as_str())
    }

    /// Get the metadata file path for a job
    fn metadata_path(&self, job_id: &JobId) -> PathBuf {
        self.job_dir(job_id).join("metadata.json")
    }

    /// Get the log file path for a job
    fn log_path(&self, job_id: &JobId) -> PathBuf {
        self.job_dir(job_id).join("logs.jsonl")
    }

    /// Get the results file path for a job
    fn results_path(&self, job_id: &JobId) -> PathBuf {
        self.job_dir(job_id).join("results.json")
    }

    /// Save a job to storage
    pub fn save_job(&self, job: &Job) -> Result<()> {
        let job_dir = self.job_dir(&job.id);
        fs::create_dir_all(&job_dir)
            .context(format!("Failed to create job directory: {:?}", job_dir))?;

        let metadata_path = self.metadata_path(&job.id);
        let metadata =
            serde_json::to_string_pretty(job).context("Failed to serialize job metadata")?;

        fs::write(&metadata_path, metadata)
            .context(format!("Failed to write job metadata: {:?}", metadata_path))?;

        Ok(())
    }

    /// Load a job from storage
    pub fn load_job(&self, job_id: &JobId) -> Result<Job> {
        let metadata_path = self.metadata_path(job_id);
        let metadata = fs::read_to_string(&metadata_path)
            .context(format!("Failed to read job metadata: {:?}", metadata_path))?;

        let mut job: Job =
            serde_json::from_str(&metadata).context("Failed to deserialize job metadata")?;

        // Update paths
        job.log_path = Some(self.log_path(job_id).to_string_lossy().to_string());
        job.results_path = Some(self.results_path(job_id).to_string_lossy().to_string());

        Ok(job)
    }

    /// List all job IDs in storage
    pub fn list_jobs(&self) -> Result<Vec<JobId>> {
        let mut job_ids = Vec::new();

        if !self.base_dir.exists() {
            return Ok(job_ids);
        }

        for entry in fs::read_dir(&self.base_dir).context(format!(
            "Failed to read jobs directory: {:?}",
            self.base_dir
        ))? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    if let Some(name_str) = dir_name.to_str() {
                        job_ids.push(JobId::from(name_str));
                    }
                }
            }
        }

        Ok(job_ids)
    }

    /// Delete a job from storage
    pub fn delete_job(&self, job_id: &JobId) -> Result<()> {
        let job_dir = self.job_dir(job_id);
        if job_dir.exists() {
            fs::remove_dir_all(&job_dir)
                .context(format!("Failed to delete job directory: {:?}", job_dir))?;
        }
        Ok(())
    }

    /// Append a log entry to job's log file
    pub fn append_log(&self, job_id: &JobId, entry: &LogEntry) -> Result<()> {
        let log_path = self.log_path(job_id);
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .context(format!("Failed to open log file: {:?}", log_path))?;

        let log_line = serde_json::to_string(entry).context("Failed to serialize log entry")?;

        writeln!(file, "{}", log_line)
            .context(format!("Failed to write to log file: {:?}", log_path))?;

        Ok(())
    }

    /// Read log entries from job's log file
    pub fn read_logs(
        &self,
        job_id: &JobId,
        lines: Option<usize>,
        level_filter: Option<&str>,
    ) -> Result<Vec<LogEntry>> {
        let log_path = self.log_path(job_id);

        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let file =
            File::open(&log_path).context(format!("Failed to open log file: {:?}", log_path))?;
        let reader = BufReader::new(file);

        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                // Apply level filter if specified
                if let Some(filter_level) = level_filter {
                    if entry.level.to_string().to_lowercase() != filter_level.to_lowercase() {
                        continue;
                    }
                }
                entries.push(entry);
            }
        }

        // Apply line limit if specified (take last N lines)
        if let Some(limit) = lines {
            if entries.len() > limit {
                let skip = entries.len() - limit;
                entries.drain(0..skip);
            }
        }

        Ok(entries)
    }

    /// Save job results to file
    pub fn save_results(&self, job_id: &JobId, results: &serde_json::Value) -> Result<()> {
        let results_path = self.results_path(job_id);
        let results_json =
            serde_json::to_string_pretty(results).context("Failed to serialize results")?;

        fs::write(&results_path, results_json)
            .context(format!("Failed to write results file: {:?}", results_path))?;

        Ok(())
    }

    /// Load job results from file
    pub fn load_results(&self, job_id: &JobId) -> Result<serde_json::Value> {
        let results_path = self.results_path(job_id);

        if !results_path.exists() {
            return Ok(serde_json::json!({"status": "no results yet"}));
        }

        let results_str = fs::read_to_string(&results_path)
            .context(format!("Failed to read results file: {:?}", results_path))?;

        let results = serde_json::from_str(&results_str).context("Failed to parse results JSON")?;

        Ok(results)
    }

    /// Get the base directory path
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Clean up old completed jobs (older than specified days)
    pub fn cleanup_old_jobs(&self, days: u32) -> Result<Vec<JobId>> {
        use chrono::Duration;

        let cutoff = chrono::Utc::now() - Duration::days(days as i64);
        let mut deleted_jobs = Vec::new();

        for job_id in self.list_jobs()? {
            if let Ok(job) = self.load_job(&job_id) {
                if job.is_terminal() {
                    if let Some(completed_at) = job.completed_at {
                        if completed_at < cutoff {
                            self.delete_job(&job_id)?;
                            deleted_jobs.push(job_id);
                        }
                    }
                }
            }
        }

        Ok(deleted_jobs)
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let job_ids = self.list_jobs()?;
        let total_jobs = job_ids.len();
        let mut total_size: u64 = 0;

        for job_id in &job_ids {
            let job_dir = self.job_dir(job_id);
            if let Ok(size) = dir_size(&job_dir) {
                total_size += size;
            }
        }

        Ok(StorageStats {
            total_jobs,
            total_size_bytes: total_size,
            base_dir: self.base_dir.clone(),
        })
    }
}

impl Default for JobStorage {
    fn default() -> Self {
        match Self::new() {
            Ok(storage) => storage,
            Err(e) => {
                panic!("Failed to create job storage: {}", e);
            }
        }
    }
}

/// Storage statistics
#[derive(Debug)]
pub struct StorageStats {
    pub total_jobs: usize,
    pub total_size_bytes: u64,
    pub base_dir: PathBuf,
}

impl StorageStats {
    /// Get total size in human-readable format
    pub fn size_human(&self) -> String {
        humanize_bytes(self.total_size_bytes)
    }
}

/// Calculate directory size recursively
fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;

    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            size += dir_size(&path)?;
        }
    }

    Ok(size)
}

/// Convert bytes to human-readable format
fn humanize_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(humanize_bytes(500), "500 B");
        assert_eq!(humanize_bytes(1024), "1.00 KB");
        assert_eq!(humanize_bytes(1536), "1.50 KB");
        assert_eq!(humanize_bytes(1048576), "1.00 MB");
        assert_eq!(humanize_bytes(1073741824), "1.00 GB");
    }
}
