# 🦀 Rusti Framework

> Un framework web moderne pour Rust, inspiré de Django et construit sur Axum

[![Version](https://img.shields.io/badge/version-1.0-blue.svg)](https://github.com/seb-alliot/rusti)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## ✨ Pourquoi Rusti ?

Rusti combine la **familiarité de Django** avec les **performances de Rust**. Si vous connaissez Django, vous vous sentirez immédiatement chez vous.

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

## 🚀 Caractéristiques principales

| Fonctionnalité | Description |
|----------------|-------------|
|  **Django-like** | Syntaxe familière, patterns éprouvés |
|  **Performances** | Basé sur Axum et Tokio |
|  **Sécurité intégrée** | CSRF, sessions, validation |
|  **Templates Tera** | Moteur inspiré de Jinja2 |
|  **ORM SeaORM** | Support multi-bases (PostgreSQL, MySQL, SQLite) |
|  **Configuration flexible** | Builder pattern + variables d'environnement |
|  **Debug avancé** | Pages d'erreur détaillées en développement |
|  **Flash messages** | Messages entre requêtes |
|  **Reverse routing** | URLs générées automatiquement |

##  Installation

```toml
[dependencies]
rusti = "1.0"
tokio = { version = "1", features = ["full"] }
```

### Choix de la base de données

```toml
# SQLite (par défaut)
rusti = "0.1"

# PostgreSQL
rusti = { version = "0.1", features = ["postgres"] }

# MySQL / MariaDB
rusti = { version = "0.1", features = ["mysql"] }

# Toutes les bases
rusti = { version = "0.1", features = ["all-databases"] }
```

##  Guide de démarrage rapide

### 1. Créer votre projet

```bash
cargo new mon-app
cd mon-app
cargo add rusti tokio --features full
```

### 2. Structure recommandée

```
mon-app/
├── src/
│   ├── main.rs          # Point d'entrée
│   ├── urls.rs          # Routes
│   └── views.rs         # Handlers
├── templates/           # Templates Tera
│   └── index.html
├── static/              # CSS, JS, images
│   └── css/
│       └── main.css
├── media/               # Fichiers uploadés
└── .env                 # Configuration
```

### 3. Code minimal (`src/main.rs`)

```rust
use rusti::prelude::*;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "votre-clef-secrete")
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

### 4. Définir vos routes (`src/urls.rs`)

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

### 5. Créer vos handlers (`src/views.rs`)

```rust
use rusti::prelude::*;

pub async fn index(
    template: Template,
    mut message: Message,
) -> Response {
    let _ = message.success("Bienvenue sur Rusti !").await;
    
    let context = Context::from_serialize(json!({
        "title": "Accueil",
        "items": vec!["Rust", "Django", "Axum"],
    })).unwrap_or_default();

    template.render("index.html", &context)
}

pub async fn about(template: Template) -> Response {
    let context = Context::from_serialize(json!({
        "title": "À propos",
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

### 6. Template de base (`templates/index.html`)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href='{% static "css/main.css" %}'>
</head>
<body>
    <nav>
        <a href='{% link "home" %}'>Accueil</a>
        <a href='{% link "about" %}'>À propos</a>
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

### 7. Lancer l'application

```bash
cargo run
```

Ouvrez http://127.0.0.1:3000 🎉

## 📚 Documentation complète

- **[Guide de démarrage](docs/GETTING_STARTED.md)** - Tutorial complet pas à pas
- **[Templates & Balises](docs/TEMPLATES.md)** - Système de templates personnalisé
- **[Base de données](docs/DATABASE.md)** - Configuration et ORM Django-like
- **[Configuration](docs/CONFIGURATION.md)** - Settings et variables d'environnement
- **[Référence API](docs/API.md)** - Documentation complète des types et fonctions

## 🎨 Fonctionnalités avancées

### ORM Django-like avec SeaORM

```rust
use rusti::prelude::*;

// Définir votre modèle
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub age: i32,
}

// Activer l'API Django-like
impl_objects!(Entity);

// Utiliser comme Django !
pub async fn list_users(db: Extension<Arc<DatabaseConnection>>) -> Response {
    // Récupérer tous les adultes, triés par âge
    let adults = Entity::objects
        .filter(user::Column::Age.gte(18))
        .order_by_desc(user::Column::Age)
        .all(&db)
        .await?;
    
    // Query complexe avec chaînage
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
    // ... logique de création ...
    
    let _ = message.success("Article créé avec succès !").await;
    let _ = message.info("N'oubliez pas de le publier").await;
    let _ = message.error("Erreur lors de l'upload").await;
    
    redirect("/posts")
}
```

### Protection CSRF automatique

```html
<form method="post" action="/submit">
    {% csrf %}
    <input type="text" name="title">
    <button type="submit">Envoyer</button>
</form>
```

### Reverse Routing

```html
<!-- Dans vos templates -->
<a href='{% link "user_profile", id=42, name="alice" %}'>
    Voir le profil
</a>

<!-- Génère automatiquement : /user/42/alice -->
```

```rust
// Dans votre code Rust
use rusti::reverse_with_parameters;

let url = reverse_with_parameters("user_profile", &[
    ("id", "42"),
    ("name", "alice"),
]).unwrap();
```

## 🔧 Configuration avancée

### Avec base de données

```rust
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();
    
    // Configuration de la base de données
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

### Fichier `.env`

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=votre-clef-super-secrete

# Base de données PostgreSQL
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

## 🐛 Pages de debug élégantes

En mode développement, Rusti affiche des pages d'erreur détaillées :

- ✅ Stack trace complète
- ✅ Informations de requête HTTP
- ✅ Source du template avec numéro de ligne
- ✅ Liste des templates disponibles
- ✅ Variables d'environnement
- ✅ Version de Rust utilisée

## 🤝 Comparaison avec Django

| Concept Django | Équivalent Rusti |
|----------------|------------------|
| `settings.py` | `Settings::builder()` |
| `urls.py` | `urlpatterns! { ... }` |
| `views.py` | Handlers Axum |
| `models.py` | SeaORM entities |
| `{% url 'name' %}` | `{% link "name" %}` |
| `{% static 'file' %}` | `{% static "file" %}` |
| `messages.success()` | `message.success().await` |
| `{% csrf_token %}` | `{% csrf %}` |
| `Model.objects.filter()` | `Entity::objects.filter()` |

## 📖 Exemples

Consultez le dossier `examples/` pour des projets complets :

- **`demo-app`** - Application complète avec templates, static files, formulaires
- **`api-rest`** - API JSON avec base de données
- **`blog`** - Blog avec authentification et CRUD

## 🛠️ Développement

```bash
# Cloner le dépôt
git clone https://github.com/votre-repo/rusti
cd rusti

# Builder le framework
cargo build

# Lancer les tests
cargo test

# Générer la documentation
cargo doc --open

# Lancer l'exemple
cd examples/demo-app
cargo run
```

## 📄 Licence

Ce projet est sous double licence MIT / Apache-2.0.

```
Licence MIT

Copyright (c) 2025 Itsuki

L'autorisation est accordée, gratuitement, à toute personne obtenant une copie
de ce logiciel et des fichiers de documentation associés, de traiter le logiciel
sans restriction, y compris les droits d'utiliser, copier, modifier, fusionner,
publier, distribuer, sous-licencier et/ou vendre des copies du logiciel.
```

## 🙏 Remerciements

- Inspiré par [Django](https://www.djangoproject.com/)
- Construit sur [Axum](https://github.com/tokio-rs/axum)
- Templates [Tera](https://github.com/Keats/tera)
- ORM [SeaORM](https://www.sea-ql.org/SeaORM/)

## 📞 Support

- 📖 [Documentation](https://docs.rs/rusti)
- 💬 [Discussions GitHub](https://github.com/votre-repo/rusti/discussions)
- 🐛 [Issues](https://github.com/votre-repo/rusti/issues)

---

**Développé avec ❤️ en Rust**
