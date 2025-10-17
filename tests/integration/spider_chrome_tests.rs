//! Integration tests for spider-chrome Phase 1 implementation
//!
//! Tests basic automation features:
//! - Page navigation and HTML capture
//! - Screenshot capture
//! - PDF generation
//! - Stealth feature preservation
//! - Performance parity with chromiumoxide

#[cfg(feature = "headless")]
mod spider_chrome_tests {
    use anyhow::Result;
    use riptide_headless_hybrid::{HybridHeadlessLauncher, LauncherConfig};
    use riptide_stealth::StealthPreset;
    use std::time::Instant;

    /// Test basic page navigation and HTML capture
    #[tokio::test]
    async fn test_spider_chrome_basic_navigation() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Navigate to example.com
        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Verify we can get HTML content
        let html = session.content().await?;
        assert!(!html.is_empty(), "HTML content should not be empty");
        assert!(
            html.contains("Example Domain") || html.contains("example"),
            "HTML should contain expected content"
        );

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test screenshot capture functionality
    #[tokio::test]
    async fn test_spider_chrome_screenshot() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Take screenshot
        let screenshot_data = session.screenshot().await?;
        assert!(!screenshot_data.is_empty(), "Screenshot data should not be empty");
        assert!(
            screenshot_data.len() > 1000,
            "Screenshot should be reasonably sized"
        );

        // Verify PNG header
        assert_eq!(
            &screenshot_data[0..8],
            &[137, 80, 78, 71, 13, 10, 26, 10],
            "Screenshot should be valid PNG"
        );

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test screenshot save to file
    #[tokio::test]
    async fn test_spider_chrome_screenshot_to_file() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let temp_dir = tempfile::tempdir()?;
        let screenshot_path = temp_dir.path().join("test_screenshot.png");

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Save screenshot
        session
            .screenshot_to_file(screenshot_path.to_str().unwrap())
            .await?;

        // Verify file exists and has content
        assert!(screenshot_path.exists(), "Screenshot file should exist");
        let file_size = tokio::fs::metadata(&screenshot_path).await?.len();
        assert!(file_size > 1000, "Screenshot file should have content");

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test PDF generation functionality
    #[tokio::test]
    async fn test_spider_chrome_pdf_generation() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Generate PDF
        let pdf_data = session.pdf().await?;
        assert!(!pdf_data.is_empty(), "PDF data should not be empty");
        assert!(pdf_data.len() > 100, "PDF should be reasonably sized");

        // Verify PDF header (%PDF)
        assert_eq!(
            &pdf_data[0..4],
            b"%PDF",
            "PDF should have valid header"
        );

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test PDF save to file
    #[tokio::test]
    async fn test_spider_chrome_pdf_to_file() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let temp_dir = tempfile::tempdir()?;
        let pdf_path = temp_dir.path().join("test_output.pdf");

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Save PDF
        session.pdf_to_file(pdf_path.to_str().unwrap()).await?;

        // Verify file exists and has content
        assert!(pdf_path.exists(), "PDF file should exist");
        let file_size = tokio::fs::metadata(&pdf_path).await?.len();
        assert!(file_size > 100, "PDF file should have content");

