//! Authorization-enhanced extraction facade
//!
//! This module extends UrlExtractionFacade with authorization capabilities.
//! It demonstrates how to integrate authorization policies into facade methods.

use crate::authorization::{AuthorizationContext, AuthorizationPolicy, Resource};
use crate::facades::extraction::{ExtractedDoc, Result, UrlExtractionFacade, UrlExtractionOptions};
use std::sync::Arc;

/// Extension trait for UrlExtractionFacade to add authorization methods.
///
/// This follows the Extension Object pattern to add authorization without
/// modifying the existing facade structure.
#[allow(async_fn_in_trait)]
pub trait AuthorizedExtractionFacade {
    /// Extract content from URL with authorization checks.
    ///
    /// This is the authorized version of `extract_from_url` that performs
    /// authorization before extraction.
    ///
    /// # Arguments
    ///
    /// * `url` - URL to extract content from
    /// * `options` - Extraction options
    /// * `authz_ctx` - Authorization context with user identity and permissions
    /// * `policies` - Authorization policies to check
    ///
    /// # Returns
    ///
    /// * `Ok(ExtractedDoc)` if authorized and extraction succeeds
    /// * `Err(RiptideError::PermissionDenied)` if not authorized
    /// * `Err(RiptideError::*)` for other errors
    async fn extract_with_authorization(
        &self,
        url: &str,
        options: UrlExtractionOptions,
        authz_ctx: &AuthorizationContext,
        policies: &[Arc<dyn AuthorizationPolicy>],
    ) -> Result<ExtractedDoc>;

    /// Extract from HTML with authorization checks.
    async fn extract_html_with_authorization(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
        authz_ctx: &AuthorizationContext,
        policies: &[Arc<dyn AuthorizationPolicy>],
    ) -> Result<ExtractedDoc>;
}

impl AuthorizedExtractionFacade for UrlExtractionFacade {
    async fn extract_with_authorization(
        &self,
        url: &str,
        options: UrlExtractionOptions,
        authz_ctx: &AuthorizationContext,
        policies: &[Arc<dyn AuthorizationPolicy>],
    ) -> Result<ExtractedDoc> {
        // 1. Authorization checks - BEFORE business logic
        authorize_url_access(url, authz_ctx, policies)?;

        tracing::info!(
            user_id = %authz_ctx.user_id,
            tenant_id = %authz_ctx.tenant_id,
            url = %url,
            "Authorization passed for URL extraction"
        );

        // 2. Proceed with business logic
        self.extract_from_url(url, options).await
    }

    async fn extract_html_with_authorization(
        &self,
        url: &str,
        html: &str,
        options: UrlExtractionOptions,
        authz_ctx: &AuthorizationContext,
        policies: &[Arc<dyn AuthorizationPolicy>],
    ) -> Result<ExtractedDoc> {
        // 1. Authorization checks
        authorize_url_access(url, authz_ctx, policies)?;

        tracing::info!(
            user_id = %authz_ctx.user_id,
            tenant_id = %authz_ctx.tenant_id,
            url = %url,
            "Authorization passed for HTML extraction"
        );

        // 2. Proceed with business logic
        self.extract_from_html(url, html, options).await
    }
}

