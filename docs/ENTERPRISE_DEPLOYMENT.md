# StableRWA Enterprise Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying the StableRWA platform in enterprise environments using Docker containers, monitoring, and production-grade configurations.

## Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+ recommended), macOS, or Windows with WSL2
- **CPU**: Minimum 8 cores, Recommended 16+ cores
- **Memory**: Minimum 16GB RAM, Recommended 32GB+ RAM
- **Storage**: Minimum 100GB SSD, Recommended 500GB+ NVMe SSD
- **Network**: Stable internet connection with sufficient bandwidth

### Software Dependencies

- Docker Engine 20.10+
- Docker Compose 2.0+
- Git 2.30+
- curl/wget for health checks
- netcat (nc) for port testing

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/arkCyber/StableRWA.git
cd StableRWA
```

### 2. Environment Setup

```bash
# Copy environment template
cp .env.example .env

# Edit environment variables
nano .env
```

### 3. Basic Deployment

```bash
# Start basic infrastructure
docker-compose -f docker-compose.basic.yml up -d

# Run basic tests
./scripts/basic_docker_test.sh
```

### 4. Enterprise Deployment

```bash
# Start full enterprise stack
docker-compose -f docker-compose.enterprise.yml up -d

# Run enterprise tests
./scripts/enterprise_docker_test.sh
```

## Architecture Overview

### Core Services

1. **PostgreSQL Database** - Primary data storage
2. **Redis Cache** - Session and cache management
3. **Ganache Blockchain** - Local Ethereum development network
4. **MinIO Storage** - S3-compatible object storage

### Monitoring Stack

1. **Prometheus** - Metrics collection and alerting
2. **Grafana** - Visualization and dashboards
3. **Jaeger** - Distributed tracing
4. **NGINX** - Load balancing and reverse proxy

### Application Services

1. **Gateway Service** - API gateway and routing
2. **Asset Service** - RWA tokenization and management
3. **Oracle Service** - Price feeds and external data
4. **AI Service** - Machine learning and analytics

## Configuration

### Environment Variables

```bash
# Database Configuration
POSTGRES_DB=stablerwa
POSTGRES_USER=stablerwa
POSTGRES_PASSWORD=StableRWA2024!

# Redis Configuration
REDIS_PASSWORD=StableRWA2024!

# Blockchain Configuration
GANACHE_NETWORK_ID=1337
GANACHE_CHAIN_ID=1337

# MinIO Configuration
MINIO_ROOT_USER=stablerwa
MINIO_ROOT_PASSWORD=StableRWA2024!

# Monitoring Configuration
GRAFANA_ADMIN_PASSWORD=StableRWA2024!
```

### Security Configuration

#### SSL/TLS Setup

```bash
# Generate SSL certificates
mkdir -p config/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout config/ssl/key.pem \
  -out config/ssl/cert.pem
```

#### Firewall Configuration

```bash
# Allow required ports
sudo ufw allow 80/tcp    # HTTP
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 5432/tcp  # PostgreSQL
sudo ufw allow 6379/tcp  # Redis
sudo ufw allow 8545/tcp  # Ganache
sudo ufw allow 9000/tcp  # MinIO
sudo ufw allow 9090/tcp  # Prometheus
sudo ufw allow 3000/tcp  # Grafana
```

## Deployment Strategies

### Development Environment

```bash
# Start development stack
docker-compose -f docker-compose.dev.yml up -d

# Enable hot reloading
export RUST_ENV=development
export LOG_LEVEL=debug
```

### Staging Environment

```bash
# Start staging stack
docker-compose -f docker-compose.staging.yml up -d

# Run integration tests
./scripts/integration_test.sh
```

### Production Environment

```bash
# Start production stack
docker-compose -f docker-compose.prod.yml up -d

# Run production health checks
./scripts/production_health_check.sh
```

## Monitoring and Observability

### Prometheus Metrics

Access Prometheus at: `http://localhost:9090`

