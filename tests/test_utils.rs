//! Common test utilities that can be included in test files

/// Create a standard test search parameters object
pub fn create_test_params(
    query: &str,
    limit: Option<u32>,
) -> omnisearch_mcp::common::types::BaseSearchParams {
    omnisearch_mcp::common::types::BaseSearchParams {
        query: query.to_string(),
        limit,
        include_domains: None,
        exclude_domains: None,
    }
}

/// Create test search parameters with domain filtering
pub fn create_test_params_with_domains(
    query: &str,
    limit: Option<u32>,
    include_domains: Option<Vec<String>>,
    exclude_domains: Option<Vec<String>>,
) -> omnisearch_mcp::common::types::BaseSearchParams {
    omnisearch_mcp::common::types::BaseSearchParams {
        query: query.to_string(),
        limit,
        include_domains,
        exclude_domains,
    }
}

/// Validate a search result has the required fields
pub fn validate_search_result(
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
