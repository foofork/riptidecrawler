//! Ultra-thin LLM HTTP handler (I/O only)
use crate::errors::ApiError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_facade::authorization::AuthorizationContext;
use riptide_facade::facades::{LlmFacade, LlmRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ExecutePromptRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub prompt: String,
    pub model: String,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub max_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct ExecutePromptResponse {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

pub async fn execute_prompt(
    State(facade): State<LlmFacade>,
    authz_ctx: AuthorizationContext,
    Json(req): Json<ExecutePromptRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if req.prompt.is_empty() || req.model.is_empty() {
        return Err(ApiError::invalid_request("prompt and model required"));
    }
    let llm_req = LlmRequest {
        tenant_id: req.tenant_id,
        user_id: req.user_id,
        prompt: req.prompt,
        model: req.model,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        system_prompt: None,
        parameters: HashMap::new(),
    };
    let resp = facade.execute_prompt(llm_req, &authz_ctx).await?;
    Ok((StatusCode::OK, Json(ExecutePromptResponse {
        text: resp.text,
        provider: resp.provider,
        model: resp.model,
        prompt_tokens: resp.usage.prompt_tokens,
        completion_tokens: resp.usage.completion_tokens,
        total_tokens: resp.usage.total_tokens,
    })))
}
