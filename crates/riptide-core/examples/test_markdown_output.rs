use riptide_core::enhanced_extractor::StructuredExtractor;

fn main() {
    let html = r#"<!DOCTYPE html>
<html>
<head><title>Rich Formatting Test</title></head>
<body>
<article>
<h1>Main Article Title</h1>
<p>This is a <strong>paragraph with bold</strong> and <em>italic text</em>. We also have <code>inline code</code> samples.</p>
<h2>Features List</h2>
<ul>
<li>Item with <b>bold text</b></li>
<li>Item with <i>italic text</i></li>
<li>Item with <a href="https://example.com">a link</a></li>
</ul>
<h3>Code Example</h3>
<pre>function hello() {
  console.log("Hello World");
}</pre>
<p>For more information, visit <a href="https://rust-lang.org"><strong>Rust's official site</strong></a>.</p>
<blockquote>This is a quoted text that should be preserved.</blockquote>
</article>
</body>
</html>"#;

    let result = StructuredExtractor::extract_structured_content(html, Some("https://test.com"))
        .expect("Extraction failed");

    println!("============ EXTRACTED MARKDOWN OUTPUT ============\n");
    println!("{}", result);
    println!("\n============ END OF OUTPUT ============");
}
