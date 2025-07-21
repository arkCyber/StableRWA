# StableRWA 平台

**🌟 世界级企业 Web3 RWA 平台**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Web3](https://img.shields.io/badge/web3-enabled-green.svg)](https://web3.foundation)
[![DeFi](https://img.shields.io/badge/defi-integrated-purple.svg)](https://defipulse.com)

**中文** | [English](README_EN.md)

## 🎯 项目概述

StableRWA 是一个**世界级的企业 Web3 平台**，专注于现实世界资产（RWA）代币化，集成完整的 DeFi 生态系统、AI 驱动的风险管理、监管科技自动化和先进的隐私保护。采用 Rust 构建，确保最高的性能、安全性和可扩展性。

![StableRWA 仪表板](assets/screenshots/dashboard-main.png)

### 🤖 AI 服务界面
![AI 服务](assets/screenshots/ai-services.png)
*先进的 AI 驱动服务，包括资产估值、风险评估和市场智能分析*

## 🎯 AI 赋能技术框架特性

**🤖 AI 智能化** - 集成 OpenAI GPT 模型，提供智能资产估值、风险评估、市场分析  
**📦 模块化架构** - 提供可复用的核心组件和微服务模块，支持二次开发  
**🔧 开发工具链** - 完整的开发、测试、部署工具集，AI 辅助代码生成  
**🏗️ 基础设施** - 企业级的安全、监控、配置管理框架  
**🔌 扩展接口** - 标准化的 API 和插件系统，支持 AI 模块扩展  
**📚 技术文档** - 完整的框架使用指南和最佳实践，AI 生成的代码示例  

## ✨ 核心特性

🤖 **AI 智能赋能** - OpenAI GPT 集成，智能资产分析、自动化决策、风险评估  
🔗 **多链支持** - 支持 Ethereum、Polygon、Solana 等主流区块链网络  
🏢 **企业级架构** - 微服务架构，支持水平扩展和高并发处理  
🔐 **安全可靠** - JWT 认证、RBAC 权限控制、数据加密、审计日志  
💰 **支付集成** - 集成 Stripe、PayPal 等主流支付服务商  
📊 **实时监控** - Prometheus 指标、Grafana 仪表板、分布式链路追踪  
🔄 **事件驱动** - CQRS 和事件溯源模式，保证数据一致性  
🐳 **容器化部署** - Docker 和 Kubernetes 原生支持  
🧪 **完整测试** - 单元测试、集成测试、负载测试覆盖  
⚡ **二次开发友好** - 模块化设计，支持快速定制和扩展  

## 🧠 AI 智能功能

### 🎯 智能资产管理
- **AI 资产估值** - 基于市场数据和历史趋势的智能估值
- **风险评估** - AI 驱动的风险分析和预警系统
- **市场预测** - 利用机器学习预测资产价格趋势
- **智能推荐** - 个性化的投资建议和资产配置

### 🤖 AI 辅助开发
- **代码生成** - AI 辅助生成智能合约和业务逻辑
- **自动化测试** - AI 生成测试用例和测试数据
- **文档生成** - 自动生成 API 文档和技术文档
- **代码审查** - AI 辅助代码质量检查和安全审计

### 📈 智能分析
- **数据洞察** - AI 分析用户行为和市场趋势
- **异常检测** - 智能识别异常交易和风险事件
- **性能优化** - AI 驱动的系统性能优化建议
- **预测分析** - 基于历史数据的业务预测

## 🏗️ 系统架构

StableRWA 采用现代微服务架构，集成 AI 服务，基于事件驱动通信模式，专为高可扩展性、可靠性和可维护性而设计。

### 🔧 核心服务

| 服务 | 描述 | 端口 | 技术栈 |
|------|------|------|--------|
| **Gateway** | API 网关 & 负载均衡器 | 8080 | Rust, Axum, Tower |
| **User Service** | 用户管理 & 身份认证 | 8081 | Rust, SQLx, JWT |
| **Asset Service** | 资产管理 & 代币化 | 8082 | Rust, Web3, 区块链 |
| **Payment Service** | 支付处理 & 金融操作 | 8083 | Rust, Stripe, 银行 API |
| **AI Service** | AI 智能分析 & 决策 | 8090 | Rust, OpenAI, 机器学习 |

### 🧱 核心库

| 库 | 用途 | 功能特性 |
|----|------|----------|
| **core-blockchain** | 区块链集成 | 多链支持、智能合约、Web3 |
| **core-config** | 配置管理 | 环境配置、验证、热重载 |
| **core-database** | 数据库操作 | 连接池、迁移、事务 |
| **core-events** | 事件驱动架构 | 事件溯源、CQRS、消息队列 |
| **core-observability** | 监控日志 | 指标、链路追踪、健康检查 |
| **core-security** | 安全加密 | JWT、加密、限流 |
| **core-utils** | 通用工具 | 验证、序列化、错误处理 |
| **core-ai** | AI 智能服务 | OpenAI 集成、机器学习、数据分析 |

## 🚀 快速开始

### 环境要求

- **Rust** 1.75+ 和 Cargo
- **Docker** 20.10+ 和 Docker Compose
- **PostgreSQL** 15+ (或使用 Docker)
- **Redis** 7+ (或使用 Docker)
- **Node.js** 18+ (用于前端开发)
- **OpenAI API Key** (用于 AI 功能)

### 🐳 Docker 开发环境

1. **克隆仓库:**
```bash
git clone https://github.com/arkSong/StableRWA.git
cd StableRWA
```

2. **配置环境变量:**
```bash
cp .env.example .env
# 编辑 .env 文件，添加 OpenAI API Key
echo "OPENAI_API_KEY=sk-your-openai-api-key" >> .env
```

3. **启动所有服务:**
```bash
docker-compose up -d
```

4. **验证服务运行:**
```bash
# 检查服务健康状态
curl http://localhost:8080/health

# 测试 AI 服务
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{"prompt": "分析房地产代币化的市场前景"}'
```

### 🛠️ 本地开发环境

1. **设置数据库:**
```bash
# 启动 PostgreSQL 和 Redis
docker-compose up -d postgres redis

# 运行数据库迁移
cargo install sqlx-cli
sqlx migrate run --database-url postgresql://rwa_user:rwa_password@localhost:5432/rwa_dev
```

2. **构建并运行服务:**
```bash
# 构建所有服务
cargo build --workspace

# 运行各个服务 (在不同终端中)
cargo run --bin service-gateway
cargo run --bin service-user
cargo run --bin service-asset
cargo run --bin service-payment
cargo run --bin ai-service
```

## 🤖 AI 功能使用示例

### 智能资产分析
```bash
# 资产估值分析
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "分析曼哈顿一个建于2020年、面积10,000平方英尺的商业地产的市场价值",
    "max_tokens": 200,
    "temperature": 0.7
  }'

# 风险评估
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "评估在当前市场条件下代币化500万美元房地产投资组合的投资风险",
    "max_tokens": 150,
    "temperature": 0.5
  }'

# 市场趋势分析
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "RWA代币化的当前趋势是什么，它们对传统金融有什么影响？",
    "max_tokens": 300,
    "temperature": 0.8
  }'
```

### AI 模型信息
```bash
# 获取可用的AI模型和功能
curl http://localhost:8090/ai/model
```

### 智能资产估值
```bash
curl -X POST http://localhost:8090/ai-asset-valuation \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_type": "real_estate",
    "location": "北京市朝阳区",
    "area": 120,
    "year_built": 2020
  }'
```

### 风险评估
```bash
curl -X POST http://localhost:8090/ai-risk-assessment \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_id": "asset-123",
    "investment_amount": 1000000,
    "time_horizon": "1年"
  }'
```

### 市场分析
```bash
curl -X POST http://localhost:8090/ai-market-analysis \
  -H 'Content-Type: application/json' \
  -d '{
    "market": "房地产",
    "region": "一线城市",
    "analysis_type": "趋势预测"
  }'
```

## 🔧 二次开发指南

### 添加自定义 AI 模块

1. **创建 AI 插件:**
```rust
use stablerwa_framework::core_ai::{AIPlugin, AIRequest, AIResponse};

pub struct CustomAIPlugin;

impl AIPlugin for CustomAIPlugin {
    async fn process(&self, request: AIRequest) -> Result<AIResponse, AIError> {
        // 实现自定义 AI 逻辑
        Ok(AIResponse::new("Custom AI response"))
    }
}
```

2. **注册插件:**
```rust
let ai_service = AIService::builder()
    .add_plugin(Box::new(CustomAIPlugin))
    .build();
```

### 扩展区块链支持

1. **实现区块链适配器:**
```rust
use stablerwa_framework::core_blockchain::{BlockchainAdapter, ChainConfig};

pub struct CustomChainAdapter;

impl BlockchainAdapter for CustomChainAdapter {
    async fn deploy_contract(&self, contract: &Contract) -> Result<String, BlockchainError> {
        // 实现自定义区块链部署逻辑
    }
}
```

### 添加自定义服务

1. **创建新的微服务:**
```rust
use stablerwa_framework::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let framework = StableRWAFramework::builder()
        .with_config_from_env()
        .with_database()
        .with_ai_service()
        .build()
        .await?;
    
    // 启动自定义服务
    framework.run_custom_service(MyCustomService).await
}
```

## 📚 文档

### API 文档
- **OpenAPI 规范**: 服务运行时访问 `/docs`
- **Postman 集合**: `docs/api/StableRWA.postman_collection.json`
- **API 示例**: `docs/api/examples/`
- **AI API 文档**: `docs/api/ai-endpoints.md`

### 架构文档
- **系统设计**: `docs/architecture/system-design.md`
- **AI 架构**: `docs/architecture/ai-architecture.md`
- **数据库架构**: `docs/architecture/database-schema.md`
- **事件流**: `docs/architecture/event-flow.md`
- **安全模型**: `docs/architecture/security.md`

### 开发文档
- **二次开发指南**: `docs/development/secondary-development.md`
- **AI 集成指南**: `docs/development/ai-integration.md`
- **插件开发**: `docs/development/plugin-development.md`
- **自定义服务**: `docs/development/custom-services.md`

## 🧪 测试

### 运行测试

```bash
# 单元测试
cargo test --workspace --lib

# 集成测试
cargo test --workspace --test '*'

# AI 功能测试
cargo test --package ai-service

# 负载测试 (需要 k6)
k6 run tests/load/api-load-test.js

# 烟雾测试
./scripts/smoke-tests.sh development
```

## 🚢 部署

### 生产环境部署

```bash
# 部署到 Kubernetes
./scripts/deploy.sh -e production -v v1.0.0 deploy

# 部署到测试环境
./scripts/deploy.sh -e staging -v develop deploy
```

## 🤝 贡献

我们欢迎贡献！请查看我们的 [贡献指南](CONTRIBUTING.md) 了解详情。

### 开发工作流
1. Fork 仓库
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 进行更改并添加测试
4. 运行测试套件: `cargo test --workspace`
5. 提交更改: `git commit -m 'Add amazing feature'`
6. 推送到分支: `git push origin feature/amazing-feature`
7. 打开 Pull Request

## 🤝 支持单位

StableRWA 平台得到以下领先技术公司的支持：

### 🏢 企业合作伙伴
- **同大方舟未来网络科技有限公司（香港）**
  *Tongda Ark Future Network Technology Co., Ltd. (Hong Kong)*
- **ARKMETA CRYPTO NETWORK LIMITED**
- **龙眼慧（上海）网络科技有限公司**
  *Longan Wisdom (Shanghai) Network Technology Co., Ltd.*

这些组织为企业级 Web3 RWA 解决方案的发展提供战略指导、技术专长和资源支持。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 📞 支持

- **文档**: [docs.stablerwa.com](https://docs.stablerwa.com)
- **问题**: [GitHub Issues](https://github.com/arkSong/StableRWA/issues)
- **讨论**: [GitHub Discussions](https://github.com/arkSong/StableRWA/discussions)
- **邮箱**: arksong2018@gmail.com

---

**由 StableRWA 团队用 ❤️ 构建**

## 联系方式
- **作者:** arkSong (arksong2018@gmail.com)
- 如有问题、建议或贡献，请联系作者。

---

**StableRWA - AI 赋能的未来资产代币化平台** 🚀🤖
