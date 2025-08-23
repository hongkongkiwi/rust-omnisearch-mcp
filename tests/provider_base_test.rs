use omnisearch_mcp::common::provider_base::*;
use omnisearch_mcp::common::types::ErrorType;

#[test]
fn test_api_key_provider_valid_key() {
    struct TestProvider;
    impl ApiKeyProvider for TestProvider {}

    let provider = TestProvider;
    let api_key = Some("test_key".to_string());

    let result = provider.validate_api_key(api_key.as_ref(), "test_provider");
    assert!(result.is_ok());
}

#[test]
fn test_api_key_provider_missing_key() {
    struct TestProvider;
    impl ApiKeyProvider for TestProvider {}

    let provider = TestProvider;
    let api_key: Option<String> = None;

    let result = provider.validate_api_key(api_key.as_ref(), "test_provider");
    assert!(result.is_err());

    let error = result.unwrap_err();
    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("Missing API key"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_multi_credential_provider_all_valid() {
    struct TestProvider;
    impl MultiCredentialProvider for TestProvider {}

    let provider = TestProvider;
    let cred1 = "cred1".to_string();
    let cred2 = "cred2".to_string();
    let cred3 = "cred3".to_string();
    let credentials = vec![Some(&cred1), Some(&cred2), Some(&cred3)];
    let error_messages = vec!["Missing cred1", "Missing cred2", "Missing cred3"];

    let result = provider.validate_credentials(credentials, error_messages, "test_provider");
    assert!(result.is_ok());
}

#[test]
fn test_multi_credential_provider_missing_first() {
    struct TestProvider;
    impl MultiCredentialProvider for TestProvider {}

    let provider = TestProvider;
    let cred2 = "cred2".to_string();
    let cred3 = "cred3".to_string();
    let credentials = vec![None, Some(&cred2), Some(&cred3)];
    let error_messages = vec!["Missing cred1", "Missing cred2", "Missing cred3"];

    let result = provider.validate_credentials(credentials, error_messages, "test_provider");
    assert!(result.is_err());

    let error = result.unwrap_err();
    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("Missing cred1"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_multi_credential_provider_missing_middle() {
    struct TestProvider;
    impl MultiCredentialProvider for TestProvider {}

    let provider = TestProvider;
    let cred1 = "cred1".to_string();
    let cred3 = "cred3".to_string();
    let credentials = vec![Some(&cred1), None, Some(&cred3)];
    let error_messages = vec!["Missing cred1", "Missing cred2", "Missing cred3"];

    let result = provider.validate_credentials(credentials, error_messages, "test_provider");
    assert!(result.is_err());

    let error = result.unwrap_err();
    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert!(error.message.contains("Missing cred2"));
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_provider_utils_provider_error() {
    let error = ProviderUtils::provider_error(
        ErrorType::ApiError,
        "Test error message".to_string(),
        "test_provider".to_string(),
    );

    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert_eq!(error.message, "Test error message");
    assert_eq!(error.provider, "test_provider");
}

#[test]
fn test_provider_utils_join_domains() {
    let domains = vec![
        "example.com".to_string(),
        "test.com".to_string(),
        "demo.org".to_string(),
    ];

    let result = ProviderUtils::join_domains(&domains);
    assert_eq!(result, "example.com,test.com,demo.org");
}

#[test]
fn test_provider_utils_create_site_filter() {
    let domains = vec!["example.com".to_string(), "test.com".to_string()];

    let result = ProviderUtils::create_site_filter(&domains);
    assert_eq!(result, "site:example.com OR site:test.com");
}

#[test]
fn test_provider_utils_empty_collections() {
    // Test with empty domains
    let empty_domains: Vec<String> = vec![];
    let result = ProviderUtils::join_domains(&empty_domains);
    assert_eq!(result, "");

    let result2 = ProviderUtils::create_site_filter(&empty_domains);
    assert_eq!(result2, "");
}
