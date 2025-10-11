//! Session manager for persistent browser sessions and cookie storage
//!
//! ACTIVELY USED: Integrated via SessionLayer in main.rs and handlers/sessions.rs
//! - AppState has session_manager field (state.rs:69)
//! - Session routes available at /api/sessions/* (handlers/sessions.rs)
//! - Middleware applied to all routes (main.rs)

use super::storage::SessionStorage;
use super::types::{Cookie, CookieJar, Session, SessionConfig, SessionError, SessionStats};
use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tracing::{debug, info};

/// High-level session manager providing a convenient API for session operations
#[derive(Clone, Debug)]
pub struct SessionManager {
    storage: SessionStorage,
    config: SessionConfig,
}

#[allow(dead_code)]
impl SessionManager {
    /// Create a new session manager with the given configuration
    pub async fn new(config: SessionConfig) -> Result<Self> {
        let storage = SessionStorage::new(config.clone()).await?;

        Ok(Self { storage, config })
    }

    /// Get or create a session for the given session ID
    ///
    /// If the session exists and is not expired, it will be returned.
    /// If the session doesn't exist or is expired, a new one will be created.
    pub async fn get_or_create_session(&self, session_id: &str) -> Result<Session> {
        match self.storage.get_session(session_id).await? {
            Some(mut session) => {
                // Update last accessed time
                session.touch(self.config.default_ttl);
                self.storage.store_session(session.clone()).await?;
                Ok(session)
            }
            None => {
                // Create new session
                let session = self.storage.create_session(session_id.to_string()).await?;
                info!(
                    session_id = %session_id,
                    "Created new session"
                );
                Ok(session)
            }
        }
    }

