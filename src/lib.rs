//! # Omnisearch MCP
//!
//! A comprehensive Model Context Protocol (MCP) server providing unified access to multiple
//! search providers and AI tools. This crate can be used both as a standalone binary server
//! and as a library for integrating omnisearch capabilities into other applications.
//!
//! ## Features
//!
//! - **Search Providers**: Tavily, Google, Reddit, DuckDuckGo, Baidu, Bright Data, Exa, Brave
//! - **AI Services**: Perplexity AI, Kagi FastGPT
//! - **Content Processing**: Jina Reader, Kagi Summarizer, Tavily Extract, Firecrawl suite
//! - **Enhancement Tools**: Kagi Enrichment, Jina Grounding
//!
//! ## Library Usage
//!
//! ```rust
//! use omnisearch_mcp::{OmnisearchClient, SearchRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with available providers
//!     let client = OmnisearchClient::new().await?;
//!     
//!     // Perform a search
//!     let request = SearchRequest::new("rust programming")
//!         .limit(10)
//!         .include_domains(&["github.com", "docs.rs"]);
//!     
//!     let results = client.search(request).await?;
//!     
//!     for result in results {
//!         println!("{}: {}", result.title, result.url);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! This crate supports conditional compilation of providers through feature flags:
//!
//! - `default` = `["server", "all-providers"]`
//! - `server` - MCP server functionality
//! - `all-providers` - Enable all available providers
//! - `search-providers` - Enable all search providers
//! - Individual provider flags: `tavily`, `google`, `reddit`, etc.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod common;
pub mod config;
pub mod providers;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub mod server;

// Re-export common types for library users
pub use common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};

// Re-export configuration functions
pub use config::{validate_config, Config, CONFIG};

// Re-export provider initialization
pub use providers::create_providers;

#[cfg(feature = "server")]
pub use providers::initialize_providers;

// Re-export server functionality when the server feature is enabled
#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub use server::{register_tools, setup_handlers};

// High-level client API for library usage
mod client;
pub use client::{OmnisearchClient, SearchRequest, SearchResponse};

/// The current version of the omnisearch-mcp crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the omnisearch system with available providers
///
/// This is a convenience function that sets up logging, validates configuration,
/// and initializes available search providers based on environment variables.
///
/// # Example
///
/// ```rust
/// use omnisearch_mcp;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let providers = omnisearch_mcp::initialize().await?;
///     println!("Initialized {} search providers", providers.len());
///     Ok(())
/// }
/// ```
pub async fn initialize() -> Result<Vec<Box<dyn SearchProvider>>, ProviderError> {
    // Initialize tracing for library users
    tracing_subscriber::fmt::init();
    
    // Validate configuration
    validate_config().map_err(|e| ProviderError::new(
        crate::common::types::ErrorType::ApiError,
        format!("Configuration validation failed: {}", e),
        "omnisearch".to_string(),
        None,
    ))?;
    
    // Initialize and return available providers
    Ok(create_providers())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::SearchProvider;

    #[test]
    fn test_config_loading() {
        // This test will pass if the config loads without panicking
        let _config = &*config::CONFIG;
        assert!(true);
    }

    #[test]
    fn test_tavily_provider_creation() {
        let provider = providers::search::TavilySearchProvider::new();
        assert_eq!(provider.name(), "tavily");
        assert!(!provider.description().is_empty());
    }
}
