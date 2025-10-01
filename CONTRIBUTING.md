# Contributing to DD Blind Box

Thank you for your interest in contributing to DD Blind Box! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Docker (for CosmWasm optimization)
- Git

### Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/dd_blind_box.git
   cd dd_blind_box
   ```

3. Install dependencies:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Code Style

### Rust Conventions

- Follow standard Rust formatting: `cargo fmt`
- Use clippy for linting: `cargo clippy`
- Document all public functions and structs
- Use meaningful variable and function names

### CosmWasm Best Practices

- Use `cosmwasm_std` types and functions
- Implement proper error handling with custom error types
- Follow the CW721 standard for NFT operations
- Use `cw-storage-plus` for state management

## Testing

### Test Requirements

- All new features must include comprehensive tests
- Maintain 100% test coverage for critical functions
- Include both unit tests and integration tests
- Test edge cases and error conditions

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test settlement

# Run with output
cargo test -- --nocapture
```

### Test Structure

- Unit tests: Test individual functions in isolation
- Integration tests: Test complete workflows
- Error boundary tests: Test error conditions and edge cases

## Security Considerations

### Security Requirements

- All administrative functions must have proper access control
- Input validation for all user-provided data
- Protection against common attacks (reentrancy, DoS, etc.)
- Secure random number generation for fair distribution

### Security Review Process

1. All security-related changes require review
2. New attack vectors must be documented
3. Security tests must be added for new vulnerabilities
4. Consider external security audits for major changes

## Pull Request Process

### Before Submitting

1. Ensure all tests pass: `cargo test`
2. Run clippy: `cargo clippy`
3. Format code: `cargo fmt`
4. Update documentation if needed
5. Add tests for new functionality

### PR Requirements

- Clear description of changes
- Reference any related issues
- Include test coverage information
- Update README.md if adding new features
- Ensure CI passes

### Review Process

1. Automated tests must pass
2. Code review by maintainers
3. Security review for sensitive changes
4. Documentation review

## Issue Reporting

### Bug Reports

When reporting bugs, please include:

- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (Rust version, OS, etc.)
- Relevant logs or error messages

### Feature Requests

For feature requests, please include:

- Clear description of the proposed feature
- Use case and motivation
- Potential implementation approach
- Impact on existing functionality

## Documentation

### Code Documentation

- Document all public APIs
- Include examples for complex functions
- Use rustdoc format for documentation comments
- Keep documentation up to date with code changes

### README Updates

- Update README.md for new features
- Include usage examples
- Update installation instructions if needed
- Keep feature list current

## Release Process

### Version Bumping

- Follow semantic versioning (SemVer)
- Update version in Cargo.toml
- Update CHANGELOG.md
- Create release notes

### Release Checklist

1. All tests pass
2. Documentation is updated
3. Version is bumped
4. CHANGELOG.md is updated
5. Release notes are prepared
6. Tag is created

## Community Guidelines

### Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the project's mission and values

### Communication

- Use clear and concise language
- Provide context for questions and suggestions
- Be patient with newcomers
- Share knowledge and best practices

## Getting Help

### Resources

- GitHub Issues for bug reports and feature requests
- GitHub Discussions for questions and general discussion
- Documentation in the repository
- CosmWasm documentation for framework-specific questions

### Contact

- Create an issue for technical questions
- Use discussions for general questions
- Tag maintainers for urgent issues

Thank you for contributing to DD Blind Box!
