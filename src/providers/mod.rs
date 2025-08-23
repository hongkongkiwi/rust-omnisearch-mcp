use crate::{
    common::provider_factory::ProviderFactory, config::CONFIG, server::register_search_provider,
};

// Import search providers
pub mod baidu;
pub mod brightdata;
pub mod duckduckgo;
pub mod exa;
pub mod google;
pub mod reddit;
pub mod search;

pub fn initialize_providers() {
    // Initialize search providers using the factory
    let search_providers = ProviderFactory::create_search_providers();

    for provider in search_providers {
        // All our new providers are regular search providers (not AI response providers)
        register_search_provider(provider, false);
    }

    // Initialize AI response providers
    if CONFIG.ai_response.perplexity.api_key.is_some() {
        // register_search_provider(Box::new(PerplexityProvider::new()), true);
    }

    if CONFIG.ai_response.kagi_fastgpt.api_key.is_some() {
        // register_search_provider(Box::new(KagiFastGPTProvider::new()), true);
    }

    // Initialize processing providers
    if CONFIG.processing.jina_reader.api_key.is_some() {
        // register_processing_provider(Box::new(JinaReaderProvider::new()));
    }

    if CONFIG.processing.kagi_summarizer.api_key.is_some() {
        // register_processing_provider(Box::new(KagiSummarizerProvider::new()));
    }

    if CONFIG.processing.tavily_extract.api_key.is_some() {
        // register_processing_provider(Box::new(TavilyExtractProvider::new()));
    }

    if CONFIG.processing.firecrawl_scrape.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlScrapeProvider::new()));
    }

    if CONFIG.processing.firecrawl_crawl.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlCrawlProvider::new()));
    }

    if CONFIG.processing.firecrawl_map.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlMapProvider::new()));
    }

    if CONFIG.processing.firecrawl_extract.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlExtractProvider::new()));
    }

    if CONFIG.processing.firecrawl_actions.api_key.is_some() {
        // register_processing_provider(Box::new(FirecrawlActionsProvider::new()));
    }

    // Initialize enhancement providers
    if CONFIG.enhancement.jina_grounding.api_key.is_some() {
        // register_enhancement_provider(Box::new(JinaGroundingProvider::new()));
    }

    if CONFIG.enhancement.kagi_enrichment.api_key.is_some() {
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
