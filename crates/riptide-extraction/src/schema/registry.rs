//! Schema registry for storing and retrieving schemas

use super::types::ExtractionSchema;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry for managing extraction schemas
pub struct SchemaRegistry {
    schemas: HashMap<String, Vec<ExtractionSchema>>,
}

/// Schema list item for registry queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaListItem {
    pub name: String,
    pub version: String,
    pub goal: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub usage_count: u64,
    pub success_rate: Option<f64>,
}

/// Request for listing schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRequest {
    pub tag: Option<String>,
    pub goal: Option<String>,
    pub public_only: bool,
    pub limit: u32,
}

/// Response for listing schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub schemas: Vec<SchemaListItem>,
    pub total: u32,
}

impl SchemaRegistry {
    /// Create a new schema registry
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a new schema
    pub fn register(&mut self, schema: ExtractionSchema) -> Result<()> {
        let versions = self.schemas.entry(schema.name.clone()).or_default();

        // Check if version already exists
        if versions.iter().any(|s| s.version == schema.version) {
            anyhow::bail!("Schema version {} already exists", schema.version);
        }

        versions.push(schema);
        Ok(())
    }

    /// Get a schema by name and optionally version
    pub fn get(&self, name: &str, version: Option<&str>) -> Result<ExtractionSchema> {
        let versions = self
            .schemas
            .get(name)
            .context(format!("Schema '{}' not found", name))?;

        if let Some(ver) = version {
            versions
                .iter()
                .find(|s| s.version == ver)
                .cloned()
                .context(format!("Schema version '{}' not found", ver))
        } else {
            // Return latest version
            versions
                .last()
                .cloned()
                .context("No schema versions available")
        }
    }

    /// List all schemas matching criteria
    pub fn list(&self, request: &ListRequest) -> Result<ListResponse> {
        let mut all_schemas: Vec<&ExtractionSchema> = self
            .schemas
            .values()
            .flat_map(|versions| versions.iter())
            .collect();

        // Apply filters
        if request.public_only {
            all_schemas.retain(|s| s.metadata.is_public);
        }

        if let Some(tag) = &request.tag {
            all_schemas.retain(|s| s.metadata.tags.contains(tag));
        }

        if let Some(goal) = &request.goal {
            all_schemas.retain(|s| s.goal == *goal);
        }

        let total = all_schemas.len() as u32;

        // Apply limit
        let limited_schemas: Vec<&ExtractionSchema> = all_schemas
            .into_iter()
            .take(request.limit as usize)
            .collect();

        let items: Vec<SchemaListItem> = limited_schemas
            .into_iter()
            .map(|s| SchemaListItem {
                name: s.name.clone(),
                version: s.version.clone(),
                goal: s.goal.clone(),
                description: s.description.clone(),
                tags: s.metadata.tags.clone(),
                is_public: s.metadata.is_public,
                usage_count: s.metadata.usage_count,
                success_rate: s.metadata.success_rate,
            })
            .collect();

        Ok(ListResponse {
            schemas: items,
            total,
        })
    }

    /// Remove a schema
    pub fn remove(&mut self, name: &str, version: Option<&str>) -> Result<()> {
        if let Some(ver) = version {
            // Remove specific version
            if let Some(versions) = self.schemas.get_mut(name) {
                let original_len = versions.len();
                versions.retain(|s| s.version != ver);

                if versions.len() == original_len {
                    anyhow::bail!("Schema version '{}' not found", ver);
                }

                // Remove entry if no versions left
                if versions.is_empty() {
                    self.schemas.remove(name);
                }
            } else {
                anyhow::bail!("Schema '{}' not found", name);
            }
        } else {
            // Remove all versions
            self.schemas
                .remove(name)
                .context(format!("Schema '{}' not found", name))?;
        }

        Ok(())
    }

