//! Integration tests for Authorization Framework
//!
//! These tests verify the authorization system works correctly
//! in isolation from the rest of the facade layer.

use riptide_facade::authorization::policies::{
    RbacPolicy, ResourceOwnershipPolicy, TenantScopingPolicy,
};
use riptide_facade::authorization::{
    AuthorizationContext, AuthorizationPolicy, PolicyChain, Resource,
};
use std::collections::HashSet;

fn create_test_context(user_id: &str, tenant_id: &str, roles: Vec<&str>) -> AuthorizationContext {
    AuthorizationContext::new(user_id, tenant_id, roles, HashSet::new())
}

#[test]
fn test_authorization_context_creation() {
    let ctx = AuthorizationContext::new(
        "user123",
        "tenant1",
        vec!["admin"],
        HashSet::from(["read:all".to_string()]),
    );

    assert_eq!(ctx.user_id, "user123");
    assert_eq!(ctx.tenant_id, "tenant1");
    assert!(ctx.has_role("admin"));
    assert!(ctx.has_permission("read:all"));
}

#[test]
fn test_tenant_scoping_policy() {
    let policy = TenantScopingPolicy::new();
    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

    // Same tenant - allowed
    let resource = Resource::Extraction {
        url: "https://example.com".to_string(),
        tenant_id: "tenant1".to_string(),
    };
    assert!(policy.authorize(&ctx, &resource).is_ok());

    // Different tenant - denied
    let resource = Resource::Extraction {
        url: "https://example.com".to_string(),
        tenant_id: "tenant2".to_string(),
    };
    assert!(policy.authorize(&ctx, &resource).is_err());
}

#[test]
fn test_rbac_policy() {
    let mut policy = RbacPolicy::new();
    policy.require_role_for_resource("url", vec!["viewer", "editor"]);

    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
    let resource = Resource::Url("https://example.com".to_string());

    // Viewer can access URLs
    assert!(policy.authorize(&ctx, &resource).is_ok());

    // Non-viewer cannot
    let ctx = create_test_context("user123", "tenant1", vec!["guest"]);
    assert!(policy.authorize(&ctx, &resource).is_err());
}

#[test]
fn test_resource_ownership_policy() {
    let mut policy = ResourceOwnershipPolicy::new();
    policy.register_owner("resource1", "user123");

    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
    let resource = Resource::Custom {
        resource_type: "document".to_string(),
        resource_id: "resource1".to_string(),
    };

    // Owner can access
    assert!(policy.authorize(&ctx, &resource).is_ok());

    // Non-owner cannot
    let ctx = create_test_context("user456", "tenant1", vec!["viewer"]);
    assert!(policy.authorize(&ctx, &resource).is_err());
}

#[test]
fn test_policy_chain_multiple_policies() {
    let mut rbac = RbacPolicy::new();
    rbac.require_role_for_resource("url", vec!["viewer"]);

    let chain = PolicyChain::new()
        .add_policy(Box::new(TenantScopingPolicy::new()))
        .add_policy(Box::new(rbac));

    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
    let resource = Resource::Url("https://example.com".to_string());

    // Both policies should pass
    assert!(chain.authorize(&ctx, &resource).is_ok());
}

#[test]
fn test_policy_chain_one_fails() {
    let mut rbac = RbacPolicy::new();
    rbac.require_role_for_resource("url", vec!["admin"]);

    let chain = PolicyChain::new()
        .add_policy(Box::new(TenantScopingPolicy::new()))
        .add_policy(Box::new(rbac));

    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
    let resource = Resource::Url("https://example.com".to_string());

    // RBAC should fail
    assert!(chain.authorize(&ctx, &resource).is_err());
}

#[test]
fn test_admin_overrides() {
    // Tenant scoping with admin override
    let policy = TenantScopingPolicy::with_admin_override();
    let ctx = create_test_context("admin", "tenant1", vec!["system_admin"]);

    let resource = Resource::Extraction {
        url: "https://example.com".to_string(),
        tenant_id: "tenant2".to_string(),
    };

    // Admin should bypass tenant check
    assert!(policy.authorize(&ctx, &resource).is_ok());

    // Ownership with admin override
    let mut ownership_policy = ResourceOwnershipPolicy::with_admin_override();
    ownership_policy.register_owner("resource1", "user123");

    let ctx = create_test_context("admin", "tenant1", vec!["admin"]);
    let resource = Resource::Custom {
        resource_type: "document".to_string(),
        resource_id: "resource1".to_string(),
    };

    // Admin should bypass ownership check
    assert!(ownership_policy.authorize(&ctx, &resource).is_ok());
}

#[test]
fn test_resource_types() {
    let url_resource = Resource::Url("https://example.com".to_string());
    assert_eq!(url_resource.resource_type(), "url");
    assert_eq!(url_resource.identifier(), "https://example.com");
    assert!(url_resource.tenant_id().is_none());

    let extraction_resource = Resource::Extraction {
        url: "https://example.com".to_string(),
        tenant_id: "tenant1".to_string(),
    };
    assert_eq!(extraction_resource.resource_type(), "extraction");
    assert_eq!(extraction_resource.tenant_id(), Some("tenant1"));

    let profile_resource = Resource::Profile("profile123".to_string());
    assert_eq!(profile_resource.resource_type(), "profile");
    assert_eq!(profile_resource.identifier(), "profile123");

    let session_resource = Resource::Session("session456".to_string());
    assert_eq!(session_resource.resource_type(), "session");

    let pipeline_resource = Resource::Pipeline {
        pipeline_id: "pipe789".to_string(),
        tenant_id: "tenant1".to_string(),
    };
    assert_eq!(pipeline_resource.resource_type(), "pipeline");
    assert_eq!(pipeline_resource.tenant_id(), Some("tenant1"));
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

    // Admin can access profiles
    let ctx = create_test_context("user123", "tenant1", vec!["admin"]);
    let resource = Resource::Profile("profile123".to_string());
    assert!(policy.authorize(&ctx, &resource).is_ok());

    // Viewer cannot access profiles
    let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);
    assert!(policy.authorize(&ctx, &resource).is_err());
}

#[test]
fn test_complex_authorization_scenario() {
    // Scenario: Multi-tenant system with RBAC and ownership
    let mut rbac = RbacPolicy::with_defaults();
    let mut ownership = ResourceOwnershipPolicy::with_admin_override();

    // Register some resources
    ownership.register_owner("resource1", "user123");
    ownership.register_owner("resource2", "user456");

    let chain = PolicyChain::new()
        .add_policy(Box::new(TenantScopingPolicy::new()))
        .add_policy(Box::new(rbac))
        .add_policy(Box::new(ownership));

    // User can access their own resource
    let ctx = create_test_context("user123", "tenant1", vec!["editor"]);
    let resource = Resource::Custom {
        resource_type: "extraction".to_string(),
        resource_id: "resource1".to_string(),
    };
    assert!(chain.authorize(&ctx, &resource).is_ok());

    // User cannot access other user's resource
    let resource = Resource::Custom {
        resource_type: "extraction".to_string(),
        resource_id: "resource2".to_string(),
    };
    assert!(chain.authorize(&ctx, &resource).is_err());

    // Admin can access all resources
    let ctx = create_test_context("admin", "tenant1", vec!["admin"]);
    let resource = Resource::Custom {
        resource_type: "extraction".to_string(),
        resource_id: "resource2".to_string(),
    };
    assert!(chain.authorize(&ctx, &resource).is_ok());
}
