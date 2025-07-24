// =====================================================================================
// File: core-smart-contract/benches/smart_contract_benchmarks.rs
// Description: Benchmark tests for smart contract service performance
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use uuid::Uuid;

use core_smart_contract::{
    AuditConfig, CompilationResult, CompilerConfig, ContractFilter, ContractMetadata,
    ContractState, ContractVersion, DeploymentConfig, DeploymentResult, EthereumDeployer,
    HealthScore, ProxyConfig, ProxyDeployer, ProxyType, SmartContract, SmartContractCompiler,
    SmartContractDeployer, SmartContractService, SmartContractServiceImpl, SolidityCompiler,
    UpgradeConfig, UpgradeStrategy, VyperCompiler,
};

/// Benchmark configuration
struct BenchmarkConfig {
    pub compiler_config: CompilerConfig,
    pub deployment_config: DeploymentConfig,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            compiler_config: CompilerConfig {
                version: "0.8.19".to_string(),
                optimization_enabled: true,
                optimization_runs: 200,
                evm_version: "london".to_string(),
                libraries: HashMap::new(),
                output_selection: vec![
                    "abi".to_string(),
                    "evm.bytecode".to_string(),
                    "metadata".to_string(),
                ],
            },
            deployment_config: DeploymentConfig {
                network: "ethereum".to_string(),
                deployer_address: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6".to_string(),
                gas_limit: Some(3_000_000),
                gas_price: Some(20_000_000_000),
                constructor_args: Vec::new(),
                libraries: HashMap::new(),
                proxy_config: None,
                verification_config: None,
                monitoring_config: None,
            },
        }
    }
}

