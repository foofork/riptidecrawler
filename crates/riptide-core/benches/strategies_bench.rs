//! Benchmarks for extraction strategies and chunking performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riptide_core::strategies::*;
use riptide_core::strategies::chunking::ChunkingMode;
use tokio::runtime::Runtime;

fn create_test_content(size: usize) -> String {
    let base_html = r#"
    <html>
    <head>
        <title>Benchmark Test Article</title>
        <meta name="description" content="Performance benchmarking test content">
        <meta name="author" content="Benchmark Author">
        <meta property="og:title" content="OG Benchmark Article">
        <meta property="og:description" content="OpenGraph description for benchmarking">
        <meta property="article:published_time" content="2023-12-01T10:00:00Z">
    </head>
    <body>
        <article>
            <h1>Benchmark Test Article</h1>
            <div class="author">Benchmark Author</div>
            <time datetime="2023-12-01">December 1, 2023</time>
            <div class="content">
    "#;

    let end_html = r#"
            </div>
        </article>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "JSON-LD Benchmark Article",
            "author": {"@type": "Person", "name": "JSON-LD Author"},
            "datePublished": "2023-12-01T10:00:00Z"
        }
        </script>
    </body>
    </html>
    "#;

    let mut content = String::from(base_html);
    let paragraph = "<p>This is a benchmark paragraph containing substantial content for performance testing. It includes enough text to provide meaningful performance metrics while representing realistic content extraction scenarios. The paragraph contains various HTML elements and text patterns that extraction strategies need to process efficiently.</p>\n";

    while content.len() + end_html.len() < size {
        content.push_str(paragraph);
    }

    content.push_str(end_html);
    content
}

fn bench_extraction_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sizes = vec![1024, 10240, 102400]; // 1KB, 10KB, 100KB

    let mut group = c.benchmark_group("extraction_strategies");

    for size in sizes {
        let content = create_test_content(size);
        group.throughput(Throughput::Bytes(content.len() as u64));

        // Trek extraction benchmark
        group.bench_with_input(
            BenchmarkId::new("trek", size),
            &content,
            |b, content| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            extraction::trek::extract(black_box(content), "http://example.com")
                                .await
                                .unwrap()
                        )
                    })
                })
            },
        );

        // CSS JSON extraction benchmark
        let selectors = extraction::css_json::default_selectors();
        group.bench_with_input(
            BenchmarkId::new("css_json", size),
            &content,
            |b, content| {
                let selectors = selectors.clone();
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            extraction::css_json::extract(black_box(content), "http://example.com", &selectors)
                                .await
                                .unwrap()
                        )
                    })
                })
            },
        );

        // Regex extraction benchmark
        let patterns = extraction::regex::default_patterns();
        group.bench_with_input(
            BenchmarkId::new("regex", size),
            &content,
            |b, content| {
                let patterns = patterns.clone();
                b.iter(|| {
                    rt.block_on(async {
                        black_box(
                            extraction::regex::extract(black_box(content), "http://example.com", &patterns)
                                .await
                                .unwrap()
                        )
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_chunking_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let content = "This is a comprehensive test sentence for chunking performance evaluation. ".repeat(500);

    let mut group = c.benchmark_group("chunking_strategies");
    group.throughput(Throughput::Bytes(content.len() as u64));

    // Sliding window chunking
    let sliding_config = ChunkingConfig {
        mode: ChunkingMode::Sliding,
        token_max: 100,
        overlap: 20,
        preserve_sentences: true,
        deterministic: true,
    };

    group.bench_function("sliding_window", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    chunking::sliding::chunk_sliding_window(black_box(&content), &sliding_config)
                        .await
                        .unwrap()
                )
            })
        })
    });

    // Sentence chunking
    let sentence_config = ChunkingConfig {
        mode: ChunkingMode::Sentence { max_sentences: 5 },
        token_max: 200,
        overlap: 0,
        preserve_sentences: true,
        deterministic: true,
    };

    group.bench_function("sentence", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    chunking::sentence::chunk_by_sentences(black_box(&content), 5, &sentence_config)
                        .await
                        .unwrap()
                )
            })
        })
    });

    // Fixed size chunking
    let fixed_config = ChunkingConfig {
        mode: ChunkingMode::Fixed { size: 200, by_tokens: false },
        token_max: 1000,
        overlap: 0,
        preserve_sentences: false,
        deterministic: true,
    };

    group.bench_function("fixed_size", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    chunking::fixed::chunk_fixed_size(black_box(&content), 200, false, &fixed_config)
                        .await
                        .unwrap()
                )
            })
        })
    });

    // Regex chunking
    let regex_config = ChunkingConfig::default();
    let pattern = r"\.\s+";

    group.bench_function("regex_chunking", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    chunking::regex::chunk_by_regex(black_box(&content), pattern, 50, &regex_config)
                        .await
                        .unwrap()
                )
            })
        })
    });

    // Topic chunking
    let topic_config = ChunkingConfig::default();

    group.bench_function("topic", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    chunking::topic::chunk_by_topics(black_box(&content), 0.3, &topic_config)
                        .await
                        .unwrap()
                )
            })
        })
    });

    group.finish();
}

