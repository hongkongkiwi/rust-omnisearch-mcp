//! Comprehensive configuration validation tests

use omnisearch_mcp::config::*;
use std::env;

#[test]
fn test_config_creation_with_no_env_vars() {
    // Clear all environment variables
    let keys_to_clear = vec![
        "TAVILY_API_KEY",
        "BRAVE_API_KEY",
        "KAGI_API_KEY",
        "GOOGLE_API_KEY",
        "GOOGLE_SEARCH_ENGINE_ID",
        "REDDIT_CLIENT_ID",
        "REDDIT_CLIENT_SECRET",
        "REDDIT_USER_AGENT",
        "SERPAPI_API_KEY",
        "BRIGHTDATA_USERNAME",
        "BRIGHTDATA_PASSWORD",
        "EXA_API_KEY",
        "PERPLEXITY_API_KEY",
        "JINA_AI_API_KEY",
        "FIRECRAWL_API_KEY",
        "FIRECRAWL_BASE_URL",
    ];
    
    let mut saved_values = vec![];
    for key in &keys_to_clear {
        saved_values.push((key.to_string(), env::var(key).ok()));
        env::remove_var(key);
    }
    
    // Create config without any env vars
    let config = Config::new();
    
    // All API keys should be None
    assert!(config.search.tavily.api_key.is_none());
    assert!(config.search.brave.api_key.is_none());
    assert!(config.search.kagi.api_key.is_none());
    assert!(config.search.google.api_key.is_none());
    assert!(config.search.google.search_engine_id.is_none());
    assert!(config.search.reddit.client_id.is_none());
    assert!(config.search.reddit.client_secret.is_none());
    assert!(config.search.reddit.user_agent.is_none());
    assert!(config.search.baidu.api_key.is_none());
    assert!(config.search.brightdata.username.is_none());
    assert!(config.search.brightdata.password.is_none());
    assert!(config.search.exa.api_key.is_none());
    assert!(config.ai_response.perplexity.api_key.is_none());
    assert!(config.processing.jina_reader.api_key.is_none());
    assert!(config.processing.firecrawl_scrape.api_key.is_none());
    assert!(config.enhancement.jina_grounding.api_key.is_none());
    
    // But base URLs should be set
    assert_eq!(config.search.tavily.base_url, "https://api.tavily.com");
    assert_eq!(config.search.google.base_url, "https://www.googleapis.com/customsearch/v1");
    assert_eq!(config.search.reddit.base_url, "https://oauth.reddit.com");
    
    // Restore environment variables
    for (key, value) in saved_values {
        if let Some(val) = value {
            env::set_var(&key, val);
        }
    }
}

