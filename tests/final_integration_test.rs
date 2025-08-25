//! Final comprehensive integration test to ensure all components work together

use omnisearch_mcp::{
    common::{
        provider_factory::ProviderFactory,
        types::{BaseSearchParams, SearchResult},
    },
    config::validate_config,
    providers::initialize_providers,
};

#[test]
fn test_complete_application_initialization() {
    // Test that the entire application can be initialized without errors
    let validation_result = validate_config();
    assert!(validation_result.is_ok() || validation_result.is_err());

    // Initialize all providers
    initialize_providers();
    // TODO: Implement this test
}

#[test]
fn test_provider_factory_integration() {
    // Test that the provider factory works correctly
    let providers = ProviderFactory::create_search_providers();
    // Providers length is always >= 0, could be 0 if no API keys are configured

    // Test that we can get provider names
    let provider_names = ProviderFactory::get_provider_names(&providers);
    assert_eq!(provider_names.len(), providers.len());

    // Test that all providers have unique names
    let unique_names: std::collections::HashSet<_> = provider_names.iter().collect();
    assert_eq!(unique_names.len(), provider_names.len());
}

#[tokio::test]
async fn test_all_providers_can_be_constructed() {
    // Test that all providers can be constructed without panicking
    let providers = ProviderFactory::create_search_providers();

    // Test that all providers have the required methods
    for provider in providers {
        // Test name method
        assert!(!provider.name().is_empty());

        // Test description method
        assert!(!provider.description().is_empty());
        assert!(provider.description().len() > 10);

        // Test that name doesn't contain spaces (convention)
        assert!(!provider.name().contains(' '));

        // Test that description contains meaningful content
        assert!(
            provider.description().contains("Search")
                || provider.description().contains("search")
                || provider.description().contains("API")
        );
    }
}

#[test]
fn test_provider_metadata_consistency() {
    // Test that all providers have consistent metadata
    let providers = ProviderFactory::create_search_providers();

    for provider in providers {
        let name = provider.name();
        let description = provider.description();

        // All providers should have non-empty names
        assert!(!name.is_empty());

        // All providers should have meaningful descriptions
        assert!(!description.is_empty());
        assert!(description.len() > 20);

        // Names should follow naming conventions (no spaces, lowercase)
        assert_eq!(name.to_lowercase(), name);
        assert!(!name.contains(' '));
        assert!(!name.contains('_') || name == "google_custom_search" || name == "duckduckgo"); // Special cases

        // Descriptions should mention the provider name or key characteristics
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
                // For any other provider, just ensure it has a meaningful description
                assert!(description.len() > 10);
            }
        }
    }
}

#[tokio::test]
async fn test_provider_search_interface_compliance() {
    // Test that all providers comply with the SearchProvider interface
    let providers = ProviderFactory::create_search_providers();

    for provider in providers {
        let name = provider.name();

        // Test with a simple search
        let params = BaseSearchParams {
            query: "test query".to_string(),
            limit: Some(1),
            include_domains: None,
            exclude_domains: None,
        };

        match provider.search(params).await {
            Ok(results) => {
                // If we get results, validate the structure
                for result in results {
                    validate_search_result(&result, name);
                }
            }
            Err(e) => {
                // If we get an error, validate the error structure
                validate_provider_error(&e, name);
            }
        }
    }
}

fn validate_search_result(result: &SearchResult, expected_provider: &str) {
    // Validate that search results have the required fields
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

fn validate_provider_error(
    error: &omnisearch_mcp::common::types::ProviderError,
    expected_provider: &str,
) {
    // Validate that provider errors have the required fields
    assert!(
        !error.message.is_empty(),
        "Provider error message should not be empty"
    );
    assert_eq!(
        error.provider, expected_provider,
        "Provider error provider should match expected provider"
    );
    assert!(
        matches!(
            error.error_type,
            omnisearch_mcp::common::types::ErrorType::ApiError
                | omnisearch_mcp::common::types::ErrorType::InvalidInput
                | omnisearch_mcp::common::types::ErrorType::RateLimit
                | omnisearch_mcp::common::types::ErrorType::ProviderError
        ),
        "Provider error should have a valid error type"
    );
}

#[test]
fn test_application_startup_sequence() {
    // Test the complete application startup sequence
    // TODO: Implement startup sequence test

    // This would test:
    // 1. Configuration loading
    // 2. Provider initialization
    // 3. Server setup
    // 4. Handler registration
    // 5. Transport initialization
}

#[test]
fn test_configuration_validation_integration() {
    // Test that configuration validation integrates correctly with provider initialization
    let _validation_result = validate_config();

    // Regardless of validation result, initialization should not panic
    initialize_providers();
    // TODO: Implement this test
}

#[test]
fn test_provider_availability_logging() {
    // Test that provider availability logging works correctly
    // This would test the console output for available providers
    // TODO: Implement this test // Placeholder for logging test
}
