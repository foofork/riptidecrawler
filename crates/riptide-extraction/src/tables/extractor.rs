//! Table extraction logic

use super::types::{TableData, TableExtractRequest, TableExtractResponse, TableSummary};
use anyhow::{Context, Result};
use std::io::Read;

/// Table extractor for HTML content
pub struct TableExtractor;

impl TableExtractor {
    /// Extract tables from HTML content source
    pub async fn extract_from_source(source: TableSource<'_>) -> Result<String> {
        match source {
            TableSource::Stdin => {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .context("Failed to read from stdin")?;
                Ok(buffer)
            }
            TableSource::Url(url) => Self::fetch_from_url(url).await,
            TableSource::File(path) => {
                std::fs::read_to_string(path).context(format!("Failed to read file: {}", path))
            }
            TableSource::Content(content) => Ok(content.to_string()),
        }
    }

    /// Fetch HTML content from URL
    async fn fetch_from_url(url: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .user_agent("RipTide/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to fetch URL")?;

        response.text().await.context("Failed to read response")
    }

    /// Extract table data from API response with full data export
    pub async fn fetch_full_table_data<C: ApiClient>(
        client: &C,
        table_summary: &TableSummary,
        format: &str,
    ) -> Result<TableData> {
        let export_format = match format {
            "json" => "json",
            "csv" => "csv",
            "markdown" => "markdown",
            _ => "markdown",
        };

        let export_url = format!(
            "/api/v1/tables/{}/export?format={}&include_headers=true",
            table_summary.id, export_format
        );

        let content = client.get_text(&export_url).await.context(format!(
            "Failed to fetch table export for ID: {}",
            table_summary.id
        ))?;

        let data =
            super::parser::parse_content_to_table_data(&content, export_format, table_summary)?;

        Ok(TableData {
            id: table_summary.id.clone(),
            rows: table_summary.rows,
            columns: table_summary.columns,
            headers: table_summary.headers.clone(),
            data,
            caption: table_summary.metadata.caption.clone(),
        })
    }

    /// Create extraction request from HTML content
    pub fn create_request(html_content: String) -> TableExtractRequest {
        TableExtractRequest { html_content }
    }
}

/// Source of HTML content for table extraction
pub enum TableSource<'a> {
    Stdin,
    Url(&'a str),
    File(&'a str),
    Content(&'a str),
}

/// Trait for API client operations (allows testing with mock clients)
#[async_trait::async_trait]
pub trait ApiClient: Send + Sync {
    async fn post_json<T: serde::Serialize + Send + Sync>(
        &self,
        endpoint: &str,
        request: &T,
    ) -> Result<TableExtractResponse>;

    async fn get_text(&self, endpoint: &str) -> Result<String>;
}
