use crate::common::types::{
    BaseSearchParams, ErrorType, ProviderError, SearchProvider, SearchResult,
};
use crate::config::CONFIG;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct RedditSearchResponse {
    data: RedditSearchData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditSearchData {
    children: Vec<RedditPostWrapper>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditPostWrapper {
    data: RedditPost,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedditPost {
    title: String,
    url: String,
    selftext: String,
    permalink: String,
    score: Option<i64>,
}

pub struct RedditSearchProvider {
    client: Client,
}

impl RedditSearchProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(CONFIG.search.reddit.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl SearchProvider for RedditSearchProvider {
    fn name(&self) -> &'static str {
        "reddit"
    }

    fn description(&self) -> &'static str {
        "Search Reddit posts using OAuth2 authentication. Provides access to discussions and content from Reddit communities. Requires Reddit API credentials (client ID, client secret, user agent)."
    }

    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError> {
        // Note: For simplicity, we're not implementing full OAuth2 flow here
        // In a real implementation, you would need to properly authenticate with Reddit's API
        // using the client credentials flow

        let client_id = CONFIG.search.reddit.client_id.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Reddit client ID".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        let client_secret = CONFIG.search.reddit.client_secret.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Reddit client secret".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        let user_agent = CONFIG.search.reddit.user_agent.as_ref().ok_or_else(|| {
            ProviderError::new(
                ErrorType::ApiError,
                "Missing Reddit user agent".to_string(),
                self.name().to_string(),
                None,
            )
        })?;

        // Prepare query parameters
        let limit_str = params.limit.unwrap_or(5).to_string();
        let query_params = vec![
            ("q", params.query.as_str()),
            ("limit", limit_str.as_str()),
            ("sort", "relevance"),
            ("type", "link"),
        ];

        // Make the request
        let response = self
            .client
            .get(&format!("{}/search", CONFIG.search.reddit.base_url))
            .header("User-Agent", user_agent)
            .basic_auth(client_id, Some(client_secret))
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
                        "Invalid Reddit API credentials".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                403 => {
                    return Err(ProviderError::new(
                        ErrorType::ApiError,
                        "Reddit API access forbidden".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                429 => {
                    return Err(ProviderError::new(
                        ErrorType::RateLimit,
                        "Reddit API rate limit exceeded".to_string(),
                        self.name().to_string(),
                        None,
                    ))
                }
                500 => {
                    return Err(ProviderError::new(
                        ErrorType::ProviderError,
                        "Reddit API internal error".to_string(),
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
        let data: RedditSearchResponse = response.json().await.map_err(|e| {
            ProviderError::new(
                ErrorType::ApiError,
                format!("Failed to parse response: {}", e),
                self.name().to_string(),
                Some(e.into()),
            )
        })?;

        // Convert to SearchResult format
        let results = data
            .data
            .children
            .into_iter()
            .map(|post_wrapper| {
                let post = post_wrapper.data;
                SearchResult {
                    title: post.title,
                    url: format!("https://reddit.com{}", post.permalink),
                    snippet: if post.selftext.is_empty() {
                        "No text content available".to_string()
                    } else {
                        post.selftext.chars().take(200).collect::<String>()
                    },
                    score: post.score.map(|s| s as f64),
                    source_provider: self.name().to_string(),
                }
            })
            .collect();

        Ok(results)
    }
}
