//! Real-world HTML extraction tests

use anyhow::Result;
use riptide_html::*;

#[cfg(test)]
mod css_selector_tests {
    use super::*;
    use riptide_html::selectors::{CssExtractor, SelectorConfig, Transformer};

    #[test]
    fn test_basic_css_extraction() {
        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body>
                    <article class="main-content">
                        <h1 id="title">Main Title</h1>
                        <div class="author">John Doe</div>
                        <p class="content">This is the main content.</p>
                        <p class="content">Second paragraph.</p>
                    </article>
                </body>
            </html>
        "#;

        let config = SelectorConfig {
            title: Some("h1#title".to_string()),
            author: Some(".author".to_string()),
            content: Some("p.content".to_string()),
            ..Default::default()
        };

        let extractor = CssExtractor::new(config);
        let result = extractor.extract(html).unwrap();

        assert_eq!(result.title, Some("Main Title".to_string()));
        assert_eq!(result.author, Some("John Doe".to_string()));
        assert!(result.content.contains("This is the main content"));
        assert!(result.content.contains("Second paragraph"));
    }

    #[test]
    fn test_has_text_filter() {
        let html = r#"
            <div class="price">$99.99</div>
            <div class="price">Contact for pricing</div>
            <div class="price">€89.99</div>
        "#;

        let extractor = CssExtractor::default();
        let prices = extractor
            .select_with_filter(html, ".price", ":has-text($)")
            .unwrap();

        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0], "$99.99");
    }

    #[test]
    fn test_transformers() {
        let html = r#"
            <div class="data">
                <span class="price">  $1,234.56  </span>
                <span class="date">2024-03-15</span>
                <a href="/page" class="link">Link</a>
            </div>
        "#;

        let extractor = CssExtractor::default();

        // Test trim transformer
        let price = extractor
            .extract_and_transform(html, ".price", vec![Transformer::Trim])
            .unwrap();
        assert_eq!(price, Some("$1,234.56".to_string()));

        // Test number extraction
        let number = extractor
            .extract_and_transform(html, ".price", vec![Transformer::ExtractNumber])
            .unwrap();
        assert_eq!(number, Some("1234.56".to_string()));

        // Test date ISO transform
        let date = extractor
            .extract_and_transform(html, ".date", vec![Transformer::DateISO])
            .unwrap();
        assert_eq!(date, Some("2024-03-15T00:00:00Z".to_string()));

        // Test absolute URL
        let url = extractor
            .extract_and_transform(
                html,
                ".link",
                vec![Transformer::AbsoluteUrl("https://example.com".to_string())],
            )
            .unwrap();
        assert_eq!(url, Some("https://example.com/page".to_string()));
    }

    #[test]
    fn test_complex_selectors() {
        let html = r#"
            <div class="container">
                <div class="item">
                    <h3>Item 1</h3>
                    <p>Description 1</p>
                </div>
                <div class="item">
                    <h3>Item 2</h3>
                    <p>Description 2</p>
                </div>
                <div class="item featured">
                    <h3>Featured Item</h3>
                    <p>Special description</p>
                </div>
            </div>
        "#;

        let extractor = CssExtractor::default();

        // Test descendant selector
        let items = extractor.select_all(html, ".container .item h3").unwrap();
        assert_eq!(items.len(), 3);

        // Test multiple classes
        let featured = extractor.select_all(html, ".item.featured").unwrap();
        assert_eq!(featured.len(), 1);

        // Test nth-child
        let second = extractor
            .select(html, ".container .item:nth-child(2) h3")
            .unwrap();
        assert_eq!(second, Some("Item 2".to_string()));
    }
}

#[cfg(test)]
mod table_extraction_tests {
    use super::*;
    use riptide_html::tables::{TableConfig, TableExtractor, TableFormat};

