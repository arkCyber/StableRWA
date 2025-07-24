# =====================================================================================
# Makefile for Asset Service
# Enterprise-grade build, test, and deployment automation
# =====================================================================================

# Variables
CARGO = cargo
DOCKER = docker
DOCKER_COMPOSE = docker-compose
SERVICE_NAME = asset-service
IMAGE_NAME = $(SERVICE_NAME)
VERSION ?= latest
REGISTRY ?= localhost:5000

# Environment
ENV ?= development
CONFIG_FILE = .env

# Database
DB_NAME = asset_db
DB_USER = asset_user
DB_PASSWORD = asset_password
DB_HOST = localhost
DB_PORT = 5432
DATABASE_URL = postgresql://$(DB_USER):$(DB_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(DB_NAME)

# Test Database
TEST_DB_NAME = test_asset_db
TEST_DB_USER = test_user
TEST_DB_PASSWORD = test_password
TEST_DATABASE_URL = postgresql://$(TEST_DB_USER):$(TEST_DB_PASSWORD)@$(DB_HOST):$(DB_PORT)/$(TEST_DB_NAME)

# Colors for output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[0;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

.PHONY: help build test clean install dev docker lint format check security audit migrate setup

# Default target
all: build

## Help
help: ## Show this help message
	@echo "$(BLUE)Asset Service - Available Commands$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Usage: make $(BLUE)<target>$(NC)\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

## Development
dev: ## Start development server with hot reload
	@echo "$(BLUE)Starting development server...$(NC)"
	$(CARGO) watch -x run

install: ## Install dependencies
	@echo "$(BLUE)Installing dependencies...$(NC)"
	$(CARGO) fetch

setup: ## Setup development environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@if [ ! -f $(CONFIG_FILE) ]; then \
		echo "$(YELLOW)Creating .env file from template...$(NC)"; \
		cp .env.example .env; \
	fi
	@echo "$(GREEN)Environment setup complete!$(NC)"
	@echo "$(YELLOW)Please update .env file with your configuration$(NC)"

## Building
build: ## Build the application
	@echo "$(BLUE)Building application...$(NC)"
	$(CARGO) build

build-release: ## Build optimized release version
	@echo "$(BLUE)Building release version...$(NC)"
	$(CARGO) build --release

## Testing
test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	$(CARGO) test

test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(NC)"
	$(CARGO) test --lib

test-integration: ## Run integration tests only
	@echo "$(BLUE)Running integration tests...$(NC)"
	$(CARGO) test --test integration_tests

test-coverage: ## Generate test coverage report
	@echo "$(BLUE)Generating test coverage...$(NC)"
	$(CARGO) tarpaulin --out Html --output-dir coverage

test-load: ## Run load tests
	@echo "$(BLUE)Running load tests...$(NC)"
	$(CARGO) test --test load_test --release

## Code Quality
lint: ## Run linter
	@echo "$(BLUE)Running linter...$(NC)"
	$(CARGO) clippy -- -D warnings

format: ## Format code
	@echo "$(BLUE)Formatting code...$(NC)"
	$(CARGO) fmt

format-check: ## Check code formatting
	@echo "$(BLUE)Checking code formatting...$(NC)"
	$(CARGO) fmt -- --check

check: ## Run all checks (format, lint, test)
	@echo "$(BLUE)Running all checks...$(NC)"
	@$(MAKE) format-check
	@$(MAKE) lint
	@$(MAKE) test

## Security
security: ## Run security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	$(CARGO) audit

audit: security ## Alias for security

## Database
db-setup: ## Setup database
	@echo "$(BLUE)Setting up database...$(NC)"
	@createdb -h $(DB_HOST) -p $(DB_PORT) -U postgres $(DB_NAME) || true
	@createuser -h $(DB_HOST) -p $(DB_PORT) -U postgres $(DB_USER) || true
	@psql -h $(DB_HOST) -p $(DB_PORT) -U postgres -c "ALTER USER $(DB_USER) PASSWORD '$(DB_PASSWORD)';" || true
	@psql -h $(DB_HOST) -p $(DB_PORT) -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE $(DB_NAME) TO $(DB_USER);" || true

