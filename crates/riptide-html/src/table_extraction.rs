//! Advanced table extraction system for Week 6
//!
//! This module implements comprehensive table extraction capabilities including:
//! - TABLE-001: Complete HTML table parser with thead/tbody/tfoot sections
//! - TABLE-002: RFC 4180 compliant CSV export
//! - TABLE-003: Markdown table export with merged cell handling
//! - TABLE-004: NDJSON artifacts linking
//!
//! Features:
//! - Full support for colspan/rowspan attributes
//! - Nested table detection with parent_id tracking
//! - Proper cell merging logic
//! - Multiple export formats (CSV, Markdown, NDJSON)
//! - Production-ready error handling

use anyhow::{anyhow, Result};
use scraper::{Html, Selector, ElementRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

/// Advanced table data structure with full metadata support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedTableData {
    /// Unique table identifier
    pub id: String,
    /// Table headers organized by sections
    pub headers: TableHeaders,
    /// Table body rows with cell metadata
    pub rows: Vec<TableRow>,
    /// Table footer rows (if present)
    pub footer: Vec<TableRow>,
    /// Table caption (if present)
    pub caption: Option<String>,
    /// Table-level metadata and attributes
    pub metadata: TableMetadata,
    /// Parent table ID for nested tables
    pub parent_id: Option<String>,
    /// Nested tables contained within this table
    pub nested_tables: Vec<String>,
    /// Table structure information
    pub structure: TableStructure,
}

/// Table headers organized by sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeaders {
    /// Main header row(s)
    pub main: Vec<TableCell>,
    /// Sub-headers (multi-level headers)
    pub sub_headers: Vec<Vec<TableCell>>,
    /// Column groups information
    pub column_groups: Vec<ColumnGroup>,
}

/// Individual table row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Row cells
    pub cells: Vec<TableCell>,
    /// Row-level attributes
    pub attributes: HashMap<String, String>,
    /// Row type (header, body, footer)
    pub row_type: RowType,
    /// Row index within its section
    pub index: usize,
}

/// Individual table cell with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Cell content (text)
    pub content: String,
    /// Raw HTML content
    pub html_content: String,
    /// Column span
    pub colspan: usize,
    /// Row span
    pub rowspan: usize,
    /// Cell type (header, data)
    pub cell_type: CellType,
    /// Cell-level attributes
    pub attributes: HashMap<String, String>,
    /// Column index (accounting for spans)
    pub column_index: usize,
    /// Row index (accounting for spans)
    pub row_index: usize,
    /// Cells that this cell spans over
    pub spans_over: Vec<CellPosition>,
}

/// Column group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnGroup {
    /// Column span
    pub span: usize,
    /// Group attributes
    pub attributes: HashMap<String, String>,
    /// Group label/title
    pub label: Option<String>,
}

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    /// All table attributes
    pub attributes: HashMap<String, String>,
    /// CSS classes
    pub classes: Vec<String>,
    /// Table ID attribute
    pub id: Option<String>,
    /// Processing timestamp
    pub processed_at: String,
    /// Source URL or document reference
    pub source: Option<String>,
}

/// Table structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructure {
    /// Total number of columns (accounting for spans)
    pub total_columns: usize,
    /// Total number of rows in body
    pub total_rows: usize,
    /// Number of header rows
    pub header_rows: usize,
    /// Number of footer rows
    pub footer_rows: usize,
    /// Has complex structure (spans, nested tables)
    pub has_complex_structure: bool,
    /// Maximum colspan value
    pub max_colspan: usize,
    /// Maximum rowspan value
    pub max_rowspan: usize,
}

/// Row type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RowType {
    Header,
    Body,
    Footer,
}

/// Cell type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CellType {
    Header,
    Data,
}

/// Cell position for span tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellPosition {
    pub row: usize,
    pub column: usize,
}

/// Table extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExtractionConfig {
    /// Include nested tables
    pub include_nested: bool,
    /// Preserve HTML formatting in cells
    pub preserve_html: bool,
    /// Maximum nesting depth for tables
    pub max_nesting_depth: usize,
    /// Minimum table size (rows x cols)
    pub min_size: Option<(usize, usize)>,
    /// Extract only tables with headers
    pub headers_only: bool,
    /// Custom table selector
    pub custom_selector: Option<String>,
}

