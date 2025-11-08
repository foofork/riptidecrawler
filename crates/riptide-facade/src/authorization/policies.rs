//! Authorization Policy Implementations
//!
//! This module provides concrete implementations of authorization policies:
//! - `TenantScopingPolicy`: Prevents cross-tenant data access
//! - `RbacPolicy`: Role-based access control
//! - `ResourceOwnershipPolicy`: Owner-only access control
//!
//! All policies implement the `AuthorizationPolicy` trait and can be
//! composed together using `PolicyChain`.

use super::{AuthorizationContext, AuthorizationPolicy, Resource};
use riptide_types::error::{Result as RiptideResult, RiptideError};
use std::collections::HashMap;

/// Tenant Scoping Policy - Prevents cross-tenant access.
///
/// This policy ensures that users can only access resources within their
/// own tenant. It's critical for multi-tenancy isolation.
///
/// # Example
///
/// ```
/// use riptide_facade::authorization::{AuthorizationContext, Resource};
/// use riptide_facade::authorization::policies::TenantScopingPolicy;
/// use riptide_facade::authorization::AuthorizationPolicy;
/// use std::collections::HashSet;
///
/// let policy = TenantScopingPolicy::new();
///
/// let ctx = AuthorizationContext::new(
///     "user123",
///     "tenant1",
///     vec!["viewer"],
///     HashSet::new(),
/// );
///
/// // Same tenant - allowed
/// let resource = Resource::Extraction {
///     url: "https://example.com".to_string(),
///     tenant_id: "tenant1".to_string(),
/// };
/// assert!(policy.authorize(&ctx, &resource).is_ok());
///
/// // Different tenant - denied
/// let resource = Resource::Extraction {
///     url: "https://example.com".to_string(),
///     tenant_id: "tenant2".to_string(),
/// };
/// assert!(policy.authorize(&ctx, &resource).is_err());
/// ```
#[derive(Debug, Default)]
pub struct TenantScopingPolicy {
    /// If true, allow system admins to access all tenants
    allow_admin_override: bool,
}

impl TenantScopingPolicy {
    /// Create a new tenant scoping policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy that allows admins to bypass tenant checks.
    pub fn with_admin_override() -> Self {
        Self {
            allow_admin_override: true,
        }
    }
}

impl AuthorizationPolicy for TenantScopingPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        // Check for admin override
        if self.allow_admin_override && ctx.has_role("system_admin") {
            tracing::debug!(
                user_id = %ctx.user_id,
                tenant_id = %ctx.tenant_id,
                resource_type = %resource.resource_type(),
                "Tenant check bypassed for system admin"
            );
            return Ok(());
        }

        // If resource has a tenant ID, verify it matches user's tenant
        if let Some(resource_tenant_id) = resource.tenant_id() {
            if resource_tenant_id != ctx.tenant_id {
                tracing::warn!(
                    user_id = %ctx.user_id,
                    user_tenant = %ctx.tenant_id,
                    resource_tenant = %resource_tenant_id,
                    resource_type = %resource.resource_type(),
                    "Cross-tenant access denied"
                );

                return Err(RiptideError::PermissionDenied(format!(
                    "Access denied: resource belongs to tenant '{}', user belongs to tenant '{}'",
                    resource_tenant_id, ctx.tenant_id
                )));
            }
        }

        Ok(())
    }

    fn policy_name(&self) -> &'static str {
        "TenantScopingPolicy"
    }
}

