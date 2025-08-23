use crate::common::http::{create_http_client, handle_http_error};
use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct DuckDuckGoSearchResponse {
    results: Vec<DuckDuckGoResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DuckDuckGoResult {
    title: String,
    url: String,
    snippet: String,
}

pub struct DuckDuckGoSearchProvider {
    client: Client,
}

impl Default for DuckDuckGoSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl DuckDuckGoSearchProvider {
    pub fn new() -> Self {
        let client = create_http_client(CONFIG.providers.duckduckgo.timeout_seconds * 1000);
        Self { client }
    }
}

#[async_trait]
impl SearchProvider for DuckDuckGoSearchProvider {
    fn name(&self) -> &'static str {
        "duckduckgo"
    }

    fn description(&self) -> &'static str {
        "Search the web using DuckDuckGo search API. Provides privacy-focused search results without tracking. No API key required."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        // Prepare query parameters
        let limit_str = params.limit.unwrap_or(5).to_string();
        let query_params = vec![
            ("q", params.query.as_str()),
            ("kl", "us-en"),            // Set locale to US English
            ("s", "0"),                 // Start at first result
            ("dc", limit_str.as_str()), // Number of results
            ("o", "json"),              // Output format
        ];

        // Make the request
        let response = self
            .client
            .get(format!(
                "{}/search",
                CONFIG
                    .providers
                    .duckduckgo
                    .base_url
                    .as_deref()
                    .unwrap_or("https://api.duckduckgo.com")
            ))
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                ProviderError::new(
                    ErrorType::ApiError,
                    format!("Failed to send request: {}", e),
                    self.name().to_string(),
                    Some(e.into()),
                )
            })?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status();
            let error_message = match response.text().await {
                Ok(text) => text,
                Err(_) => status.to_string(),
            };

            let error = handle_http_error(
                status,
                error_message,
                self.name(),
                "DuckDuckGo API rate limit exceeded",
                "DuckDuckGo API authentication error",
                "DuckDuckGo API access forbidden",
                "DuckDuckGo API internal error",
            );
            return Err(error);
        }

        // Parse the response
        let data: DuckDuckGoSearchResponse = response.json().await.map_err(|e| {
            ProviderError::new(
                ErrorType::ApiError,
                format!("Failed to parse response: {}", e),
                self.name().to_string(),
                Some(e.into()),
            )
        })?;

        // Convert to SearchResult format
        let results = data
            .results
            .into_iter()
            .map(|result| SearchResult {
                title: result.title,
                url: result.url,
                snippet: result.snippet,
                score: None,
                source_provider: self.name().to_string(),
            })
            .collect();

        Ok(results)
    }
}
