use eyre::{eyre, Result};
use figment::{
    providers::{Env, Format, Toml, Yaml},
    Figment,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub cache: CacheConfig,
    pub rate_limiting: RateLimitingConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
    pub providers: ProvidersConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    #[serde(rename = "type")]
    pub cache_type: CacheType,
    pub ttl_seconds: u64,
    pub max_entries: usize,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Memory,
    Redis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u64,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prometheus_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub json_format: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub half_open_max_calls: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersConfig {
    pub tavily: ProviderConfig,
    pub google: GoogleProviderConfig,
    pub reddit: RedditProviderConfig,
    pub duckduckgo: ProviderConfig,
    pub baidu: ProviderConfig,
    pub brightdata: BrightDataProviderConfig,
    pub exa: ProviderConfig,
    pub brave: ProviderConfig,
    pub kagi: ProviderConfig,
    pub perplexity: ProviderConfig,
    pub jina: ProviderConfig,
    pub firecrawl: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub rate_limit: u32,
    pub timeout_seconds: u64,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleProviderConfig {
    pub enabled: bool,
    pub api_key: Option<String>,
    pub search_engine_id: Option<String>,
    pub rate_limit: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedditProviderConfig {
    pub enabled: bool,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub user_agent: Option<String>,
    pub rate_limit: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightDataProviderConfig {
    pub enabled: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub rate_limit: u32,
    pub timeout_seconds: u64,
    pub base_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 3000,
                max_connections: 1000,
            },
            cache: CacheConfig {
                enabled: true,
                cache_type: CacheType::Memory,
                ttl_seconds: 3600,
                max_entries: 10000,
                redis: RedisConfig {
                    url: "redis://localhost:6379".to_string(),
                    pool_size: 10,
                },
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_minute: 60,
                burst_size: 10,
            },
            metrics: MetricsConfig {
                enabled: true,
                prometheus_port: 9090,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                json_format: false,
            },
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_threshold: 5,
                timeout_seconds: 60,
                half_open_max_calls: 3,
            },
            providers: ProvidersConfig::default(),
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            tavily: ProviderConfig {
                enabled: true,
                api_key: std::env::var("TAVILY_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://api.tavily.com".to_string()),
            },
            google: GoogleProviderConfig {
                enabled: true,
                api_key: std::env::var("GOOGLE_API_KEY").ok(),
                search_engine_id: std::env::var("GOOGLE_SEARCH_ENGINE_ID").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
            },
            reddit: RedditProviderConfig {
                enabled: true,
                client_id: std::env::var("REDDIT_CLIENT_ID").ok(),
                client_secret: std::env::var("REDDIT_CLIENT_SECRET").ok(),
                user_agent: std::env::var("REDDIT_USER_AGENT").ok(),
                rate_limit: 60,
                timeout_seconds: 30,
            },
            duckduckgo: ProviderConfig {
                enabled: true,
                api_key: None,
                rate_limit: 30,
                timeout_seconds: 30,
                base_url: Some("https://api.duckduckgo.com".to_string()),
            },
            baidu: ProviderConfig {
                enabled: true,
                api_key: std::env::var("SERPAPI_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://serpapi.com".to_string()),
            },
            brightdata: BrightDataProviderConfig {
                enabled: true,
                username: std::env::var("BRIGHTDATA_USERNAME").ok(),
                password: std::env::var("BRIGHTDATA_PASSWORD").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://api.brightdata.com".to_string()),
            },
            exa: ProviderConfig {
                enabled: true,
                api_key: std::env::var("EXA_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://api.exa.ai".to_string()),
            },
            brave: ProviderConfig {
                enabled: true,
                api_key: std::env::var("BRAVE_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://api.search.brave.com/res/v1".to_string()),
            },
            kagi: ProviderConfig {
                enabled: true,
                api_key: std::env::var("KAGI_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://kagi.com/api/v0".to_string()),
            },
            perplexity: ProviderConfig {
                enabled: true,
                api_key: std::env::var("PERPLEXITY_API_KEY").ok(),
                rate_limit: 60,
                timeout_seconds: 60,
                base_url: Some("https://api.perplexity.ai".to_string()),
            },
            jina: ProviderConfig {
                enabled: true,
                api_key: std::env::var("JINA_AI_API_KEY").ok(),
                rate_limit: 100,
                timeout_seconds: 30,
                base_url: Some("https://api.jina.ai".to_string()),
            },
            firecrawl: ProviderConfig {
                enabled: true,
                api_key: std::env::var("FIRECRAWL_API_KEY").ok(),
                rate_limit: 60,
                timeout_seconds: 120,
                base_url: std::env::var("FIRECRAWL_BASE_URL")
                    .ok()
                    .or_else(|| Some("https://api.firecrawl.dev".to_string())),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Yaml::file("config.yaml"))
            .merge(Yaml::file("config.yml"))
            .merge(Env::prefixed("OMNISEARCH_"))
            .extract()?;

        Ok(config)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config: Config = Figment::new()
            .merge(Toml::file(path.as_ref()))
            .merge(Env::prefixed("OMNISEARCH_"))
            .extract()?;

        Ok(config)
    }

    pub fn timeout_duration(&self, provider: &str) -> Duration {
        let seconds = match provider {
            "tavily" => self.providers.tavily.timeout_seconds,
            "google" => self.providers.google.timeout_seconds,
            "reddit" => self.providers.reddit.timeout_seconds,
            "duckduckgo" => self.providers.duckduckgo.timeout_seconds,
            "baidu" => self.providers.baidu.timeout_seconds,
            "exa" => self.providers.exa.timeout_seconds,
            "brave" => self.providers.brave.timeout_seconds,
            "kagi" => self.providers.kagi.timeout_seconds,
            "perplexity" => self.providers.perplexity.timeout_seconds,
            "jina" => self.providers.jina.timeout_seconds,
            "firecrawl" => self.providers.firecrawl.timeout_seconds,
            _ => 30, // default timeout
        };
        Duration::from_secs(seconds)
    }
}

// Global config instance
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    Config::load().unwrap_or_else(|_| {
        tracing::warn!("Failed to load configuration from files, using defaults");
        Config::default()
    })
});

