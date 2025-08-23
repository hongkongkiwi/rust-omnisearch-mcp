use std::env;

// Search provider API keys
pub fn tavily_api_key() -> Option<String> {
    env::var("TAVILY_API_KEY").ok()
}

pub fn brave_api_key() -> Option<String> {
    env::var("BRAVE_API_KEY").ok()
}

pub fn kagi_api_key() -> Option<String> {
    env::var("KAGI_API_KEY").ok()
}

pub fn google_api_key() -> Option<String> {
    env::var("GOOGLE_API_KEY").ok()
}

pub fn google_search_engine_id() -> Option<String> {
    env::var("GOOGLE_SEARCH_ENGINE_ID").ok()
}

pub fn reddit_client_id() -> Option<String> {
    env::var("REDDIT_CLIENT_ID").ok()
}

pub fn reddit_client_secret() -> Option<String> {
    env::var("REDDIT_CLIENT_SECRET").ok()
}

pub fn reddit_user_agent() -> Option<String> {
    env::var("REDDIT_USER_AGENT").ok()
}

pub fn serpapi_api_key() -> Option<String> {
    env::var("SERPAPI_API_KEY").ok()
}

pub fn brightdata_username() -> Option<String> {
    env::var("BRIGHTDATA_USERNAME").ok()
}

pub fn brightdata_password() -> Option<String> {
    env::var("BRIGHTDATA_PASSWORD").ok()
}

pub fn exa_api_key() -> Option<String> {
    env::var("EXA_API_KEY").ok()
}

// AI provider API keys
pub fn perplexity_api_key() -> Option<String> {
    env::var("PERPLEXITY_API_KEY").ok()
}

// Content processing API keys
pub fn jina_ai_api_key() -> Option<String> {
    env::var("JINA_AI_API_KEY").ok()
}

pub fn firecrawl_api_key() -> Option<String> {
    env::var("FIRECRAWL_API_KEY").ok()
}

pub fn firecrawl_base_url() -> Option<String> {
    env::var("FIRECRAWL_BASE_URL").ok()
}

// Provider configuration
pub struct Config {
    pub search: SearchConfig,
    pub ai_response: AiResponseConfig,
    pub processing: ProcessingConfig,
    pub enhancement: EnhancementConfig,
}

pub struct SearchConfig {
    pub tavily: ProviderConfig,
    pub brave: ProviderConfig,
    pub kagi: ProviderConfig,
    pub google: GoogleProviderConfig,
    pub reddit: RedditProviderConfig,
    pub duckduckgo: ProviderConfig,
    pub baidu: ProviderConfig,
    pub brightdata: BrightDataProviderConfig,
    pub exa: ProviderConfig,
}

pub struct AiResponseConfig {
    pub perplexity: ProviderConfig,
    pub kagi_fastgpt: ProviderConfig,
}

pub struct ProcessingConfig {
    pub jina_reader: ProviderConfig,
    pub kagi_summarizer: ProviderConfig,
    pub tavily_extract: ProviderConfig,
    pub firecrawl_scrape: ProviderConfig,
    pub firecrawl_crawl: ProviderConfig,
    pub firecrawl_map: ProviderConfig,
    pub firecrawl_extract: ProviderConfig,
    pub firecrawl_actions: ProviderConfig,
}

pub struct EnhancementConfig {
    pub kagi_enrichment: ProviderConfig,
    pub jina_grounding: ProviderConfig,
}

pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout: u64,
}

pub struct GoogleProviderConfig {
    pub api_key: Option<String>,
    pub search_engine_id: Option<String>,
    pub base_url: String,
    pub timeout: u64,
}

pub struct RedditProviderConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub user_agent: Option<String>,
    pub base_url: String,
    pub timeout: u64,
}

