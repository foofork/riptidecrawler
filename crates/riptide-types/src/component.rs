//! Component module - Minimal component traits and types
//!
//! This module has been moved from riptide-core to riptide-types as part of P2-F1.
//! Only minimal types needed for shared interfaces are kept here.

use serde::{Deserialize, Serialize};

/// Component identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub String);

impl ComponentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMeta {
    pub id: ComponentId,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

impl ComponentMeta {
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(id),
            name: name.into(),
            version: version.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id() {
        let id = ComponentId::new("test-component");
        assert_eq!(id.as_str(), "test-component");
    }

    #[test]
    fn test_component_meta() {
        let meta = ComponentMeta::new("comp-1", "Test Component", "1.0.0")
            .with_description("A test component");

        assert_eq!(meta.id.as_str(), "comp-1");
        assert_eq!(meta.name, "Test Component");
        assert_eq!(meta.version, "1.0.0");
        assert!(meta.description.is_some());
    }
}
