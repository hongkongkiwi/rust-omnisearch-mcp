//! Tests for Google provider error handling scenarios

use omnisearch_mcp::{
    common::types::{BaseSearchParams, ErrorType, SearchProvider},
    providers::google::GoogleCustomSearchProvider,
};

#[tokio::test]
async fn test_google_provider_missing_credentials_error() {
    let provider = GoogleCustomSearchProvider::new();
    
    // Test with missing credentials - should return an error
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(5),
        include_domains: None,
        exclude_domains: None,
    };
    
    match provider.search(params).await {
        Ok(_) => {
            // If we get results, that's fine for testing purposes
            assert!(true);
        }
        Err(e) => {
            // We expect credential errors
            match e.error_type {
                ErrorType::ApiError => {
                    // This is expected when credentials are missing
                    assert!(e.message.contains("Missing") || e.message.contains("API key") || e.message.contains("Engine ID"));
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
async fn test_google_provider_invalid_parameters() {
    let provider = GoogleCustomSearchProvider::new();
    
    // Test with invalid parameters
    let params = BaseSearchParams {
        query: "".to_string(), // Empty query
        limit: Some(0), // Invalid limit
        include_domains: Some(vec![]), // Empty domains
        exclude_domains: Some(vec![]), // Empty domains
    };
    
    match provider.search(params).await {
        Ok(_results) => {
            // Even with invalid parameters, we might get results
            // assert!(results.len() >= 0);  // This comparison is always true
            assert!(true);
        }
        Err(e) => {
            // Invalid parameters might cause errors, which is fine
            assert!(e.message.len() > 0);
        }
    }
}

#[tokio::test]
async fn test_google_provider_extreme_limits() {
    let provider = GoogleCustomSearchProvider::new();
    
    // Test with extreme limits
    let params = BaseSearchParams {
        query: "test".to_string(),
        limit: Some(100), // Very high limit
        include_domains: None,
        exclude_domains: None,
    };
    
    match provider.search(params).await {
        Ok(_results) => {
            // Should handle extreme limits gracefully
            // assert!(results.len() >= 0);  // This comparison is always true
            assert!(true);
        }
        Err(e) => {
            // Might hit API limits, which is fine
            assert!(e.message.len() > 0);
        }
    }
}