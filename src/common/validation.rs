use eyre::{eyre, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{debug, warn};
use validator::{Validate, ValidationError};

use crate::common::types::BaseSearchParams;

// Validation constants
const MAX_QUERY_LENGTH: usize = 1000;
const MAX_RESULTS_LIMIT: usize = 100;
const MIN_RESULTS_LIMIT: usize = 1;
const MAX_DOMAIN_COUNT: usize = 50;
const MAX_DOMAIN_LENGTH: usize = 253; // DNS limit

lazy_static::lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(
        r"^https?://(?:[-\w.])+(?:\:[0-9]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?$"
    ).unwrap();

    static ref DOMAIN_REGEX: Regex = Regex::new(
        r"^(?:[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)*[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$"
    ).unwrap();

    static ref MALICIOUS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>").unwrap(), // XSS
        Regex::new(r"(?i)javascript:").unwrap(), // JavaScript URLs
        Regex::new(r"(?i)data:.*base64").unwrap(), // Base64 data URLs
        Regex::new(r"(?i)vbscript:").unwrap(), // VBScript
        Regex::new(r"(?i)on\w+\s*=").unwrap(), // Event handlers
        Regex::new(r"\b(union|select|insert|delete|update|drop|create|alter|exec|execute)\b").unwrap(), // SQL injection
        Regex::new(r#"[<>"'&]"#).unwrap(), // HTML special characters
    ];

    // Common malicious or problematic query patterns
    static ref BLOCKED_QUERY_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)\b(porn|xxx|adult|nude|sex|erotic)\b").unwrap(),
        Regex::new(r"(?i)\b(crack|piracy|warez|torrent|illegal)\b").unwrap(),
        Regex::new(r"(?i)\b(bomb|weapon|terrorist|violence)\b").unwrap(),
        Regex::new(r"(?i)\b(drug|cocaine|heroin|meth)\b").unwrap(),
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ValidatedSearchParams {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Query must be between 1 and 1000 characters"
    ))]
    #[validate(custom = "validate_query_content")]
    pub query: String,

    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: Option<u32>,

    #[validate(custom = "validate_domains")]
    pub include_domains: Option<Vec<String>>,

    #[validate(custom = "validate_domains")]
    pub exclude_domains: Option<Vec<String>>,

    #[validate(custom = "validate_urls")]
    pub urls: Option<Vec<String>>,
}

impl ValidatedSearchParams {
    pub fn from_base_params(params: BaseSearchParams) -> Result<Self> {
        let validated = Self {
            query: params.query,
            limit: params.limit,
            include_domains: params.include_domains,
            exclude_domains: params.exclude_domains,
            urls: None, // BaseSearchParams doesn't have URLs
        };

        validated
            .validate()
            .map_err(|e| eyre!("Validation failed: {}", format_validation_errors(&e)))?;

        Ok(validated)
    }

    pub fn to_base_params(&self) -> BaseSearchParams {
        BaseSearchParams {
            query: self.query.clone(),
            limit: self.limit,
            include_domains: self.include_domains.clone(),
            exclude_domains: self.exclude_domains.clone(),
        }
    }
}

// Custom validation functions
fn validate_query_content(query: &str) -> std::result::Result<(), ValidationError> {
    debug!("Validating query content: {}", query);

    // Check for malicious patterns
    for pattern in MALICIOUS_PATTERNS.iter() {
        if pattern.is_match(query) {
            warn!("Query contains malicious pattern: {}", query);
            return Err(ValidationError::new("contains_malicious_content"));
        }
    }

    // Check for blocked content patterns (optional - could be configurable)
    for pattern in BLOCKED_QUERY_PATTERNS.iter() {
        if pattern.is_match(query) {
            warn!("Query contains blocked content: {}", query);
            return Err(ValidationError::new("contains_blocked_content"));
        }
    }

    // Check for excessive repetition (potential spam)
    if has_excessive_repetition(query) {
        return Err(ValidationError::new("excessive_repetition"));
    }

    // Check for control characters
    if query
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err(ValidationError::new("contains_control_characters"));
    }

    Ok(())
}

