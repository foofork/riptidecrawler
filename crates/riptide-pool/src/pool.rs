#[cfg(feature = "wasm-pool")]
use wasmtime::{component::*, Engine};

#[cfg(feature = "wasm-pool")]
use super::config::ExtractorConfig;

#[cfg(feature = "wasm-pool")]
use super::models::{CircuitBreakerState, PooledInstance};

#[cfg(feature = "wasm-pool")]
use riptide_events::{Event, EventBus, EventEmitter, PoolEvent, PoolOperation};

#[cfg(feature = "wasm-pool")]
use std::env;

#[cfg(feature = "wasm-pool")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "wasm-pool")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "wasm-pool")]
use std::time::Duration;

#[cfg(feature = "wasm-pool")]
use async_trait::async_trait;

#[cfg(feature = "wasm-pool")]
use tracing::{debug, info, warn};

#[cfg(feature = "wasm-pool")]
use anyhow::{anyhow, Result};

#[cfg(feature = "wasm-pool")]
wasmtime::component::bindgen!({
    world: "extractor",
    path: "wit",
});

/// Advanced instance pool with semaphore-based concurrency control
#[cfg(feature = "wasm-pool")]
pub struct AdvancedInstancePool {
    /// Pool configuration
    pub(super) config: ExtractorConfig,
    /// Shared engine for all instances
    pub(super) engine: Arc<Engine>,
    /// Shared component for all instances
    pub(super) component: Arc<Component>,
    /// Shared linker for all instances
    pub(super) linker: Arc<Linker<WasmResourceTracker>>,
    /// Available instances queue
    pub(super) available_instances: Arc<Mutex<VecDeque<PooledInstance>>>,
    /// Semaphore for concurrency control
    pub(super) semaphore: Arc<Semaphore>,
    /// Performance metrics
    pub(super) metrics: Arc<Mutex<PerformanceMetrics>>,
    /// Circuit breaker state
    pub(super) circuit_state: Arc<Mutex<CircuitBreakerState>>,
    /// Component path for creation
    #[allow(dead_code)]
    pub(super) component_path: String,
    /// Pool unique identifier
    pub(super) pool_id: String,
    /// Optional event bus for event emission
    pub(super) event_bus: Option<Arc<EventBus>>,
    /// Counter for pending instance acquisitions
    pub(super) pending_acquisitions: Arc<AtomicUsize>,
}

#[cfg(feature = "wasm-pool")]
impl AdvancedInstancePool {
    /// Create new instance pool with configuration
    pub async fn new(
        config: ExtractorConfig,
        engine: Engine,
        component_path: &str,
    ) -> Result<Self> {
        info!(
            max_pool_size = config.max_pool_size,
            initial_pool_size = config.initial_pool_size,
            wit_validation_enabled = config.enable_wit_validation,
            "Initializing advanced instance pool"
        );

        // Load component
        let component = Component::from_file(&engine, component_path)?;

        // Create initial metrics for tracking validation
        let initial_metrics = Arc::new(Mutex::new(PerformanceMetrics::default()));

        // P2-2: WIT validation before instantiation
        // Note: WIT validation is disabled to avoid circular dependency with riptide-core.
        // Validation can be added at a higher level if needed.
        if config.enable_wit_validation {
            warn!("WIT validation is enabled in config but not available in riptide-pool to avoid circular dependency");
        }

        let linker: wasmtime::component::Linker<WasmResourceTracker> =
            wasmtime::component::Linker::new(&engine);

        let pool = Self {
            config: config.clone(),
            engine: Arc::new(engine),
            component: Arc::new(component),
            linker: Arc::new(linker),
            available_instances: Arc::new(Mutex::new(VecDeque::with_capacity(
                config.max_pool_size,
            ))),
            semaphore: Arc::new(Semaphore::new(config.max_pool_size)),
            metrics: initial_metrics,
            circuit_state: Arc::new(Mutex::new(CircuitBreakerState::Closed {
                failure_count: 0,
                success_count: 0,
                last_failure: None,
            })),
            component_path: component_path.to_string(),
            pool_id: Uuid::new_v4().to_string(),
            event_bus: None,
            pending_acquisitions: Arc::new(AtomicUsize::new(0)),
        };

        // Pre-warm the pool
        pool.warm_up().await?;

        // Emit pool warmup event if event bus is available
        if let Some(event_bus) = &pool.event_bus {
            let warmup_event = PoolEvent::new(
                PoolOperation::PoolWarmup,
                pool.pool_id.clone(),
                "instance_pool",
            );

            if let Err(e) = event_bus.emit(warmup_event).await {
                warn!(error = %e, pool_id = %pool.pool_id, "Failed to emit pool warmup event");
            }
        }

        info!("Advanced instance pool initialized successfully");
        Ok(pool)
    }

