# 🚀 Guide de démarrage - Rusti Framework

Ce guide vous accompagne pas à pas dans la création de votre première application Rusti.

## Prérequis

- Rust 1.70 ou supérieur
- Cargo (installé avec Rust)
- Connaissances de base en Rust et développement web

## Table des matières

1. [Installation](#installation)
2. [Première application](#première-application)
3. [Structure du projet](#structure-du-projet)
4. [Routes et handlers](#routes-et-handlers)
5. [Templates](#templates)
6. [Fichiers statiques](#fichiers-statiques)
7. [Base de données](#base-de-données)
8. [Déploiement](#déploiement)

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
rusti = "0.1"
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

Ouvrez http://127.0.0.1:3000

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
│   ├── index.html       # Page d'accueil
│   └── errors/          # Pages d'erreur personnalisées
│       ├── 404.html
│       └── 500.html
├── static/              # Fichiers statiques
│   ├── css/
│   │   └── main.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
├── media/               # Fichiers uploadés par les utilisateurs
├── .env                 # Variables d'environnement
├── .env.example         # Exemple de configuration
├── Cargo.toml
└── README.md
```

### Créer la structure

```bash
mkdir -p templates/errors static/{css,js,images} media
touch src/{urls.rs,views.rs}
touch templates/{base.html,index.html}
touch static/css/main.css
touch .env.example
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

// Page d'accueil
pub async fn index(
    template: Template,
    mut message: Message,
) -> Response {
    let _ = message.info("Bienvenue sur Rusti !").await;
    
    let context = Context::from_serialize(json!({
        "title": "Accueil",
        "description": "Framework web Rust inspiré de Django",
    })).unwrap_or_default();

    template.render("index.html", &context)
}

// Page à propos
pub async fn about(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "À propos",
        "features": vec![
            "Django-like",
            "Performant",
            "Sécurisé",
            "Moderne"
        ],
    })).unwrap_or_default();
    
    template.render("about.html", &context)
}

// Page de contact
pub async fn contact(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Contact",
    })).unwrap_or_default();
    
    template.render("contact.html", &context)
}

// API JSON
pub async fn api_users() -> Response {
    let users = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
        ]
    });
    
    (StatusCode::OK, Json(users)).into_response()
}

// Détail utilisateur avec paramètre
pub async fn user_detail(
    Path(id): Path<u32>,
    template: Template,
) -> Response {
    let context = Context::from_serialize(json!({
        "user_id": id,
        "title": format!("Utilisateur #{}", id),
    })).unwrap_or_default();
    
    template.render("user_detail.html", &context)
}
```

### 3. Mettre à jour `main.rs`

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
        .server("127.0.0.1", 3000, "changez-cette-clef-en-production")
        .build();

    // Lancement de l'application
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

### Template de base (`templates/base.html`)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App Rusti{% endblock %}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
    {% block extra_css %}{% endblock %}
</head>
<body>
    <nav class="navbar">
        <div class="container">
            <a href='{% link "home" %}' class="logo">🦀 Rusti</a>
            <ul class="nav-links">
                <li><a href='{% link "home" %}'>Accueil</a></li>
                <li><a href='{% link "about" %}'>À propos</a></li>
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
            <p>&copy; 2025 Mon Application Rusti</p>
        </div>
    </footer>

    {% block extra_js %}{% endblock %}
</body>
</html>
```

### Page d'accueil (`templates/index.html`)

```html
{% extends "base.html" %}

{% block title %}{{ title }} - Mon App{% endblock %}

{% block content %}
<div class="hero">
    <h1>{{ title }}</h1>
    <p class="lead">{{ description }}</p>
    
    <div class="cta-buttons">
        <a href='{% link "about" %}' class="btn btn-primary">En savoir plus</a>
        <a href='{% link "contact" %}' class="btn btn-secondary">Nous contacter</a>
    </div>
</div>

<section class="features">
    <h2>Pourquoi choisir Rusti ?</h2>
    <div class="feature-grid">
        <div class="feature">
            <h3>🚀 Performant</h3>
            <p>Construit sur Axum et Tokio</p>
        </div>
        <div class="feature">
            <h3>🛡️ Sécurisé</h3>
            <p>CSRF, sessions, validation intégrés</p>
        </div>
        <div class="feature">
            <h3>📝 Moderne</h3>
            <p>Templates puissants avec Tera</p>
        </div>
    </div>
</section>
{% endblock %}
```

