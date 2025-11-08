//! Authorization Framework for Riptide
//!
//! This module provides authorization policies and context management for
//! securing access to application facades. It supports:
//! - Tenant scoping (multi-tenancy isolation)
//! - Role-based access control (RBAC)
//! - Resource ownership policies
//! - Fine-grained permission checks
//!
//! ## Architecture
//!
//! The authorization framework follows the hexagonal architecture principles:
//! - Policies are defined as traits (port interfaces)
//! - Concrete policy implementations are in this module
//! - Facades integrate policies through dependency injection
//! - No infrastructure concerns (HTTP, database, etc.)
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use riptide_facade::authorization::{AuthorizationContext, Resource, AuthorizationPolicy};
//! use std::collections::HashSet;
//!
//! let ctx = AuthorizationContext {
//!     user_id: "user123".to_string(),
//!     tenant_id: "tenant1".to_string(),
//!     roles: vec!["viewer".to_string()],
//!     permissions: HashSet::from(["read:urls".to_string()]),
//! };
//!
//! let resource = Resource::Url("https://example.com".to_string());
//!
//! // Policy checks happen before business logic
//! policy.authorize(&ctx, &resource)?;
//! ```

pub mod policies;

use riptide_types::error::Result as RiptideResult;
use std::collections::HashSet;

/// Authorization context carrying user identity and permissions.
///
/// This struct is passed to all facade methods requiring authorization.
/// It contains all information needed to make authorization decisions.
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    /// Unique user identifier
    pub user_id: String,

    /// Tenant identifier for multi-tenancy isolation
    pub tenant_id: String,

    /// User roles (e.g., "admin", "editor", "viewer")
    pub roles: Vec<String>,

    /// Explicit permissions (e.g., "read:urls", "write:extractions")
    pub permissions: HashSet<String>,
}

impl AuthorizationContext {
    /// Create a new authorization context.
    ///
    /// # Example
    ///
    /// ```
    /// use riptide_facade::authorization::AuthorizationContext;
    /// use std::collections::HashSet;
    ///
    /// let ctx = AuthorizationContext::new(
    ///     "user123",
    ///     "tenant1",
    ///     vec!["viewer"],
    ///     HashSet::from(["read:urls".to_string()]),
    /// );
    /// ```
    pub fn new(
        user_id: impl Into<String>,
        tenant_id: impl Into<String>,
        roles: Vec<impl Into<String>>,
        permissions: HashSet<String>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            tenant_id: tenant_id.into(),
            roles: roles.into_iter().map(|r| r.into()).collect(),
            permissions,
        }
    }

    /// Check if user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has a specific permission.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if user has any of the specified roles.
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified permissions.
    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions.iter().all(|perm| self.has_permission(perm))
    }
}

/// Resource types that can be authorized.
///
/// Each variant represents a different type of resource that requires
/// authorization checks in the application layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    /// URL resource (for extraction, scraping)
    Url(String),

    /// Browser profile resource
    Profile(String),

    /// Browser session resource
    Session(String),

    /// Extraction operation with URL and tenant scoping
    Extraction { url: String, tenant_id: String },

    /// Pipeline execution resource
    Pipeline {
        pipeline_id: String,
        tenant_id: String,
    },

    /// Any resource with custom identifier
    Custom {
        resource_type: String,
        resource_id: String,
    },
}

impl Resource {
    /// Get the tenant ID associated with this resource, if any.
    pub fn tenant_id(&self) -> Option<&str> {
        match self {
            Resource::Extraction { tenant_id, .. } => Some(tenant_id),
            Resource::Pipeline { tenant_id, .. } => Some(tenant_id),
            _ => None,
        }
    }

    /// Get the resource identifier.
    pub fn identifier(&self) -> String {
        match self {
            Resource::Url(url) => url.clone(),
            Resource::Profile(id) => id.clone(),
            Resource::Session(id) => id.clone(),
            Resource::Extraction { url, .. } => url.clone(),
            Resource::Pipeline { pipeline_id, .. } => pipeline_id.clone(),
            Resource::Custom { resource_id, .. } => resource_id.clone(),
        }
    }

    /// Get the resource type as a string.
    pub fn resource_type(&self) -> &str {
        match self {
            Resource::Url(_) => "url",
            Resource::Profile(_) => "profile",
            Resource::Session(_) => "session",
            Resource::Extraction { .. } => "extraction",
            Resource::Pipeline { .. } => "pipeline",
            Resource::Custom { resource_type, .. } => resource_type,
        }
    }
}

