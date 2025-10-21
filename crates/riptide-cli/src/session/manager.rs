#![allow(dead_code)]

use super::types::Session;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Session manager for lifecycle management
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionManager {
    current_session: Option<String>,
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            current_session: None,
            sessions: HashMap::new(),
        }
    }

    /// Load sessions from disk
    pub fn load() -> Result<Self> {
        let path = Self::storage_path()?;
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)
            .context(format!("Failed to read session file: {}", path.display()))?;
        let manager: SessionManager =
            serde_json::from_str(&content).context("Failed to parse session data")?;

        Ok(manager)
    }

    /// Save sessions to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    /// Get the storage path for sessions
    pub fn storage_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".riptide").join("sessions").join("sessions.json"))
    }

    // Note: sessions_dir() was removed as it was not being used.
    // Session storage uses storage_path() which points to sessions.json

    /// Add a new session
    pub fn add_session(&mut self, session: Session) -> Result<()> {
        if self.sessions.contains_key(&session.name) {
            return Err(anyhow!("Session '{}' already exists", session.name));
        }

        self.sessions.insert(session.name.clone(), session);
        Ok(())
    }

    /// Get a session by name
    pub fn get_session(&self, name: &str) -> Result<&Session> {
        self.sessions
            .get(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    /// Get a mutable session by name
    pub fn get_session_mut(&mut self, name: &str) -> Result<&mut Session> {
        self.sessions
            .get_mut(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    /// Remove a session
    pub fn remove_session(&mut self, name: &str) -> Result<Session> {
        self.sessions
            .remove(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    /// Set the current active session
    pub fn set_current(&mut self, name: String) -> Result<()> {
        if !self.sessions.contains_key(&name) {
            return Err(anyhow!("Session '{}' does not exist", name));
        }

        self.current_session = Some(name);
        Ok(())
    }

    /// Get the current active session
    pub fn get_current(&self) -> Option<&Session> {
        self.current_session
            .as_ref()
            .and_then(|name| self.sessions.get(name))
    }

    /// Get current session name
    pub fn current_session_name(&self) -> Option<&String> {
        self.current_session.as_ref()
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }

    /// Get all sessions as a hashmap reference
    pub fn sessions(&self) -> &HashMap<String, Session> {
        &self.sessions
    }

    /// Get total session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if a session exists
    pub fn contains_session(&self, name: &str) -> bool {
        self.sessions.contains_key(name)
    }

    /// Export session to file
    pub fn export_session(&self, name: &str, path: &str, format: &str) -> Result<()> {
        let session = self.get_session(name)?;

        let content = match format {
            "yaml" => serde_yaml::to_string(session)?,
            _ => serde_json::to_string_pretty(session)?,
        };

        fs::write(path, content)?;
        Ok(())
    }

    /// Import session from file
    pub fn import_session(
        &mut self,
        path: &str,
        name_override: Option<String>,
        overwrite: bool,
    ) -> Result<String> {
        let content = fs::read_to_string(path)?;

        let mut session: Session = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };

        if let Some(new_name) = name_override {
            session.name = new_name;
        }

        if self.sessions.contains_key(&session.name) {
            if !overwrite {
                return Err(anyhow!(
                    "Session '{}' already exists. Use --overwrite to replace it.",
                    session.name
                ));
            }
            self.remove_session(&session.name)?;
        }

        session.updated_at = chrono::Utc::now();
        let session_name = session.name.clone();
        self.add_session(session)?;

        Ok(session_name)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
