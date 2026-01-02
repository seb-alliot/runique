# 🚀 Guide de démarrage - Rusti Framework

Ce guide vous accompagne pas à pas dans la création de votre première application Rusti.

## Prérequis

* Rust (dernière version stable recommandée)
* Cargo (installé avec Rust)
* Connaissances de base en Rust et développement web

## Table des matières

1. [Installation](https://www.google.com/search?q=%23installation)
2. [Première application](https://www.google.com/search?q=%23premi%C3%A8re-application)
3. [Structure du projet](https://www.google.com/search?q=%23structure-du-projet)
4. [Routes et handlers](https://www.google.com/search?q=%23routes-et-handlers)
5. [Templates](https://www.google.com/search?q=%23templates)
6. [Fichiers statiques](https://www.google.com/search?q=%23fichiers-statiques)
7. [Base de données](https://www.google.com/search?q=%23base-de-donn%C3%A9es)
8. [Déploiement](https://www.google.com/search?q=%23d%C3%A9ploiement)

---

## Installation

### 1. Installer Rust

Si ce n'est pas déjà fait :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

```

### 2. Créer un nouveau projet

```bash
cargo new mon-app-rusti
cd mon-app-rusti

```

### 3. Ajouter les dépendances

```toml
# Cargo.toml
[dependencies]
rusti = "1.0.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

```

---

## Première application

### Application minimale

Créez `src/main.rs` :

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

Lancez l'application :

```bash
cargo run

```

Ouvrez [http://127.0.0.1:3000](http://127.0.0.1:3000)

🎉 **Félicitations !** Votre première application Rusti fonctionne.

---

## Structure du projet

Pour une application complète, organisez votre code ainsi :

```
mon-app-rusti/
├── src/
│   ├── main.rs          # Point d'entrée
│   ├── urls.rs          # Configuration des routes
│   ├── views.rs         # Handlers (logique métier)
│   └── models.rs        # Modèles de base de données (optionnel)
├── templates/           # Templates Tera
│   ├── base.html        # Template parent
│   └── index.html       # Page d'accueil
├── static/              # Fichiers statiques
│   ├── css/
│   │   └── main.css
├── .env                 # Variables d'environnement
├── Cargo.toml
└── README.md

```

---

## Routes et handlers

### 1. Définir les routes (`src/urls.rs`)

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

### 2. Créer les handlers (`src/views.rs`)

```rust
use rusti::prelude::*;

pub async fn index(template: Template, mut message: Message) -> Response {
    info!(message, "Ceci est un message d'information de test.");
    let ctx = context! {
        "title": "Accueil",
        "description": "Framework web Rust inspiré de Django",
    };
    template.render("index.html", &ctx)
}

pub async fn about(template: Template) -> Response {
    let ctx = context! {
        "title": "À propos",
        "features": vec!["Django-like", "Performant", "Sécurisé", "Moderne"],
    };
    template.render("about.html", &ctx)
}

pub async fn api_users() -> Response {
    let users = json!({"users": [{"id": 1, "name": "Alice"}]});
    (StatusCode::OK, Json(users)).into_response()
}

pub async fn user_detail(Path(id): Path<u32>, template: Template) -> Response {
    let ctx = context! { "user_id": id, "title": format!("Utilisateur #{}", id) };
    template.render("user_detail.html", &ctx)
}

```

### 3. Mettre à jour `main.rs`

```rust
use rusti::prelude::*;
use std::env; // Import nécessaire pour env::var

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .staticfiles_dirs("static")
        .media_root("media")
        .server("127.0.0.1", 3000, "changez-cette-clef-en-production")
        .build();

    RustiApp::new(settings).await?
        .routes(urls::routes()) // Corrigé : utilise 'urls' au pluriel
        .with_static_files()?
        .with_allowed_hosts(
            env::var("ALLOWED_HOSTS")
                .ok()
                .map(|s| s.split(',').map(|h| h.to_string()).collect()),
        )
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}

```

---

## Templates

### Template de base (`templates/base.html`)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}Mon App Rusti{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <nav>
        <a href='{% link "home" %}'>Accueil</a>
        <a href='{% link "about" %}'>À propos</a>
    </nav>

    <main>
        {% messages %}
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>&copy; 2026 Mon Application Rusti</p> </footer>
</body>
</html>

```

---

## Base de données

### Utiliser dans les handlers

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

pub async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let users = User::objects
        .order_by_desc(users::Column::CreatedAt)
        .limit(10)
        .all(&db)
        .await
        .unwrap_or_default();

    let context = context! { "users": users };
    template.render("users/list.html", &context)
}

```

---

## Prochaines étapes

✅ Votre application Rusti est prête !

* 📖 [Documentation des templates](https://www.google.com/search?q=TEMPLATES.md)
* 🗄️ [Guide de la base de données](DATABASE.md)

**Bon développement avec Rusti ! 🦀**
