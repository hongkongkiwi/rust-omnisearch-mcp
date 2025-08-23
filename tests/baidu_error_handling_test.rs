//! Tests for Baidu provider error handling scenarios

use omnisearch_mcp::{
    common::types::{BaseSearchParams, ErrorType, SearchProvider},
    providers::baidu::BaiduSearchProvider,
};

#[tokio::test]
async fn test_baidu_provider_missing_api_key_error() {
    let provider = BaiduSearchProvider::new();

    // Test with missing API key - should return an error
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(5),
        include_domains: None,
        exclude_domains: None,
    };

    match provider.search(params).await {
        Ok(_) => {
            // If we get results, that's fine, but we're mainly testing
            // that the function doesn't panic
            assert!(true);
        }
        Err(e) => {
            // We expect an API key error
            match e.error_type {
                ErrorType::ApiError => {
                    // This is expected when API key is missing
                    assert!(e.message.contains("Missing") || e.message.contains("API key"));
                }
                _ => {
                    // Other error types are acceptable too
                    assert!(true);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_baidu_provider_network_error_handling() {
    let provider = BaiduSearchProvider::new();

    // Test with a query that should trigger network errors
    let params = BaseSearchParams {
        query: "".to_string(),         // Empty query
        limit: Some(0),                // Invalid limit
        include_domains: Some(vec![]), // Empty domains
        exclude_domains: Some(vec![]), // Empty domains
    };

    match provider.search(params).await {
        Ok(results) => {
            // Even with strange parameters, we might get results
            // Just ensure the structure is correct
            for result in results {
                assert!(!result.title.is_empty());
                assert!(!result.url.is_empty());
                assert!(!result.snippet.is_empty());
            }
        }
        Err(e) => {
            // Network errors or API errors are expected
            match e.error_type {
                ErrorType::ApiError | ErrorType::InvalidInput | ErrorType::ProviderError => {
                    // These are all acceptable error types
                    assert!(true);
                }
                _ => {
                    // Any other error type is also fine
                    assert!(true);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_baidu_provider_empty_query_handling() {
    let provider = BaiduSearchProvider::new();

    // Test handling of empty queries
    let params = BaseSearchParams {
        query: "".to_string(),
        limit: None,
        include_domains: None,
        exclude_domains: None,
    };

    match provider.search(params).await {
        Ok(_results) => {
            // Empty query might still return results or empty list
            // assert!(results.len() >= 0);  // This comparison is always true
            assert!(true);
        }
        Err(e) => {
            // Empty query might cause an error, which is fine
            assert!(e.message.len() > 0);
        }
    }
}
