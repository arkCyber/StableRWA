// =====================================================================================
// RWA Tokenization Platform - Asset Service Performance Benchmarks
// 
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_decimal_macros::dec;
use service_asset::{
    cache::InMemoryCache,
    models::*,
    service::{AssetService, AssetServiceTrait, CreateAssetRequest, Pagination, AssetFilters},
};
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Benchmark asset creation performance
fn bench_asset_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("asset_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let repository = InMemoryAssetRepository::new();
            let cache = InMemoryCache::new();
            let service = AssetService::new(Box::new(repository), Box::new(cache));

            let request = CreateAssetRequest {
                name: "Benchmark Asset".to_string(),
                description: "Asset created for benchmarking".to_string(),
                asset_type: AssetType::RealEstate,
                total_value: dec!(1000000.00),
                owner_id: Uuid::new_v4().to_string(),
                location: Some("Test Location".to_string()),
            };

            black_box(service.create_asset(request).await.unwrap())
        })
    });
}

/// Benchmark asset retrieval performance
fn bench_asset_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup: Create assets for retrieval
    let (service, asset_ids) = rt.block_on(async {
        let repository = InMemoryAssetRepository::new();
        let cache = InMemoryCache::new();
        let service = AssetService::new(Box::new(repository), Box::new(cache));
        
        let mut asset_ids = Vec::new();
        for i in 0..100 {
            let request = CreateAssetRequest {
                name: format!("Asset {}", i),
                description: format!("Description {}", i),
                asset_type: AssetType::RealEstate,
                total_value: dec!(1000000.00),
                owner_id: Uuid::new_v4().to_string(),
                location: None,
            };
            let asset = service.create_asset(request).await.unwrap();
            asset_ids.push(asset.id.to_string());
        }
        (service, asset_ids)
    });

    c.bench_function("asset_retrieval", |b| {
        b.to_async(&rt).iter(|| async {
            let asset_id = &asset_ids[fastrand::usize(..asset_ids.len())];
            black_box(service.get_asset(asset_id).await.unwrap())
        })
    });
}

/// Benchmark asset listing with different page sizes
fn bench_asset_listing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Setup: Create many assets
    let service = rt.block_on(async {
        let repository = InMemoryAssetRepository::new();
        let cache = InMemoryCache::new();
        let service = AssetService::new(Box::new(repository), Box::new(cache));
        
        for i in 0..1000 {
            let request = CreateAssetRequest {
                name: format!("Asset {}", i),
                description: format!("Description {}", i),
                asset_type: if i % 2 == 0 { AssetType::RealEstate } else { AssetType::Commodity },
                total_value: dec!(1000000.00),
                owner_id: Uuid::new_v4().to_string(),
                location: None,
            };
            service.create_asset(request).await.unwrap();
        }
        service
    });

    let mut group = c.benchmark_group("asset_listing");
    
    for page_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("page_size", page_size),
            page_size,
            |b, &page_size| {
                b.to_async(&rt).iter(|| async {
                    let pagination = Pagination { page: 1, per_page: page_size };
                    let filters = AssetFilters {
                        asset_type: None,
                        owner_id: None,
                        status: None,
                        min_value: None,
                        max_value: None,
                    };
                    black_box(service.list_assets(pagination, filters).await.unwrap())
                })
            },
        );
    }
    group.finish();
}

/// Benchmark cache performance
fn bench_cache_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("cache_set_get", |b| {
        b.to_async(&rt).iter(|| async {
            let cache = InMemoryCache::new();
            let key = "test_key";
            let value = "test_value";
            
            // Set operation
            cache.set(key, value, Duration::from_secs(60)).await.unwrap();
            
            // Get operation
            black_box(cache.get::<String>(key).await.unwrap())
        })
    });
}

/// Benchmark concurrent asset operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("concurrent_asset_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let repository = InMemoryAssetRepository::new();
            let cache = InMemoryCache::new();
            let service = AssetService::new(Box::new(repository), Box::new(cache));
            
            let tasks: Vec<_> = (0..10).map(|i| {
                let service = &service;
                async move {
                    let request = CreateAssetRequest {
                        name: format!("Concurrent Asset {}", i),
                        description: format!("Description {}", i),
                        asset_type: AssetType::RealEstate,
                        total_value: dec!(1000000.00),
                        owner_id: Uuid::new_v4().to_string(),
                        location: None,
                    };
                    service.create_asset(request).await.unwrap()
                }
            }).collect();
            
            black_box(futures::future::join_all(tasks).await)
        })
    });
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("large_asset_batch", |b| {
        b.to_async(&rt).iter(|| async {
            let repository = InMemoryAssetRepository::new();
            let cache = InMemoryCache::new();
            let service = AssetService::new(Box::new(repository), Box::new(cache));
            
            // Create a large batch of assets
            for i in 0..100 {
                let request = CreateAssetRequest {
                    name: format!("Batch Asset {}", i),
                    description: "A".repeat(1000), // Large description
                    asset_type: AssetType::RealEstate,
                    total_value: dec!(1000000.00),
                    owner_id: Uuid::new_v4().to_string(),
                    location: Some("Location".repeat(100)),
                };
                service.create_asset(request).await.unwrap();
            }
            
            // List all assets
            let pagination = Pagination { page: 1, per_page: 100 };
            let filters = AssetFilters {
                asset_type: None,
                owner_id: None,
                status: None,
                min_value: None,
                max_value: None,
            };
            black_box(service.list_assets(pagination, filters).await.unwrap())
        })
    });
}

/// Benchmark serialization/deserialization performance
fn bench_serialization(c: &mut Criterion) {
    let asset = Asset {
        id: Uuid::new_v4(),
        name: "Test Asset".to_string(),
        description: "Test Description".to_string(),
        asset_type: AssetType::RealEstate,
        total_value: dec!(1000000.00),
        currency: "USD".to_string(),
        owner_id: Uuid::new_v4(),
        location: Some("Test Location".to_string()),
        metadata: None,
        status: AssetStatus::Active,
        is_tokenized: false,
        token_address: None,
        blockchain_network: None,
        token_supply: None,
        token_symbol: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    c.bench_function("asset_serialization", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&asset).unwrap())
        })
    });

    let serialized = serde_json::to_string(&asset).unwrap();
    c.bench_function("asset_deserialization", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<Asset>(&serialized).unwrap())
        })
    });
}

criterion_group!(
    benches,
    bench_asset_creation,
    bench_asset_retrieval,
    bench_asset_listing,
    bench_cache_operations,
    bench_concurrent_operations,
    bench_memory_usage,
    bench_serialization
);

criterion_main!(benches);
