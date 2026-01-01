# 🦀 Rusti Framework

> Un framework web moderne pour Rust, inspiré de Django et construit sur Axum

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/seb-alliot/rusti)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

## ✨ Pourquoi Rusti ?

Rusti combine **la familiarité de Django** avec **les performances de Rust**. Si vous connaissez Django, vous vous sentirez comme chez vous.

```rust
use rusti::prelude::*;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "your-secret-key")
        .build();
    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(urls::routes())
        .with_database(db)
        .with_static_files()?
        .with_default_middleware()  
        .run()
        .await?;

    Ok(())
}
```

## 🚀 Fonctionnalités principales

| Fonctionnalité | Description |
|----------------|-------------|
| 🎯 **Django-like** | Syntaxe familière, patterns éprouvés |
| ⚡ **Performance** | Construit sur Axum et Tokio |
| 🛡️ **Sécurité intégrée** | CSRF, sessions, validation |
| 📝 **Templates Tera** | Moteur inspiré de Jinja2 |
| 🗄️ **ORM SeaORM** | Support multi-bases (PostgreSQL, MySQL, SQLite) |
| 🔧 **Configuration flexible** | Builder pattern + variables d'environnement |
| 🐛 **Debug avancé** | Pages d'erreur détaillées en développement |
| 📨 **Messages Flash** | Messages entre les requêtes |
| 🔗 **Reverse Routing** | URLs générées automatiquement |

## 📦 Installation

```toml
[dependencies]
rusti = "1.0.0"
tokio = { version = "1", features = ["full"] }
```

### Sélection de la base de données

```toml
# SQLite (par défaut)
rusti = "1.0.0"

# PostgreSQL
rusti = { version = "1.0.0", features = ["postgres"] }

# MySQL / MariaDB
rusti = { version = "1.0.0", features = ["mysql"] }

# Toutes les bases de données
rusti = { version = "1.0.0", features = ["all-databases"] }
```

## 🎓 Guide de démarrage rapide

### 1. Créer votre projet

```bash
cargo new my-app
cd my-app
cargo add rusti tokio --features full
```

### 2. Structure recommandée

```
my-app/
├── src/
│   ├── main.rs          # Point d'entrée
│   ├── urls.rs          # Routes
│   ├── models.rs        # Structures principales
│   ├── forms.rs         # Formulaires
│   └── views.rs         # Handlers
├── templates/           # Templates Tera
│   └── index.html
├── static/              # CSS, JS, images
│   ├── css/
│   │   └── main.css
│   └── js/
│       └── main.js
├── media/               # Fichiers uploadés
└── .env                 # Configuration
```
### Fichier `.env`

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=votre-clé-super-secrète

# Hôtes autorisés (production)
ALLOWED_HOSTS=exemple.com,www.exemple.com

# Base de données PostgreSQL
DB_ENGINE=postgres
DB_USER=monuser
DB_PASSWORD=monmotdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mabase
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
        .server("127.0.0.1", 3000, "your-secret-key")
        .build();

    RustiApp::new(settings).await?
        .routes(urls::routes())
        .with_static_files()?
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### 4. Définir les routes (`src/urls.rs`)

```rust
use rusti::{Router, urlpatterns, view};
use crate::views;

pub fn routes() -> Router {
    urlpatterns! {

        // index
        "/" => view!{
            GET => views::index
        },
        name ="index",

        // À propos
        "/about" => view!{
            GET => views::about
        },
        name ="about",

        // Profil utilisateur
        "/user/{id}/{name}" => view! {
            GET => views::user_profile
        }, name = "user_profile",
    }
}
```

### 5. Créer les handlers (`src/views.rs`)

```rust
use rusti::prelude::*;
use rusti::context;

pub async fn index(
    template: Template,
    mut message: Message,
) -> Response {
    message.success("Bienvenue sur Rusti !").await;
    
    let ctx = context!{ 
        "title", "Accueil";
        "content", "Bienvenue sur le Framework Rusti"
    };

    template.render("index.html", &ctx)
}

pub async fn about(template: Template) -> Response {
    let ctx = context!{ 
        "title", "À propos"
    };
    
    template.render("about.html", &ctx)
}

pub async fn user_profile(
    Path((id, name)): Path<(u32, String)>,
    template: Template,
) -> Response {
    let ctx = context!{ 
        "user_id", id;
        "username", name
    };

    template.render("user_profile.html", &ctx)
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
        <a href='{% link "index" %}'>Accueil</a>
        <a href='{% link "about" %}'>À propos</a>
    </nav>

    {% messages %}

    <main>
        <h1>{{ title }}</h1>
        <p>{{ content }}</p>
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

- **[Guide de démarrage](docs/GETTING_STARTED.md)** - Tutoriel complet étape par étape
- **[Templates & Tags](docs/TEMPLATES.md)** - Système de templates personnalisé
- **[Guide base de données](docs/DATABASE.md)** - Configuration et ORM Django-like
- **[Configuration](docs/CONFIGURATION.md)** - Paramètres et variables d'environnement
- **[Référence API](docs/API.md)** - Documentation complète des types et fonctions

## 🎨 Fonctionnalités avancées

### ORM Django-like avec SeaORM

```rust
use rusti::prelude::*;
use sea_orm::entity::prelude::*;

