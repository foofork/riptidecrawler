# Authorization Framework

## Overview

The Riptide Authorization Framework provides a comprehensive, policy-based authorization system for securing application facades. It implements **Phase 2 Sprint 2.1** of the Riptide refactoring roadmap.

## Features

- **Tenant Scoping**: Multi-tenancy isolation preventing cross-tenant access
- **Role-Based Access Control (RBAC)**: Flexible role and permission management
- **Resource Ownership**: Fine-grained ownership-based access control
- **Policy Composition**: Chain multiple policies for complex authorization rules
- **Admin Overrides**: System administrators can bypass certain restrictions
- **Full Type Safety**: Leverages Rust's type system for compile-time guarantees

## Architecture

The authorization framework follows hexagonal architecture principles:

```
API Layer (riptide-api)
      ↓ creates AuthorizationContext
APPLICATION LAYER (riptide-facade/authorization) ← YOU ARE HERE
      ↓ uses policies
Domain Layer (riptide-types)
```

### Core Components

1. **AuthorizationContext**: Carries user identity, tenant ID, roles, and permissions
2. **Resource**: Enum representing different resource types (URLs, profiles, sessions, etc.)
3. **AuthorizationPolicy**: Trait for implementing authorization rules
4. **PolicyChain**: Composes multiple policies together

## Usage

### Basic Example

```rust
use riptide_facade::authorization::{AuthorizationContext, Resource};
use riptide_facade::authorization::policies::TenantScopingPolicy;
use std::collections::HashSet;

// Create authorization context
let ctx = AuthorizationContext::new(
    "user123",
    "tenant1",
    vec!["viewer"],
    HashSet::from(["read:urls".to_string()]),
);

// Create resource
let resource = Resource::Url("https://example.com".to_string());

// Apply policy
let policy = TenantScopingPolicy::new();
policy.authorize(&ctx, &resource)?;
```

### Using with Extraction Facade

```rust
use riptide_facade::authorization::{AuthorizationContext, AuthorizationPolicy};
use riptide_facade::authorization::policies::{RbacPolicy, TenantScopingPolicy};
use riptide_facade::facades::{UrlExtractionFacade, AuthorizedExtractionFacade};
use std::sync::Arc;

// Create facade
let facade = UrlExtractionFacade::new(...).await?;

// Setup authorization policies
let mut rbac = RbacPolicy::with_defaults();
let policies: Vec<Arc<dyn AuthorizationPolicy>> = vec![
    Arc::new(TenantScopingPolicy::new()),
    Arc::new(rbac),
];

// Create authorization context
let ctx = AuthorizationContext::new(
    "user123",
    "tenant1",
    vec!["editor"],
    HashSet::new(),
);

// Extract with authorization
let result = facade.extract_with_authorization(
    "https://example.com",
    options,
    &ctx,
    &policies,
).await?;
```

## Authorization Policies

### 1. Tenant Scoping Policy

Prevents cross-tenant access in multi-tenant systems.

```rust
use riptide_facade::authorization::policies::TenantScopingPolicy;

// Basic tenant scoping
let policy = TenantScopingPolicy::new();

// With admin override
let policy = TenantScopingPolicy::with_admin_override();
```

**Features**:
- Checks tenant_id on resources against user's tenant
- Optional admin override for system administrators
- Critical for multi-tenancy security

### 2. RBAC Policy

Role-based access control with flexible role requirements.

```rust
use riptide_facade::authorization::policies::RbacPolicy;

// Create custom RBAC policy
let mut policy = RbacPolicy::new();
policy.require_role_for_resource("url", vec!["viewer", "editor"]);
policy.require_role_for_resource("extraction", vec!["editor", "admin"]);

// Or use defaults
let policy = RbacPolicy::with_defaults();
```

**Default Role Requirements**:
- `url`: viewer, editor, admin
- `extraction`: editor, admin
- `pipeline`: editor, admin
- `profile`: admin
- `session`: viewer, editor, admin

### 3. Resource Ownership Policy

Restricts access to resource owners only.

```rust
use riptide_facade::authorization::policies::ResourceOwnershipPolicy;

let mut policy = ResourceOwnershipPolicy::new();
policy.register_owner("resource_id", "user123");

// Check ownership
assert!(policy.is_owner("resource_id", "user123"));

// With admin override
let policy = ResourceOwnershipPolicy::with_admin_override();
```

## Policy Composition

Combine multiple policies using PolicyChain:

```rust
use riptide_facade::authorization::PolicyChain;
use riptide_facade::authorization::policies::*;

let mut rbac = RbacPolicy::with_defaults();
let ownership = ResourceOwnershipPolicy::with_admin_override();

let chain = PolicyChain::new()
    .add_policy(Box::new(TenantScopingPolicy::new()))
    .add_policy(Box::new(rbac))
    .add_policy(Box::new(ownership));

// All policies must pass
chain.authorize(&ctx, &resource)?;
```

