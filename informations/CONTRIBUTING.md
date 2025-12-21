# Contributing to Rusti

Thank you for your interest in contributing to Rusti! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/seb-alliot/rusti/tree/main`
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Commit your changes: `git commit -am 'Add some feature'`
6. Push to the branch: `git push origin feature/your-feature-name`
7. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Build the entire workspace
cargo build --workspace

# Build with all features
cargo build --all-features

# Build documentation
cargo doc --workspace --no-deps --open
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific package
cargo test -p rusti

# Run tests with all features
cargo test --all-features
```

### Code Style

We use `rustfmt` and `clippy` to maintain code quality:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace -- -D warnings
```

## Project Structure

```
rusti/
├── rusti/              # Core framework library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── app.rs
│   │   ├── config.rs
│   │   └── ...
│   └── Cargo.toml
├── examples/
│   └── demo-app/       # Example applications
└── Cargo.toml          # Workspace configuration
```

## What to Contribute

### Ideas for Contributions

- **Bug fixes**: Fix existing issues
- **Documentation**: Improve docs, add examples
- **Features**: Implement new framework features
- **Tests**: Add test coverage
- **Examples**: Create new example applications
- **Performance**: Optimize existing code

### Areas Needing Help

- More comprehensive error messages
- Additional middleware components
- Database migration tools
- Admin interface (Django-like)
- Form handling utilities
- Authentication/authorization helpers

## Pull Request Guidelines

### Before Submitting

- [ ] Code follows project style guidelines
- [ ] Tests pass locally
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (for significant changes)
- [ ] Commit messages are clear and descriptive

### PR Description Should Include

- What the change does
- Why the change is needed
- Any breaking changes
- Related issues (if applicable)

## Code Review Process

1. A maintainer will review your PR
2. They may request changes
3. Once approved, your PR will be merged
4. Your contribution will be credited in the release notes

## Feature Requests

Have an idea for a new feature? Please:

1. Check if it's already been proposed in Issues
2. Open a new issue with the "enhancement" label
3. Describe the feature and its use case
4. Be open to discussion and feedback

## Bug Reports

When reporting bugs, please include:

- Rust version
- Rusti version
- Operating system
- Minimal code example that reproduces the issue
- Expected behavior vs. actual behavior
- Error messages or stack traces

## Questions?

If you have questions:

- Check the documentation
- Look through existing issues
- Open a new issue with the "question" label
- Join our community (if we have one)

## Code of Conduct

Be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## License

By contributing to Rusti, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

Thank you for contributing to Rusti! 🦀
