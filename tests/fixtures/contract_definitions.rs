// API Contract definitions for contract testing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiContract {
    pub name: String,
    pub version: String,
    pub endpoints: Vec<EndpointContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointContract {
    pub path: String,
    pub method: HttpMethod,
    pub request: RequestContract,
    pub response: ResponseContract,
    pub error_responses: Vec<ErrorContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContract {
    pub headers: HashMap<String, String>,
    pub query_params: Vec<QueryParam>,
    pub body_schema: Option<JsonSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContract {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body_schema: JsonSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContract {
    pub status: u16,
    pub error_code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParam {
    pub name: String,
    pub required: bool,
    pub param_type: ParamType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    pub schema_type: String,
    pub properties: HashMap<String, PropertySchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    pub prop_type: String,
    pub required: bool,
    pub description: String,
}

pub struct ContractDefinitions;

impl ContractDefinitions {
    pub fn extraction_api() -> ApiContract {
        ApiContract {
            name: "Extraction API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                EndpointContract {
                    path: "/extract".to_string(),
                    method: HttpMethod::POST,
                    request: RequestContract {
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("Content-Type".to_string(), "application/json".to_string());
                            h
                        },
                        query_params: vec![],
                        body_schema: Some(JsonSchema {
                            schema_type: "object".to_string(),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("url".to_string(), PropertySchema {
                                    prop_type: "string".to_string(),
                                    required: true,
                                    description: "URL to extract content from".to_string(),
                                });
                                props.insert("selector".to_string(), PropertySchema {
                                    prop_type: "string".to_string(),
                                    required: false,
                                    description: "CSS selector for extraction".to_string(),
                                });
                                props
                            },
                        }),
                    },
                    response: ResponseContract {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("Content-Type".to_string(), "application/json".to_string());
                            h
                        },
                        body_schema: JsonSchema {
                            schema_type: "object".to_string(),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("content".to_string(), PropertySchema {
                                    prop_type: "string".to_string(),
                                    required: true,
                                    description: "Extracted content".to_string(),
                                });
                                props.insert("metadata".to_string(), PropertySchema {
                                    prop_type: "object".to_string(),
                                    required: false,
                                    description: "Extraction metadata".to_string(),
                                });
                                props
                            },
                        },
                    },
                    error_responses: vec![
                        ErrorContract {
                            status: 400,
                            error_code: "INVALID_REQUEST".to_string(),
                            description: "Invalid request parameters".to_string(),
                        },
                        ErrorContract {
                            status: 404,
                            error_code: "URL_NOT_FOUND".to_string(),
                            description: "Target URL not accessible".to_string(),
                        },
                        ErrorContract {
                            status: 500,
                            error_code: "EXTRACTION_FAILED".to_string(),
                            description: "Failed to extract content".to_string(),
                        },
                    ],
                },
            ],
        }
    }

    pub fn streaming_api() -> ApiContract {
        ApiContract {
            name: "Streaming API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                EndpointContract {
                    path: "/stream".to_string(),
                    method: HttpMethod::GET,
                    request: RequestContract {
                        headers: HashMap::new(),
                        query_params: vec![
                            QueryParam {
                                name: "format".to_string(),
                                required: false,
                                param_type: ParamType::String,
                            },
                        ],
                        body_schema: None,
                    },
                    response: ResponseContract {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("Content-Type".to_string(), "application/x-ndjson".to_string());
                            h
                        },
                        body_schema: JsonSchema {
                            schema_type: "stream".to_string(),
                            properties: HashMap::new(),
                        },
                    },
                    error_responses: vec![],
                },
            ],
        }
    }

    pub fn health_check_api() -> ApiContract {
        ApiContract {
            name: "Health Check API".to_string(),
            version: "1.0.0".to_string(),
            endpoints: vec![
                EndpointContract {
                    path: "/healthz".to_string(),
                    method: HttpMethod::GET,
                    request: RequestContract {
                        headers: HashMap::new(),
                        query_params: vec![],
                        body_schema: None,
                    },
                    response: ResponseContract {
                        status: 200,
                        headers: {
                            let mut h = HashMap::new();
                            h.insert("Content-Type".to_string(), "application/json".to_string());
                            h
                        },
                        body_schema: JsonSchema {
                            schema_type: "object".to_string(),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("status".to_string(), PropertySchema {
                                    prop_type: "string".to_string(),
                                    required: true,
                                    description: "Health status".to_string(),
                                });
                                props.insert("timestamp".to_string(), PropertySchema {
                                    prop_type: "string".to_string(),
                                    required: true,
                                    description: "Check timestamp".to_string(),
                                });
                                props
                            },
                        },
                    },
                    error_responses: vec![],
                },
            ],
        }
    }
}

pub fn validate_contract_compliance(
    contract: &EndpointContract,
    actual_response: &HashMap<String, serde_json::Value>,
) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    // Validate required properties
    for (prop_name, prop_schema) in &contract.response.body_schema.properties {
        if prop_schema.required && !actual_response.contains_key(prop_name) {
            violations.push(format!("Missing required property: {}", prop_name));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_definitions_available() {
        let extraction = ContractDefinitions::extraction_api();
        assert_eq!(extraction.name, "Extraction API");
        assert_eq!(extraction.version, "1.0.0");
        assert!(!extraction.endpoints.is_empty());
    }

    #[test]
    fn test_contract_validation() {
        let contract = ContractDefinitions::health_check_api();
        let endpoint = &contract.endpoints[0];

        let mut response = HashMap::new();
        response.insert("status".to_string(), serde_json::json!("healthy"));
        response.insert("timestamp".to_string(), serde_json::json!("2024-01-01T00:00:00Z"));

        let result = validate_contract_compliance(endpoint, &response);
        assert!(result.is_ok());
    }
}