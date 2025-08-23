use serde::{Deserialize, Serialize};
use thiserror::Error;
use eyre;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub source_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSearchParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_contents: Option<Vec<RawContent>>,
    pub metadata: ProcessingMetadata,
    pub source_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContent {
    pub url: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls_processed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub successful_extractions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extract_depth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementResult {
    pub original_content: String,
    pub enhanced_content: String,
    pub enhancements: Vec<Enhancement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<EnhancementSource>>,
    pub source_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enhancement {
    pub r#type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementSource {
    pub title: String,
    pub url: String,
}

// Provider traits
#[async_trait::async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, params: BaseSearchParams) -> Result<Vec<SearchResult>, ProviderError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

#[async_trait::async_trait]
pub trait ProcessingProvider: Send + Sync {
    async fn process_content(
        &self,
        url: Vec<String>,
        extract_depth: Option<String>,
    ) -> Result<ProcessingResult, ProviderError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

#[async_trait::async_trait]
pub trait EnhancementProvider: Send + Sync {
    async fn enhance_content(&self, content: String) -> Result<EnhancementResult, ProviderError>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

// Error types
#[derive(Error, Debug, PartialEq)]
pub enum ErrorType {
    #[error("API Error")]
    ApiError,
    #[error("Rate Limit")]
    RateLimit,
    #[error("Invalid Input")]
    InvalidInput,
    #[error("Provider Error")]
    ProviderError,
}

#[derive(Error, Debug)]
#[error("Provider error: {message} (provider: {provider})")]
pub struct ProviderError {
    pub error_type: ErrorType,
    pub message: String,
    pub provider: String,
    #[source]
    pub source: Option<eyre::Error>,
}

impl ProviderError {
    pub fn new(
        error_type: ErrorType,
        message: String,
        provider: String,
        source: Option<eyre::Error>,
    ) -> Self {
        Self {
            error_type,
            message,
            provider,
            source,
        }
    }
}
