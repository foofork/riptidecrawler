//! TraceFacade - Distributed Tracing Management
//!
//! This facade provides a clean interface for telemetry trace storage and retrieval,
//! orchestrating multi-step workflows with authorization, idempotency, transactional
//! guarantees, and event emission.
//!
//! ## Responsibilities
//!
//! - **Submit Traces**: Store OTLP-compatible trace data with idempotency
//! - **Query Traces**: Retrieve trace metadata and complete trace spans
//! - **Delete Traces**: Remove traces with proper authorization
//! - **Event Emission**: Publish domain events for trace lifecycle
//! - **Authorization**: Enforce tenant scoping and permission checks
//!
//! ## Architecture
//!
//! This facade depends ONLY on port traits (no concrete implementations):
//! - `TelemetryBackend`: Trace storage and retrieval
//! - `TransactionManager`: ACID transaction coordination
//! - `EventBus`: Domain event publishing
//! - `AuthorizationPolicy`: Access control enforcement
//! - `IdempotencyStore`: Duplicate operation prevention
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use riptide_facade::facades::TraceFacade;
//! use riptide_facade::authorization::AuthorizationContext;
//!
//! let facade = TraceFacade::new(
//!     telemetry_backend,
//!     tx_manager,
//!     event_bus,
//!     authz_policies,
//!     idempotency_store,
//! );
//!
//! // Submit trace with automatic idempotency and authorization
//! let trace_id = facade.submit_trace(trace_data, &authz_ctx).await?;
//!
//! // Query traces with tenant scoping
//! let traces = facade.query_traces(query_params, &authz_ctx).await?;
//! ```

use crate::authorization::{AuthorizationContext, AuthorizationPolicy, Resource};
use crate::workflows::TransactionalWorkflow;
use riptide_types::error::{Result as RiptideResult, RiptideError};
use riptide_types::ports::{DomainEvent, EventBus, IdempotencyStore, TransactionManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// use std::time::Duration; // Unused for now
use tracing::{debug, info, instrument};

/// Telemetry backend port trait (defined here since it's domain-specific)
///
/// This trait abstracts trace storage/retrieval, allowing different implementations
/// (OTLP, Jaeger, Tempo, in-memory) to be injected at runtime.
#[async_trait::async_trait]
pub trait TelemetryBackend: Send + Sync {
    /// Store a complete trace with all spans
    async fn store_trace(&self, trace: &TraceData) -> RiptideResult<String>;

    /// Retrieve trace metadata matching query criteria
    async fn list_traces(&self, query: &TraceQuery) -> RiptideResult<Vec<TraceMetadata>>;

    /// Get complete trace data including all spans
    async fn get_trace(&self, trace_id: &str) -> RiptideResult<Option<CompleteTrace>>;

    /// Delete a trace (if supported by backend)
    async fn delete_trace(&self, trace_id: &str) -> RiptideResult<()>;

    /// Health check for backend connectivity
    async fn health_check(&self) -> bool;

    /// Backend type identifier
    fn backend_type(&self) -> &str;
}

/// Trace data submitted by clients (domain model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceData {
    /// Unique trace identifier (hex-encoded)
    pub trace_id: String,

    /// Tenant identifier for multi-tenancy
    pub tenant_id: String,

    /// Service name producing the trace
    pub service_name: String,

    /// Root span data
    pub root_span: SpanData,

    /// Additional spans in the trace
    pub spans: Vec<SpanData>,

    /// Trace-level metadata
    pub metadata: HashMap<String, String>,
}

/// Individual span within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanData {
    /// Span identifier
    pub span_id: String,

    /// Trace this span belongs to
    pub trace_id: String,

    /// Parent span ID (None for root)
    pub parent_span_id: Option<String>,

    /// Operation name
    pub name: String,

    /// Span kind (SERVER, CLIENT, INTERNAL, etc.)
    pub kind: String,

    /// Start timestamp (RFC3339)
    pub start_time: String,

    /// End timestamp (RFC3339)
    pub end_time: String,

    /// Status (OK, ERROR, UNSET)
    pub status: String,

    /// Span attributes
    pub attributes: HashMap<String, String>,

    /// Span events
    pub events: Vec<SpanEvent>,
}

