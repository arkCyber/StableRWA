# =====================================================================================
# RWA Tokenization Platform - Oracle Service Makefile
# 
# Author: arkSong (arksong2018@gmail.com)
# =====================================================================================

# Variables
RUST_VERSION := 1.75
PROJECT_NAME := service-oracle
DOCKER_IMAGE := rwa-platform/oracle-service
DOCKER_TAG := latest
DATABASE_URL := postgresql://postgres:postgres@localhost:5432/oracle_service
TEST_DATABASE_URL := postgresql://postgres:postgres@localhost:5432/oracle_test
REDIS_URL := redis://localhost:6379

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[0;33m
BLUE := \033[0;34m
NC := \033[0m # No Color

.PHONY: help build test clean docker dev setup lint format audit deps check-deps

# Default target
help: ## Show this help message
	@echo "$(BLUE)RWA Oracle Service - Available Commands$(NC)"
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development commands
dev: ## Start development server with hot reload
	@echo "$(BLUE)Starting development server...$(NC)"
	cargo watch -x 'run'

build: ## Build the project in release mode
	@echo "$(BLUE)Building project...$(NC)"
	cargo build --release

build-debug: ## Build the project in debug mode
	@echo "$(BLUE)Building project (debug)...$(NC)"
	cargo build

clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	docker system prune -f

# Testing commands
test: ## Run all tests
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test

test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(NC)"
	cargo test --lib

test-integration: ## Run integration tests
	@echo "$(BLUE)Running integration tests...$(NC)"
	TEST_DATABASE_URL=$(TEST_DATABASE_URL) \
	TEST_REDIS_URL=$(REDIS_URL)/1 \
	cargo test --test integration_tests

test-coverage: ## Generate test coverage report
	@echo "$(BLUE)Generating test coverage...$(NC)"
	cargo tarpaulin --out Html --output-dir coverage

benchmark: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench

# Code quality commands
lint: ## Run clippy linter
	@echo "$(BLUE)Running clippy...$(NC)"
	cargo clippy -- -D warnings

format: ## Format code with rustfmt
	@echo "$(BLUE)Formatting code...$(NC)"
	cargo fmt

format-check: ## Check code formatting
	@echo "$(BLUE)Checking code formatting...$(NC)"
	cargo fmt -- --check

audit: ## Run security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	cargo audit

check-deps: ## Check for outdated dependencies
	@echo "$(BLUE)Checking dependencies...$(NC)"
	cargo outdated

# Database commands
db-setup: ## Setup database and run migrations
	@echo "$(BLUE)Setting up database...$(NC)"
	sqlx database create --database-url $(DATABASE_URL) || true
	sqlx migrate run --database-url $(DATABASE_URL)

db-reset: ## Reset database (drop and recreate)
	@echo "$(YELLOW)Resetting database...$(NC)"
	sqlx database drop --database-url $(DATABASE_URL) -y || true
	sqlx database create --database-url $(DATABASE_URL)
	sqlx migrate run --database-url $(DATABASE_URL)

db-migrate: ## Run database migrations
	@echo "$(BLUE)Running migrations...$(NC)"
	sqlx migrate run --database-url $(DATABASE_URL)

db-rollback: ## Rollback last migration
	@echo "$(YELLOW)Rolling back migration...$(NC)"
	sqlx migrate revert --database-url $(DATABASE_URL)

db-status: ## Show migration status
	@echo "$(BLUE)Migration status:$(NC)"
	sqlx migrate info --database-url $(DATABASE_URL)

# Test database commands
test-db-setup: ## Setup test database
	@echo "$(BLUE)Setting up test database...$(NC)"
	sqlx database create --database-url $(TEST_DATABASE_URL) || true
	sqlx migrate run --database-url $(TEST_DATABASE_URL)

test-db-reset: ## Reset test database
	@echo "$(YELLOW)Resetting test database...$(NC)"
	sqlx database drop --database-url $(TEST_DATABASE_URL) -y || true
	sqlx database create --database-url $(TEST_DATABASE_URL)
	sqlx migrate run --database-url $(TEST_DATABASE_URL)

# Docker commands
docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

docker-run: ## Run Docker container
	@echo "$(BLUE)Running Docker container...$(NC)"
	docker run -p 8081:8081 \
		-e DATABASE_URL=$(DATABASE_URL) \
		-e REDIS_URL=$(REDIS_URL) \
		-e JWT_SECRET=development-secret-key-min-32-chars \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

docker-compose-up: ## Start all services with docker-compose
	@echo "$(BLUE)Starting services with docker-compose...$(NC)"
	docker-compose up -d

docker-compose-down: ## Stop all services
	@echo "$(BLUE)Stopping services...$(NC)"
	docker-compose down

docker-compose-logs: ## Show docker-compose logs
	@echo "$(BLUE)Showing logs...$(NC)"
	docker-compose logs -f

docker-compose-build: ## Build and start services
	@echo "$(BLUE)Building and starting services...$(NC)"
	docker-compose up -d --build

# Monitoring commands
monitoring-up: ## Start monitoring stack (Prometheus, Grafana)
	@echo "$(BLUE)Starting monitoring stack...$(NC)"
	docker-compose --profile monitoring up -d

monitoring-down: ## Stop monitoring stack
	@echo "$(BLUE)Stopping monitoring stack...$(NC)"
	docker-compose --profile monitoring down

# Installation and setup commands
install-deps: ## Install development dependencies
	@echo "$(BLUE)Installing development dependencies...$(NC)"
	cargo install cargo-watch
	cargo install cargo-tarpaulin
	cargo install cargo-audit
	cargo install cargo-outdated
	cargo install sqlx-cli --features postgres

setup: install-deps db-setup test-db-setup ## Complete development setup
	@echo "$(GREEN)Development environment setup complete!$(NC)"

# Production commands
prod-build: ## Build for production
	@echo "$(BLUE)Building for production...$(NC)"
	cargo build --release --locked

prod-test: ## Run production tests
	@echo "$(BLUE)Running production tests...$(NC)"
	cargo test --release

prod-docker: ## Build production Docker image
	@echo "$(BLUE)Building production Docker image...$(NC)"
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) --target production .

# Health check commands
health-check: ## Check service health
	@echo "$(BLUE)Checking service health...$(NC)"
	curl -f http://localhost:8081/health || echo "$(RED)Service is not healthy$(NC)"

metrics: ## Show service metrics
	@echo "$(BLUE)Fetching service metrics...$(NC)"
	curl -s http://localhost:8081/metrics | head -20

# Load testing commands
load-test: ## Run load tests (requires wrk)
	@echo "$(BLUE)Running load tests...$(NC)"
	wrk -t12 -c400 -d30s http://localhost:8081/health

load-test-prices: ## Load test price endpoints
	@echo "$(BLUE)Load testing price endpoints...$(NC)"
	wrk -t8 -c200 -d30s http://localhost:8081/api/v1/prices/BTC

# Utility commands
logs: ## Show application logs
	@echo "$(BLUE)Showing logs...$(NC)"
	tail -f logs/oracle-service.log 2>/dev/null || echo "$(YELLOW)No log file found$(NC)"

ps: ## Show running processes
	@echo "$(BLUE)Oracle service processes:$(NC)"
	ps aux | grep oracle-service | grep -v grep || echo "$(YELLOW)No processes found$(NC)"

env-check: ## Check environment variables
	@echo "$(BLUE)Environment check:$(NC)"
	@echo "DATABASE_URL: $${DATABASE_URL:-$(RED)Not set$(NC)}"
	@echo "REDIS_URL: $${REDIS_URL:-$(RED)Not set$(NC)}"
	@echo "JWT_SECRET: $${JWT_SECRET:-$(RED)Not set$(NC)}"
	@echo "RUST_LOG: $${RUST_LOG:-$(YELLOW)Not set (default: info)$(NC)}"

# CI/CD commands
ci-test: format-check lint test ## Run CI tests
	@echo "$(GREEN)All CI checks passed!$(NC)"

ci-build: prod-build prod-test ## CI build process
	@echo "$(GREEN)CI build completed!$(NC)"

# Documentation commands
docs: ## Generate and open documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	cargo doc --open

docs-build: ## Build documentation
	@echo "$(BLUE)Building documentation...$(NC)"
	cargo doc --no-deps

# Release commands
release-patch: ## Create patch release
	@echo "$(BLUE)Creating patch release...$(NC)"
	cargo release patch --execute

release-minor: ## Create minor release
	@echo "$(BLUE)Creating minor release...$(NC)"
	cargo release minor --execute

release-major: ## Create major release
	@echo "$(BLUE)Creating major release...$(NC)"
	cargo release major --execute

# Cleanup commands
clean-all: clean ## Clean everything including Docker
	@echo "$(BLUE)Cleaning everything...$(NC)"
	docker-compose down -v
	docker system prune -af
	cargo clean

# Quick commands for common workflows
quick-test: format lint test ## Quick development test cycle
	@echo "$(GREEN)Quick test cycle completed!$(NC)"

quick-deploy: docker-compose-build health-check ## Quick deployment
	@echo "$(GREEN)Quick deployment completed!$(NC)"

# Show project status
status: ## Show project status
	@echo "$(BLUE)Project Status:$(NC)"
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"
	@echo "Project: $(PROJECT_NAME)"
	@echo "Docker image: $(DOCKER_IMAGE):$(DOCKER_TAG)"
	@echo ""
	@make env-check
