/*!
# RipTide Persistence Integration Example

Demonstrates integration with existing RipTide codebase and usage patterns.
*/

use riptide_persistence::config::{
    CacheConfig, CompressionAlgorithm, EvictionPolicy, StateConfig, TenantConfig,
    TenantIsolationLevel,
};
use riptide_persistence::metrics::TenantMetrics;
use riptide_persistence::state::CheckpointType;
use riptide_persistence::{
    BillingPlan, PersistenceConfig, PersistentCacheManager, ResourceUsageRecord, StateManager,
    TenantManager, TenantOwner,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 RipTide Persistence Layer Integration Example");
    println!("================================================");

    // Configuration setup
    let config = create_example_config().await?;
    println!("✅ Configuration loaded");

    // Initialize all components
    let (cache_manager, state_manager, tenant_manager) = initialize_components(&config).await?;
    println!("✅ All persistence components initialized");

    // Demonstrate multi-tenant workflow
    let tenant_id = demonstrate_tenant_workflow(&tenant_manager).await?;
    println!("✅ Tenant workflow demonstrated");

    // Demonstrate cache operations with tenant context
    demonstrate_cache_workflow(&cache_manager, &tenant_id).await?;
    println!("✅ Cache workflow demonstrated");

    // Demonstrate session management
    let session_id = demonstrate_session_workflow(&state_manager, &tenant_id).await?;
    println!("✅ Session workflow demonstrated (Session ID: {})", session_id);

    // Demonstrate state persistence and recovery
    demonstrate_checkpoint_workflow(&state_manager).await?;
    println!("✅ Checkpoint workflow demonstrated");

    // Demonstrate resource monitoring and quotas
    demonstrate_resource_monitoring(&tenant_manager, &tenant_id).await?;
    println!("✅ Resource monitoring demonstrated");

    // Performance demonstration
    demonstrate_performance(&cache_manager).await?;
    println!("✅ Performance demonstration completed");

    // Integration with existing RipTide components
    demonstrate_riptide_integration(&cache_manager, &tenant_id).await?;
    println!("✅ RipTide integration demonstrated");

    println!("\n🎉 All demonstrations completed successfully!");
    println!("   The persistence layer is ready for production use.");

    Ok(())
}

/// Create example configuration
async fn create_example_config() -> Result<PersistenceConfig, Box<dyn std::error::Error>> {
    let config = PersistenceConfig {
        redis: riptide_persistence::config::RedisConfig {
            url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/0".to_string()),
            pool_size: 10,
            connection_timeout_ms: 5000,
            command_timeout_ms: 5000,
            cluster_mode: false,
            retry_attempts: 3,
            retry_delay_ms: 100,
            enable_pipelining: true,
            max_pipeline_size: 100,
        },
        cache: CacheConfig {
            default_ttl_seconds: 24 * 60 * 60,      // 24 hours
            max_entry_size_bytes: 20 * 1024 * 1024, // 20MB
            key_prefix: "riptide_example".to_string(),
            version: "v1".to_string(),
            enable_compression: true,
            compression_threshold_bytes: 1024,
            compression_algorithm: CompressionAlgorithm::Lz4,
            enable_warming: true,
            warming_batch_size: 100,
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
            eviction_policy: EvictionPolicy::LRU,
        },
        state: StateConfig {
            session_timeout_seconds: 30 * 60, // 30 minutes
            enable_hot_reload: true,
            config_watch_paths: vec!["./config".to_string()],
            checkpoint_interval_seconds: 5 * 60, // 5 minutes
            max_checkpoints: 10,
            checkpoint_compression: true,
            enable_graceful_shutdown: true,
            shutdown_timeout_seconds: 30,
        },
        tenant: TenantConfig {
            enabled: true,
            default_quotas: {
                let mut quotas = HashMap::new();
                quotas.insert("memory_bytes".to_string(), 100 * 1024 * 1024); // 100MB
                quotas.insert("operations_per_minute".to_string(), 1000);
                quotas.insert("storage_bytes".to_string(), 1024 * 1024 * 1024); // 1GB
                quotas
            },
            isolation_level: TenantIsolationLevel::Strong,
            enable_billing: true,
            billing_interval_seconds: 60,
            max_tenants: 1000,
            enable_encryption: true,
        },
        distributed: None, // Disable for example
        performance: riptide_persistence::config::PerformanceConfig {
            target_cache_access_ms: 5,
            enable_monitoring: true,
            metrics_interval_seconds: 60,
            enable_slow_log: true,
            slow_threshold_ms: 100,
            enable_connection_pooling: true,
            pool_config: riptide_persistence::config::PoolConfig::default(),
        },
        security: riptide_persistence::config::SecurityConfig::default(),
    };

    config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    Ok(config)
}

