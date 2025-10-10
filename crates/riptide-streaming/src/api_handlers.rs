//! API handlers for report generation endpoints
//!
//! This module provides HTTP handlers for the report generation API.

use crate::reports::{ReportConfig, ReportFormat, ReportGenerator, ReportTheme};
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request body for report generation
#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateReportRequest {
    /// Extraction job ID
    pub job_id: String,

    /// Report format (html, json, csv, pdf)
    #[serde(default = "default_format")]
    pub format: String,

    /// Include charts in report
    #[serde(default = "default_true")]
    pub include_charts: bool,

    /// Report sections to include
    #[serde(default)]
    pub sections: Vec<String>,

    /// Report title (optional)
    pub title: Option<String>,

    /// Report theme (light, dark, corporate, modern)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Include raw data in report
    #[serde(default)]
    pub include_raw_data: bool,

    /// Include metadata in report
    #[serde(default = "default_true")]
    pub include_metadata: bool,

    /// Chart dimensions
    pub chart_width: Option<u32>,
    pub chart_height: Option<u32>,
}

/// Response for successful report generation
#[derive(Debug, Serialize)]
pub struct GenerateReportResponse {
    /// Success status
    pub success: bool,

    /// Report ID
    pub report_id: String,

    /// Report format
    pub format: String,

    /// Report size in bytes
    pub size_bytes: usize,

    /// Generation timestamp
    pub generated_at: String,

    /// Download URL (for future use)
    pub download_url: Option<String>,

    /// Report data (embedded for small reports)
    pub data: Option<String>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<String>,
}

fn default_format() -> String {
    "html".to_string()
}

fn default_theme() -> String {
    "modern".to_string()
}

fn default_true() -> bool {
    true
}

/// Parse report format from string
fn parse_format(format: &str) -> Result<ReportFormat, String> {
    match format.to_lowercase().as_str() {
        "html" => Ok(ReportFormat::Html),
        "json" => Ok(ReportFormat::Json),
        "csv" => Ok(ReportFormat::Csv),
        "pdf" => Ok(ReportFormat::Pdf),
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

/// Parse report theme from string
fn parse_theme(theme: &str) -> ReportTheme {
    match theme.to_lowercase().as_str() {
        "light" => ReportTheme::Light,
        "dark" => ReportTheme::Dark,
        "corporate" => ReportTheme::Corporate,
        "modern" | _ => ReportTheme::Modern,
    }
}

/// Generate a report for an extraction job
///
/// POST /api/reports/generate
///
/// Request body:
/// ```json
/// {
///   "job_id": "extraction-uuid",
///   "format": "html",
///   "include_charts": true,
///   "sections": ["summary", "details", "charts"],
///   "title": "My Extraction Report",
///   "theme": "modern"
/// }
/// ```
pub async fn generate_report(
    Json(request): Json<GenerateReportRequest>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Parse format
    let format = parse_format(&request.format).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid format".to_string(),
                details: Some(e),
            }),
        )
    })?;

    // Build report configuration
    let config = ReportConfig {
        title: request
            .title
            .unwrap_or_else(|| "RipTide Extraction Report".to_string()),
        include_charts: request.include_charts,
        include_raw_data: request.include_raw_data,
        include_metadata: request.include_metadata,
        chart_width: request.chart_width.unwrap_or(800),
        chart_height: request.chart_height.unwrap_or(400),
        theme: parse_theme(&request.theme),
    };

    // Create generator
    let generator = ReportGenerator::with_config(config);

    // Generate report
    let report_data = generator
        .generate_report(&request.job_id, format.clone())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    success: false,
                    error: "Report generation failed".to_string(),
                    details: Some(e.to_string()),
                }),
            )
        })?;

    // Create response
    let response = GenerateReportResponse {
        success: true,
        report_id: uuid::Uuid::new_v4().to_string(),
        format: request.format.clone(),
        size_bytes: report_data.len(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        download_url: None,
        data: if request.format == "json" || request.format == "csv" {
            Some(String::from_utf8_lossy(&report_data).to_string())
        } else {
            None
        },
    };

    // Return appropriate response based on format
    match request.format.as_str() {
        "html" => Ok((
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            report_data,
        )
            .into_response()),
        "json" => Ok((
            StatusCode::OK,
            [(
                axum::http::header::CONTENT_TYPE,
                "application/json; charset=utf-8",
            )],
            report_data,
        )
            .into_response()),
        "csv" => Ok((
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")],
            report_data,
        )
            .into_response()),
        "pdf" => Ok((
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/pdf")],
            report_data,
        )
            .into_response()),
        _ => Ok((StatusCode::OK, Json(response)).into_response()),
    }
}

/// Get available report formats
///
/// GET /api/reports/formats
pub async fn list_formats() -> Json<HashMap<String, Vec<String>>> {
    let mut formats = HashMap::new();
    formats.insert(
        "formats".to_string(),
        vec![
            "html".to_string(),
            "json".to_string(),
            "csv".to_string(),
            "pdf".to_string(),
        ],
    );
    formats.insert(
        "themes".to_string(),
        vec![
            "light".to_string(),
            "dark".to_string(),
            "corporate".to_string(),
            "modern".to_string(),
        ],
    );
    Json(formats)
}

/// Get report configuration defaults
///
/// GET /api/reports/config/defaults
pub async fn get_default_config() -> Json<ReportConfig> {
    Json(ReportConfig::default())
}

/// Health check for reports API
///
/// GET /api/reports/health
pub async fn health_check() -> Json<HashMap<String, String>> {
    let mut health = HashMap::new();
    health.insert("status".to_string(), "healthy".to_string());
    health.insert("service".to_string(), "reports".to_string());
    health.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    Json(health)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_format() {
        assert!(matches!(parse_format("html"), Ok(ReportFormat::Html)));
        assert!(matches!(parse_format("json"), Ok(ReportFormat::Json)));
        assert!(matches!(parse_format("csv"), Ok(ReportFormat::Csv)));
        assert!(matches!(parse_format("pdf"), Ok(ReportFormat::Pdf)));
        assert!(parse_format("invalid").is_err());
    }

    #[test]
    fn test_parse_theme() {
        assert!(matches!(parse_theme("light"), ReportTheme::Light));
        assert!(matches!(parse_theme("dark"), ReportTheme::Dark));
        assert!(matches!(parse_theme("corporate"), ReportTheme::Corporate));
        assert!(matches!(parse_theme("modern"), ReportTheme::Modern));
        assert!(matches!(parse_theme("invalid"), ReportTheme::Modern));
    }

    #[tokio::test]
    async fn test_list_formats() {
        let response = list_formats().await;
        let formats = response.0;

        assert!(formats.contains_key("formats"));
        assert!(formats.contains_key("themes"));

        let format_list = &formats["formats"];
        assert!(format_list.contains(&"html".to_string()));
        assert!(format_list.contains(&"json".to_string()));
        assert!(format_list.contains(&"csv".to_string()));
        assert!(format_list.contains(&"pdf".to_string()));
    }

    #[tokio::test]
    async fn test_get_default_config() {
        let response = get_default_config().await;
        let config = response.0;

        assert_eq!(config.title, "RipTide Extraction Report");
        assert!(config.include_charts);
        assert!(config.include_metadata);
        assert!(!config.include_raw_data);
        assert_eq!(config.chart_width, 800);
        assert_eq!(config.chart_height, 400);
    }

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let health = response.0;

        assert_eq!(health.get("status"), Some(&"healthy".to_string()));
        assert_eq!(health.get("service"), Some(&"reports".to_string()));
        assert!(health.contains_key("version"));
    }
}
