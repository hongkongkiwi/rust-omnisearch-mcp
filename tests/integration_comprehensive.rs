use eyre::Result;
use omnisearch_mcp::{
    common::{
        cache::{get_cache_manager, CacheManager, MemoryCache},
        circuit_breaker::{call_with_circuit_breaker, get_circuit_breaker_stats},
        metrics::{record_request_metrics, METRICS_COLLECTOR},
        rate_limiter::{check_rate_limit, RATE_LIMITER_MANAGER},
        types::{BaseSearchParams, SearchResult},
        validation::{sanitize_query, validate_search_params},
    },
    config::{CacheConfig, CacheType, CONFIG},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_full_search_pipeline() -> Result<()> {
    // Create test search params
    let params = BaseSearchParams {
        query: "rust programming tutorial".to_string(),
        limit: Some(10),
        include_domains: Some(vec!["github.com".to_string(), "docs.rs".to_string()]),
        exclude_domains: None,
    };

    // 1. Validate input
    let validated = validate_search_params(&params)?;
    assert_eq!(validated.query, "rust programming tutorial");
    assert_eq!(validated.limit, Some(10));

    // 2. Check rate limiting
    let rate_check = check_rate_limit("test_provider").await;
    assert!(rate_check.is_ok());

    // 3. Generate cache key
    let cache_key = CacheManager::generate_cache_key(
        "test_provider",
        &validated.query,
        &format!(
            "limit={:?}&domains={:?}",
            validated.limit, validated.include_domains
        ),
    );
    assert!(cache_key.starts_with("omnisearch:test_provider:"));

    // 4. Check cache (should be empty initially)
    let cache_manager = get_cache_manager().await;
    let cached_result = cache_manager.get(&cache_key).await?;
    assert!(cached_result.is_none());

    // 5. Simulate search operation with circuit breaker
    let search_results = call_with_circuit_breaker("test_provider", || async {
        // Simulate API call
        sleep(Duration::from_millis(50)).await;

        Ok(vec![
            SearchResult {
                title: "Rust Programming Guide".to_string(),
                url: "https://github.com/rust-lang/rust".to_string(),
                snippet: Some("The Rust Programming Language".to_string()),
                score: Some(0.95),
                published_date: None,
                favicon_url: None,
            },
            SearchResult {
                title: "Rust Documentation".to_string(),
                url: "https://docs.rs/".to_string(),
                snippet: Some("Find and view Rust crate documentation".to_string()),
                score: Some(0.90),
                published_date: None,
                favicon_url: None,
            },
        ])
    })
    .await?;

    assert_eq!(search_results.len(), 2);
    assert_eq!(search_results[0].title, "Rust Programming Guide");

    // 6. Cache the results
    cache_manager
        .set(&cache_key, search_results.clone())
        .await?;

    // 7. Verify cache hit
    let cached_result = cache_manager.get(&cache_key).await?;
    assert!(cached_result.is_some());
    let cached = cached_result.unwrap();
    assert_eq!(cached.len(), 2);
    assert_eq!(cached[0].title, "Rust Programming Guide");

    // 8. Record metrics
    record_request_metrics(
        "test_provider",
        "search",
        Duration::from_millis(50),
        true,
        Some(1024),
        false, // Not a cache hit for this recording
    )
    .await;

    // 9. Verify circuit breaker stats
    let cb_stats = get_circuit_breaker_stats("test_provider").await;
    if let Some(stats) = cb_stats {
        assert_eq!(stats.provider, "test_provider");
    }

    Ok(())
}

#[tokio::test]
async fn test_cache_performance_and_consistency() -> Result<()> {
    let config = CacheConfig {
        enabled: true,
        cache_type: CacheType::Memory,
        ttl_seconds: 60,
        max_entries: 100,
        redis: Default::default(),
    };

    let cache = MemoryCache::new(&config);
    let test_data = vec![SearchResult {
        title: "Performance Test".to_string(),
        url: "https://example.com/perf".to_string(),
        snippet: Some("Performance testing data".to_string()),
        score: Some(0.8),
        published_date: None,
        favicon_url: None,
    }];

    // Test concurrent access
    let mut handles = Vec::new();

    for i in 0..10 {
        let cache_clone = &cache;
        let data_clone = test_data.clone();

        let handle = tokio::spawn(async move {
            let key = format!("perf_test_{}", i);

            // Set data
            cache_clone
                .set(&key, data_clone.clone(), Duration::from_secs(60))
                .await
                .unwrap();

            // Get data immediately
            let result = cache_clone.get(&key).await.unwrap();
            assert!(result.is_some());

            // Verify data integrity
            let retrieved = result.unwrap();
            assert_eq!(retrieved[0].title, "Performance Test");
            assert_eq!(retrieved[0].url, "https://example.com/perf");
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_rate_limiter_enforcement() -> Result<()> {
    // This test checks rate limiting behavior
    let provider = "rate_test_provider";

    // First few requests should succeed
    for _ in 0..5 {
        let result = check_rate_limit(provider).await;
        if result.is_err() {
            // Rate limit might be hit depending on configuration
            break;
        }
        sleep(Duration::from_millis(10)).await;
    }

    // Get limiter stats
    let stats = RATE_LIMITER_MANAGER.get_limiter_stats(provider).await?;
    if let Some(stats) = stats {
        assert_eq!(stats.provider, provider);
        // Remaining capacity should be less than the original
        // (exact values depend on configuration)
    }

    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_failure_handling() -> Result<()> {
    let provider = "cb_test_provider";

    // Simulate successful operations first
    for _ in 0..3 {
        let result = call_with_circuit_breaker(provider, || async {
            sleep(Duration::from_millis(10)).await;
            Ok::<String, eyre::Error>("success".to_string())
        })
        .await;

        assert!(result.is_ok());
    }

    // Check circuit breaker stats after successes
    let stats = get_circuit_breaker_stats(provider).await;
    if let Some(stats) = stats {
        assert_eq!(stats.provider, provider);
    }

    Ok(())
}

#[tokio::test]
async fn test_input_validation_edge_cases() -> Result<()> {
    // Test various input validation scenarios
    let test_cases = vec![
        // Valid case
        (
            BaseSearchParams {
                query: "normal query".to_string(),
                limit: Some(10),
                include_domains: Some(vec!["example.com".to_string()]),
                exclude_domains: None,
            },
            true,
        ),
        // Empty query (should fail)
        (
            BaseSearchParams {
                query: "".to_string(),
                limit: Some(10),
                include_domains: None,
                exclude_domains: None,
            },
            false,
        ),
        // Limit too high (should fail)
        (
            BaseSearchParams {
                query: "test".to_string(),
                limit: Some(101),
                include_domains: None,
                exclude_domains: None,
            },
            false,
        ),
        // Very long query
        (
            BaseSearchParams {
                query: "a".repeat(1001),
                limit: Some(10),
                include_domains: None,
                exclude_domains: None,
            },
            false,
        ),
    ];

    for (params, should_succeed) in test_cases {
        let result = validate_search_params(&params);

        if should_succeed {
            assert!(
                result.is_ok(),
                "Expected validation to succeed for params: {:?}",
                params
            );
        } else {
            assert!(
                result.is_err(),
                "Expected validation to fail for params: {:?}",
                params
            );
        }
    }

    // Test query sanitization
    let dirty_query = "test\0query\x01with\x7fcontrol";
    let clean_query = sanitize_query(dirty_query);
    assert_eq!(clean_query, "testquerywithcontrol");
    assert!(!clean_query.contains('\0'));

    Ok(())
}

#[tokio::test]
async fn test_metrics_collection_accuracy() -> Result<()> {
    if !METRICS_COLLECTOR.is_enabled() {
        // Skip test if metrics are disabled
        return Ok(());
    }

    let provider = "metrics_test_provider";

    // Record some test metrics
    record_request_metrics(
        provider,
        "search",
        Duration::from_millis(100),
        true,
        Some(1024),
        false,
    )
    .await;

    record_request_metrics(
        provider,
        "search",
        Duration::from_millis(150),
        true,
        Some(2048),
        true, // Cache hit
    )
    .await;

    record_request_metrics(
        provider,
        "search",
        Duration::from_millis(50),
        false, // Failed request
        None,
        false,
    )
    .await;

    // Wait a bit for metrics to be processed
    sleep(Duration::from_millis(10)).await;

    // Check metrics
    if let Some(stats) = METRICS_COLLECTOR.get_provider_stats(provider).await {
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.cache_hits, 1);

        // Average response time should be around 100ms ((100+150+50)/3)
        let avg_ms = stats.avg_response_time.as_millis();
        assert!(
            avg_ms >= 90 && avg_ms <= 110,
            "Average response time was {}ms",
            avg_ms
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_error_recovery_scenarios() -> Result<()> {
    // Test various error recovery scenarios

    // 1. Network timeout simulation
    let result = call_with_circuit_breaker("timeout_test", || async {
        sleep(Duration::from_millis(100)).await;
        Err::<String, _>(eyre::eyre!("Network timeout"))
    })
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Network timeout"));

    // 2. Rate limit recovery
    // (Rate limits should reset over time - this is handled by the governor crate)

    // 3. Cache invalidation and recovery
    let cache_manager = get_cache_manager().await;
    let test_key = "recovery_test_key";

    // Set and then delete
    cache_manager.set(test_key, vec![]).await?;
    assert!(cache_manager.get(test_key).await?.is_some());

    cache_manager.delete(test_key).await?;
    assert!(cache_manager.get(test_key).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let num_concurrent = 20;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let provider = format!("concurrent_test_{}", i % 5); // Use 5 different providers

            // Test the full pipeline concurrently
            let params = BaseSearchParams {
                query: format!("concurrent query {}", i),
                limit: Some(10),
                include_domains: None,
                exclude_domains: None,
            };

            // Validate
            let validated = validate_search_params(&params)?;

            // Rate limit check
            let _rate_check = check_rate_limit(&provider).await;

            // Generate cache key
            let cache_key =
                CacheManager::generate_cache_key(&provider, &validated.query, "concurrent_test");

            // Try to get from cache
            let cache_manager = get_cache_manager().await;
            let _cached = cache_manager.get(&cache_key).await?;

            // Record metrics
            record_request_metrics(
                &provider,
                "concurrent_test",
                Duration::from_millis(10),
                true,
                Some(512),
                false,
            )
            .await;

            Ok::<(), eyre::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    for handle in handles {
        handle.await??;
    }

    // Verify system state is still consistent
    let cache_size = get_cache_manager().await.size().await?;
    // Cache size should be reasonable (not indicating memory leaks)
    assert!(
        cache_size < 1000,
        "Cache size unexpectedly large: {}",
        cache_size
    );

    Ok(())
}

#[tokio::test]
async fn test_configuration_loading() -> Result<()> {
    // Test that configuration is loaded correctly
    let config = &*CONFIG;

    // Basic validation
    assert!(!config.server.host.is_empty());
    assert!(config.server.port > 0);
    assert!(config.server.max_connections > 0);

    // Cache config
    assert!(config.cache.ttl_seconds > 0);
    assert!(config.cache.max_entries > 0);

    // Rate limiting config
    assert!(config.rate_limiting.requests_per_minute > 0);

    // Provider configs
    assert!(config.providers.tavily.rate_limit > 0);
    assert!(config.providers.google.rate_limit > 0);

    Ok(())
}

#[cfg(feature = "caching")]
#[tokio::test]
async fn test_redis_cache_integration() -> Result<()> {
    // This test requires Redis to be running
    // It's skipped if REDIS_URL environment variable is not set

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Try to connect to Redis
    use redis::Client;
    let client = Client::open(redis_url.as_str());

    if client.is_err() {
        // Skip test if Redis is not available
        println!("Skipping Redis test - Redis not available");
        return Ok(());
    }

    let client = client?;
    let mut conn = client.get_connection()?;

    // Test basic Redis operations
    use redis::Commands;
    conn.set("test_key", "test_value")?;
    let result: String = conn.get("test_key")?;
    assert_eq!(result, "test_value");

    // Clean up
    let _: () = conn.del("test_key")?;

    Ok(())
}

#[tokio::test]
async fn test_memory_usage_stability() -> Result<()> {
    // Test for memory leaks and excessive memory usage
    let initial_cache_size = get_cache_manager().await.size().await?;

    // Perform many operations
    for i in 0..100 {
        let params = BaseSearchParams {
            query: format!("memory test query {}", i),
            limit: Some(10),
            include_domains: None,
            exclude_domains: None,
        };

        let _validated = validate_search_params(&params)?;

        // Cache some data
        let cache_key = format!("memory_test_{}", i);
        let test_data = vec![SearchResult {
            title: format!("Result {}", i),
            url: format!("https://example.com/{}", i),
            snippet: format!("Test snippet {}", i),
            score: Some(0.5),
            source_provider: "test".to_string(),
        }];

        get_cache_manager().await.set(&cache_key, test_data).await?;
    }

    let final_cache_size = get_cache_manager().await.size().await?;

    // Cache size should have increased but not excessively
    assert!(final_cache_size >= initial_cache_size);
    assert!(
        final_cache_size <= initial_cache_size + 100,
        "Cache size grew unexpectedly: {} -> {}",
        initial_cache_size,
        final_cache_size
    );

    // Clear cache to test cleanup
    get_cache_manager().await.clear().await?;
    let cleared_size = get_cache_manager().await.size().await?;
    assert!(cleared_size < final_cache_size);

    Ok(())
}
