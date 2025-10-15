use riptide_core::enhanced_extractor::StructuredExtractor;

fn main() {
    let html = r#"<!DOCTYPE html>
<html>
<body>
<article>
<h1>Comprehensive Markdown Test</h1>
<h2>Inline Formatting</h2>
<p>Regular text with <strong>bold</strong>, <em>italic</em>, <code>code</code>, and <a href="https://example.com">links</a>.</p>
<p>Nested: <strong><em>bold+italic</em></strong>, <a href="/test"><strong>bold link</strong></a>, <code><strong>bold code</strong></code>.</p>

<h2>Lists</h2>
<ul>
<li>Unordered with <b>bold</b></li>
<li>Item with <i>italic</i></li>
<li>Item with <code>code</code></li>
<li>Item with <a href="https://example.com">link</a></li>
</ul>

<ol>
<li>Ordered with <strong>bold text</strong></li>
<li>Second with <em>italic text</em></li>
</ol>

<h2>Code Blocks</h2>
<pre>fn main() {
    println!("Hello, world!");
}</pre>

<h2>Blockquotes</h2>
<blockquote>
This is a quote with <strong>bold</strong> and <em>italic</em> text inside.
It can span multiple lines.
</blockquote>

<h2>Tables</h2>
<table>
<thead>
<tr>
<th>Header 1</th>
<th>Header 2</th>
</tr>
</thead>
<tbody>
<tr>
<td>Cell with <b>bold</b></td>
<td>Cell with <i>italic</i></td>
</tr>
<tr>
<td>Plain cell</td>
<td>Cell with <code>code</code></td>
</tr>
</tbody>
</table>

<h2>Mixed Content</h2>
<p>A paragraph with <strong>bold <em>and nested italic</em> text</strong> and a
<a href="https://example.com"><code>code link</code></a>.</p>

<hr>

<p>Final paragraph with <br>line<br>breaks.</p>

</article>
</body>
</html>"#;

    let result = StructuredExtractor::extract_structured_content(html, Some("https://test.com"))
        .expect("Extraction failed");

    println!("============ COMPREHENSIVE MARKDOWN OUTPUT ============\n");
    println!("{}", result);
    println!("\n============ VALIDATION ============");

    // Validate expected markdown elements
    let tests = vec![
        ("Headers", result.contains("# Comprehensive Markdown Test")),
        ("Bold", result.contains("**bold**")),
        ("Italic", result.contains("*italic*")),
        ("Inline code", result.contains("`code`")),
        ("Links", result.contains("[links](https://example.com)")),
        ("Bold+Italic nested", result.contains("***")),
        ("Bold link", result.contains("[**bold link**]")),
        ("Lists", result.contains("- Unordered with")),
        ("Ordered lists", result.contains("1. Ordered with")),
        ("Code blocks", result.contains("```")),
        ("Blockquotes", result.contains(">")),
        ("Tables", result.contains("|")),
        ("Horizontal rule", result.contains("---")),
    ];

    println!("\nValidation Results:");
    for (name, passed) in tests {
        println!("  {} {}", if passed { "✓" } else { "✗" }, name);
    }
}
