//! Ultra-thin LLM HTTP handler (I/O only) - 45 LOC target
use crate::errors::ApiError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use riptide_facade::authorization::AuthorizationContext;
use riptide_facade::facades::{LlmFacade, LlmRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct LlmReq {
    pub tenant_id: String,
    pub user_id: String,
    pub prompt: String,
    pub model: String,
    #[serde(default)]
    pub temp: f32,
    #[serde(default)]
    pub max: usize,
}

#[derive(Serialize)]
pub struct LlmResp {
    pub text: String,
    pub provider: String,
    pub model: String,
    pub tokens: (usize, usize, usize),
}

pub async fn execute(
    State(f): State<LlmFacade>,
    ctx: AuthorizationContext,
    Json(r): Json<LlmReq>,
) -> Result<impl IntoResponse, ApiError> {
    if r.prompt.is_empty() || r.model.is_empty() {
        return Err(ApiError::invalid_request("prompt and model required"));
    }
    let req = LlmRequest {
        tenant_id: r.tenant_id,
        user_id: r.user_id,
        prompt: r.prompt,
        model: r.model,
        temperature: r.temp,
        max_tokens: r.max,
        system_prompt: None,
        parameters: HashMap::new(),
    };
    let res = f.execute_prompt(req, &ctx).await?;
    Ok((StatusCode::OK, Json(LlmResp {
        text: res.text,
        provider: res.provider,
        model: res.model,
        tokens: (res.usage.prompt_tokens, res.usage.completion_tokens, res.usage.total_tokens),
    })))
}
