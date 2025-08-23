use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tracing::{debug, error, info};

use crate::{
    common::{
        cache::get_cache_manager, circuit_breaker::get_circuit_breaker_stats,
        metrics::METRICS_COLLECTOR,
    },
    config::CONFIG,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: ServiceStatus,
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub version: String,
    pub checks: HashMap<String, HealthCheck>,
    pub metrics: Option<HealthMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: CheckStatus,
    pub message: Option<String>,
    pub duration_ms: u64,
    pub last_checked: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: u64,
    pub cache_size: usize,
    pub cache_hit_rate: f64,
    pub active_providers: Vec<String>,
}

pub struct HealthChecker {
    start_time: Instant,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub async fn check_health(&self) -> HealthStatus {
        let mut checks = HashMap::new();
        let mut overall_status = ServiceStatus::Healthy;

        // Check configuration
        let config_check = self.check_configuration().await;
        if matches!(config_check.status, CheckStatus::Fail) {
            overall_status = ServiceStatus::Unhealthy;
        } else if matches!(config_check.status, CheckStatus::Warn)
            && matches!(overall_status, ServiceStatus::Healthy)
        {
            overall_status = ServiceStatus::Degraded;
        }
        checks.insert("configuration".to_string(), config_check);

        // Check cache
        let cache_check = self.check_cache().await;
        if matches!(cache_check.status, CheckStatus::Fail) {
            overall_status = ServiceStatus::Unhealthy;
        } else if matches!(cache_check.status, CheckStatus::Warn)
            && matches!(overall_status, ServiceStatus::Healthy)
        {
            overall_status = ServiceStatus::Degraded;
        }
        checks.insert("cache".to_string(), cache_check);

        // Check metrics system
        let metrics_check = self.check_metrics().await;
        if matches!(metrics_check.status, CheckStatus::Warn)
            && matches!(overall_status, ServiceStatus::Healthy)
        {
            overall_status = ServiceStatus::Degraded;
        }
        checks.insert("metrics".to_string(), metrics_check);

        // Check providers
        let providers_check = self.check_providers().await;
        if matches!(providers_check.status, CheckStatus::Fail) {
            overall_status = ServiceStatus::Unhealthy;
        } else if matches!(providers_check.status, CheckStatus::Warn)
            && matches!(overall_status, ServiceStatus::Healthy)
        {
            overall_status = ServiceStatus::Degraded;
        }
        checks.insert("providers".to_string(), providers_check);

        // Check circuit breakers
        let circuit_breaker_check = self.check_circuit_breakers().await;
        if matches!(circuit_breaker_check.status, CheckStatus::Warn)
            && matches!(overall_status, ServiceStatus::Healthy)
        {
            overall_status = ServiceStatus::Degraded;
        }
        checks.insert("circuit_breakers".to_string(), circuit_breaker_check);

        // Collect metrics if enabled
        let metrics = if CONFIG.metrics.enabled {
            Some(self.collect_metrics().await)
        } else {
            None
        };

        HealthStatus {
            status: overall_status,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks,
            metrics,
        }
    }

