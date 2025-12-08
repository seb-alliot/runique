# 🎨 Templates de projet Rusti

Ce document contient des templates prêts à l'emploi pour différents types de projets.

## 📄 Template minimal

### main.rs
```rust
use rusti::{RustiApp, Settings, Router, routing::get};

async fn index() -> &'static str {
    "Hello, Rusti!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();
    
    RustiApp::new(settings).await?
        .routes(Router::new().route("/", get(index)))
        .run()
        .await?;
    
    Ok(())
}
```

### Cargo.toml
```toml
[package]
name = "mon-app"
version = "0.1.0"
edition = "2021"

[dependencies]
rusti = "0.1"
tokio = { version = "1", features = ["full"] }
```

---

## 🌐 Template avec templates HTML

### Structure
```
src/
  ├── main.rs
  └── views.rs
templates/
  ├── base.html
  ├── index.html
  └── errors/
      ├── 404.html
      └── 500.html
static/
  └── css/
      └── main.css
```

### main.rs
```rust
use rusti::{RustiApp, Settings, Router, routing::get};

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::builder()
        .debug(cfg!(debug_assertions))
        .templates_dir("templates")
        .static_dir("static")
        .server("127.0.0.1", 3000)
        .build();

    let routes = Router::new()
        .route("/", get(views::index));

    RustiApp::new(settings).await?
        .routes(routes)
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### views.rs
```rust
use rusti::{
    Extension, Response, StatusCode,
    Context, Tera, Settings,
};
use rusti::middleware::TeraSafe;
use std::sync::Arc;
use serde_json::json;

pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Accueil",
    })).unwrap_or_default();

    tera.render_safe("index.html", &context, StatusCode::OK, &config)
}
```

### templates/base.html
```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/main.css">
</head>
<body>
    <nav>
        <a href="/">Accueil</a>
    </nav>
    
    <main>
        {% block content %}{% endblock %}
    </main>
    
    <footer>
        <p>&copy; 2024 Mon Application</p>
    </footer>
</body>
</html>
```

### templates/index.html
```html
{% extends "base.html" %}

{% block title %}{{ title }}{% endblock %}

{% block content %}
<h1>{{ title }}</h1>
<p>Bienvenue sur mon application Rusti !</p>
{% endblock %}
```

---

## 🗄️ Template avec base de données

### Cargo.toml
```toml
[dependencies]
rusti = { version = "0.1", features = ["orm"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sea-orm = { version = "2.0.0-rc.18", features = ["sqlx-postgres", "runtime-tokio", "macros"] }
```

### main.rs
```rust
use rusti::{RustiApp, Settings, Router, routing::get};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

mod views;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::from_env();

    let routes = Router::new()
        .route("/", get(views::index))
        .route("/users", get(views::list_users));

    RustiApp::new(settings).await?
        .with_database().await?
        .routes(routes)
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### views.rs
```rust
use rusti::{
    Extension, Response, StatusCode,
    Context, Tera, Settings, DatabaseConnection,
};
use rusti::middleware::TeraSafe;
use std::sync::Arc;
use serde_json::json;
use sea_orm::*;

pub async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    // Exemple de requête SeaORM
    // let users = User::find().all(db.as_ref()).await.unwrap_or_default();
    
    let context = Context::from_serialize(json!({
        "title": "Utilisateurs",
        "users": vec!["User 1", "User 2"],
    })).unwrap_or_default();

    tera.render_safe("users.html", &context, StatusCode::OK, &config)
}
```

### .env
```env
IP_SERVER=127.0.0.1
PORT=3000

DB_ENGINE=postgres
POSTGRES_USER=myuser
POSTGRES_PASSWORD=mypassword
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=mydb
```

---

## 🔌 Template API REST

### main.rs
```rust
use rusti::{RustiApp, Settings, Router, routing::{get, post, put, delete}};

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::default_values();

    let routes = Router::new()
        .route("/api/status", get(api::status))
        .route("/api/users", get(api::list_users))
        .route("/api/users", post(api::create_user))
        .route("/api/users/:id", get(api::get_user))
        .route("/api/users/:id", put(api::update_user))
        .route("/api/users/:id", delete(api::delete_user));

    RustiApp::new(settings).await?
        .routes(routes)
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### api.rs
```rust
use rusti::{Response, StatusCode, Path};
use rusti::response::json_response;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

pub async fn status() -> Response {
    json_response(
        StatusCode::OK,
        json!({
            "status": "ok",
            "version": "1.0.0"
        })
    )
}

pub async fn list_users() -> Response {
    let users = vec![
        User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
        User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
    ];
    
    json_response(StatusCode::OK, json!(users))
}

pub async fn get_user(Path(id): Path<u32>) -> Response {
    json_response(
        StatusCode::OK,
        json!({
            "id": id,
            "name": format!("User {}", id),
            "email": format!("user{}@example.com", id)
        })
    )
}

pub async fn create_user() -> Response {
    json_response(
        StatusCode::CREATED,
        json!({ "message": "User created" })
    )
}

pub async fn update_user(Path(id): Path<u32>) -> Response {
    json_response(
        StatusCode::OK,
        json!({ "message": format!("User {} updated", id) })
    )
}

pub async fn delete_user(Path(id): Path<u32>) -> Response {
    json_response(
        StatusCode::OK,
        json!({ "message": format!("User {} deleted", id) })
    )
}
```

---

## 🎯 Template fullstack (Web + API)

Combine les templates HTML et API avec une structure modulaire :

```
src/
  ├── main.rs
  ├── web/
  │   ├── mod.rs
  │   └── views.rs
  ├── api/
  │   ├── mod.rs
  │   └── handlers.rs
  └── models/
      └── mod.rs
templates/
static/
```

### main.rs
```rust
use rusti::{RustiApp, Settings, Router, routing::{get, post}};

mod web;
mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    let routes = Router::new()
        // Routes web
        .route("/", get(web::views::index))
        .route("/dashboard", get(web::views::dashboard))
        // Routes API
        .route("/api/data", get(api::handlers::get_data))
        .route("/api/data", post(api::handlers::post_data));

    RustiApp::new(settings).await?
        .routes(routes)
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## 🚀 Commandes utiles

```bash
# Créer un nouveau projet
cargo new mon-app
cd mon-app

# Ajouter rusti
cargo add rusti
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json

# Créer la structure
mkdir -p templates/errors static/css media

# Lancer en mode dev
cargo watch -x run

# Build optimisé
cargo build --release

# Lancer les tests
cargo test
```

## 📦 Publier sur crates.io

Une fois le framework prêt :

```bash
cd rusti
cargo publish --dry-run
cargo publish
```

---

Choisissez le template qui correspond à vos besoins et commencez à coder ! 🎉
