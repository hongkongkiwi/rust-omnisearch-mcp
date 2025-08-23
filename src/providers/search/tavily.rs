use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct TavilySearchResponse {
    results: Vec<TavilyResult>,
    response_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f64,
}

pub struct TavilySearchProvider {
    client: Client,
}

impl Default for TavilySearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TavilySearchProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(CONFIG.search.tavily.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl SearchProvider for TavilySearchProvider {
    fn name(&self) -> &'static str {
        "tavily"
    }

    fn description(&self) -> &'static str {
        "Search the web using Tavily Search API. Best for factual queries requiring reliable sources and citations. Supports domain filtering through API parameters (include_domains/exclude_domains). Provides high-quality results for technical, scientific, and academic topics. Use when you need verified information with strong citation support."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        let api_key = CONFIG.search.tavily.api_key.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing API key".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        // Prepare request body
        let mut request_body = serde_json::Map::new();
        request_body.insert("query".to_string(), serde_json::Value::String(params.query));
        request_body.insert(
            "max_results".to_string(),
            serde_json::Value::Number(params.limit.unwrap_or(5).into()),
        );
        request_body.insert(
            "search_depth".to_string(),
            serde_json::Value::String("basic".to_string()),
        );
        request_body.insert(
            "topic".to_string(),
            serde_json::Value::String("general".to_string()),
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
            .post(format!("{}/search", CONFIG.search.tavily.base_url))
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

            match status.as_u16() {
                401 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "Invalid API key".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                403 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "API key does not have access to this endpoint".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                429 => {
                    return Err(ProviderError::new(
                        ErrorType::RateLimit,
                        "Rate limit exceeded".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                500 => {
                    return Err(ProviderError::new(
                        ErrorType::ProviderError,
                        "Tavily API internal error".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                _ => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        format!("Unexpected error: {}", error_message),
                        self.name().to_string(),
                        None,
                    ))
                }
            }
        }

        // Parse the response
        let data: TavilySearchResponse = response.json().await.map_err(|e| {
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
                snippet: result.content,
                score: Some(result.score),
                source_provider: self.name().to_string(),
            })
            .collect();

        Ok(results)
    }
}
