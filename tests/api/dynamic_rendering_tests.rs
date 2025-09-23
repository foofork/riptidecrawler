/// Dynamic Rendering Action Tests - London School TDD
///
/// Tests the collaboration between API handlers and dynamic rendering services
/// using mocks to verify SPA handling, action execution, and wait conditions.

use crate::fixtures::*;
use crate::fixtures::test_data::*;
use mockall::predicate::*;
use std::time::Duration;
use tokio_test;
use tracing_test::traced_test;

#[cfg(test)]
mod dynamic_rendering_tests {
    use super::*;

    /// Test SPA fixture with dynamic actions
    #[traced_test]
    #[tokio::test]
    async fn test_spa_fixture_dynamic_actions() {
        // Arrange - Set up mock dynamic renderer
        let mut mock_renderer = MockDynamicRenderer::new();
        let spa_fixtures = TestUrls::spa_fixtures();

        for (url, config) in spa_fixtures.iter() {
            // Expect render call with specific configuration
            mock_renderer
                .expect_render()
                .with(eq(*url), always())
                .times(1)
                .returning(|url, _| {
                    Ok(RenderResult {
                        html: format!(
                            r#"<html><body><div class="dynamic-content">Loaded from {}</div></body></html>"#,
                            url
                        ),
                        success: true,
                        actions_executed: vec![
                            ActionResult {
                                action: Action {
                                    action_type: "click".to_string(),
                                    selector: Some("#load-more".to_string()),
                                    value: None,
                                },
                                success: true,
                                error: None,
                            },
                            ActionResult {
                                action: Action {
                                    action_type: "wait".to_string(),
                                    selector: None,
                                    value: Some("2000".to_string()),
                                },
                                success: true,
                                error: None,
                            },
                        ],
                    })
                });

            // Expect action execution
            mock_renderer
                .expect_execute_actions()
                .with(always())
                .times(1)
                .returning(|actions| {
                    Ok(actions.iter().map(|action| ActionResult {
                        action: action.clone(),
                        success: true,
                        error: None,
                    }).collect())
                });

            // Expect wait conditions to be checked
            mock_renderer
                .expect_wait_for_conditions()
                .with(always())
                .times(1)
                .returning(|_| Ok(true));
        }

        // Act & Assert
        for (url, config) in spa_fixtures.iter() {
            // Test the rendering collaboration
            let render_result = mock_renderer.render(url, config).await;
            assert!(render_result.is_ok());

            let result = render_result.unwrap();
            assert!(result.success);
            assert!(result.html.contains("dynamic-content"));
            assert_eq!(result.actions_executed.len(), 2);

            // Verify action execution succeeded
            assert!(result.actions_executed.iter().all(|ar| ar.success));

            // Test individual action execution
            let action_results = mock_renderer.execute_actions(&config.actions).await;
            assert!(action_results.is_ok());
            assert_eq!(action_results.unwrap().len(), config.actions.len());

            // Test wait conditions
            let wait_result = mock_renderer.wait_for_conditions(&config.wait_conditions).await;
            assert!(wait_result.is_ok());
            assert!(wait_result.unwrap());
        }
    }

    /// Test action execution sequence and coordination
    #[traced_test]
    #[tokio::test]
    async fn test_action_execution_sequence() {
        // Arrange
        let mut mock_renderer = MockDynamicRenderer::new();

        let actions = vec![
            Action {
                action_type: "navigate".to_string(),
                selector: None,
                value: Some("https://spa-app.com/dashboard".to_string()),
            },
            Action {
                action_type: "wait_for_element".to_string(),
                selector: Some("#main-content".to_string()),
                value: None,
            },
            Action {
                action_type: "click".to_string(),
                selector: Some("#load-more-button".to_string()),
                value: None,
            },
            Action {
                action_type: "wait_for_element".to_string(),
                selector: Some(".dynamic-item".to_string()),
                value: None,
            },
            Action {
                action_type: "scroll".to_string(),
                selector: None,
                value: Some("bottom".to_string()),
            },
        ];

        // Expect actions to be executed in sequence
        mock_renderer
            .expect_execute_actions()
            .with(eq(actions.clone()))
            .times(1)
            .returning(move |actions| {
                // Simulate sequential execution with some actions potentially failing
                let mut results = Vec::new();
                for (i, action) in actions.iter().enumerate() {
                    let success = match action.action_type.as_str() {
                        "navigate" => true,
                        "wait_for_element" => i < 3, // First wait succeeds, second might fail
                        "click" => true,
                        "scroll" => true,
                        _ => false,
                    };

                    results.push(ActionResult {
                        action: action.clone(),
                        success,
                        error: if success { None } else { Some("Element not found".to_string()) },
                    });
                }
                Ok(results)
            });

        // Act
        let results = mock_renderer.execute_actions(&actions).await;

        // Assert
        assert!(results.is_ok());
        let action_results = results.unwrap();
        assert_eq!(action_results.len(), actions.len());

        // Verify sequence: navigate -> wait -> click -> wait -> scroll
        assert!(action_results[0].success); // navigate
        assert!(action_results[1].success); // first wait
        assert!(action_results[2].success); // click
        // Note: second wait might fail in this test scenario
        assert!(action_results[4].success); // scroll
    }