/// Event within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,

    /// Event timestamp (RFC3339)
    pub timestamp: String,

    /// Event attributes
    pub attributes: HashMap<String, String>,
}

/// Query parameters for trace retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceQuery {
    /// Tenant ID for multi-tenancy scoping
    pub tenant_id: String,

    /// Time range in seconds (lookback)
    pub time_range_secs: u64,

    /// Maximum results to return
    pub limit: usize,

    /// Filter by service name
    pub service_filter: Option<String>,

    /// Filter by status
    pub status_filter: Option<String>,

    /// Filter by minimum duration (ms)
    pub min_duration_ms: Option<u64>,
}

/// Trace metadata returned in queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Trace ID
    pub trace_id: String,

    /// Tenant ID
    pub tenant_id: String,

    /// Service name
    pub service_name: String,

    /// Root span ID
    pub root_span_id: String,

    /// Trace start time (RFC3339)
    pub start_time: String,

    /// Total duration in milliseconds
    pub duration_ms: u64,

    /// Number of spans
    pub span_count: usize,

    /// Overall trace status
    pub status: String,

    /// Trace attributes
    pub attributes: HashMap<String, String>,
}

/// Complete trace with all spans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTrace {
    /// Trace metadata
    pub metadata: TraceMetadata,

    /// All spans in the trace
    pub spans: Vec<SpanData>,
}

/// TraceFacade orchestrates trace management workflows
///
/// This facade implements the application layer's trace management use cases,
/// coordinating authorization, idempotency, transactions, and event emission.
pub struct TraceFacade<TM>
where
    TM: TransactionManager,
{
    /// Telemetry backend for trace storage
    telemetry: Arc<dyn TelemetryBackend>,

    /// Transaction manager for ACID guarantees
    #[allow(dead_code)]
    tx_manager: Arc<TM>,

    /// Event bus for domain event publishing
    #[allow(dead_code)]
    event_bus: Arc<dyn EventBus>,

    /// Authorization policies (tenant scoping, RBAC, etc.)
    authz_policies: Vec<Box<dyn AuthorizationPolicy>>,

    /// Idempotency store for duplicate prevention
    #[allow(dead_code)]
    idempotency_store: Arc<dyn IdempotencyStore>,

    /// Transactional workflow orchestrator
    workflow: TransactionalWorkflow<TM>,
}

