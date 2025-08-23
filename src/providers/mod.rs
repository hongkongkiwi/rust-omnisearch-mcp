use crate::{common::provider_factory::ProviderFactory, config::CONFIG};

#[cfg(feature = "server")]
use crate::server::register_search_provider;

use crate::common::types::SearchProvider;

// Import search providers
pub mod baidu;
pub mod brightdata;
pub mod duckduckgo;
pub mod exa;
pub mod google;
pub mod reddit;
pub mod search;

/// Create and return available search providers (for library usage)
pub fn create_providers() -> Vec<Box<dyn SearchProvider>> {
    ProviderFactory::create_search_providers()
}

/// Initialize providers and register them with the MCP server (for server usage)
#[cfg(feature = "server")]
pub fn initialize_providers() {
    // Initialize search providers using the factory
    let search_providers = create_providers();

    for provider in search_providers {
        // All our new providers are regular search providers (not AI response providers)
        register_search_provider(provider, false);
    }

    // Initialize AI response providers (using appropriate provider configs)
    if CONFIG.providers.perplexity.api_key.is_some() {
        // register_search_provider(Box::new(PerplexityProvider::new()), true);
    }

    if CONFIG.providers.kagi.api_key.is_some() {
        // register_search_provider(Box::new(KagiFastGPTProvider::new()), true);
    }

    // Initialize processing providers (using appropriate provider configs)
    if CONFIG.providers.jina.api_key.is_some() {
        // register_processing_provider(Box::new(JinaReaderProvider::new()));
    }

    if CONFIG.providers.kagi.api_key.is_some() {
        // register_processing_provider(Box::new(KagiSummarizerProvider::new()));
    }

    if CONFIG.providers.tavily.api_key.is_some() {
        // register_processing_provider(Box::new(TavilyExtractProvider::new()));
    }

    if CONFIG.providers.firecrawl.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlScrapeProvider::new()));
        // register_processing_provider(Box::new(FirecrawlCrawlProvider::new()));
        // register_processing_provider(Box::new(FirecrawlMapProvider::new()));
        // register_processing_provider(Box::new(FirecrawlExtractProvider::new()));
        // register_processing_provider(Box::new(FirecrawlActionsProvider::new()));
    }

    // Initialize enhancement providers (using appropriate provider configs)
    if CONFIG.providers.jina.api_key.is_some() {
        // register_enhancement_provider(Box::new(JinaGroundingProvider::new()));
    }

    if CONFIG.providers.kagi.api_key.is_some() {
        // register_enhancement_provider(Box::new(KagiEnrichmentProvider::new()));
    }

    // Log available providers
    println!("Available providers:");

    let search_providers = &*crate::server::tools::AVAILABLE_PROVIDERS
        .search
        .read()
        .unwrap();
    if !search_providers.is_empty() {
        println!(
            "- Search: {}",
            search_providers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    } else {
        println!("- Search: None available (missing API keys)");
    }

    let ai_response_providers = &*crate::server::tools::AVAILABLE_PROVIDERS
        .ai_response
        .read()
        .unwrap();
    if !ai_response_providers.is_empty() {
        println!(
            "- AI Response: {}",
            ai_response_providers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    } else {
        println!("- AI Response: None available (missing API keys)");
    }

    let processing_providers = &*crate::server::tools::AVAILABLE_PROVIDERS
        .processing
        .read()
        .unwrap();
    if !processing_providers.is_empty() {
        println!(
            "- Processing: {}",
            processing_providers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    } else {
        println!("- Processing: None available (missing API keys)");
    }

    let enhancement_providers = &*crate::server::tools::AVAILABLE_PROVIDERS
        .enhancement
        .read()
        .unwrap();
    if !enhancement_providers.is_empty() {
        println!(
            "- Enhancement: {}",
            enhancement_providers
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    } else {
        println!("- Enhancement: None available (missing API keys)");
    }
}
