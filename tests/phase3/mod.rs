/// Phase 3 implementation tests for Crawl4AI parity features
///
/// This module contains comprehensive tests for the new dynamic content handling,
/// stealth features, and PDF processing capabilities introduced in Phase 3.

pub mod dynamic_tests;
pub mod stealth_tests;
pub mod pdf_tests;
pub mod integration_tests;

// Re-export test utilities
pub use dynamic_tests::*;
pub use stealth_tests::*;
pub use pdf_tests::*;

/// Test utilities for Phase 3 features
pub mod test_utils {
    use riptide_core::dynamic::{DynamicConfig, WaitCondition, ScrollConfig, PageAction};
    use riptide_core::stealth::StealthConfig;
    use riptide_core::pdf::PdfConfig;
    use std::time::Duration;

    /// Create a basic dynamic config for testing
    pub fn create_test_dynamic_config() -> DynamicConfig {
        DynamicConfig {
            wait_for: Some(WaitCondition::DomContentLoaded),
            scroll: Some(ScrollConfig::default()),
            actions: vec![
                PageAction::Wait(WaitCondition::DomContentLoaded),
                PageAction::Screenshot {
                    filename: Some("test.png".to_string()),
                    full_page: true,
                },
            ],
            capture_artifacts: true,
            timeout: Duration::from_secs(30),
            viewport: None,
        }
    }

    /// Create a basic stealth config for testing
    pub fn create_test_stealth_config() -> StealthConfig {
        StealthConfig::default()
    }

    /// Create a basic PDF config for testing
    pub fn create_test_pdf_config() -> PdfConfig {
        PdfConfig::default()
    }

    /// Generate test PDF data with valid header
    pub fn create_test_pdf_data() -> Vec<u8> {
        let pdf_content = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n\
1 0 obj\n\
<<\n\
/Type /Catalog\n\
/Pages 2 0 R\n\
>>\n\
endobj\n\
\n\
2 0 obj\n\
<<\n\
/Type /Pages\n\
/Kids [3 0 R]\n\
/Count 1\n\
>>\n\
endobj\n\
\n\
3 0 obj\n\
<<\n\
/Type /Page\n\
/Parent 2 0 R\n\
/MediaBox [0 0 612 792]\n\
/Resources <<\n\
  /Font <<\n\
    /F1 <<\n\
      /Type /Font\n\
      /Subtype /Type1\n\
      /BaseFont /Helvetica\n\
    >>\n\
  >>\n\
>>\n\
/Contents 4 0 R\n\
>>\n\
endobj\n\
\n\
4 0 obj\n\
<<\n\
/Length 44\n\
>>\n\
stream\n\
BT\n\
/F1 12 Tf\n\
100 700 Td\n\
(Hello, PDF World!) Tj\n\
ET\n\
endstream\n\
endobj\n\
\n\
xref\n\
0 5\n\
0000000000 65535 f \n\
0000000015 00000 n \n\
0000000074 00000 n \n\
0000000131 00000 n \n\
0000000384 00000 n \n\
trailer\n\
<<\n\
/Size 5\n\
/Root 1 0 R\n\
>>\n\
startxref\n\
478\n\
%%EOF";

        pdf_content.to_vec()
    }

    /// Generate test HTML with dynamic content
    pub fn create_test_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <meta name="description" content="A test page for dynamic rendering">
    <meta property="og:title" content="Test Page">
    <meta property="og:description" content="Test description">
</head>
<body>
    <div class="loading" style="display: block;">Loading...</div>
    <div class="content" style="display: none;">
        <h1>Dynamic Content</h1>
        <p>This content loads dynamically.</p>
        <button class="show-more">Show More</button>
        <div class="more-content" style="display: none;">
            <p>Additional content revealed by clicking.</p>
        </div>
    </div>
    <script>
        setTimeout(() => {
            document.querySelector('.loading').style.display = 'none';
            document.querySelector('.content').style.display = 'block';
            window.dataLoaded = true;
        }, 1000);

        document.querySelector('.show-more').addEventListener('click', () => {
            document.querySelector('.more-content').style.display = 'block';
        });
    </script>
</body>
</html>"#.to_string()
    }

    /// Create mock HTTP response headers
    pub fn create_test_headers() -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();
        headers.insert("content-type".to_string(), "text/html; charset=utf-8".to_string());
        headers.insert("content-length".to_string(), "1024".to_string());
        headers.insert("server".to_string(), "nginx/1.18.0".to_string());
        headers.insert("x-custom-header".to_string(), "test-value".to_string());
        headers
    }
}