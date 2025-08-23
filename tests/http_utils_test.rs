use omnisearch_mcp::common::http::*;
use omnisearch_mcp::common::types::ErrorType;

#[test]
fn test_create_http_client() {
    let _client = create_http_client(5000);
    // Just test that we can create a client without panicking
    assert!(true);
}

#[test]
fn test_handle_http_error_400() {
    let error = handle_http_error(
        reqwest::StatusCode::BAD_REQUEST,
        "Bad request".to_string(),
        "test_provider",
        "Rate limit exceeded",
        "Auth error",
        "Forbidden",
        "Internal error",
    );

    match error.error_type {
        ErrorType::InvalidInput => assert!(true),
        _ => assert!(false, "Expected InvalidInput"),
    }
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_handle_http_error_401() {
    let error = handle_http_error(
        reqwest::StatusCode::UNAUTHORIZED,
        "Unauthorized".to_string(),
        "test_provider",
        "Rate limit exceeded",
        "Invalid API key",
        "Forbidden",
        "Internal error",
    );

    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("Invalid API key"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_handle_http_error_403() {
    let error = handle_http_error(
        reqwest::StatusCode::FORBIDDEN,
        "Forbidden".to_string(),
        "test_provider",
        "Rate limit exceeded",
        "Auth error",
        "API key does not have access",
        "Internal error",
    );

    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("API key does not have access"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_handle_http_error_429() {
    let error = handle_http_error(
        reqwest::StatusCode::TOO_MANY_REQUESTS,
        "Too many requests".to_string(),
        "test_provider",
        "Rate limit exceeded. Please try again later.",
        "Auth error",
        "Forbidden",
        "Internal error",
    );

    match error.error_type {
        ErrorType::RateLimit => assert!(true),
        _ => assert!(false, "Expected RateLimit"),
    }
    assert!(error.message.contains("Rate limit exceeded"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_handle_http_error_500() {
    let error = handle_http_error(
        reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        "Internal server error".to_string(),
        "test_provider",
        "Rate limit exceeded",
        "Auth error",
        "Forbidden",
        "Provider internal error occurred",
    );

    match error.error_type {
        ErrorType::ProviderError => assert!(true),
        _ => assert!(false, "Expected ProviderError"),
    }
    assert!(error.message.contains("Provider internal error"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_handle_http_error_unexpected() {
    let error = handle_http_error(
        reqwest::StatusCode::NOT_FOUND,
        "Not found".to_string(),
        "test_provider",
        "Rate limit exceeded",
        "Auth error",
        "Forbidden",
        "Internal error",
    );

    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("Unexpected error"));
    assert!(error.message.contains("Not found"));
    assert_eq!(error.provider, "test_provider");
}
