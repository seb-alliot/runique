# 📚 Runique Framework Documentation

Welcome to the complete documentation for Runique, a modern web framework for Rust inspired by Django.

## 🎯 Quick Navigation

| Document | Description | For Whom? |
|----------|-------------|-----------|
| **[README](README.md)** | Overview and installation | Everyone |
| **[GETTING_STARTED](GETTING_STARTED.md)** | Step-by-step tutorial | Beginners |
| **[TEMPLATES](TEMPLATES.md)** | Template system | Frontend developers |
| **[DATABASE](DATABASE.md)** | ORM and database | Backend developers |
| **[CONFIGURATION](CONFIGURATION.md)** | Complete configuration | DevOps / Production |

---

## 📖 Learning Paths

### 🌱 Beginner Level

1. **[README](README.md)** - Understand what Runique is
2. **[GETTING_STARTED](GETTING_STARTED.md)** - Create your first application
3. **[TEMPLATES](TEMPLATES.md)** - Master templates

**Estimated time:** 2-3 hours

### 🚀 Intermediate Level

1. **[DATABASE](DATABASE.md)** - Use Django-like ORM
2. **[CONFIGURATION](CONFIGURATION.md)** - Configure your application
3. Examples in `examples/demo-app`

**Estimated time:** 4-6 hours

### ⚡ Advanced Level

1. Custom middleware
2. Performance optimizations
3. Production deployment
4. Multi-service architecture

**Estimated time:** Variable

---

## 🎓 Task-Based Guide

### "I want to create a simple web application"