db-migrate: ## Run database migrations
	@echo "$(BLUE)Running database migrations...$(NC)"
	sqlx migrate run --database-url $(DATABASE_URL)

db-migrate-revert: ## Revert last database migration
	@echo "$(BLUE)Reverting last migration...$(NC)"
	sqlx migrate revert --database-url $(DATABASE_URL)

db-reset: ## Reset database (drop and recreate)
	@echo "$(YELLOW)Resetting database...$(NC)"
	@dropdb -h $(DB_HOST) -p $(DB_PORT) -U postgres $(DB_NAME) --if-exists
	@$(MAKE) db-setup
	@$(MAKE) db-migrate

db-seed: ## Seed database with test data
	@echo "$(BLUE)Seeding database...$(NC)"
	$(CARGO) run --bin seed_database

## Test Database
test-db-setup: ## Setup test database
	@echo "$(BLUE)Setting up test database...$(NC)"
	@createdb -h $(DB_HOST) -p $(DB_PORT) -U postgres $(TEST_DB_NAME) || true
	@createuser -h $(DB_HOST) -p $(DB_PORT) -U postgres $(TEST_DB_USER) || true
	@psql -h $(DB_HOST) -p $(DB_PORT) -U postgres -c "ALTER USER $(TEST_DB_USER) PASSWORD '$(TEST_DB_PASSWORD)';" || true
	@psql -h $(DB_HOST) -p $(DB_PORT) -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE $(TEST_DB_NAME) TO $(TEST_DB_USER);" || true

test-db-migrate: ## Run test database migrations
	@echo "$(BLUE)Running test database migrations...$(NC)"
	sqlx migrate run --database-url $(TEST_DATABASE_URL)

test-db-reset: ## Reset test database
	@echo "$(YELLOW)Resetting test database...$(NC)"
	@dropdb -h $(DB_HOST) -p $(DB_PORT) -U postgres $(TEST_DB_NAME) --if-exists
	@$(MAKE) test-db-setup
	@$(MAKE) test-db-migrate

## Docker
docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	$(DOCKER) build -t $(IMAGE_NAME):$(VERSION) .

docker-build-multi: ## Build multi-architecture Docker image
	@echo "$(BLUE)Building multi-architecture Docker image...$(NC)"
	$(DOCKER) buildx build --platform linux/amd64,linux/arm64 -t $(IMAGE_NAME):$(VERSION) .

docker-run: ## Run Docker container
	@echo "$(BLUE)Running Docker container...$(NC)"
	$(DOCKER) run -p 8080:8080 -p 9090:9090 --env-file .env $(IMAGE_NAME):$(VERSION)

docker-push: ## Push Docker image to registry
	@echo "$(BLUE)Pushing Docker image to registry...$(NC)"
	$(DOCKER) tag $(IMAGE_NAME):$(VERSION) $(REGISTRY)/$(IMAGE_NAME):$(VERSION)
	$(DOCKER) push $(REGISTRY)/$(IMAGE_NAME):$(VERSION)

## Docker Compose
compose-up: ## Start all services with Docker Compose
	@echo "$(BLUE)Starting services with Docker Compose...$(NC)"
	$(DOCKER_COMPOSE) up -d

compose-down: ## Stop all services
	@echo "$(BLUE)Stopping services...$(NC)"
	$(DOCKER_COMPOSE) down

compose-logs: ## Show logs from all services
	@echo "$(BLUE)Showing logs...$(NC)"
	$(DOCKER_COMPOSE) logs -f

compose-build: ## Build all services with Docker Compose
	@echo "$(BLUE)Building services with Docker Compose...$(NC)"
	$(DOCKER_COMPOSE) build

compose-restart: ## Restart all services
	@echo "$(BLUE)Restarting services...$(NC)"
	$(DOCKER_COMPOSE) restart

## Monitoring
metrics: ## Show Prometheus metrics
	@echo "$(BLUE)Fetching metrics...$(NC)"
	@curl -s http://localhost:9090/metrics | head -20

health: ## Check service health
	@echo "$(BLUE)Checking service health...$(NC)"
	@curl -s http://localhost:8080/health | jq .

logs: ## Show application logs
	@echo "$(BLUE)Showing application logs...$(NC)"
	@tail -f logs/asset-service.log 2>/dev/null || echo "$(YELLOW)Log file not found$(NC)"

