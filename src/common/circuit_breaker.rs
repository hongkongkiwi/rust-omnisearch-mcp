use std::{collections::HashMap, sync::Arc, time::{Duration, Instant}};
use tokio::sync::RwLock;
use tracing::{debug, warn, error, info};
use eyre::{Result, eyre};
use async_trait::async_trait;

use crate::config::CONFIG;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, requests rejected
    HalfOpen,  // Testing if service recovered
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub provider: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
    pub state_changed_at: Instant,
}

#[async_trait]
pub trait CircuitBreakerProvider: Send + Sync {
    async fn call<F, Fut, T>(&self, provider: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static;
    
    async fn get_stats(&self, provider: &str) -> Option<CircuitBreakerStats>;
    async fn reset(&self, provider: &str) -> Result<()>;
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_max_calls: u32,
    failure_count: u32,
    success_count: u32,
    state: CircuitState,
    last_failure_time: Option<Instant>,
    state_changed_at: Instant,
    half_open_calls: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_duration: Duration, half_open_max_calls: u32) -> Self {
        Self {
            failure_threshold,
            timeout_duration,
            half_open_max_calls,
            failure_count: 0,
            success_count: 0,
            state: CircuitState::Closed,
            last_failure_time: None,
            state_changed_at: Instant::now(),
            half_open_calls: 0,
        }
    }

    pub async fn call<F, Fut, T>(&mut self, provider: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    debug!("Circuit breaker transitioning to half-open for provider: {}", provider);
                    self.state = CircuitState::HalfOpen;
                    self.state_changed_at = Instant::now();
                    self.half_open_calls = 0;
                } else {
                    debug!("Circuit breaker is open, rejecting call for provider: {}", provider);
                    return Err(eyre!("Circuit breaker is open for provider: {}", provider));
                }
            }
            CircuitState::HalfOpen => {
                if self.half_open_calls >= self.half_open_max_calls {
                    debug!("Circuit breaker half-open call limit reached for provider: {}", provider);
                    return Err(eyre!("Circuit breaker half-open call limit reached for provider: {}", provider));
                }
                self.half_open_calls += 1;
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success(provider).await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure(provider).await;
                Err(err)
            }
        }
    }

    async fn on_success(&mut self, provider: &str) {
        self.success_count += 1;
        
        match self.state {
            CircuitState::HalfOpen => {
                debug!("Circuit breaker success in half-open state for provider: {}", provider);
                self.state = CircuitState::Closed;
                self.state_changed_at = Instant::now();
                self.failure_count = 0;
                self.half_open_calls = 0;
                info!("Circuit breaker closed for provider: {}", provider);
            }
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                if self.failure_count > 0 {
                    debug!("Resetting failure count for provider: {} after success", provider);
                    self.failure_count = 0;
                }
            }
            CircuitState::Open => {
                // Should not happen
                warn!("Unexpected success in open state for provider: {}", provider);
            }
        }
    }

    async fn on_failure(&mut self, provider: &str) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        
        debug!("Circuit breaker failure #{} for provider: {}", self.failure_count, provider);

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    warn!("Circuit breaker opening for provider: {} after {} failures", provider, self.failure_count);
                    self.state = CircuitState::Open;
                    self.state_changed_at = Instant::now();
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker reopening for provider: {} after failure in half-open state", provider);
                self.state = CircuitState::Open;
                self.state_changed_at = Instant::now();
                self.half_open_calls = 0;
            }
            CircuitState::Open => {
                // Already open, just log
                debug!("Additional failure in open state for provider: {}", provider);
            }
        }
    }

    fn should_attempt_reset(&self) -> bool {
        matches!(self.state, CircuitState::Open) &&
        self.state_changed_at.elapsed() >= self.timeout_duration
    }

    pub fn get_stats(&self, provider: &str) -> CircuitBreakerStats {
        CircuitBreakerStats {
            provider: provider.to_string(),
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            last_failure_time: self.last_failure_time,
            state_changed_at: self.state_changed_at,
        }
    }

    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
        self.state_changed_at = Instant::now();
        self.half_open_calls = 0;
    }
}

pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    enabled: bool,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_max_calls: u32,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        let config = &CONFIG.circuit_breaker;
        
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            enabled: config.enabled,
            failure_threshold: config.failure_threshold,
            timeout_duration: Duration::from_secs(config.timeout_seconds),
            half_open_max_calls: config.half_open_max_calls,
        }
    }

    async fn get_or_create_breaker(&self, provider: &str) -> Arc<RwLock<CircuitBreaker>> {
        let mut breakers = self.breakers.write().await;
        
        if !breakers.contains_key(provider) {
            let breaker = CircuitBreaker::new(
                self.failure_threshold,
                self.timeout_duration,
                self.half_open_max_calls,
            );
            breakers.insert(provider.to_string(), breaker);
            debug!("Created circuit breaker for provider: {}", provider);
        }

        // This is a bit tricky - we need to return an Arc<RwLock<CircuitBreaker>>
        // but we can't clone the CircuitBreaker directly due to the HashMap structure
        // Instead, we'll use a different approach
        drop(breakers);
        
        let breakers = self.breakers.read().await;
        // We'll need to restructure this - for now, let's use a different approach
        Arc::new(RwLock::new(CircuitBreaker::new(
            self.failure_threshold,
            self.timeout_duration,
            self.half_open_max_calls,
        )))
    }
}

#[async_trait]
impl CircuitBreakerProvider for CircuitBreakerManager {
    async fn call<F, Fut, T>(&self, provider: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send + 'static,
    {
        if !self.enabled {
            return operation().await;
        }

        let mut breakers = self.breakers.write().await;
        
        if !breakers.contains_key(provider) {
            let breaker = CircuitBreaker::new(
                self.failure_threshold,
                self.timeout_duration,
                self.half_open_max_calls,
            );
            breakers.insert(provider.to_string(), breaker);
            debug!("Created circuit breaker for provider: {}", provider);
        }

        let breaker = breakers.get_mut(provider).unwrap();
        breaker.call(provider, operation).await
    }

    async fn get_stats(&self, provider: &str) -> Option<CircuitBreakerStats> {
        if !self.enabled {
            return None;
        }

        let breakers = self.breakers.read().await;
        breakers.get(provider).map(|breaker| breaker.get_stats(provider))
    }

    async fn reset(&self, provider: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(provider) {
            breaker.reset();
            info!("Reset circuit breaker for provider: {}", provider);
        }
        
        Ok(())
    }
}

// Global circuit breaker manager
use once_cell::sync::Lazy;

pub static CIRCUIT_BREAKER_MANAGER: Lazy<CircuitBreakerManager> = Lazy::new(CircuitBreakerManager::new);

// Convenience functions
pub async fn call_with_circuit_breaker<F, Fut, T>(provider: &str, operation: F) -> Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T>> + Send,
    T: Send + 'static,
{
    CIRCUIT_BREAKER_MANAGER.call(provider, operation).await
}

pub async fn get_circuit_breaker_stats(provider: &str) -> Option<CircuitBreakerStats> {
    CIRCUIT_BREAKER_MANAGER.get_stats(provider).await
}

pub async fn reset_circuit_breaker(provider: &str) -> Result<()> {
    CIRCUIT_BREAKER_MANAGER.reset(provider).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(60), 2);
        
        // Should allow calls in closed state
        let result = breaker.call("test", || async { Ok("success") }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(breaker.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open_state() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_millis(100), 2);
        
        // Trigger failures to open the breaker
        let _result1 = breaker.call("test", || async { 
            Err::<(), _>(eyre!("error 1")) 
        }).await;
        
        let _result2 = breaker.call("test", || async { 
            Err::<(), _>(eyre!("error 2")) 
        }).await;
        
        assert_eq!(breaker.state, CircuitState::Open);
        
        // Next call should be rejected
        let result3 = breaker.call("test", || async { Ok("should not execute") }).await;
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("Circuit breaker is open"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_state() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_millis(50), 2);
        
        // Open the breaker
        let _result1 = breaker.call("test", || async { 
            Err::<(), _>(eyre!("error 1")) 
        }).await;
        let _result2 = breaker.call("test", || async { 
            Err::<(), _>(eyre!("error 2")) 
        }).await;
        
        assert_eq!(breaker.state, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        
        // Next call should transition to half-open
        let result = breaker.call("test", || async { Ok("success") }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();
        
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let result = manager.call("test_provider", || {
            let counter = Arc::clone(&counter_clone);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok("success")
            }
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_disabled_circuit_breaker() {
        let manager = CircuitBreakerManager {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
            failure_threshold: 3,
            timeout_duration: Duration::from_secs(60),
            half_open_max_calls: 2,
        };
        
        let result = manager.call("test_provider", || async {
            Err::<(), _>(eyre!("This should still execute when disabled"))
        }).await;
        
        // Should execute and return the error (not circuit breaker error)
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("This should still execute"));
    }
}