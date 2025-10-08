//! Flamegraph generation for performance analysis

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Flamegraph generator for CPU and memory profiling
pub struct FlamegraphGenerator {
    session_id: Uuid,
    output_dir: PathBuf,
    profile_data: Option<Vec<u8>>,
    is_recording: bool,
}

impl FlamegraphGenerator {
    /// Create a new flamegraph generator
    pub fn new(session_id: Uuid) -> Result<Self> {
        let output_dir = std::env::temp_dir().join(format!("riptide_flamegraphs_{}", session_id));
        std::fs::create_dir_all(&output_dir)?;

        debug!(
            session_id = %session_id,
            output_dir = ?output_dir,
            "Created flamegraph generator"
        );

        Ok(Self {
            session_id,
            output_dir,
            profile_data: None,
            is_recording: false,
        })
    }

    /// Start recording profile data
    pub async fn start_recording(&mut self) -> Result<()> {
        if self.is_recording {
            warn!("Flamegraph recording already started");
            return Ok(());
        }

        info!(session_id = %self.session_id, "Starting flamegraph recording");

        // Initialize profiling if pprof feature is available
        #[cfg(feature = "memory-profiling")]
        {
            // use pprof::ProfilerGuard;

            // This would start pprof profiling
            // For now, we'll simulate this
            self.profile_data = Some(Vec::new());
        }

        self.is_recording = true;

        debug!(session_id = %self.session_id, "Flamegraph recording started");
        Ok(())
    }

    /// Stop recording and generate flamegraph
    pub async fn stop_recording_and_generate(&mut self) -> Result<String> {
        if !self.is_recording {
            return Err(anyhow::anyhow!("Flamegraph recording not started"));
        }

        info!(session_id = %self.session_id, "Stopping flamegraph recording and generating graph");

        self.is_recording = false;

        // Generate the flamegraph
        let flamegraph_path = self.generate_flamegraph().await?;

        info!(
            session_id = %self.session_id,
            path = %flamegraph_path,
            "Flamegraph generated successfully"
        );

        Ok(flamegraph_path)
    }