## Resource Types

The framework supports multiple resource types:

```rust
use riptide_facade::authorization::Resource;

// Simple URL resource
let resource = Resource::Url("https://example.com".to_string());

// Extraction with tenant scoping
let resource = Resource::Extraction {
    url: "https://example.com".to_string(),
    tenant_id: "tenant1".to_string(),
};

// Browser profile
let resource = Resource::Profile("profile123".to_string());

// Browser session
let resource = Resource::Session("session456".to_string());

// Pipeline execution
let resource = Resource::Pipeline {
    pipeline_id: "pipe789".to_string(),
    tenant_id: "tenant1".to_string(),
};

// Custom resource
let resource = Resource::Custom {
    resource_type: "document".to_string(),
    resource_id: "doc123".to_string(),
};
```

## Authorization Context

The AuthorizationContext carries all information needed for authorization decisions:

```rust
use riptide_facade::authorization::AuthorizationContext;
use std::collections::HashSet;

let ctx = AuthorizationContext::new(
    "user123",                    // user_id
    "tenant1",                    // tenant_id
    vec!["editor", "reviewer"],   // roles
    HashSet::from([
        "read:urls".to_string(),
        "write:extractions".to_string(),
    ]),                          // permissions
);

// Check roles
assert!(ctx.has_role("editor"));
assert!(ctx.has_any_role(&["admin", "editor"]));

// Check permissions
assert!(ctx.has_permission("read:urls"));
assert!(ctx.has_all_permissions(&["read:urls", "write:extractions"]));
```

## Testing

The framework includes comprehensive tests:

```bash
# Run all authorization tests
cargo test authorization

# Run integration tests
cargo test --test authorization_integration_test

# Run specific policy tests
cargo test tenant_scoping
cargo test rbac
cargo test ownership
```

## Quality Gates

All code must pass:

- ✅ Compilation without errors
- ✅ Zero clippy warnings (`cargo clippy -- -D warnings`)
- ✅ >90% test coverage
- ✅ Comprehensive documentation

## Examples

### Multi-Tenant SaaS Application

```rust
// Each tenant isolated by TenantScopingPolicy
let tenant_policy = TenantScopingPolicy::new();

// Different roles have different permissions
let mut rbac = RbacPolicy::with_defaults();

// Combine for complete authorization
let policies = vec![
    Arc::new(tenant_policy),
    Arc::new(rbac),
];

// User from tenant1 cannot access tenant2 resources
let ctx = AuthorizationContext::new("user1", "tenant1", vec!["editor"], HashSet::new());
let resource = Resource::Extraction {
    url: "https://example.com".to_string(),
    tenant_id: "tenant2".to_string(),
};

// This will fail with PermissionDenied
assert!(authorize(&ctx, &resource, &policies).is_err());
```

### Resource Ownership

```rust
let mut ownership = ResourceOwnershipPolicy::new();

// Register ownership
ownership.register_owner("document1", "user123");
ownership.register_owner("document2", "user456");

// Only owner can access
let ctx = AuthorizationContext::new("user123", "tenant1", vec!["viewer"], HashSet::new());
let resource = Resource::Custom {
    resource_type: "document".to_string(),
    resource_id: "document1".to_string(),
};

assert!(ownership.authorize(&ctx, &resource).is_ok());

// Different user cannot access
let ctx = AuthorizationContext::new("user456", "tenant1", vec!["viewer"], HashSet::new());
assert!(ownership.authorize(&ctx, &resource).is_err());
```

## Implementation Notes

### Architectural Compliance

- ✅ No HTTP types (authorization is protocol-agnostic)
- ✅ No database types (policies are in-memory, persistence is infrastructure)
- ✅ No infrastructure dependencies
- ✅ Uses port traits for extensibility
- ✅ Dependency injection for policy composition

### Performance Considerations

- Policies are lightweight and execute in microseconds
- No database lookups during authorization (use caching at infrastructure layer)
- Policy chains short-circuit on first failure
- Thread-safe with `Send + Sync` bounds

### Security Best Practices

1. **Always check authorization BEFORE business logic**
2. **Use multiple policies for defense in depth**
3. **Default deny (no policies = no access for sensitive resources)**
4. **Log authorization failures for security monitoring**
5. **Admin overrides should be used sparingly and logged**

## Future Enhancements

Potential extensions (not in current scope):

- Attribute-Based Access Control (ABAC)
- Time-based access restrictions
- Rate limiting per user/tenant
- Audit logging integration
- Dynamic policy loading from database
- Permission inheritance hierarchies

## References

- [Hexagonal Architecture](../../ARCHITECTURE.md)
- [Phase 2 Refactoring Roadmap](../../REFACTORING_ROADMAP.md)
- [riptide-types Error Handling](../riptide-types/src/error/README.md)

## License

This authorization framework is part of the Riptide project and follows the same license.
