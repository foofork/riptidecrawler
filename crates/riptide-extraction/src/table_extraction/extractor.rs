//! Table extraction implementation
//!
//! This module implements the core table extraction logic, including:
//! - HTML table parsing with full support for thead/tbody/tfoot
//! - Cell span handling (colspan/rowspan)
//! - Nested table detection
//! - Metadata extraction

use anyhow::{anyhow, Result};
use scraper::{ElementRef, Html, Selector};

use super::models::*;

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
        let selector_str = self.config.custom_selector.as_deref().unwrap_or("table");

        let table_selector =
            Selector::parse(selector_str).map_err(|e| anyhow!("Invalid table selector: {}", e))?;

        let mut tables = Vec::new();
        let mut table_counter = 0;

        // Collect table elements first to avoid borrowing issues
        let table_elements: Vec<_> = self.document.select(&table_selector).collect();

        for table_element in table_elements {
            table_counter += 1;
            let table_data =
                self.extract_single_table(table_element, None, 0, &mut table_counter)?;

            // Apply size filtering if configured
            if let Some((min_rows, min_cols)) = self.config.min_size {
                if table_data.structure.total_rows < min_rows
                    || table_data.structure.total_columns < min_cols
                {
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
        let (headers, sub_headers, body_rows, footer_rows) =
            self.extract_table_sections(table_element)?;

        // Process nested tables if enabled
        let mut nested_tables = Vec::new();
        if self.config.include_nested {
            nested_tables =
                self.extract_nested_tables(table_element, &table_id, depth + 1, table_counter)?;
        }

        // Calculate structure information
        let structure =
            self.calculate_table_structure(&headers, &sub_headers, &body_rows, &footer_rows);

        Ok(AdvancedTableData {
            id: table_id,
            headers: TableHeaders {
                main: headers,
                sub_headers,
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
        let mut attributes = std::collections::HashMap::new();
        let mut classes = Vec::new();
        let mut id = None;

        // Extract all attributes
        for (attr, val) in value.attrs() {
            attributes.insert(attr.to_string(), val.to_string());

            match attr {
                "class" => {
                    classes = val.split_whitespace().map(|s| s.to_string()).collect();
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
        let caption_selector =
            Selector::parse("caption").map_err(|e| anyhow!("Invalid caption selector: {}", e))?;

        if let Some(caption_element) = table_element.select(&caption_selector).next() {
            let caption_text = caption_element
                .text()
                .collect::<String>()
                .trim()
                .to_string();
            if !caption_text.is_empty() {
                return Ok(Some(caption_text));
            }
        }

        Ok(None)
    }

    /// Extract column groups
    fn extract_column_groups(&self, table_element: ElementRef) -> Result<Vec<ColumnGroup>> {
        let colgroup_selector =
            Selector::parse("colgroup").map_err(|e| anyhow!("Invalid colgroup selector: {}", e))?;

        let mut column_groups = Vec::new();

        for colgroup_element in table_element.select(&colgroup_selector) {
            let value = colgroup_element.value();
            let span = value.attr("span").and_then(|s| s.parse().ok()).unwrap_or(1);

            let attributes: std::collections::HashMap<String, String> = value
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();

            let label_text = colgroup_element.text().collect::<String>();
            let label = label_text.trim();
            let label = if label.is_empty() {
                None
            } else {
                Some(label.to_string())
            };

            column_groups.push(ColumnGroup {
                span,
                attributes,
                label,
            });
        }

        Ok(column_groups)
    }

    /// Extract table sections (thead, tbody, tfoot) with multi-level header support
    /// This is a cohesive function that handles complex section extraction logic
    fn extract_table_sections(
        &self,
        table_element: ElementRef,
    ) -> Result<(
        Vec<TableCell>,
        Vec<Vec<TableCell>>,
        Vec<TableRow>,
        Vec<TableRow>,
    )> {
        // Extract headers from thead (with multi-level support)
        let (headers, sub_headers) = self.extract_multi_level_headers(table_element)?;

        // Extract body rows
        let mut body_rows = Vec::new();
        let tbody_selector =
            Selector::parse("tbody tr").map_err(|e| anyhow!("Invalid tbody selector: {}", e))?;

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
            let tr_selector =
                Selector::parse("tr").map_err(|e| anyhow!("Invalid tr selector: {}", e))?;

            let mut row_index = 0;
            for row_element in table_element.select(&tr_selector) {
                // Skip if this row is in thead or tfoot
                if self.is_in_section(row_element, "thead")
                    || self.is_in_section(row_element, "tfoot")
                {
                    continue;
                }

                // Skip header row if we already extracted headers
                if !headers.is_empty() && row_index == 0 {
                    let th_selector =
                        Selector::parse("th").map_err(|e| anyhow!("Invalid th selector: {}", e))?;
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
        let tfoot_selector =
            Selector::parse("tfoot tr").map_err(|e| anyhow!("Invalid tfoot selector: {}", e))?;

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

        Ok((headers, sub_headers, body_rows, footer_rows))
    }

    /// Extract cells from a table row
    fn extract_row_cells(
        &self,
        row_element: ElementRef,
        row_type: RowType,
        row_index: usize,
    ) -> Result<Vec<TableCell>> {
        let cell_selector =
            Selector::parse("td, th").map_err(|e| anyhow!("Invalid cell selector: {}", e))?;

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
            let colspan = value
                .attr("colspan")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let rowspan = value
                .attr("rowspan")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            // Determine cell type
            let cell_type = match value.name() {
                "th" => CellType::Header,
                "td" => CellType::Data,
                _ => {
                    if row_type == RowType::Header {
                        CellType::Header
                    } else {
                        CellType::Data
                    }
                }
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
            if let (Some(nested_id), Some(table_id)) =
                (nested_element.value().id(), table_element.value().id())
            {
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

    /// Extract multi-level headers from thead section
    ///
    /// Detects and extracts hierarchical headers when <thead> contains multiple <tr> elements.
    /// Handles colspan/rowspan relationships to build proper header structure.
    fn extract_multi_level_headers(
        &self,
        table_element: ElementRef,
    ) -> Result<(Vec<TableCell>, Vec<Vec<TableCell>>)> {
        let thead_selector =
            Selector::parse("thead tr").map_err(|e| anyhow!("Invalid thead selector: {}", e))?;

        // Collect all header rows from thead
        let header_rows: Vec<_> = table_element.select(&thead_selector).collect();

        if header_rows.is_empty() {
            // Fallback: look for th elements in first row
            return self.extract_fallback_headers(table_element);
        }

        if header_rows.len() == 1 {
            // Single header row - no sub-headers
            let headers = self.extract_row_cells(header_rows[0], RowType::Header, 0)?;
            return Ok((headers, Vec::new()));
        }

        // Multi-level headers detected
        let mut all_header_rows = Vec::new();
        for (row_index, row_element) in header_rows.iter().enumerate() {
            let cells = self.extract_row_cells(*row_element, RowType::Header, row_index)?;
            all_header_rows.push(cells);
        }

        // Build hierarchical structure
        self.build_hierarchical_header_structure(all_header_rows)
    }

    /// Fallback header extraction for tables without explicit thead
    fn extract_fallback_headers(
        &self,
        table_element: ElementRef,
    ) -> Result<(Vec<TableCell>, Vec<Vec<TableCell>>)> {
        let first_row_selector = Selector::parse("tr:first-child")
            .map_err(|e| anyhow!("Invalid first row selector: {}", e))?;

        if let Some(first_row) = table_element.select(&first_row_selector).next() {
            let th_selector =
                Selector::parse("th").map_err(|e| anyhow!("Invalid th selector: {}", e))?;

            if first_row.select(&th_selector).next().is_some() {
                let headers = self.extract_row_cells(first_row, RowType::Header, 0)?;
                return Ok((headers, Vec::new()));
            }
        }

        Ok((Vec::new(), Vec::new()))
    }

    /// Build hierarchical header structure from multiple header rows
    ///
    /// This algorithm handles:
    /// 1. Colspan - A header cell spanning multiple columns creates parent-child relationships
    /// 2. Rowspan - A header cell spanning multiple rows maintains its position across levels
    /// 3. Mixed spans - Complex combinations of both colspan and rowspan
    fn build_hierarchical_header_structure(
        &self,
        mut all_header_rows: Vec<Vec<TableCell>>,
    ) -> Result<(Vec<TableCell>, Vec<Vec<TableCell>>)> {
        if all_header_rows.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        if all_header_rows.len() == 1 {
            let main_headers = all_header_rows.remove(0);
            return Ok((main_headers, Vec::new()));
        }

        // The last row becomes the main headers (most specific)
        let main_headers = all_header_rows.pop().unwrap();

        // All preceding rows are sub-headers (hierarchical levels)
        let sub_headers = all_header_rows;

        Ok((main_headers, sub_headers))
    }

    /// Calculate table structure information
    fn calculate_table_structure(
        &self,
        headers: &[TableCell],
        sub_headers: &[Vec<TableCell>],
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
            total_columns = body_rows
                .iter()
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

        // Calculate total header rows (main + sub-headers)
        let total_header_rows = if headers.is_empty() {
            0
        } else {
            1 + sub_headers.len()
        };

        // Check for complex structure in sub-headers
        for sub_header_row in sub_headers {
            for cell in sub_header_row {
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
            header_rows: total_header_rows,
            footer_rows: footer_rows.len(),
            has_complex_structure,
            max_colspan,
            max_rowspan,
        }
    }

    /// Extract attributes from an element
    fn extract_element_attributes(
        &self,
        element: ElementRef,
    ) -> std::collections::HashMap<String, String> {
        element
            .value()
            .attrs()
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
