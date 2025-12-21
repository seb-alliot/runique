# 🚀 Getting Started - Rusti Framework

This guide will walk you through creating your first Rusti application step by step.

## Prerequisites

- Rust 1.70 or higher
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

If not already done:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
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
rusti = "0.1"
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

For a complete application, organize your code as follows:

```
my-rusti-app/
├── src/
│   ├── main.rs          # Entry point
│   ├── urls.rs          # Route configuration
│   ├── views.rs         # Handlers (business logic)
│   └── models.rs        # Database models (optional)
├── templates/           # Tera templates
│   ├── base.html        # Parent template
│   ├── index.html       # Homepage
│   └── errors/          # Custom error pages
│       ├── 404.html
│       └── 500.html
├── static/              # Static files
│   ├── css/
│   │   └── main.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
├── media/               # User uploaded files
├── .env                 # Environment variables
├── .env.example         # Configuration example
├── Cargo.toml
└── README.md
```

### Create the Structure

```bash
mkdir -p templates/errors static/{css,js,images} media
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
    let _ = message.info("Welcome to Rusti!").await;
    
    let context = Context::from_serialize(json!({
        "title": "Home",
        "description": "Rust web framework inspired by Django",
    })).unwrap_or_default();

    template.render("index.html", &context)
}

// About page
pub async fn about(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "About",
        "features": vec![
            "Django-like",
            "Performant",
            "Secure",
            "Modern"
        ],
    })).unwrap_or_default();
    
    template.render("about.html", &context)
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
    let context = Context::from_serialize(json!({
        "user_id": id,
        "title": format!("User #{}", id),
    })).unwrap_or_default();
    
    template.render("user_detail.html", &context)
}
```

### 3. Update `main.rs`

```rust
use rusti::prelude::*;

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

    // Launch application
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
            <p>&copy; 2025 My Rusti Application</p>
        </div>
    </footer>

    {% block extra_js %}{% endblock %}
</body>
</html>
```

See [TEMPLATES.md](TEMPLATES.md) for complete template documentation.

---

## Database

### 1. PostgreSQL Configuration

Add the feature in `Cargo.toml`:

```toml
[dependencies]
rusti = { version = "0.1", features = ["postgres"] }
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
```

Create `.env`:

```env
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
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

### 3. Use in Handlers

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

pub async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Django-like query
    let users = User::objects
        .order_by_desc(users::Column::CreatedAt)
        .limit(10)
        .all(&db)
        .await
        .unwrap_or_default();
    
    let context = Context::from_serialize(json!({
        "users": users,
    })).unwrap_or_default();
    
    template.render("users/list.html", &context)
}
```

See [DATABASE.md](DATABASE.md) for complete database documentation.

---

## Deployment

### Production Build

```bash
cargo build --release
```

The executable will be in `target/release/my-rusti-app`

### Production Environment Variables

```env
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=your-super-secret-and-long-key
DB_ENGINE=postgres
DB_URL=postgresql://user:pass@host:5432/dbname
```

### nginx Configuration (reverse proxy)

```nginx
server {
    listen 80;
    server_name myapp.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /static {
        alias /path/to/static;
    }

    location /media {
        alias /path/to/media;
    }
}
```

---

## Next Steps

✅ You now have a working Rusti application!

To go further:

- 📖 [Template Documentation](TEMPLATES.md)
- 🗄️ [Database Guide](DATABASE.md)
- 🔧 [Advanced Configuration](CONFIGURATION.md)

**Happy coding with Rusti! 🦀**
