// =====================================================================================
// File: service-gateway/src/routing.rs
// Description: Service registry and routing for API Gateway
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use crate::GatewayError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Service instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub health_check_url: Option<String>,
    pub metadata: HashMap<String, String>,
    #[serde(skip)]
    pub last_heartbeat: Instant,
    pub status: ServiceStatus,
    pub weight: u32,
}

impl ServiceInstance {
    pub fn new(
        id: String,
        name: String,
        host: String,
        port: u16,
    ) -> Self {
        Self {
            id,
            name,
            host,
            port,
            protocol: "http".to_string(),
            health_check_url: None,
            metadata: HashMap::new(),
            last_heartbeat: Instant::now(),
            status: ServiceStatus::Healthy,
            weight: 100,
        }
    }

    pub fn url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }

    pub fn with_protocol(mut self, protocol: String) -> Self {
        self.protocol = protocol;
        self
    }

    pub fn with_health_check(mut self, health_check_url: String) -> Self {
        self.health_check_url = Some(health_check_url);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ServiceStatus::Healthy)
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
        if self.status == ServiceStatus::Unhealthy {
            self.status = ServiceStatus::Healthy;
            info!("Service instance {} is now healthy", self.id);
        }
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.last_heartbeat.elapsed() > timeout
    }
}

/// Service status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Draining,
}

/// Load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    IpHash,
}

/// Service registry for managing service instances
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    load_balancer_state: Arc<RwLock<HashMap<String, LoadBalancerState>>>,
    health_check_interval: Duration,
    heartbeat_timeout: Duration,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            load_balancer_state: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: Duration::from_secs(30),
            heartbeat_timeout: Duration::from_secs(90),
        }
    }

    /// Register a service instance
    pub async fn register(&self, instance: ServiceInstance) -> Result<(), GatewayError> {
        let mut services = self.services.write().await;
        let service_name = instance.name.clone();
        
        let instances = services.entry(service_name.clone()).or_insert_with(Vec::new);
        
        // Remove existing instance with same ID
        instances.retain(|i| i.id != instance.id);
        
        // Add new instance
        instances.push(instance.clone());
        
        info!(
            service_name = %service_name,
            instance_id = %instance.id,
            url = %instance.url(),
            "Service instance registered"
        );

        Ok(())
    }

    /// Deregister a service instance
    pub async fn deregister(&self, service_name: &str, instance_id: &str) -> Result<(), GatewayError> {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            let initial_count = instances.len();
            instances.retain(|i| i.id != instance_id);
            
            if instances.len() < initial_count {
                info!(
                    service_name = %service_name,
                    instance_id = %instance_id,
                    "Service instance deregistered"
                );
            }
            
            // Remove service entry if no instances left
            if instances.is_empty() {
                services.remove(service_name);
            }
        }

        Ok(())
    }

    /// Get service URL using load balancing
    pub async fn get_service_url(&self, service_name: &str) -> Option<String> {
        let services = self.services.read().await;
        let instances = services.get(service_name)?;
        
        // Filter healthy instances
        let healthy_instances: Vec<&ServiceInstance> = instances
            .iter()
            .filter(|i| i.is_healthy())
            .collect();

        if healthy_instances.is_empty() {
            warn!("No healthy instances found for service: {}", service_name);
            return None;
        }

        // Use round-robin load balancing
        let mut lb_state = self.load_balancer_state.write().await;
        let state = lb_state.entry(service_name.to_string()).or_insert_with(LoadBalancerState::new);
        
        let selected_instance = match state.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let index = state.round_robin_index % healthy_instances.len();
                state.round_robin_index = (state.round_robin_index + 1) % healthy_instances.len();
                healthy_instances[index]
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.select_weighted_instance(&healthy_instances, state)
            }
            LoadBalancingStrategy::Random => {
                use rand::Rng;
                let index = rand::thread_rng().gen_range(0..healthy_instances.len());
                healthy_instances[index]
            }
            _ => healthy_instances[0], // Default to first instance
        };

        debug!(
            service_name = %service_name,
            selected_instance = %selected_instance.id,
            url = %selected_instance.url(),
            "Service instance selected"
        );

        Some(selected_instance.url())
    }

    /// Select instance using weighted round-robin
    fn select_weighted_instance<'a>(
        &self,
        instances: &[&'a ServiceInstance],
        state: &mut LoadBalancerState,
    ) -> &'a ServiceInstance {
        let total_weight: u32 = instances.iter().map(|i| i.weight).sum();
        
        if total_weight == 0 {
            return instances[0];
        }

        state.current_weight += total_weight;
        
        let mut selected = instances[0];
        let mut max_weight = 0;

        for instance in instances {
            let effective_weight = instance.weight * total_weight / instance.weight;
            if effective_weight > max_weight {
                max_weight = effective_weight;
                selected = instance;
            }
        }

        state.current_weight -= selected.weight;
        selected
    }

    /// Update service instance heartbeat
    pub async fn heartbeat(&self, service_name: &str, instance_id: &str) -> Result<(), GatewayError> {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
                instance.update_heartbeat();
                debug!(
                    service_name = %service_name,
                    instance_id = %instance_id,
                    "Heartbeat received"
                );
                return Ok(());
            }
        }

        warn!(
            service_name = %service_name,
            instance_id = %instance_id,
            "Heartbeat received for unknown instance"
        );

        Err(GatewayError::ServiceUnavailable(format!(
            "Instance {} not found for service {}",
            instance_id, service_name
        )))
    }

    /// Get all services and their instances
    pub async fn get_all_services(&self) -> HashMap<String, Vec<ServiceInstance>> {
        self.services.read().await.clone()
    }

    /// Get instances for a specific service
    pub async fn get_service_instances(&self, service_name: &str) -> Option<Vec<ServiceInstance>> {
        let services = self.services.read().await;
        services.get(service_name).cloned()
    }

    /// Start background health checking
    pub async fn start_health_checker(self: Arc<Self>) {
        let registry = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(registry.health_check_interval);
            
            loop {
                interval.tick().await;
                registry.perform_health_checks().await;
                registry.cleanup_expired_instances().await;
            }
        });
    }

    /// Perform health checks on all instances
    async fn perform_health_checks(&self) {
        let services = self.services.read().await.clone();
        
        for (service_name, instances) in services {
            for instance in instances {
                if let Some(health_url) = &instance.health_check_url {
                    self.check_instance_health(&service_name, &instance.id, health_url).await;
                }
            }
        }
    }

    /// Check health of a specific instance
    async fn check_instance_health(&self, service_name: &str, instance_id: &str, health_url: &str) {
        let client = reqwest::Client::new();
        
        match client.get(health_url).timeout(Duration::from_secs(5)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.mark_instance_healthy(service_name, instance_id).await;
                } else {
                    self.mark_instance_unhealthy(service_name, instance_id).await;
                }
            }
            Err(_) => {
                self.mark_instance_unhealthy(service_name, instance_id).await;
            }
        }
    }

    /// Mark instance as healthy
    async fn mark_instance_healthy(&self, service_name: &str, instance_id: &str) {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
                if instance.status != ServiceStatus::Healthy {
                    instance.status = ServiceStatus::Healthy;
                    info!(
                        service_name = %service_name,
                        instance_id = %instance_id,
                        "Instance marked as healthy"
                    );
                }
            }
        }
    }

    /// Mark instance as unhealthy
    async fn mark_instance_unhealthy(&self, service_name: &str, instance_id: &str) {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
                if instance.status != ServiceStatus::Unhealthy {
                    instance.status = ServiceStatus::Unhealthy;
                    warn!(
                        service_name = %service_name,
                        instance_id = %instance_id,
                        "Instance marked as unhealthy"
                    );
                }
            }
        }
    }

    /// Clean up expired instances
    async fn cleanup_expired_instances(&self) {
        let mut services = self.services.write().await;
        let mut removed_count = 0;

        for (service_name, instances) in services.iter_mut() {
            let initial_count = instances.len();
            instances.retain(|instance| {
                if instance.is_expired(self.heartbeat_timeout) {
                    warn!(
                        service_name = %service_name,
                        instance_id = %instance.id,
                        "Removing expired instance"
                    );
                    false
                } else {
                    true
                }
            });
            removed_count += initial_count - instances.len();
        }

        // Remove empty service entries
        services.retain(|_, instances| !instances.is_empty());

        if removed_count > 0 {
            info!("Cleaned up {} expired service instances", removed_count);
        }
    }
}

