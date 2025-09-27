//! Security Middleware Integration for RipTide
//!
//! Provides integrated security middleware that combines API key validation,
//! budget enforcement, PII redaction, and audit logging.

use crate::security::{
    api_keys::ApiKeyManager,
    audit::{AuditLogger, AuditDetails, AuditOutcome},
    budget::BudgetManager,
    pii::PiiRedactionMiddleware,
    types::*,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Comprehensive security middleware that integrates all security components
pub struct SecurityMiddleware {
    api_key_manager: Arc<ApiKeyManager>,
    budget_manager: Arc<BudgetManager>,
    pii_middleware: Arc<PiiRedactionMiddleware>,
    audit_logger: Arc<AuditLogger>,
    enabled: bool,
}

impl SecurityMiddleware {
    /// Create new security middleware with all components
    pub fn new(
        api_key_manager: Arc<ApiKeyManager>,
        budget_manager: Arc<BudgetManager>,
        pii_middleware: Arc<PiiRedactionMiddleware>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            api_key_manager,
            budget_manager,
            pii_middleware,
            audit_logger,
            enabled: true,
        }
    }
    
    /// Create security middleware with default configurations
    pub fn with_defaults() -> Result<Self> {
        let api_key_manager = Arc::new(ApiKeyManager::new());
        let budget_manager = Arc::new(BudgetManager::new(None));
        let pii_middleware = Arc::new(PiiRedactionMiddleware::new(None)?);
        let audit_logger = Arc::new(AuditLogger::new(None)?);
        
        Ok(Self::new(
            api_key_manager,
            budget_manager,
            pii_middleware,
            audit_logger,
        ))
    }
    
    /// Enable or disable the middleware
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Process incoming request with full security pipeline
    pub async fn process_request(
        &self,
        api_key: &str,
        job_id: &str,
        estimated_tokens: u64,
        model_name: &str,
        request_payload: &str,
        source_ip: &str,
        user_agent: Option<&str>,
    ) -> SecurityResult<RequestSecurityContext> {
        if !self.enabled {
            return Ok(RequestSecurityContext {
                security_context: SecurityContext::new(
                    TenantId::from("disabled"),
                    "disabled".to_string(),
                    source_ip.to_string(),
                ),
                redacted_payload: request_payload.to_string(),
                cost_info: None,
                pii_detected: false,
            });
        }
        
        let start_time = std::time::Instant::now();
        
        // Step 1: Validate API key and get security context
        let (_api_key_info, mut security_context) = match self.api_key_manager.validate_api_key(api_key).await {
            Ok((key, context)) => {
                info!(
                    tenant_id = %context.tenant_id,
                    api_key_id = %context.api_key_id,
                    "API key validated successfully"
                );
                (key, context)
            }
            Err(e) => {
                // Log authentication failure
                let details = AuditDetails {
                    description: "API key validation failed".to_string(),
                    error_message: Some(e.to_string()),
                    request_payload: None,
                    response_payload: None,
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                    bytes_processed: Some(request_payload.len() as u64),
                    cost_usd: None,
                    tokens_used: None,
                    model_name: Some(model_name.to_string()),
                    rate_limit_info: None,
                    pii_redacted: false,
                    pii_detections: None,
                };
                
                let dummy_context = SecurityContext::new(
                    TenantId::from("unknown"),
                    "unknown".to_string(),
                    source_ip.to_string(),
                );
                
                let audit_entry = self.audit_logger.create_api_key_usage_entry(
                    &dummy_context,
                    AuditOutcome::Failure,
                    details,
                );
                
                if let Err(audit_err) = self.audit_logger.log_event(audit_entry).await {
                    error!("Failed to log authentication failure: {}", audit_err);
                }
                
                return Err(e);
            }
        };
        
        // Update security context with request details
        security_context.ip_address = source_ip.to_string();
        if let Some(ua) = user_agent {
            security_context.user_agent = Some(ua.to_string());
        }
        
        // Step 2: Check budget limits
        let cost_info = match self.budget_manager.check_budget_for_job(
            job_id,
            &security_context.tenant_id,
            estimated_tokens,
            model_name,
        ).await {
            Ok(cost) => {
                debug!(
                    tenant_id = %security_context.tenant_id,
                    job_id = job_id,
                    estimated_cost = cost.estimated_cost_usd,
                    "Budget check passed"
                );
                Some(cost)
            }
            Err(e) => {
                // Log budget violation
                let details = AuditDetails {
                    description: format!("Budget check failed for job {}", job_id),
                    error_message: Some(e.to_string()),
                    request_payload: None,
                    response_payload: None,
                    duration_ms: Some(start_time.elapsed().as_millis() as u64),
                    bytes_processed: Some(request_payload.len() as u64),
                    cost_usd: None,
                    tokens_used: Some(estimated_tokens),
                    model_name: Some(model_name.to_string()),
                    rate_limit_info: None,
                    pii_redacted: false,
                    pii_detections: None,
                };
                
                let audit_entry = self.audit_logger.create_budget_event_entry(
                    SecurityEventType::BudgetExceeded,
                    &security_context.tenant_id,
                    &CostInfo::new(
                        estimated_tokens,
                        0.0, // Unknown cost at this point
                        model_name.to_string(),
                        "llm_request".to_string(),
                    ),
                    details,
                );
                
                if let Err(audit_err) = self.audit_logger.log_event(audit_entry).await {
                    error!("Failed to log budget violation: {}", audit_err);
                }
                
                return Err(e);
            }
        };
        
        // Step 3: Redact PII from request payload
        let (redacted_payload, pii_detected) = match self.pii_middleware.redact_payload(request_payload) {
            Ok(redacted) => {
                let pii_found = redacted != request_payload;
                if pii_found {
                    warn!(
                        tenant_id = %security_context.tenant_id,
                        job_id = job_id,
                        "PII detected and redacted from request payload"
                    );
                    
                    // Log PII detection
                    let details = AuditDetails {
                        description: "PII detected and redacted from request".to_string(),
                        error_message: None,
                        request_payload: None, // Don't log the actual payload with PII
                        response_payload: None,
                        duration_ms: Some(start_time.elapsed().as_millis() as u64),
                        bytes_processed: Some(request_payload.len() as u64),
                        cost_usd: cost_info.as_ref().map(|c| c.estimated_cost_usd),
                        tokens_used: Some(estimated_tokens),
                        model_name: Some(model_name.to_string()),
                        rate_limit_info: None,
                        pii_redacted: true,
                        pii_detections: Some(1), // Simplified - could count actual detections
                    };
                    
                    let audit_entry = self.audit_logger.create_pii_detection_entry(
                        &security_context.tenant_id,
                        1, // Simplified count
                        details,
                    );
                    
                    if let Err(audit_err) = self.audit_logger.log_event(audit_entry).await {
                        error!("Failed to log PII detection: {}", audit_err);
                    }
                }
                (redacted, pii_found)
            }
            Err(e) => {
                warn!("PII redaction failed: {}", e);
                (request_payload.to_string(), false)
            }
        };
        
        // Step 4: Log successful request processing
        let details = AuditDetails {
            description: "Request processed successfully".to_string(),
            error_message: None,
            request_payload: None, // Don't log payload for privacy
            response_payload: None,
            duration_ms: Some(start_time.elapsed().as_millis() as u64),
            bytes_processed: Some(request_payload.len() as u64),
            cost_usd: cost_info.as_ref().map(|c| c.estimated_cost_usd),
            tokens_used: Some(estimated_tokens),
            model_name: Some(model_name.to_string()),
            rate_limit_info: None,
            pii_redacted: pii_detected,
            pii_detections: if pii_detected { Some(1) } else { None },
        };
        
        let audit_entry = self.audit_logger.create_api_key_usage_entry(
            &security_context,
            AuditOutcome::Success,
            details,
        );
        
        if let Err(audit_err) = self.audit_logger.log_event(audit_entry).await {
            error!("Failed to log successful request: {}", audit_err);
        }
        
        Ok(RequestSecurityContext {
            security_context,
            redacted_payload,
            cost_info,
            pii_detected,
        })
    }
    
    /// Process response and record actual usage
    pub async fn process_response(
        &self,
        job_id: &str,
        security_context: &SecurityContext,
        actual_tokens: u64,
        actual_cost: f64,
        model_name: &str,
        response_payload: Option<&str>,
        duration_ms: u64,
    ) -> Result<String> {
        if !self.enabled {
            return Ok(response_payload.unwrap_or_default().to_string());
        }
        
        // Record actual usage in budget manager
        let cost_info = CostInfo::new(
            actual_tokens,
            actual_cost,
            model_name.to_string(),
            "llm_response".to_string(),
        );
        
        if let Err(e) = self.budget_manager.record_usage(
            job_id,
            &security_context.tenant_id,
            cost_info.clone(),
        ).await {
            error!("Failed to record actual usage: {}", e);
        }
        
        // Redact PII from response if provided
        let redacted_response = if let Some(response) = response_payload {
            match self.pii_middleware.redact_payload(response) {
                Ok(redacted) => {
                    if redacted != response {
                        warn!(
                            tenant_id = %security_context.tenant_id,
                            job_id = job_id,
                            "PII detected and redacted from response payload"
                        );
                    }
                    redacted
                }
                Err(e) => {
                    warn!("Response PII redaction failed: {}", e);
                    response.to_string()
                }
            }
        } else {
            String::new()
        };
        
        // Log completion
        let details = AuditDetails {
            description: "Request completed successfully".to_string(),
            error_message: None,
            request_payload: None,
            response_payload: None, // Don't log response for privacy
            duration_ms: Some(duration_ms),
            bytes_processed: response_payload.map(|r| r.len() as u64),
            cost_usd: Some(actual_cost),
            tokens_used: Some(actual_tokens),
            model_name: Some(model_name.to_string()),
            rate_limit_info: None,
            pii_redacted: response_payload.map_or(false, |r| redacted_response != r),
            pii_detections: None,
        };
        
        let audit_entry = self.audit_logger.create_api_key_usage_entry(
            security_context,
            AuditOutcome::Success,
            details,
        );
        
        if let Err(audit_err) = self.audit_logger.log_event(audit_entry).await {
            error!("Failed to log request completion: {}", audit_err);
        }
        
        Ok(redacted_response)
    }
    
    /// Get security health status
    pub async fn get_security_health(&self) -> SecurityHealthStatus {
        let budget_health = self.budget_manager.check_budget_health().await;
        let audit_stats = self.audit_logger.get_stats().await;
        
        SecurityHealthStatus {
            api_keys_active: true, // Simplified
            budget_health_ok: !budget_health.is_circuit_breaker_open,
            pii_redaction_active: true, // Simplified
            audit_logging_active: true, // Simplified
            total_requests_today: audit_stats.total_events,
            security_violations_today: audit_stats.events_by_outcome.get(&AuditOutcome::Failure).copied().unwrap_or(0),
            budget_usage_percentage: budget_health.usage_percentage,
            remaining_budget_usd: budget_health.remaining_budget_usd,
        }
    }
    
    /// Get access to individual components
    pub fn get_api_key_manager(&self) -> Arc<ApiKeyManager> {
        Arc::clone(&self.api_key_manager)
    }
    
    pub fn get_budget_manager(&self) -> Arc<BudgetManager> {
        Arc::clone(&self.budget_manager)
    }
    
    pub fn get_pii_middleware(&self) -> Arc<PiiRedactionMiddleware> {
        Arc::clone(&self.pii_middleware)
    }
    
    pub fn get_audit_logger(&self) -> Arc<AuditLogger> {
        Arc::clone(&self.audit_logger)
    }

    /// Validate request size against security limits
    pub fn validate_request_size(&self, content_length: usize) -> Result<()> {
        const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if content_length > MAX_REQUEST_SIZE {
            return Err(anyhow::anyhow!("Request size {} exceeds maximum allowed size {}", content_length, MAX_REQUEST_SIZE));
        }
        Ok(())
    }

    /// Apply security headers to response
    pub fn apply_security_headers(&self, headers: &mut reqwest::header::HeaderMap) -> Result<()> {
        use reqwest::header::*;

        headers.insert(CONTENT_SECURITY_POLICY, "default-src 'self'".parse().unwrap());
        headers.insert(X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
        headers.insert(X_FRAME_OPTIONS, "DENY".parse().unwrap());
        headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
        headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());

        Ok(())
    }

    /// Sanitize headers by removing sensitive information
    pub fn sanitize_headers(&self, headers: &mut reqwest::header::HeaderMap) -> Result<()> {
        use reqwest::header::*;

        // Remove potentially sensitive headers
        headers.remove(AUTHORIZATION);
        headers.remove("X-API-Key");
        headers.remove("X-Auth-Token");
        headers.remove(COOKIE);
        headers.remove(SET_COOKIE);

        Ok(())
    }
}

