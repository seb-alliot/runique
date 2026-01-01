# 🦀 Rusti Framework

> A modern web framework for Rust, inspired by Django and built on Axum

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/seb-alliot/rusti.git)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## ✨ Why Rusti?

Rusti combines **Django's familiarity** with **Rust's performance**. If you know Django, you'll feel right at home.

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .routes(urlpatterns! {
            "/" => get(index), name = "home",
            "/about" => get(about), name = "about",
        })
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .run()
        .await?;

    Ok(())
}
```

## 🚀 Key Features

| Feature | Description |
|---------|-------------|
| 🎯 **Django-like** | Familiar syntax, proven patterns |
| ⚡ **Performance** | Built on Axum and Tokio |
| 🛡️ **Built-in Security** | CSRF, sessions, validation |
| 📝 **Tera Templates** | Jinja2-inspired engine |
| 🗄️ **SeaORM ORM** | Multi-database support (PostgreSQL, MySQL, SQLite) |
| 🔧 **Flexible Config** | Builder pattern + environment variables |
| 🐛 **Advanced Debug** | Detailed error pages in development |
| 📨 **Flash Messages** | Messages between requests |
| 🔗 **Reverse Routing** | Automatically generated URLs |

## 📦 Installation

```toml
[dependencies]
rusti = "1.0"
tokio = { version = "1", features = ["full"] }
```

### Database Selection

```toml
# SQLite (default)
rusti = "0.1"

# PostgreSQL
rusti = { version = "0.1", features = ["postgres"] }

# MySQL / MariaDB
rusti = { version = "0.1", features = ["mysql"] }

# All databases
rusti = { version = "0.1", features = ["all-databases"] }
```

## 📚 Documentation

- **[Getting Started](informations/documentation%20english/GETTING_STARTED.md)** - Quick start guide
- **[Full Documentation](informations/documentation%20english/)** - Complete documentation
- **[Cours d'implémentation](informations/cours/)** - Learn how to implement features yourself
- **[Examples](examples/demo-app/)** - Working examples

## 🎓 Learning Resources

- **[Cours Rusti](informations/cours/)** - Step-by-step implementation guides
- **[Documentation française](informations/documentation%20french/)** - Documentation complète en français

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test allowed_hosts
```

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](informations/documentation%20english/CONTRIBUTING.md) for guidelines.

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- Inspired by [Django](https://www.djangoproject.com/)
- Built on [Axum](https://github.com/tokio-rs/axum)
- Uses [SeaORM](https://www.sea-ql.org/SeaORM/) for database operations
- Template engine powered by [Tera](https://keats.github.io/tera/)

---

**Made with 🦀 Rust**
