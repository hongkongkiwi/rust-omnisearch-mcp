use omnisearch_mcp::{
    common::types::{BaseSearchParams, SearchProvider},
    providers::google::GoogleCustomSearchProvider,
};

fn create_test_params(query: &str, limit: Option<u32>) -> BaseSearchParams {
    BaseSearchParams {
        query: query.to_string(),
        limit,
        include_domains: None,
        exclude_domains: None,
    }
}

fn validate_search_result(
    result: &omnisearch_mcp::common::types::SearchResult,
    expected_provider: &str,
) {
    assert!(
        !result.title.is_empty(),
        "Search result title should not be empty"
    );
    assert!(
        !result.url.is_empty(),
        "Search result URL should not be empty"
    );
    assert!(
        !result.snippet.is_empty(),
        "Search result snippet should not be empty"
    );
    assert_eq!(
        result.source_provider, expected_provider,
        "Search result source provider should match expected provider"
    );

    // Score is optional, so we don't validate it unless it's present
    if let Some(score) = result.score {
        assert!(
            score >= 0.0,
            "Search result score should be non-negative if present"
        );
    }
}

#[tokio::test]
async fn test_google_provider_search() {
    let provider = GoogleCustomSearchProvider::new();

    // Test with a simple query
    let params = create_test_params("rust programming language", Some(3));

    // Note: This test will fail if no Google API key is configured
    // but it's still useful to verify the method signature and structure
    match provider.search(params).await {
        Ok(results) => {
            // If we get results, verify the structure
            assert!(
                !results.is_empty(),
                "Should return at least one search result"
            );
            for result in results {
                validate_search_result(&result, "google_custom_search");
            }
        }
        Err(e) => {
            // Check that we get an appropriate error for missing credentials
            assert!(
                e.message.contains("Missing Google API key")
                    || e.message.contains("Missing Google Custom Search Engine ID")
            );
        }
    }
}

#[test]
fn test_google_provider_creation() {
    let provider = GoogleCustomSearchProvider::new();
    assert_eq!(provider.name(), "google_custom_search");
    assert!(!provider.description().is_empty());
    assert!(provider.description().contains("Google Custom Search API"));
}
