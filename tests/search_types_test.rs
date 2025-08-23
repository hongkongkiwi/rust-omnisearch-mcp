use omnisearch_mcp::common::types::{BaseSearchParams, SearchResult};

#[test]
fn test_search_params_creation() {
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(10),
        include_domains: Some(vec!["example.com".to_string()]),
        exclude_domains: Some(vec!["exclude.com".to_string()]),
    };
    
    assert_eq!(params.query, "test query");
    assert_eq!(params.limit, Some(10));
    assert_eq!(params.include_domains, Some(vec!["example.com".to_string()]));
    assert_eq!(params.exclude_domains, Some(vec!["exclude.com".to_string()]));
}

#[test]
fn test_search_params_with_none_values() {
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: None,
        include_domains: None,
        exclude_domains: None,
    };
    
    assert_eq!(params.query, "test query");
    assert_eq!(params.limit, None);
    assert_eq!(params.include_domains, None);
    assert_eq!(params.exclude_domains, None);
}

#[test]
fn test_search_params_with_empty_domains() {
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(5),
        include_domains: Some(vec![]),
        exclude_domains: Some(vec![]),
    };
    
    assert_eq!(params.query, "test query");
    assert_eq!(params.limit, Some(5));
    assert_eq!(params.include_domains, Some(vec![]));
    assert_eq!(params.exclude_domains, Some(vec![]));
}

#[test]
fn test_search_result_creation() {
    let result = SearchResult {
        title: "Test Title".to_string(),
        url: "https://example.com".to_string(),
        snippet: "Test snippet content".to_string(),
        score: Some(0.85),
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "Test snippet content");
    assert_eq!(result.score, Some(0.85));
    assert_eq!(result.source_provider, "test_provider");
}

#[test]
fn test_search_result_without_score() {
    let result = SearchResult {
        title: "Test Title".to_string(),
        url: "https://example.com".to_string(),
        snippet: "Test snippet content".to_string(),
        score: None,
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "Test snippet content");
    assert_eq!(result.score, None);
    assert_eq!(result.source_provider, "test_provider");
}