#[test]
fn test_config_creation_with_all_env_vars() {
    // Set all environment variables
    let test_values = vec![
        ("TAVILY_API_KEY", "test-tavily-key"),
        ("BRAVE_API_KEY", "test-brave-key"),
        ("KAGI_API_KEY", "test-kagi-key"),
        ("GOOGLE_API_KEY", "test-google-key"),
        ("GOOGLE_SEARCH_ENGINE_ID", "test-google-cx"),
        ("REDDIT_CLIENT_ID", "test-reddit-id"),
        ("REDDIT_CLIENT_SECRET", "test-reddit-secret"),
        ("REDDIT_USER_AGENT", "test-user-agent"),
        ("SERPAPI_API_KEY", "test-serpapi-key"),
        ("BRIGHTDATA_USERNAME", "test-username"),
        ("BRIGHTDATA_PASSWORD", "test-password"),
        ("EXA_API_KEY", "test-exa-key"),
        ("PERPLEXITY_API_KEY", "test-perplexity-key"),
        ("JINA_AI_API_KEY", "test-jina-key"),
        ("FIRECRAWL_API_KEY", "test-firecrawl-key"),
        ("FIRECRAWL_BASE_URL", "https://custom.firecrawl.url"),
    ];
    
    // Save original values
    let mut saved_values = vec![];
    for (key, value) in &test_values {
        saved_values.push((key.to_string(), env::var(key).ok()));
        env::set_var(key, value);
    }
    
    // Create config with all env vars set
    let config = Config::new();
    
    // All API keys should be Some
    assert_eq!(config.search.tavily.api_key, Some("test-tavily-key".to_string()));
    assert_eq!(config.search.brave.api_key, Some("test-brave-key".to_string()));
    assert_eq!(config.search.kagi.api_key, Some("test-kagi-key".to_string()));
    assert_eq!(config.search.google.api_key, Some("test-google-key".to_string()));
    assert_eq!(config.search.google.search_engine_id, Some("test-google-cx".to_string()));
    assert_eq!(config.search.reddit.client_id, Some("test-reddit-id".to_string()));
    assert_eq!(config.search.reddit.client_secret, Some("test-reddit-secret".to_string()));
    assert_eq!(config.search.reddit.user_agent, Some("test-user-agent".to_string()));
    assert_eq!(config.search.baidu.api_key, Some("test-serpapi-key".to_string()));
    assert_eq!(config.search.brightdata.username, Some("test-username".to_string()));
    assert_eq!(config.search.brightdata.password, Some("test-password".to_string()));
    assert_eq!(config.search.exa.api_key, Some("test-exa-key".to_string()));
    assert_eq!(config.ai_response.perplexity.api_key, Some("test-perplexity-key".to_string()));
    assert_eq!(config.processing.jina_reader.api_key, Some("test-jina-key".to_string()));
    assert_eq!(config.processing.firecrawl_scrape.api_key, Some("test-firecrawl-key".to_string()));
    assert_eq!(config.enhancement.jina_grounding.api_key, Some("test-jina-key".to_string()));
    
    // Custom base URL should be used
    assert!(config.processing.firecrawl_scrape.base_url.contains("custom.firecrawl.url"));
    
    // Restore environment variables
    for (key, value) in saved_values {
        if let Some(val) = value {
            env::set_var(&key, val);
        } else {
            env::remove_var(&key);
        }
    }
}

#[test]
fn test_config_timeouts() {
    let config = Config::new();
    
    // Verify timeout values are reasonable
    assert!(config.search.tavily.timeout > 0);
    assert!(config.search.tavily.timeout <= 60000); // 1 minute max
    
    // Long-running operations should have higher timeouts
    assert!(config.processing.firecrawl_crawl.timeout >= config.search.tavily.timeout);
    assert!(config.processing.firecrawl_actions.timeout >= config.search.tavily.timeout);
    
    // AI response timeouts should be reasonable
    assert!(config.ai_response.perplexity.timeout >= 30000); // At least 30 seconds
}

#[test]
fn test_config_base_urls_are_valid() {
    let config = Config::new();
    
    let urls = vec![
        &config.search.tavily.base_url,
        &config.search.brave.base_url,
        &config.search.kagi.base_url,
        &config.search.google.base_url,
        &config.search.reddit.base_url,
        &config.search.duckduckgo.base_url,
        &config.search.baidu.base_url,
        &config.search.brightdata.base_url,
        &config.search.exa.base_url,
        &config.ai_response.perplexity.base_url,
        &config.ai_response.kagi_fastgpt.base_url,
        &config.processing.jina_reader.base_url,
        &config.processing.kagi_summarizer.base_url,
        &config.processing.tavily_extract.base_url,
    ];
    
    for url in urls {
        assert!(url.starts_with("https://"), "URL should use HTTPS: {}", url);
        assert!(!url.ends_with('/'), "URL should not end with slash: {}", url);
        assert!(url.len() > 10, "URL should be substantial: {}", url);
    }
}

