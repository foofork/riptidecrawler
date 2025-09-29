//! Flamegraph generation for performance visualization

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// Flamegraph generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlameGraphConfig {
    pub sampling_frequency: u32, // Hz
    pub output_directory: String,
    pub auto_generate: bool,
    pub generation_interval: Duration,
}

/// Flamegraph generator
pub struct FlameGraphGenerator {
    config: FlameGraphConfig,
    started: Arc<RwLock<bool>>,
    generation_count: Arc<RwLock<u64>>,
}

impl FlameGraphGenerator {
    pub fn new(config: FlameGraphConfig) -> crate::Result<Self> {
        // Ensure output directory exists
        std::fs::create_dir_all(&config.output_directory)
            .map_err(|e| crate::PerformanceError::Io(e))?;

        Ok(Self {
            config,
            started: Arc::new(RwLock::new(false)),
            generation_count: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn start(&self) -> crate::Result<()> {
        if *self.started.read().await {
            return Ok(());
        }

        info!("Starting flamegraph generator");
        *self.started.write().await = true;

        if self.config.auto_generate {
            let interval = self.config.generation_interval;
            let config = self.config.clone();
            let generation_count = Arc::clone(&self.generation_count);
            let started = Arc::clone(&self.started);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(interval);

                while *started.read().await {
                    interval.tick().await;

                    if let Err(e) = Self::generate_flamegraph(&config, &generation_count).await {
                        error!("Failed to generate flamegraph: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    pub async fn stop(&self) -> crate::Result<()> {
        info!("Stopping flamegraph generator");
        *self.started.write().await = false;
        Ok(())
    }

    pub async fn generate_on_demand(&self) -> crate::Result<PathBuf> {
        info!("Generating flamegraph on demand");
        Self::generate_flamegraph(&self.config, &self.generation_count).await
    }

    async fn generate_flamegraph(
        config: &FlameGraphConfig,
        generation_count: &Arc<RwLock<u64>>,
    ) -> crate::Result<PathBuf> {
        let start_time = Instant::now();
        let count = {
            let mut count = generation_count.write().await;
            *count += 1;
            *count
        };

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("flamegraph_{}_{}.svg", timestamp, count);
        let output_path = Path::new(&config.output_directory).join(&filename);

        #[cfg(feature = "flamegraph")]
        {
            Self::generate_with_flamegraph_crate(config, &output_path).await?;
        }

        #[cfg(not(feature = "flamegraph"))]
        {
            Self::generate_mock_flamegraph(&output_path).await?;
        }

        let generation_time = start_time.elapsed();
        info!(
            "Generated flamegraph: {} (took {:?})",
            output_path.display(),
            generation_time
        );

        Ok(output_path)
    }

    #[cfg(feature = "flamegraph")]
    async fn generate_with_flamegraph_crate(
        config: &FlameGraphConfig,
        output_path: &PathBuf,
    ) -> crate::Result<()> {
        use pprof::ProfilerGuard;
        use std::fs::File;

        // Start profiling
        let guard = ProfilerGuard::new(config.sampling_frequency as i32)
            .map_err(|e| crate::PerformanceError::ProfilingError(format!("Failed to start profiler: {}", e)))?;

        // Wait for sampling period (in a real implementation, this would be controlled differently)
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Build the report
        if let Ok(report) = guard.report().build() {
            let file = File::create(output_path)
                .map_err(|e| crate::PerformanceError::Io(e))?;

            // Generate flamegraph
            report.flamegraph(file)
                .map_err(|e| crate::PerformanceError::ProfilingError(format!("Failed to generate flamegraph: {}", e)))?;
        }

        Ok(())
    }

    #[cfg(not(feature = "flamegraph"))]
    async fn generate_mock_flamegraph(output_path: &PathBuf) -> crate::Result<()> {
        // Generate a simple mock SVG flamegraph
        let mock_svg = r#"<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg version="1.1" width="800" height="400" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="background" y1="0%" y2="100%">
      <stop offset="0%" style="stop-color:#eeeeee"/>
      <stop offset="100%" style="stop-color:#cccccc"/>
    </linearGradient>
  </defs>
  <rect width="100%" height="100%" fill="url(#background)"/>
  <text x="400" y="30" text-anchor="middle" font-size="20" font-family="Arial">Mock Flamegraph</text>

  <!-- Mock flame rectangles -->
  <rect x="50" y="50" width="700" height="30" fill="#ff6b6b" stroke="#000" stroke-width="1"/>
  <text x="400" y="70" text-anchor="middle" font-size="12" font-family="Arial">main (100%)</text>

  <rect x="80" y="90" width="300" height="25" fill="#4ecdc4" stroke="#000" stroke-width="1"/>
  <text x="230" y="107" text-anchor="middle" font-size="10" font-family="Arial">spider::crawl (43%)</text>

  <rect x="400" y="90" width="200" height="25" fill="#45b7d1" stroke="#000" stroke-width="1"/>
  <text x="500" y="107" text-anchor="middle" font-size="10" font-family="Arial">html::parse (29%)</text>

  <rect x="620" y="90" width="100" height="25" fill="#f9ca24" stroke="#000" stroke-width="1"/>
  <text x="670" y="107" text-anchor="middle" font-size="10" font-family="Arial">network::fetch (14%)</text>

  <rect x="100" y="125" width="120" height="20" fill="#6c5ce7" stroke="#000" stroke-width="1"/>
  <text x="160" y="138" text-anchor="middle" font-size="9" font-family="Arial">url::parse (17%)</text>

  <rect x="240" y="125" width="100" height="20" fill="#a29bfe" stroke="#000" stroke-width="1"/>
  <text x="290" y="138" text-anchor="middle" font-size="9" font-family="Arial">http::get (14%)</text>

  <text x="400" y="200" text-anchor="middle" font-size="14" font-family="Arial" fill="#666">
    This is a mock flamegraph generated for demonstration purposes.
  </text>
  <text x="400" y="220" text-anchor="middle" font-size="12" font-family="Arial" fill="#666">
    Enable the 'flamegraph' feature for real profiling data.
  </text>
</svg>"#;

        tokio::fs::write(output_path, mock_svg).await
            .map_err(|e| crate::PerformanceError::Io(e))?;

        Ok(())
    }

    pub async fn get_generation_count(&self) -> u64 {
        *self.generation_count.read().await
    }

    pub fn get_output_directory(&self) -> &str {
        &self.config.output_directory
    }

    pub async fn list_generated_flamegraphs(&self) -> crate::Result<Vec<PathBuf>> {
        let mut flamegraphs = Vec::new();
        let dir = tokio::fs::read_dir(&self.config.output_directory).await
            .map_err(|e| crate::PerformanceError::Io(e))?;

        let mut entries = dir;
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| crate::PerformanceError::Io(e))? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                if let Some(filename) = path.file_name() {
                    if filename.to_string_lossy().starts_with("flamegraph_") {
                        flamegraphs.push(path);
                    }
                }
            }
        }

        flamegraphs.sort();
        Ok(flamegraphs)
    }
}