impl<TM> TraceFacade<TM>
where
    TM: TransactionManager + 'static,
{
    /// Create new TraceFacade with injected dependencies
    ///
    /// # Arguments
    ///
    /// * `telemetry` - Telemetry backend implementation
    /// * `tx_manager` - Transaction manager for ACID operations
    /// * `event_bus` - Event bus for domain events
    /// * `authz_policies` - Authorization policies to enforce
    /// * `idempotency_store` - Idempotency store for duplicate prevention
    ///
    /// # Returns
    ///
    /// New `TraceFacade` instance ready for use
    pub fn new(
        telemetry: Arc<dyn TelemetryBackend>,
        tx_manager: Arc<TM>,
        event_bus: Arc<dyn EventBus>,
        authz_policies: Vec<Box<dyn AuthorizationPolicy>>,
        idempotency_store: Arc<dyn IdempotencyStore>,
    ) -> Self {
        let workflow = TransactionalWorkflow::new(
            tx_manager.clone(),
            event_bus.clone(),
            idempotency_store.clone(),
        );

        Self {
            telemetry,
            tx_manager,
            event_bus,
            authz_policies,
            idempotency_store,
            workflow,
        }
    }

    /// Submit trace with idempotency and authorization
    ///
    /// This method orchestrates the complete trace submission workflow:
    /// 1. Authorization check (tenant scoping)
    /// 2. Idempotency check (prevent duplicates)
    /// 3. Store trace in backend
    /// 4. Emit domain event (trace.submitted)
    /// 5. Release idempotency lock
    ///
    /// # Arguments
    ///
    /// * `trace_data` - Trace data to store
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(trace_id)` - Trace submitted successfully
    /// * `Err(_)` - Authorization failed, duplicate detected, or storage error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let trace_id = facade.submit_trace(trace_data, &authz_ctx).await?;
    /// println!("Trace submitted: {}", trace_id);
    /// ```
    #[instrument(skip(self, trace_data, authz_ctx), fields(trace_id = %trace_data.trace_id, tenant_id = %trace_data.tenant_id))]
    pub async fn submit_trace(
        &self,
        trace_data: TraceData,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<String> {
        debug!("Submitting trace");

        // Step 1: Authorization - enforce tenant scoping
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "trace".to_string(),
                resource_id: trace_data.trace_id.clone(),
            },
        )?;

        // Verify tenant ID matches authorization context
        if trace_data.tenant_id != authz_ctx.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Trace tenant_id does not match authorization context".to_string(),
            ));
        }

        // Step 2-5: Execute transactional workflow with idempotency
        let trace_id = trace_data.trace_id.clone();
        let idem_key = format!("trace:submit:{}", trace_id);

        let result = self
            .workflow
            .execute(&idem_key, |_tx| {
                let telemetry = self.telemetry.clone();
                let trace_data = trace_data.clone();
                let trace_id = trace_id.clone();

                Box::pin(async move {
                    // Store trace in backend
                    let stored_id = telemetry.store_trace(&trace_data).await?;

                    // Prepare domain event
                    let event = DomainEvent::new(
                        "trace.submitted",
                        trace_id.clone(),
                        serde_json::json!({
                            "trace_id": stored_id,
                            "tenant_id": trace_data.tenant_id,
                            "service_name": trace_data.service_name,
                            "span_count": trace_data.spans.len() + 1, // +1 for root span
                        }),
                    );

                    Ok((stored_id, vec![event]))
                })
            })
            .await?;

        info!(trace_id = %result, "Trace submitted successfully");
        Ok(result)
    }

    /// Query traces with authorization and tenant scoping
    ///
    /// This method retrieves trace metadata matching the query criteria,
    /// automatically filtering by tenant ID from the authorization context.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for trace retrieval
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<TraceMetadata>)` - List of matching traces
    /// * `Err(_)` - Authorization failed or query error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let query = TraceQuery {
    ///     tenant_id: "tenant1".to_string(),
    ///     time_range_secs: 3600,
    ///     limit: 100,
    ///     service_filter: Some("api".to_string()),
    ///     status_filter: None,
    ///     min_duration_ms: Some(100),
    /// };
    /// let traces = facade.query_traces(query, &authz_ctx).await?;
    /// ```
    #[instrument(skip(self, query, authz_ctx), fields(tenant_id = %query.tenant_id))]
    pub async fn query_traces(
        &self,
        query: TraceQuery,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<Vec<TraceMetadata>> {
        debug!("Querying traces");

        // Authorization - enforce tenant scoping
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "trace_query".to_string(),
                resource_id: query.tenant_id.clone(),
            },
        )?;

        // Verify tenant ID matches authorization context
        if query.tenant_id != authz_ctx.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Query tenant_id does not match authorization context".to_string(),
            ));
        }

        // Retrieve traces from backend
        let traces = self.telemetry.list_traces(&query).await?;

        // Apply additional filters if specified
        let filtered: Vec<TraceMetadata> = traces
            .into_iter()
            .filter(|t| {
                // Status filter
                if let Some(ref status) = query.status_filter {
                    if &t.status != status {
                        return false;
                    }
                }

                // Min duration filter
                if let Some(min_duration) = query.min_duration_ms {
                    if t.duration_ms < min_duration {
                        return false;
                    }
                }

                true
            })
            .collect();

        info!(trace_count = filtered.len(), "Query completed successfully");
        Ok(filtered)
    }

    /// Get complete trace data with authorization
    ///
    /// Retrieves full trace data including all spans and events.
    ///
    /// # Arguments
    ///
    /// * `trace_id` - Trace identifier
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(Some(CompleteTrace))` - Trace found and authorized
    /// * `Ok(None)` - Trace not found
    /// * `Err(_)` - Authorization failed or retrieval error
    #[instrument(skip(self, authz_ctx), fields(trace_id = %trace_id))]
    pub async fn get_trace(
        &self,
        trace_id: &str,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<Option<CompleteTrace>> {
        debug!("Getting trace");

        // Authorization check
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "trace".to_string(),
                resource_id: trace_id.to_string(),
            },
        )?;

        // Retrieve trace
        let trace = self.telemetry.get_trace(trace_id).await?;

        // Verify tenant scoping if trace found
        if let Some(ref t) = trace {
            if t.metadata.tenant_id != authz_ctx.tenant_id {
                return Err(RiptideError::PermissionDenied(
                    "Trace belongs to different tenant".to_string(),
                ));
            }
        }

        Ok(trace)
    }

    /// Delete trace with authorization
    ///
    /// Removes trace from backend storage with proper authorization checks
    /// and event emission.
    ///
    /// # Arguments
    ///
    /// * `trace_id` - Trace identifier to delete
    /// * `authz_ctx` - Authorization context
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Trace deleted successfully
    /// * `Err(_)` - Authorization failed or deletion error
    #[instrument(skip(self, authz_ctx), fields(trace_id = %trace_id))]
    pub async fn delete_trace(
        &self,
        trace_id: &str,
        authz_ctx: &AuthorizationContext,
    ) -> RiptideResult<()> {
        debug!("Deleting trace");

        // Authorization check - require delete permission
        self.authorize(
            authz_ctx,
            &Resource::Custom {
                resource_type: "trace".to_string(),
                resource_id: trace_id.to_string(),
            },
        )?;

        // Verify user has delete permission
        if !authz_ctx.has_permission("delete:traces") {
            return Err(RiptideError::PermissionDenied(
                "delete:traces permission required".to_string(),
            ));
        }

        // Retrieve trace first to verify tenant and get metadata
        let trace = self
            .telemetry
            .get_trace(trace_id)
            .await?
            .ok_or_else(|| RiptideError::NotFound(format!("Trace not found: {}", trace_id)))?;

        // Verify tenant scoping
        if trace.metadata.tenant_id != authz_ctx.tenant_id {
            return Err(RiptideError::PermissionDenied(
                "Cannot delete trace from different tenant".to_string(),
            ));
        }

        // Execute deletion in transactional workflow
        let idem_key = format!("trace:delete:{}", trace_id);
        let tenant_id = trace.metadata.tenant_id.clone();

        self.workflow
            .execute(&idem_key, |_tx| {
                let telemetry = self.telemetry.clone();
                let trace_id = trace_id.to_string();
                let tenant_id = tenant_id.clone();

                Box::pin(async move {
                    // Delete from backend
                    telemetry.delete_trace(&trace_id).await?;

                    // Emit deletion event
                    let event = DomainEvent::new(
                        "trace.deleted",
                        trace_id.clone(),
                        serde_json::json!({
                            "trace_id": trace_id,
                            "tenant_id": tenant_id,
                        }),
                    );

                    Ok(((), vec![event]))
                })
            })
            .await?;

        info!(trace_id = %trace_id, "Trace deleted successfully");
        Ok(())
    }

    /// Health check for trace backend
    ///
    /// Verifies backend connectivity and returns health status.
    ///
    /// # Returns
    ///
    /// * `true` - Backend is healthy
    /// * `false` - Backend is unhealthy or unreachable
    pub async fn health_check(&self) -> bool {
        self.telemetry.health_check().await
    }

    /// Get backend type identifier
    ///
    /// # Returns
    ///
    /// Backend type string (e.g., "jaeger", "tempo", "in-memory")
    pub fn backend_type(&self) -> &str {
        self.telemetry.backend_type()
    }

    // Private helper: Run all authorization policies
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        for policy in &self.authz_policies {
            policy.authorize(ctx, resource)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::policies::TenantScopingPolicy;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;

    // Mock implementations for testing

    struct MockTelemetryBackend {
        traces: Arc<Mutex<HashMap<String, CompleteTrace>>>,
    }

    impl MockTelemetryBackend {
        fn new() -> Self {
            Self {
                traces: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl TelemetryBackend for MockTelemetryBackend {
        async fn store_trace(&self, trace: &TraceData) -> RiptideResult<String> {
            let complete_trace = CompleteTrace {
                metadata: TraceMetadata {
                    trace_id: trace.trace_id.clone(),
                    tenant_id: trace.tenant_id.clone(),
                    service_name: trace.service_name.clone(),
                    root_span_id: trace.root_span.span_id.clone(),
                    start_time: trace.root_span.start_time.clone(),
                    duration_ms: 100,
                    span_count: trace.spans.len() + 1,
                    status: "OK".to_string(),
                    attributes: trace.metadata.clone(),
                },
                spans: {
                    let mut spans = vec![trace.root_span.clone()];
                    spans.extend(trace.spans.clone());
                    spans
                },
            };

            self.traces
                .lock()
                .await
                .insert(trace.trace_id.clone(), complete_trace);
            Ok(trace.trace_id.clone())
        }

        async fn list_traces(&self, query: &TraceQuery) -> RiptideResult<Vec<TraceMetadata>> {
            let traces = self.traces.lock().await;
            let results: Vec<TraceMetadata> = traces
                .values()
                .filter(|t| t.metadata.tenant_id == query.tenant_id)
                .map(|t| t.metadata.clone())
                .collect();
            Ok(results)
        }

        async fn get_trace(&self, trace_id: &str) -> RiptideResult<Option<CompleteTrace>> {
            let traces = self.traces.lock().await;
            Ok(traces.get(trace_id).cloned())
        }

        async fn delete_trace(&self, trace_id: &str) -> RiptideResult<()> {
            self.traces.lock().await.remove(trace_id);
            Ok(())
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn backend_type(&self) -> &str {
            "mock"
        }
    }

    struct MockTransactionManager;
    struct MockTransaction(String);

    #[async_trait::async_trait]
    impl TransactionManager for MockTransactionManager {
        type Transaction = MockTransaction;

        async fn begin(&self) -> RiptideResult<Self::Transaction> {
            Ok(MockTransaction(uuid::Uuid::new_v4().to_string()))
        }

        async fn commit(&self, _tx: MockTransaction) -> RiptideResult<()> {
            Ok(())
        }

        async fn rollback(&self, _tx: MockTransaction) -> RiptideResult<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl riptide_types::ports::Transaction for MockTransaction {
        fn id(&self) -> &str {
            &self.0
        }

        async fn execute<F, T>(&mut self, f: F) -> RiptideResult<T>
        where
            F: FnOnce() -> RiptideResult<T> + Send,
            T: Send,
        {
            f()
        }
    }

    struct MockEventBus;

    #[async_trait::async_trait]
    impl EventBus for MockEventBus {
        async fn publish(&self, _event: DomainEvent) -> riptide_types::error::Result<()> {
            Ok(())
        }

        async fn subscribe(
            &self,
            _handler: Arc<dyn riptide_types::ports::EventHandler>,
        ) -> riptide_types::error::Result<riptide_types::ports::SubscriptionId> {
            Ok(uuid::Uuid::new_v4().to_string())
        }

        async fn unsubscribe(&self, _subscription_id: &str) -> riptide_types::error::Result<()> {
            Ok(())
        }
    }

    struct MockIdempotencyStore;

    #[async_trait::async_trait]
    impl IdempotencyStore for MockIdempotencyStore {
        async fn try_acquire(
            &self,
            key: &str,
            ttl: Duration,
        ) -> RiptideResult<riptide_types::ports::IdempotencyToken> {
            Ok(riptide_types::ports::IdempotencyToken::new(
                key.to_string(),
                ttl,
            ))
        }

        async fn release(
            &self,
            _token: riptide_types::ports::IdempotencyToken,
        ) -> RiptideResult<()> {
            Ok(())
        }

        async fn exists(&self, _key: &str) -> RiptideResult<bool> {
            Ok(false)
        }
    }

    fn create_test_facade() -> TraceFacade<MockTransactionManager> {
        let telemetry = Arc::new(MockTelemetryBackend::new());
        let tx_manager = Arc::new(MockTransactionManager);
        let event_bus = Arc::new(MockEventBus) as Arc<dyn EventBus>;
        let authz_policies: Vec<Box<dyn AuthorizationPolicy>> =
            vec![Box::new(TenantScopingPolicy::new())];
        let idempotency_store = Arc::new(MockIdempotencyStore) as Arc<dyn IdempotencyStore>;

        TraceFacade::new(
            telemetry,
            tx_manager,
            event_bus,
            authz_policies,
            idempotency_store,
        )
    }

    fn create_test_trace_data() -> TraceData {
        TraceData {
            trace_id: "abc123".to_string(),
            tenant_id: "tenant1".to_string(),
            service_name: "test-service".to_string(),
            root_span: SpanData {
                span_id: "span1".to_string(),
                trace_id: "abc123".to_string(),
                parent_span_id: None,
                name: "root".to_string(),
                kind: "SERVER".to_string(),
                start_time: "2024-01-01T00:00:00Z".to_string(),
                end_time: "2024-01-01T00:00:01Z".to_string(),
                status: "OK".to_string(),
                attributes: HashMap::new(),
                events: vec![],
            },
            spans: vec![],
            metadata: HashMap::new(),
        }
    }

    fn create_test_authz_ctx() -> AuthorizationContext {
        AuthorizationContext::new(
            "user1",
            "tenant1",
            vec!["admin"],
            HashSet::from([
                "read:traces".to_string(),
                "write:traces".to_string(),
                "delete:traces".to_string(),
            ]),
        )
    }

    #[tokio::test]
    async fn test_submit_trace_success() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.submit_trace(trace_data, &authz_ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "abc123");
    }

    #[tokio::test]
    async fn test_submit_trace_tenant_mismatch() {
        let facade = create_test_facade();
        let mut trace_data = create_test_trace_data();
        trace_data.tenant_id = "different-tenant".to_string();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.submit_trace(trace_data, &authz_ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_query_traces_success() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = create_test_authz_ctx();

        // Submit trace first
        facade.submit_trace(trace_data, &authz_ctx).await.unwrap();

        // Query traces
        let query = TraceQuery {
            tenant_id: "tenant1".to_string(),
            time_range_secs: 3600,
            limit: 10,
            service_filter: None,
            status_filter: None,
            min_duration_ms: None,
        };

        let result = facade.query_traces(query, &authz_ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_query_traces_with_filters() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = create_test_authz_ctx();

        facade.submit_trace(trace_data, &authz_ctx).await.unwrap();

        // Query with status filter
        let query = TraceQuery {
            tenant_id: "tenant1".to_string(),
            time_range_secs: 3600,
            limit: 10,
            service_filter: None,
            status_filter: Some("OK".to_string()),
            min_duration_ms: None,
        };

        let result = facade.query_traces(query, &authz_ctx).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_trace_success() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = create_test_authz_ctx();

        facade
            .submit_trace(trace_data.clone(), &authz_ctx)
            .await
            .unwrap();

        let result = facade.get_trace("abc123", &authz_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_get_trace_not_found() {
        let facade = create_test_facade();
        let authz_ctx = create_test_authz_ctx();

        let result = facade.get_trace("nonexistent", &authz_ctx).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_trace_success() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = create_test_authz_ctx();

        facade
            .submit_trace(trace_data.clone(), &authz_ctx)
            .await
            .unwrap();

        let result = facade.delete_trace("abc123", &authz_ctx).await;
        assert!(result.is_ok());

        // Verify trace deleted
        let get_result = facade.get_trace("abc123", &authz_ctx).await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_trace_without_permission() {
        let facade = create_test_facade();
        let trace_data = create_test_trace_data();
        let authz_ctx = AuthorizationContext::new(
            "user1",
            "tenant1",
            vec!["viewer"],
            HashSet::from(["read:traces".to_string()]), // No delete permission
        );

        facade
            .submit_trace(trace_data, &create_test_authz_ctx())
            .await
            .unwrap();

        let result = facade.delete_trace("abc123", &authz_ctx).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            RiptideError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn test_health_check() {
        let facade = create_test_facade();
        assert!(facade.health_check().await);
    }

    #[tokio::test]
    async fn test_backend_type() {
        let facade = create_test_facade();
        assert_eq!(facade.backend_type(), "mock");
    }
}
