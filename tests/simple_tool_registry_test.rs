//! Simple tests for the tool registry system

use omnisearch_mcp::server::tools::{AvailableProviders, ToolRegistry, AVAILABLE_PROVIDERS};

#[test]
fn test_available_providers_creation() {
    let _providers = AvailableProviders::new();

    // All categories should start empty
    assert_eq!(providers.search.read().unwrap().len(), 0);
    assert_eq!(providers.ai_response.read().unwrap().len(), 0);
    assert_eq!(providers.processing.read().unwrap().len(), 0);
    assert_eq!(providers.enhancement.read().unwrap().len(), 0);
}

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();

    // Registry should be created successfully - we can't access private fields
    // but we can test that creation doesn't panic
    // Test passes if compilation succeeds
}

#[test]
fn test_global_available_providers_access() {
    // Test that we can access the global available providers
    let search_count = AVAILABLE_PROVIDERS.search.read().unwrap().len();
    let ai_count = AVAILABLE_PROVIDERS.ai_response.read().unwrap().len();
    let processing_count = AVAILABLE_PROVIDERS.processing.read().unwrap().len();
    let enhancement_count = AVAILABLE_PROVIDERS.enhancement.read().unwrap().len();

    // Counts should be non-negative (trivial but tests access)
    assert!(search_count >= 0);
    assert!(ai_count >= 0);
    assert!(processing_count >= 0);
    assert!(enhancement_count >= 0);
}

#[test]
fn test_available_providers_thread_safety() {
    use std::thread;

    let mut handles = vec![];

    // Access available providers from multiple threads
    for i in 0..5 {
        let handle = thread::spawn(move || {
            {
                let mut search = AVAILABLE_PROVIDERS.search.write().unwrap();
                search.insert(format!("thread-search-{}", i));
            }
            {
                let search = AVAILABLE_PROVIDERS.search.read().unwrap();
                assert!(!search.is_empty());
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should have at least some providers registered
    assert!(AVAILABLE_PROVIDERS.search.read().unwrap().len() >= 5);
}

#[test]
fn test_available_providers_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let _providers = Arc::new(AvailableProviders::new());
    let mut handles = vec![];

    // Test concurrent read/write access
    for i in 0..3 {
        let _providers_clone = Arc::clone(&providers);
        let handle = thread::spawn(move || {
            // Write to different categories
            match i {
                0 => {
                    let mut search = providers_clone.search.write().unwrap();
                    search.insert(format!("concurrent-search-{}", i));
                }
                1 => {
                    let mut ai = providers_clone.ai_response.write().unwrap();
                    ai.insert(format!("concurrent-ai-{}", i));
                }
                _ => {
                    let mut processing = providers_clone.processing.write().unwrap();
                    processing.insert(format!("concurrent-processing-{}", i));
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify data was written correctly
    assert!(providers
        .search
        .read()
        .unwrap()
        .contains("concurrent-search-0"));
    assert!(providers
        .ai_response
        .read()
        .unwrap()
        .contains("concurrent-ai-1"));
    assert!(providers
        .processing
        .read()
        .unwrap()
        .contains("concurrent-processing-2"));
}

#[test]
fn test_provider_names_deduplication() {
    let _providers = AvailableProviders::new();

    // Add duplicate names to search providers
    {
        let mut search = providers.search.write().unwrap();
        search.insert("duplicate-name".to_string());
        search.insert("duplicate-name".to_string()); // HashSet should deduplicate
        search.insert("unique-name".to_string());
    }

    // Should only have 2 unique entries
    assert_eq!(providers.search.read().unwrap().len(), 2);
    assert!(providers.search.read().unwrap().contains("duplicate-name"));
    assert!(providers.search.read().unwrap().contains("unique-name"));
}

#[test]
fn test_available_providers_categories_independent() {
    let _providers = AvailableProviders::new();

    // Add providers to different categories
    {
        let mut search = providers.search.write().unwrap();
        search.insert("provider-1".to_string());
    }
    {
        let mut ai = providers.ai_response.write().unwrap();
        ai.insert("provider-2".to_string());
    }
    {
        let mut processing = providers.processing.write().unwrap();
        processing.insert("provider-3".to_string());
    }
    {
        let mut enhancement = providers.enhancement.write().unwrap();
        enhancement.insert("provider-4".to_string());
    }

    // Each category should have exactly one provider
    assert_eq!(providers.search.read().unwrap().len(), 1);
    assert_eq!(providers.ai_response.read().unwrap().len(), 1);
    assert_eq!(providers.processing.read().unwrap().len(), 1);
    assert_eq!(providers.enhancement.read().unwrap().len(), 1);

    // Check correct providers are in correct categories
    assert!(providers.search.read().unwrap().contains("provider-1"));
    assert!(providers.ai_response.read().unwrap().contains("provider-2"));
    assert!(providers.processing.read().unwrap().contains("provider-3"));
    assert!(providers.enhancement.read().unwrap().contains("provider-4"));
}
