use omnisearch_mcp::{
    common::types::{BaseSearchParams, SearchProvider},
    providers::{
        baidu::BaiduSearchProvider, brightdata::BrightDataSearchProvider,
        duckduckgo::DuckDuckGoSearchProvider, exa::ExaSearchProvider,
        google::GoogleCustomSearchProvider, reddit::RedditSearchProvider,
    },
};

#[test]
fn test_google_provider_creation() {
    let provider = GoogleCustomSearchProvider::new();
    assert_eq!(provider.name(), "google_custom_search");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_reddit_provider_creation() {
    let provider = RedditSearchProvider::new();
    assert_eq!(provider.name(), "reddit");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_duckduckgo_provider_creation() {
    let provider = DuckDuckGoSearchProvider::new();
    assert_eq!(provider.name(), "duckduckgo");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_baidu_provider_creation() {
    let provider = BaiduSearchProvider::new();
    assert_eq!(provider.name(), "baidu");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_brightdata_provider_creation() {
    let provider = BrightDataSearchProvider::new();
    assert_eq!(provider.name(), "brightdata");
    assert!(!provider.description().is_empty());
}

#[test]
fn test_exa_provider_creation() {
    let provider = ExaSearchProvider::new();
    assert_eq!(provider.name(), "exa");
    assert!(!provider.description().is_empty());
}

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
    assert_eq!(
        params.include_domains,
        Some(vec!["example.com".to_string()])
    );
    assert_eq!(
        params.exclude_domains,
        Some(vec!["exclude.com".to_string()])
    );
}