    #[test]
    fn test_simple_table_extraction() {
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

        let extractor = TableExtractor::new(TableConfig::default());
        let tables = extractor.extract_tables(html).unwrap();

        assert_eq!(tables.len(), 1);
        let table = &tables[0];
        assert_eq!(table.headers, vec!["Name", "Age", "City"]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_complex_table_with_colspan_rowspan() {
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

        let extractor = TableExtractor::new(TableConfig {
            handle_colspan: true,
            handle_rowspan: true,
            ..Default::default()
        });

        let tables = extractor.extract_tables(html).unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert_eq!(table.rows[0][0], "Widget A");
        assert_eq!(table.rows[0].len(), 5); // Product + 4 data columns
    }

    #[test]
    fn test_table_to_csv_export() {
        let html = r#"
            <table>
                <tr><th>Name</th><th>Quote</th></tr>
                <tr><td>Alice</td><td>Hello, "world"!</td></tr>
                <tr><td>Bob</td><td>Line 1
Line 2</td></tr>
            </table>
        "#;

        let extractor = TableExtractor::new(TableConfig::default());
        let tables = extractor.extract_tables(html).unwrap();
        let csv = extractor.to_csv(&tables[0]).unwrap();

        assert!(csv.contains("\"Name\",\"Quote\""));
        assert!(csv.contains("\"Alice\",\"Hello, \"\"world\"\"!\""));
        assert!(csv.contains("\"Line 1\nLine 2\""));
    }

    #[test]
    fn test_table_to_markdown() {
        let html = r#"
            <table>
                <tr><th>Feature</th><th>Status</th></tr>
                <tr><td>CSS Extraction</td><td>✓</td></tr>
                <tr><td>Table Export</td><td>✓</td></tr>
            </table>
        "#;

        let extractor = TableExtractor::new(TableConfig::default());
        let tables = extractor.extract_tables(html).unwrap();
        let markdown = extractor.to_markdown(&tables[0]).unwrap();

        assert!(markdown.contains("| Feature | Status |"));
        assert!(markdown.contains("|---------|--------|"));
        assert!(markdown.contains("| CSS Extraction | ✓ |"));
    }

    #[test]
    fn test_nested_table_extraction() {
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

        let extractor = TableExtractor::new(TableConfig {
            extract_nested: true,
            ..Default::default()
        });

        let tables = extractor.extract_tables(html).unwrap();
        assert_eq!(tables.len(), 2);

        // Parent table
        assert!(tables[0].id.as_ref().unwrap().contains("parent"));

        // Nested table
        assert!(tables[1].id.as_ref().unwrap().contains("nested"));
        assert!(tables[1].parent_id.is_some());
    }
}

#[cfg(test)]
mod chunking_tests {
    use super::*;
    use riptide_html::chunking::{
        ChunkConfig, ChunkingStrategy, FixedSizeChunker, TextTilingChunker,
    };

    #[test]
    fn test_fixed_size_chunking() {
        let text = "This is a test document. ".repeat(100);
        let config = ChunkConfig {
            chunk_size: 100,
            chunk_overlap: 20,
            ..Default::default()
        };

        let chunker = FixedSizeChunker::new(config);
        let chunks = chunker.chunk(&text).unwrap();

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.text.len() <= 100);
        }

        // Test overlap
        for i in 1..chunks.len() {
            let prev_end = &chunks[i - 1].text[chunks[i - 1].text.len() - 20..];
            let curr_start = &chunks[i].text[..20.min(chunks[i].text.len())];
            assert_eq!(prev_end, curr_start);
        }
    }

    #[test]
    fn test_text_tiling_chunking() {
        let text = r#"
            Introduction to Machine Learning

            Machine learning is a subset of artificial intelligence.
            It involves training algorithms to make predictions.
            The field has grown rapidly in recent years.

            Types of Machine Learning

            There are three main types: supervised, unsupervised, and reinforcement.
            Supervised learning uses labeled data.
            Unsupervised learning finds patterns in unlabeled data.

            Applications

            ML is used in many industries today.
            Healthcare uses it for diagnosis.
            Finance uses it for fraud detection.
        "#;

        let chunker = TextTilingChunker::new(ChunkConfig::default());
        let chunks = chunker.chunk(text).unwrap();

        // Should detect topic boundaries
        assert!(chunks.len() >= 3);

        // Each chunk should contain related content
        assert!(chunks[0].text.contains("Introduction"));
        assert!(chunks[1].text.contains("Types"));
        assert!(chunks[2].text.contains("Applications"));
    }

