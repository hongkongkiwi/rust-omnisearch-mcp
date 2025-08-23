use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct BaiduSearchResponse {
    organic_results: Vec<BaiduResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BaiduResult {
    title: String,
    link: String,
    snippet: String,
}

pub struct BaiduSearchProvider {
    client: Client,
}

impl Default for BaiduSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BaiduSearchProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(CONFIG.providers.baidu.timeout_seconds * 1000))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl SearchProvider for BaiduSearchProvider {
    fn name(&self) -> &'static str {
        "baidu"
    }

    fn description(&self) -> &'static str {
        "Search the web using Baidu Search via SerpApi. Provides search results from China's leading search engine. Requires SerpApi API key."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        let api_key = CONFIG.providers.baidu.api_key.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing SerpApi API key".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        // Prepare query parameters
        let mut query_params = vec![
            ("engine", "baidu".to_string()),
            ("api_key", api_key.clone()),
            ("q", params.query.clone()),
        ];

        if let Some(limit) = params.limit {
            let limit_str = limit.to_string();
            query_params.push(("num", limit_str));
        }

        // Make the request
        let response = self
            .client
            .get(format!("{}/search", CONFIG.providers.baidu.base_url.as_deref().unwrap_or("https://serpapi.com")))
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

            match status.as_u16() {
                401 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "Invalid SerpApi API key".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                403 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "SerpApi API access forbidden".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                429 => {
                    return Err(ProviderError::new(
                        ErrorType::RateLimit,
                        "SerpApi rate limit exceeded".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                500 => {
                    return Err(ProviderError::new(
                        ErrorType::ProviderError,
                        "SerpApi internal error".to_string(),
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
        let data: BaiduSearchResponse = response.json().await.map_err(|e| {
            ProviderError::new(
                ErrorType::ApiError,
                format!("Failed to parse response: {}", e),
                self.name().to_string(),
                Some(e.into()),
            )
        })?;

        // Convert to SearchResult format
        let results = data
            .organic_results
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
