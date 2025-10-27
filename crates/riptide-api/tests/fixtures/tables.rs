//! Table extraction test fixtures
//!
//! Provides sample table data for testing table extraction and export endpoints

use serde::{Deserialize, Serialize};

/// Test fixture for extracted table data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFixture {
    /// Unique table identifier
    pub id: String,
    /// Source URL where table was extracted from
    pub source_url: Option<String>,
    /// Raw HTML content
    pub html_content: Option<String>,
    /// Table headers
    pub headers: Vec<String>,
    /// Table data rows
    pub data: Vec<Vec<String>>,
    /// Number of rows (including header)
    pub rows: usize,
    /// Number of columns
    pub columns: usize,
    /// Whether table has colspan/rowspan
    pub has_spans: bool,
    /// Additional metadata
    pub metadata: TableMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    /// Table element ID attribute
    pub element_id: Option<String>,
    /// Table classes
    pub classes: Vec<String>,
    /// Extraction timestamp
    pub extracted_at: String,
    /// Data types detected
    pub data_types: Vec<String>,
}

/// Get default table fixtures for testing
pub fn get_default_table_fixtures() -> Vec<TableFixture> {
    vec![
        // Simple product table
        TableFixture {
            id: "table_12345".to_string(),
            source_url: Some("https://example.com/products".to_string()),
            html_content: Some(get_products_table_html()),
            headers: vec![
                "Product ID".to_string(),
                "Name".to_string(),
                "Price".to_string(),
                "Category".to_string(),
            ],
            data: vec![
                vec!["001".to_string(), "Laptop".to_string(), "$999.99".to_string(), "Electronics".to_string()],
                vec!["002".to_string(), "Mouse".to_string(), "$24.99".to_string(), "Accessories".to_string()],
                vec!["003".to_string(), "Keyboard".to_string(), "$79.99".to_string(), "Accessories".to_string()],
            ],
            rows: 4, // 1 header + 3 data
            columns: 4,
            has_spans: false,
            metadata: TableMetadata {
                element_id: Some("products".to_string()),
                classes: vec!["data-table".to_string()],
                extracted_at: "2025-10-27T00:00:00Z".to_string(),
                data_types: vec!["string".to_string(), "string".to_string(), "currency".to_string(), "string".to_string()],
            },
        },

        // Complex table with spans
        TableFixture {
            id: "table_67890".to_string(),
            source_url: Some("https://example.com/reports".to_string()),
            html_content: Some(get_complex_table_html()),
            headers: vec![
                "Quarter".to_string(),
                "Sales".to_string(),
                "Profit".to_string(),
                "Notes".to_string(),
            ],
            data: vec![
                vec!["Q1 2024".to_string(), "$10,000".to_string(), "$2,000".to_string(), "Good performance".to_string()],
                vec!["Q2 2024".to_string(), "$15,000".to_string(), "$3,500".to_string(), "Strong growth".to_string()],
                vec!["Q3 2024".to_string(), "$12,000".to_string(), "$2,800".to_string(), "Seasonal dip".to_string()],
            ],
            rows: 4,
            columns: 4,
            has_spans: true,
            metadata: TableMetadata {
                element_id: Some("quarterly-report".to_string()),
                classes: vec!["financial-table".to_string(), "complex".to_string()],
                extracted_at: "2025-10-27T00:00:00Z".to_string(),
                data_types: vec!["string".to_string(), "currency".to_string(), "currency".to_string(), "string".to_string()],
            },
        },

        // Financial data table
        TableFixture {
            id: "table_financial_001".to_string(),
            source_url: Some("https://example.com/financial-reports".to_string()),
            html_content: Some(get_financial_table_html()),
            headers: vec![
                "Metric".to_string(),
                "2023".to_string(),
                "2024".to_string(),
                "Change %".to_string(),
            ],
            data: vec![
                vec!["Revenue".to_string(), "$1,200,000".to_string(), "$1,500,000".to_string(), "+25%".to_string()],
                vec!["Expenses".to_string(), "$800,000".to_string(), "$950,000".to_string(), "+18.75%".to_string()],
                vec!["Net Income".to_string(), "$400,000".to_string(), "$550,000".to_string(), "+37.5%".to_string()],
            ],
            rows: 4,
            columns: 4,
            has_spans: false,
            metadata: TableMetadata {
                element_id: Some("financial-metrics".to_string()),
                classes: vec!["financial-data".to_string()],
                extracted_at: "2025-10-27T00:00:00Z".to_string(),
                data_types: vec!["string".to_string(), "currency".to_string(), "currency".to_string(), "percentage".to_string()],
            },
        },
    ]
}

