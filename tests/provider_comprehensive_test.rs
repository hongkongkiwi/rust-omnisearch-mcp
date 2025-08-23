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
fn test_all_provider_construction() {
    // Test that all providers can be constructed without panicking
    let tavily = TavilySearchProvider::new();
    let google = GoogleCustomSearchProvider::new();
    let reddit = RedditSearchProvider::new();
    let duckduckgo = DuckDuckGoSearchProvider::new();
    let baidu = BaiduSearchProvider::new();
    let brightdata = BrightDataSearchProvider::new();
    let exa = ExaSearchProvider::new();

    // Verify all providers have unique names
    let names = [
        tavily.name(),
        google.name(),
        reddit.name(),
        duckduckgo.name(),
        baidu.name(),
        brightdata.name(),
        exa.name(),
    ];

    let unique_names: std::collections::HashSet<_> = names.iter().collect();
    assert_eq!(
        unique_names.len(),
        names.len(),
        "All provider names should be unique"
    );

    // Verify expected names
    assert_eq!(tavily.name(), "tavily");
    assert_eq!(google.name(), "google_custom_search");
    assert_eq!(reddit.name(), "reddit");
    assert_eq!(duckduckgo.name(), "duckduckgo");
    assert_eq!(baidu.name(), "baidu");
    assert_eq!(brightdata.name(), "brightdata");
    assert_eq!(exa.name(), "exa");

    assert!(true);
}

#[test]
fn test_all_provider_descriptions() {
    // Test that all providers have meaningful descriptions
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

        assert!(
            !description.is_empty(),
            "Provider {} should have a non-empty description",
            name
        );
        assert!(
            description.len() > 20,
            "Provider {} description should be meaningful (got: {} chars)",
            name,
            description.len()
        );

        // Each provider should mention its key characteristics in the description
        match name {
            "tavily" => {
                assert!(
                    description.contains("Tavily")
                        || description.contains("factual")
                        || description.contains("reliable")
                );
            }
            "google_custom_search" => {
                assert!(
                    description.contains("Google")
                        || description.contains("Custom Search")
                        || description.contains("reliable")
                );
            }
            "reddit" => {
                assert!(
                    description.contains("Reddit")
                        || description.contains("OAuth2")
                        || description.contains("discussions")
                );
            }
            "duckduckgo" => {
                assert!(
                    description.contains("DuckDuckGo")
                        || description.contains("privacy")
                        || description.contains("tracking")
                );
            }
            "baidu" => {
                assert!(
                    description.contains("Baidu")
                        || description.contains("China")
                        || description.contains("SerpApi")
                );
            }
            "brightdata" => {
                assert!(
                    description.contains("Bright Data")
                        || description.contains("SERP")
                        || description.contains("filtering")
                );
            }
            "exa" => {
                assert!(
                    description.contains("Exa")
                        || description.contains("scores")
                        || description.contains("relevance")
                );
            }
            _ => {
                // For any other provider, just ensure it has a reasonable description
                assert!(description.len() > 10);
            }
        }
    }
}

#[test]
fn test_provider_traits() {
    // Test that all providers implement the required traits correctly
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
        // Test name method
        assert!(
            !provider.name().is_empty(),
            "Provider name should not be empty"
        );
        assert!(
            !provider.name().contains(" "),
            "Provider names should not contain spaces"
        );

        // Test description method
        assert!(
            !provider.description().is_empty(),
            "Provider description should not be empty"
        );
        assert!(
            provider.description().len() > 10,
            "Provider description should be meaningful"
        );
    }
}