#[test]
fn test_individual_env_var_getters() {
    // Test all individual getter functions
    env::set_var("TAVILY_API_KEY", "test-value");
    assert_eq!(tavily_api_key(), Some("test-value".to_string()));
    env::remove_var("TAVILY_API_KEY");
    assert_eq!(tavily_api_key(), None);
    
    env::set_var("BRAVE_API_KEY", "brave-value");
    assert_eq!(brave_api_key(), Some("brave-value".to_string()));
    env::remove_var("BRAVE_API_KEY");
    assert_eq!(brave_api_key(), None);
    
    env::set_var("KAGI_API_KEY", "kagi-value");
    assert_eq!(kagi_api_key(), Some("kagi-value".to_string()));
    env::remove_var("KAGI_API_KEY");
    assert_eq!(kagi_api_key(), None);
    
    env::set_var("GOOGLE_API_KEY", "google-value");
    assert_eq!(google_api_key(), Some("google-value".to_string()));
    env::remove_var("GOOGLE_API_KEY");
    assert_eq!(google_api_key(), None);
    
    env::set_var("GOOGLE_SEARCH_ENGINE_ID", "google-cx");
    assert_eq!(google_search_engine_id(), Some("google-cx".to_string()));
    env::remove_var("GOOGLE_SEARCH_ENGINE_ID");
    assert_eq!(google_search_engine_id(), None);
    
    // Test Reddit configs
    env::set_var("REDDIT_CLIENT_ID", "reddit-id");
    env::set_var("REDDIT_CLIENT_SECRET", "reddit-secret");
    env::set_var("REDDIT_USER_AGENT", "reddit-agent");
    assert_eq!(reddit_client_id(), Some("reddit-id".to_string()));
    assert_eq!(reddit_client_secret(), Some("reddit-secret".to_string()));
    assert_eq!(reddit_user_agent(), Some("reddit-agent".to_string()));
    env::remove_var("REDDIT_CLIENT_ID");
    env::remove_var("REDDIT_CLIENT_SECRET");
    env::remove_var("REDDIT_USER_AGENT");
    
    // Test other providers
    env::set_var("SERPAPI_API_KEY", "serpapi-value");
    assert_eq!(serpapi_api_key(), Some("serpapi-value".to_string()));
    env::remove_var("SERPAPI_API_KEY");
    
    env::set_var("BRIGHTDATA_USERNAME", "bd-user");
    env::set_var("BRIGHTDATA_PASSWORD", "bd-pass");
    assert_eq!(brightdata_username(), Some("bd-user".to_string()));
    assert_eq!(brightdata_password(), Some("bd-pass".to_string()));
    env::remove_var("BRIGHTDATA_USERNAME");
    env::remove_var("BRIGHTDATA_PASSWORD");
    
    env::set_var("EXA_API_KEY", "exa-value");
    assert_eq!(exa_api_key(), Some("exa-value".to_string()));
    env::remove_var("EXA_API_KEY");
    
    env::set_var("PERPLEXITY_API_KEY", "perplexity-value");
    assert_eq!(perplexity_api_key(), Some("perplexity-value".to_string()));
    env::remove_var("PERPLEXITY_API_KEY");
    
    env::set_var("JINA_AI_API_KEY", "jina-value");
    assert_eq!(jina_ai_api_key(), Some("jina-value".to_string()));
    env::remove_var("JINA_AI_API_KEY");
    
    env::set_var("FIRECRAWL_API_KEY", "firecrawl-value");
    env::set_var("FIRECRAWL_BASE_URL", "https://custom.firecrawl.dev");
    assert_eq!(firecrawl_api_key(), Some("firecrawl-value".to_string()));
    assert_eq!(firecrawl_base_url(), Some("https://custom.firecrawl.dev".to_string()));
    env::remove_var("FIRECRAWL_API_KEY");
    env::remove_var("FIRECRAWL_BASE_URL");
}