/// Role-Based Access Control (RBAC) Policy.
///
/// This policy checks if the user has required roles to access resources.
/// Different resource types can require different roles.
///
/// # Example
///
/// ```
/// use riptide_facade::authorization::{AuthorizationContext, Resource};
/// use riptide_facade::authorization::policies::RbacPolicy;
/// use riptide_facade::authorization::AuthorizationPolicy;
/// use std::collections::{HashMap, HashSet};
///
/// let mut policy = RbacPolicy::new();
///
/// // URLs require 'viewer' or 'editor' role
/// policy.require_role_for_resource("url", vec!["viewer", "editor"]);
///
/// let ctx = AuthorizationContext::new(
///     "user123",
///     "tenant1",
///     vec!["viewer"],
///     HashSet::new(),
/// );
///
/// let resource = Resource::Url("https://example.com".to_string());
/// assert!(policy.authorize(&ctx, &resource).is_ok());
/// ```
#[derive(Debug, Default)]
pub struct RbacPolicy {
    /// Maps resource types to required roles
    resource_role_requirements: HashMap<String, Vec<String>>,
}

impl RbacPolicy {
    /// Create a new RBAC policy with default rules.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy with standard role requirements.
    pub fn with_defaults() -> Self {
        let mut policy = Self::new();

        // Standard role requirements
        policy.require_role_for_resource("url", vec!["viewer", "editor", "admin"]);
        policy.require_role_for_resource("extraction", vec!["editor", "admin"]);
        policy.require_role_for_resource("pipeline", vec!["editor", "admin"]);
        policy.require_role_for_resource("profile", vec!["admin"]);
        policy.require_role_for_resource("session", vec!["viewer", "editor", "admin"]);

        policy
    }

    /// Specify which roles are allowed to access a resource type.
    ///
    /// Any of the specified roles will grant access (OR logic).
    pub fn require_role_for_resource(
        &mut self,
        resource_type: impl Into<String>,
        allowed_roles: Vec<impl Into<String>>,
    ) {
        self.resource_role_requirements.insert(
            resource_type.into(),
            allowed_roles.into_iter().map(|r| r.into()).collect(),
        );
    }
}

impl AuthorizationPolicy for RbacPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        let resource_type = resource.resource_type();

        // If no role requirements defined for this resource type, allow
        let Some(required_roles) = self.resource_role_requirements.get(resource_type) else {
            tracing::debug!(
                resource_type = %resource_type,
                "No RBAC rules defined for resource type, allowing access"
            );
            return Ok(());
        };

        // Check if user has any of the required roles
        let has_required_role = required_roles.iter().any(|role| ctx.has_role(role));

        if !has_required_role {
            tracing::warn!(
                user_id = %ctx.user_id,
                user_roles = ?ctx.roles,
                required_roles = ?required_roles,
                resource_type = %resource_type,
                "RBAC check failed: user lacks required role"
            );

            return Err(RiptideError::PermissionDenied(format!(
                "Access denied: user must have one of these roles: {}",
                required_roles.join(", ")
            )));
        }

        tracing::debug!(
            user_id = %ctx.user_id,
            resource_type = %resource_type,
            "RBAC check passed"
        );

        Ok(())
    }

    fn policy_name(&self) -> &'static str {
        "RbacPolicy"
    }
}

/// Resource Ownership Policy - Restricts access to resource owners.
///
/// This policy maintains a mapping of resources to owners and ensures
/// only owners (or admins) can access the resource.
///
/// # Example
///
/// ```
/// use riptide_facade::authorization::{AuthorizationContext, Resource};
/// use riptide_facade::authorization::policies::ResourceOwnershipPolicy;
/// use riptide_facade::authorization::AuthorizationPolicy;
/// use std::collections::HashSet;
///
/// let mut policy = ResourceOwnershipPolicy::new();
///
/// // Register resource ownership
/// policy.register_owner("https://example.com", "user123");
///
/// let ctx = AuthorizationContext::new(
///     "user123",
///     "tenant1",
///     vec!["viewer"],
///     HashSet::new(),
/// );
///
/// let resource = Resource::Url("https://example.com".to_string());
/// assert!(policy.authorize(&ctx, &resource).is_ok());
///
/// // Different user - denied
/// let other_ctx = AuthorizationContext::new(
///     "user456",
///     "tenant1",
///     vec!["viewer"],
///     HashSet::new(),
/// );
/// assert!(policy.authorize(&other_ctx, &resource).is_err());
/// ```
#[derive(Debug, Default)]
pub struct ResourceOwnershipPolicy {
    /// Maps resource identifiers to owner user IDs
    resource_owners: HashMap<String, String>,

