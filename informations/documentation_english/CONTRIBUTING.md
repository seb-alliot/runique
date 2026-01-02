Voici la traduction en anglais de ton guide de contribution, en respectant le format Markdown et les terminologies standards de l'écosystème Open Source.

---

# 🤝 Contributing Guide - Rusti Framework

Thank you for your interest in contributing to Rusti! This guide will help you get started.

## Table of Contents

1. [Code of Conduct](https://www.google.com/search?q=%23code-of-conduct)
2. [How to Contribute](https://www.google.com/search?q=%23how-to-contribute)
3. [Environment Setup](https://www.google.com/search?q=%23environment-setup)
4. [Contribution Workflow](https://www.google.com/search?q=%23contribution-workflow)
5. [Code Standards](https://www.google.com/search?q=%23code-standards)
6. [Testing](https://www.google.com/search?q=%23testing)
7. [Documentation](https://www.google.com/search?q=%23documentation)

---

## Code of Conduct

We are committed to creating a welcoming and inclusive community. By participating in this project, you agree to:

* Respect all contributors
* Accept constructive criticism
* Focus on what is best for the community
* Show empathy towards others

---

## How to Contribute

### 🐛 Report a Bug

1. Check that the bug has not already been reported in the [Issues](https://github.com/your-repo/rusti/issues)
2. Open a new issue using the "Bug Report" template
3. Provide a minimal reproducible example
4. Include system information (OS, Rust version, etc.)

### ✨ Propose a Feature

1. Open an issue using the "Feature Request" template
2. Explain the problem you want to solve
3. Describe your proposed solution
4. Discuss with the community before coding

### 📝 Improve Documentation

Documentation is just as important as code!

* Typos corrections
* Clarifications
* New examples
* Translations

---

## Environment Setup

### Prerequisites

* Rust 1.70 or higher
* Git
* PostgreSQL, MySQL, or SQLite (for DB tests)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-repo/rusti.git
cd rusti

# Install dependencies
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-features -- -D warnings

```

### Project Structure

```
rusti/
├── rusti/                  # Framework core
│   ├── src/
│   │   ├── lib.rs
│   │   ├── app.rs
│   │   ├── settings.rs
│   │   ├── middleware/
│   │   ├── database/
│   │   └── ...
│   ├── templates/          # Internal templates
│   ├── static/             # Framework assets
│   └── tests/
│
├── examples/
│   └── demo-app/           # Example application
│
├── docs/                   # Documentation
│   ├── README.md
│   ├── GETTING_STARTED.md
│   └── ...
│
└── Cargo.toml              # Workspace root

```

---

## Contribution Workflow

### 1. Fork and Clone

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR-USERNAME/rusti.git
cd rusti
git remote add upstream https://github.com/your-repo/rusti.git

```

### 2. Create a Branch

```bash
# Feature
git checkout -b feature/my-awesome-feature

# Bugfix
git checkout -b fix/bug-fix

# Documentation
git checkout -b docs/improve-docs

```

### 3. Develop

```bash
# Make your changes

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy --all-features -- -D warnings

```

### 4. Commit

Use clear commit messages:

```bash
# ✅ Good
git commit -m "feat: add WebSocket support"
git commit -m "fix: correct CSRF validation"
git commit -m "docs: improve ORM examples"

# ❌ Bad
git commit -m "update"
git commit -m "fix stuff"
git commit -m "WIP"

```

**Commit Format:**

* `feat:` New feature
* `fix:` Bug fix
* `docs:` Documentation
* `style:` Formatting, no code change
* `refactor:` Refactoring
* `test:` Adding/modifying tests
* `chore:` Maintenance (dependencies, etc.)

### 5. Push and Pull Request

```bash
# Push to your fork
git push origin feature/my-awesome-feature

# Create a Pull Request on GitHub

```

**Pull Request Template:**

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation

## Tests
- [ ] Unit tests added/modified
- [ ] Integration tests added/modified
- [ ] All tests passed

## Checklist
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)

```

---

## Code Standards

### Rust Style

Follow standard Rust conventions:

```rust
// ✅ Good
pub struct RustiApp {
    router: Router,
    config: Arc<Settings>,
}

impl RustiApp {
    pub fn new(settings: Settings) -> Result<Self> {
        // ...
    }
}

// ❌ Bad
pub struct rustiApp {
    Router: Router,
    CONFIG: Arc<Settings>,
}

```

### Documentation

Document all public functions:

```rust
/// Creates a new instance of RustiApp
///
/// # Examples
///
/// ```rust
/// use rusti::{RustiApp, Settings};
///
/// let app = RustiApp::new(Settings::default_values())?;
/// ```
///
/// # Errors
///
/// Returns an error if the configuration is invalid
pub fn new(settings: Settings) -> Result<Self> {
    // ...
}

```

### Error Handling

Use `Result` and appropriate error types:

```rust
// ✅ Good
pub fn connect(&self) -> Result<DatabaseConnection, DbErr> {
    // ...
}

// ❌ Bad
pub fn connect(&self) -> DatabaseConnection {
    // panic! if error
}

```

### Testing

Write tests for every feature:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_builder() {
        let settings = Settings::builder()
            .debug(true)
            .build();

        assert!(settings.debug);
    }

    #[tokio::test]
    async fn test_app_creation() {
        let settings = Settings::default_values();
        let app = RustiApp::new(settings).await;

        assert!(app.is_ok());
    }
}

```

---

## Testing

### Run All Tests

```bash
# Unit and integration tests
cargo test

# Tests for a specific feature
cargo test --features postgres

# Tests with detailed output
cargo test -- --nocapture

# Parallel tests
cargo test -- --test-threads=4

```

### Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate report
cargo tarpaulin --out Html --output-dir coverage

```

### Benchmarks

```bash
# Install criterion
cargo install cargo-criterion

# Run benchmarks
cargo bench

```

---

## Documentation

### Code Documentation

```bash
# Generate documentation
cargo doc

# Open in browser
cargo doc --open

# Including private items
cargo doc --document-private-items

```

### Markdown Documentation

Documentation files are located in `docs/`:

* Use clear headings
* Include code examples
* Add links between documents
* Keep an accessible tone

### Examples

Examples in `examples/` must:

* Be functional (`cargo run` should work)
* Be well-commented
* Cover a real-world use case
* Include a README.md

---

## Code Review

All Pull Requests are reviewed by maintainers. Please be patient and open to suggestions.

### Review Criteria

* ✅ Clean and well-structured code
* ✅ Tests passing
* ✅ Documentation updated
* ✅ No undocumented breaking changes
* ✅ Acceptable performance
* ✅ Security respected

### After Review

* Respond to comments
* Perform requested changes
* Mark conversations as resolved
* Request a new review

---

## Getting Started

### "Good first issue"

Look for issues labeled `good first issue` to start:

* Simple bugs
* Documentation improvements
* Small features

### Mentors

Don't hesitate to ask for help:

* Comment on the issue
* Join GitHub discussions
* Ask questions (there are no stupid questions!)

---

## Resources

### Rust Documentation

* [The Rust Book](https://doc.rust-lang.org/book/)
* [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
* [Async Book](https://rust-lang.github.io/async-book/)

### Main Dependencies

* [Axum](https://docs.rs/axum/)
* [Tokio](https://tokio.rs/)
* [SeaORM](https://www.sea-ql.org/SeaORM/)
* [Tera](https://keats.github.io/tera/)

### Useful Tools

* [rust-analyzer](https://rust-analyzer.github.io/) - IDE LSP
* [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-reload
* [cargo-edit](https://github.com/killercup/cargo-edit) - Manage dependencies

---

## Questions?

* 💬 [GitHub Discussions](https://github.com/your-repo/rusti/discussions)
* 🐛 [Issues](https://github.com/your-repo/rusti/issues)
* 📧 Email: [your-email@example.com]

---

## Acknowledgments

Thank you for contributing to Rusti! Every contribution, no matter how small, helps improve the framework.

**Together, let's build the best web framework for Rust! 🦀**

---

Souhaitez-vous que je traduise également le **CHANGELOG** ou un autre document technique pour finaliser la version anglaise de votre dépôt ?