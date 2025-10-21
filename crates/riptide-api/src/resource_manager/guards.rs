//! Resource guards for RAII-based resource management.
//!
//! Provides automatic resource cleanup through Drop implementations.
//! Guards ensure resources are properly released even if errors occur.

use crate::resource_manager::{
    memory_manager::MemoryManager, metrics::ResourceMetrics, wasm_manager::WasmInstanceManager,
};
use riptide_browser::pool::BrowserCheckout;
use std::sync::Arc;
use tokio::sync::OwnedSemaphorePermit;

/// Resource guard for render operations
///
/// Holds all resources needed for a render operation and ensures
/// proper cleanup through RAII (Drop trait).
pub struct RenderResourceGuard {
    /// Browser checkout for render operations
    #[allow(dead_code)] // RAII guard field, accessed by browser() method
    pub browser_checkout: BrowserCheckout,
    #[allow(dead_code)] // Holds WASM resources, dropped on guard drop
    wasm_guard: WasmGuard,
    memory_tracked: usize,
    memory_manager: Arc<MemoryManager>,
    metrics: Arc<ResourceMetrics>,
}

impl RenderResourceGuard {
    /// Create a new render resource guard
    pub(crate) fn new(
        browser_checkout: BrowserCheckout,
        wasm_guard: WasmGuard,
        memory_tracked: usize,
        memory_manager: Arc<MemoryManager>,
        metrics: Arc<ResourceMetrics>,
    ) -> Self {
        Self {
            browser_checkout,
            wasm_guard,
            memory_tracked,
            memory_manager,
            metrics,
        }
    }

    /// Get the browser checkout for use
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn browser(&self) -> &BrowserCheckout {
        &self.browser_checkout
    }

    /// Get memory tracked by this guard
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn memory_tracked_mb(&self) -> usize {
        self.memory_tracked
    }

    // Note: cleanup() method is not provided for RenderResourceGuard because
    // BrowserCheckout::cleanup() consumes self, and we can't move out of
    // a field while self is borrowed. Instead, rely on the Drop implementation
    // which will handle cleanup via BrowserCheckout's own Drop.
    //
    // For explicit cleanup before drop, manage the BrowserCheckout lifecycle
    // directly without storing in a guard.
}

impl std::fmt::Debug for RenderResourceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderResourceGuard")
            .field("memory_tracked", &self.memory_tracked)
            .field("browser_checkout", &"<BrowserCheckout>")
            .field("wasm_guard", &"<WasmGuard>")
            .finish()
    }
}

impl Drop for RenderResourceGuard {
    fn drop(&mut self) {
        let memory_manager = self.memory_manager.clone();
        let metrics = self.metrics.clone();
        let memory_tracked = self.memory_tracked;

        // Decrement active browser count
        metrics
            .headless_active
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        // Note: browser_checkout will be moved in cleanup task below
        // We can't call cleanup() here directly because it consumes self
        // and Drop takes &mut self. The BrowserCheckout's own Drop
        // will handle checkin if cleanup() is not explicitly called.
        // For explicit cleanup, users should call cleanup() before drop.

        // Spawn cleanup task for async operations
        tokio::spawn(async move {
            memory_manager.track_deallocation(memory_tracked);
        });
    }
}

/// Resource guard for PDF operations
///
/// Manages PDF semaphore permit and memory tracking.
pub struct PdfResourceGuard {
    _permit: OwnedSemaphorePermit,
    memory_tracked: usize,
    memory_manager: Arc<MemoryManager>,
    metrics: Arc<ResourceMetrics>,
}

impl PdfResourceGuard {
    /// Create a new PDF resource guard
    pub(crate) fn new(
        permit: OwnedSemaphorePermit,
        memory_tracked: usize,
        memory_manager: Arc<MemoryManager>,
        metrics: Arc<ResourceMetrics>,
    ) -> Self {
        Self {
            _permit: permit,
            memory_tracked,
            memory_manager,
            metrics,
        }
    }

    /// Get memory tracked by this guard
    #[allow(dead_code)] // Reserved for future monitoring API
    pub fn memory_tracked_mb(&self) -> usize {
        self.memory_tracked
    }
}

impl std::fmt::Debug for PdfResourceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfResourceGuard")
            .field("memory_tracked", &self.memory_tracked)
            .field("_permit", &"<OwnedSemaphorePermit>")
            .finish()
    }
}

impl Drop for PdfResourceGuard {
    fn drop(&mut self) {
        let memory_manager = self.memory_manager.clone();
        let metrics = self.metrics.clone();
        let memory_tracked = self.memory_tracked;

        // Decrement active PDF count
        metrics
            .pdf_active
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        // Spawn cleanup task
        tokio::spawn(async move {
            memory_manager.track_deallocation(memory_tracked);
        });
    }
}

/// WASM guard with instance tracking
///
/// Keeps the WASM instance manager alive for the duration of the operation.
pub struct WasmGuard {
    #[allow(dead_code)] // Keeps manager Arc alive for lifetime of guard
    pub(crate) manager: Arc<WasmInstanceManager>,
}

impl WasmGuard {
    /// Create a new WASM guard
    pub(crate) fn new(manager: Arc<WasmInstanceManager>) -> Self {
        Self { manager }
    }
}

impl std::fmt::Debug for WasmGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmGuard").finish_non_exhaustive()
    }
}

/// Generic resource guard for future extensibility
///
/// Currently unused but reserved for future resource types.
#[allow(dead_code)]
pub struct ResourceGuard {
    pub resource_type: String,
    pub acquired_at: std::time::Instant,
    pub timeout: std::time::Duration,
    _guard: Option<Arc<dyn Send + Sync>>,
}

impl std::fmt::Debug for ResourceGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceGuard")
            .field("resource_type", &self.resource_type)
            .field("acquired_at", &self.acquired_at)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_manager::ResourceMetrics;

    #[tokio::test]
    async fn test_wasm_guard_creation() {
        let metrics = Arc::new(ResourceMetrics::new());
        let manager = Arc::new(WasmInstanceManager::new(metrics).unwrap());
        let guard = WasmGuard::new(manager);

        // Guard should exist and be debuggable
        assert!(format!("{:?}", guard).contains("WasmGuard"));
    }

    #[test]
    fn test_resource_guard_debug() {
        let guard = ResourceGuard {
            resource_type: "test".to_string(),
            acquired_at: std::time::Instant::now(),
            timeout: std::time::Duration::from_secs(30),
            _guard: None,
        };

        let debug_str = format!("{:?}", guard);
        assert!(debug_str.contains("ResourceGuard"));
        assert!(debug_str.contains("test"));
    }
}
