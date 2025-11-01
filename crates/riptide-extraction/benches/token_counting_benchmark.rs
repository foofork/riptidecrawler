//! Benchmark comparing approximate vs exact token counting
//!
//! Run with: cargo bench --bench token_counting_benchmark

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use riptide_extraction::chunking::utils::{count_tokens, count_tokens_batch, count_tokens_exact};
use tokio::runtime::Runtime;

fn benchmark_approximate_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("approximate_token_counting");

    let test_texts = vec![
        ("short", "Hello world"),
        ("medium", "This is a medium length text with several words that should be counted appropriately."),
        ("long", "This is a much longer piece of text that contains many more words and should demonstrate the performance characteristics of the token counting algorithm. We want to ensure that even with longer texts, the performance remains acceptable and within our target latency requirements. The text continues here with more content to make it sufficiently long for benchmarking purposes."),
    ];

    for (name, text) in test_texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, &text| {
            b.iter(|| count_tokens(black_box(text)));
        });
    }

    group.finish();
}

fn benchmark_exact_counting(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("exact_token_counting");

    let test_texts = vec![
        ("short", "Hello world"),
        ("medium", "This is a medium length text with several words that should be counted appropriately."),
        ("long", "This is a much longer piece of text that contains many more words and should demonstrate the performance characteristics of the token counting algorithm. We want to ensure that even with longer texts, the performance remains acceptable and within our target latency requirements. The text continues here with more content to make it sufficiently long for benchmarking purposes."),
    ];

    for (name, text) in test_texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, &text| {
            b.to_async(&rt)
                .iter(|| async { count_tokens_exact(black_box(text)).await.unwrap() });
        });
    }

    group.finish();
}

fn benchmark_batch_counting(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("batch_token_counting");

    let texts: Vec<&str> = vec![
        "First chunk of text",
        "Second chunk with more content",
        "Third chunk",
        "Fourth chunk with some more words",
        "Fifth and final chunk for this batch test",
    ];

    group.bench_function("batch_5_chunks", |b| {
        b.to_async(&rt)
            .iter(|| async { count_tokens_batch(black_box(&texts)).await.unwrap() });
    });

    // Larger batch
    let large_batch: Vec<&str> = (0..20)
        .map(|i| match i % 3 {
            0 => "Short text",
            1 => "Medium length text with some content",
            _ => "Longer text that has more words and content to process",
        })
        .collect();

    group.bench_function("batch_20_chunks", |b| {
        b.to_async(&rt)
            .iter(|| async { count_tokens_batch(black_box(&large_batch)).await.unwrap() });
    });

    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("cache_performance");

    let text = "This is a repeated text that will be cached.";

    group.bench_function("first_call_cache_miss", |b| {
        b.to_async(&rt).iter(|| async {
            // This will be a cache hit after the first iteration
            count_tokens_exact(black_box(text)).await.unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_approximate_counting,
    benchmark_exact_counting,
    benchmark_batch_counting,
    benchmark_cache_performance
);
criterion_main!(benches);
