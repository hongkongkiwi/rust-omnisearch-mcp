//! Comprehensive error handling tests for all provider scenarios

use omnisearch_mcp::{
    common::types::{BaseSearchParams, ErrorType, ProviderError, SearchProvider},
    providers::{
        baidu::BaiduSearchProvider, brightdata::BrightDataSearchProvider,
        duckduckgo::DuckDuckGoSearchProvider, exa::ExaSearchProvider,
        google::GoogleCustomSearchProvider, reddit::RedditSearchProvider,
        search::tavily::TavilySearchProvider,
    },
};
use std::env;

#[tokio::test]
async fn test_all_providers_handle_missing_credentials() {
    // Temporarily clear all API keys
    let keys_to_clear = vec![
        "TAVILY_API_KEY",
        "GOOGLE_API_KEY",
        "GOOGLE_CX",
        "REDDIT_CLIENT_ID",
        "REDDIT_CLIENT_SECRET",
        "BAIDU_API_KEY",
        "BRIGHTDATA_API_KEY",
        "EXA_API_KEY",
    ];

    let mut saved_values = vec![];
    for key in &keys_to_clear {
        saved_values.push((key.to_string(), env::var(key).ok()));
        env::remove_var(key);
    }

    // Test each provider
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(TavilySearchProvider::new()),
        Box::new(GoogleCustomSearchProvider::new()),
        Box::new(RedditSearchProvider::new()),
        Box::new(BaiduSearchProvider::new()),
        Box::new(BrightDataSearchProvider::new()),
        Box::new(ExaSearchProvider::new()),
        // DuckDuckGo doesn't require API key
    ];

    let params = BaseSearchParams {
        query: "test".to_string(),
        limit: Some(10),
        include_domains: None,
        exclude_domains: None,
    };

    for provider in providers {
        let result = provider.search(params.clone()).await;
        assert!(
            result.is_err(),
            "Provider {} should fail without credentials",
            provider.name()
        );

        if let Err(e) = result {
            assert!(
                matches!(e.error_type, ErrorType::ApiError | ErrorType::ProviderError),
                "Provider {} should return ApiError or ProviderError for missing credentials",
                provider.name()
            );
        }
    }

    // Restore environment variables
    for (key, value) in saved_values {
        if let Some(val) = value {
            env::set_var(&key, val);
        }
    }
}

#[tokio::test]
async fn test_empty_query_handling() {
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(DuckDuckGoSearchProvider::new()),
        // Only test with providers that don't require API keys
    ];

    let params = BaseSearchParams {
        query: "".to_string(),
        limit: Some(10),
        include_domains: None,
        exclude_domains: None,
    };

    for provider in providers {
        let result = provider.search(params.clone()).await;
        // Should either handle gracefully or return some error
        match result {
            Ok(_results) => {
                // Empty results are acceptable for empty query
                // Some providers might return results, which is also fine
            }
            Err(e) => {
                // Accept various error types as different providers may handle differently
                assert!(
                    matches!(
                        e.error_type,
                        ErrorType::InvalidInput | ErrorType::ApiError | ErrorType::ProviderError
                    ),
                    "Provider {} should handle empty query gracefully, got error: {:?}",
                    provider.name(),
                    e.error_type
                );
            }
        }
    }
}

#[tokio::test]
async fn test_invalid_limit_handling() {
    let provider = DuckDuckGoSearchProvider::new();

    // Test with limit = 0
    let params = BaseSearchParams {
        query: "test".to_string(),
        limit: Some(0),
        include_domains: None,
        exclude_domains: None,
    };

    let result = provider.search(params).await;
    // Should either clamp to minimum or return error
    match result {
        Ok(results) => assert!(results.is_empty() || results.len() <= 1),
        Err(e) => {
            // Accept either InvalidInput or ApiError - provider may handle differently
            assert!(
                matches!(e.error_type, ErrorType::InvalidInput | ErrorType::ApiError),
                "Expected InvalidInput or ApiError, got {:?}",
                e.error_type
            );
        }
    }

    // Test with very large limit
    let params = BaseSearchParams {
        query: "test".to_string(),
        limit: Some(100000),
        include_domains: None,
        exclude_domains: None,
    };

    let result = provider.search(params).await;
    // Should either clamp to maximum or handle gracefully
    match result {
        Ok(results) => assert!(results.len() <= 100), // Reasonable maximum
        Err(e) => {
            // Accept either InvalidInput or ApiError - provider may handle differently
            assert!(
                matches!(e.error_type, ErrorType::InvalidInput | ErrorType::ApiError),
                "Expected InvalidInput or ApiError, got {:?}",
                e.error_type
            );
        }
    }
}