fn validate_domains(domains: &[String]) -> std::result::Result<(), ValidationError> {
    if domains.len() > MAX_DOMAIN_COUNT {
        return Err(ValidationError::new("too_many_domains"));
    }

    let mut seen_domains = HashSet::new();

    for domain in domains {
        // Check domain length
        if domain.len() > MAX_DOMAIN_LENGTH {
            return Err(ValidationError::new("domain_too_long"));
        }

        // Check domain format
        if !DOMAIN_REGEX.is_match(domain) {
            return Err(ValidationError::new("invalid_domain_format"));
        }

        // Check for duplicates
        if !seen_domains.insert(domain.to_lowercase()) {
            return Err(ValidationError::new("duplicate_domain"));
        }

        // Check for suspicious domains
        if is_suspicious_domain(domain) {
            warn!("Suspicious domain detected: {}", domain);
            return Err(ValidationError::new("suspicious_domain"));
        }
    }

    Ok(())
}

fn validate_urls(urls: &[String]) -> std::result::Result<(), ValidationError> {
    if urls.len() > MAX_DOMAIN_COUNT {
        return Err(ValidationError::new("too_many_urls"));
    }

    let mut seen_urls = HashSet::new();

    for url in urls {
        // Check URL format
        if !URL_REGEX.is_match(url) {
            return Err(ValidationError::new("invalid_url_format"));
        }

        // Check for duplicates
        if !seen_urls.insert(url.to_lowercase()) {
            return Err(ValidationError::new("duplicate_url"));
        }

        // Check for suspicious URLs
        if is_suspicious_url(url) {
            warn!("Suspicious URL detected: {}", url);
            return Err(ValidationError::new("suspicious_url"));
        }

        // Validate URL length
        if url.len() > 2048 {
            return Err(ValidationError::new("url_too_long"));
        }
    }

    Ok(())
}

// Helper functions
fn has_excessive_repetition(text: &str) -> bool {
    // Check for repeated characters (more than 10 in a row)
    let chars: Vec<char> = text.chars().collect();
    let mut count = 1;

    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            count += 1;
            if count > 10 {
                return true;
            }
        } else {
            count = 1;
        }
    }

    // Check for repeated words
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 5 {
        return false;
    }

    let mut word_counts = std::collections::HashMap::new();
    for word in words {
        *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }

    // If any word appears more than 30% of the time, it's excessive
    let max_count = *word_counts.values().max().unwrap_or(&0);
    max_count as f64 / words.len() as f64 > 0.3
}

fn is_suspicious_domain(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();

    // Check for suspicious TLDs (this could be configurable)
    let suspicious_tlds = ["tk", "ml", "ga", "cf", "xyz"];
    if let Some(tld) = domain_lower.split('.').last() {
        if suspicious_tlds.contains(&tld) {
            return true;
        }
    }

    // Check for excessive hyphens or numbers
    let hyphen_count = domain_lower.matches('-').count();
    let number_count = domain_lower.chars().filter(|c| c.is_numeric()).count();

    if hyphen_count > 3 || number_count > domain_lower.len() / 2 {
        return true;
    }

    // Check for homograph attacks (basic check)
    if domain_lower.chars().any(|c| !c.is_ascii()) {
        return true;
    }

    false
}

fn is_suspicious_url(url: &str) -> bool {
    let url_lower = url.to_lowercase();

    // Check for suspicious patterns in URLs
    let suspicious_patterns = [
        "bit.ly",
        "tinyurl",
        "t.co",
        "goo.gl", // URL shorteners
        "iplogger",
        "grabify",
        "blasze",           // IP loggers
        "pastebin.com/raw", // Raw pastes
        "discord.gg",       // Discord invites (could be spam)
    ];

    for pattern in suspicious_patterns {
        if url_lower.contains(pattern) {
            return true;
        }
    }

    // Check for IP addresses instead of domains
    let domain_part = if let Some(start) = url_lower.find("://") {
        if let Some(end) = url_lower[start + 3..].find('/') {
            &url_lower[start + 3..start + 3 + end]
        } else {
            &url_lower[start + 3..]
        }
    } else {
        return true; // Invalid URL format
    };

    // Basic IP address detection
    if domain_part.split('.').count() == 4
        && domain_part
            .split('.')
            .all(|part| part.parse::<u8>().is_ok())
    {
        return true;
    }

    false
}

