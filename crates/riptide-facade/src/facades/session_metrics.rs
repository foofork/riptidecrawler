//! BusinessMetrics integration for SessionFacade
//!
//! This module extends SessionFacade with business metrics capabilities.

use super::session::{SessionConfig, SessionFacade};
use crate::metrics::BusinessMetrics;
use riptide_types::error::Result as RiptideResult;
use riptide_types::ports::{Clock, IdempotencyStore, Session, SessionStorage};
use std::sync::Arc;

/// Wrapper for SessionFacade with integrated metrics
pub struct MetricsSessionFacade {
    facade: SessionFacade,
    metrics: Arc<BusinessMetrics>,
}

impl MetricsSessionFacade {
    /// Create a new metrics-enabled session facade
    pub fn new(
        storage: Arc<dyn SessionStorage>,
        idempotency: Arc<dyn IdempotencyStore>,
        clock: Arc<dyn Clock>,
        metrics: Arc<BusinessMetrics>,
    ) -> Self {
        Self {
            facade: SessionFacade::new(storage, idempotency, clock),
            metrics,
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        storage: Arc<dyn SessionStorage>,
        idempotency: Arc<dyn IdempotencyStore>,
        clock: Arc<dyn Clock>,
        config: SessionConfig,
        metrics: Arc<BusinessMetrics>,
    ) -> Self {
        Self {
            facade: SessionFacade::with_config(storage, idempotency, clock, config),
            metrics,
        }
    }

    /// Create a new session (automatically records metrics)
    pub async fn create_session(&self, user_id: &str, tenant_id: &str) -> RiptideResult<Session> {
        let result = self.facade.create_session(user_id, tenant_id).await;

        // Record metrics
        if result.is_ok() {
            self.metrics.record_session_created();
        }

        result
    }

    /// Terminate a session (automatically records metrics)
    pub async fn terminate_session(&self, session_id: &str) -> RiptideResult<()> {
        let result = self.facade.terminate_session(session_id).await;

        // Record metrics
        if result.is_ok() {
            self.metrics.record_session_closed();
        }

        result
    }

    /// Get current active session count
    pub async fn active_session_count(&self) -> RiptideResult<i64> {
        let count = self.facade.active_session_count().await?;
        self.metrics.record_session_active(count);
        Ok(count)
    }

    /// Get reference to underlying facade
    pub fn facade(&self) -> &SessionFacade {
        &self.facade
    }
}
