//! Performance profiling endpoints (ultra-thin, delegates to ProfilingFacade)

use crate::context::ApplicationContext;
use crate::errors::ApiError;
use axum::{extract::State, response::Json};
use riptide_facade::facades::{
    AllocationMetrics, BottleneckAnalysis, CpuMetrics, HeapSnapshot, LeakDetectionResult,
    MemoryMetrics, ProfilingFacade,
};

pub async fn get_memory_profile(
    State(_state): State<ApplicationContext>,
) -> Result<Json<MemoryMetrics>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let metrics = facade.get_memory_metrics().await.map_err(ApiError::from)?;
    Ok(Json(metrics))
}

pub async fn get_cpu_profile(
    State(_state): State<ApplicationContext>,
) -> Result<Json<CpuMetrics>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let metrics = facade.get_cpu_metrics().await.map_err(ApiError::from)?;
    Ok(Json(metrics))
}

pub async fn get_bottleneck_analysis(
    State(_state): State<ApplicationContext>,
) -> Result<Json<BottleneckAnalysis>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let analysis = facade.analyze_bottlenecks().await.map_err(ApiError::from)?;
    Ok(Json(analysis))
}

pub async fn get_allocation_metrics(
    State(_state): State<ApplicationContext>,
) -> Result<Json<AllocationMetrics>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let metrics = facade
        .get_allocation_metrics()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(metrics))
}

pub async fn trigger_leak_detection(
    State(_state): State<ApplicationContext>,
) -> Result<Json<LeakDetectionResult>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let result = facade.detect_leaks().await.map_err(ApiError::from)?;
    Ok(Json(result))
}

pub async fn trigger_heap_snapshot(
    State(_state): State<ApplicationContext>,
) -> Result<Json<HeapSnapshot>, ApiError> {
    let facade = ProfilingFacade::new(Default::default()).map_err(ApiError::from)?;
    let snapshot = facade.create_snapshot().await.map_err(ApiError::from)?;
    Ok(Json(snapshot))
}
