//! Ultra-thin trace backend HTTP handler
//!
//! This handler contains ONLY HTTP I/O concerns (validation, DTO mapping).
//! ALL business logic resides in TraceFacade (riptide-facade).
//!
//! ## Responsibilities (HTTP Layer Only)
//!
//! - Validate HTTP request format
//! - Map DTOs to domain types
//! - Call facade methods
//! - Map domain responses to DTOs
//! - NO loops, NO business logic, NO multi-step orchestration

use crate::errors::ApiError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_facade::authorization::AuthorizationContext;
use riptide_facade::facades::{TraceData, TraceFacade, TraceQuery};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP DTO: Submit trace request
#[derive(Debug, Deserialize)]
pub struct SubmitTraceRequest {
    pub trace_id: String,
    pub tenant_id: String,
    pub service_name: String,
    pub root_span: SpanDto,
    pub spans: Vec<SpanDto>,
    pub metadata: HashMap<String, String>,
}

/// HTTP DTO: Span data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanDto {
    pub span_id: String,
    pub name: String,
    pub kind: String,
    pub start_time: String,
    pub end_time: String,
    pub status: String,
}

/// Submit trace (POST /traces)
pub async fn submit_trace<TM>(
    State(facade): State<TraceFacade<TM>>,
    authz_ctx: AuthorizationContext,
    Json(req): Json<SubmitTraceRequest>,
) -> Result<impl IntoResponse, ApiError>
where
    TM: riptide_types::ports::TransactionManager + 'static,
{
    // HTTP validation only
    if req.trace_id.is_empty() || req.tenant_id.is_empty() {
        return Err(ApiError::invalid_request("trace_id and tenant_id required"));
    }

    // Map DTO → Domain
    let trace_data = TraceData {
        trace_id: req.trace_id,
        tenant_id: req.tenant_id,
        service_name: req.service_name,
        root_span: req.root_span.into(),
        spans: req.spans.into_iter().map(Into::into).collect(),
        metadata: req.metadata,
    };

    // Call facade (business logic)
    let trace_id = facade.submit_trace(trace_data, &authz_ctx).await?;

    // Map Domain → DTO
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "trace_id": trace_id }))))
}

// Total: 40 LOC (excluding comments/blanks)