    /// Pre-warm the pool with initial instances
    async fn warm_up(&self) -> Result<()> {
        debug!(
            warm_count = self.config.initial_pool_size,
            "Warming up instance pool"
        );

        // Create instances without holding the lock
        let mut created_instances = Vec::new();
        for i in 0..self.config.initial_pool_size {
            let instance = self.create_instance().await?;
            created_instances.push(instance);
            debug!(instance_index = i, "Instance pre-warmed");
        }

        // Now add all instances to the pool in one go
        {
            let mut instances = self.available_instances.lock().await;
            for instance in created_instances {
                instances.push_back(instance);
            }
        }

        // Update metrics
        let pool_size = {
            let instances = self.available_instances.lock().await;
            instances.len()
        };

        {
            let mut metrics = self.metrics.lock().await;
            metrics.pool_size = pool_size;
        }

        info!(pool_size = pool_size, "Pool warm-up completed");

        Ok(())
    }

    /// Extract content with full instance pooling and fallback support
    pub async fn extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Check circuit breaker
        if self.is_circuit_open().await {
            return self.fallback_extract(html, url, mode).await;
        }

        let start_time = Instant::now();

        // Increment pending acquisitions counter
        self.pending_acquisitions.fetch_add(1, Ordering::Relaxed);

        // Acquire semaphore permit with timeout
        let timeout_duration =
            Duration::from_millis(self.config.extraction_timeout.unwrap_or(30000));
        let permit = match timeout(timeout_duration, self.semaphore.acquire()).await {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                // Decrement counter on semaphore error
                self.pending_acquisitions.fetch_sub(1, Ordering::Relaxed);
                return Err(anyhow!("Semaphore closed"));
            }
            Err(_) => {
                self.record_timeout().await;

                // Emit timeout event
                if let Some(event_bus) = &self.event_bus {
                    let event = riptide_events::ExtractionEvent::new(
                        riptide_events::ExtractionOperation::Timeout,
                        url.to_string(),
                        format!("{:?}", mode),
                        &format!("pool-{}", self.pool_id),
                    )
                    .with_duration(start_time.elapsed());

                    if let Err(e) = event_bus.emit(event).await {
                        warn!(error = %e, "Failed to emit timeout event");
                    }
                }

                // Decrement counter before fallback
                self.pending_acquisitions.fetch_sub(1, Ordering::Relaxed);
                return self.fallback_extract(html, url, mode).await;
            }
        };

        let semaphore_wait_time = start_time.elapsed();

        // Get or create instance
        let mut instance = self.get_or_create_instance().await?;

        // Emit instance acquired event
        if let Some(event_bus) = &self.event_bus {
            let event = PoolEvent::new(
                PoolOperation::InstanceAcquired,
                self.pool_id.clone(),
                "instance_pool",
            )
            .with_instance_id(instance.id.clone());

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, instance_id = %instance.id, "Failed to emit instance acquired event");
            }
        }

        // Perform extraction with epoch timeout
        let extraction_result = self
            .extract_with_instance(&mut instance, html, url, mode.clone())
            .await;

        // Update metrics and return instance
        let success = extraction_result.is_ok();
        instance.record_usage(success);

        // Emit extraction completion event
        if let Some(event_bus) = &self.event_bus {
            let duration = start_time.elapsed();
            let operation = if success {
                riptide_events::ExtractionOperation::Completed
            } else {
                riptide_events::ExtractionOperation::Failed
            };

            let mut event = riptide_events::ExtractionEvent::new(
                operation,
                url.to_string(),
                format!("{:?}", mode),
                &format!("pool-{}", self.pool_id),
            )
            .with_duration(duration);

            if let Err(ref error) = extraction_result {
                event = event.with_error(error.to_string());
            }

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit extraction event");
            }
        }

        self.return_instance(instance).await;

        // Update circuit breaker
        self.record_extraction_result(success, start_time.elapsed())
            .await;

        // Update semaphore wait time metric
        self.update_semaphore_wait_time(semaphore_wait_time).await;

        // Decrement pending acquisitions counter before releasing permit
        self.pending_acquisitions.fetch_sub(1, Ordering::Relaxed);

        // Release permit
        drop(permit);

        match extraction_result {
            Ok(doc) => Ok(doc),
            Err(e) => {
                // Fallback to native extraction if WASM fails
                tracing::warn!(
                    url = %url,
                    error = %e,
                    "WASM extraction failed, falling back to native extraction"
                );

                // Use native parser as fallback
                use riptide_extraction::native_parser::NativeHtmlParser;
                let native_parser = NativeHtmlParser::default();

                match native_parser.parse_headless_html(html, url) {
                    Ok(mut doc) => {
                        // Mark that fallback occurred
                        if let Some(ref mut metadata) = doc.parser_metadata {
                            metadata.fallback_occurred = true;
                            metadata.primary_error = Some(e.to_string());
                            metadata.parser_used = "native_fallback".to_string();
                        }
                        Ok(doc)
                    }
                    Err(native_err) => {
                        tracing::error!(
                            url = %url,
                            wasm_error = %e,
                            native_error = %native_err,
                            "Both WASM and native extraction failed"
                        );
                        Err(anyhow!(
                            "Both WASM ({}) and native ({}) extraction failed",
                            e,
                            native_err
                        ))
                    }
                }
            }
        }
    }

    /// Get or create instance from pool
    async fn get_or_create_instance(&self) -> Result<PooledInstance> {
        // Try to get from pool first
        let (maybe_instance, pool_empty) = {
            let mut instances = self.available_instances.lock().await;
            let pool_empty = instances.is_empty();
            let maybe_instance = instances.pop_front();
            (maybe_instance, pool_empty)
        }; // Lock dropped here

        if let Some(instance) = maybe_instance {
            if instance.is_healthy(&self.config) {
                return Ok(instance);
            }
            // Instance is unhealthy, create new one
            debug!(instance_id = %instance.id, "Discarding unhealthy instance");

            // Emit unhealthy instance event
            if let Some(event_bus) = &self.event_bus {
                let mut event = PoolEvent::new(
                    PoolOperation::InstanceUnhealthy,
                    self.pool_id.clone(),
                    "instance_pool",
                )
                .with_instance_id(instance.id.clone());

                event.add_metadata("reason", "health_check_failed");
                event.add_metadata("use_count", &instance.use_count.to_string());
                event.add_metadata("failure_count", &instance.failure_count.to_string());

                if let Err(e) = event_bus.emit(event).await {
                    warn!(error = %e, instance_id = %instance.id, "Failed to emit instance unhealthy event");
                }
            }
        } else if pool_empty {
            // Pool is empty, emit pool exhausted event
            self.emit_pool_exhausted_event().await;
        }

        // Create new instance if needed
        self.create_instance().await
    }

    /// Create new instance
    pub async fn create_instance(&self) -> Result<PooledInstance> {
        debug!("Creating new WASM instance");

        let instance = PooledInstance::new(
            self.engine.clone(),
            self.component.clone(),
            self.linker.clone(),
            self.config.memory_limit_pages.unwrap_or(256) as usize,
        );

        debug!(instance_id = %instance.id, "New WASM instance created");

        // Emit instance created event
        if let Some(event_bus) = &self.event_bus {
            let event = PoolEvent::new(
                PoolOperation::InstanceCreated,
                self.pool_id.clone(),
                "instance_pool",
            )
            .with_instance_id(instance.id.clone());

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, instance_id = %instance.id, "Failed to emit instance created event");
            }
        }

        Ok(instance)
    }

    /// Extract content using specific instance
    async fn extract_with_instance(
        &self,
        instance: &mut PooledInstance,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Create fresh store to prevent state leaks
        let mut store = instance.create_fresh_store();

        // Set epoch deadline for timeout handling
        store.set_epoch_deadline(self.config.epoch_timeout_ms);

        // Spawn epoch advancement task
        let engine_weak = Arc::downgrade(&instance.engine);
        tokio::spawn(async move {
            sleep(Duration::from_millis(30000)).await; // 30 second timeout
            if let Some(engine) = engine_weak.upgrade() {
                engine.increment_epoch();
            }
        });

        // Instantiate component with fresh bindings
        let bindings = Extractor::instantiate(&mut store, &instance.component, &*instance.linker)
            .map_err(|e| anyhow!("Component instantiation failed: {}", e))?;

        // Convert mode to WIT format
        let wit_mode = self.convert_extraction_mode(mode);

        // Execute extraction (sync call, use epoch deadline for timeout)
        let result = bindings
            .interface0
            .call_extract(&mut store, html, url, &wit_mode);

        match result {
            Ok(Ok(content)) => {
                // Success - convert to internal format
                Ok(ExtractedDoc {
                    url: content.url,
                    title: content.title,
                    byline: content.byline,
                    published_iso: content.published_iso,
                    markdown: Some(content.markdown),
                    text: content.text,
                    links: content.links,
                    media: content.media,
                    language: content.language,
                    reading_time: content.reading_time,
                    quality_score: content.quality_score,
                    parser_metadata: Some(ParserMetadata {
                        parser_used: "wasm".to_string(),
                        confidence_score: 0.9,
                        fallback_occurred: false,
                        parse_time_ms: 0,
                        extraction_path: None,
                        primary_error: None,
                    }),
                    word_count: content.word_count,
                    categories: content.categories,
                    site_name: content.site_name,
                    description: content.description,
                    html: None, // Not populated during normal extraction
                })
            }
            Ok(Err(extraction_error)) => {
                Err(anyhow!("WASM extraction error: {:?}", extraction_error))
            }
            Err(e) => Err(anyhow!("Component call failed: {}", e)),
        }
    }

    /// Return instance to pool
    pub async fn return_instance(&self, instance: PooledInstance) {
        let instance_id = instance.id.clone();
        let is_healthy = instance.is_healthy(&self.config);

        if is_healthy {
            // Add healthy instance back to pool
            {
                let mut instances = self.available_instances.lock().await;
                instances.push_back(instance);
            } // Lock dropped here

            // Emit instance released event
            if let Some(event_bus) = &self.event_bus {
                let event = PoolEvent::new(
                    PoolOperation::InstanceReleased,
                    self.pool_id.clone(),
                    "instance_pool",
                )
                .with_instance_id(instance_id.clone());

                if let Err(e) = event_bus.emit(event).await {
                    warn!(error = %e, instance_id = %instance_id, "Failed to emit instance released event");
                }
            }
        } else {
            // Emit instance destroyed event for unhealthy instances
            if let Some(event_bus) = &self.event_bus {
                let event = PoolEvent::new(
                    PoolOperation::InstanceDestroyed,
                    self.pool_id.clone(),
                    "instance_pool",
                )
                .with_instance_id(instance_id.clone());

                if let Err(e) = event_bus.emit(event).await {
                    warn!(error = %e, instance_id = %instance_id, "Failed to emit instance destroyed event");
                }
            }
        }

        // Update pool size metric
        let pool_size = {
            let instances = self.available_instances.lock().await;
            instances.len()
        };

        {
            let mut metrics = self.metrics.lock().await;
            metrics.pool_size = pool_size;
        }
    }

    /// Fallback to native extraction
    async fn fallback_extract(
        &self,
        html: &str,
        url: &str,
        mode: ExtractionMode,
    ) -> Result<ExtractedDoc> {
        // Record fallback usage
        {
            let mut metrics = self.metrics.lock().await;
            metrics.fallback_extractions += 1;
        }

        // Emit fallback used event
        if let Some(event_bus) = &self.event_bus {
            let event = riptide_events::ExtractionEvent::new(
                riptide_events::ExtractionOperation::FallbackUsed,
                url.to_string(),
                format!("{:?}", mode),
                &format!("pool-{}", self.pool_id),
            );

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit fallback used event");
            }
        }

        warn!("Using fallback extraction for URL: {}", url);

        // Use scraper for HTML parsing fallback
        use scraper::{Html, Selector};
        let document = Html::parse_document(html);

        // Extract title
        let mut title = None;
        for selector_str in &["title", "h1", "[property='og:title']"] {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    if *selector_str == "[property='og:title']" {
                        if let Some(content) = element.value().attr("content") {
                            title = Some(content.to_string());
                            break;
                        }
                    } else {
                        let text = element
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string();
                        if !text.is_empty() {
                            title = Some(text);
                            break;
                        }
                    }
                }
            }
        }

        // Extract main content
        let mut text = String::new();
        for selector_str in &["article", ".content", "main", "body"] {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    text = element.text().collect::<Vec<_>>().join(" ");
                    if text.len() > 100 {
                        break;
                    }
                }
            }
        }

        // Clean up text
        text = text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(5000) // Limit to 5000 chars
            .collect();

        // Extract links
        let mut links = vec![];
        if let Ok(selector) = Selector::parse("a[href]") {
            for element in document.select(&selector).take(50) {
                if let Some(href) = element.value().attr("href") {
                    links.push(href.to_string());
                }
            }
        }

        // Extract media (images)
        let mut media = vec![];
        if let Ok(selector) = Selector::parse("img[src]") {
            for element in document.select(&selector).take(20) {
                if let Some(src) = element.value().attr("src") {
                    media.push(src.to_string());
                }
            }
        }

        // Create simple markdown
        let markdown = if let Some(ref t) = title {
            format!("# {}\n\n{}", t, text.chars().take(2000).collect::<String>())
        } else {
            format!(
                "# Content\n\n{}",
                text.chars().take(2000).collect::<String>()
            )
        };

        Ok(ExtractedDoc {
            url: url.to_string(),
            title,
            text,
            markdown: Some(markdown),
            links,
            media,
            ..Default::default()
        })
    }

    /// Convert ExtractionMode to WIT format
    #[cfg(feature = "wasm-pool")]
    fn convert_extraction_mode(
        &self,
        mode: ExtractionMode,
    ) -> exports::riptide::extractor::extract::ExtractionMode {
        match mode {
            ExtractionMode::Article => {
                exports::riptide::extractor::extract::ExtractionMode::Article
            }
            ExtractionMode::Full => exports::riptide::extractor::extract::ExtractionMode::Full,
            ExtractionMode::Metadata => {
                exports::riptide::extractor::extract::ExtractionMode::Metadata
            }
            ExtractionMode::Custom(selectors) => {
                exports::riptide::extractor::extract::ExtractionMode::Custom(selectors)
            }
        }
    }

    /// Check if circuit breaker is open
    async fn is_circuit_open(&self) -> bool {
        let state = self.circuit_state.lock().await;
        match *state {
            CircuitBreakerState::Open { opened_at, .. } => {
                opened_at.elapsed() < Duration::from_millis(self.config.circuit_breaker_timeout)
            }
            _ => false,
        }
    }

    /// Record extraction result for circuit breaker
    async fn record_extraction_result(&self, success: bool, duration: Duration) {
        // Phase 1: Update metrics in scoped block
        let (circuit_breaker_trips, failed_extractions, total_extractions) = {
            let mut metrics = self.metrics.lock().await;

            // Update basic metrics
            metrics.total_extractions += 1;
            if success {
                metrics.successful_extractions += 1;
            } else {
                metrics.failed_extractions += 1;
            }

            // Update average processing time
            let new_time = duration.as_millis() as f64;
            metrics.avg_processing_time_ms = if metrics.total_extractions == 1 {
                new_time
            } else {
                (metrics.avg_processing_time_ms + new_time) / 2.0
            };

            (
                metrics.circuit_breaker_trips,
                metrics.failed_extractions,
                metrics.total_extractions,
            )
        }; // Metrics lock dropped here

        // Phase 2: Update circuit breaker state in scoped block
        let (should_emit_trip_event, should_emit_reset_event, trip_metrics, successful_extractions) = {
            let mut state = self.circuit_state.lock().await;
            let mut should_emit_trip = false;
            let mut should_emit_reset = false;
            let mut trip_data = None;
            let mut success_count = 0;

            // Update circuit breaker state
            let new_state = match &*state {
                CircuitBreakerState::Closed {
                    failure_count,
                    success_count: sc,
                    ..
                } => {
                    let new_failure_count = if success { 0 } else { failure_count + 1 };
                    let new_success_count = if success { sc + 1 } else { *sc };
                    let total_requests = new_failure_count + new_success_count;

                    if total_requests >= 10 {
                        let failure_rate =
                            (new_failure_count as f64 / total_requests as f64) * 100.0;
                        if failure_rate >= self.config.circuit_breaker_failure_threshold as f64 {
                            // Need to update metrics again for circuit breaker trips
                            let new_trips = circuit_breaker_trips + 1;

                            warn!(
                                failure_rate = failure_rate,
                                threshold = self.config.circuit_breaker_failure_threshold,
                                "Circuit breaker opened due to high failure rate"
                            );

                            // Mark that we should emit event after locks are released
                            should_emit_trip = true;
                            trip_data = Some((
                                self.config.circuit_breaker_failure_threshold,
                                new_trips,
                                failed_extractions,
                                total_extractions,
                            ));

                            CircuitBreakerState::Open {
                                opened_at: Instant::now(),
                                failure_count: new_failure_count,
                            }
                        } else {
                            CircuitBreakerState::Closed {
                                failure_count: new_failure_count,
                                success_count: new_success_count,
                                last_failure: if success { None } else { Some(Instant::now()) },
                            }
                        }
                    } else {
                        CircuitBreakerState::Closed {
                            failure_count: new_failure_count,
                            success_count: new_success_count,
                            last_failure: if success { None } else { Some(Instant::now()) },
                        }
                    }
                }
                CircuitBreakerState::Open {
                    opened_at,
                    failure_count,
                } => {
                    if opened_at.elapsed()
                        >= Duration::from_millis(self.config.circuit_breaker_timeout)
                    {
                        info!("Circuit breaker transitioning to half-open");
                        CircuitBreakerState::HalfOpen {
                            test_requests: 0,
                            start_time: Instant::now(),
                        }
                    } else {
                        CircuitBreakerState::Open {
                            opened_at: *opened_at,
                            failure_count: *failure_count,
                        }
                    }
                }
                CircuitBreakerState::HalfOpen {
                    test_requests,
                    start_time,
                } => {
                    if success {
                        info!("Circuit breaker closing after successful test request");

                        // Mark that we should emit reset event after locks are released
                        should_emit_reset = true;
                        success_count = 1; // Track successful extractions for event

                        CircuitBreakerState::Closed {
                            failure_count: 0,
                            success_count: 1,
                            last_failure: None,
                        }
                    } else if *test_requests >= 3 {
                        warn!("Circuit breaker reopening after failed test requests");
                        CircuitBreakerState::Open {
                            opened_at: Instant::now(),
                            failure_count: 1,
                        }
                    } else {
                        CircuitBreakerState::HalfOpen {
                            test_requests: test_requests + 1,
                            start_time: *start_time,
                        }
                    }
                }
            };

            *state = new_state;

            (
                should_emit_trip,
                should_emit_reset,
                trip_data,
                success_count,
            )
        }; // Circuit state lock dropped here

        // Update metrics if circuit breaker tripped (needs separate lock acquisition)
        if should_emit_trip_event {
            let mut metrics = self.metrics.lock().await;
            metrics.circuit_breaker_trips += 1;
        } // Metrics lock dropped here

        // Phase 3: Emit events without holding any locks
        if should_emit_trip_event {
            if let Some((failure_threshold, total_trips, failed_extractions, total_extractions)) =
                trip_metrics
            {
                if let Some(event_bus) = &self.event_bus {
                    let event_bus = event_bus.clone();
                    let pool_id = self.pool_id.clone();

                    tokio::spawn(async move {
                        let mut event = PoolEvent::new(
                            PoolOperation::CircuitBreakerTripped,
                            pool_id,
                            "instance_pool",
                        );

                        event.add_metadata("failure_threshold", &failure_threshold.to_string());
                        event.add_metadata("total_trips", &total_trips.to_string());
                        event.add_metadata("failed_extractions", &failed_extractions.to_string());
                        event.add_metadata("total_extractions", &total_extractions.to_string());

                        if let Err(e) = event_bus.emit(event).await {
                            warn!(error = %e, "Failed to emit circuit breaker tripped event");
                        }
                    });
                }
            }
        }

        if should_emit_reset_event {
            if let Some(event_bus) = &self.event_bus {
                let event_bus = event_bus.clone();
                let pool_id = self.pool_id.clone();
                let total_trips = circuit_breaker_trips;

                tokio::spawn(async move {
                    let mut event = PoolEvent::new(
                        PoolOperation::CircuitBreakerReset,
                        pool_id,
                        "instance_pool",
                    );

                    event.add_metadata("total_trips", &total_trips.to_string());
                    event.add_metadata(
                        "successful_extractions",
                        &successful_extractions.to_string(),
                    );

                    if let Err(e) = event_bus.emit(event).await {
                        warn!(error = %e, "Failed to emit circuit breaker reset event");
                    }
                });
            }
        }
    }

    /// Record timeout occurrence
    async fn record_timeout(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.failed_extractions += 1;
        metrics.total_extractions += 1;
    }

    /// Record epoch timeout
    #[allow(dead_code)]
    async fn record_epoch_timeout(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.epoch_timeouts += 1;
    }

    /// Update semaphore wait time metric
    async fn update_semaphore_wait_time(&self, wait_time: Duration) {
        let mut metrics = self.metrics.lock().await;
        let wait_ms = wait_time.as_millis() as f64;
        metrics.semaphore_wait_time_ms = if metrics.total_extractions == 1 {
            wait_ms
        } else {
            (metrics.semaphore_wait_time_ms + wait_ms) / 2.0
        };
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().await.clone()
    }

    /// Get pool status for health checks
    pub async fn get_pool_status(&self) -> (usize, usize, usize) {
        let available = self.available_instances.lock().await.len();
        let max_size = self.config.max_pool_size;
        let active = max_size - available;
        (available, active, max_size)
    }

    /// Set event bus for event emission
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Get pool ID
    pub fn pool_id(&self) -> &str {
        &self.pool_id
    }

    /// Create pool metrics for event emission
    pub async fn get_pool_metrics_for_events(&self) -> PoolMetrics {
        let (available, active, total) = self.get_pool_status().await;
        let performance_metrics = self.get_metrics().await;

        PoolMetrics {
            available_instances: available,
            active_instances: active,
            total_instances: total,
            pending_acquisitions: self.pending_acquisitions.load(Ordering::Relaxed),
            success_rate: if performance_metrics.total_extractions > 0 {
                performance_metrics.successful_extractions as f64
                    / performance_metrics.total_extractions as f64
            } else {
                1.0
            },
            avg_acquisition_time_ms: performance_metrics.semaphore_wait_time_ms as u64,
            avg_latency_ms: performance_metrics.avg_processing_time_ms as u64,
        }
    }

    /// Emit pool exhausted event when no instances are available
    async fn emit_pool_exhausted_event(&self) {
        if let Some(event_bus) = &self.event_bus {
            let mut event = PoolEvent::new(
                PoolOperation::PoolExhausted,
                self.pool_id.clone(),
                "instance_pool",
            );

            // Add pool status information
            let (available, active, total) = self.get_pool_status().await;
            event.add_metadata("available_instances", &available.to_string());
            event.add_metadata("active_instances", &active.to_string());
            event.add_metadata("total_instances", &total.to_string());

            if let Err(e) = event_bus.emit(event).await {
                warn!(error = %e, "Failed to emit pool exhausted event");
            }
        }
    }
}

