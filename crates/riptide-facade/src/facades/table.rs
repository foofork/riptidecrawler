//! Table facade for table extraction and storage operations

use crate::error::RiptideError;
use riptide_extraction::table_extraction::{
    extract_tables_advanced, AdvancedTableData, TableExtractionConfig,
};

#[cfg(feature = "llm")]
use riptide_intelligence::TableAnalyzer;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Result type for table operations
pub type Result<T> = std::result::Result<T, RiptideError>;

/// Table summary for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableSummary {
    pub id: String,
    pub rows: usize,
    pub columns: usize,
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub metadata: TableMetadata,
}

/// Table metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableMetadata {
    pub has_headers: bool,
    pub data_types: Vec<String>,
    pub has_complex_structure: bool,
    pub caption: Option<String>,
    pub css_classes: Vec<String>,
    pub html_id: Option<String>,
}

/// Options for table extraction
#[derive(Debug, Clone)]
pub struct TableExtractionOptions {
    pub include_headers: bool,
    pub detect_data_types: bool,
}

/// Full extraction configuration with HTML validation
#[derive(Debug, Clone)]
pub struct TableExtractionRequest {
    pub html_content: String,
    pub include_nested: bool,
    pub preserve_html: bool,
    pub max_nesting_depth: usize,
    pub min_size: Option<(usize, usize)>,
    pub headers_only: bool,
    pub include_headers: bool,
    pub detect_data_types: bool,
}

/// Table export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableFormat {
    Csv,
    Markdown,
}

impl TableFormat {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "markdown" | "md" => Some(Self::Markdown),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Markdown => "md",
        }
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Csv => "text/csv",
            Self::Markdown => "text/markdown",
        }
    }
}

/// Table cache service for storing extracted tables
pub struct TableCacheService {
    cache: Arc<Mutex<HashMap<String, AdvancedTableData>>>,
}

impl TableCacheService {
    /// Create a new table cache service
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store a table and return its ID
    pub async fn store(&self, table: AdvancedTableData) -> String {
        let id = Uuid::new_v4().to_string();
        let mut cache = self.cache.lock().await;
        cache.insert(id.clone(), table);
        id
    }

    /// Retrieve a table by ID
    pub async fn get(&self, id: &str) -> Option<AdvancedTableData> {
        let cache = self.cache.lock().await;
        cache.get(id).cloned()
    }

    /// Remove a table from cache
    pub async fn remove(&self, id: &str) -> Option<AdvancedTableData> {
        let mut cache = self.cache.lock().await;
        cache.remove(id)
    }
}

impl Default for TableCacheService {
    fn default() -> Self {
        Self::new()
    }
}

/// Facade for table operations
pub struct TableFacade {
    #[cfg(feature = "llm")]
    analyzer: TableAnalyzer,
    cache: TableCacheService,
}

