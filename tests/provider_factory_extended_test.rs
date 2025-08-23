//! Extended tests for provider factory to improve coverage

use omnisearch_mcp::common::provider_factory::ProviderFactory;
use omnisearch_mcp::common::types::SearchProvider;
use async_trait::async_trait;

#[test]
fn test_provider_factory_with_empty_provider_list() {
    // Test getting provider names from an empty list
    let empty_providers: Vec<Box<dyn SearchProvider>> = vec![];
    let names = ProviderFactory::get_provider_names(&empty_providers);
    
    assert_eq!(names.len(), 0);
    assert_eq!(names, Vec::<String>::new());
}

#[test]
fn test_provider_factory_with_single_provider() {
    // Test with a single provider
    struct TestProvider;
    
    #[async_trait]
    impl SearchProvider for TestProvider {
        fn name(&self) -> &'static str {
            "test_provider"
        }
        
        fn description(&self) -> &'static str {
            "A test provider for testing"
        }
        
        async fn search(
            &self,
            _params: omnisearch_mcp::common::types::BaseSearchParams,
        ) -> Result<Vec<omnisearch_mcp::common::types::SearchResult>, omnisearch_mcp::common::types::ProviderError> {
            Ok(vec![])
        }
    }
    
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(TestProvider {}),
    ];
    
    let names = ProviderFactory::get_provider_names(&providers);
    
    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "test_provider");
}

#[test]
fn test_provider_factory_with_duplicate_names() {
    // Test with providers that have the same name (shouldn't happen in practice, but good to test)
    struct TestProvider1;
    struct TestProvider2;
    
    #[async_trait]
    impl SearchProvider for TestProvider1 {
        fn name(&self) -> &'static str {
            "duplicate_name"
        }
        
        fn description(&self) -> &'static str {
            "First test provider"
        }
        
        async fn search(
            &self,
            _params: omnisearch_mcp::common::types::BaseSearchParams,
        ) -> Result<Vec<omnisearch_mcp::common::types::SearchResult>, omnisearch_mcp::common::types::ProviderError> {
            Ok(vec![])
        }
    }
    
    #[async_trait]
    impl SearchProvider for TestProvider2 {
        fn name(&self) -> &'static str {
            "duplicate_name"
        }
        
        fn description(&self) -> &'static str {
            "Second test provider"
        }
        
        async fn search(
            &self,
            _params: omnisearch_mcp::common::types::BaseSearchParams,
        ) -> Result<Vec<omnisearch_mcp::common::types::SearchResult>, omnisearch_mcp::common::types::ProviderError> {
            Ok(vec![])
        }
    }
    
    let providers: Vec<Box<dyn SearchProvider>> = vec![
        Box::new(TestProvider1 {}),
        Box::new(TestProvider2 {}),
    ];
    
    let names = ProviderFactory::get_provider_names(&providers);
    
    assert_eq!(names.len(), 2);
    assert_eq!(names, vec!["duplicate_name", "duplicate_name"]);
}

#[test]
fn test_provider_factory_creation_methods() {
    // Test that we can create the factory
    let _factory = ProviderFactory {};
    assert!(true);
}