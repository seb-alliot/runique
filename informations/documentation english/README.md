# 🦀 Rusti Framework

> A modern web framework for Rust, inspired by Django and built on Axum

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/your-repo/rusti)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
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
rusti = "0.1"
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

## 🎓 Quick Start Guide

### 1. Create Your Project

```bash
cargo new my-app
cd my-app
cargo add rusti tokio --features full
```

### 2. Recommended Structure

```
my-app/
├── src/
│   ├── main.rs          # Entry point
│   ├── urls.rs          # Routes
│   └── views.rs         # Handlers
├── templates/           # Tera templates
│   └── index.html
├── static/              # CSS, JS, images
│   └── css/
│       └── main.css
├── media/               # User uploads
└── .env                 # Configuration
```

### 3. Minimal Code (`src/main.rs`)

```rust
use rusti::prelude::*;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "your-secret-key")
        .build();

    RustiApp::new(settings).await?
        .routes(urls::routes())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### 4. Define Routes (`src/urls.rs`)

```rust
use rusti::{Router, urlpatterns};
use crate::views;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => get(views::index), name = "home",
        "/about" => get(views::about), name = "about",
        "/user/{id}/{name}" => get(views::user_profile), name = "user_profile",
    }
}
```

### 5. Create Handlers (`src/views.rs`)

```rust
use rusti::prelude::*;

pub async fn index(
    template: Template,
    mut message: Message,
) -> Response {
    let _ = message.success("Welcome to Rusti!").await;
    
    let context = Context::from_serialize(json!({
        "title": "Home",
        "items": vec!["Rust", "Django", "Axum"],
    })).unwrap_or_default();

    template.render("index.html", &context)
}

pub async fn about(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "About",
    })).unwrap_or_default();
    
    template.render("about.html", &context)
}

pub async fn user_profile(
    Path((id, name)): Path<(u32, String)>,
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "user_id": id,
        "username": name,
    })).unwrap_or_default();
    
    template.render("user_profile.html", &context)
}
```

### 6. Base Template (`templates/index.html`)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <nav>
        <a href='{% link "home" %}'>Home</a>
        <a href='{% link "about" %}'>About</a>
    </nav>

    {% messages %}

    <main>
        <h1>{{ title }}</h1>
        <ul>
        {% for item in items %}
            <li>{{ item }}</li>
        {% endfor %}
        </ul>
    </main>
</body>
</html>
```

### 7. Run the Application

```bash
cargo run
```

Open http://127.0.0.1:3000 🎉

## 📚 Complete Documentation

- **[Getting Started Guide](docs/GETTING_STARTED.md)** - Complete step-by-step tutorial
- **[Templates & Tags](docs/TEMPLATES.md)** - Custom template system
- **[Database Guide](docs/DATABASE.md)** - Configuration and Django-like ORM
- **[Configuration](docs/CONFIGURATION.md)** - Settings and environment variables
- **[API Reference](docs/API.md)** - Complete types and functions documentation

## 🎨 Advanced Features

### Django-like ORM with SeaORM

```rust
use rusti::prelude::*;

// Define your model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// Enable Django-like API
impl_objects!(Entity);

// Use like Django!
pub async fn list_users(db: Extension<Arc<DatabaseConnection>>) -> Response {
    // Get all adults, sorted by age
    let adults = Entity::objects
        .filter(user::Column::Age.gte(18))
        .order_by_desc(user::Column::Age)
        .all(&db)
        .await?;
    
    // Complex query with chaining
    let recent_active = Entity::objects
        .filter(user::Column::IsActive.eq(true))
        .exclude(user::Column::Email.like("%@banned.com"))
        .order_by_desc(user::Column::CreatedAt)
        .limit(10)
        .all(&db)
        .await?;
}
```

### Flash Messages

```rust
pub async fn create_post(mut message: Message) -> Response {
    // ... creation logic ...
    
    let _ = message.success("Post created successfully!").await;
    let _ = message.info("Don't forget to publish it").await;
    let _ = message.error("Error uploading file").await;
    
    redirect("/posts")
}
```

### Automatic CSRF Protection

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="title">
    <button type="submit">Submit</button>
</form>
```

### Reverse Routing

```html
<!-- In templates -->
<a href='{% link "user_profile", id=42, name="alice" %}'>
    View profile
</a>

<!-- Automatically generates: /user/42/alice -->
```

```rust
// In Rust code
use rusti::reverse_with_parameters;

let url = reverse_with_parameters("user_profile", &[
    ("id", "42"),
    ("name", "alice"),
]).unwrap();
```

## 🔧 Advanced Configuration

### With Database

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    
    // Database configuration
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    RustiApp::new(settings).await?
        .with_database(db)
        .routes(routes())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;
    
    Ok(())
}
```

### `.env` File

```env
# Server
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-super-secret-key

# PostgreSQL Database
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

## 🐛 Elegant Debug Pages

In development mode, Rusti displays detailed error pages:

- ✅ Complete stack trace
- ✅ HTTP request information
- ✅ Template source with line numbers
- ✅ List of available templates
- ✅ Environment variables
- ✅ Rust version used

## 🤝 Django Comparison

| Django Concept | Rusti Equivalent |
|---------------|------------------|
| `settings.py` | `Settings::builder()` |
| `urls.py` | `urlpatterns! { ... }` |
| `views.py` | Axum handlers |
| `models.py` | SeaORM entities |
| `{% url 'name' %}` | `{% link "name" %}` |
| `{% static 'file' %}` | `{% static "file" %}` |
| `messages.success()` | `message.success().await` |
| `{% csrf_token %}` | `{% csrf %}` |
| `Model.objects.filter()` | `Entity::objects.filter()` |

## 📖 Examples

Check the `examples/` directory for complete projects:

- **`demo-app`** - Complete application with templates, static files, forms
- **`rest-api`** - JSON API with database
- **`blog`** - Blog with authentication and CRUD

## 🛠️ Development

```bash
# Clone the repository
git clone https://github.com/your-repo/rusti
cd rusti

# Build the framework
cargo build

# Run tests
cargo test

# Generate documentation
cargo doc --open

# Run the example
cd examples/demo-app
cargo run
```

## 📄 License

This project is dual-licensed under MIT / Apache-2.0.

```
MIT License

Copyright (c) 2025 Itsuki

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files, to deal in the software
without restriction, including the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the software.
```

## 🙏 Acknowledgments

- Inspired by [Django](https://www.djangoproject.com/)
- Built on [Axum](https://github.com/tokio-rs/axum)
- Templates [Tera](https://github.com/Keats/tera)
- ORM [SeaORM](https://www.sea-ql.org/SeaORM/)

## 📞 Support

- 📖 [Documentation](https://docs.rs/rusti)
- 💬 [GitHub Discussions](https://github.com/your-repo/rusti/discussions)
- 🐛 [Issues](https://github.com/your-repo/rusti/issues)

---

**Developed with ❤️ in Rust**
