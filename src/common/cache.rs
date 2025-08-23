use async_trait::async_trait;
use eyre::Result;
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

#[cfg(feature = "caching")]
use redis::aio::ConnectionManager;

use crate::common::types::SearchResult;
use crate::config::{CacheConfig, CacheType, CONFIG};

pub type CacheKey = String;
pub type CacheValue = Vec<SearchResult>;

#[async_trait]
pub trait CacheProvider: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<CacheValue>>;
    async fn set(&self, key: &str, value: CacheValue, ttl: Duration) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn size(&self) -> Result<usize>;
}

pub struct MemoryCache {
    cache: MokaCache<String, CacheValue>,
}

impl MemoryCache {
    pub fn new(config: &CacheConfig) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(config.max_entries as u64)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .build();

        Self { cache }
    }
}

#[async_trait]
impl CacheProvider for MemoryCache {
    async fn get(&self, key: &str) -> Result<Option<CacheValue>> {
        let value = self.cache.get(key).await;
        debug!(
            "Cache {} for key: {}",
            if value.is_some() { "hit" } else { "miss" },
            key
        );
        Ok(value)
    }

    async fn set(&self, key: &str, value: CacheValue, _ttl: Duration) -> Result<()> {
        self.cache.insert(key.to_string(), value).await;
        debug!("Cached {} results for key: {}", value.len(), key);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.cache.remove(key).await;
        debug!("Removed cache entry for key: {}", key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.cache.invalidate_all();
        info!("Cleared memory cache");
        Ok(())
    }

    async fn size(&self) -> Result<usize> {
        Ok(self.cache.entry_count() as usize)
    }
}

#[cfg(feature = "caching")]
pub struct RedisCache {
    connection_manager: ConnectionManager,
    ttl: Duration,
}

#[cfg(feature = "caching")]
impl RedisCache {
    pub async fn new(config: &CacheConfig) -> Result<Self> {
        use redis::Client;

        let client = Client::open(config.redis.url.as_str())?;
        let connection_manager = ConnectionManager::new(client).await?;

        info!("Connected to Redis cache at: {}", config.redis.url);

        Ok(Self {
            connection_manager,
            ttl: Duration::from_secs(config.ttl_seconds),
        })
    }
}

#[cfg(feature = "caching")]
#[async_trait]
impl CacheProvider for RedisCache {
    async fn get(&self, key: &str) -> Result<Option<CacheValue>> {
        use redis::AsyncCommands;

        let mut conn = self.connection_manager.clone();
        let cached_data: Option<String> = conn.get(key).await?;

        match cached_data {
            Some(data) => {
                match serde_json::from_str::<CacheValue>(&data) {
                    Ok(value) => {
                        debug!("Cache hit for key: {}", key);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        error!("Failed to deserialize cached data for key {}: {}", key, e);
                        // Remove corrupted cache entry
                        let _: () = conn.del(key).await?;
                        Ok(None)
                    }
                }
            }
            None => {
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
        }
    }

    async fn set(&self, key: &str, value: CacheValue, ttl: Duration) -> Result<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_manager.clone();
        let serialized = serde_json::to_string(&value)?;

        let _: () = conn.set_ex(key, serialized, ttl.as_secs()).await?;
        debug!("Cached {} results for key: {}", value.len(), key);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_manager.clone();
        let _: () = conn.del(key).await?;
        debug!("Removed cache entry for key: {}", key);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_manager.clone();
        let _: () = conn.cmd("FLUSHALL").query_async(&mut conn).await?;
        info!("Cleared Redis cache");
        Ok(())
    }

    async fn size(&self) -> Result<usize> {
        use redis::AsyncCommands;

        let mut conn = self.connection_manager.clone();
        let dbsize: u64 = conn.cmd("DBSIZE").query_async(&mut conn).await?;
        Ok(dbsize as usize)
    }
}

pub struct CacheManager {
    provider: Box<dyn CacheProvider>,
    enabled: bool,
}

impl CacheManager {
    pub async fn new() -> Result<Self> {
        let config = &CONFIG.cache;

        if !config.enabled {
            info!("Cache disabled");
            return Ok(Self {
                provider: Box::new(MemoryCache::new(config)),
                enabled: false,
            });
        }

        let provider: Box<dyn CacheProvider> = match config.cache_type {
            CacheType::Memory => {
                info!("Using memory cache with {} max entries", config.max_entries);
                Box::new(MemoryCache::new(config))
            }
            #[cfg(feature = "caching")]
            CacheType::Redis => {
                info!("Using Redis cache");
                Box::new(RedisCache::new(config).await?)
            }
            #[cfg(not(feature = "caching"))]
            CacheType::Redis => {
                error!(
                    "Redis cache requested but caching feature not enabled, falling back to memory"
                );
                Box::new(MemoryCache::new(config))
            }
        };

        Ok(Self {
            provider,
            enabled: config.enabled,
        })
    }

    pub fn generate_cache_key(provider: &str, query: &str, params: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{}:{}:{}", provider, query, params).hash(&mut hasher);
        format!("omnisearch:{}:{:x}", provider, hasher.finish())
    }

    pub async fn get(&self, key: &str) -> Result<Option<CacheValue>> {
        if !self.enabled {
            return Ok(None);
        }
        self.provider.get(key).await
    }

    pub async fn set(&self, key: &str, value: CacheValue) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let ttl = Duration::from_secs(CONFIG.cache.ttl_seconds);
        self.provider.set(key, value, ttl).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.provider.delete(key).await
    }

    pub async fn clear(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        self.provider.clear().await
    }

    pub async fn size(&self) -> Result<usize> {
        if !self.enabled {
            return Ok(0);
        }
        self.provider.size().await
    }
}

// Global cache manager instance
use once_cell::sync::OnceCell;

static CACHE_MANAGER: OnceCell<CacheManager> = OnceCell::new();

pub async fn get_cache_manager() -> &'static CacheManager {
    CACHE_MANAGER
        .get_or_init(|| async {
            CacheManager::new().await.unwrap_or_else(|e| {
                error!("Failed to initialize cache manager: {}", e);
                CacheManager {
                    provider: Box::new(MemoryCache::new(&CONFIG.cache)),
                    enabled: false,
                }
            })
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::SearchResult;

    fn create_test_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "Test Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: Some("Test snippet 1".to_string()),
                ..Default::default()
            },
            SearchResult {
                title: "Test Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                snippet: Some("Test snippet 2".to_string()),
                ..Default::default()
            },
        ]
    }

    #[tokio::test]
    async fn test_memory_cache_operations() {
        let config = CacheConfig {
            enabled: true,
            cache_type: CacheType::Memory,
            ttl_seconds: 60,
            max_entries: 100,
            redis: Default::default(),
        };

        let cache = MemoryCache::new(&config);
        let test_results = create_test_results();

        // Test set and get
        cache
            .set("test_key", test_results.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved = cache.get("test_key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 2);

        // Test delete
        cache.delete("test_key").await.unwrap();
        let retrieved = cache.get("test_key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheManager::generate_cache_key("tavily", "rust programming", "limit=10");
        let key2 = CacheManager::generate_cache_key("tavily", "rust programming", "limit=10");
        let key3 = CacheManager::generate_cache_key("tavily", "python programming", "limit=10");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert!(key1.starts_with("omnisearch:tavily:"));
    }
}
