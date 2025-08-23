use omnisearch_mcp::common::types::*;

#[test]
fn test_provider_error_creation() {
    let error = ProviderError::new(
        ErrorType::ApiError,
        "Test API error".to_string(),
        "test_provider".to_string(),
        None,
    );
    
    // Use match instead of assert_eq for ErrorType
    match error.error_type {
        ErrorType::ApiError => assert!(true),
        _ => assert!(false, "Expected ApiError"),
    }
    assert_eq!(error.message, "Test API error");
    assert_eq!(error.provider, "test_provider");
    assert!(error.source.is_none());
}

#[test]
fn test_provider_error_with_source() {
    let source_error = anyhow::anyhow!("Source error");
    let error = ProviderError::new(
        ErrorType::RateLimit,
        "Rate limit exceeded".to_string(),
        "test_provider".to_string(),
        Some(source_error),
    );
    
    match error.error_type {
        ErrorType::RateLimit => assert!(true),
        _ => assert!(false, "Expected RateLimit"),
    }
    assert_eq!(error.message, "Rate limit exceeded");
    assert_eq!(error.provider, "test_provider");
    assert!(error.source.is_some());
}

#[test]
fn test_error_type_display() {
    assert_eq!(format!("{}", ErrorType::ApiError), "API Error");
    assert_eq!(format!("{}", ErrorType::RateLimit), "Rate Limit");
    assert_eq!(format!("{}", ErrorType::InvalidInput), "Invalid Input");
    assert_eq!(format!("{}", ErrorType::ProviderError), "Provider Error");
}

#[test]
fn test_provider_error_display() {
    let error = ProviderError::new(
        ErrorType::ApiError,
        "Test error".to_string(),
        "test_provider".to_string(),
        None,
    );
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("Provider error"));
    assert!(error_string.contains("Test error"));
    assert!(error_string.contains("test_provider"));
}

#[test]
fn test_search_result_with_score() {
    let result = SearchResult {
        title: "Test Title".to_string(),
        url: "https://example.com".to_string(),
        snippet: "Test snippet".to_string(),
        score: Some(0.85),
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "Test snippet");
    assert_eq!(result.score, Some(0.85));
    assert_eq!(result.source_provider, "test_provider");
}

#[test]
fn test_search_result_without_score() {
    let result = SearchResult {
        title: "Test Title".to_string(),
        url: "https://example.com".to_string(),
        snippet: "Test snippet".to_string(),
        score: None,
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.title, "Test Title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "Test snippet");
    assert_eq!(result.score, None);
    assert_eq!(result.source_provider, "test_provider");
}

#[test]
fn test_base_search_params_with_all_fields() {
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: Some(10),
        include_domains: Some(vec!["example.com".to_string(), "test.com".to_string()]),
        exclude_domains: Some(vec!["exclude.com".to_string()]),
    };
    
    assert_eq!(params.query, "test query");
    assert_eq!(params.limit, Some(10));
    assert_eq!(params.include_domains, Some(vec!["example.com".to_string(), "test.com".to_string()]));
    assert_eq!(params.exclude_domains, Some(vec!["exclude.com".to_string()]));
}

#[test]
fn test_base_search_params_with_none_fields() {
    let params = BaseSearchParams {
        query: "test query".to_string(),
        limit: None,
        include_domains: None,
        exclude_domains: None,
    };
    
    assert_eq!(params.query, "test query");
    assert_eq!(params.limit, None);
    assert_eq!(params.include_domains, None);
    assert_eq!(params.exclude_domains, None);
}

#[test]
fn test_processing_result_struct() {
    let result = ProcessingResult {
        content: "Processed content".to_string(),
        raw_contents: Some(vec![
            RawContent {
                url: "https://example.com".to_string(),
                content: "Raw content".to_string(),
            }
        ]),
        metadata: ProcessingMetadata {
            title: Some("Test Title".to_string()),
            author: Some("Test Author".to_string()),
            date: Some("2023-01-01".to_string()),
            word_count: Some(100),
            failed_urls: Some(vec!["https://failed.com".to_string()]),
            urls_processed: Some(5),
            successful_extractions: Some(3),
            extract_depth: Some("advanced".to_string()),
        },
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.content, "Processed content");
    assert!(result.raw_contents.is_some());
    assert_eq!(result.source_provider, "test_provider");
    
    let raw_content = result.raw_contents.unwrap();
    assert_eq!(raw_content.len(), 1);
    assert_eq!(raw_content[0].url, "https://example.com");
    assert_eq!(raw_content[0].content, "Raw content");
    
    let metadata = result.metadata;
    assert_eq!(metadata.title, Some("Test Title".to_string()));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.date, Some("2023-01-01".to_string()));
    assert_eq!(metadata.word_count, Some(100));
    assert_eq!(metadata.failed_urls, Some(vec!["https://failed.com".to_string()]));
    assert_eq!(metadata.urls_processed, Some(5));
    assert_eq!(metadata.successful_extractions, Some(3));
    assert_eq!(metadata.extract_depth, Some("advanced".to_string()));
}

#[test]
fn test_enhancement_result_struct() {
    let result = EnhancementResult {
        original_content: "Original content".to_string(),
        enhanced_content: "Enhanced content".to_string(),
        enhancements: vec![
            Enhancement {
                r#type: "summary".to_string(),
                description: "Added summary".to_string(),
            }
        ],
        sources: Some(vec![
            EnhancementSource {
                title: "Source Title".to_string(),
                url: "https://source.com".to_string(),
            }
        ]),
        source_provider: "test_provider".to_string(),
    };
    
    assert_eq!(result.original_content, "Original content");
    assert_eq!(result.enhanced_content, "Enhanced content");
    assert_eq!(result.source_provider, "test_provider");
    assert_eq!(result.enhancements.len(), 1);
    assert!(result.sources.is_some());
    
    let enhancement = &result.enhancements[0];
    assert_eq!(enhancement.r#type, "summary");
    assert_eq!(enhancement.description, "Added summary");
    
    let sources = result.sources.unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].title, "Source Title");
    assert_eq!(sources[0].url, "https://source.com");
}