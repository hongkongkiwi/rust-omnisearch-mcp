use crate::common::http::{create_http_client, handle_http_error};
use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GoogleCustomSearchResponse {
    items: Option<Vec<GoogleResult>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleResult {
    title: String,
    link: String,
    snippet: String,
}

pub struct GoogleCustomSearchProvider {
    client: Client,
}

impl Default for GoogleCustomSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GoogleCustomSearchProvider {
    pub fn new() -> Self {
        let client = create_http_client(CONFIG.providers.google.timeout_seconds * 1000);
        Self { client }
    }
}

#[async_trait]
impl SearchProvider for GoogleCustomSearchProvider {
    fn name(&self) -> &'static str {
        "google_custom_search"
    }

    fn description(&self) -> &'static str {
        "Search the web using Google Custom Search API. Provides reliable web search results with snippets. Requires a Google API key and Custom Search Engine ID."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        let api_key = CONFIG.providers.google.api_key.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Google API key".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        let search_engine_id = CONFIG
            .providers
            .google
            .search_engine_id
            .as_ref()
            .ok_or_else(|| {
                ProviderError::new(
                    ErrorType::ApiError,
                    "Missing Google Custom Search Engine ID".to_string(),
                    self.name().to_string(),
                    None,
                )
            })?;

        // Prepare query parameters
        let limit_str = params.limit.unwrap_or(5).to_string();
        let mut query_params = vec![
            ("key", api_key.clone()),
            ("cx", search_engine_id.clone()),
            ("q", params.query.clone()),
            ("num", limit_str),
        ];

        if let Some(include_domains) = &params.include_domains {
            if !include_domains.is_empty() {
                let site_filter = include_domains
                    .iter()
                    .map(|d| format!("site:{}", d))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                let query_with_site = format!("{} ({})", params.query, site_filter);
                query_params.retain(|(k, _)| *k != "q");
                query_params.push(("q", query_with_site));
            }
        }

        // Make the request
        let response = self
            .client
            .get(format!(
                "{}/search",
                "https://www.googleapis.com/customsearch/v1"
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
                "Rate limit exceeded",
                "Invalid API key or unauthorized",
                "API key does not have access to this endpoint",
                "Google Custom Search API internal error",
            );
            return Err(error);
        }

        // Parse the response
        let data: GoogleCustomSearchResponse = response.json().await.map_err(|e| {
            ProviderError::new(
                ErrorType::ApiError,
                format!("Failed to parse response: {}", e),
                self.name().to_string(),
                Some(e.into()),
            )
        })?;

        // Convert to SearchResult format
        let results = data
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|result| SearchResult {
                title: result.title,
                url: result.link,
                snippet: result.snippet,
                score: None,
                source_provider: self.name().to_string(),
            })
            .collect();

        Ok(results)
    }
}
