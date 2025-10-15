use riptide_core::enhanced_extractor::StructuredExtractor;

fn main() {
    let html = r#"
        <html>
        <body>
            <article>
                <h1>Main Title</h1>
                <p>First paragraph with some content.</p>
                <h2>Section Header</h2>
                <p>Another paragraph.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
                <p>Final paragraph with <a href="/link">a link</a>.</p>
            </article>
        </body>
        </html>
    "#;

    let content =
        StructuredExtractor::extract_structured_content(html, Some("https://example.com")).unwrap();
    println!("Content:\n{}", content);
    println!("\nChecking for: [a link](https://example.com/link)");
    println!(
        "Contains: {}",
        content.contains("[a link](https://example.com/link)")
    );
}