/// Get products table HTML
fn get_products_table_html() -> String {
    r#"
    <table id="products" class="data-table">
        <thead>
            <tr>
                <th>Product ID</th>
                <th>Name</th>
                <th>Price</th>
                <th>Category</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>001</td>
                <td>Laptop</td>
                <td>$999.99</td>
                <td>Electronics</td>
            </tr>
            <tr>
                <td>002</td>
                <td>Mouse</td>
                <td>$24.99</td>
                <td>Accessories</td>
            </tr>
            <tr>
                <td>003</td>
                <td>Keyboard</td>
                <td>$79.99</td>
                <td>Accessories</td>
            </tr>
        </tbody>
    </table>
    "#.to_string()
}

/// Get complex table with spans HTML
fn get_complex_table_html() -> String {
    r#"
    <table id="quarterly-report" class="financial-table complex">
        <thead>
            <tr>
                <th rowspan="2">Quarter</th>
                <th colspan="2">Financial Metrics</th>
                <th rowspan="2">Notes</th>
            </tr>
            <tr>
                <th>Sales</th>
                <th>Profit</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>Q1 2024</td>
                <td>$10,000</td>
                <td>$2,000</td>
                <td>Good performance</td>
            </tr>
            <tr>
                <td>Q2 2024</td>
                <td>$15,000</td>
                <td>$3,500</td>
                <td>Strong growth</td>
            </tr>
            <tr>
                <td>Q3 2024</td>
                <td>$12,000</td>
                <td>$2,800</td>
                <td>Seasonal dip</td>
            </tr>
        </tbody>
    </table>
    "#.to_string()
}

/// Get financial table HTML
fn get_financial_table_html() -> String {
    r#"
    <table id="financial-metrics" class="financial-data">
        <thead>
            <tr>
                <th>Metric</th>
                <th>2023</th>
                <th>2024</th>
                <th>Change %</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>Revenue</td>
                <td>$1,200,000</td>
                <td>$1,500,000</td>
                <td>+25%</td>
            </tr>
            <tr>
                <td>Expenses</td>
                <td>$800,000</td>
                <td>$950,000</td>
                <td>+18.75%</td>
            </tr>
            <tr>
                <td>Net Income</td>
                <td>$400,000</td>
                <td>$550,000</td>
                <td>+37.5%</td>
            </tr>
        </tbody>
    </table>
    "#.to_string()
}

/// Convert table to CSV format for export testing
pub fn table_to_csv(table: &TableFixture) -> String {
    let mut csv = String::new();

    // Add headers
    csv.push_str(&table.headers.join(","));
    csv.push('\n');

    // Add data rows
    for row in &table.data {
        csv.push_str(&row.join(","));
        csv.push('\n');
    }

    csv
}

/// Convert table to Markdown format for export testing
pub fn table_to_markdown(table: &TableFixture) -> String {
    let mut md = String::new();

    // Add headers
    md.push_str("| ");
    md.push_str(&table.headers.join(" | "));
    md.push_str(" |\n");

    // Add separator
    md.push_str("|");
    for _ in &table.headers {
        md.push_str(" --- |");
    }
    md.push('\n');

    // Add data rows
    for row in &table.data {
        md.push_str("| ");
        md.push_str(&row.join(" | "));
        md.push_str(" |\n");
    }

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_fixtures() {
        let fixtures = get_default_table_fixtures();
        assert_eq!(fixtures.len(), 3, "Should have 3 default table fixtures");

        // Verify table_12345 exists
        let table = fixtures.iter().find(|t| t.id == "table_12345");
        assert!(table.is_some(), "Should have table_12345 fixture");
    }

    #[test]
    fn test_table_to_csv() {
        let fixtures = get_default_table_fixtures();
        let table = &fixtures[0];

        let csv = table_to_csv(table);
        assert!(csv.contains("Product ID,Name,Price,Category"), "CSV should contain headers");
        assert!(csv.contains("001,Laptop,$999.99,Electronics"), "CSV should contain data");
    }

    #[test]
    fn test_table_to_markdown() {
        let fixtures = get_default_table_fixtures();
        let table = &fixtures[0];

        let md = table_to_markdown(table);
        assert!(md.contains("| Product ID | Name | Price | Category |"), "Markdown should contain headers");
        assert!(md.contains("| --- |"), "Markdown should contain separator");
        assert!(md.contains("| 001 | Laptop | $999.99 | Electronics |"), "Markdown should contain data");
    }
}