impl TableFacade {
    /// Create a new table facade
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "llm")]
            analyzer: TableAnalyzer::new(),
            cache: TableCacheService::new(),
        }
    }

    /// Store tables and create summaries
    pub async fn store_and_summarize(
        &self,
        tables: Vec<AdvancedTableData>,
        options: &TableExtractionOptions,
    ) -> Vec<TableSummary> {
        let mut summaries = Vec::new();

        for table in tables {
            // Extract headers and sample data based on options
            let (headers, sample_data) = if options.include_headers {
                let headers: Vec<String> = table
                    .headers
                    .main
                    .iter()
                    .map(|cell| cell.content.clone())
                    .collect();

                let sample_data: Vec<Vec<String>> = table
                    .rows
                    .iter()
                    .take(3)
                    .map(|row| row.cells.iter().map(|cell| cell.content.clone()).collect())
                    .collect();

                (headers, sample_data)
            } else {
                (vec![], vec![])
            };

            // Detect data types if enabled
            let data_types = if options.detect_data_types {
                #[cfg(feature = "llm")]
                {
                    self.analyzer.detect_column_types(&table)
                }
                #[cfg(not(feature = "llm"))]
                {
                    vec![]
                }
            } else {
                vec![]
            };

            // Create summary
            let summary = TableSummary {
                id: String::new(), // Will be set after storage
                rows: table.structure.total_rows,
                columns: table.structure.total_columns,
                headers: headers.clone(),
                data: sample_data,
                metadata: TableMetadata {
                    has_headers: !headers.is_empty(),
                    data_types,
                    has_complex_structure: table.structure.has_complex_structure,
                    caption: table.caption.clone(),
                    css_classes: table.metadata.classes.clone(),
                    html_id: table.metadata.id.clone(),
                },
            };

            // Store table and update ID
            let id = self.cache.store(table).await;
            let summary = TableSummary { id, ..summary };
            summaries.push(summary);
        }

        summaries
    }

    /// Get a stored table by ID
    pub async fn get_table(&self, id: &str) -> Option<AdvancedTableData> {
        self.cache.get(id).await
    }

    /// Extract tables from HTML with full validation and orchestration
    ///
    /// This is the comprehensive extraction method that:
    /// - Validates HTML content (size, emptiness)
    /// - Configures extraction with all options
    /// - Performs extraction using riptide-extraction
    /// - Stores tables in cache
    /// - Returns summaries for API response
    pub async fn extract_tables_full(
        &self,
        request: TableExtractionRequest,
    ) -> Result<Vec<TableSummary>> {
        // 1. Validate HTML content
        if request.html_content.trim().is_empty() {
            return Err(RiptideError::Validation(
                "HTML content cannot be empty".to_string(),
            ));
        }

        if request.html_content.len() > 10_000_000 {
            return Err(RiptideError::Validation(
                "HTML content too large (max 10MB)".to_string(),
            ));
        }

        // 2. Configure extraction
        let config = TableExtractionConfig {
            include_nested: request.include_nested,
            preserve_html: request.preserve_html,
            max_nesting_depth: request.max_nesting_depth,
            min_size: request.min_size,
            headers_only: request.headers_only,
            custom_selector: None,
        };

        // 3. Extract tables using riptide-extraction
        let tables = extract_tables_advanced(&request.html_content, Some(config))
            .await
            .map_err(|e| RiptideError::Extraction(format!("Table extraction failed: {}", e)))?;

        // 4. Store tables and create summaries
        let options = TableExtractionOptions {
            include_headers: request.include_headers,
            detect_data_types: request.detect_data_types,
        };

        Ok(self.store_and_summarize(tables, &options).await)
    }

    /// Export table in specified format
    ///
    /// Retrieves a table by ID and exports it in the requested format.
    /// Supports CSV and Markdown with optional headers and metadata.
    pub async fn export_table(
        &self,
        table_id: &str,
        format: TableFormat,
        include_headers: bool,
        include_metadata: bool,
    ) -> Result<(String, &'static str)> {
        // 1. Retrieve table from cache
        let table = self.cache.get(table_id).await.ok_or_else(|| {
            RiptideError::Other(anyhow::anyhow!(
                "Table with ID '{}' not found or expired",
                table_id
            ))
        })?;

        // 2. Export based on format
        let (content, content_type) = match format {
            TableFormat::Csv => {
                let csv_content = table
                    .to_csv(include_headers)
                    .map_err(|e| RiptideError::Extraction(format!("CSV export failed: {}", e)))?;
                (csv_content, format.content_type())
            }
            TableFormat::Markdown => {
                let md_content = table.to_markdown(include_metadata).map_err(|e| {
                    RiptideError::Extraction(format!("Markdown export failed: {}", e))
                })?;
                (md_content, format.content_type())
            }
        };

        Ok((content, content_type))
    }

    /// Validate export format string
    pub fn validate_format(format_str: &str) -> Result<TableFormat> {
        TableFormat::from_str(format_str).ok_or_else(|| {
            RiptideError::Validation("Format must be 'csv' or 'markdown'".to_string())
        })
    }

    /// Get extraction statistics for a table request
    pub async fn get_extraction_stats(&self, _table_id: &str) -> Result<ExtractionStats> {
        // TODO: Implement actual stats tracking in Phase 6
        // For now, return placeholder stats
        Ok(ExtractionStats {
            total_extractions: 0,
            successful_extractions: 0,
            failed_extractions: 0,
            avg_extraction_time_ms: 0.0,
        })
    }
}

