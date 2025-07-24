# Asset Service

Enterprise-grade asset management microservice for Real World Asset (RWA) tokenization platform.

## Overview

The Asset Service is a production-ready microservice built with Rust and Actix-web that provides comprehensive asset management capabilities including:

- **Asset Lifecycle Management**: Create, read, update, delete assets
- **Asset Tokenization**: Convert real-world assets into blockchain tokens
- **Valuation Management**: Track and manage asset valuations over time
- **Metadata Management**: Store and retrieve asset metadata
- **Enterprise Features**: Caching, monitoring, health checks, rate limiting

## Features

### Core Functionality
- ✅ RESTful API for asset management
- ✅ Asset tokenization on multiple blockchain networks
- ✅ Real-time asset valuation tracking
- ✅ Comprehensive metadata management
- ✅ Multi-tenant support with user isolation

### Enterprise Features
- ✅ **Monitoring & Metrics**: Prometheus metrics, health checks
- ✅ **Caching**: Redis and in-memory caching with compression
- ✅ **Security**: JWT authentication, rate limiting, security headers
- ✅ **Observability**: Structured logging, distributed tracing
- ✅ **High Availability**: Database connection pooling, graceful shutdown
- ✅ **Configuration**: Environment-based configuration management

### Production Ready
- ✅ **Docker Support**: Multi-stage builds, non-root user
- ✅ **Database Migrations**: Automated schema management
- ✅ **Error Handling**: Comprehensive error types and handling
- ✅ **Testing**: Unit tests, integration tests, load testing utilities
- ✅ **Documentation**: OpenAPI/Swagger documentation

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │    │     Gateway     │    │   Asset Service │
│     (Nginx)     │────│   (API Gateway) │────│   (This Service)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                        │
                       ┌─────────────────┐             │
                       │     Cache       │             │
                       │    (Redis)      │◄────────────┤
                       └─────────────────┘             │
                                                        │
                       ┌─────────────────┐             │
                       │    Database     │             │
                       │  (PostgreSQL)   │◄────────────┤
                       └─────────────────┘             │
                                                        │
                       ┌─────────────────┐             │
                       │   Blockchain    │             │
                       │   (Ethereum)    │◄────────────┘
                       └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 13+
- Redis 6+ (optional, for caching)
- Docker & Docker Compose (for containerized deployment)

### Local Development

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd service-asset
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Start dependencies**
   ```bash
   docker-compose up -d postgres redis
   ```

4. **Run database migrations**
   ```bash
   sqlx migrate run
   ```

5. **Start the service**
   ```bash
   cargo run
   ```

The service will be available at `http://localhost:8080`

### Docker Deployment

1. **Build and start all services**
   ```bash
   docker-compose up -d
   ```

2. **Check service health**
   ```bash
   curl http://localhost:8080/health
   ```

## API Documentation

### Base URL
- Development: `http://localhost:8080/api/v1`
- Production: `https://your-domain.com/api/v1`

### Authentication
All API endpoints (except health checks) require JWT authentication:
```
Authorization: Bearer <jwt-token>
```

### Core Endpoints

#### Assets
- `GET /assets` - List assets with pagination and filtering
- `POST /assets` - Create a new asset
- `GET /assets/{id}` - Get asset by ID
- `PUT /assets/{id}` - Update asset
- `DELETE /assets/{id}` - Delete asset
- `POST /assets/{id}/tokenize` - Tokenize an asset

#### Valuations
- `GET /assets/{id}/valuations` - Get asset valuations
- `POST /assets/{id}/valuations` - Add new valuation
- `GET /valuations/{id}` - Get specific valuation
- `PUT /valuations/{id}` - Update valuation
- `DELETE /valuations/{id}` - Delete valuation

#### Health & Monitoring
- `GET /health` - Comprehensive health check
- `GET /ready` - Readiness probe (Kubernetes)
- `GET /live` - Liveness probe (Kubernetes)
- `GET /metrics` - Prometheus metrics

### Example Requests

#### Create Asset
```bash
curl -X POST http://localhost:8080/api/v1/assets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "name": "Downtown Office Building",
    "description": "Prime commercial real estate",
    "asset_type": "RealEstate",
    "total_value": "5000000.00",
    "metadata": {
      "location": "123 Business St, City, State",
      "size": "50000 sq ft",
      "year_built": 2020
    }
  }'
```

#### Tokenize Asset
```bash
curl -X POST http://localhost:8080/api/v1/assets/{id}/tokenize \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "blockchain_network": "ethereum",
    "token_supply": 1000000,
    "token_symbol": "BLDG001"
  }'
```

## Configuration

The service uses environment-based configuration. Key settings:

### Server Configuration
- `HOST` - Server bind address (default: 0.0.0.0)
- `PORT` - Server port (default: 8080)
- `WORKERS` - Number of worker threads

### Database Configuration
- `DATABASE_URL` - PostgreSQL connection string
- `DATABASE_MAX_CONNECTIONS` - Connection pool size

### Cache Configuration
- `REDIS_URL` - Redis connection string (optional)
- `CACHE_TTL_SECONDS` - Default cache TTL

### Security Configuration
- `JWT_SECRET` - JWT signing secret (required)
- `ENCRYPTION_KEY` - Data encryption key

### Blockchain Configuration
- `ETHEREUM_RPC_URL` - Ethereum mainnet RPC endpoint
- `ETHEREUM_TESTNET_RPC_URL` - Ethereum testnet RPC endpoint

## Monitoring

### Metrics
The service exposes Prometheus metrics at `/metrics`:
- HTTP request metrics (duration, count, status)
- Database query metrics
- Cache hit/miss rates
- Business metrics (assets created, tokenized, etc.)
- System metrics (memory usage, connection pools)

### Health Checks
- **Liveness**: `/live` - Basic service availability
- **Readiness**: `/ready` - Service ready to handle requests
- **Health**: `/health` - Comprehensive health status

### Logging
Structured JSON logging with configurable levels:
```bash
RUST_LOG=debug cargo run
```

## Development

### Running Tests
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Load tests
cargo test --test load_test --release
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Security audit
cargo audit
```

### Database Migrations
```bash
# Create new migration
sqlx migrate add create_assets_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Deployment

### Production Checklist
- [ ] Configure proper JWT secrets
- [ ] Set up SSL/TLS certificates
- [ ] Configure database connection pooling
- [ ] Set up monitoring and alerting
- [ ] Configure log aggregation
- [ ] Set up backup procedures
- [ ] Configure rate limiting
- [ ] Set up health check endpoints

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: asset-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: asset-service
  template:
    metadata:
      labels:
        app: asset-service
    spec:
      containers:
      - name: asset-service
        image: asset-service:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: asset-service-secrets
              key: database-url
        livenessProbe:
          httpGet:
            path: /live
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:
- Create an issue in the repository
- Contact: arksong2018@gmail.com

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.
