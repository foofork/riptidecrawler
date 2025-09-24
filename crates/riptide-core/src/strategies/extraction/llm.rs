//! LLM-based extraction strategy (hook-based, disabled by default)

use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::process::Command;
use crate::strategies::{ExtractedContent, extraction::*};

pub struct LlmExtractor {
    model: Option<String>,
    prompt_template: Option<String>,
    enabled: bool,
}

impl LlmExtractor {
    pub fn new(model: Option<String>, prompt_template: Option<String>, enabled: bool) -> Self {
        Self {
            model,
            prompt_template,
            enabled,
        }
    }
}

impl ContentExtractor for LlmExtractor {
    async fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent> {
        if !self.enabled {
            return Err(anyhow!("LLM extraction is disabled"));
        }

        // Use external hook for LLM processing
        let result = call_llm_hook(html, url, &self.model, &self.prompt_template).await?;

        Ok(ExtractedContent {
            title: result.title.unwrap_or_else(|| "Untitled".to_string()),
            content: result.content,
            summary: result.summary,
            url: url.to_string(),
            strategy_used: "llm".to_string(),
            extraction_confidence: result.confidence.unwrap_or(0.8),
        })
    }

    fn confidence_score(&self, _html: &str) -> f64 {
        if self.enabled {
            0.9 // High confidence for LLM when enabled
        } else {
            0.0 // No confidence when disabled
        }
    }

    fn strategy_name(&self) -> &'static str {
        "llm"
    }
}

#[derive(Debug, Deserialize)]
struct LlmResult {
    title: Option<String>,
    content: String,
    summary: Option<String>,
    confidence: Option<f64>,
}

/// Call external LLM hook for processing
async fn call_llm_hook(
    html: &str,
    url: &str,
    model: &Option<String>,
    prompt_template: &Option<String>,
) -> Result<LlmResult> {
    // Create temporary file for HTML content
    use std::fs;
    use uuid::Uuid;

    let temp_id = Uuid::new_v4();
    let temp_file = format!("/tmp/riptide_llm_{}.html", temp_id);

    fs::write(&temp_file, html)?;

    // Prepare command arguments
    let mut args = vec![
        "extract".to_string(),
        "--file".to_string(),
        temp_file.clone(),
        "--url".to_string(),
        url.to_string(),
        "--format".to_string(),
        "json".to_string(),
    ];

    if let Some(model_name) = model {
        args.extend(vec!["--model".to_string(), model_name.clone()]);
    }

    if let Some(template) = prompt_template {
        args.extend(vec!["--template".to_string(), template.clone()]);
    }

    // Execute hook command
    let output = Command::new("riptide-llm-hook")
        .args(&args)
        .output();

    // Clean up temp file
    let _ = fs::remove_file(&temp_file);

    match output {
        Ok(output) => {
            if output.status.success() {
                let result_str = String::from_utf8_lossy(&output.stdout);
                let result: LlmResult = serde_json::from_str(&result_str)
                    .map_err(|e| anyhow!("Failed to parse LLM result: {}", e))?;
                Ok(result)
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow!("LLM hook failed: {}", error))
            }
        }
        Err(e) => {
            // If hook is not available, provide fallback
            tracing::warn!("LLM hook not available: {}", e);
            Ok(LlmResult {
                title: Some("LLM Hook Unavailable".to_string()),
                content: "LLM extraction hook is not installed or configured.".to_string(),
                summary: Some("LLM extraction requires external hook installation.".to_string()),
                confidence: Some(0.1),
            })
        }
    }
}

/// Default prompt template for content extraction
pub fn default_prompt_template() -> String {
    r#"
Extract the following information from the given HTML content:

1. Title: The main title or headline of the content
2. Content: The main body text, cleaned and formatted
3. Summary: A brief summary or description (if available)

Please respond in JSON format:
{
    "title": "extracted title",
    "content": "main content text",
    "summary": "brief summary",
    "confidence": 0.85
}

HTML Content:
{html}

URL: {url}
"#.to_string()
}

/// Direct extraction function
pub async fn extract(
    html: &str,
    url: &str,
    model: Option<&str>,
    prompt_template: Option<&str>,
) -> Result<ExtractedContent> {
    let extractor = LlmExtractor::new(
        model.map(|s| s.to_string()),
        prompt_template.map(|s| s.to_string()),
        true, // Enabled for direct calls
    );
    extractor.extract(html, url).await
}

/// Check if LLM hook is available
pub async fn is_hook_available() -> bool {
    Command::new("riptide-llm-hook")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}