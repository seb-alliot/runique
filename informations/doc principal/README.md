# 🦀 Rusti Framework

Un framework web moderne pour Rust, inspiré de Django et construit sur Axum.

## ✨ Caractéristiques

- **🎯 Inspiré de Django** - Configuration et structure familières pour les développeurs Python
- **⚡ Performant** - Construit sur Axum et Tokio pour des performances exceptionnelles
- **🛡️ Gestion d'erreur sophistiquée** - Pages de debug détaillées en développement
- **📝 Templating avec Tera** - Moteur de template Jinja2-like
- **🗄️ ORM intégré** - Support optionnel de SeaORM
- **🔧 Configuration flexible** - Builder pattern et variables d'environnement
- **🎨 Middleware extensible** - Système de middleware inspiré de Django

## 🚀 Installation

Ajoutez Rusti à votre `Cargo.toml` :

```toml
[dependencies]
rusti = "0.1"
tokio = { version = "1", features = ["full"] }
```

## 📖 Exemple rapide

```rust


// src/main.rs
use rusti::{RustiApp, Settings};

mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Configuration de l'application !!
    // Vous pouvez personnaliser les paramètres du settings ici
    // La clef secrète doit être changée pour la production( secret_key dans le server)
    // elle peux être importé du .env comme toute variable d'environnement
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secrete_key")
        .build();

    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(url::urls())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}

//src/url.rs
use rusti::{Router, urlpatterns};
use crate::views;

pub fn urls() -> Router {
    urlpatterns! {
        "/" => get(views::index), name ="index",
        "/about" => get(views::about), name ="about",
        "/user/{id}/{name}" => get(views::user_profile), name ="user_profile",
    }
}


//src/view.rs
use rusti::{
    Context,
    Message,
    Path,
    Response,
    Template,
    json,
};
/// Page d'accueil
pub async fn index(
    template: Template,
    mut message: Message,

) -> Response {
    message.success( "Ceci est un message de succès de test.").await.unwrap();
    message.info("Ceci est un message d'information de test.").await.unwrap();
    message.error("Ceci est un message d'erreur de test.").await.unwrap();

    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
    })).unwrap_or_default();

    template.render("index.html", &context)
}
```

## Structure du projet

```
my-app/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── url.rs
│   └── views.rs
├── templates/
│   ├── index.html
│   └── errors/
│       ├── 404.html
│       └── 500.html
├── static/
│   └── css/
│       └── main.css
│   └── js/
│       └── main.js
└── media/
```

##  Configuration

### Configuration par défaut

```rust
use rusti::Settings;

let settings = Settings::default_values();
```

### Configuration avec builder

```rust
/* Les variables d'environnement sont acceptées */
let settings = Settings::builder()
    .debug(true)
    .templates_dir("templates")
    .static_dir("static")
    .media_dir("media")
    .server("127.0.0.1", 3000)
    .build();
```

### Variables d'environnement

Créez un fichier `.env` :

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

##  Templating

Rusti utilise Tera pour le templating. Créez vos templates dans le répertoire `templates/` :

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ title }}</title>
</head>
<body>
    <h1>{{ title }}</h1>
    {% for item in items %}
        <p>{{ item }}</p>
    {% endfor %}
</body>
</html>
```

Dans votre handler :

```rust
use rusti::{
    Context,
    Message,
    Path,
    Response,
    Template,
    json,
};
/// Page d'accueil
pub async fn index(
    template: Template,
        mut message: Message,
) -> Response {

    message.success( "Ceci est un exemple d'utilisation.").await.unwrap();
    /*error et info sont disponible aussi afin de varier les utilisations */
    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",

    })).unwrap_or_default();

    template.render("index.html", &context)
}
```

##  Base de données (feature `orm`)

```rust
use rusti::{RustiApp, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RustiApp::new(settings).await?
        .with_database_custom().await?  // Active la connexion DB
        .routes(routes)
        .run()
        .await?;

    Ok(())
}
```

## 🛠️ Middleware personnalisé

```rust
use axum::{middleware::Next, extract::Request, response::Response};

async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Votre logique ici
    let response = next.run(request).await;
    response
}

// Dans votre app
let app = RustiApp::new(settings).await?
    .routes(routes)
    .build()
    .layer(axum::middleware::from_fn(my_middleware));
```

## 🐛 Gestion des erreurs

Rusti offre une gestion d'erreur sophistiquée :

- **Mode développement** : Pages de debug détaillées avec stack trace, informations de requête, etc.
- **Mode production** : Pages d'erreur simples et élégantes

Les templates d'erreur personnalisés doivent être placés dans `templates/errors/` :
- `404.html` - Page non trouvée
- `500.html` - Erreur serveur
- `debug_error.html` - Page de debug (optionnel)

## 📝 Routing

### Méthode standard

```rust
use rusti::{Router, routing::{get, post}};

let router = Router::new()
    .route("/", get(index))
    .route("/user/:id", get(user_detail))
    .route("/api/data", post(create_data));
```

### Avec la macro `routes!`

```rust
use rusti::routes;

let router = routes![
    "/" => get(index),
    "/about" => get(about),
    "/contact" => post(contact),
];
```

## 🔧 Features

- `orm` (défaut) - Active le support SeaORM

```toml
# Sans ORM
rusti = { version = "0.1", default-features = false }
```

## 📦 Exemple complet

Consultez le dossier `examples/demo-app` pour un exemple complet d'application.

Pour lancer l'exemple :

```bash
cd examples/demo-app
cargo run
```

Puis ouvrez http://localhost:3000

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.

## 📄 Licence

MIT OR Apache-2.0

## 🙏 Remerciements

- Inspiré par Django
- Construit sur Axum et Tokio
- Templates Tera
- ORM SeaORM