// Définir votre modèle
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub age: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Activer l'API Django-like
impl_objects!(Entity);

// Utiliser comme Django !
pub async fn list_users(db: Extension<Arc<DatabaseConnection>>) -> Response {
    // Récupérer tous les adultes, triés par âge
    let adults = Entity::objects
        .filter(Column::Age.gte(18))
        .order_by_desc(Column::Age)
        .all(&**db)
        .await?;
    
    // Requête complexe avec chaînage
    let recent_active = Entity::objects
        .filter(Column::IsActive.eq(true))
        .exclude(Column::Email.like("%@banned.com"))
        .order_by_desc(Column::CreatedAt)
        .limit(10)
        .all(&**db)
        .await?;
}
```

### Formulaires automatiques avec validation

```rust
use rusti::prelude::*;
use sea_orm::entity::prelude::*;

// Définir votre modèle
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub age: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Générer automatiquement un formulaire
#[derive(DeriveModelForm)]
struct User;

// Utiliser dans vos vues
pub async fn register(template: Template) -> Response {
    let form = UserForm::build();
    
    let ctx = context! {
        "form", form
    };
    
    template.render("register.html", &ctx)
}

pub async fn register_submit(
    ExtractForm(form): ExtractForm<UserForm>,
    db: Extension<Arc<DatabaseConnection>>,
    mut message: Message,
) -> Response {
    if form.is_not_valid() {
        message.error("Échec de la validation").await;
        return redirect("/register");
    }
    
    // Enregistrer en base de données
    form.save(&**db).await.unwrap();
    message.success("Inscription réussie !").await;
    redirect("/dashboard")
}
```

### Messages Flash

```rust
pub async fn create_post(mut message: Message) -> Response {
    // ... logique de création ...
    
    message.success("Article créé avec succès !").await;
    message.info("N'oubliez pas de le publier").await;
    message.error("Erreur lors de l'upload du fichier").await;
    
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
<!-- Dans les templates -->
<a href='{% link "user_profile", id=42, name="alice" %}'>
    Voir le profil
</a>

<!-- Génère automatiquement : /user/42/alice -->
```

```rust
// Dans le code Rust
use rusti::reverse_with_parameters;

let url = reverse_with_parameters("user_profile", &[
    ("id", "42"),
    ("name", "alice"),
]).unwrap();
Redirect::to(&url).into_response()
```

## 🔧 Configuration avancée

### Avec base de données

```rust
use rusti::{
    RustiApp,
    Settings,
    DatabaseConfig,
    tokio,
    CspConfig,
};
mod url;
mod views;
mod models;
mod forms;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    println!("Connecté à la base de données {}", db_config.engine.name());

    // Configuration de l'application
    // Vous pouvez personnaliser les paramètres ici
    // Ils peuvent être importés du .env comme toute variable d'environnement
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(url::urls())
        .with_database(db)
        .with_static_files()?
        .with_allowed_hosts(env::var("ALLOWED_HOSTS")
            .ok()
            .map(|s| s.split(',').map(|h| h.to_string()).collect()))
        .with_sanitize_text_inputs(true)
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```



## 🐛 Pages de debug élégantes

En mode développement, Rusti affiche des pages d'erreur détaillées :

- ✅ Stack trace complète
- ✅ Informations de la requête HTTP
- ✅ Source du template avec numéros de ligne
- ✅ Liste des templates disponibles
- ✅ Variables d'environnement
- ✅ Version de Rust utilisée

## 🤝 Comparaison avec Django

| Concept Django | Équivalent Rusti |
|----------------|------------------|
| `settings.py` | `Settings::builder()` |
| `urls.py` | `urlpatterns! { ... }` |
| `views.py` | Handlers Axum |
| `models.py` | Entités SeaORM |
| `{% url 'name' %}` | `{% link "name" %}` |
| `{% static 'file' %}` | `{% static "file" %}` |
| `messages.success()` | `message.success().await` |
| `{% csrf_token %}` | `{% csrf %}` |
| `Model.objects.filter()` | `Entity::objects.filter()` |

## 📖 Exemples

Consultez le répertoire `examples/` pour des projets complets :

- **`demo-app`** - Application complète avec templates, fichiers statiques, formulaires
- **`rest-api`** - API JSON avec base de données
- **`blog`** - Blog avec authentification et CRUD

## 🛠️ Développement

```bash
# Cloner le dépôt
git clone https://github.com/seb-alliot/rusti
cd rusti

# Compiler le framework
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
MIT License

Copyright (c) 2025 Itsuki

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files, to deal in the software
without restriction, including the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the software.
```

## 🙏 Remerciements

- Inspiré de [Django](https://www.djangoproject.com/)
- Construit sur [Axum](https://github.com/tokio-rs/axum)
- Templates [Tera](https://github.com/Keats/tera)
- ORM [SeaORM](https://www.sea-ql.org/SeaORM/)

## 📞 Support

- 📖 [Documentation](https://docs.rs/rusti)
- 💬 [GitHub Discussions](https://github.com/seb-alliot/rusti/discussions)
- 🐛 [Issues](https://github.com/seb-alliot/rusti/issues)

---

**Développé avec ❤️ en Rust**