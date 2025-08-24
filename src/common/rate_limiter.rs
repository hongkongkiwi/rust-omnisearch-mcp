use eyre::{eyre, Result};
use governor::{
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorLimiter,
};
use std::{collections::HashMap, num::NonZeroU32, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::config::CONFIG;

pub type ProviderRateLimiter =
    GovernorLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>;

#[derive(Clone)]
pub struct RateLimiterManager {
    limiters: Arc<RwLock<HashMap<String, Arc<ProviderRateLimiter>>>>,
    enabled: bool,
}

impl Default for RateLimiterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterManager {
    pub fn new() -> Self {
        let enabled = CONFIG.rate_limiting.enabled;

        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }

    pub async fn get_or_create_limiter(&self, provider: &str) -> Result<Arc<ProviderRateLimiter>> {
        if !self.enabled {
            // Return a very permissive rate limiter when disabled
            let quota = Quota::per_minute(NonZeroU32::new(u32::MAX).unwrap());
            return Ok(Arc::new(GovernorLimiter::direct(quota)));
        }

        let mut limiters = self.limiters.write().await;

        if let Some(limiter) = limiters.get(provider) {
            return Ok(Arc::clone(limiter));
        }

        // Get provider-specific rate limit from config
        let rate_limit = self.get_provider_rate_limit(provider);
        let quota = Quota::per_minute(
            NonZeroU32::new(rate_limit)
                .ok_or_else(|| eyre!("Rate limit must be greater than 0"))?,
        );

        let limiter = Arc::new(GovernorLimiter::direct(quota));
        limiters.insert(provider.to_string(), Arc::clone(&limiter));

        debug!(
            "Created rate limiter for provider '{}' with {} requests/minute",
            provider, rate_limit
        );

        Ok(limiter)
    }

    fn get_provider_rate_limit(&self, provider: &str) -> u32 {
        match provider {
            "tavily" => CONFIG.providers.tavily.rate_limit,
            "google" => CONFIG.providers.google.rate_limit,
            "reddit" => CONFIG.providers.reddit.rate_limit,
            "duckduckgo" => CONFIG.providers.duckduckgo.rate_limit,
            "baidu" => CONFIG.providers.baidu.rate_limit,
            "exa" => CONFIG.providers.exa.rate_limit,
            "brave" => CONFIG.providers.brave.rate_limit,
            "kagi" => CONFIG.providers.kagi.rate_limit,
            "perplexity" => CONFIG.providers.perplexity.rate_limit,
            "jina" => CONFIG.providers.jina.rate_limit,
            "firecrawl" => CONFIG.providers.firecrawl.rate_limit,
            "brightdata" => CONFIG.providers.brightdata.rate_limit,
            _ => {
                warn!("Unknown provider '{}', using default rate limit", provider);
                60 // Default rate limit
            }
        }
    }

    pub async fn check_rate_limit(&self, provider: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let limiter = self.get_or_create_limiter(provider).await?;

        match limiter.check() {
            Ok(_) => {
                debug!("Rate limit check passed for provider: {}", provider);
                Ok(())
            }
            Err(_) => {
                warn!("Rate limit exceeded for provider: {}", provider);
                Err(eyre!("Rate limit exceeded for provider: {}", provider))
            }
        }
    }

    pub async fn wait_for_rate_limit(&self, provider: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let limiter = self.get_or_create_limiter(provider).await?;

        limiter.until_ready().await;
        debug!("Rate limit wait completed for provider: {}", provider);
        Ok(())
    }

    pub async fn reset_limiter(&self, provider: &str) -> Result<()> {
        let mut limiters = self.limiters.write().await;
        limiters.remove(provider);
        debug!("Reset rate limiter for provider: {}", provider);
        Ok(())
    }

    pub async fn get_limiter_stats(&self, provider: &str) -> Result<Option<RateLimiterStats>> {
        if !self.enabled {
            return Ok(None);
        }

        let limiters = self.limiters.read().await;
        if let Some(_limiter) = limiters.get(provider) {
            // Governor rate limiter doesn't provide snapshot functionality in this version
            // Return basic stats
            Ok(Some(RateLimiterStats {
                provider: provider.to_string(),
                remaining_burst: 0, // Not available in this version
                next_replenishment: Some(Duration::from_secs(0)), // Not available
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    pub provider: String,
    pub remaining_burst: u32,
    pub next_replenishment: Option<Duration>,
}

// Global rate limiter manager
use once_cell::sync::Lazy;

pub static RATE_LIMITER_MANAGER: Lazy<RateLimiterManager> = Lazy::new(RateLimiterManager::new);

// Convenience functions
pub async fn check_rate_limit(provider: &str) -> Result<()> {
    RATE_LIMITER_MANAGER.check_rate_limit(provider).await
}

pub async fn wait_for_rate_limit(provider: &str) -> Result<()> {
    RATE_LIMITER_MANAGER.wait_for_rate_limit(provider).await
}

pub async fn get_limiter_stats(provider: &str) -> Result<Option<RateLimiterStats>> {
    RATE_LIMITER_MANAGER.get_limiter_stats(provider).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let manager = RateLimiterManager::new();
        let limiter = manager
            .get_or_create_limiter("test_provider")
            .await
            .unwrap();

        // Should get the same limiter instance for the same provider
        let limiter2 = manager
            .get_or_create_limiter("test_provider")
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&limiter, &limiter2));
    }

    #[tokio::test]
    async fn test_rate_limiting_behavior() {
        let manager = RateLimiterManager::new();

        // This test would need a very low rate limit to be practical
        // In real scenarios, you'd set up a test config
        let result = manager.check_rate_limit("tavily").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_stats() {
        let manager = RateLimiterManager::new();

        // Create a limiter
        let _limiter = manager
            .get_or_create_limiter("test_provider")
            .await
            .unwrap();

        // Get stats
        let stats = manager.get_limiter_stats("test_provider").await.unwrap();

        if manager.enabled {
            assert!(stats.is_some());
            let stats = stats.unwrap();
            assert_eq!(stats.provider, "test_provider");
        } else {
            assert!(stats.is_none());
        }
    }

    #[tokio::test]
    async fn test_disabled_rate_limiting() {
        let manager = RateLimiterManager {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        };

        // When disabled, all operations should pass
        assert!(manager.check_rate_limit("any_provider").await.is_ok());
        assert!(manager.wait_for_rate_limit("any_provider").await.is_ok());

        let stats = manager.get_limiter_stats("any_provider").await.unwrap();
        assert!(stats.is_none());
    }
}
