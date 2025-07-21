# RWA Platform DID System - 测试指南

## 快速开始

### 运行所有测试
```bash
cd core-did
cargo test --lib --no-default-features
```

### 运行特定模块测试
```bash
# 测试DID核心功能
cargo test --lib did::tests

# 测试密钥管理
cargo test --lib key_manager::tests

# 测试解析器
cargo test --lib resolver::tests

# 测试DID文档
cargo test --lib document::tests
```

### 运行单个测试
```bash
# 运行特定测试
cargo test --lib test_did_generation

# 运行测试并显示输出
cargo test --lib test_did_generation -- --nocapture
```

## 测试覆盖率

### 安装覆盖率工具
```bash
cargo install cargo-tarpaulin
```

### 生成覆盖率报告
```bash
# 生成文本报告
cargo tarpaulin --lib --no-default-features --out Stdout

# 生成HTML报告
cargo tarpaulin --lib --no-default-features --out Html --output-dir ./coverage

# 生成多种格式报告
cargo tarpaulin --lib --no-default-features --out Html --out Lcov --output-dir ./coverage
```

## 测试分类

### 单元测试
测试单个函数或方法的功能：
```bash
# DID标识符测试
cargo test --lib did::tests::test_did_new
cargo test --lib did::tests::test_did_from_str_valid

# 密钥管理测试
cargo test --lib key_manager::tests::test_keypair_generate_ed25519
cargo test --lib key_manager::tests::test_keypair_sign_ed25519
```

### 集成测试
测试多个组件的协作：
```bash
# 端到端工作流测试
cargo test --lib test_did_document_complex_scenario
cargo test --lib test_key_manager_sign_verify_roundtrip
```

### 并发测试
测试多线程和异步操作：
```bash
# 并发操作测试
cargo test --lib test_key_manager_concurrent_operations
cargo test --lib test_concurrent_resolution
```

## 性能测试

### 基准测试
```bash
# 安装基准测试工具
cargo install cargo-criterion

# 运行基准测试（如果有的话）
cargo bench
```

### 性能分析
```bash
# 使用perf进行性能分析
cargo test --lib --release -- --nocapture

# 内存使用分析
valgrind --tool=massif cargo test --lib
```

## 调试测试

### 详细输出
```bash
# 显示所有测试输出
cargo test --lib -- --nocapture

# 显示测试执行时间
cargo test --lib -- --nocapture --test-threads=1
```

### 调试特定测试
```bash
# 使用调试模式运行测试
RUST_LOG=debug cargo test --lib test_name -- --nocapture

# 使用GDB调试
rust-gdb --args target/debug/deps/core_did-* test_name
```

## 测试环境配置

### 环境变量
```bash
# 设置日志级别
export RUST_LOG=debug

# 设置测试线程数
export RUST_TEST_THREADS=1

# 设置测试超时
export RUST_TEST_TIME_UNIT=60000
```

### 依赖项
确保安装了以下依赖：
```toml
[dev-dependencies]
tokio-test = "0.4"
fake = "2.10"
```

## 持续集成

### GitHub Actions配置
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: |
          cd core-did
          cargo test --lib --no-default-features
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --lib --no-default-features --out Lcov
      - name: Upload coverage
        uses: codecov/codecov-action@v1
```

## 测试最佳实践

### 编写测试
1. **命名约定**: 使用`test_`前缀
2. **测试结构**: 使用Arrange-Act-Assert模式
3. **独立性**: 每个测试应该独立运行
4. **清晰性**: 测试名称应该描述测试的内容

### 示例测试
```rust
#[tokio::test]
async fn test_did_creation_and_validation() {
    // Arrange
    let method_specific_id = "123456789".to_string();
    
    // Act
    let did = Did::new(method_specific_id.clone());
    
    // Assert
    assert_eq!(did.method, "rwa");
    assert_eq!(did.method_specific_id, method_specific_id);
    assert!(did.validate().is_ok());
}
```

### 异步测试
```rust
#[tokio::test]
async fn test_async_operation() {
    let key_manager = KeyManager::new();
    
    let result = key_manager.generate_key(
        "test-key".to_string(),
        KeyType::Ed25519,
        vec![KeyPurpose::Authentication],
    ).await;
    
    assert!(result.is_ok());
}
```

## 故障排除

### 常见问题

1. **编译错误**
   ```bash
   # 清理并重新构建
   cargo clean
   cargo test --lib
   ```

2. **测试超时**
   ```bash
   # 增加超时时间
   cargo test --lib -- --test-timeout 60
   ```

3. **内存不足**
   ```bash
   # 减少并发测试线程
   cargo test --lib -- --test-threads=1
   ```

### 调试技巧
- 使用`println!`或`dbg!`宏进行调试
- 使用`RUST_BACKTRACE=1`获取详细错误信息
- 使用`--nocapture`查看测试输出

## 测试报告

### 生成测试报告
```bash
# 生成JUnit格式报告
cargo test --lib -- -Z unstable-options --format json --report-time

# 生成详细的测试报告
cargo test --lib -- --format pretty
```

### 查看覆盖率报告
```bash
# 在浏览器中查看HTML报告
open coverage/tarpaulin-report.html

# 查看命令行报告
cargo tarpaulin --lib --no-default-features --out Stdout
```

## 贡献指南

### 添加新测试
1. 在相应模块的`tests`模块中添加测试
2. 遵循现有的命名约定
3. 确保测试覆盖边界条件
4. 运行所有测试确保没有回归

### 测试审查清单
- [ ] 测试名称清晰描述功能
- [ ] 测试独立且可重复
- [ ] 包含正常和异常情况
- [ ] 异步测试使用`#[tokio::test]`
- [ ] 测试通过且覆盖率合理
