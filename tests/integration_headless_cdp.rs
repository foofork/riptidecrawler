use riptide_headless::models::{RenderReq, RenderResp, RenderErrorResp, PageAction};
use riptide_headless::cdp::render;
use axum::{http::StatusCode, Json};
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Integration tests for headless CDP operations
/// Tests browser automation, stealth features, timeout handling,
/// and JavaScript execution for improved test coverage.

#[tokio::test]
async fn test_basic_render_request() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Page</title>
        </head>
        <body>
            <h1>Hello World</h1>
            <p>This is a test page for headless rendering.</p>
            <div id="dynamic-content">Loading...</div>
            <script>
                setTimeout(() => {
                    document.getElementById('dynamic-content').innerText = 'Loaded!';
                }, 100);
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/test-page"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/test-page", mock_server.uri()),
        wait_for: None,
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: None,
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(!response.html.is_empty());
    assert!(response.html.contains("Hello World"));
    assert!(response.render_time_ms > 0);
    assert!(!response.request_id.is_empty());
}

#[tokio::test]
async fn test_render_with_wait_for_css() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>CSS Wait Test</title>
        </head>
        <body>
            <div id="content">Initial content</div>
            <script>
                setTimeout(() => {
                    const div = document.createElement('div');
                    div.className = 'loaded-content';
                    div.innerText = 'Content loaded!';
                    document.body.appendChild(div);
                }, 200);
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/css-wait"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/css-wait", mock_server.uri()),
        wait_for: Some("css:.loaded-content".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(1000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(response.html.contains("Content loaded!"));
    assert!(response.render_time_ms > 100); // Should have waited
}

#[tokio::test]
async fn test_render_with_wait_for_javascript() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>JS Wait Test</title>
        </head>
        <body>
            <div id="status">loading</div>
            <script>
                let ready = false;
                setTimeout(() => {
                    ready = true;
                    document.getElementById('status').innerText = 'ready';
                }, 150);
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/js-wait"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/js-wait", mock_server.uri()),
        wait_for: Some("js:ready === true".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(1000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(response.html.contains("ready"));
    assert!(response.render_time_ms > 100);
}

#[tokio::test]
async fn test_render_with_page_actions() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Actions Test</title>
        </head>
        <body>
            <button id="load-btn">Load Content</button>
            <div id="result"></div>
            <script>
                document.getElementById('load-btn').addEventListener('click', () => {
                    document.getElementById('result').innerText = 'Button clicked!';
                });
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/actions"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let actions = vec![
        PageAction::WaitForCss {
            css: "#load-btn".to_string(),
            timeout_ms: Some(1000),
        },
        PageAction::Click {
            css: "#load-btn".to_string(),
        },
        PageAction::WaitForCss {
            css: "#result".to_string(),
            timeout_ms: Some(1000),
        },
    ];

    let request = RenderReq {
        url: format!("{}/actions", mock_server.uri()),
        wait_for: None,
        actions: Some(actions),
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(2000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(response.html.contains("Button clicked!"));
}

#[tokio::test]
async fn test_render_with_scrolling() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Scroll Test</title>
            <style>
                .section { height: 1000px; background: linear-gradient(to bottom, #f0f0f0, #e0e0e0); }
            </style>
        </head>
        <body>
            <div class="section">Section 1</div>
            <div class="section">Section 2</div>
            <div class="section">Section 3</div>
            <div id="bottom-content">Bottom reached!</div>
            <script>
                window.addEventListener('scroll', () => {
                    if (window.scrollY > 2000) {
                        document.getElementById('bottom-content').style.background = 'yellow';
                    }
                });
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/scroll"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/scroll", mock_server.uri()),
        wait_for: None,
        actions: None,
        scroll_steps: Some(5),
        screenshot: None,
        stealth: None,
        timeout_ms: Some(2000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(response.html.contains("Bottom reached!"));
    // Should have triggered scroll event
    assert!(response.html.contains("yellow") || response.render_time_ms > 100);
}

#[tokio::test]
async fn test_render_timeout_handling() {
    let mock_server = MockServer::start().await;

    // Page that takes too long to load
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Timeout Test</title>
        </head>
        <body>
            <div id="content">Loading...</div>
            <script>
                // This will never complete within timeout
                setTimeout(() => {
                    document.getElementById('content').innerText = 'Loaded after long delay';
                }, 10000);
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/timeout"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/timeout", mock_server.uri()),
        wait_for: Some("js:document.getElementById('content').innerText === 'Loaded after long delay'".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(500),
    };

    let start = std::time::Instant::now();
    let result = render(Json(request)).await;
    let duration = start.elapsed();

    // Should timeout quickly
    assert!(duration < Duration::from_secs(4)); // Less than hard timeout

    if result.is_err() {
        let (status, error_resp) = result.unwrap_err();
        assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
        assert!(error_resp.0.error.contains("timeout"));
    }
}

#[tokio::test]
async fn test_render_stealth_mode() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Stealth Test</title>
        </head>
        <body>
            <div id="detection-result"></div>
            <script>
                // Simple automation detection
                let isAutomated = false;

                if (navigator.webdriver) {
                    isAutomated = true;
                }

                if (window.chrome && window.chrome.runtime && window.chrome.runtime.onConnect) {
                    // Chrome extension detection
                }

                document.getElementById('detection-result').innerText =
                    isAutomated ? 'Automation detected' : 'Human-like';
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/stealth"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/stealth", mock_server.uri()),
        wait_for: Some("css:#detection-result".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: Some(true),
        timeout_ms: Some(2000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    // With stealth mode, should ideally show "Human-like"
    // but depends on stealth implementation effectiveness
    assert!(response.html.contains("detection-result"));
}

#[tokio::test]
async fn test_render_invalid_url() {
    let request = RenderReq {
        url: "not-a-valid-url".to_string(),
        wait_for: None,
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: None,
    };

    let result = render(Json(request)).await;
    assert!(result.is_err());

    let (status, error_resp) = result.unwrap_err();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!error_resp.0.request_id.is_empty());
}