1. [Quick installation](README.md#-installation)
2. [First application](GETTING_STARTED.md#first-application)
3. [Add templates](TEMPLATES.md)
4. [Serve static files](GETTING_STARTED.md#static-files)

### "I want to add a database"

1. [Database configuration](DATABASE.md#configuration)
2. [Define models](DATABASE.md#defining-models)
3. [Django-like API](DATABASE.md#django-like-api)
4. [Migrations](DATABASE.md#migrations)

### "I want to deploy to production"

1. [Production configuration](CONFIGURATION.md#production)
2. [Optimized build](CONFIGURATION.md#optimized-build)
3. [Security](CONFIGURATION.md#security)
4. [Production checklist](CONFIGURATION.md#production-checklist)

### "I want to create a REST API"

1. [JSON handlers](GETTING_STARTED.md#routes-and-handlers)
2. [Data validation](DATABASE.md)
3. [Error handling](CONFIGURATION.md#logging-and-tracing)

---

## 📂 Documentation Structure

```
documentation/
├── README.md                # Framework overview
├── INDEX.md                 # This file - Navigation
├── GETTING_STARTED.md       # Complete step-by-step tutorial
├── TEMPLATES.md             # Tera template system
├── DATABASE.md              # ORM and databases
└── CONFIGURATION.md         # Configuration and production
```

---

## 🔑 Key Concepts

### RuniqueApp - The Framework Core

```rust
RuniqueApp::new(settings).await?
    .routes(routes)              // Add routes
    .with_database(db)           // Optional: Database
    .with_static_files()?        // Optional: Static files
    .with_default_middleware()   // Optional: Error middleware
    .run().await?;               // Start server
```

**See:** [Getting Started - Structure](GETTING_STARTED.md#project-structure)

### Settings - Flexible Configuration

```rust
// Builder pattern
Settings::builder()
    .debug(true)
    .server("127.0.0.1", 3000, "secret")
    .templates_dir(vec!["templates".to_string()])
    .build()
```

**See:** [Configuration - Settings](CONFIGURATION.md#settings)

### urlpatterns! - Django-like Routing

```rust
urlpatterns! {
    "/" => get(index), name = "home",
    "/user/{id}" => get(user_detail), name = "user_profile",
}
```

**See:** [Getting Started - Routes](GETTING_STARTED.md#routes-and-handlers)

### Django-like ORM

```rust
Entity::objects
    .filter(Column::Age.gte(18))
    .exclude(Column::IsBanned.eq(true))
    .order_by_desc(Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?
```

**See:** [Database - Django-like API](DATABASE.md#django-like-api)

---

## 🎨 Main Features

| Feature | Documentation | Example |
|---------|---------------|---------|
| **Tera Templates** | [TEMPLATES.md](TEMPLATES.md) | `{% static "file.css" %}` |
| **Custom Tags** | [TEMPLATES.md](TEMPLATES.md#available-tags) | `{% csrf %}`, `{% messages %}` |
| **Reverse routing** | [TEMPLATES.md](TEMPLATES.md#-link-route_name-params) | `{% link "home" %}` |
| **Flash messages** | [GETTING_STARTED.md](GETTING_STARTED.md#routes-and-handlers) | `success!(message,"message");` |
| **CSRF protection** | [CONFIGURATION.md](CONFIGURATION.md#middleware) | `.with_csrf_tokens()` |
| **SeaORM ORM** | [DATABASE.md](DATABASE.md) | `Entity::objects.all()` |
| **Migrations** | [DATABASE.md](DATABASE.md#migrations) | `sea-orm-cli migrate up` |
| **Sessions** | [CONFIGURATION.md](CONFIGURATION.md) | Automatic |
| **Debug pages** | [CONFIGURATION.md](CONFIGURATION.md#production) | `debug = true` mode |

---

## 🛠️ Quick Reference

### Common Commands

```bash
# Create a project
cargo new my-app && cd my-app
cargo add runique tokio --features full

# Run in dev
cargo run

# Production build
cargo build --release

# Tests
cargo test

# Documentation
cargo doc --open

# Migrations
sea-orm-cli migrate up
sea-orm-cli migrate down
```

### Important Files

```
my-project/
├── src/
│   ├── main.rs          # Entry point
│   ├── urls.rs          # Routes
│   └── views.rs         # Handlers
├── templates/           # Tera templates
├── static/              # CSS, JS, images
├── media/               # Uploads
├── .env                 # Configuration
└── Cargo.toml
```

### Common Imports

```rust
use runique::prelude::*;  // Main import

// Or specific
use runique::{
    RuniqueApp,
    Settings,
    Router,
    Context,
    Template,
    Message,
    Response,
    StatusCode,
    Extension,
    Path,
    Json,
};
```

---

## 🐛 Troubleshooting

### "Template not found"

**Solution:** Check `templates_dir` in Settings
```rust
.templates_dir(vec!["templates".to_string()])
```

**See:** [Templates - Configuration](TEMPLATES.md#configuration)

### "CSRF token verification failed"

**Solution:** Enable CSRF middleware
```rust
.with_csrf_tokens()
```

**See:** [Configuration - Middleware](CONFIGURATION.md#middleware)

### "Database connection failed"

**Solution:** Check your `.env` and Cargo feature
```toml
runique = { version = "0.1", features = ["postgres"] }
```

**See:** [Database - Configuration](DATABASE.md#configuration)

### "Route not found with {% link %}"

**Solution:** Add `name = "..."` to your route
```rust
urlpatterns! {
    "/" => get(index), name = "home",  // ✅
}
```

**See:** [Templates - Link](TEMPLATES.md#-link-route_name-params)

---

## 💡 Practical Examples

### Example 1: Minimal Application

```rust
use runique::prelude::*;

async fn hello() -> &'static str {
    "Hello, Runique!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RuniqueApp::new(Settings::default_values()).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;
    Ok(())
}
```

**See:** [Getting Started - First app](GETTING_STARTED.md#first-application)

### Example 2: With Templates and Database

**See:** [Getting Started - Complete example](GETTING_STARTED.md#complete-example)

### Example 3: REST API

**See:** [Getting Started - JSON API](GETTING_STARTED.md#routes-and-handlers)

---

## 📚 External Resources

### Official Documentation

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [Tera Documentation](https://keats.github.io/tera/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Tokio Documentation](https://tokio.rs/)

### Inspirations

- [Django](https://www.djangoproject.com/)
- [Actix-Web](https://actix.rs/)
- [Rocket](https://rocket.rs/)

---

## 🤝 Contributing

Want to contribute to Runique? Great!

1. Fork the project
2. Create a branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under MIT.

**See:** [LICENSE-MIT](../../LICENSE-MIT.md)

---

## 📞 Support and Community

- 📖 [Complete documentation](README.md)
- 💬 [GitHub Discussions](https://github.com/seb-alliot/runique/tree/discussions)
- 🐛 [Report a bug](https://github.com/seb-alliot/runique/tree/issues)
- ⭐ [Give a star](https://github.com/seb-alliot/runique)

---

**Developed with ❤️ in Rust by Itsuki**

**Happy coding with Runique! 🦀**

*Documentation created with ❤️ by Claude for Itsuki*
