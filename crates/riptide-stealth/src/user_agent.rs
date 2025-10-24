//! User agent management and rotation strategies
//!
//! This module handles user agent rotation, filtering, and management
//! to provide realistic and varied browser identification.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// User agent rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentConfig {
    /// Pool of user agents to rotate through
    pub agents: Vec<String>,

    /// Rotation strategy
    pub strategy: RotationStrategy,

    /// Whether to include mobile user agents
    pub include_mobile: bool,

    /// Browser type preference
    pub browser_preference: BrowserType,
}

impl Default for UserAgentConfig {
    fn default() -> Self {
        Self {
            agents: default_user_agents(),
            strategy: RotationStrategy::Random,
            include_mobile: false,
            browser_preference: BrowserType::Chrome,
        }
    }
}

/// User agent rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Pick randomly from the pool
    Random,

    /// Rotate in sequence
    Sequential,

    /// Use the same agent for the entire session
    Sticky,

    /// Choose based on target domain
    DomainBased,
}

/// Browser type preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
    Mixed,
}

/// User agent manager for handling rotation and selection
pub struct UserAgentManager {
    config: UserAgentConfig,
    current_index: usize,
    session_user_agent: Option<String>,
    request_count: u64,
}

impl UserAgentManager {
    /// Create a new user agent manager
    pub fn new(config: UserAgentConfig) -> Self {
        Self {
            config,
            current_index: 0,
            session_user_agent: None,
            request_count: 0,
        }
    }

    /// Get the next user agent based on rotation strategy
    pub fn next_user_agent(&mut self) -> &str {
        if self.config.agents.is_empty() {
            return "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        }

        match self.config.strategy {
            RotationStrategy::Random => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..self.config.agents.len());
                &self.config.agents[index]
            }
            RotationStrategy::Sequential => {
                self.current_index = (self.current_index + 1) % self.config.agents.len();
                &self.config.agents[self.current_index]
            }
            RotationStrategy::Sticky => {
                if self.session_user_agent.is_none() {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..self.config.agents.len());
                    self.session_user_agent = Some(self.config.agents[index].clone());
                }
                // Safe to unwrap: we just ensured session_user_agent is Some above
                self.session_user_agent
                    .as_ref()
                    .unwrap_or(&self.config.agents[0])
            }
            RotationStrategy::DomainBased => {
                // Simple domain-based selection using hash
                let domain_hash = self.request_count.wrapping_mul(37) as usize;
                let index = domain_hash % self.config.agents.len();
                &self.config.agents[index]
            }
        }
    }

    /// Get current user agent without rotation
    pub fn current_user_agent(&self) -> Option<&String> {
        match self.config.strategy {
            RotationStrategy::Sticky => self.session_user_agent.as_ref(),
            _ => self.config.agents.get(self.current_index),
        }
    }

    /// Filter user agents by browser type
    pub fn filter_by_browser_type(&mut self, browser_type: BrowserType) {
        self.config.agents = self
            .config
            .agents
            .iter()
            .filter(|ua| matches_browser_type(ua, &browser_type))
            .cloned()
            .collect();
    }

    /// Add custom user agents
    pub fn add_user_agents(&mut self, new_agents: Vec<String>) {
        self.config.agents.extend(new_agents);
    }

    /// Remove mobile user agents
    pub fn remove_mobile_agents(&mut self) {
        self.config.agents = self
            .config
            .agents
            .iter()
            .filter(|ua| !is_mobile_user_agent(ua))
            .cloned()
            .collect();
    }

    /// Increment request count for domain-based strategy
    pub fn increment_request_count(&mut self) {
        self.request_count += 1;
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.config.agents.len()
    }
}

/// Check if user agent matches browser type
fn matches_browser_type(user_agent: &str, browser_type: &BrowserType) -> bool {
    match browser_type {
        BrowserType::Chrome => user_agent.contains("Chrome") && !user_agent.contains("Edge"),
        BrowserType::Firefox => user_agent.contains("Firefox"),
        BrowserType::Safari => user_agent.contains("Safari") && !user_agent.contains("Chrome"),
        BrowserType::Edge => user_agent.contains("Edge"),
        BrowserType::Mixed => true,
    }
}

/// Check if user agent is for mobile device
fn is_mobile_user_agent(user_agent: &str) -> bool {
    user_agent.contains("Mobile")
        || user_agent.contains("Android")
        || user_agent.contains("iPhone")
        || user_agent.contains("iPad")
}

/// Default user agents for rotation
fn default_user_agents() -> Vec<String> {
    vec![
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string(),
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15".to_string(),
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_manager_creation() {
        let config = UserAgentConfig::default();
        let manager = UserAgentManager::new(config);
        assert!(manager.agent_count() > 0);
    }

    #[test]
    fn test_sequential_rotation() {
        let config = UserAgentConfig {
            strategy: RotationStrategy::Sequential,
            agents: vec!["UA1".to_string(), "UA2".to_string(), "UA3".to_string()],
            ..Default::default()
        };

        let mut manager = UserAgentManager::new(config);

        let ua1 = manager.next_user_agent().to_string();
        let ua2 = manager.next_user_agent().to_string();
        let ua3 = manager.next_user_agent().to_string();
        let ua4 = manager.next_user_agent().to_string(); // Should wrap around

        assert_eq!(ua1, "UA2"); // Index starts at 0, increments to 1
        assert_eq!(ua2, "UA3");
        assert_eq!(ua3, "UA1"); // Wraps to index 0
        assert_eq!(ua4, "UA2"); // Back to index 1
    }

    #[test]
    fn test_sticky_rotation() {
        let config = UserAgentConfig {
            strategy: RotationStrategy::Sticky,
            agents: vec!["UA1".to_string(), "UA2".to_string(), "UA3".to_string()],
            ..Default::default()
        };

        let mut manager = UserAgentManager::new(config);

        let ua1 = manager.next_user_agent().to_string();
        let ua2 = manager.next_user_agent().to_string();

        assert_eq!(ua1, ua2); // Should be the same for sticky strategy
    }

    #[test]
    fn test_browser_type_filtering() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        assert!(matches_browser_type(ua, &BrowserType::Chrome));
        assert!(!matches_browser_type(ua, &BrowserType::Firefox));
    }

    #[test]
    fn test_mobile_detection() {
        let mobile_ua =
            "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15";
        let desktop_ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

        assert!(is_mobile_user_agent(mobile_ua));
        assert!(!is_mobile_user_agent(desktop_ua));
    }
}