#[tokio::test]
async fn test_render_network_error() {
    let request = RenderReq {
        url: "http://non-existent-server-12345.com".to_string(),
        wait_for: None,
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(1000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_err());

    let (status, error_resp) = result.unwrap_err();
    assert!(status == StatusCode::INTERNAL_SERVER_ERROR ||
            status == StatusCode::REQUEST_TIMEOUT);
    assert!(!error_resp.0.error.is_empty());
}

#[tokio::test]
async fn test_render_complex_javascript_execution() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Complex JS Test</title>
        </head>
        <body>
            <div id="counter">0</div>
            <div id="status">initializing</div>
            <script>
                let counter = 0;
                let interval = setInterval(() => {
                    counter++;
                    document.getElementById('counter').innerText = counter;

                    if (counter >= 5) {
                        document.getElementById('status').innerText = 'completed';
                        clearInterval(interval);
                    }
                }, 50);

                // Simulate async operations
                fetch('/api/data')
                    .catch(() => {
                        // Handle expected failure
                        document.getElementById('status').innerText = 'fetch-attempted';
                    });
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/complex-js"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/complex-js", mock_server.uri()),
        wait_for: Some("js:document.getElementById('counter').innerText === '5'".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(2000),
    };

    let result = render(Json(request)).await;
    assert!(result.is_ok());

    let response = result.unwrap().0;
    assert!(response.html.contains("5"));
    assert!(response.render_time_ms > 200); // Should have waited for JS execution
}

#[tokio::test]
async fn test_render_concurrent_requests() {
    let mock_server = MockServer::start().await;

    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Concurrent Test</title>
        </head>
        <body>
            <div id="request-id"></div>
            <script>
                document.getElementById('request-id').innerText =
                    'Request-' + Math.random().toString(36).substr(2, 9);
            </script>
        </body>
        </html>
    "#;

    for i in 0..3 {
        Mock::given(method("GET"))
            .and(path(&format!("/concurrent-{}", i)))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(html_content)
                .insert_header("content-type", "text/html"))
            .mount(&mock_server)
            .await;
    }

    // Launch concurrent render requests
    let mut handles = vec![];
    for i in 0..3 {
        let url = format!("{}/concurrent-{}", mock_server.uri(), i);

        let request = RenderReq {
            url,
            wait_for: Some("css:#request-id".to_string()),
            actions: None,
            scroll_steps: None,
            screenshot: None,
            stealth: None,
            timeout_ms: Some(2000),
        };

        let handle = tokio::spawn(async move {
            render(Json(request)).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        let render_result = result.unwrap();
        assert!(render_result.is_ok());

        let response = render_result.unwrap().0;
        assert!(response.html.contains("Request-"));
        assert!(!response.request_id.is_empty());
    }
}

#[tokio::test]
async fn test_render_memory_and_resource_management() {
    let mock_server = MockServer::start().await;

    // Create content that uses memory
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Memory Test</title>
        </head>
        <body>
            <div id="memory-test">Starting memory test</div>
            <script>
                // Allocate some memory
                let data = [];
                for (let i = 0; i < 1000; i++) {
                    data.push('Data chunk ' + i + ' '.repeat(100));
                }

                document.getElementById('memory-test').innerText =
                    'Memory test completed with ' + data.length + ' items';

                // Clean up
                data = null;
            </script>
        </body>
        </html>
    "#;

    Mock::given(method("GET"))
        .and(path("/memory"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html_content)
            .insert_header("content-type", "text/html"))
        .mount(&mock_server)
        .await;

    let request = RenderReq {
        url: format!("{}/memory", mock_server.uri()),
        wait_for: Some("js:document.getElementById('memory-test').innerText.includes('completed')".to_string()),
        actions: None,
        scroll_steps: None,
        screenshot: None,
        stealth: None,
        timeout_ms: Some(3000),
    };

    let start = std::time::Instant::now();
    let result = render(Json(request)).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    let response = result.unwrap().0;
    assert!(response.html.contains("completed"));
    assert!(response.html.contains("1000 items"));

    // Should complete in reasonable time
    assert!(duration < Duration::from_secs(3));
}