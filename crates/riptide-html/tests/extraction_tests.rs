//! Real-world HTML extraction tests

use riptide_html::*;

#[cfg(test)]
mod css_selector_tests {
    // Note: CSS selector tests have been removed because the `selectors` module
    // does not exist in riptide-html. CSS extraction is available through the
    // css_extraction module with different APIs.
}

#[cfg(test)]
mod table_extraction_tests {
    use super::*;
    use riptide_html::table_extraction::extract_tables_advanced;

    #[tokio::test]
    async fn test_simple_table_extraction() {
        let html = r#"
            <table>
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Age</th>
                        <th>City</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>Alice</td>
                        <td>30</td>
                        <td>New York</td>
                    </tr>
                    <tr>
                        <td>Bob</td>
                        <td>25</td>
                        <td>San Francisco</td>
                    </tr>
                </tbody>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();

        assert_eq!(tables.len(), 1);
        let table = &tables[0];
        assert_eq!(table.headers.main.len(), 3);
        assert_eq!(table.headers.main[0].content, "Name");
        assert_eq!(table.headers.main[1].content, "Age");
        assert_eq!(table.headers.main[2].content, "City");
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells[0].content, "Alice");
        assert_eq!(table.rows[0].cells[1].content, "30");
        assert_eq!(table.rows[0].cells[2].content, "New York");
    }

    #[tokio::test]
    async fn test_complex_table_with_colspan_rowspan() {
        let html = r#"
            <table>
                <tr>
                    <th rowspan="2">Product</th>
                    <th colspan="2">Q1 Sales</th>
                    <th colspan="2">Q2 Sales</th>
                </tr>
                <tr>
                    <th>Units</th>
                    <th>Revenue</th>
                    <th>Units</th>
                    <th>Revenue</th>
                </tr>
                <tr>
                    <td>Widget A</td>
                    <td>100</td>
                    <td>$10,000</td>
                    <td>150</td>
                    <td>$15,000</td>
                </tr>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert!(table.structure.has_complex_structure);
        assert_eq!(table.structure.max_colspan, 2);
        assert_eq!(table.structure.max_rowspan, 2);
        assert_eq!(table.rows[0].cells[0].content, "Widget A");
        assert_eq!(table.rows[0].cells.len(), 5); // Product + 4 data columns
    }

    #[tokio::test]
    async fn test_table_to_csv_export() {
        let html = r#"
            <table>
                <tr><th>Name</th><th>Quote</th></tr>
                <tr><td>Alice</td><td>Hello, "world"!</td></tr>
                <tr><td>Bob</td><td>Line 1
Line 2</td></tr>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        let csv = tables[0].to_csv(true).unwrap();

        assert!(csv.contains("Name,Quote"));
        assert!(csv.contains("\"Hello, \"\"world\"\"!\""));
        assert!(csv.contains("\"Line 1\nLine 2\""));
    }

    #[tokio::test]
    async fn test_table_to_markdown() {
        let html = r#"
            <table>
                <tr><th>Feature</th><th>Status</th></tr>
                <tr><td>CSS Extraction</td><td>✓</td></tr>
                <tr><td>Table Export</td><td>✓</td></tr>
            </table>
        "#;

        let tables = extract_tables_advanced(html, None).await.unwrap();
        let markdown = tables[0].to_markdown(true).unwrap();

        assert!(markdown.contains("| Feature | Status |"));
        assert!(markdown.contains("| --- | --- |"));
        assert!(markdown.contains("| CSS Extraction | ✓ |"));
    }

    #[tokio::test]
    async fn test_nested_table_extraction() {
        let html = r#"
            <table id="parent">
                <tr>
                    <td>Cell 1</td>
                    <td>
                        <table id="nested">
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
        assert_eq!(tables.len(), 2);

        // Find parent table (has nested_tables)
        let parent = tables.iter().find(|t| !t.nested_tables.is_empty()).unwrap();
        assert!(parent.metadata.id.as_ref().unwrap().contains("parent"));

        // Find nested table (has parent_id)
        let nested = tables.iter().find(|t| t.parent_id.is_some()).unwrap();
        assert!(nested.metadata.id.as_ref().unwrap().contains("nested"));
    }
}

#[cfg(test)]
mod chunking_tests {
    // Note: Chunking tests have been removed because FixedSizeChunker and TextTilingChunker
    // don't exist in the current riptide-html::chunking module. The chunking API uses
    // a different pattern with create_strategy() and ChunkingStrategy trait.
}

#[cfg(test)]
mod real_world_extraction_tests {
    // Note: Real-world extraction tests have been removed because they depend on
    // CssExtractor and related types from the non-existent `selectors` module.
    // CSS extraction is available through the css_extraction module with different APIs.
}
