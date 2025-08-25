//! High-level client API for using omnisearch as a library.
//!
//! This module provides a convenient interface for applications that want to use
//! omnisearch functionality without running a full MCP server.

use crate::common::types::{BaseSearchParams, ProviderError, SearchProvider, SearchResult};
use crate::{create_providers, validate_config};
use std::collections::HashMap;

/// A high-level client for performing omnisearch operations.
///
/// The client automatically discovers and initializes available search providers
/// based on environment variables and API keys.
///
/// # Example
///
/// ```rust
/// use omnisearch_mcp::{OmnisearchClient, SearchRequest};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = OmnisearchClient::new().await?;
///
///     let request = SearchRequest::new("rust async programming")
///         .limit(5)
///         .provider("tavily");
///
///     let response = client.search(request).await?;
///
///     println!("Found {} results from {} providers",
///         response.results.len(),
///         response.providers_used.len());
///
///     for result in response.results {
///         println!("- {}: {}", result.title, result.url);
///     }
///
///     Ok(())
/// }
/// ```
pub struct OmnisearchClient {
    providers: HashMap<String, Box<dyn SearchProvider>>,
}

impl OmnisearchClient {
    /// Create a new omnisearch client.
    ///
    /// This will validate the configuration and initialize all available providers
    /// based on the current environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails or no providers can be initialized.
    pub async fn new() -> Result<Self, ProviderError> {
        // Validate configuration first
        validate_config().map_err(|e| {
            ProviderError::new(
                crate::common::types::ErrorType::ApiError,
                format!("Configuration validation failed: {}", e),
                "client".to_string(),
                None,
            )
        })?;

        // Initialize providers
        let provider_list = create_providers();

        if provider_list.is_empty() {
            return Err(ProviderError::new(
                crate::common::types::ErrorType::ApiError,
                "No search providers could be initialized. Please check your API keys.".to_string(),
                "client".to_string(),
                None,
            ));
        }

        // Convert to HashMap for easy access
        let mut providers = HashMap::new();
        for provider in provider_list {
            providers.insert(provider.name().to_string(), provider);
        }

        Ok(Self { providers })
    }

    /// Get the names of all available providers.
    pub fn available_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a specific provider is available.
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Get information about a specific provider.
    pub fn provider_info(&self, name: &str) -> Option<(String, String)> {
        self.providers
            .get(name)
            .map(|p| (p.name().to_string(), p.description().to_string()))
    }

    /// Perform a search using the specified request parameters.
    ///
    /// If no specific provider is requested, this will try providers in a sensible order
    /// until one succeeds or all fail.
    pub async fn search(&self, request: SearchRequest) -> Result<SearchResponse, ProviderError> {
        let query = request.query.clone();
        let preferred_provider = request.preferred_provider.clone();
        let params = request.into_search_params();

        if let Some(provider_name) = &preferred_provider {
            // Use specific provider
            if let Some(provider) = self.providers.get(provider_name) {
                let results = provider.search(params).await?;
                return Ok(SearchResponse {
                    results,
                    providers_used: vec![provider_name.clone()],
                    query: query.clone(),
                });
            } else {
                return Err(ProviderError::new(
                    crate::common::types::ErrorType::InvalidInput,
                    format!("Provider '{}' is not available", provider_name),
                    "client".to_string(),
                    None,
                ));
            }
        }

        // Try providers in preferred order
        let provider_order = ["tavily", "google", "duckduckgo", "reddit", "exa", "brave"];
        let mut last_error = None;

        for provider_name in provider_order {
            if let Some(provider) = self.providers.get(provider_name) {
                match provider.search(params.clone()).await {
                    Ok(results) => {
                        return Ok(SearchResponse {
                            results,
                            providers_used: vec![provider_name.to_string()],
                            query: query.clone(),
                        });
                    }
                    Err(e) => {
                        last_error = Some(e);
                        continue;
                    }
                }
            }
        }

        // If we get here, all providers failed
        Err(last_error.unwrap_or_else(|| {
            ProviderError::new(
                crate::common::types::ErrorType::ProviderError,
                "No providers available for search".to_string(),
                "client".to_string(),
                None,
            )
        }))
    }

