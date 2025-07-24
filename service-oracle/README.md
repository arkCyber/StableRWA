# RWA Tokenization Platform - Oracle Service

A high-performance, enterprise-grade price oracle service built in Rust for the RWA (Real World Assets) tokenization platform. This service provides reliable, aggregated price data from multiple sources with comprehensive monitoring, caching, and subscription capabilities.

## Features

### Core Functionality
- **Multi-Provider Price Aggregation**: Combines data from CoinGecko, Binance, CoinMarketCap, and custom providers
- **Real-time Price Feeds**: Configurable price feeds with customizable update intervals
- **Advanced Aggregation Methods**: Mean, median, weighted average, and volume-weighted price calculations
- **Outlier Detection**: Automatic detection and removal of price outliers using IQR method
- **Subscription System**: WebSocket, webhook, and SSE-based price update notifications

### Enterprise Features
- **High Availability**: Built-in health checks, circuit breakers, and failover mechanisms
- **Comprehensive Monitoring**: Prometheus metrics, structured logging, and performance tracking
- **Caching Layer**: Redis-based caching with configurable TTL and cache warming
- **Rate Limiting**: Per-provider rate limiting with exponential backoff
- **Database Partitioning**: Automatic monthly partitioning for historical price data
- **Security**: JWT authentication, API key management, and CORS support

### Reliability & Performance
- **Fault Tolerance**: Graceful degradation when providers are unavailable
- **Horizontal Scaling**: Stateless design with external state management
- **Connection Pooling**: Optimized database and Redis connection management
- **Background Processing**: Asynchronous price updates and maintenance tasks

## Quick Start

### Using Docker Compose (Recommended)

1. **Clone and setup**:
```bash
git clone <repository-url>
cd service-oracle
```

2. **Configure environment**:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. **Start services**:
```bash
docker-compose up -d
```

4. **Verify deployment**:
```bash
curl http://localhost:8081/health
```

### Manual Installation

#### Prerequisites
- Rust 1.75+
- PostgreSQL 15+
- Redis 7+

#### Build and Run

1. **Install dependencies**:
```bash
cargo build --release
```

2. **Setup database**:
```bash
# Create database and run migrations
psql -U postgres -c "CREATE DATABASE oracle_service;"
sqlx migrate run --database-url "postgresql://postgres:password@localhost/oracle_service"
```

3. **Configure environment**:
```bash
export DATABASE_URL="postgresql://postgres:password@localhost/oracle_service"
export REDIS_URL="redis://localhost:6379"
export JWT_SECRET="your-super-secret-jwt-key-min-32-chars"
```

4. **Run service**:
```bash
cargo run --release
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server bind address | `127.0.0.1` |
| `SERVER_PORT` | Server port | `8081` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | Required |
| `JWT_SECRET` | JWT signing secret (min 32 chars) | Required |
| `COINGECKO_API_KEY` | CoinGecko Pro API key | Optional |
| `COINMARKETCAP_API_KEY` | CoinMarketCap API key | Optional |
| `RUST_LOG` | Log level | `info` |

### Configuration File

Create `config/oracle.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8081
workers = 4

[database]
url = "postgresql://user:pass@localhost/oracle_service"
max_connections = 10

[redis]
url = "redis://localhost:6379"
max_connections = 10

[providers.coingecko]
enabled = true
api_key = "your-api-key"
weight = 1.00
rate_limit_per_minute = 50