#[test]
fn test_validate_config_with_no_keys() {
    // Clear all environment variables
    let keys_to_clear = vec![
        "TAVILY_API_KEY",
        "BRAVE_API_KEY",
        "KAGI_API_KEY",
        "GOOGLE_API_KEY",
        "GOOGLE_SEARCH_ENGINE_ID",
        "REDDIT_CLIENT_ID",
        "REDDIT_CLIENT_SECRET",
        "REDDIT_USER_AGENT",
        "SERPAPI_API_KEY",
        "BRIGHTDATA_USERNAME",
        "BRIGHTDATA_PASSWORD",
        "EXA_API_KEY",
        "PERPLEXITY_API_KEY",
        "JINA_AI_API_KEY",
        "FIRECRAWL_API_KEY",
    ];
    
    let mut saved_values = vec![];
    for key in &keys_to_clear {
        saved_values.push((key.to_string(), env::var(key).ok()));
        env::remove_var(key);
    }
    
    // Validation should still succeed but log warnings
    let result = validate_config();
    assert!(result.is_ok());
    
    // Restore environment variables
    for (key, value) in saved_values {
        if let Some(val) = value {
            env::set_var(&key, val);
        }
    }
}

#[test]
fn test_validate_config_with_some_keys() {
    // Set only some keys
    let keys_to_set = vec![
        ("TAVILY_API_KEY", "test-tavily"),
        ("GOOGLE_API_KEY", "test-google"),
        ("KAGI_API_KEY", "test-kagi"),
    ];
    
    let keys_to_clear = vec![
        "BRAVE_API_KEY",
        "REDDIT_CLIENT_ID",
        "REDDIT_CLIENT_SECRET",
        "SERPAPI_API_KEY",
        "BRIGHTDATA_USERNAME",
        "BRIGHTDATA_PASSWORD",
        "EXA_API_KEY",
        "PERPLEXITY_API_KEY",
        "JINA_AI_API_KEY",
        "FIRECRAWL_API_KEY",
    ];
    
    // Save original state
    let mut saved_values = vec![];
    for (key, _) in &keys_to_set {
        saved_values.push((key.to_string(), env::var(key).ok()));
    }
    for key in &keys_to_clear {
        saved_values.push((key.to_string(), env::var(key).ok()));
        env::remove_var(key);
    }
    
    // Set test keys
    for (key, value) in &keys_to_set {
        env::set_var(key, value);
    }
    
    // Validation should succeed
    let result = validate_config();
    assert!(result.is_ok());
    
    // Restore environment variables
    for (key, value) in saved_values {
        if let Some(val) = value {
            env::set_var(&key, val);
        } else {
            env::remove_var(&key);
        }
    }
}

#[test]
fn test_duckduckgo_no_api_key_required() {
    let config = Config::new();
    
    // DuckDuckGo should explicitly not require an API key
    assert!(config.search.duckduckgo.api_key.is_none());
    assert!(!config.search.duckduckgo.base_url.is_empty());
    assert!(config.search.duckduckgo.timeout > 0);
}

#[test]
fn test_firecrawl_base_url_customization() {
    // Test without custom base URL
    env::remove_var("FIRECRAWL_BASE_URL");
    let config1 = Config::new();
    assert!(config1.processing.firecrawl_scrape.base_url.contains("api.firecrawl.dev"));
    
    // Test with custom base URL
    env::set_var("FIRECRAWL_BASE_URL", "https://custom.firecrawl.com");
    let config2 = Config::new();
    assert!(config2.processing.firecrawl_scrape.base_url.contains("custom.firecrawl.com"));
    
    env::remove_var("FIRECRAWL_BASE_URL");
}

#[test]
fn test_config_struct_sizes() {
    // Ensure config structs are reasonable in size
    use std::mem;
    
    assert!(mem::size_of::<Config>() < 10000); // Should be reasonable
    assert!(mem::size_of::<ProviderConfig>() < 1000);
    assert!(mem::size_of::<GoogleProviderConfig>() < 1000);
    assert!(mem::size_of::<RedditProviderConfig>() < 1000);
    assert!(mem::size_of::<BrightDataProviderConfig>() < 1000);
}

#[test]
fn test_global_config_lazy_initialization() {
    // Test that the global CONFIG can be accessed
    let config = &*CONFIG;
    
    // Should have valid base URLs
    assert!(!config.search.tavily.base_url.is_empty());
    assert!(!config.search.google.base_url.is_empty());
    
    // Should have reasonable timeouts
    assert!(config.search.tavily.timeout > 0);
    assert!(config.ai_response.perplexity.timeout >= 30000);
}