/// Authorization policy trait.
///
/// Policies implement specific authorization rules (tenant scoping, RBAC, etc.).
/// They are composable - facades can use multiple policies in sequence.
///
/// # Thread Safety
///
/// Policies must be `Send + Sync` to be used in async contexts.
pub trait AuthorizationPolicy: Send + Sync {
    /// Authorize an operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Authorization context with user identity and permissions
    /// * `resource` - Resource being accessed
    ///
    /// # Returns
    ///
    /// * `Ok(())` if authorized
    /// * `Err(RiptideError::PermissionDenied)` if not authorized
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = policy.authorize(&ctx, &resource);
    /// match result {
    ///     Ok(()) => println!("Authorized"),
    ///     Err(e) => println!("Denied: {}", e),
    /// }
    /// ```
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()>;

    /// Get the policy name for logging/debugging.
    fn policy_name(&self) -> &'static str {
        "UnnamedPolicy"
    }
}

/// Collection of authorization policies applied in sequence.
///
/// This allows composing multiple policies together. All policies must
/// pass for authorization to succeed.
#[derive(Default)]
pub struct PolicyChain {
    policies: Vec<Box<dyn AuthorizationPolicy>>,
}

impl PolicyChain {
    /// Create a new empty policy chain.
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Add a policy to the chain.
    pub fn add_policy(mut self, policy: Box<dyn AuthorizationPolicy>) -> Self {
        self.policies.push(policy);
        self
    }

    /// Check all policies in the chain.
    ///
    /// Returns `Ok(())` only if all policies pass.
    pub fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        for policy in &self.policies {
            policy.authorize(ctx, resource)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorization_context_creation() {
        let ctx = AuthorizationContext::new(
            "user123",
            "tenant1",
            vec!["admin", "editor"],
            HashSet::from(["read:urls".to_string(), "write:urls".to_string()]),
        );

        assert_eq!(ctx.user_id, "user123");
        assert_eq!(ctx.tenant_id, "tenant1");
        assert_eq!(ctx.roles.len(), 2);
        assert_eq!(ctx.permissions.len(), 2);
    }

    #[test]
    fn test_has_role() {
        let ctx = AuthorizationContext::new(
            "user123",
            "tenant1",
            vec!["admin", "editor"],
            HashSet::new(),
        );

        assert!(ctx.has_role("admin"));
        assert!(ctx.has_role("editor"));
        assert!(!ctx.has_role("viewer"));
    }

    #[test]
    fn test_has_permission() {
        let ctx = AuthorizationContext::new(
            "user123",
            "tenant1",
            vec!["viewer"],
            HashSet::from(["read:urls".to_string()]),
        );

        assert!(ctx.has_permission("read:urls"));
        assert!(!ctx.has_permission("write:urls"));
    }

    #[test]
    fn test_has_any_role() {
        let ctx = AuthorizationContext::new("user123", "tenant1", vec!["editor"], HashSet::new());

        assert!(ctx.has_any_role(&["admin", "editor"]));
        assert!(!ctx.has_any_role(&["admin", "viewer"]));
    }

    #[test]
    fn test_has_all_permissions() {
        let ctx = AuthorizationContext::new(
            "user123",
            "tenant1",
            vec!["admin"],
            HashSet::from(["read:urls".to_string(), "write:urls".to_string()]),
        );

        assert!(ctx.has_all_permissions(&["read:urls", "write:urls"]));
        assert!(!ctx.has_all_permissions(&["read:urls", "delete:urls"]));
    }

    #[test]
    fn test_resource_tenant_id() {
        let url_resource = Resource::Url("https://example.com".to_string());
        assert!(url_resource.tenant_id().is_none());

        let extraction_resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant1".to_string(),
        };
        assert_eq!(extraction_resource.tenant_id(), Some("tenant1"));
    }

    #[test]
    fn test_resource_identifier() {
        let url_resource = Resource::Url("https://example.com".to_string());
        assert_eq!(url_resource.identifier(), "https://example.com");

        let profile_resource = Resource::Profile("profile123".to_string());
        assert_eq!(profile_resource.identifier(), "profile123");
    }

    #[test]
    fn test_resource_type() {
        let url_resource = Resource::Url("https://example.com".to_string());
        assert_eq!(url_resource.resource_type(), "url");

        let extraction_resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant1".to_string(),
        };
        assert_eq!(extraction_resource.resource_type(), "extraction");
    }

    #[test]
    fn test_policy_chain_empty() {
        let chain = PolicyChain::new();
        let ctx = AuthorizationContext::new("user123", "tenant1", vec!["admin"], HashSet::new());
        let resource = Resource::Url("https://example.com".to_string());

        // Empty chain should always authorize
        assert!(chain.authorize(&ctx, &resource).is_ok());
    }
}
