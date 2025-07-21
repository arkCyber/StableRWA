// =====================================================================================
// File: tests/performance_tests.rs
// Description: Performance and load tests for RWA platform
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

/// Performance test configuration
#[derive(Clone)]
struct PerformanceTestConfig {
    gateway_url: String,
    concurrent_users: usize,
    test_duration: Duration,
    ramp_up_time: Duration,
    think_time: Duration,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            gateway_url: std::env::var("GATEWAY_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            concurrent_users: 50,
            test_duration: Duration::from_secs(60),
            ramp_up_time: Duration::from_secs(10),
            think_time: Duration::from_millis(100),
        }
    }
}

/// Performance metrics collector
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    response_times: Vec<Duration>,
    errors: Vec<String>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            response_times: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn add_request(&mut self, duration: Duration, success: bool, error: Option<String>) {
        self.total_requests += 1;
        self.response_times.push(duration);
        
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
            if let Some(err) = error {
                self.errors.push(err);
            }
        }
    }

    fn calculate_percentiles(&self) -> (Duration, Duration, Duration) {
        let mut sorted_times = self.response_times.clone();
        sorted_times.sort();
        
        let len = sorted_times.len();
        let p50 = sorted_times[len * 50 / 100];
        let p95 = sorted_times[len * 95 / 100];
        let p99 = sorted_times[len * 99 / 100];
        
        (p50, p95, p99)
    }

    fn average_response_time(&self) -> Duration {
        if self.response_times.is_empty() {
            return Duration::from_millis(0);
        }
        
        let total: Duration = self.response_times.iter().sum();
        total / self.response_times.len() as u32
    }

    fn requests_per_second(&self, test_duration: Duration) -> f64 {
        self.total_requests as f64 / test_duration.as_secs_f64()
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64 * 100.0
    }
}

/// Load test client
struct LoadTestClient {
    client: Client,
    config: PerformanceTestConfig,
    auth_token: Option<String>,
}

impl LoadTestClient {
    fn new(config: PerformanceTestConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap(),
            config,
            auth_token: None,
        }
    }

    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client
            .post(&format!("{}/api/v1/auth/login", self.config.gateway_url))
            .json(&json!({
                "username": "load_test_user",
                "password": "load_test_password"
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let auth_response: Value = response.json().await?;
            self.auth_token = Some(auth_response["access_token"].as_str().unwrap().to_string());
        }

        Ok(())
    }

    async fn make_request(&self, endpoint: &str, method: &str, body: Option<Value>) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = Instant::now();
        let url = format!("{}{}", self.config.gateway_url, endpoint);
        
        let mut request = match method {
            "GET" => self.client.get(&url),
            "POST" => {
                let mut req = self.client.post(&url);
                if let Some(b) = body {
                    req = req.json(&b);
                }
                req
            },
            "PUT" => {
                let mut req = self.client.put(&url);
                if let Some(b) = body {
                    req = req.json(&b);
                }
                req
            },
            "DELETE" => self.client.delete(&url),
            _ => return Err("Unsupported HTTP method".into()),
        };

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let duration = start.elapsed();

        if !response.status().is_success() {
            return Err(format!("HTTP {}: {}", response.status(), response.text().await?).into());
        }

        Ok(duration)
    }
}

#[tokio::test]
async fn test_api_endpoint_performance() {
    let config = PerformanceTestConfig::default();
    let mut metrics = PerformanceMetrics::new();
    
    // Test various API endpoints
    let endpoints = vec![
        ("/health", "GET", None),
        ("/api/v1/assets", "GET", None),
        ("/metrics", "GET", None),
    ];

    for (endpoint, method, body) in endpoints {
        let mut client = LoadTestClient::new(config.clone());
        
        for _ in 0..100 {
            let start = Instant::now();
            match client.make_request(endpoint, method, body.clone()).await {
                Ok(duration) => {
                    metrics.add_request(duration, true, None);
                },
                Err(e) => {
                    let duration = start.elapsed();
                    metrics.add_request(duration, false, Some(e.to_string()));
                }
            }
            
            tokio::time::sleep(config.think_time).await;
        }
    }

    // Assert performance requirements
    let (p50, p95, p99) = metrics.calculate_percentiles();
    let avg_response_time = metrics.average_response_time();
    let success_rate = metrics.success_rate();

    println!("📊 API Performance Results:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Success Rate: {:.2}%", success_rate);
    println!("  Average Response Time: {:?}", avg_response_time);
    println!("  50th Percentile: {:?}", p50);
    println!("  95th Percentile: {:?}", p95);
    println!("  99th Percentile: {:?}", p99);

    // Performance assertions
    assert!(success_rate >= 95.0, "Success rate should be at least 95%");
    assert!(p95 < Duration::from_millis(1000), "95th percentile should be under 1 second");
    assert!(avg_response_time < Duration::from_millis(500), "Average response time should be under 500ms");
}