pub struct BrightDataProviderConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub base_url: String,
    pub timeout: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            search: SearchConfig {
                tavily: ProviderConfig {
                    api_key: tavily_api_key(),
                    base_url: "https://api.tavily.com".to_string(),
                    timeout: 30000,
                },
                brave: ProviderConfig {
                    api_key: brave_api_key(),
                    base_url: "https://api.search.brave.com/res/v1".to_string(),
                    timeout: 10000,
                },
                kagi: ProviderConfig {
                    api_key: kagi_api_key(),
                    base_url: "https://kagi.com/api/v0".to_string(),
                    timeout: 20000,
                },
                google: GoogleProviderConfig {
                    api_key: google_api_key(),
                    search_engine_id: google_search_engine_id(),
                    base_url: "https://www.googleapis.com/customsearch/v1".to_string(),
                    timeout: 10000,
                },
                reddit: RedditProviderConfig {
                    client_id: reddit_client_id(),
                    client_secret: reddit_client_secret(),
                    user_agent: reddit_user_agent(),
                    base_url: "https://oauth.reddit.com".to_string(),
                    timeout: 10000,
                },
                duckduckgo: ProviderConfig {
                    api_key: None,
                    base_url: "https://api.duckduckgo.com".to_string(),
                    timeout: 10000,
                },
                baidu: ProviderConfig {
                    api_key: serpapi_api_key(),
                    base_url: "https://serpapi.com".to_string(),
                    timeout: 10000,
                },
                brightdata: BrightDataProviderConfig {
                    username: brightdata_username(),
                    password: brightdata_password(),
                    base_url: "https://api.brightdata.com/serp".to_string(),
                    timeout: 10000,
                },
                exa: ProviderConfig {
                    api_key: exa_api_key(),
                    base_url: "https://api.exa.ai".to_string(),
                    timeout: 10000,
                },
            },
            ai_response: AiResponseConfig {
                perplexity: ProviderConfig {
                    api_key: perplexity_api_key(),
                    base_url: "https://api.perplexity.ai".to_string(),
                    timeout: 60000,
                },
                kagi_fastgpt: ProviderConfig {
                    api_key: kagi_api_key(),
                    base_url: "https://kagi.com/api/v0/fastgpt".to_string(),
                    timeout: 30000,
                },
            },
            processing: ProcessingConfig {
                jina_reader: ProviderConfig {
                    api_key: jina_ai_api_key(),
                    base_url: "https://api.jina.ai/v1/reader".to_string(),
                    timeout: 30000,
                },
                kagi_summarizer: ProviderConfig {
                    api_key: kagi_api_key(),
                    base_url: "https://kagi.com/api/v0/summarize".to_string(),
                    timeout: 30000,
                },
                tavily_extract: ProviderConfig {
                    api_key: tavily_api_key(),
                    base_url: "https://api.tavily.com".to_string(),
                    timeout: 30000,
                },
                firecrawl_scrape: ProviderConfig {
                    api_key: firecrawl_api_key(),
                    base_url: firecrawl_base_url()
                        .unwrap_or_else(|| "https://api.firecrawl.dev/v1/scrape".to_string()),
                    timeout: 60000,
                },
                firecrawl_crawl: ProviderConfig {
                    api_key: firecrawl_api_key(),
                    base_url: firecrawl_base_url()
                        .unwrap_or_else(|| "https://api.firecrawl.dev/v1/crawl".to_string()),
                    timeout: 120000,
                },
                firecrawl_map: ProviderConfig {
                    api_key: firecrawl_api_key(),
                    base_url: firecrawl_base_url()
                        .unwrap_or_else(|| "https://api.firecrawl.dev/v1/map".to_string()),
                    timeout: 60000,
                },
                firecrawl_extract: ProviderConfig {
                    api_key: firecrawl_api_key(),
                    base_url: firecrawl_base_url()
                        .unwrap_or_else(|| "https://api.firecrawl.dev/v1/extract".to_string()),
                    timeout: 60000,
                },
                firecrawl_actions: ProviderConfig {
                    api_key: firecrawl_api_key(),
                    base_url: firecrawl_base_url()
                        .unwrap_or_else(|| "https://api.firecrawl.dev/v1/scrape".to_string()),
                    timeout: 90000,
                },
            },
            enhancement: EnhancementConfig {
                kagi_enrichment: ProviderConfig {
                    api_key: kagi_api_key(),
                    base_url: "https://kagi.com/api/v0/enrich".to_string(),
                    timeout: 20000,
                },
                jina_grounding: ProviderConfig {
                    api_key: jina_ai_api_key(),
                    base_url: "https://api.jina.ai/v1/ground".to_string(),
                    timeout: 20000,
                },
            },
        }
    }
}

