use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct RenderReq {
    pub url: String,
    pub wait_for: Option<String>, // css selector
    pub scroll_steps: Option<u32>,
}

#[derive(Serialize)]
pub struct RenderResp {
    pub final_url: String,
    pub html: String,
    pub screenshot_b64: Option<String>,
}

#[derive(Serialize)]
pub struct RenderErrorResp {
    pub error: String,
    pub request_id: Option<String>,
    pub duration_ms: u64,
}