    /// If true, users with 'admin' role can access all resources
    allow_admin_override: bool,
}

impl ResourceOwnershipPolicy {
    /// Create a new resource ownership policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy that allows admins to access all resources.
    pub fn with_admin_override() -> Self {
        Self {
            resource_owners: HashMap::new(),
            allow_admin_override: true,
        }
    }

    /// Register a resource owner.
    ///
    /// This records that a specific user owns a resource.
    pub fn register_owner(&mut self, resource_id: impl Into<String>, owner_id: impl Into<String>) {
        self.resource_owners
            .insert(resource_id.into(), owner_id.into());
    }

    /// Remove resource ownership record.
    pub fn unregister_resource(&mut self, resource_id: &str) {
        self.resource_owners.remove(resource_id);
    }

    /// Check if a user owns a resource.
    pub fn is_owner(&self, resource_id: &str, user_id: &str) -> bool {
        self.resource_owners
            .get(resource_id)
            .map(|owner| owner == user_id)
            .unwrap_or(false)
    }
}

impl AuthorizationPolicy for ResourceOwnershipPolicy {
    fn authorize(&self, ctx: &AuthorizationContext, resource: &Resource) -> RiptideResult<()> {
        // Admin override
        if self.allow_admin_override && ctx.has_role("admin") {
            tracing::debug!(
                user_id = %ctx.user_id,
                resource_type = %resource.resource_type(),
                "Ownership check bypassed for admin"
            );
            return Ok(());
        }

        let resource_id = resource.identifier();

        // If no ownership record exists, allow (resource not tracked)
        let Some(owner_id) = self.resource_owners.get(&resource_id) else {
            tracing::debug!(
                resource_id = %resource_id,
                "No ownership record found, allowing access"
            );
            return Ok(());
        };

        // Check ownership
        if owner_id != &ctx.user_id {
            tracing::warn!(
                user_id = %ctx.user_id,
                owner_id = %owner_id,
                resource_id = %resource_id,
                "Ownership check failed: user is not the owner"
            );

            return Err(RiptideError::PermissionDenied(format!(
                "Access denied: resource '{}' is owned by another user",
                resource_id
            )));
        }

        tracing::debug!(
            user_id = %ctx.user_id,
            resource_id = %resource_id,
            "Ownership check passed"
        );

        Ok(())
    }

