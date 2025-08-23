use omnisearch_mcp::{
    common::types::{BaseSearchParams, SearchProvider},
    providers::brightdata::BrightDataSearchProvider,
};

#[tokio::test]
async fn test_brightdata_provider_search() {
    let provider = BrightDataSearchProvider::new();

    // Test with a simple query
    let params = BaseSearchParams {
        query: "rust programming language".to_string(),
        limit: Some(3),
        include_domains: None,
        exclude_domains: None,
    };

    // Note: This test will fail if no BrightData credentials are configured
    // but it's still useful to verify the method signature and structure
    match provider.search(params).await {
        Ok(results) => {
            // If we get results, verify the structure
            for result in results {
                assert!(!result.title.is_empty());
                assert!(!result.url.is_empty());
                assert!(!result.snippet.is_empty());
                assert_eq!(result.source_provider, "brightdata");
            }
        }
        Err(e) => {
            // Check that we get an appropriate error for missing credentials
            assert!(
                e.message.contains("Missing Bright Data username")
                    || e.message.contains("Missing Bright Data password")
            );
        }
    }
}
