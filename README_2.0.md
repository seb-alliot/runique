# 🦀 Runique 2.0 - Django-Inspired Web Framework for Rust

**[🇫🇷 Français](#-runique-20---framework-web-inspiré-de-django-pour-rust) | [🇬🇧 English](#english-section)**

---

## 🇫🇷 Runique 2.0 - Framework Web Inspiré de Django pour Rust

### 📖 Table des matières

1. [Introduction](#introduction)
2. [Installation et Setup](#installation-et-setup)
3. [Architecture](#architecture)
4. [Démarrer une Application](#démarrer-une-application)
5. [Configuration](#configuration)
6. [Routage](#routage)
7. [Formulaires](#formulaires)
8. [Templates](#templates)
9. [Base de Données et ORM](#base-de-données-et-orm)
10. [Middleware et Sécurité](#middleware-et-sécurité)
11. [Flash Messages](#flash-messages)
12. [Exemples Complets](#exemples-complets)

---

### 📝 Introduction

**Runique 2.0** est une refonte complète du framework Runique, construite sur **Axum 0.7+** avec une architecture moderne et modulaire. Elle combine les meilleures pratiques de Django avec la puissance et la sécurité de Rust.

#### Caractéristiques principales:
- ✅ **Axum moderne** - Web framework haute performance
- ✅ **Modulaire** - Architecture par domaine (config, database, formulaire, etc.)
- ✅ **ORM SeaORM** - Requêtes type Django
- ✅ **Tera Templates** - Moteur de templates avec filtres personnalisés
- ✅ **Sécurité renforcée** - CSRF protection, CSP, validation d'hosts
- ✅ **Formulaires built-in** - Système de formulaires avec validation
- ✅ **Sessions** - Gestion des sessions avec tower_sessions
- ✅ **Middleware moderne** - tower-http, tower_sessions

---

### 💾 Installation et Setup

#### Prérequis:
- Rust 1.75+ (`rustup update`)
- PostgreSQL 12+ (ou SQLite pour le dev)
- Node.js pour les assets (optionnel)

#### Créer une nouvelle application:

```bash
# Cloner le projet
git clone https://github.com/seb-alliot/runique.git
cd runique

# Compiler
cargo build

# Lancer le serveur de développement
cargo run -p demo-app
```

Le serveur sera disponible à `http://127.0.0.1:3000` 🚀

#### Configuration du `.env`:

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# Base de données PostgreSQL
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=your_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://postgres:your_password@localhost:5432/runique

# Templates et assets
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# Sécurité
SECRETE_KEY=your_secret_key_here
ALLOWED_HOSTS=localhost,127.0.0.1,.example.com
```

---

### 🏗️ Architecture

Runique 2.0 est organisée en **modules fonctionnels**:

```
runique/src/
├── config_runique/          # Configuration et settings
├── data_base_runique/       # ORM et database config
├── formulaire/              # Système de formulaires
├── gardefou/                # Middleware (sécurité)
├── macro_runique/           # Macros utilitaires
├── moteur_engine/           # Moteur principal
├── request_context/         # Contexte de requête
├── runique_body/            # Builder d'application
└── utils/                   # Utilitaires divers
```

**Points clés:**
- **RuniqueEngine** = État principal de l'app (remplace AppState)
- **RuniqueContext** = Contexte injecté dans chaque requête
- **TemplateContext** = Contexte pour les templates

---

### 🚀 Démarrer une Application

#### Minimal Example:

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env().unwrap();

    let app = RuniqueApp::new(config)
        .with_database()
        .await
        .unwrap()
        .with_routes(routes())
        .build()
        .await
        .unwrap();

    app.run("127.0.0.1:3000").await.unwrap();
}

fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/users", post(create_user))
}

pub async fn index(mut template: TemplateContext){
    "Bienvenue sur Runique 2.0!"
}

async fn create_user(
    template: TemplateContext,
) -> Response {
    template.context(context_update!{
        "title" => "Créer un utilisateur",
        success!(template.messages => format!("Bienvenue {}, votre compte a été créé !", user.username))
        }).render("users/create.html")
}

### ⚙️ Configuration

#### RuniqueConfig:

La configuration se charge depuis `.env`:

```rust
let config = RuniqueConfig::from_env()?;

// Accès aux paramètres:
println!("Debug: {}", config.debug);
println!("DB: {}", config.database_url);
println!("Secret: {}", config.secret_key);
```

#### Variables d'environnement:

| Variable | Defaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | 127.0.0.1 | Adresse IP du serveur |
| `PORT` | 3000 | Port du serveur |
| `DEBUG` | true | Mode debug |
| `DATABASE_URL` | - | URL de connexion DB |
| `SECRETE_KEY` | - | Clé secrète CSRF |
| `ALLOWED_HOSTS` | * | Hosts autorisés |
| `TEMPLATES_DIR` | templates | Répertoire des templates |

---

### 🛣️ Routage

#### Macro `urlpatterns!`:

```rust
use runique::urlpatterns;
use crate::views;

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => views!( Get => views::index ), name="index",
        "user_list" => views!( Get = > views::user_list), name="user",
        "/users/<id>" => view!(Get => views::user_detail),name="user-detail",

        "user_create" => views! (
            Get => views::inscription_form,
            Post => views::create_user), name="inscription",
    };
    router
}

async fn index(template: TemplateContext) -> Response {
    template.context(context_update!{
        "title" => "Accueil"
    }).render("index.html")
}
```

#### Extraction de paramètres:

```rust
use axum::extract::Path;

async fn user_detail(
    Path(id): Path<i32>,
    template: TemplateContext,
) -> Response {
    // Utiliser `id`
    template.render("users/detail.html", &context! {
        "user_id" => id
    })
}

```

---

### 📋 Formulaires

#### Définir un formulaire:

```rust
use runique::formulaire::RuniqueForm;
use runique::formulaire::fields::*;

#[derive(RuniqueForm, Clone)]
pub struct RegisterForm {
    #[field(text, label = "Nom d'utilisateur")]
    pub username: String,

    #[field(email, label = "Email")]
    pub email: String,

    #[field(password, label = "Mot de passe")]
    pub password: String,
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<User> {
        // Sauvegarder dans la DB
    }
}
```

#### Utiliser un formulaire dans une vue:

```rust
use runique::formulaire::ExtractForm;

// Afficher le formulaire
async fn register_form(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let form = RegisterForm::build(ctx.engine.tera.clone());

    template.render("register.html", &context! {
        "form" => form,
        "title" => "Créer un compte"
    })
}

// Traiter la soumission
async fn register_submit(
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<RegisterForm>,
) -> Response {
    if form.is_valid().await {
        let db = ctx.engine.db.clone();
        match form.save(&*db).await {
            Ok(user) => {
                success!(ctx.flash => "Compte créé avec succès!");
                return Redirect::to("/login").into_response();
            }
            Err(e) => {
                form.get_form_mut().database_error(&e);
            }
        }
    }

    template.render("register.html", &context! {
        "form" => form
    })
}
```

#### Template de formulaire:

```html
<form method="POST" action="/register">
    {% for field in form.get_form().fields.values() %}
        {% if field.field_type != "hidden" %}
            <div class="form-group">
                <label for="{{ field.name }}">{{ field.label }}</label>
                {% if field.field_type == "textarea" %}
                    <textarea name="{{ field.name }}" id="{{ field.name }}">{{ field.value }}</textarea>
                {% else %}
                    <input type="{{ field.field_type }}"
                           name="{{ field.name }}"
                           id="{{ field.name }}"
                           value="{{ field.value }}"
                           {% if field.is_required %}required{% endif %}>
                {% endif %}
            </div>
        {% endif %}
    {% endfor %}

    <!-- CSRF Token -->
    {{ csrf_token | csrf_field | safe }}

    <button type="submit">Soumettre</button>
</form>
```

---

### 🎨 Templates

Runique utilise **Tera** (similaire à Jinja2) comme moteur de templates.

#### Filtres personnalisés:

```html
<!-- Assets -->
 <link rel="stylesheet" href='{% static "css/main.css" %}'>

<img src='{% media "media.avif" %}' alt="Logo">

<!-- Génération d'URLs -->
<a href={% link "detail_user" %}>Afficher un user</a>

<!-- CSRF -->
{% csrf %}

<!-- Formulaires -->
{% form.name_form %}
{% form.name_form.field }
```

#### Héritage de templates:

```html
<!-- base.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Runique{% endblock %}</title>
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>

<!-- page.html -->
{% extends "base.html" %}

{% block title %}Ma Page{% endblock %}

{% block content %}
    <h1>Contenu de la page</h1>
{% endblock %}
```

#### Variables globales:

```html
<!-- Disponibles dans tous les templates -->
{{ debug }}            <!-- true/false -->
{{ csrf_token }}       <!-- Token CSRF -->
{{ messages }}         <!-- Flash messages -->
```

---

### 🗄️ Base de Données et ORM

#### Modèles SeaORM:

```rust
use sea_orm::prelude::*;
use runique::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);  // ← Important!
```

#### Requêtes ORM:

```rust
use sea_orm::QueryFilter;
use demo_app::models::users;

// Récupérer tous les utilisateurs
let all_users = users::Entity::find()
    .all(&db)
    .await?;

// Filtrer
let admins = users::Entity::find()
    .filter(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// Par ID
let user = users::Entity::find_by_id(1)
    .one(&db)
    .await?;

// Premier résultat
let first_user = users::Entity::find()
    .one(&db)
    .await?;

// Créer
let new_user = users::ActiveModel {
    username: Set("john".to_string()),
    email: Set("john@example.com".to_string()),
    ..Default::default()
}.insert(&db).await?;

// Mettre à jour
let mut user = users::ActiveModel {
    id: Set(1),
    username: Set("jane".to_string()),
    ..Default::default()
};
user.update(&db).await?;

// Supprimer
users::Entity::delete_by_id(1)
    .exec(&db)
    .await?;
```

#### Objects Manager (Django-like):

```rust
// Style Django
let objects = users::Entity::objects();

// .all() - Tous les enregistrements
let users = objects.all().all(&db).await?;

// .filter() - Avec condition
let admins = objects
    .filter(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// .exclude() - Inverse du filtre
let non_admins = objects
    .exclude(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// .get() - Par ID
let user = objects.get(1).one(&db).await?;
```

---

### 🛡️ Middleware et Sécurité

#### Middleware Stack (ordre d'exécution):

```rust
// Déclaration (l'ordre est INVERSÉ):
.layer(session_layer)           // Exécute en dernier (plus proche de l'app)
.layer(sanitize_middleware)
.layer(csrf_middleware)
.layer(flash_middleware)
.layer(error_handler_middleware)
.layer(extension_injection)      // Exécute en premier (injecte les extensions)
```

**Important:** Middleware declared first = Executed last!

#### CSRF Protection:

```rust
// Automatique via middleware
// Token injecté dans TemplateContext

// Dans les formulaires:
{% csrf %}

// Validation automatique POST/PUT/PATCH/DELETE
// Token depuis form field ou X-CSRF-Token header
```

#### ALLOWED_HOSTS Validation:

```env
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

```rust
// Automatique - protège contre Host Header Injection
// Retourne 400 Bad Request si host non autorisé
```

#### Authentification:

```rust
use runique::gardefou::composant_middleware::login_requiert::*;

// Vérifier si authentifié
if is_authenticated(&session).await {
    // L'utilisateur est connecté
}

// Récupérer l'ID utilisateur
if let Some(user_id) = get_user_id(&session).await {
    println!("User: {}", user_id);
}

// Connecter un utilisateur
login_user(&session, 123, "john").await?;

// Déconnecter
logout(&session).await?;

// Middleware de protection
.layer(axum::middleware::from_fn(login_required))
```

#### CSP Nonce (pour scripts inline):

```rust
use runique::utils::csp_nonce::generate_csp_nonce;

let nonce = generate_csp_nonce();
// Génère: "abc123def456..."

// Dans les templates:
<script nonce="{{ nonce }}">
    console.log("Safe inline script");
</script>
```

---

### 💬 Flash Messages

#### Macros:

```rust
use runique::{success, info, warning, error, flash_now};

// Redirection avec message
success!(ctx.messages => "Utilisateur créé!");
info!(ctx.messages => "Opération en cours...");
warning!(ctx.messages => "Attention!");
error!(ctx.messages => "Erreur!");

// Message immédiat (dans la même requête)
let messages = flash_now! {
    success => "Succès!",
    error => "Erreur!"
};
```

#### Affichage dans templates:

```html
    {% messages %}
````

```
### La balise contient

{% if messages %}
    <div class="flash-messages">
        {% for message in messages %}
        <div class="message message-{{ message.level }}">
            {{ message.content }}
        </div>
        {% endfor %}
    </div>
{% endif %}
```

```
#### Extracteur Message:

```rust
use runique::request_context::Message;

async fn my_handler(message: Message) -> Response {
    // Ajouter un message
    message.success("Tout va bien!").await?;
    message.error("Ça a échoué").await?;

    // Récupérer tous les messages
    let all = message.get_all().await?;

    // Récupérer et nettoyer
    let msgs = message.pop_all().await?;
}
```

---

### 📚 Exemples Complets

#### Exemple 1: CRUD simple

```rust
// models/post.rs
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// views.rs
use demo_app::models::posts;

// Liste
async fn post_list(ctx: RuniqueContext, template: TemplateContext) -> Response {
    let db = ctx.engine.db.clone();
    let posts = posts::Entity::find().all(&*db).await.unwrap_or_default();

    template.render("posts/list.html", &context! {
        "posts" => posts
    })
}

// Détail
async fn post_detail(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match posts::Entity::find_by_id(id).one(&*db).await {
        Ok(Some(post)) => {
            template.render("posts/detail.html", &context! {
                "post" => post
            })
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

// Créer
async fn post_create(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(form): ExtractForm<PostForm>,
) -> Response {
    if form.is_valid().await {
        let db = ctx.engine.db.clone();
        match form.save(&*db).await {
            Ok(_) => {
                success!(ctx.flash => "Article créé!");
                return Redirect::to("/posts").into_response();
            }
            Err(e) => {
                error!(ctx.flash => format!("Erreur: {}", e));
            }
        }
    }

    template.render("posts/form.html", &context! {
        "form" => form
    })
}
```

#### Exemple 2: Avec authentification

```rust
// Middleware protection
async fn protected_routes() -> Router {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/profile", get(profile))
        .layer(axum::middleware::from_fn(login_required))
}

async fn dashboard(
    session: Session,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    if let Some(user_id) = get_user_id(&session).await {
        template.render("dashboard.html", &context! {
            "user_id" => user_id
        })
    } else {
        Redirect::to("/login").into_response()
    }
}
```

---

### 🎓 Bonnes pratiques

1. **Toujours cloner la DB:**
   ```rust
   let db = ctx.engine.db.clone();
   users::Entity::find().one(&*db).await?;
   ```

2. **Utiliser les macros:**
   ```rust
   success!(ctx.flash => "Message");
   ```

3. **Formulaires par requête:**
   ```rust
   let form = MyForm::build(ctx.engine.tera.clone());
   // Pas de state partagé!
   ```

4. **Templates sécurisés:**
   ```html
   {{ user_input | escape }}
   ```

5. **Middleware order (REVERSE):**
   ```
   Déclaré premier = Exécuté dernier
   ```

---

## 🇬🇧 English Section

### 📖 Table of Contents

1. [Introduction](#english-introduction)
2. [Installation & Setup](#english-installation)
3. [Architecture](#english-architecture)
4. [Creating an Application](#english-app)
5. [Configuration](#english-config)
6. [Routing](#english-routing)
7. [Forms](#english-forms)
8. [Templates](#english-templates)
9. [Database & ORM](#english-orm)
10. [Middleware & Security](#english-middleware)
11. [Flash Messages](#english-flash)
12. [Complete Examples](#english-examples)

---

### 📝 English Introduction {#english-introduction}

**Runique 2.0** is a complete rewrite of the Runique framework, built on **Axum 0.7+** with modern, modular architecture. It combines the best practices of Django with the power and safety of Rust.

#### Key Features:
- ✅ **Modern Axum** - High-performance web framework
- ✅ **Modular** - Domain-driven architecture
- ✅ **SeaORM** - Django-like database queries
- ✅ **Tera Templates** - Template engine with custom filters
- ✅ **Enhanced Security** - CSRF protection, CSP, host validation
- ✅ **Built-in Forms** - Form system with validation
- ✅ **Sessions** - Session management with tower_sessions
- ✅ **Modern Middleware** - tower-http, tower_sessions

---

### 💾 Installation & Setup {#english-installation}

#### Prerequisites:
- Rust 1.70+ (`rustup update`)
- PostgreSQL 12+ (or SQLite for dev)
- Node.js for assets (optional)

#### Create a new application:

```bash
git clone https://github.com/yourusername/runique.git
cd runique
cargo build
cargo run -p demo-app
```

Server available at `http://127.0.0.1:3000` 🚀

#### .env Configuration:

```env
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=your_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
DATABASE_URL=postgres://postgres:your_password@localhost:5432/runique

TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

SECRETE_KEY=your_secret_key_here
ALLOWED_HOSTS=localhost,127.0.0.1,.example.com
```

---

### 🏗️ Architecture {#english-architecture}

Runique 2.0 is organized into **functional modules**:

```
runique/src/
├── config_runique/          # Configuration & settings
├── data_base_runique/       # ORM & database config
├── formulaire/              # Form system
├── gardefou/                # Middleware (security)
├── macro_runique/           # Utility macros
├── moteur_engine/           # Main engine
├── request_context/         # Request context
├── runique_body/            # App builder
└── utils/                   # Utilities
```

**Key concepts:**
- **RuniqueEngine** = Main app state (replaces AppState)
- **RuniqueContext** = Injected into each request
- **TemplateContext** = Context for templates

---

### 🚀 Creating an Application {#english-app}

#### Minimal Example:

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env().unwrap();

    let app = RuniqueApp::new(config)
        .with_database()
        .await
        .unwrap()
        .with_routes(routes())
        .build()
        .await
        .unwrap();

    app.run("127.0.0.1:3000").await.unwrap();
}

fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/users", post(create_user))
}

async fn index() -> impl IntoResponse {
    "Welcome to Runique 2.0!"
}

async fn create_user(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    template.render("users/create.html", &context! {
        "title" => "Create User"
    })
}
```

---

### ⚙️ Configuration {#english-config}

#### RuniqueConfig:

Configuration is loaded from `.env`:

```rust
let config = RuniqueConfig::from_env()?;

println!("Debug: {}", config.debug);
println!("DB: {}", config.database_url);
println!("Secret: {}", config.secret_key);
```

#### Environment Variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | 127.0.0.1 | Server IP address |
| `PORT` | 3000 | Server port |
| `DEBUG` | true | Debug mode |
| `DATABASE_URL` | - | DB connection URL |
| `SECRETE_KEY` | - | CSRF secret key |
| `ALLOWED_HOSTS` | * | Allowed hosts |
| `TEMPLATES_DIR` | templates | Templates directory |

---

### 🛣️ Routing {#english-routing}

#### urlpatterns! Macro:

```rust
use runique::urlpatterns;

pub fn routes() -> Router {
    urlpatterns! {
        "index" => "/" => get(index),
        "user_list" => "/users" => get(user_list),
        "user_detail" => "/users/<id>" => get(user_detail),
        "user_create" => "/users" => post(create_user),
    }
}

async fn user_detail(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    template.render("users/detail.html", &context! {
        "user_id" => id
    })
}
```

---

### 📋 Forms {#english-forms}

#### Define a Form:

```rust
use runique::formulaire::RuniqueForm;
use runique::formulaire::fields::*;

#[derive(RuniqueForm, Clone)]
pub struct RegisterForm {
    #[field(text, label = "Username")]
    pub username: String,

    #[field(email, label = "Email")]
    pub email: String,

    #[field(password, label = "Password")]
    pub password: String,
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<User> {
        // Save to DB
    }
}
```

#### Use Form in View:

```rust
use runique::formulaire::ExtractForm;

// Display form
async fn register_form(
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let form = RegisterForm::build(ctx.engine.tera.clone());

    template.render("register.html", &context! {
        "form" => form,
        "title" => "Create Account"
    })
}

// Handle submission
async fn register_submit(
    mut ctx: RuniqueContext,
    template: TemplateContext,
    ExtractForm(mut form): ExtractForm<RegisterForm>,
) -> Response {
    if form.is_valid().await {
        let db = ctx.engine.db.clone();
        match form.save(&*db).await {
            Ok(user) => {
                success!(ctx.flash => "Account created successfully!");
                return Redirect::to("/login").into_response();
            }
            Err(e) => {
                form.get_form_mut().database_error(&e);
            }
        }
    }

    template.render("register.html", &context! {
        "form" => form
    })
}
```

---

### 🎨 Templates {#english-templates}

Runique uses **Tera** as template engine (similar to Jinja2).

#### Custom Filters:

```html
<!-- Assets -->
<link rel="stylesheet" href="{{ 'css/style.css' | static }}">
<img src="{{ 'profile.jpg' | media }}" alt="Profile">

<!-- URL generation -->
<a href="{{ 'user_detail' | link }}">View User</a>

<!-- CSRF -->
{{ csrf_token | csrf_field | safe }}

<!-- Forms -->
{{ form | form }}
```

#### Template Inheritance:

```html
<!-- base.html -->
<!DOCTYPE html>
<html>
<head>
    <title>{% block title %}Runique{% endblock %}</title>
</head>
<body>
    {% block content %}{% endblock %}
</body>
</html>

<!-- page.html -->
{% extends "base.html" %}

{% block title %}My Page{% endblock %}

{% block content %}
    <h1>Page content</h1>
{% endblock %}
```

---

### 🗄️ Database & ORM {#english-orm}

#### SeaORM Models:

```rust
use sea_orm::prelude::*;
use runique::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);  // Important!
```

#### ORM Queries:

```rust
use sea_orm::QueryFilter;
use demo_app::models::users;

// Get all users
let all_users = users::Entity::find()
    .all(&db)
    .await?;

// Filter
let admins = users::Entity::find()
    .filter(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// By ID
let user = users::Entity::find_by_id(1)
    .one(&db)
    .await?;

// Create
let new_user = users::ActiveModel {
    username: Set("john".to_string()),
    email: Set("john@example.com".to_string()),
    ..Default::default()
}.insert(&db).await?;

// Update
let mut user = users::ActiveModel {
    id: Set(1),
    username: Set("jane".to_string()),
    ..Default::default()
};
user.update(&db).await?;

// Delete
users::Entity::delete_by_id(1)
    .exec(&db)
    .await?;
```

#### Objects Manager (Django-like):

```rust
let objects = users::Entity::objects();

// .all() - Get all records
let users = objects.all().all(&db).await?;

// .filter() - With condition
let admins = objects
    .filter(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// .exclude() - Opposite of filter
let non_admins = objects
    .exclude(users::Column::IsAdmin.eq(true))
    .all(&db)
    .await?;

// .get() - By ID
let user = objects.get(1).one(&db).await?;
```

---

### 🛡️ Middleware & Security {#english-middleware}

#### Middleware Stack (Execution Order):

```rust
// Declaration (order is REVERSED):
.layer(session_layer)           // Executes last
.layer(sanitize_middleware)
.layer(csrf_middleware)
.layer(flash_middleware)
.layer(error_handler_middleware)
.layer(extension_injection)      // Executes first
```

**Important:** First declared = Last executed!

#### CSRF Protection:

```rust
// Automatic via middleware
// Token injected into TemplateContext

// In forms:
{{ csrf_token | csrf_field | safe }}

// Validation automatic on POST/PUT/PATCH/DELETE
// Token from form field or X-CSRF-Token header
```

#### ALLOWED_HOSTS Validation:

```env
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

Automatic protection against Host Header Injection attacks.

#### Authentication:

```rust
use runique::gardefou::composant_middleware::login_requiert::*;

// Check if authenticated
if is_authenticated(&session).await {
    // User is logged in
}

// Get user ID
if let Some(user_id) = get_user_id(&session).await {
    println!("User: {}", user_id);
}

// Login user
login_user(&session, 123, "john").await?;

// Logout
logout(&session).await?;

// Protect routes
.layer(axum::middleware::from_fn(login_required))
```

#### CSP Nonce (for inline scripts):

```rust
use runique::utils::csp_nonce::generate_csp_nonce;

let nonce = generate_csp_nonce();

// In templates:
<script nonce="{{ nonce }}">
    console.log("Safe inline script");
</script>
```

---

### 💬 Flash Messages {#english-flash}

#### Macros:

```rust
use runique::{success, info, warning, error, flash_now};

success!(ctx.flash => "User created!");
info!(ctx.flash => "Operation in progress...");
warning!(ctx.flash => "Warning!");
error!(ctx.flash => "Error!");

// Immediate message (same request)
let messages = flash_now! {
    success => "Success!",
    error => "Error!"
};
```

#### Display in Templates:

```html
{% for message in messages %}
    <div class="alert alert-{{ message.level | lower }}">
        {{ message.content }}
    </div>
{% endfor %}
```

#### Message Extractor:

```rust
use runique::request_context::Message;

async fn my_handler(message: Message) -> Response {
    message.success("All good!").await?;
    message.error("Failed").await?;

    let all = message.get_all().await?;
    let msgs = message.pop_all().await?;
}
```

---

### 📚 Complete Examples {#english-examples}

#### Example 1: Simple CRUD

```rust
// models/post.rs
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

// views.rs
async fn post_list(ctx: RuniqueContext, template: TemplateContext) -> Response {
    let db = ctx.engine.db.clone();
    let posts = posts::Entity::find().all(&*db).await.unwrap_or_default();

    template.render("posts/list.html", &context! {
        "posts" => posts
    })
}

async fn post_detail(
    Path(id): Path<i32>,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    let db = ctx.engine.db.clone();

    match posts::Entity::find_by_id(id).one(&*db).await {
        Ok(Some(post)) => {
            template.render("posts/detail.html", &context! {
                "post" => post
            })
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}
```

#### Example 2: With Authentication

```rust
async fn protected_routes() -> Router {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/profile", get(profile))
        .layer(axum::middleware::from_fn(login_required))
}

async fn dashboard(
    session: Session,
    ctx: RuniqueContext,
    template: TemplateContext,
) -> Response {
    if let Some(user_id) = get_user_id(&session).await {
        template.render("dashboard.html", &context! {
            "user_id" => user_id
        })
    } else {
        Redirect::to("/login").into_response()
    }
}
```

---

### 🎓 Best Practices

1. **Always clone the DB:**
   ```rust
   let db = ctx.engine.db.clone();
   users::Entity::find().one(&*db).await?;
   ```

2. **Use macros:**
   ```rust
   success!(ctx.flash => "Message");
   ```

3. **Forms per request:**
   ```rust
   let form = MyForm::build(ctx.engine.tera.clone());
   // No shared state!
   ```

4. **Secure templates:**
   ```html
   {{ user_input | escape }}
   ```

5. **Middleware order (REVERSE):**
   ```
   Declared first = Executed last
   ```

---

## 📄 License

MIT License - See LICENSE-MIT.md

## 🤝 Contributing

Contributions welcome! Please read SECURITY.md before submitting.

## 📞 Support

- 📖 [Documentation](https://docs.rs/runique)
- 🐛 [Report Issues](https://github.com/yourusername/runique/issues)
- 💬 [Discussions](https://github.com/yourusername/runique/discussions)


![Security](https://img.shields.io/badge/Security-Prisme_Verified-orange?style=for-the-badge&logo=shield-lock)
![Performance](https://img.shields.io/badge/Performance-Zero--Copy-green?style=for-the-badge&logo=rust)

###  Extracteur est en cour de modification
Son fonctionnement va evoluer , pour plus de details attendais la fin de son implementation, sa concernera aussi en partis les formulaires et le csrf token

---

**Made with ❤️ by the Runique team**
