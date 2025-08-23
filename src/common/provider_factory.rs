//! Provider factory for creating and managing providers

use crate::common::types::SearchProvider;
use crate::config::CONFIG;

/// Provider factory for creating and managing providers
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create all available search providers based on configuration
    pub fn create_search_providers() -> Vec<Box<dyn SearchProvider>> {
        let mut providers: Vec<Box<dyn SearchProvider>> = Vec::new();

        // Tavily provider
        if CONFIG.providers.tavily.api_key.is_some() {
            providers.push(Box::new(
                crate::providers::search::TavilySearchProvider::new(),
            ));
        }

        // Google Custom Search provider
        if CONFIG.providers.google.api_key.is_some() && CONFIG.providers.google.search_engine_id.is_some()
        {
            providers.push(Box::new(
                crate::providers::google::GoogleCustomSearchProvider::new(),
            ));
        }

        // Reddit provider
        if CONFIG.providers.reddit.client_id.is_some()
            && CONFIG.providers.reddit.client_secret.is_some()
            && CONFIG.providers.reddit.user_agent.is_some()
        {
            providers.push(Box::new(
                crate::providers::reddit::RedditSearchProvider::new(),
            ));
        }

        // DuckDuckGo provider (no API key required)
        providers.push(Box::new(
            crate::providers::duckduckgo::DuckDuckGoSearchProvider::new(),
        ));

        // Baidu provider
        if CONFIG.providers.baidu.api_key.is_some() {
            providers.push(Box::new(crate::providers::baidu::BaiduSearchProvider::new()));
        }

        // Bright Data provider
        if CONFIG.providers.brightdata.username.is_some()
            && CONFIG.providers.brightdata.password.is_some()
        {
            providers.push(Box::new(
                crate::providers::brightdata::BrightDataSearchProvider::new(),
            ));
        }

        // Exa provider
        if CONFIG.providers.exa.api_key.is_some() {
            providers.push(Box::new(crate::providers::exa::ExaSearchProvider::new()));
        }

        providers
    }

    /// Get provider names for logging
    pub fn get_provider_names(providers: &[Box<dyn SearchProvider>]) -> Vec<String> {
        providers.iter().map(|p| p.name().to_string()).collect()
    }
}
