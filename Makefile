# =====================================================================================
# File: Makefile
# Description: Enterprise-grade build and test automation for StableRWA Platform
# Author: arkSong (arksong2018@gmail.com)
# Framework: StableRWA - Enterprise RWA Tokenization Technology Framework Platform
# =====================================================================================

# Project configuration
PROJECT_NAME := stablerwa
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
RUST_VERSION := $(shell rustc --version | cut -d' ' -f2)
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")

# Directories
SRC_DIR := .
TARGET_DIR := target
DOCS_DIR := docs
SCRIPTS_DIR := scripts
CONFIG_DIR := config
TESTS_DIR := tests

# Build configuration
CARGO_FLAGS := --release
CARGO_TEST_FLAGS := --all-features
COVERAGE_THRESHOLD := 90
RUST_LOG := info
RUST_BACKTRACE := 1

# Docker configuration
DOCKER_REGISTRY := ghcr.io
DOCKER_NAMESPACE := arkcyber
DOCKER_IMAGE := $(DOCKER_REGISTRY)/$(DOCKER_NAMESPACE)/$(PROJECT_NAME)
DOCKER_TAG := $(VERSION)

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
PURPLE := \033[0;35m
CYAN := \033[0;36m
NC := \033[0m # No Color

# Default target
.DEFAULT_GOAL := help

# Help target
.PHONY: help
help: ## Show this help message
	@echo "$(CYAN)StableRWA Platform - Enterprise Build System$(NC)"
	@echo "$(CYAN)=============================================$(NC)"
	@echo ""
	@echo "$(YELLOW)Project Information:$(NC)"
	@echo "  Name: $(PROJECT_NAME)"
	@echo "  Version: $(VERSION)"
	@echo "  Rust Version: $(RUST_VERSION)"
	@echo "  Git Commit: $(GIT_COMMIT)"
	@echo "  Git Branch: $(GIT_BRANCH)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development targets
.PHONY: setup
setup: ## Setup development environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	@rustup update
	@rustup component add rustfmt clippy
	@cargo install cargo-audit cargo-tarpaulin cargo-nextest cargo-watch
	@cargo install cargo-machete cargo-outdated cargo-license
	@echo "$(GREEN)Development environment setup complete$(NC)"

.PHONY: deps
deps: ## Install and update dependencies
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@cargo fetch
	@cargo update
	@echo "$(GREEN)Dependencies updated$(NC)"

.PHONY: check
check: ## Check code without building
	@echo "$(BLUE)Checking code...$(NC)"
	@cargo check --workspace --all-targets --all-features
	@echo "$(GREEN)Code check complete$(NC)"

.PHONY: fmt
fmt: ## Format code
	@echo "$(BLUE)Formatting code...$(NC)"
	@cargo fmt --all
	@echo "$(GREEN)Code formatting complete$(NC)"

.PHONY: fmt-check
fmt-check: ## Check code formatting
	@echo "$(BLUE)Checking code formatting...$(NC)"
	@cargo fmt --all -- --check
	@echo "$(GREEN)Code formatting check complete$(NC)"

.PHONY: clippy
clippy: ## Run clippy lints
	@echo "$(BLUE)Running clippy lints...$(NC)"
	@cargo clippy --workspace --all-targets --all-features -- -D warnings
	@echo "$(GREEN)Clippy lints complete$(NC)"

.PHONY: audit
audit: ## Security audit
	@echo "$(BLUE)Running security audit...$(NC)"
	@cargo audit
	@echo "$(GREEN)Security audit complete$(NC)"

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@echo "$(BLUE)Checking for outdated dependencies...$(NC)"
	@cargo outdated
	@echo "$(GREEN)Outdated dependencies check complete$(NC)"

.PHONY: unused-deps
unused-deps: ## Check for unused dependencies
	@echo "$(BLUE)Checking for unused dependencies...$(NC)"
	@cargo machete
	@echo "$(GREEN)Unused dependencies check complete$(NC)"

# Build targets
.PHONY: build
build: ## Build the project
	@echo "$(BLUE)Building project...$(NC)"
	@cargo build $(CARGO_FLAGS) --workspace
	@echo "$(GREEN)Build complete$(NC)"

.PHONY: build-dev
build-dev: ## Build the project in development mode
	@echo "$(BLUE)Building project (development)...$(NC)"
	@cargo build --workspace
	@echo "$(GREEN)Development build complete$(NC)"

.PHONY: clean
clean: ## Clean build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf $(TARGET_DIR)/coverage
	@rm -rf $(TARGET_DIR)/test-results
	@rm -rf $(TARGET_DIR)/reports
	@echo "$(GREEN)Clean complete$(NC)"

# Test targets
.PHONY: test
test: ## Run all tests
	@echo "$(BLUE)Running all tests...$(NC)"
	@RUST_LOG=$(RUST_LOG) RUST_BACKTRACE=$(RUST_BACKTRACE) \
		cargo nextest run $(CARGO_TEST_FLAGS)
	@echo "$(GREEN)All tests complete$(NC)"

.PHONY: test-unit
test-unit: ## Run unit tests only
	@echo "$(BLUE)Running unit tests...$(NC)"
	@RUST_LOG=$(RUST_LOG) RUST_BACKTRACE=$(RUST_BACKTRACE) \
		cargo nextest run --lib $(CARGO_TEST_FLAGS)
	@echo "$(GREEN)Unit tests complete$(NC)"

.PHONY: test-integration
test-integration: ## Run integration tests only
	@echo "$(BLUE)Running integration tests...$(NC)"
	@RUST_LOG=$(RUST_LOG) RUST_BACKTRACE=$(RUST_BACKTRACE) \
		cargo nextest run --test '*' $(CARGO_TEST_FLAGS)
	@echo "$(GREEN)Integration tests complete$(NC)"

.PHONY: test-watch
test-watch: ## Run tests in watch mode
	@echo "$(BLUE)Running tests in watch mode...$(NC)"
	@cargo watch -x "nextest run $(CARGO_TEST_FLAGS)"

.PHONY: coverage
coverage: ## Generate test coverage report
	@echo "$(BLUE)Generating coverage report...$(NC)"
	@mkdir -p $(TARGET_DIR)/coverage
	@cargo tarpaulin --out html --output-dir $(TARGET_DIR)/coverage \
		--all-features --workspace --timeout 300 \
		--exclude-files "tests/*" "benches/*"
	@echo "$(GREEN)Coverage report generated: $(TARGET_DIR)/coverage/tarpaulin-report.html$(NC)"

.PHONY: bench
bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	@cargo bench --workspace --all-features
	@echo "$(GREEN)Benchmarks complete$(NC)"

# Enterprise test targets
.PHONY: test-enterprise
test-enterprise: ## Run enterprise test suite
	@echo "$(BLUE)Running enterprise test suite...$(NC)"
	@chmod +x $(SCRIPTS_DIR)/run_enterprise_tests.sh
	@$(SCRIPTS_DIR)/run_enterprise_tests.sh
	@echo "$(GREEN)Enterprise test suite complete$(NC)"

.PHONY: test-security
test-security: ## Run security tests only
	@echo "$(BLUE)Running security tests...$(NC)"
	@chmod +x $(SCRIPTS_DIR)/run_enterprise_tests.sh
	@$(SCRIPTS_DIR)/run_enterprise_tests.sh --security-only
	@echo "$(GREEN)Security tests complete$(NC)"

.PHONY: test-performance
test-performance: ## Run performance tests only
	@echo "$(BLUE)Running performance tests...$(NC)"
	@chmod +x $(SCRIPTS_DIR)/run_enterprise_tests.sh
	@$(SCRIPTS_DIR)/run_enterprise_tests.sh --performance-only
	@echo "$(GREEN)Performance tests complete$(NC)"

.PHONY: test-compliance
test-compliance: ## Run compliance tests only
	@echo "$(BLUE)Running compliance tests...$(NC)"
	@chmod +x $(SCRIPTS_DIR)/run_enterprise_tests.sh
	@$(SCRIPTS_DIR)/run_enterprise_tests.sh --compliance-only
	@echo "$(GREEN)Compliance tests complete$(NC)"

# Docker targets
.PHONY: docker-build
docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	@docker build -f Dockerfile.enterprise -t $(DOCKER_IMAGE):$(DOCKER_TAG) .
	@docker tag $(DOCKER_IMAGE):$(DOCKER_TAG) $(DOCKER_IMAGE):latest
	@echo "$(GREEN)Docker image built: $(DOCKER_IMAGE):$(DOCKER_TAG)$(NC)"

.PHONY: docker-push
docker-push: docker-build ## Push Docker image to registry
	@echo "$(BLUE)Pushing Docker image...$(NC)"
	@docker push $(DOCKER_IMAGE):$(DOCKER_TAG)
	@docker push $(DOCKER_IMAGE):latest
	@echo "$(GREEN)Docker image pushed$(NC)"

.PHONY: docker-run
docker-run: ## Run Docker container locally
	@echo "$(BLUE)Running Docker container...$(NC)"
	@docker run -p 8080:8080 --rm $(DOCKER_IMAGE):$(DOCKER_TAG)

.PHONY: docker-test
docker-test: ## Run tests in Docker environment
	@echo "$(BLUE)Running tests in Docker...$(NC)"
	@docker-compose -f docker-compose.test.yml up -d
	@sleep 10
	@docker-compose -f docker-compose.test.yml exec -T postgres pg_isready -U postgres
	@$(MAKE) test
	@docker-compose -f docker-compose.test.yml down
	@echo "$(GREEN)Docker tests complete$(NC)"

# Documentation targets
.PHONY: docs
docs: ## Generate documentation
	@echo "$(BLUE)Generating documentation...$(NC)"
	@cargo doc --workspace --all-features --no-deps
	@echo "$(GREEN)Documentation generated: $(TARGET_DIR)/doc/$(NC)"

.PHONY: docs-open
docs-open: docs ## Generate and open documentation
	@echo "$(BLUE)Opening documentation...$(NC)"
	@cargo doc --workspace --all-features --no-deps --open

# Release targets
.PHONY: pre-release
pre-release: fmt-check clippy audit test coverage ## Run pre-release checks
	@echo "$(BLUE)Running pre-release checks...$(NC)"
	@echo "$(GREEN)Pre-release checks complete$(NC)"

.PHONY: release
release: pre-release build ## Create a release build
	@echo "$(BLUE)Creating release build...$(NC)"
	@echo "$(GREEN)Release build complete$(NC)"

.PHONY: tag
tag: ## Create a git tag for the current version
	@echo "$(BLUE)Creating git tag v$(VERSION)...$(NC)"
	@git tag -a v$(VERSION) -m "Release version $(VERSION)"
	@git push origin v$(VERSION)
	@echo "$(GREEN)Git tag v$(VERSION) created$(NC)"

# Development environment targets
.PHONY: dev-up
dev-up: ## Start development environment
	@echo "$(BLUE)Starting development environment...$(NC)"
	@docker-compose -f docker-compose.test.yml up -d postgres redis ganache
	@echo "$(GREEN)Development environment started$(NC)"

.PHONY: dev-down
dev-down: ## Stop development environment
	@echo "$(BLUE)Stopping development environment...$(NC)"
	@docker-compose -f docker-compose.test.yml down
	@echo "$(GREEN)Development environment stopped$(NC)"

.PHONY: dev-logs
dev-logs: ## Show development environment logs
	@docker-compose -f docker-compose.test.yml logs -f

.PHONY: dev-reset
dev-reset: dev-down ## Reset development environment
	@echo "$(BLUE)Resetting development environment...$(NC)"
	@docker-compose -f docker-compose.test.yml down -v
	@docker-compose -f docker-compose.test.yml up -d postgres redis ganache
	@echo "$(GREEN)Development environment reset$(NC)"

# Utility targets
.PHONY: install
install: build ## Install binaries locally
	@echo "$(BLUE)Installing binaries...$(NC)"
	@cargo install --path . --force
	@echo "$(GREEN)Binaries installed$(NC)"

.PHONY: licenses
licenses: ## Generate license report
	@echo "$(BLUE)Generating license report...$(NC)"
	@cargo license --json > $(TARGET_DIR)/licenses.json
	@echo "$(GREEN)License report generated: $(TARGET_DIR)/licenses.json$(NC)"

.PHONY: size
size: build ## Show binary sizes
	@echo "$(BLUE)Binary sizes:$(NC)"
	@find $(TARGET_DIR)/release -maxdepth 1 -type f -executable -exec ls -lh {} \; | \
		awk '{print "  " $$9 ": " $$5}'

.PHONY: info
info: ## Show project information
	@echo "$(CYAN)Project Information$(NC)"
	@echo "$(CYAN)==================$(NC)"
	@echo "Name: $(PROJECT_NAME)"
	@echo "Version: $(VERSION)"
	@echo "Rust Version: $(RUST_VERSION)"
	@echo "Build Date: $(BUILD_DATE)"
	@echo "Git Commit: $(GIT_COMMIT)"
	@echo "Git Branch: $(GIT_BRANCH)"
	@echo "Docker Image: $(DOCKER_IMAGE):$(DOCKER_TAG)"

# CI/CD targets
.PHONY: ci
ci: fmt-check clippy audit test coverage ## Run CI pipeline
	@echo "$(GREEN)CI pipeline complete$(NC)"

.PHONY: cd
cd: ci build docker-build ## Run CD pipeline
	@echo "$(GREEN)CD pipeline complete$(NC)"

# Maintenance targets
.PHONY: update
update: ## Update all dependencies and tools
	@echo "$(BLUE)Updating dependencies and tools...$(NC)"
	@rustup update
	@cargo update
	@cargo install-update -a
	@echo "$(GREEN)Update complete$(NC)"

.PHONY: health
health: ## Check project health
	@echo "$(BLUE)Checking project health...$(NC)"
	@$(MAKE) fmt-check
	@$(MAKE) clippy
	@$(MAKE) audit
	@$(MAKE) outdated
	@$(MAKE) unused-deps
	@echo "$(GREEN)Project health check complete$(NC)"

# Composite targets
.PHONY: all
all: clean setup deps build test docs ## Build everything from scratch

.PHONY: quick
quick: fmt clippy test ## Quick development cycle

.PHONY: full
full: clean all test-enterprise coverage docs ## Full build and test cycle

# Make sure scripts are executable
$(SCRIPTS_DIR)/run_enterprise_tests.sh:
	@chmod +x $@

# Phony targets to avoid conflicts with files
.PHONY: all quick full ci cd
