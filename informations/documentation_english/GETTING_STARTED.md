# 🚀 Getting Started - Rusti Framework

This guide will walk you through creating your first application with Rusti, step by step.

## Prerequisites

- Rust 1.75 or higher
- Cargo (installed with Rust)
- Basic knowledge of Rust and web development

## Table of Contents

1. [Installation](#installation)
2. [First Application](#first-application)
3. [Project Structure](#project-structure)
4. [Routes and Handlers](#routes-and-handlers)
5. [Templates](#templates)
6. [Static Files](#static-files)
7. [Database](#database)
8. [Deployment](#deployment)

---

## Installation

### 1. Install Rust

If you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh

```

### 2. Create a New Project

```bash
cargo new my-rusti-app
cd my-rusti-app

```

### 3. Add Dependencies

```toml
# Cargo.toml
[dependencies]
rusti = "1.0.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

```

---

## First Application

### Minimal Application

Create `src/main.rs`:

```rust
use rusti::prelude::*;

async fn hello() -> &'static str {
    "Hello, Rusti!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;

    Ok(())
}

```

Run the application:

```bash
cargo run

```

Open http://127.0.0.1:3000

🎉 **Congratulations!** Your first Rusti application is running.

---

## Project Structure

For a full-scale application, organize your code as follows:

```
my-rusti-app/
├── src/
│   ├── main.rs          # Entry point
│   ├── urls.rs          # Route configuration
│   ├── views.rs         # Handlers (business logic)
│   └── models.rs        # Database models (optional)
├── templates/           # Tera templates
│   ├── base.html        # Parent template
│   └── index.html       # Homepage

├── static/              # Static files
│   ├── css/
│   │   └── main.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
├── media/               # User-uploaded files
├── .env                 # Environment variables
├── .env.example         # Configuration example
├── Cargo.toml
└── README.md

```

### Create the Structure

```bash
mkdir -p templates/ static/{css,js,images} media
touch src/{urls.rs,views.rs}
touch templates/{base.html,index.html}
touch static/css/main.css
touch .env.example

```

---

## Routes and Handlers

### 1. Define Routes (`src/urls.rs`)

```rust
use rusti::{Router, urlpatterns};
use crate::views;

pub fn routes() -> Router {
    urlpatterns! {
        "/" => get(views::index), name = "home",
        "/about" => get(views::about), name = "about",
        "/contact" => get(views::contact), name = "contact",
        "/api/users" => get(views::api_users), name = "api_users",
        "/user/{id}" => get(views::user_detail), name = "user_detail",
    }
}

```

### 2. Create Handlers (`src/views.rs`)

```rust
use rusti::prelude::*;

// Homepage
pub async fn index(
    template: Template,
    mut message: Message,
) -> Response {
    info!(message, "This is a test information message.");

    let ctx = context! {
        "title": "Home",
        "description": "A Django-inspired Rust web framework",
    };

    template.render("index.html", &ctx)
}

// About page
pub async fn about(template: Template) -> Response {
    let ctx = context! {
        "title": "About",
        "features": vec![
            "Django-like",
            "High performance",
            "Secure",
            "Modern"
        ],
    };

    template.render("about.html", &ctx)
}

// Contact page
pub async fn contact(template: Template) -> Response {
    let ctx = context! {
        "title": "Contact",
    };

    template.render("contact.html", &ctx)
}

// JSON API
pub async fn api_users() -> Response {
    let users = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
        ]
    });

    (StatusCode::OK, Json(users)).into_response()
}

// User detail with parameter
pub async fn user_detail(
    Path(id): Path<u32>,
    template: Template,
) -> Response {
    let ctx = context! {
        "user_id": id,
        "title": format!("User #{}", id),
    };

    template.render("user_detail.html", &ctx)
}

```

### 3. Update `main.rs`

```rust
use rusti::prelude::*;
use std::env;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .staticfiles_dirs("static")
        .media_root("media")
        .server("127.0.0.1", 3000, "change-this-key-in-production")
        .build();

    // Start application
    RustiApp::new(settings).await?
        .routes(urls::routes())
        .with_static_files()?
        .with_allowed_hosts(
            env::var("ALLOWED_HOSTS")
                .ok()
                .map(|s| s.split(',').map(|h| h.to_string()).collect()),
        )
        .with_sanitize_text_inputs(false)
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}

```

---

## Templates

### Base Template (`templates/base.html`)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}My Rusti App{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <nav class="navbar">
        <div class="container">
            <a href='{% link "home" %}' class="logo">🦀 Rusti</a>
            <ul class="nav-links">
                <li><a href='{% link "home" %}'>Home</a></li>
                <li><a href='{% link "about" %}'>About</a></li>
                <li><a href='{% link "contact" %}'>Contact</a></li>
            </ul>
        </div>
    </nav>

    <main class="container">
        {% messages %}

        {% block content %}
        {% endblock %}
    </main>

    <footer class="footer">
        <div class="container">
            <p>&copy; 2026 My Rusti Application</p>
        </div>
    </footer>

    {% block extra_js %}{% endblock %}
</body>
</html>

```

---

## Database

### 1. PostgreSQL Configuration

Add the feature in `Cargo.toml`:

```toml
[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

```

### 2. Define a Model (`src/models.rs`)

```rust
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Enable Django-like API
impl_objects!(Entity);

```

---

## Next Steps

✅ You now have a functional Rusti application!

To go further:

* 📖 [Template Documentation](https://www.google.com/search?q=TEMPLATES.md)
* 🗄️ [Database Guide](DATABASE.md)
* 🔧 [Advanced Configuration](https://www.google.com/search?q=CONFIGURATION.md)
* 🎨 [Full API Reference](API.md)

**Happy coding with Rusti! 🦀**
