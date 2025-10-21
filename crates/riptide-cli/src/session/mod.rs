#![allow(dead_code)]

pub mod manager;
pub mod types;

pub use manager::SessionManager;
pub use types::{Cookie, Session};

use anyhow::Result;

/// Get the current session if one is set
pub fn get_current_session() -> Result<Option<Session>> {
    let manager = SessionManager::load()?;
    Ok(manager.get_current().cloned())
}

/// Get a specific session by name
pub fn get_session_by_name(name: &str) -> Result<Session> {
    let manager = SessionManager::load()?;
    Ok(manager.get_session(name)?.clone())
}

/// Load session and mark it as used
pub fn use_session(name: &str) -> Result<Session> {
    let mut manager = SessionManager::load()?;
    let session = manager.get_session_mut(name)?;
    session.mark_used();
    let session_clone = session.clone();
    manager.save()?;
    Ok(session_clone)
}