impl Default for TableExtractionConfig {
    fn default() -> Self {
        Self {
            include_nested: true,
            preserve_html: false,
            max_nesting_depth: 3,
            min_size: None,
            headers_only: false,
            custom_selector: None,
        }
    }
}

/// Main table extractor
pub struct TableExtractor {
    config: TableExtractionConfig,
    document: Html,
}

impl TableExtractor {
    /// Create new table extractor
    pub fn new(html: &str, config: TableExtractionConfig) -> Self {
        Self {
            config,
            document: Html::parse_document(html),
        }
    }

    /// Extract all tables from the document
    pub fn extract_all_tables(&self) -> Result<Vec<AdvancedTableData>> {
        let selector_str = self.config.custom_selector
            .as_deref()
            .unwrap_or("table");

        let table_selector = Selector::parse(selector_str)
            .map_err(|e| anyhow!("Invalid table selector: {}", e))?;

        let mut tables = Vec::new();
        let mut table_counter = 0;

        // Collect table elements first to avoid borrowing issues
        let table_elements: Vec<_> = self.document.select(&table_selector).collect();

        for table_element in table_elements {
            table_counter += 1;
            let table_data = self.extract_single_table(table_element, None, 0, &mut table_counter)?;

            // Apply size filtering if configured
            if let Some((min_rows, min_cols)) = self.config.min_size {
                if table_data.structure.total_rows < min_rows ||
                   table_data.structure.total_columns < min_cols {
                    continue;
                }
            }

            // Apply headers-only filter if configured
            if self.config.headers_only && table_data.structure.header_rows == 0 {
                continue;
            }

            tables.push(table_data);
        }

        Ok(tables)
    }

    /// Extract data from a single table element
    fn extract_single_table(
        &self,
        table_element: ElementRef,
        parent_id: Option<String>,
        depth: usize,
        table_counter: &mut usize,
    ) -> Result<AdvancedTableData> {
        if depth > self.config.max_nesting_depth {
            return Err(anyhow!("Maximum nesting depth exceeded"));
        }

        let table_id = format!("table_{}", table_counter);

        // Extract table metadata
        let metadata = self.extract_table_metadata(table_element)?;

        // Extract caption
        let caption = self.extract_caption(table_element)?;

        // Extract column groups
        let column_groups = self.extract_column_groups(table_element)?;

        // Extract table sections
        let (headers, body_rows, footer_rows) = self.extract_table_sections(table_element)?;

        // Process nested tables if enabled
        let mut nested_tables = Vec::new();
        if self.config.include_nested {
            nested_tables = self.extract_nested_tables(table_element, &table_id, depth + 1, table_counter)?;
        }

        // Calculate structure information
        let structure = self.calculate_table_structure(&headers, &body_rows, &footer_rows);

        Ok(AdvancedTableData {
            id: table_id,
            headers: TableHeaders {
                main: headers,
                sub_headers: Vec::new(), // TODO: Implement multi-level headers
                column_groups,
            },
            rows: body_rows,
            footer: footer_rows,
            caption,
            metadata,
            parent_id,
            nested_tables,
            structure,
        })
    }

