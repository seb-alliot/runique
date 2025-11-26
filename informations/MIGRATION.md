# 📦 Guide de migration - De ton code actuel vers Rusti Framework

Ce guide t'aide à migrer ton code existant vers la structure du framework Rusti.

## Comparaison des structures

### Avant (ton code actuel)

```
src/
├── main.rs
├── path.rs
├── rust_view.rs
├── settings.rs
└── utils/
    ├── db/
    ├── middleware/
    ├── server/
    └── struct_config/
```

### Après (avec Rusti)

```
mon-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── views.rs
└── templates/
```

Le framework Rusti encapsule toute la logique dans la bibliothèque !

## Étape 1 : Créer le workspace

```bash
# À la racine de ton projet
mkdir rusti-app
cd rusti-app
```

Copie le contenu de `/home/claude/rusti-framework/` dans ce répertoire.

## Étape 2 : Migration du main.rs

### Avant

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = settings::Settings::default_values();
    let db: DatabaseConnection = utils::db::pool::connected_db::connect_db(&config).await?;
    let _ = utils::server::server::runserver(Arc::new(db), Arc::new(config)).await?;
    Ok(())
}
```

### Après

```rust
use rusti::{RustiApp, Settings, Router, routing::get};

mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let settings = Settings::default_values();
    
    let routes = Router::new()
        .route("/", get(views::index));
    
    RustiApp::new(settings).await?
        .routes(routes)
        .with_database().await?
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;
    
    Ok(())
}
```

## Étape 3 : Migration des views

### Avant (rust_view.rs)

```rust
pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue",
        "debug": config.debug,
    })).unwrap_or_default();

    return_render(&tera, "index1.html", &context, StatusCode::OK, &config)
}
```

### Après (views.rs)

```rust
use rusti::{Extension, Response, StatusCode, Context, Tera, Settings};
use rusti::middleware::TeraSafe;
use std::sync::Arc;
use serde_json::json;

pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue",
        "debug": config.debug,
    })).unwrap_or_default();

    tera.render_safe("index.html", &context, StatusCode::OK, &config)
}
```

**Changements :**
- `return_render()` → `tera.render_safe()`
- Import de `TeraSafe` trait
- Imports simplifiés via `rusti::`

## Étape 4 : Migration du routing

### Avant (path.rs)

```rust
pub fn path_url(
    static_files: ServeDir,
    media_files: ServeDir,
    config: Arc<Settings>,
    db: Arc<DatabaseConnection>,
    tera: Arc<Tera>,
) -> Router {
    Router::new()
        .route("/", get(rust_view::index))
        .nest_service(&static_url, static_files)
        .nest_service(&media_url, media_files)
        .fallback(|| async move {
            render_simple_404(&tera_fallback)
        })
        .with_state(db)
        .layer(middleware::from_fn(error_handler_middleware))
        .layer(Extension(config))
        .layer(Extension(tera))
}
```

### Après (main.rs)

```rust
let routes = Router::new()
    .route("/", get(views::index))
    .route("/about", get(views::about));

RustiApp::new(settings).await?
    .routes(routes)
    .with_static_files()?  // Gère automatiquement les fichiers statiques
    .with_database().await?  // Configure la DB
    .with_sessions()  // Configure les sessions
    .with_default_middleware()  // Middleware d'erreur, timeouts, etc.
    .run()
    .await?;
```

**Avantages :**
- Pas besoin de gérer manuellement les layers
- Configuration automatique des fichiers statiques
- Middleware d'erreur intégré
- Ordre des layers correct par défaut

## Étape 5 : Migration des templates

Tes templates restent identiques ! Juste déplace-les :

```bash
# Avant
src/utils/frontend/templates/

# Après
templates/
```

Même chose pour les fichiers statiques et média :

```bash
# Avant
src/utils/frontend/static/
src/utils/frontend/media/

