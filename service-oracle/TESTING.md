# Oracle Service - 企业级测试指南

## 📋 测试概述

RWA Oracle Service 采用企业级测试标准，确保代码质量、可靠性和性能。本文档详细说明了测试策略、执行方法和质量标准。

## 🎯 测试目标

- **代码覆盖率**: >95%
- **功能覆盖**: 100% 的公共 API
- **错误处理**: 100% 的错误路径
- **性能标准**: API 响应时间 <100ms
- **并发处理**: 支持 1000+ 并发请求

## 🏗️ 测试架构

### 测试层次结构

```
测试金字塔
    ┌─────────────────┐
    │   E2E Tests     │  ← 端到端测试 (少量)
    ├─────────────────┤
    │ Integration     │  ← 集成测试 (适量)
    │    Tests        │
    ├─────────────────┤
    │   Unit Tests    │  ← 单元测试 (大量)
    └─────────────────┘
```

### 测试类型

1. **单元测试** (Unit Tests)
   - 测试单个函数和方法
   - 快速执行 (<1s)
   - 高覆盖率 (>95%)

2. **集成测试** (Integration Tests)
   - 测试组件间交互
   - 数据库和缓存集成
   - API 端点测试

3. **性能测试** (Performance Tests)
   - 负载测试
   - 压力测试
   - 并发测试

4. **安全测试** (Security Tests)
   - 依赖漏洞扫描
   - 代码安全审计
   - 输入验证测试

## 🚀 快速开始

### 环境准备

```bash
# 1. 安装依赖
cargo install cargo-tarpaulin  # 覆盖率工具
cargo install cargo-audit      # 安全审计
cargo install sqlx-cli         # 数据库工具

# 2. 启动测试服务
docker-compose up -d postgres redis

# 3. 设置环境变量
export TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/oracle_test"
export TEST_REDIS_URL="redis://localhost:6379/1"
```

### 运行测试

```bash
# 运行所有测试
./scripts/run_tests.sh

# 运行特定类型的测试
./scripts/run_tests.sh --unit           # 单元测试
./scripts/run_tests.sh --integration    # 集成测试
./scripts/run_tests.sh --coverage       # 覆盖率测试

# 使用 Cargo 直接运行
cargo test                              # 所有测试
cargo test --lib                        # 单元测试
cargo test --test integration_tests     # 集成测试
```

## 📊 测试覆盖率

### 当前覆盖率状态

| 模块 | 覆盖率 | 测试数量 | 状态 |
|------|--------|----------|------|
| models | 98% | 20 | ✅ |
| service | 95% | 15 | ✅ |
| aggregator | 97% | 18 | ✅ |
| cache | 94% | 12 | ✅ |
| health | 96% | 8 | ✅ |
| metrics | 99% | 12 | ✅ |
| handlers | 93% | 15 | ✅ |
| providers | 96% | 32 | ✅ |
| config | 98% | 18 | ✅ |
| error | 100% | 15 | ✅ |

### 生成覆盖率报告

```bash
# 生成 HTML 报告
cargo tarpaulin --out Html --output-dir coverage

# 生成 XML 报告 (CI/CD)
cargo tarpaulin --out Xml --output-dir coverage

# 查看报告
open coverage/tarpaulin-report.html
```

## 🧪 测试编写指南

### 测试命名规范

```rust
#[test]
fn test_function_name_scenario_expected_result() {
    // 测试实现
}

// 示例
#[test]
fn test_aggregate_price_mean_with_valid_data_returns_correct_average() {
    // ...
}

#[test]
fn test_get_asset_price_with_invalid_asset_id_returns_validation_error() {
    // ...
}
```

### 测试结构 (AAA 模式)

```rust
#[test]
fn test_example() {
    // Arrange - 准备测试数据和环境
    let input_data = create_test_data();
    let expected_result = ExpectedResult::new();
    
    // Act - 执行被测试的功能
    let actual_result = function_under_test(input_data);
    
    // Assert - 验证结果
    assert_eq!(actual_result, expected_result);
}
```

### Mock 和测试数据

```rust
// 使用测试工厂函数
fn create_test_asset_price(asset_id: &str, price: Decimal) -> AssetPrice {
    AssetPrice {
        asset_id: asset_id.to_string(),
        price,
        currency: "USD".to_string(),
        timestamp: Utc::now(),
        confidence: dec!(0.95),
        source: "test".to_string(),
        metadata: Some(HashMap::new()),
    }
}

// Mock 外部依赖
struct MockProvider {
    should_fail: bool,
    response_delay: Duration,
}

#[async_trait]
impl PriceProvider for MockProvider {
    async fn get_price(&self, asset_id: &str, currency: &str) -> OracleResult<AssetPrice> {
        if self.should_fail {
            return Err(OracleError::Provider {
                provider: "mock".to_string(),
                message: "Mock failure".to_string(),
            });
        }
        
        tokio::time::sleep(self.response_delay).await;
        Ok(create_test_asset_price(asset_id, dec!(50000.0)))
    }
}
```

## 🔧 测试工具和配置