## Deployment
deploy-staging: ## Deploy to staging environment
	@echo "$(BLUE)Deploying to staging...$(NC)"
	@$(MAKE) docker-build
	@$(MAKE) docker-push
	@echo "$(GREEN)Deployment to staging complete!$(NC)"

deploy-production: ## Deploy to production environment
	@echo "$(BLUE)Deploying to production...$(NC)"
	@echo "$(YELLOW)This will deploy to production. Are you sure? [y/N]$(NC)"
	@read -r CONFIRM && [ "$$CONFIRM" = "y" ] || exit 1
	@$(MAKE) docker-build
	@$(MAKE) docker-push
	@echo "$(GREEN)Deployment to production complete!$(NC)"

## Utilities
clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	$(CARGO) clean
	$(DOCKER) system prune -f

docs: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	$(CARGO) doc --open

benchmark: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	$(CARGO) bench

profile: ## Profile the application
	@echo "$(BLUE)Profiling application...$(NC)"
	$(CARGO) build --release
	perf record --call-graph=dwarf target/release/$(SERVICE_NAME)
	perf report

## CI/CD
ci-test: ## Run CI tests
	@echo "$(BLUE)Running CI tests...$(NC)"
	@$(MAKE) format-check
	@$(MAKE) lint
	@$(MAKE) security
	@$(MAKE) test

ci-build: ## Build for CI
	@echo "$(BLUE)Building for CI...$(NC)"
	@$(MAKE) build-release

## Environment Management
env-dev: ## Switch to development environment
	@echo "$(BLUE)Switching to development environment...$(NC)"
	@cp .env.development .env 2>/dev/null || echo "$(YELLOW)Development config not found$(NC)"

env-staging: ## Switch to staging environment
	@echo "$(BLUE)Switching to staging environment...$(NC)"
	@cp .env.staging .env 2>/dev/null || echo "$(YELLOW)Staging config not found$(NC)"

env-prod: ## Switch to production environment
	@echo "$(BLUE)Switching to production environment...$(NC)"
	@cp .env.production .env 2>/dev/null || echo "$(YELLOW)Production config not found$(NC)"

## Backup and Restore
backup: ## Backup database
	@echo "$(BLUE)Backing up database...$(NC)"
	@mkdir -p backups
	@pg_dump $(DATABASE_URL) > backups/backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "$(GREEN)Database backup complete!$(NC)"

restore: ## Restore database from backup
	@echo "$(BLUE)Restoring database...$(NC)"
	@echo "$(YELLOW)Enter backup file path:$(NC)"
	@read -r BACKUP_FILE && psql $(DATABASE_URL) < "$$BACKUP_FILE"
	@echo "$(GREEN)Database restore complete!$(NC)"

## Performance Testing
perf-test: ## Run performance tests
	@echo "$(BLUE)Running performance tests...$(NC)"
	@$(MAKE) build-release
	@echo "$(YELLOW)Starting service in background...$(NC)"
	@./target/release/$(SERVICE_NAME) &
	@sleep 5
	@echo "$(BLUE)Running load test...$(NC)"
	@wrk -t12 -c400 -d30s http://localhost:8080/health
	@pkill $(SERVICE_NAME)

## Maintenance
update-deps: ## Update dependencies
	@echo "$(BLUE)Updating dependencies...$(NC)"
	$(CARGO) update

check-outdated: ## Check for outdated dependencies
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	$(CARGO) outdated

## Quick Commands
quick-test: format lint test ## Quick development cycle (format, lint, test)

quick-deploy: check docker-build docker-push ## Quick deployment (check, build, push)

## Status
status: ## Show service status
	@echo "$(BLUE)Service Status:$(NC)"
	@echo "  Build: $(shell $(CARGO) --version)"
	@echo "  Docker: $(shell $(DOCKER) --version | head -1)"
	@echo "  Database: $(shell psql $(DATABASE_URL) -c 'SELECT version();' -t 2>/dev/null | head -1 || echo 'Not connected')"
	@echo "  Service: $(shell curl -s http://localhost:8080/health | jq -r .status 2>/dev/null || echo 'Not running')"