    /// Perform a search across multiple providers and combine results.
    ///
    /// This method will query multiple providers sequentially and return the first successful result.
    /// For true parallel searching, providers would need to implement Send + Sync.
    pub async fn multi_search(
        &self,
        request: SearchRequest,
        max_providers: usize,
    ) -> Result<SearchResponse, ProviderError> {
        let query = request.query.clone();
        let preferred_provider = request.preferred_provider.clone();
        let params = request.into_search_params();
        let mut provider_names = Vec::new();

        // Launch searches across available providers
        let providers_to_use: Vec<_> = if let Some(preferred) = &preferred_provider {
            if self.providers.contains_key(preferred) {
                vec![preferred.as_str()]
            } else {
                return Err(ProviderError::new(
                    crate::common::types::ErrorType::InvalidInput,
                    format!("Preferred provider '{}' not available", preferred),
                    "client".to_string(),
                    None,
                ));
            }
        } else {
            self.providers
                .keys()
                .take(max_providers)
                .map(|s| s.as_str())
                .collect()
        };

        for provider_name in providers_to_use {
            if let Some(provider) = self.providers.get(provider_name) {
                let params_clone = params.clone();
                let provider_name = provider_name.to_string();

                // Try each provider sequentially
                match provider.search(params_clone).await {
                    Ok(results) => {
                        provider_names.push(provider_name);
                        return Ok(SearchResponse {
                            results,
                            providers_used: provider_names,
                            query: query.clone(),
                        });
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(ProviderError::new(
            crate::common::types::ErrorType::ProviderError,
            "All provider searches failed".to_string(),
            "client".to_string(),
            None,
        ))
    }
}

/// A request for performing a search.
///
/// This builder-style struct allows you to configure search parameters
/// in a fluent, readable way.
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
    pub include_domains: Option<Vec<String>>,
    pub exclude_domains: Option<Vec<String>>,
    pub preferred_provider: Option<String>,
}

impl SearchRequest {
    /// Create a new search request with the given query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: None,
            include_domains: None,
            exclude_domains: None,
            preferred_provider: None,
        }
    }

    /// Set the maximum number of results to return.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include only results from the specified domains.
    pub fn include_domains(mut self, domains: &[&str]) -> Self {
        self.include_domains = Some(domains.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Exclude results from the specified domains.
    pub fn exclude_domains(mut self, domains: &[&str]) -> Self {
        self.exclude_domains = Some(domains.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Prefer a specific search provider.
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.preferred_provider = Some(provider.into());
        self
    }

    /// Convert this request into BaseSearchParams for use with providers.
    pub(crate) fn into_search_params(self) -> BaseSearchParams {
        BaseSearchParams {
            query: self.query,
            limit: self.limit,
            include_domains: self.include_domains,
            exclude_domains: self.exclude_domains,
        }
    }
}

/// The response from a search operation.
#[derive(Debug)]
pub struct SearchResponse {
    /// The search results.
    pub results: Vec<SearchResult>,
    /// The names of providers that were used to generate these results.
    pub providers_used: Vec<String>,
    /// The original search query.
    pub query: String,
}

impl SearchResponse {
    /// Get the number of results.
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if the response is empty.
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get results from a specific provider.
    pub fn results_from_provider(&self, provider: &str) -> Vec<&SearchResult> {
        self.results
            .iter()
            .filter(|r| r.source_provider == provider)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_request_builder() {
        let request = SearchRequest::new("test query")
            .limit(10)
            .include_domains(&["example.com", "test.org"])
            .provider("tavily");

        assert_eq!(request.query, "test query");
        assert_eq!(request.limit, Some(10));
        assert_eq!(request.preferred_provider, Some("tavily".to_string()));
        assert_eq!(
            request.include_domains,
            Some(vec!["example.com".to_string(), "test.org".to_string()])
        );
    }

    #[test]
    fn test_search_params_conversion() {
        let request = SearchRequest::new("test")
            .limit(5)
            .exclude_domains(&["spam.com"]);

        let params = request.into_search_params();
        assert_eq!(params.query, "test");
        assert_eq!(params.limit, Some(5));
        assert_eq!(params.exclude_domains, Some(vec!["spam.com".to_string()]));
    }
}