/// Helper function to authorize URL access.
///
/// This applies all authorization policies in sequence.
/// All policies must pass for authorization to succeed.
fn authorize_url_access(
    url: &str,
    authz_ctx: &AuthorizationContext,
    policies: &[Arc<dyn AuthorizationPolicy>],
) -> Result<()> {
    let resource = Resource::Url(url.to_string());

    for policy in policies {
        policy
            .authorize(authz_ctx, &resource)
            .map_err(|e| crate::error::RiptideError::validation(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization::policies::{RbacPolicy, TenantScopingPolicy};
    use riptide_extraction::fallback_extract;
    use riptide_extraction::ContentExtractor;
    use riptide_types::ExtractedContent;
    use std::collections::HashSet;

    // Mock extractor for testing
    struct MockExtractor;

    #[async_trait::async_trait]
    impl ContentExtractor for MockExtractor {
        async fn extract(&self, html: &str, url: &str) -> anyhow::Result<ExtractedContent> {
            fallback_extract(html, url).await
        }

        fn confidence_score(&self, _html: &str) -> f64 {
            0.8
        }

        fn strategy_name(&self) -> &'static str {
            "mock"
        }
    }

    fn create_test_facade() -> UrlExtractionFacade {
        let http_client = Arc::new(reqwest::Client::new());
        let extractor: Arc<dyn ContentExtractor> = Arc::new(MockExtractor);

        UrlExtractionFacade::with_thresholds(
            http_client,
            extractor,
            0.7,
            0.3,
            std::time::Duration::from_secs(30),
        )
    }

    fn create_test_context(
        user_id: &str,
        tenant_id: &str,
        roles: Vec<&str>,
    ) -> AuthorizationContext {
        AuthorizationContext::new(user_id, tenant_id, roles, HashSet::new())
    }

    #[tokio::test]
    async fn test_authorized_extraction_success() {
        let facade = create_test_facade();

        // Setup authorization
        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["viewer", "editor"]);

        let policies: Vec<Arc<dyn AuthorizationPolicy>> =
            vec![Arc::new(TenantScopingPolicy::new()), Arc::new(rbac)];

        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let html = r#"
            <html>
                <head><title>Test Page</title></head>
                <body><p>Test content</p></body>
            </html>
        "#;

        let options = UrlExtractionOptions::default();

        let result = facade
            .extract_html_with_authorization("https://example.com", html, options, &ctx, &policies)
            .await;

        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.url, "https://example.com");
    }

    #[tokio::test]
    async fn test_authorized_extraction_denied() {
        let facade = create_test_facade();

        // Setup authorization - require admin role
        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["admin"]);

        let policies: Vec<Arc<dyn AuthorizationPolicy>> = vec![Arc::new(rbac)];

        // User only has viewer role
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let html = "<html><body>Test</body></html>";
        let options = UrlExtractionOptions::default();

        let result = facade
            .extract_html_with_authorization("https://example.com", html, options, &ctx, &policies)
            .await;

        // Should be denied
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Permission denied"));
    }

    #[tokio::test]
    async fn test_multiple_policies_all_pass() {
        let facade = create_test_facade();

        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["editor"]);

        let policies: Vec<Arc<dyn AuthorizationPolicy>> =
            vec![Arc::new(TenantScopingPolicy::new()), Arc::new(rbac)];

        let ctx = create_test_context("user123", "tenant1", vec!["editor"]);

        let html = "<html><body>Test</body></html>";
        let options = UrlExtractionOptions::default();

        let result = facade
            .extract_html_with_authorization("https://example.com", html, options, &ctx, &policies)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_policies_one_fails() {
        let facade = create_test_facade();

        // RBAC requires admin
        let mut rbac = RbacPolicy::new();
        rbac.require_role_for_resource("url", vec!["admin"]);

        let policies: Vec<Arc<dyn AuthorizationPolicy>> =
            vec![Arc::new(TenantScopingPolicy::new()), Arc::new(rbac)];

        // User only has editor role
        let ctx = create_test_context("user123", "tenant1", vec!["editor"]);

        let html = "<html><body>Test</body></html>";
        let options = UrlExtractionOptions::default();

        let result = facade
            .extract_html_with_authorization("https://example.com", html, options, &ctx, &policies)
            .await;

        // RBAC should fail
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authorization_with_extraction_resource() {
        let _facade = create_test_facade();

        // Test with an Extraction resource (has tenant_id)
        let ctx = create_test_context("user123", "tenant1", vec!["viewer"]);

        let tenant_policy = TenantScopingPolicy::new();

        // Same tenant - should pass
        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant1".to_string(),
        };

        assert!(tenant_policy.authorize(&ctx, &resource).is_ok());

        // Different tenant - should fail
        let resource = Resource::Extraction {
            url: "https://example.com".to_string(),
            tenant_id: "tenant2".to_string(),
        };

        assert!(tenant_policy.authorize(&ctx, &resource).is_err());
    }
}
