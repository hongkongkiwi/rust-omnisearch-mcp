use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct BrightDataSearchResponse {
    results: Vec<BrightDataResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BrightDataResult {
    title: String,
    url: String,
    description: String,
}

pub struct BrightDataSearchProvider {
    client: Client,
}

impl Default for BrightDataSearchProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BrightDataSearchProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(CONFIG.providers.brightdata.timeout_seconds * 1000))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl SearchProvider for BrightDataSearchProvider {
    fn name(&self) -> &'static str {
        "brightdata"
    }

    fn description(&self) -> &'static str {
        "Search the web using Bright Data SERP API. Provides high-quality search results with advanced filtering options. Requires Bright Data API credentials."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        let username = CONFIG.providers.brightdata.username.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Bright Data username".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        let password = CONFIG.providers.brightdata.password.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Bright Data password".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        // Prepare query parameters
        let limit_str = params.limit.unwrap_or(5).to_string();
        let mut query_params = vec![("q", params.query.clone()), ("limit", limit_str)];

        if let Some(include_domains) = &params.include_domains {
            if !include_domains.is_empty() {
                let include_domains_str = include_domains.join(",");
                query_params.push(("include_domains", include_domains_str));
            }
        }

        if let Some(exclude_domains) = &params.exclude_domains {
            if !exclude_domains.is_empty() {
                let exclude_domains_str = exclude_domains.join(",");
                query_params.push(("exclude_domains", exclude_domains_str));
            }
        }

        // Make the request
        let response = self
            .client
            .get(format!("{}/search", "https://api.brightdata.com/serp"))
            .basic_auth(username, Some(password))
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
                        "Invalid Bright Data credentials".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                403 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "Bright Data API access forbidden".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                429 => {
                    return Err(ProviderError::new(
                        ErrorType::RateLimit,
                        "Bright Data rate limit exceeded".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                500 => {
                    return Err(ProviderError::new(
                        ErrorType::ProviderError,
                        "Bright Data API internal error".to_string(),
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
        let data: BrightDataSearchResponse = response.json().await.map_err(|e| {
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
                snippet: result.description,
                score: None,
                source_provider: self.name().to_string(),
            })
            .collect();

        Ok(results)
    }
}
