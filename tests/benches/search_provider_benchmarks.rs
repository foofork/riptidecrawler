use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use search_provider_tests::mocks::{MockSearchProvider, MockHttpClient, MockHttpResponse};
use std::time::Duration;

fn bench_url_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("url_detection");

    let test_cases = vec![
        "https://eventmesh.apache.org/docs",
        "http://localhost:8080/api",
        "eventmesh architecture",
        "apache kafka vs eventmesh",
        "https://github.com/apache/eventmesh",
    ];

    for case in &test_cases {
        group.bench_with_input(
            BenchmarkId::new("is_url", case),
            case,
            |b, input| {
                // This would use the actual NoneProvider implementation
                // For now, we'll benchmark the regex pattern directly
                let url_pattern = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
                b.iter(|| {
                    black_box(url_pattern.is_match(black_box(input)));
                });
            },
        );
    }

    group.finish();
}

fn bench_mock_search_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("mock_search");
    group.throughput(Throughput::Elements(1));

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("fast_mock_search", |b| {
        let provider = MockSearchProvider::new();
        b.to_async(&rt).iter(|| async {
            black_box(provider.search("test query").await)
        });
    });

    group.bench_function("slow_mock_search", |b| {
        let provider = MockSearchProvider::new()
            .with_delay(Duration::from_millis(100));
        b.to_async(&rt).iter(|| async {
            black_box(provider.search("test query").await)
        });
    });

    group.finish();
}

fn bench_concurrent_searches(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_searches");
    let rt = tokio::runtime::Runtime::new().unwrap();

    for concurrency in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", concurrency),
            concurrency,
            |b, &concurrency| {
                let provider = std::sync::Arc::new(MockSearchProvider::new());

                b.to_async(&rt).iter(|| async {
                    let mut handles = vec![];

                    for i in 0..concurrency {
                        let provider_clone = provider.clone();
                        let query = format!("test query {}", i);

                        let handle = tokio::spawn(async move {
                            provider_clone.search(&query).await
                        });
                        handles.push(handle);
                    }

                    let results = futures::future::join_all(handles).await;
                    black_box(results);
                });
            },
        );
    }

    group.finish();
}

fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let sample_response = r#"{
        "organic": [
            {
                "title": "Apache EventMesh",
                "link": "https://eventmesh.apache.org/",
                "snippet": "Apache EventMesh is a dynamic event-driven application runtime"
            },
            {
                "title": "EventMesh GitHub",
                "link": "https://github.com/apache/eventmesh",
                "snippet": "Event-driven application runtime for cloud native applications"
            }
        ],
        "searchParameters": {
            "q": "EventMesh Apache",
            "type": "search",
            "engine": "google"
        }
    }"#;

    group.bench_function("parse_serper_response", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(black_box(sample_response)));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_url_detection,
    bench_mock_search_performance,
    bench_concurrent_searches,
    bench_json_parsing
);

criterion_main!(benches);