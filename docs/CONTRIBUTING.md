# Contributing to StableRWA Platform

Thank you for your interest in contributing to the StableRWA platform! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Security](#security)

## Code of Conduct

This project adheres to a code of conduct that we expect all contributors to follow. Please read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before contributing.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Docker and Docker Compose
- PostgreSQL 15+
- Redis 7+
- Git

### Development Environment Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/arkSong/rwa-platform.git
   cd rwa-platform
   ```

2. **Install Rust toolchain**
   ```bash
   rustup update stable
   rustup component add rustfmt clippy
   ```

3. **Start development services**
   ```bash
   docker-compose up -d postgres redis rabbitmq
   ```

4. **Run database migrations**
   ```bash
   cargo run --bin migrate
   ```

5. **Run tests**
   ```bash
   cargo test
   ```

6. **Start the development server**
   ```bash
   cargo run --bin service-gateway
   ```

## Development Workflow

### Branch Strategy

We use a Git flow-based branching strategy:

- `main`: Production-ready code
- `develop`: Integration branch for features
- `feature/*`: Feature development branches
- `hotfix/*`: Critical bug fixes
- `release/*`: Release preparation branches

### Workflow Steps

1. **Create a feature branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write code following our coding standards
   - Add tests for new functionality
   - Update documentation as needed

3. **Test your changes**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

5. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

## Coding Standards

### Rust Style Guide

We follow the official Rust style guide with some additional conventions:

#### Code Formatting
- Use `cargo fmt` to format code
- Maximum line length: 100 characters
- Use 4 spaces for indentation

#### Naming Conventions
- **Functions and variables**: `snake_case`
- **Types and traits**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

#### Documentation
- All public APIs must have documentation comments
- Use `///` for doc comments
- Include examples in documentation when appropriate

```rust
/// Calculates the total value of an asset portfolio.
///
/// # Arguments
///
/// * `assets` - A vector of assets to calculate the total value for
/// * `currency` - The currency to express the total value in
///
/// # Returns
///
/// Returns the total value as a `Decimal` or an error if calculation fails.
///
/// # Examples
///
/// ```
/// use stablerwa::calculate_portfolio_value;
/// 
/// let assets = vec![/* ... */];
/// let total = calculate_portfolio_value(&assets, "USD")?;
/// ```
pub fn calculate_portfolio_value(
    assets: &[Asset],
    currency: &str,
) -> Result<Decimal, CalculationError> {
    // Implementation
}
```

#### Error Handling
- Use `Result<T, E>` for fallible operations
- Create custom error types using `thiserror`
- Provide meaningful error messages

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {id}")]
    NotFound { id: String },
    #[error("Invalid asset type: {asset_type}")]
    InvalidType { asset_type: String },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

#### Async Code
- Use `async/await` for asynchronous operations
- Prefer `tokio` for async runtime
- Use `async-trait` for async traits

### Code Organization

#### Module Structure
```
src/
├── lib.rs          # Library root
├── error.rs        # Error types
├── types.rs        # Common types
├── config.rs       # Configuration
├── handlers/       # HTTP handlers
├── services/       # Business logic
├── repositories/   # Data access
└── utils/          # Utility functions
```

#### Dependency Management
- Keep dependencies minimal and well-justified
- Use specific version ranges in `Cargo.toml`
- Regularly update dependencies for security

### Database Guidelines

#### Migrations
- Use descriptive migration names
- Include both `up` and `down` migrations
- Test migrations on sample data

#### Queries
- Use parameterized queries to prevent SQL injection
- Optimize queries for performance
- Use database transactions for consistency

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test service interactions
3. **End-to-End Tests**: Test complete user workflows

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_asset_success() {
        // Arrange
        let asset_data = AssetData {
            name: "Test Asset".to_string(),
            // ... other fields
        };
        
        // Act
        let result = create_asset(asset_data).await;
        
        // Assert
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.name, "Test Asset");
    }
    
    #[tokio::test]
    async fn test_create_asset_invalid_data() {
        // Test error cases
    }
}
```

### Test Data
- Use factories for creating test data
- Clean up test data after tests
- Use separate test database

### Coverage
- Aim for 80%+ code coverage
- Focus on critical business logic
- Use `cargo tarpaulin` for coverage reports

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **API Documentation**: Generated from doc comments
3. **Architecture Documentation**: High-level system design
4. **User Documentation**: End-user guides and tutorials

### Documentation Standards

- Write clear, concise documentation
- Include examples where helpful
- Keep documentation up-to-date with code changes
- Use proper grammar and spelling

### Generating Documentation

```bash
# Generate API documentation
cargo doc --no-deps --open

# Generate architecture diagrams
# (requires mermaid-cli)
mmdc -i docs/architecture.mmd -o docs/architecture.png
```

## Pull Request Process

### Before Submitting

1. **Code Quality**
   - [ ] Code follows style guidelines
   - [ ] All tests pass
   - [ ] No clippy warnings
   - [ ] Code is properly formatted

2. **Documentation**
   - [ ] Public APIs are documented
   - [ ] README updated if needed
   - [ ] Architecture docs updated if needed

3. **Testing**
   - [ ] New features have tests
   - [ ] Bug fixes have regression tests
   - [ ] Integration tests pass

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

### Review Process

1. **Automated Checks**: CI pipeline must pass
2. **Code Review**: At least one approval required
3. **Security Review**: For security-sensitive changes
4. **Architecture Review**: For significant architectural changes

## Issue Reporting

### Bug Reports

Use the bug report template and include:

- **Environment**: OS, Rust version, dependencies
- **Steps to Reproduce**: Clear, numbered steps
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Additional Context**: Logs, screenshots, etc.

### Feature Requests

Use the feature request template and include:

- **Problem Statement**: What problem does this solve?
- **Proposed Solution**: How should it work?
- **Alternatives**: Other solutions considered
- **Additional Context**: Use cases, examples

## Security

### Reporting Security Issues

**DO NOT** create public issues for security vulnerabilities.

Instead, email security@stablerwa.com with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Security Guidelines

- Never commit secrets or credentials
- Use environment variables for configuration
- Validate all user inputs
- Use secure coding practices
- Keep dependencies updated

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Email**: security@stablerwa.com for security issues

### Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Project Documentation](docs/)

## Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes for significant contributions
- Annual contributor appreciation

Thank you for contributing to StableRWA! 🚀