/// Statistics for table extraction operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractionStats {
    pub total_extractions: u64,
    pub successful_extractions: u64,
    pub failed_extractions: u64,
    pub avg_extraction_time_ms: f64,
}

impl Default for TableFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_extraction::table_extraction::TableHeaders;

    #[tokio::test]
    async fn test_table_cache_service() {
        let cache = TableCacheService::new();

        // Create a mock table with proper structure
        let table = create_mock_table();

        let id = cache.store(table.clone()).await;
        assert!(!id.is_empty());

        let retrieved = cache.get(&id).await;
        assert!(retrieved.is_some());

        let removed = cache.remove(&id).await;
        assert!(removed.is_some());

        let not_found = cache.get(&id).await;
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_extract_tables_full_validation() {
        let facade = TableFacade::new();

        // Test empty HTML validation
        let empty_request = TableExtractionRequest {
            html_content: "   ".to_string(),
            include_nested: true,
            preserve_html: false,
            max_nesting_depth: 3,
            min_size: None,
            headers_only: false,
            include_headers: true,
            detect_data_types: false,
        };

        let result = facade.extract_tables_full(empty_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));

        // Test HTML too large
        let large_request = TableExtractionRequest {
            html_content: "x".repeat(10_000_001),
            include_nested: true,
            preserve_html: false,
            max_nesting_depth: 3,
            min_size: None,
            headers_only: false,
            include_headers: true,
            detect_data_types: false,
        };

        let result = facade.extract_tables_full(large_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[tokio::test]
    async fn test_table_format_validation() {
        // Valid formats
        assert_eq!(TableFormat::from_str("csv"), Some(TableFormat::Csv));
        assert_eq!(TableFormat::from_str("CSV"), Some(TableFormat::Csv));
        assert_eq!(
            TableFormat::from_str("markdown"),
            Some(TableFormat::Markdown)
        );
        assert_eq!(TableFormat::from_str("md"), Some(TableFormat::Markdown));

        // Invalid format
        assert_eq!(TableFormat::from_str("invalid"), None);

        // Test validate_format
        assert!(TableFacade::validate_format("csv").is_ok());
        assert!(TableFacade::validate_format("markdown").is_ok());
        assert!(TableFacade::validate_format("invalid").is_err());
    }

    #[tokio::test]
    async fn test_table_format_properties() {
        let csv = TableFormat::Csv;
        assert_eq!(csv.extension(), "csv");
        assert_eq!(csv.content_type(), "text/csv");

        let md = TableFormat::Markdown;
        assert_eq!(md.extension(), "md");
        assert_eq!(md.content_type(), "text/markdown");
    }

    #[tokio::test]
    async fn test_export_table_not_found() {
        let facade = TableFacade::new();

        let result = facade
            .export_table("non-existent-id", TableFormat::Csv, true, false)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_store_and_summarize() {
        let facade = TableFacade::new();
        let tables = vec![create_mock_table()];
        let options = TableExtractionOptions {
            include_headers: true,
            detect_data_types: false,
        };

        let summaries = facade.store_and_summarize(tables, &options).await;
        assert_eq!(summaries.len(), 1);
        assert!(!summaries[0].id.is_empty());
        assert_eq!(summaries[0].rows, 0);
        assert_eq!(summaries[0].columns, 0);
    }

    fn create_mock_table() -> AdvancedTableData {
        AdvancedTableData {
            id: "test-table".to_string(),
            parent_id: None,
            headers: TableHeaders {
                main: vec![],
                sub_headers: vec![],
                column_groups: vec![],
            },
            rows: vec![],
            footer: vec![],
            nested_tables: vec![],
            structure: riptide_extraction::table_extraction::TableStructure {
                total_rows: 0,
                total_columns: 0,
                header_rows: 0,
                max_rowspan: 1,
                footer_rows: 0,
                max_colspan: 1,
                has_complex_structure: false,
            },
            metadata: riptide_extraction::table_extraction::TableMetadata {
                id: Some("test-table".to_string()),
                classes: vec![],
                attributes: std::collections::HashMap::new(),
                source: Some("test".to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
            },
            caption: None,
        }
    }
}
