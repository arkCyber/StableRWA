// =====================================================================================
// File: testing/src/lib.rs
// Description: Testing utilities and frameworks for StableRWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

//! # Testing Utils
//! 
//! This crate provides comprehensive testing utilities for the StableRWA platform,
//! including test fixtures, factories, mocks, and integration test helpers.

pub mod fixtures;
pub mod factories;
pub mod mocks;
pub mod containers;
pub mod assertions;
pub mod setup;
pub mod helpers;

// Re-export commonly used testing utilities
pub use fixtures::*;
pub use factories::*;
pub use mocks::*;
pub use containers::*;
pub use assertions::*;
pub use setup::*;
pub use helpers::*;

// Re-export external testing dependencies
pub use mockall;
pub use proptest;
pub use fake;
pub use wiremock;
pub use testcontainers;
pub use tokio_test;
pub use assert_matches;
pub use pretty_assertions;

use serde::{Deserialize, Serialize};
use std::sync::Once;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Load test environment variables
        let _ = dotenvy::from_filename(".env.test");
        
        // Initialize tracing for tests
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().with_test_writer());
        
        let _ = subscriber.try_init();
    });
}

/// Test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub test_timeout_seconds: u64,
    pub parallel_tests: bool,
    pub cleanup_after_tests: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://test:test@localhost:5432/test_db".to_string(),
            redis_url: "redis://localhost:6379/1".to_string(),
            test_timeout_seconds: 30,
            parallel_tests: true,
            cleanup_after_tests: true,
        }
    }
}

/// Test result wrapper
#[derive(Debug)]
pub struct TestResult<T> {
    pub result: Result<T, Box<dyn std::error::Error + Send + Sync>>,
    pub duration: std::time::Duration,
    pub metadata: std::collections::HashMap<String, String>,
}

impl<T> TestResult<T> {
    pub fn new(result: Result<T, Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self {
            result,
            duration: std::time::Duration::default(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    pub fn is_err(&self) -> bool {
        self.result.is_err()
    }
}

/// Test suite trait for organizing related tests
pub trait TestSuite {
    fn name(&self) -> &'static str;
    fn setup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn teardown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn run_tests(&mut self) -> Vec<TestResult<()>>;
}

/// Macro for creating test suites
#[macro_export]
macro_rules! test_suite {
    ($name:ident, $($test:ident),*) => {
        pub struct $name;
        
        impl TestSuite for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }
            
            fn setup(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
            
            fn teardown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                Ok(())
            }
            
            fn run_tests(&mut self) -> Vec<TestResult<()>> {
                vec![
                    $(
                        {
                            let start = std::time::Instant::now();
                            let result = std::panic::catch_unwind(|| {
                                $test()
                            });
                            let duration = start.elapsed();
                            
                            match result {
                                Ok(_) => TestResult::new(Ok(())).with_duration(duration),
                                Err(e) => {
                                    let error_msg = if let Some(s) = e.downcast_ref::<String>() {
                                        s.clone()
                                    } else if let Some(s) = e.downcast_ref::<&str>() {
                                        s.to_string()
                                    } else {
                                        "Test panicked".to_string()
                                    };
                                    TestResult::new(Err(error_msg.into())).with_duration(duration)
                                }
                            }
                        }
                    ),*
                ]
            }
        }
    };
}

/// Async test runner
pub async fn run_async_test<F, Fut, T>(test_fn: F) -> TestResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let start = std::time::Instant::now();
    let result = test_fn().await;
    let duration = start.elapsed();
    
    TestResult::new(result).with_duration(duration)
}

/// Test timeout wrapper
pub async fn with_timeout<F, T>(
    duration: std::time::Duration,
    future: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: std::future::Future<Output = T>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Test timed out".into()),
    }
}

/// Performance test utilities
pub mod perf {
    use std::time::{Duration, Instant};
    
    pub struct PerformanceTest {
        pub name: String,
        pub iterations: usize,
        pub warmup_iterations: usize,
        pub results: Vec<Duration>,
    }
    
    impl PerformanceTest {
        pub fn new(name: String, iterations: usize) -> Self {
            Self {
                name,
                iterations,
                warmup_iterations: iterations / 10,
                results: Vec::new(),
            }
        }
        
