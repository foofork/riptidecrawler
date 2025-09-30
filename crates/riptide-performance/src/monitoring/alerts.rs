//! Performance alert types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub metric: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
}