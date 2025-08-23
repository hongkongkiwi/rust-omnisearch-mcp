use omnisearch_mcp::common::provider_factory::ProviderFactory;

// Note: These tests will primarily test the structure and not the actual provider creation
// since that depends on environment variables

#[test]
fn test_provider_factory_creation() {
    // Just test that we can create the factory
    let _factory = ProviderFactory;
    assert!(true);
}

#[test]
fn test_provider_factory_get_provider_names_empty() {
    let empty_providers: Vec<Box<dyn omnisearch_mcp::common::types::SearchProvider>> = vec![];
    let names = ProviderFactory::get_provider_names(&empty_providers);
    assert_eq!(names.len(), 0);
}