    /// Get an existing session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, SessionError> {
        self.storage.get_session(session_id).await
    }

    /// Create a new session with auto-generated ID
    pub async fn create_session(&self) -> Result<Session> {
        let session_id = Session::generate_session_id();
        self.storage
            .create_session(session_id)
            .await
            .map_err(|e| e.into())
    }

    /// Update session metadata
    pub async fn update_session(&self, session: Session) -> Result<(), SessionError> {
        self.storage.store_session(session).await
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: &str) -> Result<(), SessionError> {
        self.storage.remove_session(session_id).await
    }

    /// Set a cookie for a session
    pub async fn set_cookie(
        &self,
        session_id: &str,
        domain: &str,
        cookie: Cookie,
    ) -> Result<(), SessionError> {
        let mut session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        session.cookies.set_cookie(domain, cookie.clone());
        self.storage.store_session(session.clone()).await?;

        // Persist cookies to disk if enabled
        if self.config.persist_cookies {
            self.persist_cookies_to_disk(session_id, &session.cookies)
                .await?;
        }

        debug!(
            session_id = %session_id,
            domain = %domain,
            cookie_name = %cookie.name,
            "Cookie set for session"
        );

        Ok(())
    }

    /// Get a cookie from a session
    pub async fn get_cookie(
        &self,
        session_id: &str,
        domain: &str,
        name: &str,
    ) -> Result<Option<Cookie>, SessionError> {
        let session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        Ok(session.cookies.get_cookie(domain, name).cloned())
    }

    /// Get all cookies for a domain from a session
    pub async fn get_cookies_for_domain(
        &self,
        session_id: &str,
        domain: &str,
    ) -> Result<Vec<Cookie>, SessionError> {
        let session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        Ok(session
            .cookies
            .get_cookies_for_domain(domain)
            .map(|cookies| cookies.values().cloned().collect())
            .unwrap_or_default())
    }

    /// Get all cookies from a session
    pub async fn get_all_cookies(&self, session_id: &str) -> Result<CookieJar, SessionError> {
        let session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        Ok(session.cookies)
    }

    /// Remove a cookie from a session
    pub async fn remove_cookie(
        &self,
        session_id: &str,
        domain: &str,
        name: &str,
    ) -> Result<Option<Cookie>, SessionError> {
        let mut session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        let removed_cookie = session.cookies.remove_cookie(domain, name);
        self.storage.store_session(session.clone()).await?;

        // Update persistent cookies if enabled
        if self.config.persist_cookies {
            self.persist_cookies_to_disk(session_id, &session.cookies)
                .await?;
        }

        debug!(
            session_id = %session_id,
            domain = %domain,
            cookie_name = %name,
            removed = removed_cookie.is_some(),
            "Cookie removal attempt"
        );

        Ok(removed_cookie)
    }

    /// Clear all cookies from a session
    pub async fn clear_cookies(&self, session_id: &str) -> Result<(), SessionError> {
        let mut session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        session.cookies.clear();
        self.storage.store_session(session.clone()).await?;

        // Update persistent cookies if enabled
        if self.config.persist_cookies {
            self.persist_cookies_to_disk(session_id, &session.cookies)
                .await?;
        }

        debug!(
            session_id = %session_id,
            "All cookies cleared from session"
        );

        Ok(())
    }

    /// Update session expiry time
    pub async fn extend_session(
        &self,
        session_id: &str,
        additional_time: Duration,
    ) -> Result<(), SessionError> {
        let mut session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        session.expires_at += additional_time;
        session.touch(self.config.default_ttl);
        self.storage.store_session(session).await?;

        debug!(
            session_id = %session_id,
            additional_seconds = additional_time.as_secs(),
            "Session expiry extended"
        );

        Ok(())
    }

    /// Get session user data directory
    pub async fn get_user_data_dir(&self, session_id: &str) -> Result<PathBuf, SessionError> {
        let session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        Ok(session.user_data_dir.clone())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(&self) -> Result<usize> {
        self.storage.cleanup_expired().await
    }

    /// Get session statistics
    pub async fn get_stats(&self) -> Result<SessionStats> {
        self.storage.get_stats().await
    }

    /// List all active session IDs
    pub async fn list_sessions(&self) -> Vec<String> {
        self.storage.list_sessions().await
    }

    /// Import cookies from a Netscape format cookie file
    pub async fn import_cookies_from_file(
        &self,
        session_id: &str,
        cookie_file_path: &PathBuf,
    ) -> Result<usize, SessionError> {
        let content =
            fs::read_to_string(cookie_file_path)
                .await
                .map_err(|e| SessionError::IoError {
                    error: format!("Failed to read cookie file: {}", e),
                })?;

        let cookies = self.parse_netscape_cookies(&content)?;
        let cookie_count = cookies.len();

        let mut session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        // Add cookies to session
        for (domain, cookie) in cookies {
            session.cookies.set_cookie(&domain, cookie);
        }

        self.storage.store_session(session.clone()).await?;

        // Persist to disk if enabled
        if self.config.persist_cookies {
            self.persist_cookies_to_disk(session_id, &session.cookies)
                .await?;
        }

        info!(
            session_id = %session_id,
            cookie_count = cookie_count,
            file = %cookie_file_path.display(),
            "Imported cookies from file"
        );

        Ok(cookie_count)
    }

    /// Export cookies to a Netscape format cookie file
    pub async fn export_cookies_to_file(
        &self,
        session_id: &str,
        cookie_file_path: &PathBuf,
    ) -> Result<usize, SessionError> {
        let session = self.storage.get_session(session_id).await?.ok_or_else(|| {
            SessionError::SessionNotFound {
                session_id: session_id.to_string(),
            }
        })?;

        let content = self.format_netscape_cookies(&session.cookies);
        let cookie_count = session.cookies.all_cookies().len();

        fs::write(cookie_file_path, content)
            .await
            .map_err(|e| SessionError::IoError {
                error: format!("Failed to write cookie file: {}", e),
            })?;

        info!(
            session_id = %session_id,
            cookie_count = cookie_count,
            file = %cookie_file_path.display(),
            "Exported cookies to file"
        );

        Ok(cookie_count)
    }

    /// Persist cookies to disk as JSON
    async fn persist_cookies_to_disk(
        &self,
        session_id: &str,
        cookies: &CookieJar,
    ) -> Result<(), SessionError> {
        let session_dir = self.config.base_data_dir.join(session_id);
        let cookies_file = session_dir.join("cookies.json");

        let content = serde_json::to_string_pretty(cookies).map_err(|e| {
            SessionError::SerializationError {
                error: format!("Failed to serialize cookies: {}", e),
            }
        })?;

        fs::write(&cookies_file, content)
            .await
            .map_err(|e| SessionError::IoError {
                error: format!("Failed to write cookies file: {}", e),
            })?;

        debug!(
            session_id = %session_id,
            file = %cookies_file.display(),
            "Cookies persisted to disk"
        );

        Ok(())
    }

    /// Parse Netscape format cookies
    fn parse_netscape_cookies(&self, content: &str) -> Result<Vec<(String, Cookie)>, SessionError> {
        let mut cookies = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 7 {
                continue;
            }

            let domain = parts[0].to_string();
            let path = if parts[2].is_empty() {
                None
            } else {
                Some(parts[2].to_string())
            };
            let secure = parts[3] == "TRUE";
            let expires = if parts[4] == "0" {
                None
            } else {
                parts[4]
                    .parse::<u64>()
                    .ok()
                    .map(|timestamp| SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp))
            };
            let name = parts[5].to_string();
            let value = parts[6].to_string();

            let cookie = Cookie::new(name, value)
                .with_domain(domain.clone())
                .with_path(path.unwrap_or_else(|| "/".to_string()));

            let cookie = if secure { cookie.secure() } else { cookie };
            let cookie = if let Some(exp) = expires {
                cookie.with_expires(exp)
            } else {
                cookie
            };

            cookies.push((domain, cookie));
        }

        Ok(cookies)
    }

    /// Format cookies as Netscape format
    fn format_netscape_cookies(&self, cookie_jar: &CookieJar) -> String {
        let mut lines = vec![
            "# Netscape HTTP Cookie File".to_string(),
            "# This is a generated file! Do not edit.".to_string(),
            "".to_string(),
        ];

        for (domain, domain_cookies) in &cookie_jar.cookies {
            for cookie in domain_cookies.values() {
                let expires = cookie
                    .expires
                    .map(|exp| {
                        exp.duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .to_string()
                    })
                    .unwrap_or_else(|| "0".to_string());

                let secure = if cookie.secure { "TRUE" } else { "FALSE" };
                let path = cookie.path.as_deref().unwrap_or("/");

                let line = format!(
                    "{}\tTRUE\t{}\t{}\t{}\t{}\t{}",
                    domain, path, secure, expires, cookie.name, cookie.value
                );
                lines.push(line);
            }
        }

        lines.join("\n")
    }
}