/// Configuration for RIPTIDE_WASM_INSTANCES_PER_WORKER environment variable
#[cfg(feature = "wasm-pool")]
pub fn get_instances_per_worker() -> usize {
    env::var("RIPTIDE_WASM_INSTANCES_PER_WORKER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8)
}

/// Implement EventEmitter trait for AdvancedInstancePool
#[async_trait]
#[cfg(feature = "wasm-pool")]
impl EventEmitter for AdvancedInstancePool {
    async fn emit_event<E: Event + 'static>(&self, event: E) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            event_bus.emit(event).await
        } else {
            // Log a warning but don't fail if no event bus is configured
            debug!(
                "No event bus configured for pool {}, skipping event emission",
                self.pool_id
            );
            Ok(())
        }
    }

    async fn emit_events<E: Event + 'static>(&self, events: Vec<E>) -> Result<()> {
        if let Some(event_bus) = &self.event_bus {
            for event in events {
                event_bus.emit(event).await?;
            }
        } else {
            debug!(
                "No event bus configured for pool {}, skipping batch event emission",
                self.pool_id
            );
        }
        Ok(())
    }
}

/// Factory function to create an event-aware instance pool
#[cfg(feature = "wasm-pool")]
pub async fn create_event_aware_pool(
    config: ExtractorConfig,
    engine: Engine,
    component_path: &str,
    event_bus: Option<Arc<EventBus>>,
) -> Result<AdvancedInstancePool> {
    let mut pool = AdvancedInstancePool::new(config, engine, component_path).await?;

    if let Some(bus) = event_bus {
        pool.set_event_bus(bus);
        info!(pool_id = %pool.pool_id(), "Created event-aware instance pool");
    } else {
        info!(pool_id = %pool.pool_id(), "Created standard instance pool (no event bus)");
    }

    Ok(pool)
}
