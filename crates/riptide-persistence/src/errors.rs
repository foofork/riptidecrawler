use riptide_types::error::RiptideError;
use thiserror::Error;

/// Result type for persistence operations
pub type PersistenceResult<T> = Result<T, PersistenceError>;

/// Comprehensive error types for the persistence layer
#[derive(Error, Debug)]
pub enum PersistenceError {
    /// Redis connection or operation errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Compression/decompression errors
    #[error("Compression error: {0}")]
    Compression(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Cache-specific errors
    #[error("Cache error: {0}")]
    Cache(String),

    /// State management errors
    #[error("State error: {0}")]
    State(String),

    /// Tenant management errors
    #[error("Tenant error: {0}")]
    Tenant(String),

    /// Synchronization errors
    #[error("Sync error: {0}")]
    Sync(String),

    /// Distributed coordination errors
    #[error("Coordination error: {0}")]
    Coordination(String),

    /// Performance threshold violations
    #[error("Performance error: {0}")]
    Performance(String),

    /// Security boundary violations
    #[error("Security error: {0}")]
    Security(String),

    /// Resource quota exceeded
    #[error("Quota exceeded: {resource} limit {limit} exceeded with usage {current}")]
    QuotaExceeded {
        resource: String,
        limit: u64,
        current: u64,
    },

    /// Timeout errors
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Invalid tenant access
    #[error("Invalid tenant access: tenant {tenant_id} not found or access denied")]
    InvalidTenantAccess { tenant_id: String },

    /// Data integrity errors
    #[error("Data integrity error: {0}")]
    DataIntegrity(String),

    /// File system errors (for checkpoints, config files)
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// Watch errors (for hot reload)
    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),

    /// Prometheus errors
    #[error("Metrics error: {0}")]
    Metrics(String),

    /// Generic errors
    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),

    /// RiptideError from riptide-types (for CacheStorage trait compatibility)
    #[error("Riptide error: {0}")]
    Riptide(String),
}

impl From<RiptideError> for PersistenceError {
    fn from(err: RiptideError) -> Self {
        PersistenceError::Riptide(err.to_string())
    }
}

impl PersistenceError {
    /// Create a new cache error
    pub fn cache(msg: impl Into<String>) -> Self {
        Self::Cache(msg.into())
    }

    /// Create a new state error
    pub fn state(msg: impl Into<String>) -> Self {
        Self::State(msg.into())
    }

    /// Create a new tenant error
    pub fn tenant(msg: impl Into<String>) -> Self {
        Self::Tenant(msg.into())
    }

    /// Create a new sync error
    pub fn sync(msg: impl Into<String>) -> Self {
        Self::Sync(msg.into())
    }

    /// Create a new coordination error
    pub fn coordination(msg: impl Into<String>) -> Self {
        Self::Coordination(msg.into())
    }

    /// Create a new performance error
    pub fn performance(msg: impl Into<String>) -> Self {
        Self::Performance(msg.into())
    }

    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a new compression error
    pub fn compression(msg: impl Into<String>) -> Self {
        Self::Compression(msg.into())
    }

    /// Create a new timeout error
    pub fn timeout(timeout_ms: u64) -> Self {
        Self::Timeout { timeout_ms }
    }

    /// Create a new quota exceeded error
    pub fn quota_exceeded(resource: impl Into<String>, limit: u64, current: u64) -> Self {
        Self::QuotaExceeded {
            resource: resource.into(),
            limit,
            current,
        }
    }

    /// Create a new invalid tenant access error
    pub fn invalid_tenant_access(tenant_id: impl Into<String>) -> Self {
        Self::InvalidTenantAccess {
            tenant_id: tenant_id.into(),
        }
    }

    /// Create a new data integrity error
    pub fn data_integrity(msg: impl Into<String>) -> Self {
        Self::DataIntegrity(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            PersistenceError::Redis(_) => true,
            PersistenceError::Timeout { .. } => true,
            PersistenceError::Sync(_) => true,
            PersistenceError::Coordination(_) => true,
            PersistenceError::Performance(_) => false,
            PersistenceError::Security(_) => false,
            PersistenceError::QuotaExceeded { .. } => false,
            PersistenceError::InvalidTenantAccess { .. } => false,
            PersistenceError::Configuration(_) => false,
            PersistenceError::DataIntegrity(_) => false,
            _ => false,
        }
    }

    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            PersistenceError::Redis(_) => "redis",
            PersistenceError::Serialization(_) => "serialization",
            PersistenceError::Compression(_) => "compression",
            PersistenceError::Configuration(_) => "configuration",
            PersistenceError::Cache(_) => "cache",
            PersistenceError::State(_) => "state",
            PersistenceError::Tenant(_) => "tenant",
            PersistenceError::Sync(_) => "sync",
            PersistenceError::Coordination(_) => "coordination",
            PersistenceError::Performance(_) => "performance",
            PersistenceError::Security(_) => "security",
            PersistenceError::QuotaExceeded { .. } => "quota",
            PersistenceError::Timeout { .. } => "timeout",
            PersistenceError::InvalidTenantAccess { .. } => "tenant_access",
            PersistenceError::DataIntegrity(_) => "data_integrity",
            PersistenceError::FileSystem(_) => "filesystem",
            PersistenceError::Watch(_) => "watch",
            PersistenceError::Metrics(_) => "metrics",
            PersistenceError::Generic(_) => "generic",
            PersistenceError::Riptide(_) => "riptide",
        }
    }
}
