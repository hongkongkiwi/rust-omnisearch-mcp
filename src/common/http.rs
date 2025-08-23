//! Common HTTP utilities for providers

use reqwest::Client;
use std::time::Duration;

/// Create a HTTP client with timeout
pub fn create_http_client(timeout_ms: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .expect("Failed to create HTTP client")
}

/// Handle common HTTP error responses
pub fn handle_http_error(
    status: reqwest::StatusCode,
    error_message: String,
    provider_name: &str,
    rate_limit_message: &str,
    auth_error_message: &str,
    forbidden_message: &str,
    internal_error_message: &str,
) -> crate::common::types::ProviderError {
    use crate::common::types::{ErrorType, ProviderError};
    
    match status.as_u16() {
        400 => ProviderError::new(
            ErrorType::InvalidInput,
            "Invalid request parameters".to_string(),
            provider_name.to_string(),
            None,
        ),
        401 => ProviderError::new(
            ErrorType::ApiError,
            auth_error_message.to_string(),
            provider_name.to_string(),
            None,
        ),
        403 => ProviderError::new(
            ErrorType::ApiError,
            forbidden_message.to_string(),
            provider_name.to_string(),
            None,
        ),
        429 => ProviderError::new(
            ErrorType::RateLimit,
            rate_limit_message.to_string(),
            provider_name.to_string(),
            None,
        ),
        500 => ProviderError::new(
            ErrorType::ProviderError,
            internal_error_message.to_string(),
            provider_name.to_string(),
            None,
        ),
        _ => ProviderError::new(
            ErrorType::ApiError,
            format!("Unexpected error: {}", error_message),
            provider_name.to_string(),
            None,
        ),
    }
}