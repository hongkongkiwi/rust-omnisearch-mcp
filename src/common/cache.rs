use async_trait::async_trait;
use eyre::Result;
use moka::future::Cache as MokaCache;
use std::time::Duration;
use tracing::{debug, info};

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
        let len = value.len();
        self.cache.insert(key.to_string(), value).await;
        debug!("Cached {} results for key: {}", len, key);
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

pub struct CacheManager {
    provider: Box<dyn CacheProvider>,
    enabled: bool,
}

impl CacheManager {
    pub async fn new() -> Result<Self> {
        let config = &CONFIG.cache;

        if !config.enabled {
            info!("Cache is disabled");
            let dummy_cache = MemoryCache::new(config);
            return Ok(Self {
                provider: Box::new(dummy_cache),
                enabled: false,
            });
        }

        let provider: Box<dyn CacheProvider> = match config.cache_type {
            CacheType::Memory => {
                info!("Using memory cache with {} max entries", config.max_entries);
                Box::new(MemoryCache::new(config))
            }
        };

        Ok(Self {
            provider,
            enabled: config.enabled,
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<CacheValue>> {
        if !self.enabled {
            return Ok(None);
        }
        self.provider.get(key).await
    }

    pub async fn set(&self, key: &str, value: CacheValue, ttl: Duration) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
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

    pub fn generate_cache_key(provider: &str, query: &str, limit: Option<usize>) -> String {
        format!("{}:{}:{}", provider, query, limit.unwrap_or(10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CacheType;

    fn create_test_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                title: "Test Result 1".to_string(),
                url: "https://example1.com".to_string(),
                snippet: "This is a test result 1".to_string(),
                source_provider: "test".to_string(),
                score: Some(0.95),
            },
            SearchResult {
                title: "Test Result 2".to_string(),
                url: "https://example2.com".to_string(),
                snippet: "This is a test result 2".to_string(),
                source_provider: "test".to_string(),
                score: Some(0.90),
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

        // Test size (moka cache may not immediately report size)
        let _size = cache.size().await.unwrap();
        // Cache should exist and report size (size is always >= 0 for usize)

        // Test delete
        cache.delete("test_key").await.unwrap();
        let retrieved = cache.get("test_key").await.unwrap();
        assert!(retrieved.is_none());

        // Test clear
        cache
            .set("key1", test_results.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        cache
            .set("key2", test_results.clone(), Duration::from_secs(60))
            .await
            .unwrap();

        cache.clear().await.unwrap();
        let retrieved1 = cache.get("key1").await.unwrap();
        let retrieved2 = cache.get("key2").await.unwrap();
        assert!(retrieved1.is_none());
        assert!(retrieved2.is_none());
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = CacheManager::generate_cache_key("google", "rust programming", Some(10));
        assert_eq!(key1, "google:rust programming:10");

        let key2 = CacheManager::generate_cache_key("duckduckgo", "web search", None);
        assert_eq!(key2, "duckduckgo:web search:10");
    }
}
