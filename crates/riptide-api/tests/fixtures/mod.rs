//! Test fixtures for integration testing
//!
//! This module provides test data fixtures to support integration tests
//! and ensure all API endpoints have valid test data available.

pub mod tables;
pub mod sessions;
pub mod test_data;

/// Fixture management for integration tests
pub struct FixtureManager {
    /// In-memory storage for tables
    tables: std::collections::HashMap<String, tables::TableFixture>,
    /// In-memory storage for sessions
    sessions: std::collections::HashMap<String, sessions::SessionFixture>,
}

impl FixtureManager {
    /// Create a new fixture manager with default test data
    pub fn new() -> Self {
        let mut manager = Self {
            tables: std::collections::HashMap::new(),
            sessions: std::collections::HashMap::new(),
        };

        // Load default fixtures
        manager.load_default_fixtures();
        manager
    }

    /// Load default test fixtures
    fn load_default_fixtures(&mut self) {
        // Add default table fixtures
        for fixture in tables::get_default_table_fixtures() {
            self.tables.insert(fixture.id.clone(), fixture);
        }

        // Add default session fixtures
        for fixture in sessions::get_default_session_fixtures() {
            self.sessions.insert(fixture.session_id.clone(), fixture);
        }
    }

    /// Get a table fixture by ID
    pub fn get_table(&self, id: &str) -> Option<&tables::TableFixture> {
        self.tables.get(id)
    }

    /// Get a session fixture by ID
    pub fn get_session(&self, id: &str) -> Option<&sessions::SessionFixture> {
        self.sessions.get(id)
    }

    /// Add a custom table fixture
    pub fn add_table(&mut self, fixture: tables::TableFixture) {
        self.tables.insert(fixture.id.clone(), fixture);
    }

    /// Add a custom session fixture
    pub fn add_session(&mut self, fixture: sessions::SessionFixture) {
        self.sessions.insert(fixture.session_id.clone(), fixture);
    }

    /// Clear all fixtures
    pub fn clear(&mut self) {
        self.tables.clear();
        self.sessions.clear();
    }
}

impl Default for FixtureManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_manager_initialization() {
        let manager = FixtureManager::new();

        // Should have default fixtures loaded
        assert!(manager.tables.len() > 0, "Should have default table fixtures");
        assert!(manager.sessions.len() > 0, "Should have default session fixtures");
    }

    #[test]
    fn test_get_table_fixture() {
        let manager = FixtureManager::new();

        // Should be able to retrieve a table fixture
        let table = manager.get_table("table_12345");
        assert!(table.is_some(), "Should find default table fixture");
    }

    #[test]
    fn test_get_session_fixture() {
        let manager = FixtureManager::new();

        // Should be able to retrieve a session fixture
        let session = manager.get_session("test-session-123");
        assert!(session.is_some(), "Should find default session fixture");
    }
}
