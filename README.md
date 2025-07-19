# StableRWA - AI 赋能的企业级真实世界资产代币化技术框架平台

[![CI/CD Pipeline](https://github.com/arkSong/StableRWA/actions/workflows/ci.yml/badge.svg)](https://github.com/arkSong/StableRWA/actions/workflows/ci.yml)
[![Security Audit](https://github.com/arkSong/StableRWA/actions/workflows/security.yml/badge.svg)](https://github.com/arkSong/StableRWA/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Kubernetes](https://img.shields.io/badge/kubernetes-ready-blue.svg)](https://kubernetes.io)
[![AI Powered](https://img.shields.io/badge/AI-powered-purple.svg)](https://openai.com)
[![Framework](https://img.shields.io/badge/framework-enterprise-green.svg)](https://github.com/arkSong/StableRWA)

**🚀 StableRWA** 是采用 Rust 编写的 AI 人工智能赋能的企业级真实世界资产（RWA）代币化技术框架平台。作为一个完整的可二次开发的 Web3 区块链技术框架，StableRWA 集成了先进的 AI 技术，提供了构建智能化 Web3 区块链应用所需的全套基础设施和开发工具，支持快速构建、部署和扩展 RWA 相关的区块链应用。

## 🌐 多语言文档

- **中文文档**: [README_CN.md](README_CN.md)
- **English Documentation**: [README_EN.md](README_EN.md)

## 🎯 技术框架特性

**📦 模块化架构** - 提供可复用的核心组件和微服务模块
**🔧 开发工具链** - 完整的开发、测试、部署工具集
**🏗️ 基础设施** - 企业级的安全、监控、配置管理框架
**🔌 扩展接口** - 标准化的 API 和插件系统
**📚 技术文档** - 完整的框架使用指南和最佳实践

## ✨ 核心特性

🔗 **多链支持** - 支持 Ethereum、Polygon、Solana 等主流区块链网络
🏢 **企业级架构** - 微服务架构，支持水平扩展和高并发处理
🔐 **安全可靠** - JWT 认证、RBAC 权限控制、数据加密、审计日志
💰 **支付集成** - 集成 Stripe、PayPal 等主流支付服务商
📊 **实时监控** - Prometheus 指标、Grafana 仪表板、分布式链路追踪
🔄 **事件驱动** - CQRS 和事件溯源模式，保证数据一致性
🐳 **容器化部署** - Docker 和 Kubernetes 原生支持
🧪 **完整测试** - 单元测试、集成测试、负载测试覆盖

## 🏗️ 系统架构

StableRWA 采用现代微服务架构，基于事件驱动通信模式，专为高可扩展性、可靠性和可维护性而设计。

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

### 🔄 架构图

```mermaid
graph TB
    A[API Gateway :8080] --> B[User Service :8081]
    A --> C[Asset Service :8082]
    A --> D[Payment Service :8083]
    A --> E[AI Service :8090]

    B --> E[PostgreSQL]
    C --> E
    D --> E

    B --> F[Redis Cache]
    C --> F
    D --> F

    C --> G[Blockchain Networks]
    G --> H[Ethereum]
    G --> I[Polygon]
    G --> J[Solana]

    D --> K[Payment Providers]
    K --> L[Stripe]
    K --> M[PayPal]

    N[Event Bus] --> B
    N --> C
    N --> D
    N --> E

    O[Monitoring Stack] --> P[Prometheus]
    O --> Q[Grafana]
    O --> R[Jaeger]
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75+ with Cargo
- **Docker** 20.10+ and Docker Compose
- **PostgreSQL** 15+ (or use Docker)
- **Redis** 7+ (or use Docker)
- **Node.js** 18+ (for frontend development)
- **OpenAI API Key** (for AI features)

### 🐳 Docker Development Setup

1. **Clone the repository:**
```bash
git clone https://github.com/rwa-platform/rwa-platform.git
cd rwa-platform
```

2. **Configure environment variables:**
```bash
cp .env.example .env
# Edit .env file and add your OpenAI API Key
echo "OPENAI_API_KEY=sk-your-openai-api-key" >> .env
```

3. **Start all services with Docker Compose:**
```bash
docker-compose up -d
```

4. **Verify services are running:**
```bash
# Check service health
curl http://localhost:8080/health

# Test AI service
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{"prompt": "Analyze the market prospects of real estate tokenization"}'

# View logs
docker-compose logs -f gateway
```

5. **Access the platform:**
- API Gateway: http://localhost:8080
- AI Service: http://localhost:8090
- Grafana Dashboard: http://localhost:3000 (admin/admin)
- Jaeger Tracing: http://localhost:16686
- Prometheus Metrics: http://localhost:9090

### 🛠️ Local Development Setup

1. **Set up the database:**
```bash
# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Run database migrations
cargo install sqlx-cli
sqlx migrate run --database-url postgresql://rwa_user:rwa_password@localhost:5432/rwa_dev
```

2. **Configure environment:**
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Build and run services:**
```bash
# Build all services
cargo build --workspace

# Run individual services (in separate terminals)
cargo run --bin service-gateway
cargo run --bin service-user
cargo run --bin service-asset
cargo run --bin service-payment
cargo run --bin ai-service

## 📚 Documentation

### API Documentation
- **OpenAPI Spec**: Available at `/docs` when services are running
- **Postman Collection**: `docs/api/RWA-Platform.postman_collection.json`
- **API Examples**: `docs/api/examples/`

### Architecture Documentation
- **System Design**: `docs/architecture/system-design.md`
- **Database Schema**: `docs/architecture/database-schema.md`
- **Event Flow**: `docs/architecture/event-flow.md`
- **Security Model**: `docs/architecture/security.md`

### Deployment Documentation
- **Kubernetes**: `docs/deployment/kubernetes.md`
- **Docker**: `docs/deployment/docker.md`
- **AWS**: `docs/deployment/aws.md`
- **Monitoring**: `docs/deployment/monitoring.md`

## 🤖 AI Features Usage Examples

### Intelligent Asset Analysis
```bash
# Asset valuation analysis
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "Analyze the market value of a commercial property in Manhattan with 10,000 sq ft built in 2020",
    "max_tokens": 200,
    "temperature": 0.7
  }'

# Risk assessment
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "Assess the investment risk of tokenizing a $5M real estate portfolio in current market conditions",
    "max_tokens": 150,
    "temperature": 0.5
  }'

# Market trend analysis
curl -X POST http://localhost:8090/ai/complete \
  -H 'Content-Type: application/json' \
  -d '{
    "prompt": "What are the current trends in RWA tokenization and their impact on traditional finance?",
    "max_tokens": 300,
    "temperature": 0.8
  }'
```

### AI Model Information
```bash
# Get available AI models and capabilities
curl http://localhost:8090/ai/model
```

## 🧪 Testing

### Running Tests

```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --test '*'

# Load tests (requires k6)
k6 run tests/load/api-load-test.js

# Smoke tests
./scripts/smoke-tests.sh development
```

### Test Coverage

```bash
# Generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out html --output-dir coverage/
```

## 🚢 Deployment

### Production Deployment

```bash
# Deploy to Kubernetes
./scripts/deploy.sh -e production -v v1.0.0 deploy

# Deploy to staging
./scripts/deploy.sh -e staging -v develop deploy
```

### Environment Configuration

| Environment | Purpose | URL |
|-------------|---------|-----|
| **Development** | Local development | http://localhost:8080 |
| **Staging** | Pre-production testing | https://staging-api.rwa-platform.com |
| **Production** | Live environment | https://api.rwa-platform.com |

## 🔐 Security

### Security Features
- **JWT Authentication** with refresh tokens
- **Role-based Access Control** (RBAC)
- **API Rate Limiting** and DDoS protection
- **Data Encryption** at rest and in transit
- **Audit Logging** for all operations
- **Security Headers** and CORS protection

### Security Auditing
```bash
# Run security audit
cargo audit

# Check for vulnerabilities
./scripts/security-scan.sh
```

## 📊 Monitoring & Observability

### Metrics & Monitoring
- **Prometheus** for metrics collection
- **Grafana** for visualization and dashboards
- **Jaeger** for distributed tracing
- **Custom metrics** for business KPIs

### Health Checks
```bash
# Service health
curl http://localhost:8080/health

# Detailed health with dependencies
curl http://localhost:8080/health/detailed
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the test suite: `cargo test --workspace`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Code Standards
- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **Testing**: Maintain >90% test coverage
- **Documentation**: Document all public APIs
- **Security**: Follow OWASP guidelines

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Rust Community** for excellent tooling and libraries
- **Tokio** for async runtime
- **SQLx** for database operations
- **Axum** for web framework
- **Tower** for middleware ecosystem

## 📞 Support

- **Documentation**: [docs.rwa-platform.com](https://docs.rwa-platform.com)
- **Issues**: [GitHub Issues](https://github.com/rwa-platform/rwa-platform/issues)
- **Discussions**: [GitHub Discussions](https://github.com/rwa-platform/rwa-platform/discussions)
- **Email**: support@rwa-platform.com

---

**Built with ❤️ by the RWA Platform Team**
- Frontend: Next.js app in `/webui` (use `pnpm` for package management)
- All code is type-safe, commented, and tested

---

## Contact
- **Owner:** arkSong (arksong2018@gmail.com)
- For questions, suggestions, or contributions, please contact the owner. 

---

## API Endpoints

- **Asset Service:**      `http://localhost:8080/assets`
- **User Service:**       `http://localhost:8081/users`
- **Payment Service:**    `http://localhost:8082/payments`
- **Auth Service:**       `http://localhost:8083/login`
- **Asset Service (Ethereum Block):** `http://localhost:8080/eth-block-number`
- **Asset Service (BSC Block):** `http://localhost:8080/bsc-block-number`
- **AI Service:**         `http://localhost:8090/ai/complete`

### Ethereum Integration Example

- The asset service exposes `/eth-block-number` to fetch the latest Ethereum block number using the ethers crate.
- Configure the Ethereum node URL with the `ETH_NODE_URL` environment variable (default: `http://localhost:8545`).
- Example usage:

  ```sh
  # With default local node
  curl http://localhost:8080/eth-block-number

  # With Infura (set env variable)
  ETH_NODE_URL=https://mainnet.infura.io/v3/YOUR-PROJECT-ID cargo run -p service-asset --bin service-asset
  curl http://localhost:8080/eth-block-number
  ``` 

### Binance Smart Chain (BSC) Integration Example

- The asset service exposes `/bsc-block-number` to fetch the latest Binance Smart Chain block number using the bsc-integration crate.
- Configure the BSC node URL with the `BSC_RPC_URL` environment variable (default: `https://bsc-dataseed.binance.org/`).
- Example usage:

  ```sh
  # With default public BSC node
  curl http://localhost:8080/bsc-block-number

  # With custom BSC node (set env variable)
  BSC_RPC_URL=https://bsc-dataseed1.binance.org cargo run -p service-asset --bin service-asset
  curl http://localhost:8080/bsc-block-number
  ``` 

### AI Integration Example

- The ai-service exposes `/ai/complete` to call OpenAI's completion API with a prompt.
- Configure the OpenAI API key with the `OPENAI_API_KEY` environment variable.
- Example usage:

  ```sh
  # With your OpenAI API key
  export OPENAI_API_KEY=sk-...yourkey...
  curl -X POST http://localhost:8090/ai/complete \
    -H 'Content-Type: application/json' \
    -d '{"prompt": "Hello AI, what is RWA?"}'
  ```

  ```sh
  # Get AI model information
  curl http://localhost:8090/ai/model
  ``` 