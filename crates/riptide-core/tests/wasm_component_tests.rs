mod support;

use support::wasm_component;

#[test]
fn test_component_availability() {
    if !wasm_component::component_available() {
        eprintln!("WASM component not found. Please build it first with:");
        eprintln!("cargo build -p riptide-extractor-wasm --target wasm32-wasip2 --release");
        panic!("WASM component not available");
    }
}

#[test]
fn test_simple_extraction() {
    assert!(
        wasm_component::component_available(),
        "WASM component not available. Build with: cargo build -p riptide-extractor-wasm --target wasm32-wasip2 --release"
    );

    let html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
    let result = wasm_component::extract_content(html, "https://test.com", "article")
        .expect("Should extract from valid HTML");

    assert_eq!(result.url, "https://test.com");
    assert_eq!(result.title, Some("Test".to_string()));
    assert!(result.text.contains("Content"));
}

#[test]
fn test_article_extraction() {
    assert!(
        wasm_component::component_available(),
        "WASM component not available. Build with: cargo build -p riptide-extractor-wasm --target wasm32-wasip2 --release"
    );

    let html = r#"
        <html>
        <head><title>Article Title</title></head>
        <body>
            <article>
                <h1>Article Title</h1>
                <p>First paragraph of the article.</p>
                <p>Second paragraph with more content.</p>
                <a href="https://example.com/link">Example link</a>
            </article>
        </body>
        </html>
    "#;

    let result = wasm_component::extract_content(html, "https://test.com/article", "article")
        .expect("Should extract article content");

    assert_eq!(result.url, "https://test.com/article");
    assert_eq!(result.title, Some("Article Title".to_string()));
    assert!(result.text.contains("First paragraph"));
    assert!(result.text.contains("Second paragraph"));
    assert!(result.links.len() > 0);
}