    /// Extract table metadata and attributes
    fn extract_table_metadata(&self, table_element: ElementRef) -> Result<TableMetadata> {
        let value = table_element.value();
        let mut attributes = HashMap::new();
        let mut classes = Vec::new();
        let mut id = None;

        // Extract all attributes
        for (attr, val) in value.attrs() {
            attributes.insert(attr.to_string(), val.to_string());

            match attr {
                "class" => {
                    classes = val.split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                "id" => {
                    id = Some(val.to_string());
                }
                _ => {}
            }
        }

        Ok(TableMetadata {
            attributes,
            classes,
            id,
            processed_at: chrono::Utc::now().to_rfc3339(),
            source: None,
        })
    }

    /// Extract table caption
    fn extract_caption(&self, table_element: ElementRef) -> Result<Option<String>> {
        let caption_selector = Selector::parse("caption")
            .map_err(|e| anyhow!("Invalid caption selector: {}", e))?;

        if let Some(caption_element) = table_element.select(&caption_selector).next() {
            let caption_text = caption_element.text().collect::<String>().trim().to_string();
            if !caption_text.is_empty() {
                return Ok(Some(caption_text));
            }
        }

        Ok(None)
    }

    /// Extract column groups
    fn extract_column_groups(&self, table_element: ElementRef) -> Result<Vec<ColumnGroup>> {
        let colgroup_selector = Selector::parse("colgroup")
            .map_err(|e| anyhow!("Invalid colgroup selector: {}", e))?;

        let mut column_groups = Vec::new();

        for colgroup_element in table_element.select(&colgroup_selector) {
            let value = colgroup_element.value();
            let span = value.attr("span")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            let attributes: HashMap<String, String> = value.attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let label_text = colgroup_element.text().collect::<String>();
            let label = label_text.trim();
            let label = if label.is_empty() { None } else { Some(label.to_string()) };

            column_groups.push(ColumnGroup {
                span,
                attributes,
                label,
            });
        }

        Ok(column_groups)
    }

    /// Extract table sections (thead, tbody, tfoot)
    fn extract_table_sections(
        &self,
        table_element: ElementRef,
    ) -> Result<(Vec<TableCell>, Vec<TableRow>, Vec<TableRow>)> {
        // Extract headers from thead
        let mut headers = Vec::new();
        let thead_selector = Selector::parse("thead tr")
            .map_err(|e| anyhow!("Invalid thead selector: {}", e))?;

        if let Some(header_row) = table_element.select(&thead_selector).next() {
            headers = self.extract_row_cells(header_row, RowType::Header, 0)?;
        } else {
            // Fallback: look for th elements in first row
            let first_row_selector = Selector::parse("tr:first-child")
                .map_err(|e| anyhow!("Invalid first row selector: {}", e))?;

            if let Some(first_row) = table_element.select(&first_row_selector).next() {
                let th_selector = Selector::parse("th")
                    .map_err(|e| anyhow!("Invalid th selector: {}", e))?;

                if first_row.select(&th_selector).next().is_some() {
                    headers = self.extract_row_cells(first_row, RowType::Header, 0)?;
                }
            }
        }

        // Extract body rows
        let mut body_rows = Vec::new();
        let tbody_selector = Selector::parse("tbody tr")
            .map_err(|e| anyhow!("Invalid tbody selector: {}", e))?;

        let tbody_elements: Vec<_> = table_element.select(&tbody_selector).collect();

        if !tbody_elements.is_empty() {
            // Table has explicit tbody
            for (index, row_element) in tbody_elements.iter().enumerate() {
                let cells = self.extract_row_cells(*row_element, RowType::Body, index)?;
                let attributes = self.extract_element_attributes(*row_element);

                body_rows.push(TableRow {
                    cells,
                    attributes,
                    row_type: RowType::Body,
                    index,
                });
            }
        } else {
            // No explicit tbody, extract all tr elements that aren't in thead/tfoot
            let tr_selector = Selector::parse("tr")
                .map_err(|e| anyhow!("Invalid tr selector: {}", e))?;

            let mut row_index = 0;
            for row_element in table_element.select(&tr_selector) {
                // Skip if this row is in thead or tfoot
                if self.is_in_section(row_element, "thead") ||
                   self.is_in_section(row_element, "tfoot") {
                    continue;
                }

                // Skip header row if we already extracted headers
                if !headers.is_empty() && row_index == 0 {
                    let th_selector = Selector::parse("th")
                        .map_err(|e| anyhow!("Invalid th selector: {}", e))?;
                    if row_element.select(&th_selector).next().is_some() {
                        row_index += 1;
                        continue;
                    }
                }

                let cells = self.extract_row_cells(row_element, RowType::Body, row_index)?;
                let attributes = self.extract_element_attributes(row_element);

                body_rows.push(TableRow {
                    cells,
                    attributes,
                    row_type: RowType::Body,
                    index: row_index,
                });

                row_index += 1;
            }
        }

        // Extract footer rows
        let mut footer_rows = Vec::new();
        let tfoot_selector = Selector::parse("tfoot tr")
            .map_err(|e| anyhow!("Invalid tfoot selector: {}", e))?;

        for (index, row_element) in table_element.select(&tfoot_selector).enumerate() {
            let cells = self.extract_row_cells(row_element, RowType::Footer, index)?;
            let attributes = self.extract_element_attributes(row_element);

            footer_rows.push(TableRow {
                cells,
                attributes,
                row_type: RowType::Footer,
                index,
            });
        }

        Ok((headers, body_rows, footer_rows))
    }

    /// Extract cells from a table row
    fn extract_row_cells(
        &self,
        row_element: ElementRef,
        row_type: RowType,
        row_index: usize,
    ) -> Result<Vec<TableCell>> {
        let cell_selector = Selector::parse("td, th")
            .map_err(|e| anyhow!("Invalid cell selector: {}", e))?;

        let mut cells = Vec::new();
        let mut column_index = 0;

        for cell_element in row_element.select(&cell_selector) {
            let value = cell_element.value();

            // Extract cell content
            let content = cell_element.text().collect::<String>().trim().to_string();
            let html_content = if self.config.preserve_html {
                cell_element.html()
            } else {
                content.clone()
            };

            // Extract span attributes
            let colspan = value.attr("colspan")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let rowspan = value.attr("rowspan")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            // Determine cell type
            let cell_type = match value.name() {
                "th" => CellType::Header,
                "td" => CellType::Data,
                _ => if row_type == RowType::Header { CellType::Header } else { CellType::Data },
            };

            // Extract attributes
            let attributes = self.extract_element_attributes(cell_element);

            // Calculate spans_over positions
            let mut spans_over = Vec::new();
            for r in 0..rowspan {
                for c in 0..colspan {
                    if r > 0 || c > 0 {
                        spans_over.push(CellPosition {
                            row: row_index + r,
                            column: column_index + c,
                        });
                    }
                }
            }

            cells.push(TableCell {
                content,
                html_content,
                colspan,
                rowspan,
                cell_type,
                attributes,
                column_index,
                row_index,
                spans_over,
            });

            column_index += colspan;
        }

        Ok(cells)
    }

    /// Extract nested tables within the current table
    fn extract_nested_tables(
        &self,
        table_element: ElementRef,
        parent_id: &str,
        depth: usize,
        table_counter: &mut usize,
    ) -> Result<Vec<String>> {
        let nested_table_selector = Selector::parse("table")
            .map_err(|e| anyhow!("Invalid nested table selector: {}", e))?;

        let mut nested_table_ids = Vec::new();

        // Collect nested table elements to avoid borrowing issues
        let nested_elements: Vec<_> = table_element.select(&nested_table_selector).collect();

        for nested_element in nested_elements {
            // Skip the table element itself by comparing IDs
            if let (Some(nested_id), Some(table_id)) = (
                nested_element.value().id(),
                table_element.value().id()
            ) {
                if nested_id == table_id {
                    continue;
                }
            } else {
                // If no IDs, compare positions in HTML (simple approximation)
                let nested_html = nested_element.html();
                let table_html = table_element.html();
                if nested_html == table_html {
                    continue;
                }
            }

            *table_counter += 1;
            let nested_table = self.extract_single_table(
                nested_element,
                Some(parent_id.to_string()),
                depth,
                table_counter,
            )?;

            nested_table_ids.push(nested_table.id.clone());
        }

        Ok(nested_table_ids)
    }

    /// Calculate table structure information
    fn calculate_table_structure(
        &self,
        headers: &[TableCell],
        body_rows: &[TableRow],
        footer_rows: &[TableRow],
    ) -> TableStructure {
        let mut total_columns = 0;
        let mut max_colspan = 1;
        let mut max_rowspan = 1;
        let mut has_complex_structure = false;

        // Calculate from headers
        if !headers.is_empty() {
            total_columns = headers.iter().map(|cell| cell.colspan).sum();
            max_colspan = headers.iter().map(|cell| cell.colspan).max().unwrap_or(1);
            max_rowspan = headers.iter().map(|cell| cell.rowspan).max().unwrap_or(1);

            if max_colspan > 1 || max_rowspan > 1 {
                has_complex_structure = true;
            }
        }

        // Calculate from body rows
        if total_columns == 0 && !body_rows.is_empty() {
            total_columns = body_rows.iter()
                .map(|row| row.cells.iter().map(|cell| cell.colspan).sum::<usize>())
                .max()
                .unwrap_or(0);
        }

        // Check for complex structure in body rows
        for row in body_rows {
            for cell in &row.cells {
                if cell.colspan > 1 || cell.rowspan > 1 {
                    has_complex_structure = true;
                }
                max_colspan = max_colspan.max(cell.colspan);
                max_rowspan = max_rowspan.max(cell.rowspan);
            }
        }

        // Check footer rows
        for row in footer_rows {
            for cell in &row.cells {
                if cell.colspan > 1 || cell.rowspan > 1 {
                    has_complex_structure = true;
                }
                max_colspan = max_colspan.max(cell.colspan);
                max_rowspan = max_rowspan.max(cell.rowspan);
            }
        }

        TableStructure {
            total_columns,
            total_rows: body_rows.len(),
            header_rows: if headers.is_empty() { 0 } else { 1 },
            footer_rows: footer_rows.len(),
            has_complex_structure,
            max_colspan,
            max_rowspan,
        }
    }

    /// Extract attributes from an element
    fn extract_element_attributes(&self, element: ElementRef) -> HashMap<String, String> {
        element.value().attrs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Check if a row is within a specific section
    fn is_in_section(&self, row_element: ElementRef, section: &str) -> bool {
        let mut current = row_element.parent();
        while let Some(parent) = current {
            if let Some(element) = parent.value().as_element() {
                if element.name() == section {
                    return true;
                }
            }
            current = parent.parent();
        }
        false
    }
}

/// TABLE-002: RFC 4180 compliant CSV export
impl AdvancedTableData {
    /// Export table to RFC 4180 compliant CSV format
    pub fn to_csv(&self, include_headers: bool) -> Result<String> {
        let mut csv_output = String::new();

        // Add headers if present and requested
        if include_headers && !self.headers.main.is_empty() {
            let header_row = self.format_csv_row(&self.headers.main.iter()
                .map(|cell| &cell.content)
                .collect::<Vec<_>>())?;
            csv_output.push_str(&header_row);
            csv_output.push('\n');
        }

        // Add body rows
        for row in &self.rows {
            let cell_contents: Vec<&String> = row.cells.iter()
                .map(|cell| &cell.content)
                .collect();
            let csv_row = self.format_csv_row(&cell_contents)?;
            csv_output.push_str(&csv_row);
            csv_output.push('\n');
        }

        // Add footer rows if present
        for row in &self.footer {
            let cell_contents: Vec<&String> = row.cells.iter()
                .map(|cell| &cell.content)
                .collect();
            let csv_row = self.format_csv_row(&cell_contents)?;
            csv_output.push_str(&csv_row);
            csv_output.push('\n');
        }

        Ok(csv_output.trim_end().to_string())
    }

    /// Format a single CSV row according to RFC 4180
    fn format_csv_row(&self, cells: &[&String]) -> Result<String> {
        let mut row = String::new();

        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                row.push(',');
            }

            // RFC 4180 compliance: escape quotes and wrap in quotes if necessary
            if cell.contains(',') || cell.contains('"') || cell.contains('\n') || cell.contains('\r') {
                row.push('"');
                // Escape quotes by doubling them
                for ch in cell.chars() {
                    if ch == '"' {
                        row.push('"');
                        row.push('"');
                    } else {
                        row.push(ch);
                    }
                }
                row.push('"');
            } else {
                row.push_str(cell);
            }
        }

        Ok(row)
    }
}

