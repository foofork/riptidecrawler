use crate::output;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(clap::Subcommand)]
pub enum SessionCommands {
    /// Create a new session
    New {
        /// Session name
        #[arg(long)]
        name: String,

        /// Session description
        #[arg(long)]
        description: Option<String>,

        /// Initialize with cookies from JSON file
        #[arg(long)]
        cookies: Option<String>,

        /// Initialize with headers from JSON file
        #[arg(long)]
        headers: Option<String>,

        /// Session tags for categorization
        #[arg(long)]
        tags: Option<String>,

        /// User agent string
        #[arg(long)]
        user_agent: Option<String>,

        /// Session timeout in minutes (0 = no timeout)
        #[arg(long, default_value = "0")]
        timeout: u64,
    },

    /// List all sessions
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Show only active sessions
        #[arg(long)]
        active: bool,

        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Switch to a different session
    Use {
        /// Session name to switch to
        #[arg(long)]
        name: String,

        /// Create session if it doesn't exist
        #[arg(long)]
        create: bool,
    },

    /// Show current session
    Current {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Export session state
    Export {
        /// Session name to export
        #[arg(long)]
        name: String,

        /// Output file path
        #[arg(long, short = 'o')]
        output: String,

        /// Export format (json, yaml)
        #[arg(long, default_value = "json")]
        format: String,

        /// Include cookies in export
        #[arg(long, default_value = "true")]
        include_cookies: bool,

        /// Include headers in export
        #[arg(long, default_value = "true")]
        include_headers: bool,
    },

    /// Import session state
    Import {
        /// Input file path
        #[arg(long)]
        input: String,

        /// Override session name
        #[arg(long)]
        name: Option<String>,

        /// Overwrite existing session
        #[arg(long)]
        overwrite: bool,
    },

    /// Remove a session
    Rm {
        /// Session name to remove
        #[arg(long)]
        name: String,

        /// Force removal without confirmation
        #[arg(long, short = 'f')]
        force: bool,

        /// Remove all sessions matching tag
        #[arg(long)]
        tag: Option<String>,
    },

    /// Update session metadata
    Update {
        /// Session name
        #[arg(long)]
        name: String,

        /// New description
        #[arg(long)]
        description: Option<String>,

        /// Add tags (comma-separated)
        #[arg(long)]
        add_tags: Option<String>,

        /// Remove tags (comma-separated)
        #[arg(long)]
        remove_tags: Option<String>,

        /// Update user agent
        #[arg(long)]
        user_agent: Option<String>,

        /// Update timeout in minutes
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Add cookies to session
    AddCookies {
        /// Session name
        #[arg(long)]
        name: String,

        /// Cookie name
        #[arg(long)]
        cookie_name: String,

        /// Cookie value
        #[arg(long)]
        cookie_value: String,

        /// Cookie domain
        #[arg(long)]
        domain: Option<String>,

        /// Cookie path
        #[arg(long, default_value = "/")]
        path: String,

        /// Cookie is secure
        #[arg(long)]
        secure: bool,

        /// Cookie is HTTP-only
        #[arg(long)]
        http_only: bool,

        /// Cookie expiration (RFC3339 format)
        #[arg(long)]
        expires: Option<String>,
    },

    /// Add headers to session
    AddHeaders {
        /// Session name
        #[arg(long)]
        name: String,

        /// Header name
        #[arg(long)]
        header_name: String,

        /// Header value
        #[arg(long)]
        header_value: String,
    },

    /// Clone an existing session
    Clone {
        /// Source session name
        #[arg(long)]
        from: String,

        /// New session name
        #[arg(long)]
        to: String,

        /// Clone cookies
        #[arg(long, default_value = "true")]
        cookies: bool,

        /// Clone headers
        #[arg(long, default_value = "true")]
        headers: bool,
    },

    /// Clear session data
    Clear {
        /// Session name
        #[arg(long)]
        name: String,

        /// Clear cookies
        #[arg(long)]
        cookies: bool,

        /// Clear headers
        #[arg(long)]
        headers: bool,

        /// Clear all data
        #[arg(long)]
        all: bool,
    },

    /// Show session statistics
    Stats {
        /// Session name (optional, shows all if not specified)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub cookies: Vec<Cookie>,
    pub headers: HashMap<String, String>,
    pub tags: Vec<String>,
    pub user_agent: Option<String>,
    pub timeout_minutes: u64,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionMetadata {
    pub requests_count: u64,
    pub last_request_url: Option<String>,
    pub success_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SessionManager {
    current_session: Option<String>,
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    fn new() -> Self {
        Self {
            current_session: None,
            sessions: HashMap::new(),
        }
    }

    fn load() -> Result<Self> {
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

    fn save(&self) -> Result<()> {
        let path = Self::storage_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    fn storage_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".riptide").join("sessions.json"))
    }

    fn add_session(&mut self, session: Session) -> Result<()> {
        if self.sessions.contains_key(&session.name) {
            return Err(anyhow!("Session '{}' already exists", session.name));
        }

        self.sessions.insert(session.name.clone(), session);
        Ok(())
    }

    fn get_session(&self, name: &str) -> Result<&Session> {
        self.sessions
            .get(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    fn get_session_mut(&mut self, name: &str) -> Result<&mut Session> {
        self.sessions
            .get_mut(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    fn remove_session(&mut self, name: &str) -> Result<Session> {
        self.sessions
            .remove(name)
            .ok_or_else(|| anyhow!("Session '{}' not found", name))
    }

    fn set_current(&mut self, name: String) -> Result<()> {
        if !self.sessions.contains_key(&name) {
            return Err(anyhow!("Session '{}' does not exist", name));
        }

        self.current_session = Some(name);
        Ok(())
    }

    fn get_current(&self) -> Option<&Session> {
        self.current_session
            .as_ref()
            .and_then(|name| self.sessions.get(name))
    }
}

pub async fn execute(command: SessionCommands, output_format: &str) -> Result<()> {
    match command {
        SessionCommands::New {
            name,
            description,
            cookies,
            headers,
            tags,
            user_agent,
            timeout,
        } => {
            create_session(
                name,
                description,
                cookies,
                headers,
                tags,
                user_agent,
                timeout,
                output_format,
            )
            .await
        }
        SessionCommands::List {
            tag,
            active,
            detailed,
        } => list_sessions(tag, active, detailed, output_format).await,
        SessionCommands::Use { name, create } => use_session(name, create, output_format).await,
        SessionCommands::Current { detailed } => current_session(detailed, output_format).await,
        SessionCommands::Export {
            name,
            output,
            format,
            include_cookies,
            include_headers,
        } => export_session(name, output, format, include_cookies, include_headers).await,
        SessionCommands::Import {
            input,
            name,
            overwrite,
        } => import_session(input, name, overwrite, output_format).await,
        SessionCommands::Rm { name, force, tag } => {
            remove_session(name, force, tag, output_format).await
        }
        SessionCommands::Update {
            name,
            description,
            add_tags,
            remove_tags,
            user_agent,
            timeout,
        } => {
            update_session(
                name,
                description,
                add_tags,
                remove_tags,
                user_agent,
                timeout,
                output_format,
            )
            .await
        }
        SessionCommands::AddCookies {
            name,
            cookie_name,
            cookie_value,
            domain,
            path,
            secure,
            http_only,
            expires,
        } => {
            add_cookies(
                name,
                cookie_name,
                cookie_value,
                domain,
                path,
                secure,
                http_only,
                expires,
                output_format,
            )
            .await
        }
        SessionCommands::AddHeaders {
            name,
            header_name,
            header_value,
        } => add_headers(name, header_name, header_value, output_format).await,
        SessionCommands::Clone {
            from,
            to,
            cookies,
            headers,
        } => clone_session(from, to, cookies, headers, output_format).await,
        SessionCommands::Clear {
            name,
            cookies,
            headers,
            all,
        } => clear_session(name, cookies, headers, all, output_format).await,
        SessionCommands::Stats { name } => session_stats(name, output_format).await,
    }
}

async fn create_session(
    name: String,
    description: Option<String>,
    cookies_file: Option<String>,
    headers_file: Option<String>,
    tags: Option<String>,
    user_agent: Option<String>,
    timeout: u64,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    // Parse cookies from file
    let cookies = if let Some(path) = cookies_file {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        Vec::new()
    };

    // Parse headers from file
    let headers = if let Some(path) = headers_file {
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content)?
    } else {
        HashMap::new()
    };

    // Parse tags
    let tags_vec = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let session = Session {
        name: name.clone(),
        description,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_used_at: None,
        cookies,
        headers,
        tags: tags_vec,
        user_agent,
        timeout_minutes: timeout,
        metadata: SessionMetadata {
            requests_count: 0,
            last_request_url: None,
            success_count: 0,
            error_count: 0,
        },
    };

    manager.add_session(session)?;
    manager.save()?;

    match output_format {
        "json" => output::print_json(&manager.get_session(&name)?),
        _ => {
            output::print_success(&format!("Session '{}' created successfully", name));
        }
    }

    Ok(())
}

async fn list_sessions(
    tag: Option<String>,
    active: bool,
    detailed: bool,
    output_format: &str,
) -> Result<()> {
    let manager = SessionManager::load()?;

    let mut sessions: Vec<&Session> = manager.sessions.values().collect();

    // Filter by tag if specified
    if let Some(filter_tag) = tag {
        sessions.retain(|s| s.tags.contains(&filter_tag));
    }

    // Filter by active if specified
    if active {
        sessions.retain(|s| {
            if let Some(timeout) = s.last_used_at {
                if s.timeout_minutes > 0 {
                    let elapsed = Utc::now().signed_duration_since(timeout);
                    elapsed.num_minutes() < s.timeout_minutes as i64
                } else {
                    true
                }
            } else {
                false
            }
        });
    }

    match output_format {
        "json" => output::print_json(&sessions),
        _ => {
            if sessions.is_empty() {
                output::print_info("No sessions found");
                return Ok(());
            }

            output::print_success(&format!("Found {} session(s)", sessions.len()));

            if detailed {
                for session in sessions {
                    println!("\n{}", "=".repeat(60));
                    display_session_detailed(session, &manager);
                }
            } else {
                let mut table = output::create_table(vec![
                    "Name",
                    "Created",
                    "Last Used",
                    "Cookies",
                    "Headers",
                    "Tags",
                ]);

                for session in sessions {
                    let is_current = manager.current_session.as_ref() == Some(&session.name);
                    let name = if is_current {
                        format!("{} *", session.name)
                    } else {
                        session.name.clone()
                    };

                    let last_used = session
                        .last_used_at
                        .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Never".to_string());

                    table.add_row(vec![
                        &name,
                        &session.created_at.format("%Y-%m-%d %H:%M").to_string(),
                        &last_used,
                        &session.cookies.len().to_string(),
                        &session.headers.len().to_string(),
                        &session.tags.join(", "),
                    ]);
                }
                println!("{table}");

                if let Some(current) = &manager.current_session {
                    output::print_info(&format!("\n* Current session: {}", current));
                }
            }
        }
    }

    Ok(())
}

fn display_session_detailed(session: &Session, manager: &SessionManager) {
    let is_current = manager.current_session.as_ref() == Some(&session.name);
    println!(
        "Name: {}{}",
        session.name,
        if is_current { " (current)" } else { "" }
    );

    if let Some(desc) = &session.description {
        println!("Description: {}", desc);
    }

    println!(
        "Created: {}",
        session.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Updated: {}",
        session.updated_at.format("%Y-%m-%d %H:%M:%S")
    );

    if let Some(last_used) = session.last_used_at {
        println!("Last Used: {}", last_used.format("%Y-%m-%d %H:%M:%S"));
    }

    if !session.tags.is_empty() {
        println!("Tags: {}", session.tags.join(", "));
    }

    if let Some(ua) = &session.user_agent {
        println!("User Agent: {}", ua);
    }

    if session.timeout_minutes > 0 {
        println!("Timeout: {} minutes", session.timeout_minutes);
    }

    println!("\nCookies: {}", session.cookies.len());
    for cookie in &session.cookies {
        println!("  - {} (domain: {:?})", cookie.name, cookie.domain);
    }

    println!("\nHeaders: {}", session.headers.len());
    for (name, value) in &session.headers {
        println!("  - {}: {}", name, value);
    }

    println!("\nStatistics:");
    println!("  Requests: {}", session.metadata.requests_count);
    println!("  Success: {}", session.metadata.success_count);
    println!("  Errors: {}", session.metadata.error_count);

    if let Some(url) = &session.metadata.last_request_url {
        println!("  Last URL: {}", url);
    }
}

async fn use_session(name: String, create: bool, output_format: &str) -> Result<()> {
    let mut manager = SessionManager::load()?;

    if !manager.sessions.contains_key(&name) {
        if create {
            // Create new session
            let session = Session {
                name: name.clone(),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_used_at: Some(Utc::now()),
                cookies: Vec::new(),
                headers: HashMap::new(),
                tags: Vec::new(),
                user_agent: None,
                timeout_minutes: 0,
                metadata: SessionMetadata {
                    requests_count: 0,
                    last_request_url: None,
                    success_count: 0,
                    error_count: 0,
                },
            };
            manager.add_session(session)?;
        } else {
            return Err(anyhow!(
                "Session '{}' not found. Use --create to create it.",
                name
            ));
        }
    }

    // Update last used time
    if let Ok(session) = manager.get_session_mut(&name) {
        session.last_used_at = Some(Utc::now());
        session.updated_at = Utc::now();
    }

    manager.set_current(name.clone())?;
    manager.save()?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({"current_session": name})),
        _ => {
            output::print_success(&format!("Switched to session '{}'", name));
        }
    }

    Ok(())
}

async fn current_session(detailed: bool, output_format: &str) -> Result<()> {
    let manager = SessionManager::load()?;

    match manager.get_current() {
        Some(session) => match output_format {
            "json" => output::print_json(&session),
            _ => {
                if detailed {
                    display_session_detailed(session, &manager);
                } else {
                    output::print_success(&format!("Current session: {}", session.name));
                }
            }
        },
        None => {
            output::print_info("No current session set");
        }
    }

    Ok(())
}

async fn export_session(
    name: String,
    output_path: String,
    format: String,
    include_cookies: bool,
    include_headers: bool,
) -> Result<()> {
    let manager = SessionManager::load()?;
    let mut session = manager.get_session(&name)?.clone();

    if !include_cookies {
        session.cookies.clear();
    }
    if !include_headers {
        session.headers.clear();
    }

    let content = match format.as_str() {
        "yaml" => serde_yaml::to_string(&session)?,
        _ => serde_json::to_string_pretty(&session)?,
    };

    fs::write(&output_path, content)?;
    output::print_success(&format!("Session exported to: {}", output_path));

    Ok(())
}

async fn import_session(
    input_path: String,
    name_override: Option<String>,
    overwrite: bool,
    output_format: &str,
) -> Result<()> {
    let content = fs::read_to_string(&input_path)?;

    let mut session: Session = if input_path.ends_with(".yaml") || input_path.ends_with(".yml") {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    if let Some(new_name) = name_override {
        session.name = new_name;
    }

    let mut manager = SessionManager::load()?;

    if manager.sessions.contains_key(&session.name) {
        if !overwrite {
            return Err(anyhow!(
                "Session '{}' already exists. Use --overwrite to replace it.",
                session.name
            ));
        }
        manager.remove_session(&session.name)?;
    }

    session.updated_at = Utc::now();

    let session_name = session.name.clone();
    manager.add_session(session)?;
    manager.save()?;

    match output_format {
        "json" => output::print_json(&manager.get_session(&session_name)?),
        _ => {
            output::print_success(&format!("Session '{}' imported successfully", session_name));
        }
    }

    Ok(())
}

async fn remove_session(
    name: String,
    force: bool,
    tag: Option<String>,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    let sessions_to_remove: Vec<String> = if let Some(filter_tag) = tag {
        manager
            .sessions
            .values()
            .filter(|s| s.tags.contains(&filter_tag))
            .map(|s| s.name.clone())
            .collect()
    } else {
        vec![name.clone()]
    };

    if sessions_to_remove.is_empty() {
        output::print_info("No sessions found to remove");
        return Ok(());
    }

    if !force {
        output::print_info(&format!(
            "About to remove {} session(s): {}",
            sessions_to_remove.len(),
            sessions_to_remove.join(", ")
        ));
        output::print_info("Use --force to confirm removal");
        return Ok(());
    }

    let mut removed_count = 0;
    for session_name in sessions_to_remove {
        if manager.remove_session(&session_name).is_ok() {
            removed_count += 1;
        }
    }

    manager.save()?;

    match output_format {
        "json" => output::print_json(&serde_json::json!({"removed": removed_count})),
        _ => {
            output::print_success(&format!("Removed {} session(s)", removed_count));
        }
    }

    Ok(())
}

async fn update_session(
    name: String,
    description: Option<String>,
    add_tags: Option<String>,
    remove_tags: Option<String>,
    user_agent: Option<String>,
    timeout: Option<u64>,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    {
        let session = manager.get_session_mut(&name)?;

        if let Some(desc) = description {
            session.description = Some(desc);
        }

        if let Some(tags_str) = add_tags {
            let new_tags: Vec<String> = tags_str.split(',').map(|s| s.trim().to_string()).collect();
            for tag in new_tags {
                if !session.tags.contains(&tag) {
                    session.tags.push(tag);
                }
            }
        }

        if let Some(tags_str) = remove_tags {
            let remove_tags: Vec<String> =
                tags_str.split(',').map(|s| s.trim().to_string()).collect();
            session.tags.retain(|t| !remove_tags.contains(t));
        }

        if let Some(ua) = user_agent {
            session.user_agent = Some(ua);
        }

        if let Some(t) = timeout {
            session.timeout_minutes = t;
        }

        session.updated_at = Utc::now();
    }

    manager.save()?;

    match output_format {
        "json" => output::print_json(manager.get_session(&name)?),
        _ => {
            output::print_success(&format!("Session '{}' updated successfully", name));
        }
    }

    Ok(())
}

async fn add_cookies(
    name: String,
    cookie_name: String,
    cookie_value: String,
    domain: Option<String>,
    path: String,
    secure: bool,
    http_only: bool,
    expires: Option<String>,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    {
        let session = manager.get_session_mut(&name)?;

        let expires_dt = if let Some(exp_str) = expires {
            Some(DateTime::parse_from_rfc3339(&exp_str)?.with_timezone(&Utc))
        } else {
            None
        };

        let cookie = Cookie {
            name: cookie_name.clone(),
            value: cookie_value,
            domain,
            path,
            secure,
            http_only,
            expires: expires_dt,
        };

        // Remove existing cookie with same name if it exists
        session.cookies.retain(|c| c.name != cookie_name);
        session.cookies.push(cookie);

        session.updated_at = Utc::now();
    }

    manager.save()?;

    match output_format {
        "json" => output::print_json(manager.get_session(&name)?),
        _ => {
            output::print_success(&format!(
                "Cookie '{}' added to session '{}'",
                cookie_name, name
            ));
        }
    }

    Ok(())
}

async fn add_headers(
    name: String,
    header_name: String,
    header_value: String,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    {
        let session = manager.get_session_mut(&name)?;
        session.headers.insert(header_name.clone(), header_value);
        session.updated_at = Utc::now();
    }

    manager.save()?;

    match output_format {
        "json" => output::print_json(manager.get_session(&name)?),
        _ => {
            output::print_success(&format!(
                "Header '{}' added to session '{}'",
                header_name, name
            ));
        }
    }

    Ok(())
}

async fn clone_session(
    from: String,
    to: String,
    clone_cookies: bool,
    clone_headers: bool,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;
    let source = manager.get_session(&from)?.clone();

    let new_session = Session {
        name: to.clone(),
        description: source.description.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_used_at: None,
        cookies: if clone_cookies {
            source.cookies.clone()
        } else {
            Vec::new()
        },
        headers: if clone_headers {
            source.headers.clone()
        } else {
            HashMap::new()
        },
        tags: source.tags.clone(),
        user_agent: source.user_agent.clone(),
        timeout_minutes: source.timeout_minutes,
        metadata: SessionMetadata {
            requests_count: 0,
            last_request_url: None,
            success_count: 0,
            error_count: 0,
        },
    };

    manager.add_session(new_session)?;
    manager.save()?;

    match output_format {
        "json" => output::print_json(&manager.get_session(&to)?),
        _ => {
            output::print_success(&format!("Session '{}' cloned to '{}'", from, to));
        }
    }

    Ok(())
}

async fn clear_session(
    name: String,
    clear_cookies: bool,
    clear_headers: bool,
    clear_all: bool,
    output_format: &str,
) -> Result<()> {
    let mut manager = SessionManager::load()?;

    {
        let session = manager.get_session_mut(&name)?;

        if clear_all || clear_cookies {
            session.cookies.clear();
        }

        if clear_all || clear_headers {
            session.headers.clear();
        }

        session.updated_at = Utc::now();
    }

    manager.save()?;

    match output_format {
        "json" => output::print_json(manager.get_session(&name)?),
        _ => {
            output::print_success(&format!("Session '{}' data cleared", name));
        }
    }

    Ok(())
}

async fn session_stats(name: Option<String>, output_format: &str) -> Result<()> {
    let manager = SessionManager::load()?;

    if let Some(session_name) = name {
        let session = manager.get_session(&session_name)?;
        match output_format {
            "json" => output::print_json(&session.metadata),
            _ => {
                output::print_success(&format!("Session '{}' Statistics", session_name));
                println!("Total Requests: {}", session.metadata.requests_count);
                println!("Successful: {}", session.metadata.success_count);
                println!("Errors: {}", session.metadata.error_count);

                if session.metadata.requests_count > 0 {
                    let success_rate = (session.metadata.success_count as f64
                        / session.metadata.requests_count as f64)
                        * 100.0;
                    println!("Success Rate: {:.1}%", success_rate);
                }

                if let Some(url) = &session.metadata.last_request_url {
                    println!("Last Request: {}", url);
                }
            }
        }
    } else {
        // Show stats for all sessions
        let total_sessions = manager.sessions.len();
        let total_requests: u64 = manager
            .sessions
            .values()
            .map(|s| s.metadata.requests_count)
            .sum();
        let total_success: u64 = manager
            .sessions
            .values()
            .map(|s| s.metadata.success_count)
            .sum();

        match output_format {
            "json" => output::print_json(&serde_json::json!({
                "total_sessions": total_sessions,
                "total_requests": total_requests,
                "total_success": total_success,
            })),
            _ => {
                output::print_success("Overall Session Statistics");
                println!("Total Sessions: {}", total_sessions);
                println!("Total Requests: {}", total_requests);
                println!("Total Successful: {}", total_success);

                if total_requests > 0 {
                    let success_rate = (total_success as f64 / total_requests as f64) * 100.0;
                    println!("Overall Success Rate: {:.1}%", success_rate);
                }
            }
        }
    }

    Ok(())
}
