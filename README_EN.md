# StableRWA Platform

**🌟 World-Class Enterprise Web3 RWA Platform**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Docker](https://img.shields.io/badge/docker-ready-blue.svg)](https://www.docker.com)
[![Web3](https://img.shields.io/badge/web3-enabled-green.svg)](https://web3.foundation)
[![DeFi](https://img.shields.io/badge/defi-integrated-purple.svg)](https://defipulse.com)

[中文文档](README_CN.md) | **English**

## 🎯 Overview

StableRWA is a **world-class enterprise Web3 platform** for Real World Asset (RWA) tokenization, featuring complete DeFi ecosystem integration, AI-driven risk management, regulatory technology automation, and advanced privacy protection. Built with Rust for maximum performance, security, and scalability.

![StableRWA Dashboard](assets/screenshots/dashboard-main.png)

### 🤖 AI Services Interface
![AI Services](assets/screenshots/ai-services.png)
*Advanced AI-powered services for asset valuation, risk assessment, and market intelligence*

## 🎯 AI-Powered Technology Framework Features

**🤖 AI Intelligence** - Integrated OpenAI GPT models for intelligent asset valuation, risk assessment, and market analysis  
**📦 Modular Architecture** - Reusable core components and microservice modules supporting secondary development  
**🔧 Development Toolchain** - Complete development, testing, and deployment tools with AI-assisted code generation  
**🏗️ Infrastructure Framework** - Enterprise-grade security, monitoring, and configuration management  
**🔌 Extension Interfaces** - Standardized APIs and plugin system supporting AI module extensions  
**📚 Technical Documentation** - Comprehensive framework usage guides and best practices with AI-generated code examples  

## ✨ Core Features

🤖 **AI Intelligence Empowerment** - OpenAI GPT integration for intelligent asset analysis, automated decision-making, and risk assessment  
🔗 **Multi-Chain Support** - Support for Ethereum, Polygon, Solana, and other mainstream blockchain networks  
🏢 **Enterprise Architecture** - Microservices architecture supporting horizontal scaling and high-concurrency processing  
🔐 **Security & Reliability** - JWT authentication, RBAC access control, data encryption, and audit logging  
💰 **Payment Integration** - Integration with mainstream payment providers like Stripe and PayPal  
📊 **Real-time Monitoring** - Prometheus metrics, Grafana dashboards, and distributed tracing  
🔄 **Event-Driven** - CQRS and event sourcing patterns ensuring data consistency  
🐳 **Containerized Deployment** - Native Docker and Kubernetes support  
🧪 **Comprehensive Testing** - Unit testing, integration testing, and load testing coverage  
⚡ **Secondary Development Friendly** - Modular design supporting rapid customization and extension  

## 🧠 AI Intelligence Features

### 🎯 Intelligent Asset Management
- **AI Asset Valuation** - Intelligent valuation based on market data and historical trends
- **Risk Assessment** - AI-driven risk analysis and early warning systems
- **Market Prediction** - Machine learning-powered asset price trend forecasting
- **Smart Recommendations** - Personalized investment advice and asset allocation

### 🤖 AI-Assisted Development
- **Code Generation** - AI-assisted smart contract and business logic generation
- **Automated Testing** - AI-generated test cases and test data
- **Documentation Generation** - Automatic API documentation and technical documentation
- **Code Review** - AI-assisted code quality checks and security audits

### 📈 Intelligent Analytics
- **Data Insights** - AI analysis of user behavior and market trends
- **Anomaly Detection** - Intelligent identification of abnormal transactions and risk events
- **Performance Optimization** - AI-driven system performance optimization recommendations
- **Predictive Analytics** - Business forecasting based on historical data

## 🏗️ System Architecture

StableRWA adopts modern microservices architecture with integrated AI services, based on event-driven communication patterns, designed for high scalability, reliability, and maintainability.

### 🔧 Core Services

| Service | Description | Port | Technology Stack |
|---------|-------------|------|------------------|
| **Gateway** | API Gateway & Load Balancer | 8080 | Rust, Axum, Tower |
| **User Service** | User Management & Authentication | 8081 | Rust, SQLx, JWT |
| **Asset Service** | Asset Management & Tokenization | 8082 | Rust, Web3, Blockchain |
| **Payment Service** | Payment Processing & Financial Operations | 8083 | Rust, Stripe, Banking APIs |
| **AI Service** | AI Intelligence Analysis & Decision Making | 8090 | Rust, OpenAI, Machine Learning |

### 🧱 Core Libraries

| Library | Purpose | Features |
|---------|---------|----------|
| **core-blockchain** | Blockchain Integration | Multi-chain support, Smart contracts, Web3 |
| **core-config** | Configuration Management | Environment config, Validation, Hot reload |
| **core-database** | Database Operations | Connection pooling, Migrations, Transactions |
| **core-events** | Event-Driven Architecture | Event sourcing, CQRS, Message queues |
| **core-observability** | Monitoring & Logging | Metrics, Tracing, Health checks |
| **core-security** | Security & Encryption | JWT, Encryption, Rate limiting |
| **core-utils** | Common Utilities | Validation, Serialization, Error handling |
| **core-ai** | AI Intelligence Services | OpenAI integration, Machine learning, Data analysis |

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
git clone https://github.com/arkSong/StableRWA.git
cd StableRWA
```

2. **Configure environment variables:**
```bash
cp .env.example .env
# Edit .env file and add your OpenAI API Key
echo "OPENAI_API_KEY=sk-your-openai-api-key" >> .env
```

3. **Start all services:**
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
```

### 🛠️ Local Development Setup

1. **Set up the database:**
```bash
# Start PostgreSQL and Redis
docker-compose up -d postgres redis

# Run database migrations
cargo install sqlx-cli
sqlx migrate run --database-url postgresql://rwa_user:rwa_password@localhost:5432/rwa_dev
```

2. **Build and run services:**
```bash
# Build all services
cargo build --workspace

# Run individual services (in separate terminals)
cargo run --bin service-gateway
cargo run --bin service-user
cargo run --bin service-asset
cargo run --bin service-payment
cargo run --bin ai-service
```

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

### Intelligent Asset Valuation
```bash
curl -X POST http://localhost:8090/ai-asset-valuation \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_type": "real_estate",
    "location": "Manhattan, NYC",
    "area": 1200,
    "year_built": 2020
  }'
```

### Risk Assessment
```bash
curl -X POST http://localhost:8090/ai-risk-assessment \
  -H 'Content-Type: application/json' \
  -d '{
    "asset_id": "asset-123",
    "investment_amount": 1000000,
    "time_horizon": "1 year"
  }'
```

### Market Analysis
```bash
curl -X POST http://localhost:8090/ai-market-analysis \
  -H 'Content-Type: application/json' \
  -d '{
    "market": "real_estate",
    "region": "tier_1_cities",
    "analysis_type": "trend_prediction"
  }'
```

## 🔧 Secondary Development Guide

### Adding Custom AI Modules

1. **Create AI Plugin:**
```rust
use stablerwa_framework::core_ai::{AIPlugin, AIRequest, AIResponse};

pub struct CustomAIPlugin;

impl AIPlugin for CustomAIPlugin {
    async fn process(&self, request: AIRequest) -> Result<AIResponse, AIError> {
        // Implement custom AI logic
        Ok(AIResponse::new("Custom AI response"))
    }
}
```

2. **Register Plugin:**
```rust
let ai_service = AIService::builder()
    .add_plugin(Box::new(CustomAIPlugin))
    .build();
```

### Extending Blockchain Support

1. **Implement Blockchain Adapter:**
```rust
use stablerwa_framework::core_blockchain::{BlockchainAdapter, ChainConfig};

pub struct CustomChainAdapter;

impl BlockchainAdapter for CustomChainAdapter {
    async fn deploy_contract(&self, contract: &Contract) -> Result<String, BlockchainError> {
        // Implement custom blockchain deployment logic
    }
}
```

### Adding Custom Services

1. **Create New Microservice:**
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
    
    // Start custom service
    framework.run_custom_service(MyCustomService).await
}
```

## 📚 Documentation

### API Documentation
- **OpenAPI Spec**: Available at `/docs` when services are running
- **Postman Collection**: `docs/api/StableRWA.postman_collection.json`
- **API Examples**: `docs/api/examples/`
- **AI API Documentation**: `docs/api/ai-endpoints.md`

### Architecture Documentation
- **System Design**: `docs/architecture/system-design.md`
- **AI Architecture**: `docs/architecture/ai-architecture.md`
- **Database Schema**: `docs/architecture/database-schema.md`
- **Event Flow**: `docs/architecture/event-flow.md`
- **Security Model**: `docs/architecture/security.md`

### Development Documentation
- **Secondary Development Guide**: `docs/development/secondary-development.md`
- **AI Integration Guide**: `docs/development/ai-integration.md`
- **Plugin Development**: `docs/development/plugin-development.md`
- **Custom Services**: `docs/development/custom-services.md`

## 🧪 Testing

### Running Tests

```bash
# Unit tests
cargo test --workspace --lib

# Integration tests
cargo test --workspace --test '*'

# AI feature tests
cargo test --package ai-service

# Load tests (requires k6)
k6 run tests/load/api-load-test.js

# Smoke tests
./scripts/smoke-tests.sh development
```

## 🚢 Deployment

### Production Deployment

```bash
# Deploy to Kubernetes
./scripts/deploy.sh -e production -v v1.0.0 deploy

# Deploy to staging
./scripts/deploy.sh -e staging -v develop deploy
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

## 🤝 Supporting Organizations

StableRWA is proudly supported by leading technology companies:

### 🏢 Corporate Partners
- **Tongda Ark Future Network Technology Co., Ltd. (Hong Kong)**
  *同大方舟未来网络科技有限公司（香港）*
- **ARKMETA CRYPTO NETWORK LIMITED**
- **Longan Wisdom (Shanghai) Network Technology Co., Ltd.**
  *龙眼慧（上海）网络科技有限公司*

These organizations provide strategic guidance, technical expertise, and resources to advance the development of enterprise-grade Web3 RWA solutions.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- **Documentation**: [docs.stablerwa.com](https://docs.stablerwa.com)
- **Issues**: [GitHub Issues](https://github.com/arkSong/StableRWA/issues)
- **Discussions**: [GitHub Discussions](https://github.com/arkSong/StableRWA/discussions)
- **Email**: arksong2018@gmail.com

---

**Built with ❤️ by the StableRWA Team**

## Contact
- **Author:** arkSong (arksong2018@gmail.com)
- For questions, suggestions, or contributions, please contact the author.

---

**StableRWA - AI-Powered Future of Asset Tokenization Platform** 🚀🤖