    /// Generate CPU flamegraph
    pub async fn generate_cpu_flamegraph(&self) -> Result<String> {
        let output_path = self
            .output_dir
            .join(format!("cpu_flamegraph_{}.svg", self.session_id));

        #[cfg(feature = "memory-profiling")]
        {
            // Use pprof to generate CPU flamegraph
            self.generate_pprof_flamegraph("cpu", &output_path).await?;
        }

        #[cfg(not(feature = "memory-profiling"))]
        {
            // Generate a simple placeholder flamegraph
            self.generate_placeholder_flamegraph(&output_path, "CPU Profile")
                .await?;
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Generate memory flamegraph
    pub async fn generate_memory_flamegraph(&self) -> Result<String> {
        let output_path = self
            .output_dir
            .join(format!("memory_flamegraph_{}.svg", self.session_id));

        #[cfg(feature = "memory-profiling")]
        {
            // Use pprof to generate memory flamegraph
            self.generate_pprof_flamegraph("heap", &output_path).await?;
        }

        #[cfg(not(feature = "memory-profiling"))]
        {
            // Generate a simple placeholder flamegraph
            self.generate_placeholder_flamegraph(&output_path, "Memory Profile")
                .await?;
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Generate allocation flamegraph
    pub async fn generate_allocation_flamegraph(&self) -> Result<String> {
        let output_path = self
            .output_dir
            .join(format!("allocation_flamegraph_{}.svg", self.session_id));

        #[cfg(feature = "memory-profiling")]
        {
            self.generate_pprof_flamegraph("alloc", &output_path)
                .await?;
        }

        #[cfg(not(feature = "memory-profiling"))]
        {
            self.generate_placeholder_flamegraph(&output_path, "Allocation Profile")
                .await?;
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Generate flamegraph using system tools (perf + flamegraph)
    pub async fn generate_system_flamegraph(&self, duration_seconds: u64) -> Result<String> {
        let output_path = self
            .output_dir
            .join(format!("system_flamegraph_{}.svg", self.session_id));

        info!(
            duration = duration_seconds,
            "Generating system flamegraph using perf"
        );

        // Check if perf is available
        if !self.is_perf_available() {
            warn!("perf tool not available, generating placeholder");
            return self
                .generate_placeholder_flamegraph(
                    &output_path,
                    "System Profile (perf not available)",
                )
                .await;
        }

        // Record with perf
        let perf_data_path = self.output_dir.join("perf.data");
        let perf_record = Command::new("perf")
            .args([
                "record",
                "-F",
                "97", // Sample frequency
                "-g", // Call graphs
                "-o",
                perf_data_path.to_str().unwrap(),
                "--",
                "sleep",
                &duration_seconds.to_string(),
            ])
            .output();

        match perf_record {
            Ok(output) if output.status.success() => {
                // Convert perf data to flamegraph
                self.perf_to_flamegraph(&perf_data_path, &output_path)
                    .await?;
            }
            _ => {
                warn!("Failed to run perf record, generating placeholder");
                return self
                    .generate_placeholder_flamegraph(&output_path, "System Profile (perf failed)")
                    .await;
            }
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Check if perf tool is available
    fn is_perf_available(&self) -> bool {
        Command::new("perf")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Convert perf data to flamegraph
    async fn perf_to_flamegraph(&self, perf_data: &Path, output_path: &Path) -> Result<()> {
        // Convert perf.data to folded format
        let folded_path = self.output_dir.join("folded.txt");

        let perf_script = Command::new("perf")
            .args(["script", "-i", perf_data.to_str().unwrap()])
            .output()?;

        if !perf_script.status.success() {
            return Err(anyhow::anyhow!("Failed to run perf script"));
        }

        // Write folded data
        std::fs::write(&folded_path, perf_script.stdout)?;

        // Generate flamegraph
        // Simulate flamegraph.pl check without which dependency
        if false {
            // Disabled for now
            let _output = Command::new("flamegraph.pl")
                .stdin(std::fs::File::open(&folded_path)?)
                .stdout(std::fs::File::create(output_path)?)
                .spawn()?
                .wait()?;
        } else {
            // Use Rust flamegraph if available
            return self.rust_flamegraph(&folded_path, output_path).await;
        }

        Ok(())
    }

    /// Generate flamegraph using Rust flamegraph library
    async fn rust_flamegraph(&self, folded_path: &Path, output_path: &Path) -> Result<()> {
        #[cfg(feature = "bottleneck-analysis")]
        {
            // use flamegraph::{from_lines, Options};

            let folded_content = std::fs::read_to_string(folded_path)?;
            let _lines: Vec<&str> = folded_content.lines().collect();

            // Simplified fallback without flamegraph dependencies
            let flamegraph_svg = format!(
                "<svg>Flamegraph placeholder for {}</svg>",
                folded_path.display()
            );

            std::fs::write(output_path, flamegraph_svg)?;
        }

        #[cfg(not(feature = "bottleneck-analysis"))]
        {
            // Generate placeholder
            self.generate_placeholder_flamegraph(
                &output_path.to_path_buf(),
                "Flamegraph (library not available)",
            )
            .await?;
        }

        Ok(())
    }

    /// Generate flamegraph (main method)
    async fn generate_flamegraph(&self) -> Result<String> {
        // Try different methods in order of preference

        // 1. Try memory flamegraph if we have profile data
        if self.profile_data.is_some() {
            return self.generate_memory_flamegraph().await;
        }

        // 2. Try CPU flamegraph
        match self.generate_cpu_flamegraph().await {
            Ok(path) => return Ok(path),
            Err(e) => debug!("CPU flamegraph failed: {}", e),
        }

        // 3. Try system flamegraph with short duration
        match self.generate_system_flamegraph(5).await {
            Ok(path) => return Ok(path),
            Err(e) => debug!("System flamegraph failed: {}", e),
        }

        // 4. Generate placeholder as fallback
        let output_path = self
            .output_dir
            .join(format!("flamegraph_{}.svg", self.session_id));
        self.generate_placeholder_flamegraph(&output_path, "Performance Profile")
            .await
    }

    /// Generate pprof-based flamegraph
    #[cfg(feature = "memory-profiling")]
    async fn generate_pprof_flamegraph(
        &self,
        profile_type: &str,
        output_path: &PathBuf,
    ) -> Result<()> {
        // use pprof::protos::Message;

        // This is a simplified implementation
        // In a real scenario, you'd use the collected profile data
        let placeholder_svg = format!(
            r##"<svg viewBox="0 0 1200 400" xmlns="http://www.w3.org/2000/svg">
                <rect width="1200" height="400" fill="#ffffcc"/>
                <text x="600" y="200" text-anchor="middle" font-size="20" fill="#333">
                    {} Flamegraph for Session {}
                </text>
                <text x="600" y="240" text-anchor="middle" font-size="14" fill="#666">
                    Profile data would be visualized here
                </text>
            </svg>"##,
            profile_type.to_uppercase(),
            self.session_id
        );

        std::fs::write(output_path, placeholder_svg)?;
        Ok(())
    }

    /// Generate placeholder flamegraph when profiling tools aren't available
    async fn generate_placeholder_flamegraph(
        &self,
        output_path: &PathBuf,
        title: &str,
    ) -> Result<String> {
        let svg_content = format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<svg viewBox="0 0 1200 600" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#ff9999"/>
            <stop offset="50%" stop-color="#ffcc99"/>
            <stop offset="100%" stop-color="#99ff99"/>
        </linearGradient>
    </defs>

    <!-- Background -->
    <rect width="1200" height="600" fill="url(#grad1)" opacity="0.1"/>

    <!-- Title -->
    <text x="600" y="80" text-anchor="middle" font-size="24" font-weight="bold" fill="#333">
        {title}
    </text>

    <!-- Session info -->
    <text x="600" y="120" text-anchor="middle" font-size="16" fill="#666">
        Session: {session_id}
    </text>

    <!-- Simulated flame bars -->
    <rect x="100" y="200" width="1000" height="40" fill="#ff6b6b" opacity="0.8"/>
    <text x="600" y="225" text-anchor="middle" font-size="14" fill="white">main</text>

    <rect x="150" y="250" width="400" height="30" fill="#4ecdc4" opacity="0.8"/>
    <text x="350" y="270" text-anchor="middle" font-size="12" fill="white">processing</text>

    <rect x="600" y="250" width="450" height="30" fill="#45b7d1" opacity="0.8"/>
    <text x="825" y="270" text-anchor="middle" font-size="12" fill="white">network_io</text>

    <rect x="170" y="290" width="180" height="25" fill="#96ceb4" opacity="0.8"/>
    <text x="260" y="307" text-anchor="middle" font-size="10" fill="white">parse_html</text>

    <rect x="370" y="290" width="160" height="25" fill="#feca57" opacity="0.8"/>
    <text x="450" y="307" text-anchor="middle" font-size="10" fill="white">extract_data</text>

    <rect x="620" y="290" width="200" height="25" fill="#ff9ff3" opacity="0.8"/>
    <text x="720" y="307" text-anchor="middle" font-size="10" fill="white">http_request</text>

    <rect x="840" y="290" width="190" height="25" fill="#54a0ff" opacity="0.8"/>
    <text x="935" y="307" text-anchor="middle" font-size="10" fill="white">tcp_connect</text>

    <!-- Legend -->
    <text x="600" y="380" text-anchor="middle" font-size="14" fill="#333">
        Flamegraph visualization showing call stack hierarchy and time spent
    </text>

    <text x="600" y="420" text-anchor="middle" font-size="12" fill="#666">
        Width represents time spent, height represents call stack depth
    </text>

    <!-- Placeholder notice -->
    <rect x="300" y="480" width="600" height="80" fill="#f8f9fa" stroke="#dee2e6" stroke-width="2" rx="10"/>
    <text x="600" y="510" text-anchor="middle" font-size="14" fill="#6c757d">
        Placeholder Flamegraph
    </text>
    <text x="600" y="535" text-anchor="middle" font-size="12" fill="#6c757d">
        Install profiling tools (perf, flamegraph.pl) for real flame graphs
    </text>

    <!-- Timestamp -->
    <text x="1150" y="580" text-anchor="end" font-size="10" fill="#999">
        Generated: {timestamp}
    </text>
</svg>"##,
            title = title,
            session_id = self.session_id,
            timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        std::fs::write(output_path, svg_content)?;

        debug!(
            path = ?output_path,
            "Generated placeholder flamegraph"
        );

        Ok(output_path.to_string_lossy().to_string())
    }

    /// Get flamegraph output directory
    pub fn get_output_dir(&self) -> &PathBuf {
        &self.output_dir
    }

    /// Clean up old flamegraph files
    pub async fn cleanup_old_files(&self, max_age_hours: u64) -> Result<()> {
        let cutoff =
            std::time::SystemTime::now() - std::time::Duration::from_secs(max_age_hours * 3600);

        if let Ok(entries) = std::fs::read_dir(&self.output_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff {
                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                warn!(
                                    path = ?entry.path(),
                                    error = ?e,
                                    "Failed to remove old flamegraph file"
                                );
                            }
                        }
                    }
                }
            }
        }

        debug!("Cleaned up old flamegraph files");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_flamegraph_generator_creation() {
        let session_id = Uuid::new_v4();
        let generator = FlamegraphGenerator::new(session_id).unwrap();
        assert_eq!(generator.session_id, session_id);
        assert!(generator.output_dir.exists());
    }

    #[tokio::test]
    async fn test_placeholder_flamegraph() {
        let session_id = Uuid::new_v4();
        let generator = FlamegraphGenerator::new(session_id).unwrap();

        let output_path = generator.output_dir.join("test.svg");
        let result = generator
            .generate_placeholder_flamegraph(&output_path, "Test")
            .await;

        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains(&session_id.to_string()));
    }

    #[tokio::test]
    async fn test_memory_flamegraph() {
        let session_id = Uuid::new_v4();
        let generator = FlamegraphGenerator::new(session_id).unwrap();

        let result = generator.generate_memory_flamegraph().await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(std::path::Path::new(&path).exists());
    }
}
