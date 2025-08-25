use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use omnisearch_mcp::{
    common::{
        cache::{CacheManager, CacheProvider, MemoryCache},
        types::{BaseSearchParams, SearchResult},
        validation::{sanitize_query, validate_search_params},
    },
    config::{CacheConfig, CacheType},
};
use std::time::Duration;
use tokio::runtime::Runtime;

// Helper function to create test search results
fn create_test_results(count: usize) -> Vec<SearchResult> {
    (0..count)
        .map(|i| SearchResult {
            title: format!("Test Result {}", i),
            url: format!("https://example.com/{}", i),
            snippet: format!("Test snippet for result {}", i),
            score: Some(1.0 - (i as f64 / count as f64)),
            source_provider: "benchmark".to_string(),
        })
        .collect()
}

// Helper function to create test search params
fn create_test_params(query: &str, limit: Option<u32>) -> BaseSearchParams {
    BaseSearchParams {
        query: query.to_string(),
        limit,
        include_domains: Some(vec!["github.com".to_string(), "docs.rs".to_string()]),
        exclude_domains: Some(vec!["spam.com".to_string()]),
    }
}

// Benchmark cache operations
fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_operations");
    group.throughput(Throughput::Elements(1));

    // Memory cache benchmarks
    let config = CacheConfig {
        enabled: true,
        cache_type: CacheType::Memory,
        ttl_seconds: 3600,
        max_entries: 10000,
    };

    let cache = MemoryCache::new(&config);

    // Benchmark cache set operations with different result sizes
    for size in [1, 10, 50, 100, 500].iter() {
        let test_results = create_test_results(*size);

        group.bench_with_input(
            BenchmarkId::new("memory_cache_set", size),
            size,
            |b, &_size| {
                b.to_async(&rt).iter_batched(
                    || {
                        (
                            format!("test_key_{}", fastrand::u64(..)),
                            test_results.clone(),
                        )
                    },
                    |(key, results)| async move {
                        cache
                            .set(&key, results, Duration::from_secs(60))
                            .await
                            .unwrap();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    // Pre-populate cache for get benchmarks
    rt.block_on(async {
        for i in 0..1000 {
            let key = format!("bench_key_{}", i);
            let results = create_test_results(10);
            cache
                .set(&key, results, Duration::from_secs(60))
                .await
                .unwrap();
        }
    });

    // Benchmark cache get operations
    group.bench_function("memory_cache_get_hit", |b| {
        b.to_async(&rt).iter_batched(
            || format!("bench_key_{}", fastrand::usize(..1000)),
            |key| async move {
                let result = cache.get(&key).await.unwrap();
                black_box(result);
            },
            BatchSize::SmallInput,
        );
    });

    group.bench_function("memory_cache_get_miss", |b| {
        b.to_async(&rt).iter_batched(
            || format!("missing_key_{}", fastrand::u64(..)),
            |key| async move {
                let result = cache.get(&key).await.unwrap();
                black_box(result);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

// Benchmark cache key generation
fn bench_cache_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_key_generation");
    group.throughput(Throughput::Elements(1));

    let test_queries = vec![
        "simple query",
        "longer query with more words and complexity",
        "query with special characters: @#$%^&*()",
        &"x".repeat(500), // Long query
    ];

    for (i, query) in test_queries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("generate_key", i), query, |b, query| {
            b.iter(|| {
                let key = CacheManager::generate_cache_key("test_provider", query, Some(10));
                black_box(key);
            });
        });
    }

    group.finish();
}

// Benchmark validation operations
fn bench_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");
    group.throughput(Throughput::Elements(1));

    let test_cases = vec![
        create_test_params("simple query", Some(10)),
        create_test_params("longer query with multiple words and complexity", Some(50)),
        create_test_params(&"word ".repeat(100), Some(100)),
    ];

    for (i, params) in test_cases.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("validate_params", i),
            params,
            |b, params| {
                b.iter(|| {
                    let result = validate_search_params(params);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

// Benchmark query sanitization
fn bench_query_sanitization(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_sanitization");
    group.throughput(Throughput::Elements(1));

    let dirty_query = format!(
        "{}dirty query with nulls{}",
        "\0".repeat(10),
        "\x01".repeat(10)
    );
    let long_query = "test ".repeat(200);

    let test_queries = vec![
        "clean query",
        "query\0with\x01control\x7fcharacters",
        &dirty_query,
        &long_query,
    ];

    for (i, query) in test_queries.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("sanitize_query", i), query, |b, query| {
            b.iter(|| {
                let sanitized = sanitize_query(query);
                black_box(sanitized);
            });
        });
    }

    group.finish();
}

// Benchmark search result processing
fn bench_search_result_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_result_processing");

    for size in [10, 50, 100, 500, 1000].iter() {
        let results = create_test_results(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("serialize_results", size),
            &results,
            |b, results| {
                b.iter(|| {
                    let serialized = serde_json::to_string(results).unwrap();
                    black_box(serialized);
                });
            },
        );

        // Benchmark deserialization
        let serialized = serde_json::to_string(&results).unwrap();
        group.bench_with_input(
            BenchmarkId::new("deserialize_results", size),
            &serialized,
            |b, serialized| {
                b.iter(|| {
                    let deserialized: Vec<SearchResult> = serde_json::from_str(serialized).unwrap();
                    black_box(deserialized);
                });
            },
        );
    }

    group.finish();
}

// Benchmark concurrent cache access
fn bench_concurrent_cache_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_cache_access");
    group.sample_size(50); // Reduce sample size for async benchmarks

    let config = CacheConfig {
        enabled: true,
        cache_type: CacheType::Memory,
        ttl_seconds: 3600,
        max_entries: 10000,
    };

    let cache = MemoryCache::new(&config);

    // Pre-populate cache
    rt.block_on(async {
        for i in 0..100 {
            let key = format!("concurrent_key_{}", i);
            let results = create_test_results(10);
            cache
                .set(&key, results, Duration::from_secs(60))
                .await
                .unwrap();
        }
    });

    group.bench_function("concurrent_reads", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks: Vec<_> = (0..10)
                .map(|i| {
                    let cache = &cache;
                    async move {
                        let key = format!("concurrent_key_{}", i % 100);
                        cache.get(&key).await.unwrap()
                    }
                })
                .collect();

            let results = futures::future::join_all(tasks).await;
            black_box(results);
        });
    });

    group.finish();
}

// Comprehensive search simulation
fn bench_search_simulation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("search_simulation");
    group.sample_size(20); // Reduce sample size for comprehensive benchmarks

    let config = CacheConfig {
        enabled: true,
        cache_type: CacheType::Memory,
        ttl_seconds: 3600,
        max_entries: 1000,
    };

    let cache = MemoryCache::new(&config);

    group.bench_function("full_search_pipeline", |b| {
        b.to_async(&rt).iter_batched(
            || {
                (
                    format!("rust programming tutorial {}", fastrand::u64(..1000)),
                    fastrand::u32(1..=50),
                )
            },
            |(query, limit)| async {
                // 1. Create and validate search params
                let params = create_test_params(&query, Some(limit));
                let validated = validate_search_params(&params).unwrap();

                // 2. Generate cache key
                let cache_key = CacheManager::generate_cache_key(
                    "benchmark_provider",
                    &validated.query,
                    validated.limit.map(|l| l as usize),
                );

                // 3. Check cache
                let cached_result = cache.get(&cache_key).await.unwrap();

                let results = if cached_result.is_some() {
                    cached_result.unwrap()
                } else {
                    // 4. Simulate search (create mock results)
                    let mock_results = create_test_results(limit as usize);

                    // 5. Cache results
                    cache
                        .set(&cache_key, mock_results.clone(), Duration::from_secs(60))
                        .await
                        .unwrap();

                    mock_results
                };

                black_box(results);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_operations,
    bench_cache_key_generation,
    bench_validation,
    bench_query_sanitization,
    bench_search_result_processing,
    bench_concurrent_cache_access,
    bench_search_simulation
);

criterion_main!(benches);
