//! BusinessMetrics integration for ExtractionFacade
//!
//! This module extends UrlExtractionFacade with business metrics capabilities.

use super::extraction::{ExtractedDoc, Result, UrlExtractionFacade, UrlExtractionOptions};
use crate::metrics::BusinessMetrics;
use std::sync::Arc;
use std::time::Instant;

/// Extension trait for adding metrics to UrlExtractionFacade
pub trait ExtractionMetricsExt {
    /// Extract content from URL with metrics recording
    fn extract_with_metrics(
        &self,
        url: &str,
        options: UrlExtractionOptions,
        metrics: Arc<BusinessMetrics>,
    ) -> impl std::future::Future<Output = Result<ExtractedDoc>> + Send;

    /// Extract from HTML with metrics recording
    fn extract_html_with_metrics(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
        metrics: Arc<BusinessMetrics>,
    ) -> impl std::future::Future<Output = Result<ExtractedDoc>> + Send;
}

impl ExtractionMetricsExt for UrlExtractionFacade {
    async fn extract_with_metrics(
        &self,
        url: &str,
        options: UrlExtractionOptions,
        metrics: Arc<BusinessMetrics>,
    ) -> Result<ExtractedDoc> {
        let start = Instant::now();

        // Perform extraction
        let result = self.extract_from_url(url, options).await;

        // Record metrics
        let duration = start.elapsed();
        let success = result.is_ok();
        metrics.record_extraction_completed(duration, success);

        result
    }

    async fn extract_html_with_metrics(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
        metrics: Arc<BusinessMetrics>,
    ) -> Result<ExtractedDoc> {
        let start = Instant::now();

        // Perform extraction
        let result = self.extract_from_html(url, html, options).await;

        // Record metrics
        let duration = start.elapsed();
        let success = result.is_ok();
        metrics.record_extraction_completed(duration, success);

        result
    }
}

/// Wrapper for UrlExtractionFacade with integrated metrics
pub struct MetricsExtractionFacade {
    facade: UrlExtractionFacade,
    metrics: Arc<BusinessMetrics>,
}

impl MetricsExtractionFacade {
    /// Create a new metrics-enabled extraction facade
    pub fn new(facade: UrlExtractionFacade, metrics: Arc<BusinessMetrics>) -> Self {
        Self { facade, metrics }
    }

    /// Extract content from URL (automatically records metrics)
    pub async fn extract_from_url(
        &self,
        url: &str,
        options: UrlExtractionOptions,
    ) -> Result<ExtractedDoc> {
        self.facade
            .extract_with_metrics(url, options, self.metrics.clone())
            .await
    }

    /// Extract from HTML (automatically records metrics)
    pub async fn extract_from_html(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
    ) -> Result<ExtractedDoc> {
        self.facade
            .extract_html_with_metrics(url, html, options, self.metrics.clone())
            .await
    }
}