    /// Update schema metadata
    pub fn update_metadata(
        &mut self,
        name: &str,
        version: &str,
        updater: impl FnOnce(&mut ExtractionSchema),
    ) -> Result<()> {
        let versions = self
            .schemas
            .get_mut(name)
            .context(format!("Schema '{}' not found", name))?;

        let schema = versions
            .iter_mut()
            .find(|s| s.version == version)
            .context(format!("Schema version '{}' not found", version))?;

        updater(schema);

        Ok(())
    }

    /// Increment usage count
    pub fn increment_usage(&mut self, name: &str, version: &str) -> Result<()> {
        self.update_metadata(name, version, |schema| {
            schema.metadata.usage_count += 1;
        })
    }

    /// Update success rate
    pub fn update_success_rate(&mut self, name: &str, version: &str, rate: f64) -> Result<()> {
        self.update_metadata(name, version, |schema| {
            schema.metadata.success_rate = Some(rate);
        })
    }

    /// Get number of registered schemas
    pub fn count(&self) -> usize {
        self.schemas.values().map(|v| v.len()).sum()
    }

    /// Check if a schema exists
    pub fn exists(&self, name: &str, version: Option<&str>) -> bool {
        if let Some(versions) = self.schemas.get(name) {
            if let Some(ver) = version {
                versions.iter().any(|s| s.version == ver)
            } else {
                !versions.is_empty()
            }
        } else {
            false
        }
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::SchemaMetadata;

    fn create_test_schema(name: &str, version: &str) -> ExtractionSchema {
        ExtractionSchema {
            name: name.to_string(),
            version: version.to_string(),
            goal: "test".to_string(),
            description: None,
            fields: HashMap::new(),
            selectors: HashMap::new(),
            validation: None,
            metadata: SchemaMetadata::default(),
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = SchemaRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_schema() {
        let mut registry = SchemaRegistry::new();
        let schema = create_test_schema("test", "1.0.0");

        registry.register(schema).unwrap();
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_version() {
        let mut registry = SchemaRegistry::new();
        let schema1 = create_test_schema("test", "1.0.0");
        let schema2 = create_test_schema("test", "1.0.0");

        registry.register(schema1).unwrap();
        assert!(registry.register(schema2).is_err());
    }

    #[test]
    fn test_get_schema() {
        let mut registry = SchemaRegistry::new();
        let schema = create_test_schema("test", "1.0.0");

        registry.register(schema).unwrap();

        let retrieved = registry.get("test", Some("1.0.0")).unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.version, "1.0.0");
    }

    #[test]
    fn test_get_latest_version() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(create_test_schema("test", "1.0.0"))
            .unwrap();
        registry
            .register(create_test_schema("test", "2.0.0"))
            .unwrap();

        let latest = registry.get("test", None).unwrap();
        assert_eq!(latest.version, "2.0.0");
    }

    #[test]
    fn test_remove_schema() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(create_test_schema("test", "1.0.0"))
            .unwrap();

        registry.remove("test", Some("1.0.0")).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_list_schemas() {
        let mut registry = SchemaRegistry::new();
        let mut schema = create_test_schema("test1", "1.0.0");
        schema.metadata.is_public = true;
        registry.register(schema).unwrap();

        registry
            .register(create_test_schema("test2", "1.0.0"))
            .unwrap();

        let request = ListRequest {
            tag: None,
            goal: None,
            public_only: true,
            limit: 10,
        };

        let response = registry.list(&request).unwrap();
        assert_eq!(response.schemas.len(), 1);
        assert_eq!(response.total, 1);
    }

    #[test]
    fn test_increment_usage() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(create_test_schema("test", "1.0.0"))
            .unwrap();

        registry.increment_usage("test", "1.0.0").unwrap();

        let schema = registry.get("test", Some("1.0.0")).unwrap();
        assert_eq!(schema.metadata.usage_count, 1);
    }

    #[test]
    fn test_exists() {
        let mut registry = SchemaRegistry::new();
        registry
            .register(create_test_schema("test", "1.0.0"))
            .unwrap();

        assert!(registry.exists("test", Some("1.0.0")));
        assert!(!registry.exists("test", Some("2.0.0")));
        assert!(!registry.exists("other", None));
    }
}
