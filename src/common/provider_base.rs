//! Base provider functionality that can be shared across all providers

use crate::common::types::{ErrorType, ProviderError};

/// A trait for providers that need API key validation
pub trait ApiKeyProvider {
    fn validate_api_key(&self, api_key: Option<&String>, provider_name: &str) -> Result<(), ProviderError> {
        if api_key.is_none() {
            Err(ProviderError::new(
                ErrorType::ApiError,
                format!("Missing API key for {}", provider_name),
                provider_name.to_string(),
                None,
            ))
        } else {
            Ok(())
        }
    }
}

/// A trait for providers that need multiple credential validation
pub trait MultiCredentialProvider {
    fn validate_credentials(&self, credentials: Vec<Option<&String>>, error_messages: Vec<&str>, provider_name: &str) -> Result<(), ProviderError> {
        for (i, credential) in credentials.iter().enumerate() {
            if credential.is_none() {
                return Err(ProviderError::new(
                    ErrorType::ApiError,
                    error_messages[i].to_string(),
                    provider_name.to_string(),
                    None,
                ));
            }
        }
        Ok(())
    }
}

/// Common utility functions for providers
pub struct ProviderUtils;

impl ProviderUtils {
    /// Create a standardized provider error
    pub fn provider_error(error_type: ErrorType, message: String, provider_name: String) -> ProviderError {
        ProviderError::new(error_type, message, provider_name, None)
    }
    
    /// Convert a vector of strings to a comma-separated string
    pub fn join_domains(domains: &[String]) -> String {
        domains.join(",")
    }
    
    /// Create a site filter from domains
    pub fn create_site_filter(domains: &[String]) -> String {
        domains.iter().map(|d| format!("site:{}", d)).collect::<Vec<_>>().join(" OR ")
    }
}