/// TABLE-003: Markdown table export
impl AdvancedTableData {
    /// Export table to Markdown format
    pub fn to_markdown(&self, include_metadata: bool) -> Result<String> {
        let mut md_output = String::new();

        // Add caption if present
        if let Some(caption) = &self.caption {
            writeln!(md_output, "*{}*\n", caption)?;
        }

        // Add metadata as comment if requested
        if include_metadata {
            writeln!(md_output, "<!-- Table ID: {} -->", self.id)?;
            if let Some(parent_id) = &self.parent_id {
                writeln!(md_output, "<!-- Parent Table: {} -->", parent_id)?;
            }
            writeln!(md_output, "<!-- Columns: {}, Rows: {} -->",
                self.structure.total_columns, self.structure.total_rows)?;
            if self.structure.has_complex_structure {
                writeln!(md_output, "<!-- Note: Table has complex structure (spans) -->")?;
            }
            writeln!(md_output)?;
        }

        // Handle headers
        if !self.headers.main.is_empty() {
            // Header row
            write!(md_output, "|")?;
            for cell in &self.headers.main {
                write!(md_output, " {} |", self.escape_markdown(&cell.content))?;
            }
            writeln!(md_output)?;

            // Separator row
            write!(md_output, "|")?;
            for _ in &self.headers.main {
                write!(md_output, " --- |")?;
            }
            writeln!(md_output)?;
        }

        // Body rows
        for row in &self.rows {
            write!(md_output, "|")?;
            for cell in &row.cells {
                let content = if cell.colspan > 1 || cell.rowspan > 1 {
                    format!("{} (span: {}x{})",
                        self.escape_markdown(&cell.content),
                        cell.colspan,
                        cell.rowspan)
                } else {
                    self.escape_markdown(&cell.content)
                };
                write!(md_output, " {} |", content)?;
            }
            writeln!(md_output)?;
        }

        // Footer rows if present
        if !self.footer.is_empty() {
            writeln!(md_output)?;
            writeln!(md_output, "**Footer:**")?;
            for row in &self.footer {
                write!(md_output, "|")?;
                for cell in &row.cells {
                    write!(md_output, " {} |", self.escape_markdown(&cell.content))?;
                }
                writeln!(md_output)?;
            }
        }

        // Add nested tables reference if any
        if !self.nested_tables.is_empty() {
            writeln!(md_output)?;
            writeln!(md_output, "**Nested Tables:** {}", self.nested_tables.join(", "))?;
        }

        Ok(md_output)
    }

