//! Performance benchmarks for composition traits
//!
//! Measures BoxStream overhead and composition performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use futures::stream::{self, BoxStream, StreamExt};
use tokio::runtime::Runtime;

/// Baseline: Direct stream without boxing
async fn baseline_stream(count: usize) -> Vec<u64> {
    stream::iter(0..count as u64).collect().await
}

/// BoxStream: Measure boxing overhead
async fn boxed_stream(count: usize) -> Vec<u64> {
    let stream: BoxStream<'static, u64> = Box::pin(stream::iter(0..count as u64));
    stream.collect().await
}

/// Composed stream: Spider + Extractor simulation
async fn composed_stream(count: usize) -> Vec<String> {
    let urls: BoxStream<'static, u64> = Box::pin(stream::iter(0..count as u64));

    urls.map(|n| format!("Extracted: {}", n)).collect().await
}

fn benchmark_stream_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("baseline_1000", |b| {
        b.iter(|| rt.block_on(async { black_box(baseline_stream(1000).await) }))
    });

    c.bench_function("boxed_1000", |b| {
        b.iter(|| rt.block_on(async { black_box(boxed_stream(1000).await) }))
    });

    c.bench_function("composed_1000", |b| {
        b.iter(|| rt.block_on(async { black_box(composed_stream(1000).await) }))
    });
}

fn benchmark_single_item(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("baseline_single", |b| {
        b.iter(|| rt.block_on(async { black_box(baseline_stream(1).await) }))
    });

    c.bench_function("boxed_single", |b| {
        b.iter(|| rt.block_on(async { black_box(boxed_stream(1).await) }))
    });
}

criterion_group!(benches, benchmark_stream_overhead, benchmark_single_item);
criterion_main!(benches);