        pub async fn run<F, Fut, T>(&mut self, test_fn: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where
            F: Fn() -> Fut + Clone,
            Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
        {
            // Warmup
            for _ in 0..self.warmup_iterations {
                let _ = test_fn().await?;
            }
            
            // Actual test runs
            for _ in 0..self.iterations {
                let start = Instant::now();
                let _ = test_fn().await?;
                let duration = start.elapsed();
                self.results.push(duration);
            }
            
            Ok(())
        }
        
        pub fn average_duration(&self) -> Duration {
            if self.results.is_empty() {
                return Duration::default();
            }
            
            let total: Duration = self.results.iter().sum();
            total / self.results.len() as u32
        }
        
        pub fn min_duration(&self) -> Duration {
            self.results.iter().min().copied().unwrap_or_default()
        }
        
        pub fn max_duration(&self) -> Duration {
            self.results.iter().max().copied().unwrap_or_default()
        }
        
        pub fn percentile(&self, p: f64) -> Duration {
            if self.results.is_empty() {
                return Duration::default();
            }
            
            let mut sorted = self.results.clone();
            sorted.sort();
            
            let index = ((sorted.len() as f64 - 1.0) * p / 100.0) as usize;
            sorted[index]
        }
    }
}

/// Load test utilities
pub mod load {
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    
    pub struct LoadTest {
        pub name: String,
        pub concurrent_users: usize,
        pub duration_seconds: u64,
        pub ramp_up_seconds: u64,
    }
    
    impl LoadTest {
        pub fn new(name: String, concurrent_users: usize, duration_seconds: u64) -> Self {
            Self {
                name,
                concurrent_users,
                duration_seconds,
                ramp_up_seconds: duration_seconds / 10,
            }
        }
        
        pub async fn run<F, Fut, T>(&self, test_fn: F) -> Result<LoadTestResults, Box<dyn std::error::Error + Send + Sync>>
        where
            F: Fn() -> Fut + Clone + Send + Sync + 'static,
            Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send,
            T: Send + 'static,
        {
            let semaphore = Arc::new(Semaphore::new(self.concurrent_users));
            let mut handles = Vec::new();
            let start_time = std::time::Instant::now();
            let end_time = start_time + std::time::Duration::from_secs(self.duration_seconds);
            
            // Spawn concurrent tasks
            for _ in 0..self.concurrent_users {
                let permit = semaphore.clone().acquire_owned().await?;
                let test_fn = test_fn.clone();
                
                let handle = tokio::spawn(async move {
                    let _permit = permit;
                    let mut request_count = 0;
                    let mut error_count = 0;
                    
                    while std::time::Instant::now() < end_time {
                        match test_fn().await {
                            Ok(_) => request_count += 1,
                            Err(_) => error_count += 1,
                        }
                    }
                    
                    (request_count, error_count)
                });
                
                handles.push(handle);
            }
            
            // Collect results
            let mut total_requests = 0;
            let mut total_errors = 0;
            
            for handle in handles {
                let (requests, errors) = handle.await?;
                total_requests += requests;
                total_errors += errors;
            }
            
            let actual_duration = start_time.elapsed();
            
            Ok(LoadTestResults {
                total_requests,
                total_errors,
                duration: actual_duration,
                requests_per_second: total_requests as f64 / actual_duration.as_secs_f64(),
                error_rate: if total_requests > 0 {
                    total_errors as f64 / total_requests as f64
                } else {
                    0.0
                },
            })
        }
    }
    
    #[derive(Debug)]
    pub struct LoadTestResults {
        pub total_requests: usize,
        pub total_errors: usize,
        pub duration: std::time::Duration,
        pub requests_per_second: f64,
        pub error_rate: f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_test_env() {
        init_test_env();
        // Should not panic on multiple calls
        init_test_env();
    }
    
    #[test]
    fn test_config_default() {
        let config = TestConfig::default();
        assert!(!config.database_url.is_empty());
        assert!(!config.redis_url.is_empty());
        assert!(config.test_timeout_seconds > 0);
    }
    
    #[tokio::test]
    async fn test_async_test_runner() {
        let result = run_async_test(|| async {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42)
        }).await;
        
        assert!(result.is_ok());
        assert!(result.duration > std::time::Duration::default());
    }
    
    #[tokio::test]
    async fn test_timeout_wrapper() {
        let result = with_timeout(
            std::time::Duration::from_millis(100),
            async { tokio::time::sleep(std::time::Duration::from_millis(50)).await }
        ).await;
        
        assert!(result.is_ok());
        
        let result = with_timeout(
            std::time::Duration::from_millis(50),
            async { tokio::time::sleep(std::time::Duration::from_millis(100)).await }
        ).await;
        
        assert!(result.is_err());
    }
}
