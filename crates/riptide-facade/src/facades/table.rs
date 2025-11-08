//! Table facade for table extraction and storage operations

use riptide_extraction::table_extraction::AdvancedTableData;

#[cfg(feature = "llm")]
use riptide_intelligence::TableAnalyzer;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Table summary for API responses
#[derive(Debug, Clone)]
pub struct TableSummary {
    pub id: String,
    pub rows: usize,
    pub columns: usize,
    pub headers: Vec<String>,
    pub data: Vec<Vec<String>>,
    pub metadata: TableMetadata,
}

/// Table metadata
#[derive(Debug, Clone)]
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
}

impl Default for TableFacade {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riptide_extraction::table_extraction::{TableCell, TableHeaders, TableRow};

    #[tokio::test]
    async fn test_table_cache_service() {
        let cache = TableCacheService::new();

        // Create a mock table with proper structure
        let table = AdvancedTableData {
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
        };

        let id = cache.store(table.clone()).await;
        assert!(!id.is_empty());

        let retrieved = cache.get(&id).await;
        assert!(retrieved.is_some());

        let removed = cache.remove(&id).await;
        assert!(removed.is_some());

        let not_found = cache.get(&id).await;
        assert!(not_found.is_none());
    }
}