[aggregation]
default_method = "weighted_average"
min_sources = 2
max_deviation_percent = 10
confidence_threshold = 0.80
```

## API Documentation

### Health Endpoints

- `GET /health` - Comprehensive health check
- `GET /health/ready` - Readiness probe
- `GET /health/live` - Liveness probe
- `GET /metrics` - Prometheus metrics

### Price Endpoints

- `GET /api/v1/prices/{asset_id}?currency=USD` - Get current price
- `POST /api/v1/prices/batch` - Get multiple prices
- `GET /api/v1/prices/{asset_id}/history` - Get price history

### Feed Management

- `GET /api/v1/feeds` - List price feeds
- `POST /api/v1/feeds` - Create price feed
- `GET /api/v1/feeds/{feed_id}` - Get price feed
- `PUT /api/v1/feeds/{feed_id}` - Update price feed
- `DELETE /api/v1/feeds/{feed_id}` - Delete price feed

### Subscriptions

- `POST /api/v1/feeds/{feed_id}/subscribe` - Subscribe to feed
- `DELETE /api/v1/subscriptions/{subscription_id}` - Unsubscribe

### Example Requests

#### Get Bitcoin Price
```bash
curl "http://localhost:8081/api/v1/prices/BTC?currency=USD"
```

#### Create Price Feed
```bash
curl -X POST "http://localhost:8081/api/v1/feeds" \
  -H "Content-Type: application/json" \
  -d '{
    "asset_id": "BTC",
    "name": "Bitcoin USD Feed",
    "currency": "USD",
    "update_interval": 60,
    "providers": ["CoinGecko", "Binance"],
    "aggregation_method": "weighted_average",
    "deviation_threshold": 5.0
  }'
```

#### Batch Price Request
```bash
curl -X POST "http://localhost:8081/api/v1/prices/batch" \
  -H "Content-Type: application/json" \
  -d '{
    "asset_ids": ["BTC", "ETH", "ADA"],
    "currency": "USD",
    "include_metadata": true
  }'
```

## Monitoring

### Metrics

The service exposes Prometheus metrics at `/metrics`:

- Request metrics (count, duration, status codes)
- Price fetch metrics (success rate, response times)
- Provider health metrics
- Database connection metrics
- Cache performance metrics
- Business metrics (feeds, subscriptions)

### Logging

Structured JSON logging with configurable levels:

```bash
RUST_LOG=service_oracle=debug,actix_web=info cargo run
```

### Grafana Dashboard

Import the provided Grafana dashboard from `monitoring/grafana/dashboards/oracle-service.json`.

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires test database)
TEST_DATABASE_URL="postgresql://postgres:postgres@localhost/oracle_test" \
TEST_REDIS_URL="redis://localhost:6379/1" \
cargo test --test integration_tests

# Load tests
cargo install cargo-criterion
cargo criterion
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Security audit
cargo audit

# Check dependencies
cargo outdated
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert migration
sqlx migrate revert
```

## Deployment

### Production Checklist

- [ ] Set strong JWT secret (min 32 characters)
- [ ] Configure provider API keys
- [ ] Set up SSL/TLS termination
- [ ] Configure log aggregation
- [ ] Set up monitoring and alerting
- [ ] Configure backup strategy
- [ ] Review security settings
- [ ] Load test the deployment

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: oracle-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: oracle-service
  template:
    metadata:
      labels:
        app: oracle-service
    spec:
      containers:
      - name: oracle-service
        image: rwa-platform/oracle-service:latest
        ports:
        - containerPort: 8081
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: oracle-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: oracle-secrets
              key: redis-url
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8081
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8081
          initialDelaySeconds: 5
          periodSeconds: 5
```

## Architecture

### System Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Load Balancer │    │   API Gateway   │    │   Monitoring    │
│     (Nginx)     │    │                 │    │  (Prometheus)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
         ┌─────────────────────────────────────────────────────┐
         │                Oracle Service                        │
         │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
         │  │   Handlers  │  │ Aggregator  │  │   Metrics   │  │
         │  └─────────────┘  └─────────────┘  └─────────────┘  │
         │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
         │  │  Providers  │  │   Service   │  │    Cache    │  │
         │  └─────────────┘  └─────────────┘  └─────────────┘  │
         └─────────────────────────────────────────────────────┘
                                 │
         ┌─────────────────────────────────────────────────────┐
         │                 Data Layer                          │
         │  ┌─────────────┐              ┌─────────────┐       │
         │  │ PostgreSQL  │              │    Redis    │       │
         │  │ (Persistent)│              │   (Cache)   │       │
         │  └─────────────┘              └─────────────┘       │
         └─────────────────────────────────────────────────────┘
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
- Create an issue on GitHub
- Contact: arksong2018@gmail.com

---

**Author**: arkSong (arksong2018@gmail.com)  
**Version**: 1.0.0  
**Last Updated**: 2024-01-20
