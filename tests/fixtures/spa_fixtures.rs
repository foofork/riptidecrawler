// SPA test fixtures for Single Page Application testing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaTestCase {
    pub name: String,
    pub url: String,
    pub wait_condition: WaitCondition,
    pub expected_elements: Vec<ExpectedElement>,
    pub dynamic_content: bool,
    pub javascript_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    DomContentLoaded,
    NetworkIdle,
    Selector(String),
    CustomJs(String),
    Timeout(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedElement {
    pub selector: String,
    pub content: Option<String>,
    pub attribute: Option<(String, String)>,
    pub visible: bool,
}

pub struct SpaFixtures;

impl SpaFixtures {
    pub fn react_app() -> SpaTestCase {
        SpaTestCase {
            name: "React SPA".to_string(),
            url: "http://localhost:3000".to_string(),
            wait_condition: WaitCondition::Selector("[data-testid='app-loaded']".to_string()),
            expected_elements: vec![
                ExpectedElement {
                    selector: "#root".to_string(),
                    content: None,
                    attribute: None,
                    visible: true,
                },
                ExpectedElement {
                    selector: "[data-testid='main-content']".to_string(),
                    content: Some("Welcome".to_string()),
                    attribute: None,
                    visible: true,
                },
            ],
            dynamic_content: true,
            javascript_required: true,
        }
    }

    pub fn vue_app() -> SpaTestCase {
        SpaTestCase {
            name: "Vue SPA".to_string(),
            url: "http://localhost:8080".to_string(),
            wait_condition: WaitCondition::NetworkIdle,
            expected_elements: vec![
                ExpectedElement {
                    selector: "#app".to_string(),
                    content: None,
                    attribute: None,
                    visible: true,
                },
            ],
            dynamic_content: true,
            javascript_required: true,
        }
    }

    pub fn angular_app() -> SpaTestCase {
        SpaTestCase {
            name: "Angular SPA".to_string(),
            url: "http://localhost:4200".to_string(),
            wait_condition: WaitCondition::CustomJs("window.angularReady === true".to_string()),
            expected_elements: vec![
                ExpectedElement {
                    selector: "app-root".to_string(),
                    content: None,
                    attribute: None,
                    visible: true,
                },
            ],
            dynamic_content: true,
            javascript_required: true,
        }
    }

    pub fn static_spa() -> SpaTestCase {
        SpaTestCase {
            name: "Static SPA".to_string(),
            url: "http://localhost:5000".to_string(),
            wait_condition: WaitCondition::DomContentLoaded,
            expected_elements: vec![
                ExpectedElement {
                    selector: "body".to_string(),
                    content: Some("Static Content".to_string()),
                    attribute: None,
                    visible: true,
                },
            ],
            dynamic_content: false,
            javascript_required: false,
        }
    }

    pub fn all_fixtures() -> Vec<SpaTestCase> {
        vec![
            Self::react_app(),
            Self::vue_app(),
            Self::angular_app(),
            Self::static_spa(),
        ]
    }
}

pub fn create_spa_response(fixture: &SpaTestCase) -> String {
    match fixture.name.as_str() {
        "React SPA" => r#"
            <!DOCTYPE html>
            <html>
            <head><title>React App</title></head>
            <body>
                <div id="root">
                    <div data-testid="app-loaded">
                        <div data-testid="main-content">Welcome</div>
                    </div>
                </div>
            </body>
            </html>
        "#.to_string(),
        "Vue SPA" => r#"
            <!DOCTYPE html>
            <html>
            <head><title>Vue App</title></head>
            <body>
                <div id="app">
                    <h1>Vue Application</h1>
                </div>
            </body>
            </html>
        "#.to_string(),
        "Angular SPA" => r#"
            <!DOCTYPE html>
            <html>
            <head><title>Angular App</title></head>
            <body>
                <app-root>
                    <h1>Angular Application</h1>
                </app-root>
            </body>
            </html>
        "#.to_string(),
        _ => r#"
            <!DOCTYPE html>
            <html>
            <head><title>Static SPA</title></head>
            <body>Static Content</body>
            </html>
        "#.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spa_fixtures_available() {
        let fixtures = SpaFixtures::all_fixtures();
        assert_eq!(fixtures.len(), 4);

        let react = SpaFixtures::react_app();
        assert!(react.javascript_required);
        assert!(react.dynamic_content);
    }

    #[test]
    fn test_spa_response_generation() {
        let react = SpaFixtures::react_app();
        let response = create_spa_response(&react);
        assert!(response.contains("root"));
        assert!(response.contains("Welcome"));
    }
}