/// Generate test contract source code
fn generate_contract_source(complexity: usize) -> String {
    let mut source = String::from(
        r#"
        pragma solidity ^0.8.0;
        
        contract BenchmarkContract {
            mapping(address => uint256) public balances;
            mapping(address => mapping(address => uint256)) public allowances;
            
            uint256 public totalSupply;
            string public name;
            string public symbol;
            uint8 public decimals;
            
            event Transfer(address indexed from, address indexed to, uint256 value);
            event Approval(address indexed owner, address indexed spender, uint256 value);
            
            constructor() {
                totalSupply = 1000000 * 10**18;
                name = "BenchmarkToken";
                symbol = "BENCH";
                decimals = 18;
                balances[msg.sender] = totalSupply;
            }
            
            function transfer(address to, uint256 amount) public returns (bool) {
                require(balances[msg.sender] >= amount, "Insufficient balance");
                balances[msg.sender] -= amount;
                balances[to] += amount;
                emit Transfer(msg.sender, to, amount);
                return true;
            }
            
            function approve(address spender, uint256 amount) public returns (bool) {
                allowances[msg.sender][spender] = amount;
                emit Approval(msg.sender, spender, amount);
                return true;
            }
    "#,
    );

    // Add complexity by adding more functions
    for i in 0..complexity {
        source.push_str(&format!(
            r#"
            function complexFunction{}(uint256 a, uint256 b) public pure returns (uint256) {{
                uint256 result = a;
                for (uint256 i = 0; i < b; i++) {{
                    result = result * 2 + 1;
                    if (result > 1000000) {{
                        result = result % 1000000;
                    }}
                }}
                return result;
            }}
        "#,
            i
        ));
    }

    source.push_str("\n}");
    source
}

/// Create test contracts for benchmarking
fn create_benchmark_contracts(count: usize) -> Vec<(String, ContractMetadata)> {
    (0..count)
        .map(|i| {
            let source = generate_contract_source(i % 10); // Vary complexity
            let metadata = ContractMetadata {
                description: format!("Benchmark contract {}", i),
                author: "Benchmark".to_string(),
                license: "MIT".to_string(),
                compiler_version: "0.8.19".to_string(),
                optimization_enabled: true,
                optimization_runs: 200,
                evm_version: "london".to_string(),
                tags: vec!["benchmark".to_string()],
                dependencies: Vec::new(),
                interfaces: Vec::new(),
            };
            (source, metadata)
        })
        .collect()
}

/// Benchmark contract compilation
fn bench_contract_compilation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let compiler = SolidityCompiler::new(
        "solc".to_string(),
        "./cache".to_string(),
        "./temp".to_string(),
    );
    let config = CompilerConfig::default();

    let mut group = c.benchmark_group("contract_compilation");

    for complexity in [1, 5, 10, 20].iter() {
        let source = generate_contract_source(*complexity);

        group.bench_with_input(
            BenchmarkId::new("solidity_compile", complexity),
            complexity,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    compiler
                        .compile(black_box(&source), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark batch compilation
fn bench_batch_compilation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let compiler = SolidityCompiler::new(
        "solc".to_string(),
        "./cache".to_string(),
        "./temp".to_string(),
    );
    let config = CompilerConfig::default();

    let mut group = c.benchmark_group("batch_compilation");

    for batch_size in [1, 5, 10, 20].iter() {
        let sources: Vec<core_smart_contract::SourceFile> = (0..*batch_size)
            .map(|i| core_smart_contract::SourceFile {
                path: format!("Contract{}.sol", i),
                content: generate_contract_source(5),
                imports: Vec::new(),
            })
            .collect();

        group.throughput(Throughput::Elements(*batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_compile", batch_size),
            batch_size,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    compiler
                        .compile_batch(black_box(&sources), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark contract deployment
fn bench_contract_deployment(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let deployer = EthereumDeployer::new("https://mainnet.infura.io/v3/test".to_string(), 1);
    let config = BenchmarkConfig::default();

    let contract = SmartContract {
        id: Uuid::new_v4(),
        name: "BenchmarkContract".to_string(),
        address: String::new(),
        network: "ethereum".to_string(),
        version: ContractVersion::default(),
        metadata: ContractMetadata {
            description: "Benchmark contract".to_string(),
            author: "Benchmark".to_string(),
            license: "MIT".to_string(),
            compiler_version: "0.8.19".to_string(),
            optimization_enabled: true,
            optimization_runs: 200,
            evm_version: "london".to_string(),
            tags: Vec::new(),
            dependencies: Vec::new(),
            interfaces: Vec::new(),
        },
        state: ContractState::Compiled,
        abi: serde_json::Value::Null,
        bytecode: "0x608060405234801561001057600080fd5b50...".to_string(),
        source_code: None,
        proxy_config: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    c.bench_function("contract_deployment", |b| {
        b.to_async(&rt).iter(|| async {
            deployer
                .deploy(black_box(&contract), black_box(&config.deployment_config))
                .await
                .unwrap()
        });
    });
}

/// Benchmark gas estimation
fn bench_gas_estimation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let deployer = EthereumDeployer::new("https://mainnet.infura.io/v3/test".to_string(), 1);
    let config = BenchmarkConfig::default();

    let mut group = c.benchmark_group("gas_estimation");

    for complexity in [1, 5, 10, 20].iter() {
        let bytecode_size = 1000 + (complexity * 500); // Simulate different bytecode sizes
        let contract = SmartContract {
            id: Uuid::new_v4(),
            name: format!("BenchmarkContract{}", complexity),
            address: String::new(),
            network: "ethereum".to_string(),
            version: ContractVersion::default(),
            metadata: ContractMetadata {
                description: "Benchmark contract".to_string(),
                author: "Benchmark".to_string(),
                license: "MIT".to_string(),
                compiler_version: "0.8.19".to_string(),
                optimization_enabled: true,
                optimization_runs: 200,
                evm_version: "london".to_string(),
                tags: Vec::new(),
                dependencies: Vec::new(),
                interfaces: Vec::new(),
            },
            state: ContractState::Compiled,
            abi: serde_json::Value::Null,
            bytecode: "0x".to_string() + &"60".repeat(bytecode_size),
            source_code: None,
            proxy_config: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        group.bench_with_input(
            BenchmarkId::new("gas_estimation", complexity),
            complexity,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    deployer
                        .estimate_deployment_gas(
                            black_box(&contract),
                            black_box(&config.deployment_config),
                        )
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark contract service operations
fn bench_contract_service_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(async {
        let service = SmartContractServiceImpl::new();
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("contract_service_operations");

    // Benchmark contract creation
    group.bench_function("contract_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let metadata = ContractMetadata {
                description: "Benchmark contract".to_string(),
                author: "Benchmark".to_string(),
                license: "MIT".to_string(),
                compiler_version: "0.8.19".to_string(),
                optimization_enabled: true,
                optimization_runs: 200,
                evm_version: "london".to_string(),
                tags: vec!["benchmark".to_string()],
                dependencies: Vec::new(),
                interfaces: Vec::new(),
            };

            service
                .create_contract(
                    black_box("BenchmarkContract".to_string()),
                    black_box(generate_contract_source(5)),
                    black_box(metadata),
                )
                .await
                .unwrap()
        });
    });

    group.finish();
}

/// Benchmark contract filtering and querying
fn bench_contract_filtering(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(async {
        let service = SmartContractServiceImpl::new();
        service.initialize().await.unwrap();

        // Pre-populate with test contracts
        let contracts = create_benchmark_contracts(100);
        for (i, (source, metadata)) in contracts.into_iter().enumerate() {
            service
                .create_contract(format!("BenchmarkContract{}", i), source, metadata)
                .await
                .unwrap();
        }

        service
    });

    let mut group = c.benchmark_group("contract_filtering");

    let filters = vec![
        (
            "no_filter",
            ContractFilter {
                network: None,
                state: None,
                author: None,
                tags: Vec::new(),
                created_after: None,
                created_before: None,
            },
        ),
        (
            "state_filter",
            ContractFilter {
                network: None,
                state: Some(ContractState::Draft),
                author: None,
                tags: Vec::new(),
                created_after: None,
                created_before: None,
            },
        ),
        (
            "author_filter",
            ContractFilter {
                network: None,
                state: None,
                author: Some("Benchmark".to_string()),
                tags: Vec::new(),
                created_after: None,
                created_before: None,
            },
        ),
        (
            "tag_filter",
            ContractFilter {
                network: None,
                state: None,
                author: None,
                tags: vec!["benchmark".to_string()],
                created_after: None,
                created_before: None,
            },
        ),
    ];

    for (filter_name, filter) in filters {
        group.bench_function(filter_name, |b| {
            b.to_async(&rt)
                .iter(|| async { service.list_contracts(black_box(&filter)).await.unwrap() });
        });
    }

    group.finish();
}

/// Benchmark concurrent contract operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(async {
        let service = SmartContractServiceImpl::new();
        service.initialize().await.unwrap();
        service
    });

    let mut group = c.benchmark_group("concurrent_operations");

    for concurrency in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_creation", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();

                    for i in 0..concurrency {
                        let service_clone = service.clone();
                        let metadata = ContractMetadata {
                            description: format!("Concurrent contract {}", i),
                            author: "Benchmark".to_string(),
                            license: "MIT".to_string(),
                            compiler_version: "0.8.19".to_string(),
                            optimization_enabled: true,
                            optimization_runs: 200,
                            evm_version: "london".to_string(),
                            tags: vec!["concurrent".to_string()],
                            dependencies: Vec::new(),
                            interfaces: Vec::new(),
                        };

                        let handle = tokio::spawn(async move {
                            service_clone
                                .create_contract(
                                    format!("ConcurrentContract{}", i),
                                    generate_contract_source(3),
                                    metadata,
                                )
                                .await
                                .unwrap()
                        });
                        handles.push(handle);
                    }

                    let mut results = Vec::new();
                    for handle in handles {
                        results.push(handle.await.unwrap());
                    }
                    results
                });
            },
        );
    }

    group.finish();
}

/// Benchmark contract health monitoring
fn bench_health_monitoring(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(async {
        let service = SmartContractServiceImpl::new();
        service.initialize().await.unwrap();

        // Create a test contract
        let metadata = ContractMetadata {
            description: "Health test contract".to_string(),
            author: "Benchmark".to_string(),
            license: "MIT".to_string(),
            compiler_version: "0.8.19".to_string(),
            optimization_enabled: true,
            optimization_runs: 200,
            evm_version: "london".to_string(),
            tags: Vec::new(),
            dependencies: Vec::new(),
            interfaces: Vec::new(),
        };

        let contract_id = service
            .create_contract(
                "HealthTestContract".to_string(),
                generate_contract_source(5),
                metadata,
            )
            .await
            .unwrap();

        (service, contract_id)
    });

    c.bench_function("contract_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            service
                .0
                .get_contract_health(black_box(&service.1))
                .await
                .unwrap()
        });
    });
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("service_initialization", |b| {
        b.to_async(&rt).iter(|| async {
            let service = SmartContractServiceImpl::new();
            service.initialize().await.unwrap();
            service
        });
    });
}

/// Benchmark different compiler versions
fn bench_compiler_versions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let compiler = SolidityCompiler::new(
        "solc".to_string(),
        "./cache".to_string(),
        "./temp".to_string(),
    );

    let mut group = c.benchmark_group("compiler_versions");

    let versions = vec!["0.8.19", "0.8.18", "0.8.17"];
    let source = generate_contract_source(5);

    for version in versions {
        let config = CompilerConfig {
            version: version.to_string(),
            optimization_enabled: true,
            optimization_runs: 200,
            evm_version: "london".to_string(),
            libraries: HashMap::new(),
            output_selection: vec![
                "abi".to_string(),
                "evm.bytecode".to_string(),
                "metadata".to_string(),
            ],
        };

        group.bench_function(version, |b| {
            b.to_async(&rt).iter(|| async {
                compiler
                    .compile(black_box(&source), black_box(&config))
                    .await
                    .unwrap()
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_contract_compilation,
    bench_batch_compilation,
    bench_contract_deployment,
    bench_gas_estimation,
    bench_contract_service_operations,
    bench_contract_filtering,
    bench_concurrent_operations,
    bench_health_monitoring,
    bench_memory_usage,
    bench_compiler_versions
);

criterion_main!(benches);
