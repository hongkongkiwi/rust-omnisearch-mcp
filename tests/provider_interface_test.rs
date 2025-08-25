use omnisearch_mcp::{
    common::types::SearchProvider,
    providers::{
        baidu::BaiduSearchProvider, brightdata::BrightDataSearchProvider,
        duckduckgo::DuckDuckGoSearchProvider, exa::ExaSearchProvider,
        google::GoogleCustomSearchProvider, reddit::RedditSearchProvider,
        search::TavilySearchProvider,
    },
};

#[test]
fn test_all_providers_implement_search_provider_trait() {
    // This test ensures all providers correctly implement the SearchProvider trait
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(TavilySearchProvider::new()),
        Box::new(GoogleCustomSearchProvider::new()),
        Box::new(RedditSearchProvider::new()),
        Box::new(DuckDuckGoSearchProvider::new()),
        Box::new(BaiduSearchProvider::new()),
        Box::new(BrightDataSearchProvider::new()),
        Box::new(ExaSearchProvider::new()),
    ];

    for provider in providers {
        // Test that all providers have names
        assert!(!provider.name().is_empty(), "Provider should have a name");

        // Test that all providers have descriptions
        assert!(
            !provider.description().is_empty(),
            "Provider should have a description"
        );
        assert!(
            provider.description().len() > 10,
            "Provider description should be meaningful"
        );

        // Test that names are unique (this is a basic check)
        assert!(
            !provider.name().contains(" "),
            "Provider names should not contain spaces"
        );
    }
}

#[test]
fn test_provider_names_are_unique() {
    let provider_names = [
        TavilySearchProvider::new().name(),
        GoogleCustomSearchProvider::new().name(),
        RedditSearchProvider::new().name(),
        DuckDuckGoSearchProvider::new().name(),
        BaiduSearchProvider::new().name(),
        BrightDataSearchProvider::new().name(),
        ExaSearchProvider::new().name(),
    ];

    // Check for uniqueness by comparing length of vector and set
    let unique_names: std::collections::HashSet<_> = provider_names.iter().collect();
    assert_eq!(
        unique_names.len(),
        provider_names.len(),
        "All provider names should be unique"
    );
}

#[test]
fn test_expected_provider_names() {
    assert_eq!(TavilySearchProvider::new().name(), "tavily");
    assert_eq!(
        GoogleCustomSearchProvider::new().name(),
        "google_custom_search"
    );
    assert_eq!(RedditSearchProvider::new().name(), "reddit");
    assert_eq!(DuckDuckGoSearchProvider::new().name(), "duckduckgo");
    assert_eq!(BaiduSearchProvider::new().name(), "baidu");
    assert_eq!(BrightDataSearchProvider::new().name(), "brightdata");
    assert_eq!(ExaSearchProvider::new().name(), "exa");
}

#[test]
fn test_provider_descriptions_contain_key_info() {
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(TavilySearchProvider::new()),
        Box::new(GoogleCustomSearchProvider::new()),
        Box::new(RedditSearchProvider::new()),
        Box::new(DuckDuckGoSearchProvider::new()),
        Box::new(BaiduSearchProvider::new()),
        Box::new(BrightDataSearchProvider::new()),
        Box::new(ExaSearchProvider::new()),
    ];

    for provider in providers {
        let name = provider.name();
        let description = provider.description();

        // Each provider's description should contain its name or related keywords
        match name {
            "tavily" => {
                assert!(description.contains("Tavily") || description.contains("factual"));
            }
            "google_custom_search" => {
                assert!(description.contains("Google") || description.contains("Custom Search"));
            }
            "reddit" => {
                assert!(description.contains("Reddit"));
            }
            "duckduckgo" => {
                assert!(description.contains("DuckDuckGo") || description.contains("privacy"));
            }
            "baidu" => {
                assert!(description.contains("Baidu") || description.contains("China"));
            }
            "brightdata" => {
                assert!(description.contains("Bright Data") || description.contains("SERP"));
            }
            "exa" => {
                assert!(description.contains("Exa") || description.contains("score"));
            }
            _ => {
                // For any other provider, just ensure it has a meaningful description
                assert!(description.len() > 10);
            }
        }
    }
}