### Cargo 配置

```toml
# .cargo/config.toml
[alias]
test-unit = "test --lib"
test-integration = "test --test integration_tests"
test-coverage = "tarpaulin --out Html --output-dir coverage"
test-watch = "watch -x test"

[env]
TEST_DATABASE_URL = { value = "postgresql://postgres:postgres@localhost:5432/oracle_test" }
TEST_REDIS_URL = { value = "redis://localhost:6379/1" }
```

### 测试配置文件

```rust
// tests/common/mod.rs
pub fn create_test_config() -> OracleConfig {
    OracleConfig {
        aggregation: AggregationConfig {
            min_sources: 1,  // 降低测试要求
            confidence_threshold: dec!(0.5),
            // ...
        },
        // ...
    }
}
```

## 🚦 CI/CD 集成

### GitHub Actions 工作流

测试流水线包括以下阶段：

1. **代码质量检查**
   - 格式化检查 (`cargo fmt`)
   - 代码规范检查 (`cargo clippy`)
   - 安全审计 (`cargo audit`)

2. **单元测试**
   - 多 Rust 版本测试 (stable, beta)
   - 文档测试

3. **集成测试**
   - PostgreSQL 和 Redis 服务
   - 数据库迁移
   - API 端点测试

4. **覆盖率分析**
   - 生成覆盖率报告
   - 上传到 Codecov
   - 检查覆盖率阈值

5. **性能测试**
   - 基准测试
   - 负载测试

### 本地 CI 模拟

```bash
# 模拟 CI 环境
./scripts/run_tests.sh --all

# 检查代码质量
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo audit
```

## 📈 性能测试

### 基准测试

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn bench_price_aggregation() {
        let aggregator = create_test_aggregator();
        let prices = create_test_prices(100);
        
        let start = Instant::now();
        let result = aggregator.aggregate_prices(&prices, AggregationMethod::Mean).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100); // 应在 100ms 内完成
    }
}
```

### 负载测试

```bash
# 使用 wrk 进行负载测试
wrk -t12 -c400 -d30s http://localhost:8081/api/v1/prices/BTC

# 使用 Apache Bench
ab -n 1000 -c 10 http://localhost:8081/health
```

## 🛡️ 安全测试

### 依赖漏洞扫描

```bash
# 安装和运行 cargo-audit
cargo install cargo-audit
cargo audit

# 检查过时依赖
cargo install cargo-outdated
cargo outdated
```

### 输入验证测试

```rust
#[test]
fn test_input_validation() {
    // 测试 SQL 注入防护
    let malicious_input = "'; DROP TABLE users; --";
    let result = validate_asset_id(malicious_input);
    assert!(result.is_err());
    
    // 测试 XSS 防护
    let xss_input = "<script>alert('xss')</script>";
    let result = validate_asset_id(xss_input);
    assert!(result.is_err());
}
```

## 📝 测试最佳实践

### 1. 测试独立性
- 每个测试应该独立运行
- 不依赖其他测试的状态
- 使用 `setup` 和 `teardown` 函数

### 2. 测试数据管理
- 使用工厂函数创建测试数据
- 避免硬编码值
- 清理测试数据

### 3. 错误测试
- 测试所有错误路径
- 验证错误消息
- 确保错误处理不会导致崩溃

### 4. 异步测试
- 使用 `#[tokio::test]` 进行异步测试
- 设置合理的超时时间
- 测试并发场景

### 5. 测试文档
- 为复杂测试添加注释
- 说明测试目的和预期结果
- 记录测试数据的含义

## 🔍 调试测试

### 运行特定测试

```bash
# 运行特定测试函数
cargo test test_aggregate_price_mean

# 运行特定模块的测试
cargo test aggregator::tests

# 显示测试输出
cargo test -- --nocapture

# 运行被忽略的测试
cargo test -- --ignored
```

### 测试日志

```rust
#[test]
fn test_with_logging() {
    env_logger::init();
    
    log::info!("Starting test");
    let result = function_under_test();
    log::info!("Test result: {:?}", result);
    
    assert!(result.is_ok());
}
```

## 📊 测试报告

### 生成测试报告

```bash
# 运行测试并生成报告
./scripts/run_tests.sh

# 查看生成的报告
cat test_report.md
open coverage/tarpaulin-report.html
```

### 持续监控

- 设置覆盖率阈值警报
- 监控测试执行时间
- 跟踪测试失败率
- 定期审查测试质量

## 🎯 质量门禁

在代码合并前，必须满足以下条件：

- ✅ 所有测试通过
- ✅ 代码覆盖率 ≥ 80%
- ✅ 无安全漏洞
- ✅ 代码格式正确
- ✅ 无 Clippy 警告
- ✅ 性能测试通过

## 📞 支持和帮助

如果在测试过程中遇到问题：

1. 查看测试日志和错误信息
2. 检查环境配置是否正确
3. 参考本文档的故障排除部分
4. 联系开发团队获取支持

---

**维护者**: arkSong (arksong2018@gmail.com)  
**最后更新**: 2024-01-20