    async fn check_configuration(&self) -> HealthCheck {
        let start = Instant::now();

        let (status, message) = match self.validate_configuration().await {
            Ok(issues) => {
                if issues.is_empty() {
                    (CheckStatus::Pass, None)
                } else {
                    (
                        CheckStatus::Warn,
                        Some(format!("Configuration issues: {}", issues.join(", "))),
                    )
                }
            }
            Err(e) => (
                CheckStatus::Fail,
                Some(format!("Configuration validation failed: {}", e)),
            ),
        };

        HealthCheck {
            status,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
            last_checked: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    async fn validate_configuration(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check server configuration
        if CONFIG.server.port == 0 {
            issues.push("Server port not configured".to_string());
        }

        if CONFIG.server.max_connections == 0 {
            issues.push("Max connections not configured".to_string());
        }

        // Check if any providers are configured
        let mut provider_count = 0;

        if CONFIG.providers.tavily.enabled && CONFIG.providers.tavily.api_key.is_some() {
            provider_count += 1;
        }
        if CONFIG.providers.google.enabled && CONFIG.providers.google.api_key.is_some() {
            provider_count += 1;
        }
        if CONFIG.providers.reddit.enabled
            && CONFIG.providers.reddit.client_id.is_some()
            && CONFIG.providers.reddit.client_secret.is_some()
        {
            provider_count += 1;
        }
        if CONFIG.providers.duckduckgo.enabled {
            provider_count += 1; // DuckDuckGo doesn't require API key
        }

        if provider_count == 0 {
            issues.push("No search providers configured".to_string());
        }

        // Check cache configuration if enabled
        if CONFIG.cache.enabled {
            match CONFIG.cache.cache_type {
                crate::config::CacheType::Memory => {
                    if CONFIG.cache.max_entries == 0 {
                        issues.push("Memory cache max entries not configured".to_string());
                    }
                }
                crate::config::CacheType::Redis => {
                    if CONFIG.cache.redis.url.is_empty() {
                        issues.push("Redis URL not configured".to_string());
                    }
                }
            }
        }

        Ok(issues)
    }

    async fn check_cache(&self) -> HealthCheck {
        let start = Instant::now();

        let (status, message) = if !CONFIG.cache.enabled {
            (CheckStatus::Pass, Some("Cache disabled".to_string()))
        } else {
            match self.test_cache_operations().await {
                Ok(_) => (CheckStatus::Pass, None),
                Err(e) => {
                    error!("Cache health check failed: {}", e);
                    (CheckStatus::Fail, Some(format!("Cache error: {}", e)))
                }
            }
        };

        HealthCheck {
            status,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
            last_checked: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    async fn test_cache_operations(&self) -> Result<()> {
        let cache_manager = get_cache_manager().await;
        let test_key = "health_check_test";
        let test_data = vec![];

        // Test set operation
        cache_manager.set(test_key, test_data.clone()).await?;

        // Test get operation
        let result = cache_manager.get(test_key).await?;
        if result.is_none() {
            return Err(eyre::eyre!("Failed to retrieve test data from cache"));
        }

        // Test delete operation
        cache_manager.delete(test_key).await?;

        // Verify deletion
        let result = cache_manager.get(test_key).await?;
        if result.is_some() {
            return Err(eyre::eyre!("Failed to delete test data from cache"));
        }

        debug!("Cache health check passed");
        Ok(())
    }

    async fn check_metrics(&self) -> HealthCheck {
        let start = Instant::now();

        let (status, message) = if !CONFIG.metrics.enabled {
            (CheckStatus::Pass, Some("Metrics disabled".to_string()))
        } else if METRICS_COLLECTOR.is_enabled() {
            (CheckStatus::Pass, None)
        } else {
            (
                CheckStatus::Warn,
                Some("Metrics collector not properly initialized".to_string()),
            )
        };

        HealthCheck {
            status,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
            last_checked: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    async fn check_providers(&self) -> HealthCheck {
        let start = Instant::now();
        let available_providers = self.count_available_providers();

        let (status, message) = if available_providers == 0 {
            (
                CheckStatus::Fail,
                Some("No providers available".to_string()),
            )
        } else if available_providers < 3 {
            (
                CheckStatus::Warn,
                Some(format!("Only {} providers available", available_providers)),
            )
        } else {
            (
                CheckStatus::Pass,
                Some(format!("{} providers available", available_providers)),
            )
        };

        HealthCheck {
            status,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
            last_checked: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    fn count_available_providers(&self) -> usize {
        let mut count = 0;

        if CONFIG.providers.tavily.enabled && CONFIG.providers.tavily.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.google.enabled && CONFIG.providers.google.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.reddit.enabled
            && CONFIG.providers.reddit.client_id.is_some()
            && CONFIG.providers.reddit.client_secret.is_some()
        {
            count += 1;
        }
        if CONFIG.providers.duckduckgo.enabled {
            count += 1;
        }
        if CONFIG.providers.exa.enabled && CONFIG.providers.exa.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.brave.enabled && CONFIG.providers.brave.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.kagi.enabled && CONFIG.providers.kagi.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.perplexity.enabled && CONFIG.providers.perplexity.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.jina.enabled && CONFIG.providers.jina.api_key.is_some() {
            count += 1;
        }
        if CONFIG.providers.firecrawl.enabled && CONFIG.providers.firecrawl.api_key.is_some() {
            count += 1;
        }

        count
    }

    async fn check_circuit_breakers(&self) -> HealthCheck {
        let start = Instant::now();

        let (status, message) = if !CONFIG.circuit_breaker.enabled {
            (
                CheckStatus::Pass,
                Some("Circuit breakers disabled".to_string()),
            )
        } else {
            // Check if any circuit breakers are open
            let providers = ["tavily", "google", "reddit", "duckduckgo", "exa"];
            let mut open_breakers = Vec::new();

            for provider in providers {
                if let Some(stats) = get_circuit_breaker_stats(provider).await {
                    if matches!(
                        stats.state,
                        crate::common::circuit_breaker::CircuitState::Open
                    ) {
                        open_breakers.push(provider);
                    }
                }
            }

            if open_breakers.is_empty() {
                (CheckStatus::Pass, None)
            } else {
                (
                    CheckStatus::Warn,
                    Some(format!(
                        "Open circuit breakers: {}",
                        open_breakers.join(", ")
                    )),
                )
            }
        };

        HealthCheck {
            status,
            message,
            duration_ms: start.elapsed().as_millis() as u64,
            last_checked: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    async fn collect_metrics(&self) -> HealthMetrics {
        let all_stats = METRICS_COLLECTOR.get_all_stats().await;

        let mut total_requests = 0;
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut total_duration = Duration::default();
        let mut cache_hits = 0;
        let mut active_providers = Vec::new();

        for (provider, stats) in all_stats.iter() {
            total_requests += stats.total_requests;
            successful_requests += stats.successful_requests;
            failed_requests += stats.failed_requests;
            total_duration += stats.total_duration;
            cache_hits += stats.cache_hits;

            if stats.total_requests > 0 {
                active_providers.push(provider.clone());
            }
        }

        let average_response_time_ms = if total_requests > 0 {
            (total_duration.as_millis() / total_requests as u128) as u64
        } else {
            0
        };

        let cache_hit_rate = if total_requests > 0 {
            cache_hits as f64 / total_requests as f64
        } else {
            0.0
        };

        let cache_size = get_cache_manager().await.size().await.unwrap_or(0);

        HealthMetrics {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time_ms,
            cache_size,
            cache_hit_rate,
            active_providers,
        }
    }
}

// Global health checker instance
use once_cell::sync::Lazy;

pub static HEALTH_CHECKER: Lazy<HealthChecker> = Lazy::new(HealthChecker::new);

// Convenience function
pub async fn get_health_status() -> HealthStatus {
    HEALTH_CHECKER.check_health().await
}

// Readiness check (lighter weight than full health check)
pub async fn check_readiness() -> Result<()> {
    // Basic readiness checks
    if CONFIG.server.port == 0 {
        return Err(eyre::eyre!("Server not configured"));
    }

    // Check if cache is accessible (if enabled)
    if CONFIG.cache.enabled {
        let cache_manager = get_cache_manager().await;
        // Simple connectivity test
        let _ = cache_manager.size().await?;
    }

    info!("Readiness check passed");
    Ok(())
}

// Liveness check (very basic - just checks if the process is running)
pub async fn check_liveness() -> Result<()> {
    // This is always true if we can execute this function
    debug!("Liveness check passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_creation() {
        let checker = HealthChecker::new();
        assert!(checker.start_time.elapsed().as_millis() < 1000);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let checker = HealthChecker::new();
        let issues = checker.validate_configuration().await.unwrap();

        // The exact issues depend on the current configuration
        // This test mainly ensures the validation doesn't panic
        println!("Configuration issues found: {:?}", issues);
    }

    #[tokio::test]
    async fn test_provider_counting() {
        let checker = HealthChecker::new();
        let count = checker.count_available_providers();

        // Should be at least 1 (DuckDuckGo doesn't require API key)
        assert!(count >= 1, "Expected at least 1 provider, got {}", count);
    }

    #[tokio::test]
    async fn test_readiness_check() {
        let result = check_readiness().await;
        // This should generally pass unless configuration is severely broken
        if result.is_err() {
            println!("Readiness check failed: {}", result.unwrap_err());
        }
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let result = check_liveness().await;
        assert!(result.is_ok(), "Liveness check should always pass");
    }

    #[tokio::test]
    async fn test_full_health_check() {
        let status = get_health_status().await;

        assert!(!status.version.is_empty());
        assert!(status.uptime_seconds >= 0);
        assert!(!status.checks.is_empty());

        // Should have at least basic checks
        assert!(status.checks.contains_key("configuration"));
        assert!(status.checks.contains_key("cache"));
        assert!(status.checks.contains_key("providers"));

        println!(
            "Health status: {}",
            serde_json::to_string_pretty(&status).unwrap()
        );
    }
}