fn format_validation_errors(errors: &validator::ValidationErrors) -> String {
    let mut formatted = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match error.code.as_ref() {
                "length" => "Invalid length",
                "range" => "Value out of range",
                "contains_malicious_content" => "Contains potentially malicious content",
                "contains_blocked_content" => "Contains blocked content",
                "excessive_repetition" => "Contains excessive repetition",
                "contains_control_characters" => "Contains invalid control characters",
                "too_many_domains" => "Too many domains specified",
                "domain_too_long" => "Domain name too long",
                "invalid_domain_format" => "Invalid domain format",
                "duplicate_domain" => "Duplicate domains not allowed",
                "suspicious_domain" => "Suspicious domain detected",
                "too_many_urls" => "Too many URLs specified",
                "invalid_url_format" => "Invalid URL format",
                "duplicate_url" => "Duplicate URLs not allowed",
                "suspicious_url" => "Suspicious URL detected",
                "url_too_long" => "URL too long",
                _ => "Validation error",
            };
            formatted.push(format!("{}: {}", field, message));
        }
    }

    formatted.join(", ")
}

// Public validation functions
pub fn validate_search_params(params: &BaseSearchParams) -> Result<ValidatedSearchParams> {
    ValidatedSearchParams::from_base_params(params.clone())
}

pub fn sanitize_query(query: &str) -> String {
    // Remove or replace potentially problematic characters
    query
        .trim()
        .replace('\0', "") // Remove null characters
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .collect::<String>()
        .chars()
        .take(MAX_QUERY_LENGTH)
        .collect()
}

pub fn validate_provider_name(provider: &str) -> Result<()> {
    if provider.is_empty() {
        return Err(eyre!("Provider name cannot be empty"));
    }

    if provider.len() > 50 {
        return Err(eyre!("Provider name too long"));
    }

    if !provider
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(eyre!("Provider name contains invalid characters"));
    }

    Ok(())
}

pub fn validate_operation_name(operation: &str) -> Result<()> {
    if operation.is_empty() {
        return Err(eyre!("Operation name cannot be empty"));
    }

    if operation.len() > 50 {
        return Err(eyre!("Operation name too long"));
    }

    if !operation
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(eyre!("Operation name contains invalid characters"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_query() {
        let params = BaseSearchParams {
            query: "rust programming".to_string(),
            limit: Some(10),
            include_domains: Some(vec!["github.com".to_string()]),
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_malicious_query() {
        let params = BaseSearchParams {
            query: "<script>alert('xss')</script>".to_string(),
            limit: Some(10),
            include_domains: None,
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("malicious"));
    }

    #[test]
    fn test_excessive_repetition() {
        let params = BaseSearchParams {
            query: "test test test test test test test test test test".to_string(),
            limit: Some(10),
            include_domains: None,
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_domain() {
        let params = BaseSearchParams {
            query: "test query".to_string(),
            limit: Some(10),
            include_domains: Some(vec!["not-a-valid-domain".to_string()]),
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_length_limits() {
        let long_query = "a".repeat(1001);
        let params = BaseSearchParams {
            query: long_query,
            limit: Some(10),
            include_domains: None,
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_limit_validation() {
        let params = BaseSearchParams {
            query: "test".to_string(),
            limit: Some(101), // Over maximum
            include_domains: None,
            exclude_domains: None,
        };

        let result = validate_search_params(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_query() {
        let dirty_query = "test\0query\x01with\x7fcontrol";
        let clean_query = sanitize_query(dirty_query);

        assert_eq!(clean_query, "testquerywithcontrol");
    }

    #[test]
    fn test_provider_name_validation() {
        assert!(validate_provider_name("valid_provider").is_ok());
        assert!(validate_provider_name("valid-provider").is_ok());
        assert!(validate_provider_name("").is_err());
        assert!(validate_provider_name("invalid provider").is_err());
        assert!(validate_provider_name(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_suspicious_domain_detection() {
        assert!(is_suspicious_domain("example.tk"));
        assert!(is_suspicious_domain("test-with-many-hyphens-here.com"));
        assert!(is_suspicious_domain("123456789.com"));
        assert!(!is_suspicious_domain("github.com"));
        assert!(!is_suspicious_domain("docs.rs"));
    }

    #[test]
    fn test_suspicious_url_detection() {
        assert!(is_suspicious_url("https://bit.ly/123"));
        assert!(is_suspicious_url("http://192.168.1.1/test"));
        assert!(is_suspicious_url("https://iplogger.org/test"));
        assert!(!is_suspicious_url("https://github.com/user/repo"));
        assert!(!is_suspicious_url("https://docs.rs/crate"));
    }
}
