use eyre::Result;
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::config::CONFIG;

#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub provider: String,
    pub operation: String,
    pub duration: Duration,
    pub success: bool,
    pub response_size: Option<usize>,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProviderStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration: Duration,
    pub cache_hits: u64,
    pub avg_response_time: Duration,
    pub last_request_time: Option<Instant>,
}

pub struct MetricsCollector {
    enabled: bool,
    stats: Arc<RwLock<HashMap<String, ProviderStats>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let enabled = CONFIG.metrics.enabled;

        if enabled {
            Self::register_metrics();
            info!("Metrics collection enabled");
        }

        Self {
            enabled,
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn register_metrics() {
        // Request counters
        describe_counter!(
            "omnisearch_requests_total",
            "Total number of search requests by provider"
        );
        describe_counter!(
            "omnisearch_requests_successful_total",
            "Total number of successful requests by provider"
        );
        describe_counter!(
            "omnisearch_requests_failed_total",
            "Total number of failed requests by provider"
        );
        describe_counter!(
            "omnisearch_cache_hits_total",
            "Total number of cache hits by provider"
        );
        describe_counter!(
            "omnisearch_cache_misses_total",
            "Total number of cache misses by provider"
        );

        // Response time histograms
        describe_histogram!(
            "omnisearch_request_duration_seconds",
            "Request duration in seconds by provider"
        );
        describe_histogram!(
            "omnisearch_response_size_bytes",
            "Response size in bytes by provider"
        );

        // Gauges
        describe_gauge!("omnisearch_active_providers", "Number of active providers");
        describe_gauge!("omnisearch_cache_size", "Current cache size");
        describe_gauge!(
            "omnisearch_rate_limiter_remaining",
            "Remaining rate limit capacity by provider"
        );
    }

    pub async fn record_request(&self, metrics: RequestMetrics) {
        if !self.enabled {
            return;
        }

        let provider = &metrics.provider;
        let duration_seconds = metrics.duration.as_secs_f64();

        // Update Prometheus metrics
        counter!("omnisearch_requests_total", "provider" => provider.clone()).increment(1);

        if metrics.success {
            counter!("omnisearch_requests_successful_total", "provider" => provider.clone()).increment(1);
        } else {
            counter!("omnisearch_requests_failed_total", "provider" => provider.clone()).increment(1);
        }

        if metrics.cache_hit {
            counter!("omnisearch_cache_hits_total", "provider" => provider.clone()).increment(1);
        } else {
            counter!("omnisearch_cache_misses_total", "provider" => provider.clone()).increment(1);
        }

        histogram!("omnisearch_request_duration_seconds", "provider" => provider.clone()).record(duration_seconds);

        if let Some(size) = metrics.response_size {
            histogram!("omnisearch_response_size_bytes", "provider" => provider.clone()).record(size as f64);
        }

        // Update internal stats
        let mut stats = self.stats.write().await;
        let provider_stats = stats.entry(provider.clone()).or_default();

        provider_stats.total_requests += 1;
        if metrics.success {
            provider_stats.successful_requests += 1;
        } else {
            provider_stats.failed_requests += 1;
        }

        if metrics.cache_hit {
            provider_stats.cache_hits += 1;
        }

        provider_stats.total_duration += metrics.duration;
        provider_stats.avg_response_time = Duration::from_nanos(
            (provider_stats.total_duration.as_nanos() / provider_stats.total_requests as u128)
                as u64,
        );
        provider_stats.last_request_time = Some(Instant::now());

        debug!(
            "Recorded metrics for provider {}: success={}, duration={:?}",
            provider, metrics.success, metrics.duration
        );
    }

    pub async fn record_cache_size(&self, size: usize) {
        if !self.enabled {
            return;
        }

        gauge!("omnisearch_cache_size", size as f64);
    }

    pub async fn record_active_providers(&self, count: usize) {
        if !self.enabled {
            return;
        }

        gauge!("omnisearch_active_providers", count as f64);
    }

    pub async fn record_rate_limiter_remaining(&self, provider: &str, remaining: u32) {
        if !self.enabled {
            return;
        }

        gauge!("omnisearch_rate_limiter_remaining", remaining as f64, "provider" => provider.to_string());
    }

    pub async fn get_provider_stats(&self, provider: &str) -> Option<ProviderStats> {
        if !self.enabled {
            return None;
        }

        let stats = self.stats.read().await;
        stats.get(provider).cloned()
    }

    pub async fn get_all_stats(&self) -> HashMap<String, ProviderStats> {
        if !self.enabled {
            return HashMap::new();
        }

        let stats = self.stats.read().await;
        stats.clone()
    }

    pub async fn reset_stats(&self, provider: Option<&str>) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut stats = self.stats.write().await;

        match provider {
            Some(p) => {
                stats.remove(p);
                info!("Reset stats for provider: {}", p);
            }
            None => {
                stats.clear();
                info!("Reset all provider stats");
            }
        }

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// Metrics middleware for timing requests
pub struct MetricsMiddleware {
    collector: Arc<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }

    pub async fn time_request<F, Fut, T>(
        &self,
        provider: &str,
        operation: &str,
        cache_hit: bool,
        request: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start = Instant::now();
        let result = request().await;
        let duration = start.elapsed();

        let success = result.is_ok();
        let response_size = None; // Could be enhanced to measure actual response size

        let metrics = RequestMetrics {
            provider: provider.to_string(),
            operation: operation.to_string(),
            duration,
            success,
            response_size,
            cache_hit,
        };

        self.collector.record_request(metrics).await;

        result
    }
}

// Global metrics collector
use once_cell::sync::Lazy;

pub static METRICS_COLLECTOR: Lazy<Arc<MetricsCollector>> =
    Lazy::new(|| Arc::new(MetricsCollector::new()));

// Convenience functions
pub async fn record_request_metrics(
    provider: &str,
    operation: &str,
    duration: Duration,
    success: bool,
    response_size: Option<usize>,
    cache_hit: bool,
) {
    let metrics = RequestMetrics {
        provider: provider.to_string(),
        operation: operation.to_string(),
        duration,
        success,
        response_size,
        cache_hit,
    };

    METRICS_COLLECTOR.record_request(metrics).await;
}

pub async fn get_provider_stats(provider: &str) -> Option<ProviderStats> {
    METRICS_COLLECTOR.get_provider_stats(provider).await
}

pub async fn get_all_provider_stats() -> HashMap<String, ProviderStats> {
    METRICS_COLLECTOR.get_all_stats().await
}

pub fn get_metrics_middleware() -> MetricsMiddleware {
    MetricsMiddleware::new(Arc::clone(&METRICS_COLLECTOR))
}

// Prometheus metrics exporter setup
#[cfg(feature = "metrics")]
pub async fn setup_metrics_exporter() -> Result<()> {
    use metrics_exporter_prometheus::PrometheusBuilder;
    use std::net::SocketAddr;

    if !CONFIG.metrics.enabled {
        info!("Metrics disabled, skipping Prometheus setup");
        return Ok(());
    }

    let listen_addr: SocketAddr = format!("0.0.0.0:{}", CONFIG.metrics.prometheus_port)
        .parse()
        .map_err(|e| eyre::eyre!("Invalid Prometheus port: {}", e))?;

    let builder = PrometheusBuilder::new();
    builder
        .listen_address(listen_addr)
        .install()
        .map_err(|e| eyre::eyre!("Failed to install Prometheus exporter: {}", e))?;

    info!("Prometheus metrics server listening on {}", listen_addr);
    Ok(())
}

#[cfg(not(feature = "metrics"))]
pub async fn setup_metrics_exporter() -> Result<()> {
    if CONFIG.metrics.enabled {
        error!("Metrics enabled but metrics feature not compiled in");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        let metrics = RequestMetrics {
            provider: "test_provider".to_string(),
            operation: "search".to_string(),
            duration: Duration::from_millis(100),
            success: true,
            response_size: Some(1024),
            cache_hit: false,
        };

        collector.record_request(metrics).await;

        if collector.is_enabled() {
            let stats = collector.get_provider_stats("test_provider").await.unwrap();
            assert_eq!(stats.total_requests, 1);
            assert_eq!(stats.successful_requests, 1);
            assert_eq!(stats.failed_requests, 0);
            assert_eq!(stats.cache_hits, 0);
        }
    }

    #[tokio::test]
    async fn test_metrics_middleware() {
        let collector = Arc::new(MetricsCollector::new());
        let middleware = MetricsMiddleware::new(collector.clone());

        let result = middleware
            .time_request("test_provider", "search", false, || async {
                Ok::<&str, eyre::Error>("success")
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        if collector.is_enabled() {
            let stats = collector.get_provider_stats("test_provider").await.unwrap();
            assert_eq!(stats.total_requests, 1);
            assert_eq!(stats.successful_requests, 1);
        }
    }

    #[tokio::test]
    async fn test_failed_request_metrics() {
        let collector = MetricsCollector::new();

        let metrics = RequestMetrics {
            provider: "test_provider".to_string(),
            operation: "search".to_string(),
            duration: Duration::from_millis(50),
            success: false,
            response_size: None,
            cache_hit: false,
        };

        collector.record_request(metrics).await;

        if collector.is_enabled() {
            let stats = collector.get_provider_stats("test_provider").await.unwrap();
            assert_eq!(stats.total_requests, 1);
            assert_eq!(stats.successful_requests, 0);
            assert_eq!(stats.failed_requests, 1);
        }
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let collector = MetricsCollector::new();

        let metrics = RequestMetrics {
            provider: "test_provider".to_string(),
            operation: "search".to_string(),
            duration: Duration::from_millis(100),
            success: true,
            response_size: Some(1024),
            cache_hit: true,
        };

        collector.record_request(metrics).await;

        // Reset specific provider
        collector.reset_stats(Some("test_provider")).await.unwrap();

        let stats = collector.get_provider_stats("test_provider").await;
        assert!(stats.is_none());
    }
}
