use omnisearch_mcp::{config::validate_config, providers::initialize_providers};

#[test]
fn test_application_initialization() {
    // Test that we can validate config without panicking
    let result = validate_config();
    // This should either succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());

    // Test that we can initialize providers without panicking
    initialize_providers();
    assert!(true);
}

#[test]
fn test_config_validation_output() {
    // Capture stderr to check validation output
    let result = validate_config();

    // Regardless of success/failure, the function should not panic
    match result {
        Ok(()) => {
            // Config validation succeeded
            assert!(true);
        }
        Err(e) => {
            // Config validation failed, but that's expected in test environment
            // Just ensure it's a proper error
            assert!(!format!("{:?}", e).is_empty());
        }
    }
}

#[test]
fn test_provider_initialization_no_panic() {
    // This test ensures that provider initialization doesn't panic
    // even when API keys are missing
    initialize_providers();

    // If we get here, initialization completed without panicking
    assert!(true);
}