// Validate configuration and log provider availability
pub fn validate_config() -> Result<()> {
    let config = &*CONFIG;
    let mut available_providers = Vec::new();
    let mut missing_providers = Vec::new();

    // Check provider availability
    if config.providers.tavily.enabled && config.providers.tavily.api_key.is_some() {
        available_providers.push("tavily");
    } else if config.providers.tavily.enabled {
        missing_providers.push("tavily (missing TAVILY_API_KEY)");
    }

    if config.providers.google.enabled && config.providers.google.api_key.is_some() {
        available_providers.push("google");
    } else if config.providers.google.enabled {
        missing_providers.push("google (missing GOOGLE_API_KEY)");
    }

    if config.providers.reddit.enabled
        && config.providers.reddit.client_id.is_some()
        && config.providers.reddit.client_secret.is_some()
    {
        available_providers.push("reddit");
    } else if config.providers.reddit.enabled {
        missing_providers.push("reddit (missing REDDIT_CLIENT_ID/CLIENT_SECRET)");
    }

    // DuckDuckGo doesn't require API key
    if config.providers.duckduckgo.enabled {
        available_providers.push("duckduckgo");
    }

    if config.providers.exa.enabled && config.providers.exa.api_key.is_some() {
        available_providers.push("exa");
    } else if config.providers.exa.enabled {
        missing_providers.push("exa (missing EXA_API_KEY)");
    }

    // Log results
    if !available_providers.is_empty() {
        tracing::info!("Available providers: {}", available_providers.join(", "));
    } else {
        tracing::warn!("No providers available. Check your API keys.");
    }

    if !missing_providers.is_empty() {
        tracing::info!("Disabled providers: {}", missing_providers.join(", "));
    }

    // Validate server configuration
    if config.server.port == 0 {
        return Err(eyre!("Server port cannot be 0"));
    }

    if config.server.max_connections == 0 {
        return Err(eyre!("Max connections cannot be 0"));
    }

    // Validate cache configuration
    if config.cache.enabled && matches!(config.cache.cache_type, CacheType::Redis) {
        if config.cache.redis.url.is_empty() {
            return Err(eyre!("Redis URL is required when using Redis cache"));
        }
    }

    Ok(())
}

// Convenience functions for backward compatibility
pub fn get_provider_api_key(provider: &str) -> Option<String> {
    match provider {
        "tavily" => CONFIG.providers.tavily.api_key.clone(),
        "google" => CONFIG.providers.google.api_key.clone(),
        "reddit_client_id" => CONFIG.providers.reddit.client_id.clone(),
        "reddit_client_secret" => CONFIG.providers.reddit.client_secret.clone(),
        "reddit_user_agent" => CONFIG.providers.reddit.user_agent.clone(),
        "exa" => CONFIG.providers.exa.api_key.clone(),
        "brave" => CONFIG.providers.brave.api_key.clone(),
        "kagi" => CONFIG.providers.kagi.api_key.clone(),
        "perplexity" => CONFIG.providers.perplexity.api_key.clone(),
        "jina" => CONFIG.providers.jina.api_key.clone(),
        "firecrawl" => CONFIG.providers.firecrawl.api_key.clone(),
        "serpapi" => CONFIG.providers.baidu.api_key.clone(),
        "brightdata_username" => CONFIG.providers.brightdata.username.clone(),
        "brightdata_password" => CONFIG.providers.brightdata.password.clone(),
        _ => None,
    }
}