    fn policy_name(&self) -> &'static str {
        "ResourceOwnershipPolicy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn create_test_context(
        user_id: &str,
        tenant_id: &str,
        roles: Vec<&str>,
    ) -> AuthorizationContext {
        AuthorizationContext::new(user_id, tenant_id, roles, HashSet::new())
    }

    // ============================================================================
    // TenantScopingPolicy Tests
    // ============================================================================

    #[test]
    fn test_tenant_scoping_same_tenant() {
        let policy = TenantScopingPolicy::new();
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant1".to_string(),
        };

        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_tenant_scoping_different_tenant() {
        let policy = TenantScopingPolicy::new();
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant2".to_string(),
        };

        let result = policy.authorize(&ctx, &resource);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cross-tenant"));
    }

    #[test]
    fn test_tenant_scoping_no_tenant_resource() {
        let policy = TenantScopingPolicy::new();
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let resource = Resource::Url("https://example.com".to_string());

        // Resources without tenant ID should be allowed
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_tenant_scoping_admin_override() {
        let policy = TenantScopingPolicy::with_admin_override();
        let ctx = create_test_context("admin123", "tenant1", vec!["system_admin"]);

        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant2".to_string(),
        };

        // Admin should bypass tenant check
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    // ============================================================================
    // RbacPolicy Tests
    // ============================================================================

    #[test]
    fn test_rbac_with_required_role() {
        let mut policy = RbacPolicy::new();
        policy.require_role_for_resource("url", vec!["viewer", "editor"]);

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_rbac_without_required_role() {
        let mut policy = RbacPolicy::new();
        policy.require_role_for_resource("url", vec!["admin"]);

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        let result = policy.authorize(&ctx, &resource);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required role"));
    }

    #[test]
    fn test_rbac_no_rules_defined() {
        let policy = RbacPolicy::new();
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        // No rules defined - should allow
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_rbac_with_defaults() {
        let policy = RbacPolicy::with_defaults();

        // Viewer can access URLs
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());
        assert!(policy.authorize(&ctx, &resource).is_ok());

        // Viewer cannot access extractions
        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant1".to_string(),
        };
        assert!(policy.authorize(&ctx, &resource).is_err());

        // Editor can access extractions
        let ctx = create_test_context("user123", "tenant1", vec!["editor"]);
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_rbac_multiple_roles() {
        let mut policy = RbacPolicy::new();
        policy.require_role_for_resource("url", vec!["admin"]);

        let ctx = create_test_context("user123", "tenant1", vec!["viewer", "admin"]);
        let resource = Resource::Url("https://example.com".to_string());

        // User has multiple roles, one of which matches
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    // ============================================================================
    // ResourceOwnershipPolicy Tests
    // ============================================================================

    #[test]
    fn test_ownership_owner_access() {
        let mut policy = ResourceOwnershipPolicy::new();
        policy.register_owner("https://example.com", "user123");

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_ownership_non_owner_access() {
        let mut policy = ResourceOwnershipPolicy::new();
        policy.register_owner("https://example.com", "user123");

        let ctx = create_test_context("user456", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        let result = policy.authorize(&ctx, &resource);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("owned by another"));
    }

    #[test]
    fn test_ownership_no_record() {
        let policy = ResourceOwnershipPolicy::new();
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        // No ownership record - should allow
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_ownership_admin_override() {
        let mut policy = ResourceOwnershipPolicy::with_admin_override();
        policy.register_owner("https://example.com", "user123");

        let ctx = create_test_context("admin456", "tenant1", vec!["admin"]);
        let resource = Resource::Url("https://example.com".to_string());

        // Admin should bypass ownership check
        assert!(policy.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_ownership_is_owner() {
        let mut policy = ResourceOwnershipPolicy::new();
        policy.register_owner("resource1", "user123");

        assert!(policy.is_owner("resource1", "user123"));
        assert!(!policy.is_owner("resource1", "user456"));
        assert!(!policy.is_owner("resource2", "user123"));
    }

    #[test]
    fn test_ownership_unregister() {
        let mut policy = ResourceOwnershipPolicy::new();
        policy.register_owner("resource1", "user123");

        assert!(policy.is_owner("resource1", "user123"));

        policy.unregister_resource("resource1");

        assert!(!policy.is_owner("resource1", "user123"));
    }

    // ============================================================================
    // Integration Tests
    // ============================================================================

    #[test]
    fn test_policy_chain_all_pass() {
        use crate::authorization::PolicyChain;

        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["viewer"]);

        let tenant_policy = TenantScopingPolicy::new();

        let chain = PolicyChain::new()
            .add_policy(Box::new(tenant_policy))
            .add_policy(Box::new(rbac));

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        assert!(chain.authorize(&ctx, &resource).is_ok());
    }

    #[test]
    fn test_policy_chain_one_fails() {
        use crate::authorization::PolicyChain;

        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["admin"]);

        let tenant_policy = TenantScopingPolicy::new();

        let chain = PolicyChain::new()
            .add_policy(Box::new(tenant_policy))
            .add_policy(Box::new(rbac));

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
        let resource = Resource::Url("https://example.com".to_string());

        // RBAC should fail
        assert!(chain.authorize(&ctx, &resource).is_err());
    }
}