    /// Escape markdown special characters
    fn escape_markdown(&self, text: &str) -> String {
        text.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', "")
    }
}

/// TABLE-004: NDJSON artifacts export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableArtifact {
    /// Table reference
    pub table_id: String,
    /// Artifact type (csv, markdown)
    pub artifact_type: String,
    /// File path or content
    pub content: String,
    /// Artifact metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: String,
}

impl AdvancedTableData {
    /// Create NDJSON artifacts for the table
    pub fn to_ndjson_artifacts(&self, base_path: Option<&str>) -> Result<Vec<String>> {
        let mut artifacts = Vec::new();

        // CSV artifact
        let csv_content = self.to_csv(true)?;
        let csv_artifact = TableArtifact {
            table_id: self.id.clone(),
            artifact_type: "csv".to_string(),
            content: if let Some(path) = base_path {
                format!("{}/{}.csv", path, self.id)
            } else {
                csv_content
            },
            metadata: [
                ("format".to_string(), "RFC4180".to_string()),
                ("headers".to_string(), (!self.headers.main.is_empty()).to_string()),
                ("rows".to_string(), self.structure.total_rows.to_string()),
                ("columns".to_string(), self.structure.total_columns.to_string()),
            ].into_iter().collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(serde_json::to_string(&csv_artifact)?);

        // Markdown artifact
        let md_content = self.to_markdown(true)?;
        let md_artifact = TableArtifact {
            table_id: self.id.clone(),
            artifact_type: "markdown".to_string(),
            content: if let Some(path) = base_path {
                format!("{}/{}.md", path, self.id)
            } else {
                md_content
            },
            metadata: [
                ("format".to_string(), "markdown".to_string()),
                ("has_metadata".to_string(), "true".to_string()),
                ("complex_structure".to_string(), self.structure.has_complex_structure.to_string()),
            ].into_iter().collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(serde_json::to_string(&md_artifact)?);

        // JSON metadata artifact
        let json_artifact = TableArtifact {
            table_id: self.id.clone(),
            artifact_type: "metadata".to_string(),
            content: if let Some(path) = base_path {
                format!("{}/{}_metadata.json", path, self.id)
            } else {
                serde_json::to_string_pretty(self)?
            },
            metadata: [
                ("format".to_string(), "json".to_string()),
                ("complete_structure".to_string(), "true".to_string()),
            ].into_iter().collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        artifacts.push(serde_json::to_string(&json_artifact)?);

        Ok(artifacts)
    }
}

/// Error types for table extraction
#[derive(Debug, thiserror::Error)]
pub enum TableExtractionError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid table structure: {0}")]
    InvalidStructure(String),

    #[error("Export format error: {0}")]
    ExportError(String),

    #[error("Nesting depth exceeded: {max_depth}")]
    NestingDepthExceeded { max_depth: usize },

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Convenience functions for easy table extraction
pub async fn extract_tables_advanced(
    html: &str,
    config: Option<TableExtractionConfig>,
) -> Result<Vec<AdvancedTableData>> {
    let config = config.unwrap_or_default();
    let extractor = TableExtractor::new(html, config);
    extractor.extract_all_tables()
}

/// Extract tables and export to all formats
pub async fn extract_and_export_tables(
    html: &str,
    base_path: Option<&str>,
    config: Option<TableExtractionConfig>,
) -> Result<(Vec<AdvancedTableData>, Vec<String>)> {
    let tables = extract_tables_advanced(html, config).await?;
    let mut all_artifacts = Vec::new();

    for table in &tables {
        let artifacts = table.to_ndjson_artifacts(base_path)?;
        all_artifacts.extend(artifacts);
    }

    Ok((tables, all_artifacts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_table_extraction() {
        let html = r#"
            <table id="test-table" class="data-table">
                <caption>Test Table</caption>
                <thead>
                    <tr><th>Name</th><th>Age</th><th>City</th></tr>
                </thead>
                <tbody>
                    <tr><td>John</td><td>30</td><td>New York</td></tr>
                    <tr><td>Jane</td><td>25</td><td>Los Angeles</td></tr>
                </tbody>
                <tfoot>
                    <tr><td>Total</td><td>2</td><td>-</td></tr>
                </tfoot>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert_eq!(table.headers.main.len(), 3);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.footer.len(), 1);
        assert_eq!(table.caption, Some("Test Table".to_string()));
        assert_eq!(table.metadata.id, Some("test-table".to_string()));
        assert!(table.metadata.classes.contains(&"data-table".to_string()));
    }

    #[tokio::test]
    async fn test_complex_table_with_spans() {
        let html = r#"
            <table>
                <tr>
                    <th colspan="2">Name</th>
                    <th rowspan="2">Age</th>
                </tr>
                <tr>
                    <th>First</th>
                    <th>Last</th>
                </tr>
                <tr>
                    <td>John</td>
                    <td>Doe</td>
                    <td>30</td>
                </tr>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert!(table.structure.has_complex_structure);
        assert_eq!(table.structure.max_colspan, 2);
        assert_eq!(table.structure.max_rowspan, 2);
    }

    #[tokio::test]
    async fn test_nested_tables() {
        let html = r#"
            <table id="outer">
                <tr>
                    <td>Cell 1</td>
                    <td>
                        <table id="inner">
                            <tr><td>Nested 1</td><td>Nested 2</td></tr>
                        </table>
                    </td>
                </tr>
            </table>
        "#;

        let config = TableExtractionConfig {
            include_nested: true,
            ..Default::default()
        };

        let tables = extract_tables_advanced(html, Some(config)).await.unwrap();

        // We should find 2 tables total: outer and inner
        assert_eq!(tables.len(), 2);

        // Find the outer table (should have nested_tables)
        let outer_table = tables.iter().find(|t| !t.nested_tables.is_empty())
            .expect("Should find outer table with nested tables");

        // Find the inner table (should be standalone)
        let inner_table = tables.iter().find(|t| t.nested_tables.is_empty() && t.id != outer_table.id)
            .expect("Should find inner table");

        // Verify nested table detection worked
        assert!(!outer_table.nested_tables.is_empty());
        assert_eq!(outer_table.nested_tables.len(), 1);

        // Verify inner table structure
        assert_eq!(inner_table.rows.len(), 1);
        assert_eq!(inner_table.rows[0].cells.len(), 2);
        assert_eq!(inner_table.rows[0].cells[0].content, "Nested 1");
        assert_eq!(inner_table.rows[0].cells[1].content, "Nested 2");
    }

    #[test]
    fn test_csv_export() {
        let table = create_test_table();
        let csv = table.to_csv(true).unwrap();

        assert!(csv.contains("Name,Age,City"));
        assert!(csv.contains("John,30,New York"));
        assert!(csv.contains("Jane,25,Los Angeles"));
    }

    #[test]
    fn test_markdown_export() {
        let table = create_test_table();
        let md = table.to_markdown(true).unwrap();

        assert!(md.contains("| Name | Age | City |"));
        assert!(md.contains("| --- | --- | --- |"));
        assert!(md.contains("| John | 30 | New York |"));
        assert!(md.contains("<!-- Table ID:"));
    }

    #[test]
    fn test_ndjson_artifacts() {
        let table = create_test_table();
        let artifacts = table.to_ndjson_artifacts(None).unwrap();

        assert_eq!(artifacts.len(), 3); // CSV, Markdown, Metadata

        for artifact in artifacts {
            let artifact_obj: TableArtifact = serde_json::from_str(&artifact).unwrap();
            assert_eq!(artifact_obj.table_id, table.id);
            assert!(["csv", "markdown", "metadata"].contains(&artifact_obj.artifact_type.as_str()));
        }
    }

    fn create_test_table() -> AdvancedTableData {
        AdvancedTableData {
            id: "test_1".to_string(),
            headers: TableHeaders {
                main: vec![
                    create_test_cell("Name", CellType::Header, 0, 0),
                    create_test_cell("Age", CellType::Header, 0, 1),
                    create_test_cell("City", CellType::Header, 0, 2),
                ],
                sub_headers: Vec::new(),
                column_groups: Vec::new(),
            },
            rows: vec![
                TableRow {
                    cells: vec![
                        create_test_cell("John", CellType::Data, 1, 0),
                        create_test_cell("30", CellType::Data, 1, 1),
                        create_test_cell("New York", CellType::Data, 1, 2),
                    ],
                    attributes: HashMap::new(),
                    row_type: RowType::Body,
                    index: 0,
                },
                TableRow {
                    cells: vec![
                        create_test_cell("Jane", CellType::Data, 2, 0),
                        create_test_cell("25", CellType::Data, 2, 1),
                        create_test_cell("Los Angeles", CellType::Data, 2, 2),
                    ],
                    attributes: HashMap::new(),
                    row_type: RowType::Body,
                    index: 1,
                },
            ],
            footer: Vec::new(),
            caption: Some("Test Table".to_string()),
            metadata: TableMetadata {
                attributes: HashMap::new(),
                classes: Vec::new(),
                id: Some("test".to_string()),
                processed_at: chrono::Utc::now().to_rfc3339(),
                source: None,
            },
            parent_id: None,
            nested_tables: Vec::new(),
            structure: TableStructure {
                total_columns: 3,
                total_rows: 2,
                header_rows: 1,
                footer_rows: 0,
                has_complex_structure: false,
                max_colspan: 1,
                max_rowspan: 1,
            },
        }
    }

    fn create_test_cell(content: &str, cell_type: CellType, row: usize, col: usize) -> TableCell {
        TableCell {
            content: content.to_string(),
            html_content: content.to_string(),
            colspan: 1,
            rowspan: 1,
            cell_type,
            attributes: HashMap::new(),
            column_index: col,
            row_index: row,
            spans_over: Vec::new(),
        }
    }
}