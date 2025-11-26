# 🚀 Guide de démarrage rapide - Rusti Framework

## Installation

### 1. Créer un nouveau projet

```bash
cargo new mon-app
cd mon-app
```

### 2. Ajouter Rusti

Dans `Cargo.toml` :

```toml
[dependencies]
rusti = { path = "../rusti" }  # ou version = "0.1" quand publié
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 3. Structure des dossiers

```bash
mkdir -p templates static/css media
```

## Premier handler

### src/main.rs

```rust
use rusti::{RustiApp, Settings, Router, routing::get};

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Configuration
    let settings = Settings::builder()
        .debug(true)
        .templates_dir("templates")
        .static_dir("static")
        .media_dir("media")
        .server("127.0.0.1", 3000)
        .build();

    // Routes
    let routes = Router::new()
        .route("/", get(views::index));

    // Lancer l'app
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

### src/views.rs

```rust
use rusti::{
    Extension,
    Response,
    StatusCode,
    Context,
    Tera,
    Settings,
};
use rusti::middleware::TeraSafe;
use std::sync::Arc;
use serde_json::json;

pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue",
        "message": "Hello from Rusti!",
    })).unwrap_or_default();

    tera.render_safe("index.html", &context, StatusCode::OK, &config)
}
```

### templates/index.html

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href="/static/css/main.css">
</head>
<body>
    <h1>{{ title }}</h1>
    <p>{{ message }}</p>
</body>
</html>
```

### static/css/main.css

```css
body {
    font-family: Arial, sans-serif;
    max-width: 800px;
    margin: 50px auto;
    padding: 20px;
}

h1 {
    color: #667eea;
}
```

## Lancer l'application

```bash
cargo run
```

Ouvrez http://localhost:3000

## Prochaines étapes

1. **Ajouter des routes** : Consultez la section Routing du README
2. **Utiliser la base de données** : Activez la feature `orm`
3. **Personnaliser les templates d'erreur** : Créez `templates/errors/404.html` et `500.html`
4. **Ajouter du middleware** : Consultez la section Middleware du README

## Exemples de routes

```rust
use rusti::{Router, routing::{get, post, put, delete}};

let routes = Router::new()
    // Pages
    .route("/", get(views::index))
    .route("/about", get(views::about))
    
    // API
    .route("/api/users", get(api::list_users))
    .route("/api/users", post(api::create_user))
    .route("/api/users/:id", get(api::get_user))
    .route("/api/users/:id", put(api::update_user))
    .route("/api/users/:id", delete(api::delete_user));
```

## Handler avec paramètres

```rust
use rusti::{Extension, Response, StatusCode, Path, Context, Tera, Settings};
use rusti::middleware::TeraSafe;
use std::sync::Arc;
use serde_json::json;

pub async fn user_detail(
    Path(user_id): Path<u32>,
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "user_id": user_id,
        "username": format!("user_{}", user_id),
    })).unwrap_or_default();

    tera.render_safe("user.html", &context, StatusCode::OK, &config)
}
```

## JSON Response

```rust
use rusti::{Response, StatusCode};
use rusti::response::json_response;
use serde_json::json;

pub async fn api_status() -> Response {
    json_response(
        StatusCode::OK,
        json!({
            "status": "ok",
            "version": "1.0.0"
        })
    )
}
```

## Avec la base de données

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();
    
    RustiApp::new(settings).await?
        .with_database().await?  // Ajoute la connexion DB
        .routes(routes)
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;
    
    Ok(())
}
```

Dans vos handlers, vous aurez accès à la DB :

```rust
use rusti::{Extension, DatabaseConnection};
use std::sync::Arc;

pub async fn my_handler(
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    // Utilisez SeaORM ici
    // ...
}
```

## Besoin d'aide ?

- 📖 Consultez le README complet
- 🔍 Regardez les exemples dans `examples/demo-app`
- 💬 Ouvrez une issue sur GitHub