        // Verify PDF header
        let pdf_content = tokio::fs::read(&pdf_path).await?;
        assert_eq!(&pdf_content[0..4], b"%PDF", "PDF should have valid header");

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test stealth features are preserved
    #[tokio::test]
    async fn test_spider_chrome_stealth_preservation() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Launch with high stealth
        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::High))
            .await?;

        // Execute JavaScript to check for webdriver flag
        let webdriver_check = session
            .execute_script("return navigator.webdriver === undefined;")
            .await?;

        assert!(
            webdriver_check.as_bool().unwrap_or(false),
            "navigator.webdriver should be undefined with stealth"
        );

        // Check for chrome object
        let chrome_check = session
            .execute_script("return typeof window.chrome !== 'undefined';")
            .await?;

        assert!(
            chrome_check.as_bool().unwrap_or(false),
            "window.chrome should exist with stealth"
        );

        // Clean up
        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test multiple stealth presets
    #[tokio::test]
    async fn test_spider_chrome_stealth_presets() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;
        let test_url = "https://example.com";

        // Test each preset
        let presets = vec![
            StealthPreset::None,
            StealthPreset::Low,
            StealthPreset::Medium,
            StealthPreset::High,
        ];

        for preset in presets {
            let session = launcher.launch_page(test_url, Some(preset.clone())).await?;

            // Verify page loads
            let html = session.content().await?;
            assert!(!html.is_empty(), "HTML should load with {:?} preset", preset);

            session.close().await?;
        }

        launcher.shutdown().await?;
        Ok(())
    }

    /// Test session statistics tracking
    #[tokio::test]
    async fn test_spider_chrome_statistics() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Initial stats
        let stats_before = launcher.stats().await;
        assert_eq!(stats_before.total_requests, 0);

        // Launch a few sessions
        for i in 0..3 {
            let session = launcher
                .launch_page("https://example.com", Some(StealthPreset::Low))
                .await?;
            let _ = session.content().await;
            session.close().await?;
        }

        // Check updated stats
        let stats_after = launcher.stats().await;
        assert_eq!(
            stats_after.total_requests, 3,
            "Should track total requests"
        );
        assert_eq!(
            stats_after.successful_requests, 3,
            "Should track successful requests"
        );
        assert!(
            stats_after.avg_response_time_ms > 0.0,
            "Should track response time"
        );

        launcher.shutdown().await?;
        Ok(())
    }

    /// Test error handling for invalid URLs
    #[tokio::test]
    async fn test_spider_chrome_invalid_url_handling() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Try invalid URL (should handle gracefully)
        let result = launcher
            .launch_page("not-a-valid-url", Some(StealthPreset::Low))
            .await;

        // Should create session but navigation might fail
        // This tests that the launcher doesn't panic
        if let Ok(session) = result {
            let _ = session.close().await;
        }

        launcher.shutdown().await?;
        Ok(())
    }

    /// Test concurrent page launches
    #[tokio::test]
    async fn test_spider_chrome_concurrent_sessions() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        // Launch multiple pages concurrently
        let mut handles = vec![];
        for i in 0..5 {
            let launcher_clone = &launcher;
            let handle = tokio::spawn(async move {
                let session = launcher_clone
                    .launch_page("https://example.com", Some(StealthPreset::Low))
                    .await?;
                let html = session.content().await?;
                session.close().await?;
                Ok::<_, anyhow::Error>(html.len())
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let result = handle.await?;
            assert!(result.is_ok(), "Concurrent session should succeed");
        }

        launcher.shutdown().await?;
        Ok(())
    }

    /// Test launcher configuration options
    #[tokio::test]
    async fn test_spider_chrome_custom_config() -> Result<()> {
        let config = LauncherConfig {
            enable_stealth: true,
            default_stealth_preset: StealthPreset::High,
            page_timeout: std::time::Duration::from_secs(60),
            enable_monitoring: true,
            ..Default::default()
        };

        let launcher = HybridHeadlessLauncher::with_config(config).await?;

        let session = launcher.launch_page_default("https://example.com").await?;
        let html = session.content().await?;
        assert!(!html.is_empty());

        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test page navigation within session
    #[tokio::test]
    async fn test_spider_chrome_page_navigation() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Get initial content
        let html1 = session.content().await?;
        assert!(html1.contains("example") || html1.contains("Example"));

        // Navigate to different page (same domain for test reliability)
        session.navigate("https://example.org").await?;

        // Get new content
        let html2 = session.content().await?;
        // Content should have changed (different URL)
        // Note: Both might resolve to same page, so just verify content exists
        assert!(!html2.is_empty());

        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }

    /// Test wait for element functionality
    #[tokio::test]
    async fn test_spider_chrome_wait_for_element() -> Result<()> {
        let launcher = HybridHeadlessLauncher::new().await?;

        let session = launcher
            .launch_page("https://example.com", Some(StealthPreset::Low))
            .await?;

        // Wait for body element (should always exist)
        let result = session.wait_for_element("body", Some(5000)).await;
        assert!(result.is_ok(), "Should find body element");

        // Wait for non-existent element (should timeout)
        let result = session
            .wait_for_element("#non-existent-element", Some(1000))
            .await;
        assert!(result.is_err(), "Should timeout for non-existent element");

        session.close().await?;
        launcher.shutdown().await?;

        Ok(())
    }
}

// Marker to ensure tests compile even without headless feature
#[cfg(not(feature = "headless"))]
#[test]
fn test_spider_chrome_requires_headless_feature() {
    // This test passes to indicate feature is disabled
    println!("Spider-chrome tests require 'headless' feature");
}