Key metrics to monitor:
- CPU and memory usage
- Database connection pools
- API response times
- Error rates
- Blockchain transaction status

### Grafana Dashboards

Access Grafana at: `http://localhost:3000`
- Username: `admin`
- Password: `StableRWA2024!`

Pre-configured dashboards:
- System Overview
- Database Performance
- API Metrics
- Blockchain Monitoring

### Jaeger Tracing

Access Jaeger at: `http://localhost:16686`

Trace distributed requests across services to identify performance bottlenecks.

## Backup and Recovery

### Database Backup

```bash
# Create database backup
docker exec stablerwa-postgres pg_dump -U stablerwa stablerwa > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore database backup
docker exec -i stablerwa-postgres psql -U stablerwa stablerwa < backup_file.sql
```

### Redis Backup

```bash
# Create Redis backup
docker exec stablerwa-redis redis-cli -a StableRWA2024! BGSAVE

# Copy backup file
docker cp stablerwa-redis:/data/dump.rdb ./redis_backup_$(date +%Y%m%d_%H%M%S).rdb
```

### MinIO Backup

```bash
# Sync MinIO data
mc mirror stablerwa-minio/bucket ./minio_backup_$(date +%Y%m%d_%H%M%S)/
```

## Scaling and Performance

### Horizontal Scaling

```bash
# Scale specific services
docker-compose -f docker-compose.enterprise.yml up -d --scale asset-service=3
docker-compose -f docker-compose.enterprise.yml up -d --scale oracle-service=2
```

### Load Balancing

Configure NGINX for load balancing:

```nginx
upstream asset_service {
    server asset-service-1:8080;
    server asset-service-2:8080;
    server asset-service-3:8080;
}
```

### Database Optimization

```sql
-- Create indexes for better performance
CREATE INDEX idx_assets_owner ON assets(owner_address);
CREATE INDEX idx_transactions_timestamp ON transactions(created_at);
CREATE INDEX idx_prices_symbol ON price_feeds(symbol);
```

## Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check port usage
   netstat -tulpn | grep :5432
   
   # Stop conflicting services
   sudo systemctl stop postgresql
   ```

2. **Memory Issues**
   ```bash
   # Check memory usage
   docker stats
   
   # Increase memory limits in docker-compose.yml
   mem_limit: 2g
   ```

3. **Network Issues**
   ```bash
   # Check network connectivity
   docker network ls
   docker network inspect stablerwa-network
   ```

### Log Analysis

```bash
# View service logs
docker-compose logs -f postgres
docker-compose logs -f redis
docker-compose logs -f asset-service

# View system logs
journalctl -u docker.service
```

## Security Best Practices

### Container Security

1. Use non-root users in containers
2. Scan images for vulnerabilities
3. Keep base images updated
4. Use secrets management

### Network Security

1. Use private networks for internal communication
2. Implement proper firewall rules
3. Enable SSL/TLS for all external connections
4. Use VPN for remote access

### Data Security

1. Encrypt data at rest
2. Use strong passwords and rotate regularly
3. Implement proper access controls
4. Regular security audits

## Maintenance

### Regular Tasks

1. **Daily**
   - Check service health
   - Monitor resource usage
   - Review error logs

2. **Weekly**
   - Update security patches
   - Backup databases
   - Performance analysis

3. **Monthly**
   - Security audit
   - Capacity planning
   - Disaster recovery testing

### Updates and Upgrades

```bash
# Update Docker images
docker-compose pull

# Restart services with new images
docker-compose up -d

# Clean up old images
docker image prune -f
```

## Support and Documentation

- **GitHub Repository**: https://github.com/arkCyber/StableRWA
- **Documentation**: https://stablerwa.docs.com
- **Support Email**: support@stablerwa.com
- **Community Discord**: https://discord.gg/stablerwa

## License

This project is licensed under the MIT License. See LICENSE file for details.