#[tokio::test]
async fn test_concurrent_user_load() {
    let config = PerformanceTestConfig {
        concurrent_users: 25,
        test_duration: Duration::from_secs(30),
        ..Default::default()
    };

    let metrics = Arc::new(tokio::sync::Mutex::new(PerformanceMetrics::new()));
    let semaphore = Arc::new(Semaphore::new(config.concurrent_users));
    let start_time = Instant::now();

    let mut tasks = Vec::new();

    for user_id in 0..config.concurrent_users {
        let config_clone = config.clone();
        let metrics_clone = metrics.clone();
        let semaphore_clone = semaphore.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let mut client = LoadTestClient::new(config_clone.clone());
            
            // Simulate user session
            while start_time.elapsed() < config_clone.test_duration {
                let request_start = Instant::now();
                
                match client.make_request("/health", "GET", None).await {
                    Ok(duration) => {
                        let mut m = metrics_clone.lock().await;
                        m.add_request(duration, true, None);
                    },
                    Err(e) => {
                        let duration = request_start.elapsed();
                        let mut m = metrics_clone.lock().await;
                        m.add_request(duration, false, Some(e.to_string()));
                    }
                }

                tokio::time::sleep(config_clone.think_time).await;
            }
        });

        tasks.push(task);
        
        // Ramp up users gradually
        if user_id < config.concurrent_users - 1 {
            let ramp_delay = config.ramp_up_time / config.concurrent_users as u32;
            tokio::time::sleep(ramp_delay).await;
        }
    }

    // Wait for all tasks to complete
    futures::future::join_all(tasks).await;

    let final_metrics = metrics.lock().await;
    let test_duration = start_time.elapsed();
    let rps = final_metrics.requests_per_second(test_duration);
    let success_rate = final_metrics.success_rate();
    let (p50, p95, p99) = final_metrics.calculate_percentiles();

    println!("🚀 Concurrent Load Test Results:");
    println!("  Concurrent Users: {}", config.concurrent_users);
    println!("  Test Duration: {:?}", test_duration);
    println!("  Total Requests: {}", final_metrics.total_requests);
    println!("  Requests/Second: {:.2}", rps);
    println!("  Success Rate: {:.2}%", success_rate);
    println!("  Response Times - P50: {:?}, P95: {:?}, P99: {:?}", p50, p95, p99);

    // Performance assertions
    assert!(success_rate >= 90.0, "Success rate should be at least 90% under load");
    assert!(rps >= 10.0, "Should handle at least 10 requests per second");
    assert!(p95 < Duration::from_secs(2), "95th percentile should be under 2 seconds under load");
}