# Après
static/
media/
```

## Étape 6 : Migration de la configuration

### Avant (settings.rs)

```rust
impl Settings {
    pub fn default_values() -> Self {
        let base_dir = "src";
        let templates_dir = format!("{}/utils/frontend/templates", base_dir);
        // ... beaucoup de code ...
    }
}
```

### Après

```rust
// Option 1 : Défauts simples
let settings = Settings::default_values();

// Option 2 : Builder pour personnaliser
let settings = Settings::builder()
    .debug(true)
    .templates_dir("templates")
    .static_dir("static")
    .media_dir("media")
    .server("127.0.0.1", 3000)
    .build();

// Option 3 : Variables d'environnement
let settings = Settings::from_env();
```

## Étape 7 : Migration du Cargo.toml

### Avant

```toml
[dependencies]
axum = { version = "0.8.6", features = ["macros", "form"] }
tower-sessions = "0.14.0"
tower-http = { version = "0.6", features = ["fs", "util", "trace", "timeout"] }
# ... beaucoup de dépendances ...
```

### Après

```toml
[dependencies]
rusti = { path = "../rusti" }  # ou version = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Avantages :**
- Une seule dépendance principale
- Pas besoin de gérer les versions compatibles
- Features optionnelles (orm, etc.)

## Checklist de migration

- [ ] Copier les fichiers du framework dans un nouveau dossier
- [ ] Créer `examples/demo-app` ou ton app
- [ ] Migrer `main.rs` vers la nouvelle structure
- [ ] Migrer les views (renommer `rust_view.rs` → `views.rs`)
- [ ] Déplacer les templates vers `templates/`
- [ ] Déplacer les fichiers statiques vers `static/`
- [ ] Simplifier `Cargo.toml`
- [ ] Créer `.env` si nécessaire
- [ ] Tester avec `cargo run`

## Points d'attention

### 1. Chemins des templates

```rust
// Avant
"index1.html"

// Après - vérifie le nom
"index.html"
```

### 2. Gestion d'erreur

```rust
// Avant
return_render(&tera, template, context, status, config)

// Après
tera.render_safe(template, context, status, config)
```

### 3. Base de données

```rust
// Avant
let db = connect_db(&config).await?;
// Puis passer db partout

// Après
RustiApp::new(settings).await?
    .with_database().await?  // DB automatiquement dans Extension
```

### 4. Accès à la DB dans les handlers

```rust
// Reste identique !
pub async fn my_handler(
    Extension(db): Extension<Arc<DatabaseConnection>>,
) -> Response {
    // Utilise db normalement
}
```

## Nouveautés disponibles

### Macro routes!

```rust
use rusti::routes;

let router = routes![
    "/" => get(views::index),
    "/about" => get(views::about),
];
```

### Helpers de réponse

```rust
use rusti::response::{json_response, html_response, redirect};

// JSON
return json_response(StatusCode::OK, json!({"status": "ok"}));

// HTML simple
return html_response(StatusCode::OK, "<h1>Hello</h1>");

// Redirection
return redirect("/login");
```

### Extension trait TeraSafe

```rust
use rusti::middleware::TeraSafe;

// Maintenant tu peux faire
tera.render_safe(template, context, status, config)

// Au lieu de
return_render(&tera, template, context, status, config)
```

## Besoin d'aide ?

Si tu rencontres des problèmes pendant la migration :

1. Consulte le `README.md` complet
2. Regarde l'exemple dans `examples/demo-app`
3. Vérifie le `QUICKSTART.md`
4. Ouvre une issue sur GitHub

## Prochaines étapes après la migration

1. **Ajouter des tests** - Le framework facilite les tests unitaires
2. **Personnaliser les templates d'erreur** - Crée tes propres 404/500
3. **Ajouter du middleware custom** - Étends le framework
4. **Utiliser les features** - Active/désactive ORM selon tes besoins

Bon courage ! 🚀