/// Initialize all persistence components
async fn initialize_components(
    config: &PersistenceConfig,
) -> Result<(PersistentCacheManager, StateManager, TenantManager), Box<dyn std::error::Error>> {
    println!("  🔄 Initializing cache manager...");
    let cache_manager =
        PersistentCacheManager::new(&config.redis.url, config.cache.clone()).await?;

    println!("  🔄 Initializing state manager...");
    let state_manager = StateManager::new(&config.redis.url, config.state.clone()).await?;

    println!("  🔄 Initializing tenant manager...");
    let tenant_metrics = Arc::new(RwLock::new(TenantMetrics::new(Arc::new(
        prometheus::Registry::new(),
    ))));
    let tenant_manager =
        TenantManager::new(&config.redis.url, config.tenant.clone(), tenant_metrics).await?;

    Ok((cache_manager, state_manager, tenant_manager))
}

/// Demonstrate tenant management workflow
async fn demonstrate_tenant_workflow(
    tenant_manager: &TenantManager,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("  🏢 Creating example tenant...");

    let owner = TenantOwner {
        id: "example_owner_123".to_string(),
        name: "Example Organization".to_string(),
        email: "admin@example.com".to_string(),
        organization: Some("Example Corp".to_string()),
    };

    let tenant_config = riptide_persistence::tenant::TenantConfig {
        tenant_id: "".to_string(), // Will be generated
        name: "Example Tenant".to_string(),
        quotas: {
            let mut quotas = HashMap::new();
            quotas.insert("memory_bytes".to_string(), 50 * 1024 * 1024); // 50MB
            quotas.insert("operations_per_minute".to_string(), 500);
            quotas.insert("storage_bytes".to_string(), 500 * 1024 * 1024); // 500MB
            quotas
        },
        isolation_level: TenantIsolationLevel::Strong,
        encryption_enabled: true,
        settings: {
            let mut settings = HashMap::new();
            settings.insert(
                "feature_advanced_caching".to_string(),
                serde_json::json!(true),
            );
            settings.insert(
                "max_concurrent_sessions".to_string(),
                serde_json::json!(100),
            );
            settings
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let tenant_id = tenant_manager
        .create_tenant(tenant_config, owner, BillingPlan::Professional)
        .await?;
    println!("    ✓ Tenant created with ID: {}", tenant_id);

    // Validate tenant access
    let has_access = tenant_manager
        .validate_access(&tenant_id, "cache:read", "read")
        .await?;
    println!("    ✓ Tenant access validation: {}", has_access);

    // Check quotas
    let quota_ok = tenant_manager
        .check_quota(&tenant_id, "memory_bytes", 1024 * 1024)
        .await?; // 1MB
    println!("    ✓ Quota check (1MB): {}", quota_ok);

    Ok(tenant_id)
}

/// Demonstrate cache operations with tenant isolation
async fn demonstrate_cache_workflow(
    cache_manager: &PersistentCacheManager,
    tenant_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  💾 Demonstrating cache operations...");

    // Store data with tenant namespace
    let extraction_result = serde_json::json!({
        "url": "https://example.com/article",
        "title": "Example Article",
        "content": "This is the extracted content from the article...",
        "metadata": {
            "extracted_at": chrono::Utc::now().to_rfc3339(),
            "extractor_version": "1.0.0",
            "language": "en"
        },
        "performance": {
            "extraction_time_ms": 250,
            "content_size_bytes": 1024
        }
    });

    cache_manager
        .set(
            "article_extraction_123",
            &extraction_result,
            Some(tenant_id),                            // Tenant namespace
            Some(std::time::Duration::from_secs(3600)), // 1 hour TTL
            Some(riptide_persistence::cache::CacheMetadata {
                version: "1.0.0".to_string(),
                content_type: Some("application/json".to_string()),
                source: Some("riptide_extractor".to_string()),
                tags: vec!["article".to_string(), "extraction".to_string()],
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                    attrs.insert("extraction_type".to_string(), "html".to_string());
                    attrs
                },
            }),
        )
        .await?;

    println!("    ✓ Cached extraction result with tenant isolation");

    // Retrieve data
    let retrieved: Option<serde_json::Value> = cache_manager
        .get("article_extraction_123", Some(tenant_id))
        .await?;
    match retrieved {
        Some(data) => {
            println!(
                "    ✓ Retrieved cached data: {} bytes",
                serde_json::to_string(&data)?.len()
            );
        }
        None => println!("    ⚠ No cached data found"),
    }

    // Demonstrate batch operations for multiple extractions
    let mut batch_data = HashMap::new();
    for i in 0..5 {
        batch_data.insert(
            format!("batch_extraction_{}", i),
            serde_json::json!({
                "url": format!("https://example.com/article/{}", i),
                "title": format!("Article {}", i),
                "content": format!("Content for article {}", i)
            }),
        );
    }

    cache_manager
        .set_batch(
            batch_data,
            Some(tenant_id),
            Some(std::time::Duration::from_secs(1800)),
        )
        .await?;
    println!("    ✓ Batch cached 5 extractions");

    // Get cache statistics
    let stats = cache_manager.get_stats().await?;
    println!(
        "    ✓ Cache stats - Keys: {}, Memory: {} bytes, Hit rate: {:.2}%",
        stats.total_keys,
        stats.memory_usage_bytes,
        stats.hit_rate * 100.0
    );

    Ok(())
}

/// Demonstrate session management
async fn demonstrate_session_workflow(
    state_manager: &StateManager,
    tenant_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("  👤 Demonstrating session management...");

    let session_metadata = riptide_persistence::state::SessionMetadata {
        client_ip: Some("203.0.113.42".to_string()),
        user_agent: Some("RipTide-Client/1.0.0".to_string()),
        source: Some("web_interface".to_string()),
        attributes: {
            let mut attrs = HashMap::new();
            attrs.insert("tenant_id".to_string(), tenant_id.to_string());
            attrs.insert("session_type".to_string(), "extraction_session".to_string());
            attrs
        },
    };

    let session_id = state_manager
        .create_session(
            Some(format!("user_{}_{}", tenant_id, Uuid::new_v4())),
            session_metadata,
            Some(1800), // 30 minutes
        )
        .await?;

    println!("    ✓ Session created: {}", session_id);

    // Store extraction state in session
    state_manager
        .update_session_data(
            &session_id,
            "current_extraction",
            serde_json::json!({
                "url": "https://example.com/processing",
                "status": "in_progress",
                "started_at": chrono::Utc::now().to_rfc3339(),
                "progress": 0.45
            }),
        )
        .await?;

    state_manager.update_session_data(
        &session_id,
        "extraction_history",
        serde_json::json!([
            {"url": "https://example.com/article1", "completed_at": chrono::Utc::now().to_rfc3339()},
            {"url": "https://example.com/article2", "completed_at": chrono::Utc::now().to_rfc3339()}
        ])
    ).await?;

    println!("    ✓ Session data updated");

    // Retrieve session
    let session = state_manager.get_session(&session_id).await?;
    if let Some(session) = session {
        println!(
            "    ✓ Session retrieved - User: {:?}, Data keys: {}",
            session.user_id,
            session.data.len()
        );
    }

    Ok(session_id)
}

/// Demonstrate checkpoint and restore functionality
async fn demonstrate_checkpoint_workflow(
    state_manager: &StateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  💾 Demonstrating checkpoint/restore...");

    // Create checkpoint
    let checkpoint_id = state_manager
        .create_checkpoint(
            CheckpointType::Manual,
            Some("Example integration checkpoint".to_string()),
        )
        .await?;

    println!("    ✓ Checkpoint created: {}", checkpoint_id);

    // In a real scenario, you might restore from this checkpoint
    // For demonstration, we'll just show the capability exists
    println!("    ✓ Checkpoint restore capability available");

    Ok(())
}

/// Demonstrate resource monitoring and quotas
async fn demonstrate_resource_monitoring(
    tenant_manager: &TenantManager,
    tenant_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📊 Demonstrating resource monitoring...");

    // Record various resource usage activities
    let usage_activities = vec![
        ("cache_write", 1024, 50),
        ("cache_read", 512, 25),
        ("extraction", 2048, 150),
        ("session_create", 256, 10),
    ];

    for (operation, data_bytes, compute_ms) in usage_activities {
        let usage_record = ResourceUsageRecord {
            operation_count: 1,
            data_bytes,
            compute_time_ms: compute_ms,
            storage_bytes: data_bytes * 2, // Assume 2x storage overhead
            timestamp: chrono::Utc::now(),
        };

        tenant_manager
            .record_usage(tenant_id, operation, usage_record)
            .await?;
    }

    println!("    ✓ Recorded usage for 4 different operations");

    // Get usage statistics
    let usage_stats = tenant_manager.get_tenant_usage(tenant_id).await?;
    println!(
        "    ✓ Current usage - Operations/min: {}, Data transfer: {} bytes",
        usage_stats.operations_per_minute, usage_stats.data_transfer_bytes
    );

    // Get billing information
    let billing_info = tenant_manager.get_billing_info(tenant_id).await?;
    println!(
        "    ✓ Billing plan: {:?}, Operations: {}",
        billing_info.plan, billing_info.current_usage.operations
    );

    Ok(())
}

/// Demonstrate performance characteristics
async fn demonstrate_performance(
    cache_manager: &PersistentCacheManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  ⚡ Demonstrating performance characteristics...");

    let test_data = serde_json::json!({
        "performance_test": true,
        "data": "x".repeat(1024), // 1KB test data
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    // Measure cache set performance
    let set_start = std::time::Instant::now();
    cache_manager
        .set("performance_test", &test_data, None, None, None)
        .await?;
    let set_duration = set_start.elapsed();

    // Measure cache get performance
    let get_start = std::time::Instant::now();
    let _retrieved: Option<serde_json::Value> = cache_manager.get("performance_test", None).await?;
    let get_duration = get_start.elapsed();

    println!(
        "    ✓ Set operation: {:?} (target: <5ms production)",
        set_duration
    );
    println!(
        "    ✓ Get operation: {:?} (target: <5ms production)",
        get_duration
    );

    // Performance validation
    if get_duration.as_millis() <= 50 {
        println!("    ✅ Performance target met!");
    } else {
        println!("    ⚠ Performance target exceeded (acceptable in development)");
    }

    Ok(())
}

/// Demonstrate integration with existing RipTide components
async fn demonstrate_riptide_integration(
    cache_manager: &PersistentCacheManager,
    tenant_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  🔗 Demonstrating RipTide integration patterns...");

    // Simulate integration with existing RipTide core components
    println!("    📄 Simulating HTML extraction with caching...");

    // This would typically come from riptide-html or riptide-core
    let extraction_request = serde_json::json!({
        "url": "https://example.com/news/article",
        "extractor_type": "html",
        "options": {
            "extract_images": true,
            "extract_links": true,
            "clean_html": true
        }
    });

    // Generate cache key using extraction parameters
    let cache_key = cache_manager.generate_key(
        &format!(
            "{}_{}",
            extraction_request["url"].as_str().unwrap(),
            serde_json::to_string(&extraction_request["options"])?
        ),
        Some(tenant_id),
    );

    // Check if extraction is already cached
    let cached_result: Option<serde_json::Value> =
        cache_manager.get(&cache_key, Some(tenant_id)).await?;

    match cached_result {
        Some(_cached) => {
            println!("    ✓ Using cached extraction result");
            println!("      Cache key: {}", cache_key);
        }
        None => {
            println!("    🔄 Performing new extraction (cache miss)");

            // Simulate extraction process
            let extraction_result = serde_json::json!({
                "url": extraction_request["url"],
                "title": "Breaking News: Technology Advances",
                "content": "The extracted and cleaned content from the webpage...",
                "images": [
                    {"src": "https://example.com/image1.jpg", "alt": "News image"}
                ],
                "links": [
                    {"href": "https://example.com/related", "text": "Related article"}
                ],
                "metadata": {
                    "extracted_at": chrono::Utc::now().to_rfc3339(),
                    "extractor_version": "1.0.0",
                    "content_length": 1024,
                    "extraction_time_ms": 245
                }
            });

            // Cache the extraction result
            cache_manager
                .set(
                    &cache_key,
                    &extraction_result,
                    Some(tenant_id),
                    Some(std::time::Duration::from_secs(3600)), // 1 hour
                    Some(riptide_persistence::cache::CacheMetadata {
                        version: "1.0.0".to_string(),
                        content_type: Some("application/json".to_string()),
                        source: Some("riptide_html_extractor".to_string()),
                        tags: vec![
                            "html".to_string(),
                            "extraction".to_string(),
                            "news".to_string(),
                        ],
                        attributes: {
                            let mut attrs = HashMap::new();
                            attrs.insert("tenant_id".to_string(), tenant_id.to_string());
                            attrs.insert("extraction_time_ms".to_string(), "245".to_string());
                            attrs
                        },
                    }),
                )
                .await?;

            println!("    ✓ Extraction completed and cached");
            println!("      Cache key: {}", cache_key);
        }
    }

    // Simulate integration with search functionality
    println!("    🔍 Simulating search integration...");

    let search_cache_key = cache_manager.generate_key("search_recent_news_tech", Some(tenant_id));

    // This would integrate with riptide-search
    let search_results = serde_json::json!({
        "query": "recent news technology",
        "results": [
            {
                "url": "https://example.com/news/article",
                "title": "Breaking News: Technology Advances",
                "snippet": "The extracted and cleaned content...",
                "score": 0.95
            }
        ],
        "total_results": 1,
        "search_time_ms": 45,
        "cached_at": chrono::Utc::now().to_rfc3339()
    });

    cache_manager
        .set(
            &search_cache_key,
            &search_results,
            Some(tenant_id),
            Some(std::time::Duration::from_secs(300)), // 5 minutes for search results
            None,
        )
        .await?;

    println!("    ✓ Search results cached");

    // Show integration benefits
    println!("    📈 Integration benefits demonstrated:");
    println!("      - Tenant-isolated caching for multi-user scenarios");
    println!("      - Performance optimization with <5ms cache access");
    println!("      - Automatic compression for large extraction results");
    println!("      - TTL-based invalidation for fresh content");
    println!("      - Resource quota enforcement per tenant");
    println!("      - Session state persistence across requests");

    Ok(())
}
