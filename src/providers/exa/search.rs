use crate::common::http::{create_http_client, handle_http_error};
use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExaResult {
    title: String,
    url: String,
    text: String,
    score: f64,
}

pub struct ExaSearchProvider {
    client: Client,
}

impl Default for ExaSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ExaSearchProvider {
    pub fn new() -> Self {
        let client = create_http_client(CONFIG.search.exa.timeout);
        Self { client }
    }
}

#[async_trait]
impl SearchProvider for ExaSearchProvider {
    fn name(&self) -> &'static str {
        "exa"
    }

    fn description(&self) -> &'static str {
        "Search the web using Exa Search API. Provides high-quality search results with relevance scores. Requires Exa API key."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        let api_key = CONFIG.search.exa.api_key.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Exa API key".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        // Prepare request body
        let mut request_body = serde_json::Map::new();
        request_body.insert("query".to_string(), serde_json::Value::String(params.query));
        request_body.insert(
            "limit".to_string(),
            serde_json::Value::Number(params.limit.unwrap_or(5).into()),
        );

        if let Some(include_domains) = params.include_domains {
            request_body.insert(
                "include_domains".to_string(),
                serde_json::Value::Array(
                    include_domains
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }

        if let Some(exclude_domains) = params.exclude_domains {
            request_body.insert(
                "exclude_domains".to_string(),
                serde_json::Value::Array(
                    exclude_domains
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }

        // Make the request
        let response = self
            .client
            .post(format!("{}/search", CONFIG.search.exa.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
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
                "Exa rate limit exceeded",
                "Invalid Exa API key",
                "Exa API access forbidden",
                "Exa API internal error",
            );
            return Err(error);
        }

        // Parse the response
        let data: ExaSearchResponse = response.json().await.map_err(|e| {
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
                snippet: result.text,
                score: Some(result.score),
                source_provider: self.name().to_string(),
            })
            .collect();

        Ok(results)
    }
}