    #[test]
    fn test_sentence_based_chunking() {
        let text = "This is sentence one. This is sentence two! And sentence three? Sentence four.";

        let config = ChunkConfig {
            chunk_size: 50,
            respect_sentence_boundaries: true,
            ..Default::default()
        };

        let chunker = FixedSizeChunker::new(config);
        let chunks = chunker.chunk(text).unwrap();

        // Should not split sentences
        for chunk in chunks {
            assert!(
                chunk.text.ends_with('.') || chunk.text.ends_with('!') || chunk.text.ends_with('?')
            );
        }
    }
}

#[cfg(test)]
mod real_world_extraction_tests {
    use super::*;

    #[test]
    fn test_news_article_extraction() {
        let html = r#"
            <article class="news-article">
                <header>
                    <h1 class="headline">Breaking: Major Scientific Discovery</h1>
                    <div class="byline">
                        <span class="author">Dr. Jane Smith</span>
                        <time datetime="2024-03-15T10:00:00Z">March 15, 2024</time>
                    </div>
                </header>
                <div class="article-body">
                    <p class="lead">Scientists have made a groundbreaking discovery that could change our understanding of the universe.</p>
                    <p>The research, published in Nature, reveals new insights into quantum mechanics.</p>
                    <p>This finding has implications for future technology development.</p>
                </div>
                <footer class="article-tags">
                    <span class="tag">Science</span>
                    <span class="tag">Physics</span>
                    <span class="tag">Research</span>
                </footer>
            </article>
        "#;

        let extractor = CssExtractor::new(SelectorConfig {
            title: Some(".headline".to_string()),
            author: Some(".author".to_string()),
            content: Some(".article-body p".to_string()),
            published_date: Some("time[datetime]".to_string()),
            tags: Some(".tag".to_string()),
            ..Default::default()
        });

        let result = extractor.extract(html).unwrap();

        assert_eq!(
            result.title.unwrap(),
            "Breaking: Major Scientific Discovery"
        );
        assert_eq!(result.author.unwrap(), "Dr. Jane Smith");
        assert!(result.content.contains("groundbreaking discovery"));
        assert_eq!(result.tags.len(), 3);
        assert!(result.tags.contains(&"Science".to_string()));
    }

    #[test]
    fn test_ecommerce_product_extraction() {
        let html = r#"
            <div class="product-page">
                <h1 class="product-title">Premium Laptop</h1>
                <div class="price-container">
                    <span class="original-price">$1,499.99</span>
                    <span class="sale-price">$1,199.99</span>
                    <span class="discount">-20%</span>
                </div>
                <div class="product-details">
                    <ul class="specs">
                        <li>Intel Core i7 Processor</li>
                        <li>16GB RAM</li>
                        <li>512GB SSD</li>
                    </ul>
                </div>
                <div class="availability" data-stock="15">In Stock</div>
                <div class="reviews">
                    <span class="rating">4.5</span>
                    <span class="review-count">(324 reviews)</span>
                </div>
            </div>
        "#;

        let extractor = CssExtractor::default();

        // Extract product information
        let title = extractor.select(html, ".product-title").unwrap().unwrap();
        assert_eq!(title, "Premium Laptop");

        // Extract and transform price
        let price = extractor
            .extract_and_transform(html, ".sale-price", vec![Transformer::ExtractNumber])
            .unwrap()
            .unwrap();
        assert_eq!(price, "1199.99");

        // Extract specifications
        let specs = extractor.select_all(html, ".specs li").unwrap();
        assert_eq!(specs.len(), 3);
        assert!(specs.contains(&"16GB RAM".to_string()));

        // Extract stock data
        let stock = extractor
            .select_attr(html, ".availability", "data-stock")
            .unwrap()
            .unwrap();
        assert_eq!(stock, "15");
    }
}