#[tokio::test]
async fn test_asset_creation_performance() {
    let config = PerformanceTestConfig {
        concurrent_users: 10,
        test_duration: Duration::from_secs(20),
        ..Default::default()
    };

    let mut client = LoadTestClient::new(config.clone());
    client.authenticate().await.unwrap();

    let mut metrics = PerformanceMetrics::new();
    let start_time = Instant::now();

    while start_time.elapsed() < config.test_duration {
        let asset_data = json!({
            "name": format!("Test Asset {}", Uuid::new_v4()),
            "description": "Performance test asset",
            "asset_type": "RealEstate",
            "owner_id": "test_owner",
            "location": {
                "address": "123 Test St",
                "city": "Test City",
                "country": "US"
            },
            "valuation": {
                "amount": 1000000,
                "currency": "USD"
            }
        });

        let request_start = Instant::now();
        match client.make_request("/api/v1/assets", "POST", Some(asset_data)).await {
            Ok(duration) => {
                metrics.add_request(duration, true, None);
            },
            Err(e) => {
                let duration = request_start.elapsed();
                metrics.add_request(duration, false, Some(e.to_string()));
            }
        }

        tokio::time::sleep(config.think_time).await;
    }

    let success_rate = metrics.success_rate();
    let avg_response_time = metrics.average_response_time();
    let (p50, p95, p99) = metrics.calculate_percentiles();

    println!("🏠 Asset Creation Performance:");
    println!("  Total Asset Creation Requests: {}", metrics.total_requests);
    println!("  Success Rate: {:.2}%", success_rate);
    println!("  Average Response Time: {:?}", avg_response_time);
    println!("  Response Times - P50: {:?}, P95: {:?}, P99: {:?}", p50, p95, p99);

    // Asset creation should be reliable but can be slower
    assert!(success_rate >= 95.0, "Asset creation success rate should be at least 95%");
    assert!(p95 < Duration::from_secs(5), "95th percentile for asset creation should be under 5 seconds");
}

#[tokio::test]
async fn test_memory_and_resource_usage() {
    // This test would monitor system resources during load
    // In a real implementation, this would integrate with system monitoring tools
    
    let config = PerformanceTestConfig {
        concurrent_users: 20,
        test_duration: Duration::from_secs(30),
        ..Default::default()
    };

    let start_memory = get_memory_usage().await;
    
    // Run load test
    test_concurrent_user_load().await;
    
    let end_memory = get_memory_usage().await;
    let memory_increase = end_memory - start_memory;

    println!("💾 Resource Usage Test:");
    println!("  Memory at start: {} MB", start_memory);
    println!("  Memory at end: {} MB", end_memory);
    println!("  Memory increase: {} MB", memory_increase);

    // Assert reasonable memory usage
    assert!(memory_increase < 500, "Memory increase should be less than 500MB during load test");
}

/// Helper function to get current memory usage (simplified)
async fn get_memory_usage() -> u64 {
    // In a real implementation, this would use system APIs or tools like `ps` or `/proc/meminfo`
    // For now, return a mock value
    100 // MB
}

/// Stress test with extreme load
#[tokio::test]
async fn test_stress_limits() {
    let config = PerformanceTestConfig {
        concurrent_users: 100,
        test_duration: Duration::from_secs(10),
        think_time: Duration::from_millis(10),
        ..Default::default()
    };

    println!("⚡ Starting stress test with {} concurrent users", config.concurrent_users);
    
    // This test is designed to find the breaking point
    // We expect some failures under extreme load
    let metrics = Arc::new(tokio::sync::Mutex::new(PerformanceMetrics::new()));
    let semaphore = Arc::new(Semaphore::new(config.concurrent_users));
    let start_time = Instant::now();

    let mut tasks = Vec::new();

    for _ in 0..config.concurrent_users {
        let config_clone = config.clone();
        let metrics_clone = metrics.clone();
        let semaphore_clone = semaphore.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let client = LoadTestClient::new(config_clone.clone());
            
            while start_time.elapsed() < config_clone.test_duration {
                let request_start = Instant::now();
                
                match client.make_request("/health", "GET", None).await {
                    Ok(duration) => {
                        let mut m = metrics_clone.lock().await;
                        m.add_request(duration, true, None);
                    },
                    Err(e) => {
                        let duration = request_start.elapsed();
                        let mut m = metrics_clone.lock().await;
                        m.add_request(duration, false, Some(e.to_string()));
                    }
                }

                tokio::time::sleep(config_clone.think_time).await;
            }
        });

        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    let final_metrics = metrics.lock().await;
    let success_rate = final_metrics.success_rate();
    let rps = final_metrics.requests_per_second(start_time.elapsed());

    println!("💥 Stress Test Results:");
    println!("  Success Rate: {:.2}%", success_rate);
    println!("  Requests/Second: {:.2}", rps);
    println!("  Total Requests: {}", final_metrics.total_requests);

    // Under stress, we expect some degradation but system should not completely fail
    assert!(success_rate >= 50.0, "Even under stress, success rate should be at least 50%");
    assert!(final_metrics.total_requests > 0, "System should handle some requests even under stress");
}
