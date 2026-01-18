# Runique

**A Django-inspired web framework for Rust**

[![Version](https://img.shields.io/badge/version-0.1.86-blue.svg)](https://crates.io/crates/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

⚠️ **Status: Active development (v0.1.x)**

The API may change between minor versions. Complete documentation will be updated after the framework's core refactoring (v0.2.0).

---

## 🚀 Installation
```toml
[dependencies]
runique = { version = "0.1", features = ["sqlite"] }
```

**Available features:** `sqlite`, `postgres`, `mysql`, `mariadb`, `all-databases`

---

## 🎯 Key Features

- 🎨 **Django-like architecture** - Familiar API with declarative routing
- 📝 **Form system** - Automatic generation and validation
- 🔐 **Built-in security** - CSRF, CSP, sanitization, ALLOWED_HOSTS validation
- 💾 **Django-style ORM** - Based on SeaORM with intuitive API
- 🎨 **Tera templates** - Preprocessing with Django syntax
- ⚡ **Rust performance** - Native async/await with Tokio

---

## 🏁 Quick Start

### CLI Installation
```bash
cargo install runique
```

### Create a New Project
```bash
runique new my_app
cd my_app
cargo run
```

The CLI generates a complete structure with:
- User model with authentication
- Registration and login forms
- Responsive design templates
- Database configuration
- Ready-to-use migrations

---

## 📦 Minimal Example
```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## 🔧 Configuration (.env)
```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key
DEBUG=true

# Database (SQLite by default)
DB_ENGINE=sqlite
DB_NAME=app.db
```

---

## 📚 Documentation

Complete documentation will be available after API stabilization (v0.2.0).

In the meantime:
- Check examples in the `examples/` folder
- Use `cargo doc --open` for API documentation
- Join our Discord for help

---

## 🛠️ Development
```bash
# Tests
cargo test

# Formatting
cargo fmt

# Linting
cargo clippy
```

---

## 🤝 Contributing

Contributions are welcome! Open an issue or submit a PR.

---

## 📄 License

MIT - See LICENSE-MIT for details.

---

## 📧 Contact

- **GitHub**: seb-alliot/runique
- **Discord**: discord.gg/Y5zW7rbt
- **Email**: alliotsebastien04@gmail.com

---

**Built with ❤️ and 🦀**