fn bench_metadata_extraction(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let complex_html = r#"
    <html>
    <head>
        <title>Complex Metadata Test Article</title>
        <meta name="description" content="Complex metadata extraction benchmarking">
        <meta name="author" content="Benchmark Author">
        <meta name="keywords" content="benchmark, performance, metadata, extraction">
        <meta property="og:title" content="OpenGraph Benchmark Title">
        <meta property="og:description" content="OpenGraph description for benchmarking">
        <meta property="og:image" content="https://example.com/image.jpg">
        <meta property="og:url" content="https://example.com/article">
        <meta property="article:published_time" content="2023-12-01T10:00:00Z">
        <meta property="article:modified_time" content="2023-12-02T10:00:00Z">
        <meta property="article:author" content="OG Author">
        <meta property="article:section" content="Technology">
        <link rel="canonical" href="https://example.com/canonical">
    </head>
    <body>
        <script type="application/ld+json">
        {
            "@context": "https://schema.org",
            "@type": "Article",
            "headline": "JSON-LD Complex Article",
            "description": "Comprehensive JSON-LD metadata for benchmarking",
            "author": {
                "@type": "Person",
                "name": "JSON-LD Author"
            },
            "datePublished": "2023-12-01T10:00:00Z",
            "dateModified": "2023-12-02T10:00:00Z",
            "keywords": ["json-ld", "benchmark", "performance"],
            "image": "https://example.com/jsonld-image.jpg",
            "wordCount": 500
        }
        </script>
        <article itemscope itemtype="https://schema.org/Article">
            <h1 itemprop="headline">Microdata Article Title</h1>
            <div class="byline">By <span itemprop="author">Microdata Author</span></div>
            <time itemprop="datePublished" datetime="2023-12-01">December 1, 2023</time>
            <div itemprop="description">Microdata description content</div>
            <div class="content" itemprop="articleBody">
                <p>Article content with comprehensive metadata.</p>
            </div>
        </article>
    </body>
    </html>
    "#;

    c.bench_function("metadata_extraction", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(
                    metadata::extract_metadata(black_box(complex_html), "https://example.com")
                        .await
                        .unwrap()
                )
            })
        })
    });
}

fn bench_strategy_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let content = create_test_content(10240); // 10KB test content

    let configs = vec![
        ("trek_sliding", StrategyConfig {
            extraction: ExtractionStrategy::Trek,
            chunking: ChunkingConfig {
                mode: ChunkingMode::Sliding,
                token_max: 100,
                overlap: 20,
                preserve_sentences: true,
                deterministic: true,
            },
            enable_metrics: true,
            validate_schema: true,
        }),
        ("css_json_sentence", StrategyConfig {
            extraction: ExtractionStrategy::CssJson {
                selectors: extraction::css_json::default_selectors()
            },
            chunking: ChunkingConfig {
                mode: ChunkingMode::Sentence { max_sentences: 3 },
                token_max: 150,
                overlap: 0,
                preserve_sentences: true,
                deterministic: true,
            },
            enable_metrics: true,
            validate_schema: true,
        }),
        ("regex_fixed", StrategyConfig {
            extraction: ExtractionStrategy::Regex {
                patterns: extraction::regex::default_patterns()
            },
            chunking: ChunkingConfig {
                mode: ChunkingMode::Fixed { size: 200, by_tokens: false },
                token_max: 200,
                overlap: 0,
                preserve_sentences: false,
                deterministic: true,
            },
            enable_metrics: true,
            validate_schema: true,
        }),
    ];

    let mut group = c.benchmark_group("strategy_manager");
    group.throughput(Throughput::Bytes(content.len() as u64));

    for (name, config) in configs {
        group.bench_with_input(
            BenchmarkId::new("full_pipeline", name),
            &config,
            |b, config| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut manager = StrategyManager::new(config.clone());
                        black_box(
                            manager.extract_and_chunk(black_box(&content), "https://example.com")
                                .await
                                .unwrap()
                        )
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_token_counting(c: &mut Criterion) {
    let long_text = "This is a much longer text sample that contains many more words and should provide a better test of the token counting algorithm performance. It includes various types of content and punctuation marks.".repeat(10);
    let texts = vec![
        ("short", "Short text."),
        ("medium", "This is a medium length text with several words and punctuation."),
        ("long", long_text.as_str()),
    ];

    let mut group = c.benchmark_group("token_counting");

    for (name, text) in texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("count_tokens", name),
            text,
            |b, text| {
                b.iter(|| black_box(chunking::count_tokens(black_box(text))))
            },
        );
    }

    group.finish();
}

fn bench_quality_scoring(c: &mut Criterion) {
    let test_chunks = vec![
        ("high_quality", "This is a well-formed sentence with excellent structure and meaningful content. It demonstrates proper grammar, punctuation, and semantic coherence throughout the entire passage."),
        ("medium_quality", "This sentence has decent structure. Some content here."),
        ("low_quality", "short"),
    ];

    let mut group = c.benchmark_group("quality_scoring");

    for (name, content) in test_chunks {
        let metadata = chunking::ChunkMetadata {
            quality_score: 0.0,
            sentence_count: content.split('.').count(),
            word_count: content.split_whitespace().count(),
            has_complete_sentences: content.ends_with('.'),
            topic_keywords: chunking::extract_topic_keywords(content),
            chunk_type: "test".to_string(),
        };

        group.bench_with_input(
            BenchmarkId::new("calculate_quality", name),
            &(content, metadata),
            |b, (content, metadata)| {
                b.iter(|| black_box(chunking::calculate_chunk_quality(black_box(content), black_box(metadata))))
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_extraction_strategies,
    bench_chunking_strategies,
    bench_metadata_extraction,
    bench_strategy_manager,
    bench_token_counting,
    bench_quality_scoring
);
criterion_main!(benches);