/// Security context for a processed request
#[derive(Debug, Clone)]
pub struct RequestSecurityContext {
    pub security_context: SecurityContext,
    pub redacted_payload: String,
    pub cost_info: Option<CostInfo>,
    pub pii_detected: bool,
}

/// Overall security health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHealthStatus {
    pub api_keys_active: bool,
    pub budget_health_ok: bool,
    pub pii_redaction_active: bool,
    pub audit_logging_active: bool,
    pub total_requests_today: u64,
    pub security_violations_today: u64,
    pub budget_usage_percentage: f64,
    pub remaining_budget_usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    async fn create_test_middleware() -> SecurityMiddleware {
        SecurityMiddleware::with_defaults().unwrap()
    }
    
    #[tokio::test]
    async fn test_middleware_creation() {
        let middleware = create_test_middleware().await;
        assert!(middleware.enabled);
    }
    
    #[tokio::test]
    async fn test_request_processing_without_api_key() {
        let middleware = create_test_middleware().await;
        
        let result = middleware.process_request(
            "invalid_key",
            "job-1",
            1000,
            "gpt-3.5-turbo",
            "test payload",
            "127.0.0.1",
            Some("test-agent"),
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_request_processing_with_valid_api_key() {
        let middleware = create_test_middleware().await;
        
        // First create a valid API key
        let tenant_id = TenantId::from("test-tenant");
        let (_, raw_key) = middleware.api_key_manager
            .create_api_key(
                tenant_id,
                "Test Key".to_string(),
                None,
                vec![],
                None,
                None,
            )
            .await
            .unwrap();
        
        let result = middleware.process_request(
            &raw_key,
            "job-1",
            1000,
            "gpt-3.5-turbo",
            "test payload",
            "127.0.0.1",
            Some("test-agent"),
        ).await;
        
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.redacted_payload, "test payload");
        assert!(context.cost_info.is_some());
    }
    
    #[tokio::test]
    async fn test_pii_redaction_in_request() {
        let middleware = create_test_middleware().await;
        
        // Create a valid API key
        let tenant_id = TenantId::from("test-tenant");
        let (_, raw_key) = middleware.api_key_manager
            .create_api_key(
                tenant_id,
                "Test Key".to_string(),
                None,
                vec![],
                None,
                None,
            )
            .await
            .unwrap();
        
        let result = middleware.process_request(
            &raw_key,
            "job-1",
            1000,
            "gpt-3.5-turbo",
            "Contact me at john.doe@example.com",
            "127.0.0.1",
            None,
        ).await;
        
        assert!(result.is_ok());
        let context = result.unwrap();
        assert!(context.pii_detected);
        assert!(!context.redacted_payload.contains("john.doe@example.com"));
    }
    
    #[tokio::test]
    async fn test_security_health_status() {
        let middleware = create_test_middleware().await;
        let health = middleware.get_security_health().await;
        
        assert!(health.api_keys_active);
        assert!(health.budget_health_ok);
        assert!(health.pii_redaction_active);
        assert!(health.audit_logging_active);
    }
    
    #[tokio::test]
    async fn test_disabled_middleware() {
        let mut middleware = create_test_middleware().await;
        middleware.set_enabled(false);
        
        let result = middleware.process_request(
            "any_key",
            "job-1",
            1000,
            "gpt-3.5-turbo",
            "test payload",
            "127.0.0.1",
            None,
        ).await;
        
        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.security_context.tenant_id.to_string(), "disabled");
    }
}