// Global config instance
pub static CONFIG: once_cell::sync::Lazy<Config> = once_cell::sync::Lazy::new(Config::new);

// Validate required environment variables
pub fn validate_config() -> anyhow::Result<()> {
    let mut missing_keys = Vec::new();
    let mut available_keys = Vec::new();

    // Check search provider keys
    if tavily_api_key().is_none() {
        missing_keys.push("TAVILY_API_KEY");
    } else {
        available_keys.push("TAVILY_API_KEY");
    }

    if brave_api_key().is_none() {
        missing_keys.push("BRAVE_API_KEY");
    } else {
        available_keys.push("BRAVE_API_KEY");
    }

    if kagi_api_key().is_none() {
        missing_keys.push("KAGI_API_KEY");
    } else {
        available_keys.push("KAGI_API_KEY");
    }

    if google_api_key().is_none() {
        missing_keys.push("GOOGLE_API_KEY");
    } else {
        available_keys.push("GOOGLE_API_KEY");
    }

    if google_search_engine_id().is_none() {
        missing_keys.push("GOOGLE_SEARCH_ENGINE_ID");
    } else {
        available_keys.push("GOOGLE_SEARCH_ENGINE_ID");
    }

    if reddit_client_id().is_none() {
        missing_keys.push("REDDIT_CLIENT_ID");
    } else {
        available_keys.push("REDDIT_CLIENT_ID");
    }

    if reddit_client_secret().is_none() {
        missing_keys.push("REDDIT_CLIENT_SECRET");
    } else {
        available_keys.push("REDDIT_CLIENT_SECRET");
    }

    if reddit_user_agent().is_none() {
        missing_keys.push("REDDIT_USER_AGENT");
    } else {
        available_keys.push("REDDIT_USER_AGENT");
    }

    if serpapi_api_key().is_none() {
        missing_keys.push("SERPAPI_API_KEY");
    } else {
        available_keys.push("SERPAPI_API_KEY");
    }

    if brightdata_username().is_none() {
        missing_keys.push("BRIGHTDATA_USERNAME");
    } else {
        available_keys.push("BRIGHTDATA_USERNAME");
    }

    if brightdata_password().is_none() {
        missing_keys.push("BRIGHTDATA_PASSWORD");
    } else {
        available_keys.push("BRIGHTDATA_PASSWORD");
    }

    if exa_api_key().is_none() {
        missing_keys.push("EXA_API_KEY");
    } else {
        available_keys.push("EXA_API_KEY");
    }

    if perplexity_api_key().is_none() {
        missing_keys.push("PERPLEXITY_API_KEY");
    } else {
        available_keys.push("PERPLEXITY_API_KEY");
    }

    if jina_ai_api_key().is_none() {
        missing_keys.push("JINA_AI_API_KEY");
    } else {
        available_keys.push("JINA_AI_API_KEY");
    }

    if firecrawl_api_key().is_none() {
        missing_keys.push("FIRECRAWL_API_KEY");
    } else {
        available_keys.push("FIRECRAWL_API_KEY");
    }

    // Log available keys
    if !available_keys.is_empty() {
        eprintln!("Found API keys for: {}", available_keys.join(", "));
    } else {
        eprintln!("Warning: No API keys found. No providers will be available.");
    }

    // Log missing keys as informational
    if !missing_keys.is_empty() {
        eprintln!(
            "Missing API keys for: {}. Some providers will not be available.",
            missing_keys.join(", ")
        );
    }

    Ok(())
}