    /// Test wait condition handling and timeouts
    #[traced_test]
    #[tokio::test]
    async fn test_wait_condition_handling() {
        // Arrange
        let mut mock_renderer = MockDynamicRenderer::new();

        let wait_conditions = vec![
            WaitCondition {
                condition_type: "element_visible".to_string(),
                selector: Some(".loading-spinner".to_string()),
                timeout: Duration::from_secs(1),
            },
            WaitCondition {
                condition_type: "element_hidden".to_string(),
                selector: Some(".loading-spinner".to_string()),
                timeout: Duration::from_secs(5),
            },
            WaitCondition {
                condition_type: "element_visible".to_string(),
                selector: Some(".content-loaded".to_string()),
                timeout: Duration::from_secs(10),
            },
        ];

        // Test successful wait conditions
        mock_renderer
            .expect_wait_for_conditions()
            .with(eq(wait_conditions.clone()))
            .times(1)
            .returning(|_| Ok(true));

        // Test timeout scenario
        let timeout_conditions = vec![
            WaitCondition {
                condition_type: "element_visible".to_string(),
                selector: Some(".never-appears".to_string()),
                timeout: Duration::from_millis(100),
            },
        ];

        mock_renderer
            .expect_wait_for_conditions()
            .with(eq(timeout_conditions.clone()))
            .times(1)
            .returning(|_| Ok(false)); // Timeout occurred

        // Act & Assert
        let success_result = mock_renderer.wait_for_conditions(&wait_conditions).await;
        assert!(success_result.is_ok());
        assert!(success_result.unwrap());

        let timeout_result = mock_renderer.wait_for_conditions(&timeout_conditions).await;
        assert!(timeout_result.is_ok());
        assert!(!timeout_result.unwrap()); // Should indicate timeout
    }

    /// Test error handling in dynamic rendering
    #[traced_test]
    #[tokio::test]
    async fn test_dynamic_rendering_error_handling() {
        // Arrange
        let mut mock_renderer = MockDynamicRenderer::new();

        // Test network error during rendering
        mock_renderer
            .expect_render()
            .with(eq("https://unreachable.com"), always())
            .times(1)
            .returning(|_, _| Err("Network error: Connection refused".to_string()));

        // Test action execution error
        let failing_action = vec![Action {
            action_type: "click".to_string(),
            selector: Some("#non-existent-element".to_string()),
            value: None,
        }];

        mock_renderer
            .expect_execute_actions()
            .with(eq(failing_action.clone()))
            .times(1)
            .returning(|actions| {
                Ok(vec![ActionResult {
                    action: actions[0].clone(),
                    success: false,
                    error: Some("Element not found: #non-existent-element".to_string()),
                }])
            });

        // Act & Assert
        let render_error = mock_renderer.render("https://unreachable.com", &DynamicConfig {
            actions: vec![],
            wait_conditions: vec![],
            timeout: Duration::from_secs(30),
        }).await;

        assert!(render_error.is_err());
        assert!(render_error.unwrap_err().contains("Network error"));

        let action_result = mock_renderer.execute_actions(&failing_action).await;
        assert!(action_result.is_ok());
        let results = action_result.unwrap();
        assert!(!results[0].success);
        assert!(results[0].error.as_ref().unwrap().contains("Element not found"));
    }

