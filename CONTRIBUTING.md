# Contributing to RWA Platform

Thank you for your interest in contributing to the RWA Platform! This document provides guidelines and information for contributors.

## 🤝 Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before contributing.

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** 1.75+ with Cargo
- **Docker** 20.10+ and Docker Compose
- **Git** for version control
- **PostgreSQL** 15+ (or use Docker)
- **Redis** 7+ (or use Docker)

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/rwa-platform.git
   cd rwa-platform
   ```

3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/rwa-platform/rwa-platform.git
   ```

4. **Set up the development environment**:
   ```bash
   # Start infrastructure services
   docker-compose up -d postgres redis

   # Install dependencies and build
   cargo build --workspace

   # Run tests to ensure everything works
   cargo test --workspace
   ```

## 📝 How to Contribute

### Reporting Bugs

Before creating bug reports, please check the [existing issues](https://github.com/rwa-platform/rwa-platform/issues) to avoid duplicates.

When creating a bug report, include:

- **Clear title** and description
- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **Environment details** (OS, Rust version, etc.)
- **Relevant logs** or error messages
- **Screenshots** if applicable

Use the bug report template when available.

### Suggesting Enhancements

Enhancement suggestions are welcome! Please:

- **Check existing issues** for similar suggestions
- **Provide clear rationale** for the enhancement
- **Describe the proposed solution** in detail
- **Consider backwards compatibility**
- **Include examples** if applicable

### Pull Requests

1. **Create a feature branch** from `develop`:
   ```bash
   git checkout develop
   git pull upstream develop
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards
3. **Add tests** for new functionality
4. **Update documentation** as needed
5. **Commit your changes** with clear messages
6. **Push to your fork** and create a pull request

#### Pull Request Guidelines

- **Target the `develop` branch** for new features
- **Target the `main` branch** only for hotfixes
- **Include a clear description** of changes
- **Reference related issues** using keywords (e.g., "Fixes #123")
- **Ensure all tests pass** and coverage is maintained
- **Follow the PR template** when available

## 🎯 Development Guidelines

### Code Style

#### Rust Code Style

- **Follow `rustfmt`** formatting (run `cargo fmt`)
- **Address `clippy` warnings** (run `cargo clippy`)
- **Use meaningful variable names** and function names
- **Add documentation** for public APIs
- **Follow Rust naming conventions**

Example:
```rust
/// Processes a payment transaction
/// 
/// # Arguments
/// * `payment_request` - The payment details to process
/// * `user_id` - The ID of the user making the payment
/// 
/// # Returns
/// * `Result<PaymentResult, PaymentError>` - Success or error result
pub async fn process_payment(
    payment_request: &PaymentRequest,
    user_id: &str,
) -> Result<PaymentResult, PaymentError> {
    // Implementation
}
```

#### Database Migrations

- **Use descriptive migration names**
- **Include both up and down migrations**
- **Test migrations thoroughly**
- **Document breaking changes**

#### API Design

- **Follow RESTful conventions**
- **Use consistent error responses**
- **Include proper HTTP status codes**
- **Document all endpoints**

### Testing Standards

#### Unit Tests

- **Test all public functions**
- **Use descriptive test names**
- **Follow the AAA pattern** (Arrange, Act, Assert)
- **Mock external dependencies**

```rust
#[tokio::test]
async fn test_user_registration_with_valid_data() {
    // Arrange
    let user_service = UserService::new(mock_database());
    let user_data = create_valid_user_data();

    // Act
    let result = user_service.register_user(user_data).await;

    // Assert
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

#### Integration Tests

- **Test complete workflows**
- **Use test databases**
- **Clean up test data**
- **Test error scenarios**

#### Test Coverage

- **Maintain >90% test coverage**
- **Focus on critical paths**
- **Test edge cases**
- **Include performance tests for critical components**

### Documentation Standards

#### Code Documentation

- **Document all public APIs**
- **Include examples in documentation**
- **Explain complex algorithms**
- **Document error conditions**

#### README Updates

- **Update README for new features**
- **Include setup instructions**
- **Add usage examples**
- **Update dependency lists**

#### API Documentation

- **Use OpenAPI/Swagger specifications**
- **Include request/response examples**
- **Document authentication requirements**
- **Explain error responses**

## 🔧 Development Workflow

### Branch Strategy

- **`main`**: Production-ready code
- **`develop`**: Integration branch for features
- **`feature/*`**: Feature development branches
- **`hotfix/*`**: Critical bug fixes
- **`release/*`**: Release preparation branches

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

Examples:
```
feat(auth): add JWT refresh token support

fix(payment): handle stripe webhook timeout errors

docs(api): update authentication documentation
```

### Release Process

1. **Create release branch** from `develop`
2. **Update version numbers** and changelog
3. **Run full test suite**
4. **Create pull request** to `main`
5. **Tag release** after merge
6. **Deploy to production**
7. **Merge back** to `develop`

## 🛡️ Security Guidelines

### Security Best Practices

- **Never commit secrets** or credentials
- **Use environment variables** for configuration
- **Validate all inputs**
- **Follow OWASP guidelines**
- **Report security issues** privately

### Reporting Security Vulnerabilities

**DO NOT** create public issues for security vulnerabilities.

Instead, email security@rwa-platform.com with:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## 📊 Performance Guidelines

### Performance Considerations

- **Profile before optimizing**
- **Use appropriate data structures**
- **Minimize database queries**
- **Cache frequently accessed data**
- **Monitor memory usage**

### Benchmarking

- **Add benchmarks for critical paths**
- **Use `cargo bench` for performance tests**
- **Document performance requirements**
- **Monitor performance regressions**

## 🎉 Recognition

Contributors will be recognized in:
- **CONTRIBUTORS.md** file
- **Release notes** for significant contributions
- **GitHub contributors** page
- **Annual contributor highlights**

## 📞 Getting Help

If you need help:

- **Check the documentation** first
- **Search existing issues** and discussions
- **Ask in GitHub Discussions**
- **Join our Discord** (link in README)
- **Email**: contributors@rwa-platform.com

## 📋 Checklist for Contributors

Before submitting a pull request:

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated and passing
- [ ] Documentation updated
- [ ] No merge conflicts
- [ ] Commit messages follow conventions
- [ ] Security considerations addressed
- [ ] Performance impact considered

Thank you for contributing to the RWA Platform! 🚀