---

## Fichiers statiques

### CSS de base (`static/css/main.css`)

```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Navigation */
.navbar {
    background: #667eea;
    padding: 1rem 0;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.navbar .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.logo {
    color: white;
    text-decoration: none;
    font-size: 1.5rem;
    font-weight: bold;
}

.nav-links {
    display: flex;
    list-style: none;
    gap: 2rem;
}

.nav-links a {
    color: white;
    text-decoration: none;
    transition: opacity 0.3s;
}

.nav-links a:hover {
    opacity: 0.8;
}

/* Main content */
main {
    min-height: 60vh;
    padding: 2rem 0;
}

.hero {
    text-align: center;
    padding: 4rem 0;
}

.hero h1 {
    font-size: 3rem;
    margin-bottom: 1rem;
    color: #667eea;
}

.lead {
    font-size: 1.25rem;
    color: #666;
    margin-bottom: 2rem;
}

/* Buttons */
.btn {
    display: inline-block;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    text-decoration: none;
    font-weight: 600;
    transition: transform 0.2s;
}

.btn:hover {
    transform: translateY(-2px);
}

.btn-primary {
    background: #667eea;
    color: white;
}

.btn-secondary {
    background: #764ba2;
    color: white;
}

/* Flash messages */
.flash-messages {
    margin: 1rem 0;
}

.message {
    padding: 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
}

.message-success {
    background: #d4edda;
    color: #155724;
    border-left: 4px solid #28a745;
}

.message-error {
    background: #f8d7da;
    color: #721c24;
    border-left: 4px solid #dc3545;
}

.message-info {
    background: #d1ecf1;
    color: #0c5460;
    border-left: 4px solid #17a2b8;
}

/* Footer */
.footer {
    background: #f8f9fa;
    padding: 2rem 0;
    margin-top: 4rem;
    text-align: center;
    color: #666;
}
```

---

## Base de données

### 1. Configuration PostgreSQL

Ajoutez la feature dans `Cargo.toml` :

```toml
[dependencies]
rusti = { version = "0.1", features = ["postgres"] }
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls"] }
```

Créez `.env` :

```env
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

### 2. Définir un modèle (`src/models.rs`)

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

// Activer l'API Django-like
impl_objects!(Entity);
```

### 3. Utiliser dans les handlers

```rust
use rusti::prelude::*;
use crate::models::{users, Entity as User};

pub async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Query Django-like
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

### 4. Connecter la base de données

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    
    // Configuration DB
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    RustiApp::new(settings).await?
        .with_database(db)
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

## Déploiement

### Build de production

```bash
cargo build --release
```

L'exécutable sera dans `target/release/mon-app-rusti`

### Variables d'environnement en production

```env
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=votre-clef-super-secrete-et-longue
DB_ENGINE=postgres
DB_URL=postgresql://user:pass@host:5432/dbname
```

### Configuration nginx (reverse proxy)

```nginx
server {
    listen 80;
    server_name monapp.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /static {
        alias /chemin/vers/static;
    }

    location /media {
        alias /chemin/vers/media;
    }
}
```

---

## Prochaines étapes

✅ Vous avez maintenant une application Rusti fonctionnelle !

Pour aller plus loin :

- 📖 [Documentation des templates](TEMPLATES.md)
- 🗄️ [Guide de la base de données](DATABASE.md)
- 🔧 [Configuration avancée](CONFIGURATION.md)
- 🎨 [Référence API complète](API.md)

**Bon développement avec Rusti ! 🦀**
