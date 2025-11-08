//! BusinessMetrics integration for PipelineFacade
//!
//! This module extends PipelineFacade with business metrics capabilities.

use super::pipeline::{Pipeline, PipelineFacade, PipelineResult, StageStatus};
use crate::error::RiptideResult;
use crate::metrics::BusinessMetrics;
use std::sync::Arc;

/// Wrapper for PipelineFacade with integrated metrics
pub struct MetricsPipelineFacade {
    facade: Arc<PipelineFacade>,
    metrics: Arc<BusinessMetrics>,
}

impl MetricsPipelineFacade {
    /// Create a new metrics-enabled pipeline facade
    pub fn new(facade: PipelineFacade, metrics: Arc<BusinessMetrics>) -> Self {
        Self {
            facade: Arc::new(facade),
            metrics,
        }
    }

    /// Execute a pipeline (automatically records metrics)
    pub async fn execute(&self, pipeline: Pipeline) -> RiptideResult<PipelineResult> {
        let result = self.facade.execute(pipeline).await;

        // Record stage-level metrics
        if let Ok(pipeline_result) = &result {
            for stage_result in &pipeline_result.stage_results {
                let success = matches!(
                    stage_result.status,
                    StageStatus::Success | StageStatus::CachedSuccess
                );
                self.metrics
                    .record_pipeline_stage(&stage_result.stage_name, success);
            }
        }

        result
    }

    /// Get reference to underlying facade
    pub fn facade(&self) -> &PipelineFacade {
        &self.facade
    }

    /// Get metrics reference
    pub fn metrics(&self) -> &BusinessMetrics {
        &self.metrics
    }
}
