//! Centralized test data management
//!
//! This module provides utilities for managing test data across integration tests

use super::{tables::TableFixture, sessions::SessionFixture};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

/// Thread-safe test data store
pub struct TestDataStore {
    tables: Arc<RwLock<HashMap<String, TableFixture>>>,
    sessions: Arc<RwLock<HashMap<String, SessionFixture>>>,
}

impl TestDataStore {
    /// Create a new test data store
    pub fn new() -> Self {
        let store = Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        store.load_defaults();
        store
    }

    /// Load default test fixtures
    fn load_defaults(&self) {
        // Load tables
        {
            let mut tables = self.tables.write().unwrap();
            for fixture in super::tables::get_default_table_fixtures() {
                tables.insert(fixture.id.clone(), fixture);
            }
        }

        // Load sessions
        {
            let mut sessions = self.sessions.write().unwrap();
            for fixture in super::sessions::get_default_session_fixtures() {
                sessions.insert(fixture.session_id.clone(), fixture);
            }
        }
    }

    /// Get a table by ID
    pub fn get_table(&self, id: &str) -> Option<TableFixture> {
        let tables = self.tables.read().unwrap();
        tables.get(id).cloned()
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &str) -> Option<SessionFixture> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(id).cloned()
    }

    /// Add a table
    pub fn add_table(&self, fixture: TableFixture) {
        let mut tables = self.tables.write().unwrap();
        tables.insert(fixture.id.clone(), fixture);
    }

    /// Add a session
    pub fn add_session(&self, fixture: SessionFixture) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(fixture.session_id.clone(), fixture);
    }

    /// Remove a table
    pub fn remove_table(&self, id: &str) -> bool {
        let mut tables = self.tables.write().unwrap();
        tables.remove(id).is_some()
    }

    /// Remove a session
    pub fn remove_session(&self, id: &str) -> bool {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(id).is_some()
    }

    /// Clear all test data
    pub fn clear(&self) {
        {
            let mut tables = self.tables.write().unwrap();
            tables.clear();
        }
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.clear();
        }
    }

    /// Reset to defaults
    pub fn reset(&self) {
        self.clear();
        self.load_defaults();
    }

    /// Get all table IDs
    pub fn get_table_ids(&self) -> Vec<String> {
        let tables = self.tables.read().unwrap();
        tables.keys().cloned().collect()
    }

    /// Get all session IDs
    pub fn get_session_ids(&self) -> Vec<String> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().cloned().collect()
    }
}

impl Default for TestDataStore {
    fn default() -> Self {
        Self::new()
    }
}

// Global test data store instance
lazy_static::lazy_static! {
    /// Global test data store for integration tests
    pub static ref GLOBAL_TEST_DATA: TestDataStore = TestDataStore::new();
}

/// Seed test data helper for integration tests
pub fn seed_test_data() {
    GLOBAL_TEST_DATA.reset();
}

/// Clean up test data after tests
pub fn cleanup_test_data() {
    GLOBAL_TEST_DATA.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_initialization() {
        let store = TestDataStore::new();

        // Should have default data
        assert!(store.get_table_ids().len() > 0, "Should have default tables");
        assert!(store.get_session_ids().len() > 0, "Should have default sessions");
    }

    #[test]
    fn test_add_and_get_table() {
        let store = TestDataStore::new();

        let custom_table = TableFixture {
            id: "custom_table_001".to_string(),
            source_url: None,
            html_content: None,
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            data: vec![vec!["A".to_string(), "B".to_string()]],
            rows: 2,
            columns: 2,
            has_spans: false,
            metadata: super::tables::TableMetadata {
                element_id: None,
                classes: vec![],
                extracted_at: "2025-10-27T00:00:00Z".to_string(),
                data_types: vec![],
            },
        };

        store.add_table(custom_table.clone());

        let retrieved = store.get_table("custom_table_001");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "custom_table_001");
    }

    #[test]
    fn test_remove_table() {
        let store = TestDataStore::new();

        // Should have default table
        assert!(store.get_table("table_12345").is_some());

        // Remove it
        assert!(store.remove_table("table_12345"));

        // Should not exist anymore
        assert!(store.get_table("table_12345").is_none());
    }

    #[test]
    fn test_reset() {
        let store = TestDataStore::new();

        // Clear all data
        store.clear();
        assert_eq!(store.get_table_ids().len(), 0);

        // Reset should restore defaults
        store.reset();
        assert!(store.get_table_ids().len() > 0);
    }
}
