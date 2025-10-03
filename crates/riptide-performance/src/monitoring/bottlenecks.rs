//! Performance bottleneck analysis types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Bottleneck severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Location or component where bottleneck occurs
    pub location: String,
    /// Severity level of the bottleneck
    pub severity: BottleneckSeverity,
    /// Total time spent in this bottleneck
    pub time_spent: Duration,
    /// Percentage of total execution time
    pub percentage_of_total: f64,
    /// Number of times this bottleneck was observed
    pub call_count: u64,
}

/// Bottleneck analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    /// List of identified bottlenecks, sorted by severity
    pub bottlenecks: Vec<Bottleneck>,
    /// Time taken to perform the analysis
    pub analysis_time: Duration,
    /// Actionable recommendations for optimization
    pub recommendations: Vec<String>,
}