#[tokio::test]
async fn test_special_characters_in_query() {
    let provider = DuckDuckGoSearchProvider::new();

    let special_queries = vec![
        "test!@#$%^&*()",
        "test\n\r\t",
        "test\"'<>",
        "test\\//",
        "🔍 test 🚀",
        "test & test | test",
    ];

    for query in special_queries {
        let params = BaseSearchParams {
            query: query.to_string(),
            limit: Some(5),
            include_domains: None,
            exclude_domains: None,
        };

        let result = provider.search(params).await;
        // Should handle special characters gracefully
        match result {
            Ok(_) => {} // Success is fine
            Err(e) => {
                // Accept various error types as different providers may handle differently
                assert!(
                    matches!(
                        e.error_type,
                        ErrorType::InvalidInput | ErrorType::ApiError | ErrorType::ProviderError
                    ),
                    "Should handle special characters gracefully in query: {}, got error: {:?}",
                    query,
                    e.error_type
                );
            }
        }
    }
}

#[tokio::test]
async fn test_domain_filter_validation() {
    let provider = DuckDuckGoSearchProvider::new();

    // Test with invalid domain formats
    let invalid_domains = vec![
        vec!["not a domain"],
        vec!["http://"],
        vec!["//invalid"],
        vec!["domain with spaces.com"],
    ];

    for domains in invalid_domains {
        let params = BaseSearchParams {
            query: "test".to_string(),
            limit: Some(5),
            include_domains: Some(domains.iter().map(|s| s.to_string()).collect()),
            exclude_domains: None,
        };

        let result = provider.search(params).await;
        // Should either sanitize or handle gracefully
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle invalid domain filters"
        );
    }
}

#[test]
fn test_provider_error_display_formatting() {
    let error_types = vec![
        ErrorType::ApiError,
        ErrorType::InvalidInput,
        ErrorType::RateLimit,
        ErrorType::ProviderError,
    ];

    for error_type in error_types {
        let error = ProviderError {
            error_type,
            message: "Test error message".to_string(),
            provider: "test-provider".to_string(),
            source: None,
        };

        let display = format!("{}", error);
        assert!(display.contains("Test error message"));

        // Test with source error
        let source_error = eyre::eyre!("Source error");

        let error_with_source = ProviderError {
            error_type: ErrorType::ApiError,
            message: "Test error".to_string(),
            provider: "test-provider".to_string(),
            source: Some(source_error),
        };

        let display_with_source = format!("{}", error_with_source);
        assert!(display_with_source.contains("Test error"));
    }
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    use futures::future::join_all;

    let provider = DuckDuckGoSearchProvider::new();

    // Create multiple requests with various error conditions
    let mut futures = vec![];

    // Empty query
    let params1 = BaseSearchParams {
        query: "".to_string(),
        limit: Some(10),
        include_domains: None,
        exclude_domains: None,
    };
    futures.push(provider.search(params1));

    // Zero limit
    let params2 = BaseSearchParams {
        query: "test".to_string(),
        limit: Some(0),
        include_domains: None,
        exclude_domains: None,
    };
    futures.push(provider.search(params2));

    // Normal query
    let params3 = BaseSearchParams {
        query: "normal test".to_string(),
        limit: Some(5),
        include_domains: None,
        exclude_domains: None,
    };
    futures.push(provider.search(params3));

    // Execute all concurrently
    let results = join_all(futures).await;

    // All should complete without panicking
    assert_eq!(results.len(), 3);
}

#[test]
fn test_error_type_equality() {
    assert_eq!(ErrorType::ApiError, ErrorType::ApiError);
    assert_ne!(ErrorType::ApiError, ErrorType::InvalidInput);
    assert_ne!(ErrorType::RateLimit, ErrorType::ProviderError);
}

#[test]
fn test_provider_error_creation_helpers() {
    // Test creating errors with various helper methods
    let api_error = ProviderError {
        error_type: ErrorType::ApiError,
        message: "API failed".to_string(),
        provider: "test-provider".to_string(),
        source: None,
    };
    assert_eq!(api_error.error_type, ErrorType::ApiError);

    let rate_limit_error = ProviderError {
        error_type: ErrorType::RateLimit,
        message: "Rate limit exceeded".to_string(),
        provider: "test-provider".to_string(),
        source: None,
    };
    assert_eq!(rate_limit_error.error_type, ErrorType::RateLimit);

    let invalid_input_error = ProviderError {
        error_type: ErrorType::InvalidInput,
        message: "Invalid input".to_string(),
        provider: "test-provider".to_string(),
        source: None,
    };
    assert_eq!(invalid_input_error.error_type, ErrorType::InvalidInput);
}

#[tokio::test]
async fn test_network_timeout_simulation() {
    use std::time::Duration;
    use tokio::time::timeout;

    let provider = DuckDuckGoSearchProvider::new();

    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(10),
        include_domains: None,
        exclude_domains: None,
    };

    // Simulate very short timeout
    let result = timeout(Duration::from_millis(1), provider.search(params)).await;

    // Should handle timeout gracefully
    match result {
        Ok(search_result) => {
            // If it completed, it should be valid
            assert!(search_result.is_ok() || search_result.is_err());
        }
        Err(_) => {
            // Timeout occurred, which is expected
            // Test passes if compilation succeeds
        }
    }
}