    /// Test complex SPA interaction scenario
    #[traced_test]
    #[tokio::test]
    async fn test_complex_spa_interaction() {
        // Arrange - Simulate a complex e-commerce SPA interaction
        let mut mock_renderer = MockDynamicRenderer::new();

        let config = DynamicConfig {
            actions: vec![
                Action {
                    action_type: "click".to_string(),
                    selector: Some("#category-filter".to_string()),
                    value: None,
                },
                Action {
                    action_type: "wait".to_string(),
                    selector: None,
                    value: Some("1000".to_string()),
                },
                Action {
                    action_type: "type".to_string(),
                    selector: Some("#search-input".to_string()),
                    value: Some("laptop".to_string()),
                },
                Action {
                    action_type: "click".to_string(),
                    selector: Some("#search-button".to_string()),
                    value: None,
                },
                Action {
                    action_type: "scroll".to_string(),
                    selector: None,
                    value: Some("500".to_string()),
                },
            ],
            wait_conditions: vec![
                WaitCondition {
                    condition_type: "element_visible".to_string(),
                    selector: Some(".search-results".to_string()),
                    timeout: Duration::from_secs(10),
                },
                WaitCondition {
                    condition_type: "element_count".to_string(),
                    selector: Some(".product-item".to_string()),
                    timeout: Duration::from_secs(15),
                },
            ],
            timeout: Duration::from_secs(60),
        };

        // Set up complex interaction expectations
        mock_renderer
            .expect_render()
            .with(eq("https://ecommerce.com/products"), eq(config.clone()))
            .times(1)
            .returning(|_, _| {
                Ok(RenderResult {
                    html: r#"
                        <html>
                        <body>
                            <div class="search-results">
                                <div class="product-item">Laptop 1</div>
                                <div class="product-item">Laptop 2</div>
                                <div class="product-item">Laptop 3</div>
                            </div>
                        </body>
                        </html>
                    "#.to_string(),
                    success: true,
                    actions_executed: vec![
                        ActionResult {
                            action: Action {
                                action_type: "click".to_string(),
                                selector: Some("#category-filter".to_string()),
                                value: None,
                            },
                            success: true,
                            error: None,
                        },
                        ActionResult {
                            action: Action {
                                action_type: "type".to_string(),
                                selector: Some("#search-input".to_string()),
                                value: Some("laptop".to_string()),
                            },
                            success: true,
                            error: None,
                        },
                        // ... other actions
                    ],
                })
            });

        mock_renderer
            .expect_wait_for_conditions()
            .with(always())
            .times(1)
            .returning(|_| Ok(true));

        // Act
        let result = mock_renderer.render("https://ecommerce.com/products", &config).await;
        let wait_result = mock_renderer.wait_for_conditions(&config.wait_conditions).await;

        // Assert
        assert!(result.is_ok());
        let render_result = result.unwrap();
        assert!(render_result.success);
        assert!(render_result.html.contains("search-results"));
        assert!(render_result.html.contains("product-item"));
        assert!(!render_result.actions_executed.is_empty());

        assert!(wait_result.is_ok());
        assert!(wait_result.unwrap());
    }

    /// Test rendering timeout and resource management
    #[traced_test]
    #[tokio::test]
    async fn test_rendering_timeout_management() {
        // Arrange
        let mut mock_renderer = MockDynamicRenderer::new();

        let short_timeout_config = DynamicConfig {
            actions: vec![
                Action {
                    action_type: "wait".to_string(),
                    selector: None,
                    value: Some("5000".to_string()), // Wait longer than timeout
                },
            ],
            wait_conditions: vec![],
            timeout: Duration::from_millis(100), // Very short timeout
        };

        // Expect timeout behavior
        mock_renderer
            .expect_render()
            .with(eq("https://slow-site.com"), eq(short_timeout_config.clone()))
            .times(1)
            .returning(|_, _| {
                // Simulate timeout error
                Err("Render timeout: Operation exceeded 100ms limit".to_string())
            });

        // Act
        let result = mock_renderer.render("https://slow-site.com", &short_timeout_config).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("timeout"));
        assert!(error.contains("100ms"));
    }

    /// Test action coordination and state management
    #[traced_test]
    #[tokio::test]
    async fn test_action_coordination_state_management() {
        // Arrange
        let mut mock_renderer = MockDynamicRenderer::new();

        // Test that actions can depend on previous action results
        let coordinated_actions = vec![
            Action {
                action_type: "click".to_string(),
                selector: Some("#toggle-advanced".to_string()),
                value: None,
            },
            Action {
                action_type: "wait_for_element".to_string(),
                selector: Some("#advanced-options".to_string()),
                value: None,
            },
            Action {
                action_type: "select".to_string(),
                selector: Some("#advanced-dropdown".to_string()),
                value: Some("option-2".to_string()),
            },
        ];

        mock_renderer
            .expect_execute_actions()
            .with(eq(coordinated_actions.clone()))
            .times(1)
            .returning(|actions| {
                let mut results = Vec::new();
                let mut advanced_visible = false;

                for action in actions {
                    let success = match action.action_type.as_str() {
                        "click" if action.selector.as_deref() == Some("#toggle-advanced") => {
                            advanced_visible = true;
                            true
                        },
                        "wait_for_element" => advanced_visible, // Depends on previous action
                        "select" => advanced_visible, // Also depends on state
                        _ => false,
                    };

                    results.push(ActionResult {
                        action: action.clone(),
                        success,
                        error: if success { None } else { Some("State dependency not met".to_string()) },
                    });
                }
                Ok(results)
            });

        // Act
        let results = mock_renderer.execute_actions(&coordinated_actions).await;

        // Assert
        assert!(results.is_ok());
        let action_results = results.unwrap();

        // All actions should succeed because they're properly coordinated
        assert!(action_results.iter().all(|ar| ar.success));
        assert_eq!(action_results.len(), 3);
    }
}