/// Load balancer state
#[derive(Debug)]
struct LoadBalancerState {
    strategy: LoadBalancingStrategy,
    round_robin_index: usize,
    current_weight: u32,
}

impl LoadBalancerState {
    fn new() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            round_robin_index: 0,
            current_weight: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_registration() {
        let registry = ServiceRegistry::new();
        
        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            "localhost".to_string(),
            8080,
        );

        registry.register(instance).await.unwrap();
        
        let instances = registry.get_service_instances("test-service").await.unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].id, "test-1");
    }

    #[tokio::test]
    async fn test_service_deregistration() {
        let registry = ServiceRegistry::new();
        
        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            "localhost".to_string(),
            8080,
        );

        registry.register(instance).await.unwrap();
        registry.deregister("test-service", "test-1").await.unwrap();
        
        let instances = registry.get_service_instances("test-service").await;
        assert!(instances.is_none());
    }

    #[tokio::test]
    async fn test_load_balancing() {
        let registry = ServiceRegistry::new();
        
        // Register multiple instances
        for i in 1..=3 {
            let instance = ServiceInstance::new(
                format!("test-{}", i),
                "test-service".to_string(),
                "localhost".to_string(),
                8080 + i,
            );
            registry.register(instance).await.unwrap();
        }

        // Test round-robin selection
        let mut selected_urls = Vec::new();
        for _ in 0..6 {
            if let Some(url) = registry.get_service_url("test-service").await {
                selected_urls.push(url);
            }
        }

        // Should cycle through all instances
        assert_eq!(selected_urls.len(), 6);
        assert!(selected_urls.contains(&"http://localhost:8081".to_string()));
        assert!(selected_urls.contains(&"http://localhost:8082".to_string()));
        assert!(selected_urls.contains(&"http://localhost:8083".to_string()));
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let registry = ServiceRegistry::new();
        
        let instance = ServiceInstance::new(
            "test-1".to_string(),
            "test-service".to_string(),
            "localhost".to_string(),
            8080,
        );

        registry.register(instance).await.unwrap();
        
        // Test heartbeat
        let result = registry.heartbeat("test-service", "test-1").await;
        assert!(result.is_ok());
        
        // Test heartbeat for non-existent instance
        let result = registry.heartbeat("test-service", "non-existent").await;
        assert!(result.is_err());
    }
}
