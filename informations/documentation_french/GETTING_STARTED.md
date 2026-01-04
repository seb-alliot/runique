# Guide de démarrage rapide - RuniqueFramework

Bienvenue dans Runique! Ce guide vous accompagnera pas à pas dans la création de votre première application web avec Runique.

## Prérequis

- **Rust 1.75+** - [Installer Rust](https://www.rust-lang.org/tools/install)
- **Cargo** (installé automatiquement avec Rust)
- Connaissances de base en Rust (ownership, borrowing, async/await)

---

## Installation

### 1. Créer un nouveau projet

```bash
cargo new mon_app
cd mon_app
```

### 2. Ajouter Runiqueaux dépendances

Éditez `Cargo.toml` :

```toml
[package]
name = "mon_app"
version = "0.1.0.6"
edition = "2021"

[dependencies]
runique= "1.0.6"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

**Avec PostgreSQL :**

```toml
[dependencies]
runique= { version = "1.0.6", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## Application minimale

### 1. Code source (src/main.rs)

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger la configuration depuis .env
    let settings = Settings::from_env();

    // Créer et lancer l'application
    RuniqueApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        path!("", index),
    ]
}

async fn index() -> &'static str {
    "Bienvenue sur Runique!"
}
```

### 2. Configuration (.env)

Créez un fichier `.env` à la racine :

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=change-me-in-production-with-32-chars-minimum
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true
```

### 3. Lancement

```bash
cargo run
```

Ouvrez [http://localhost:8000](http://localhost:8000) dans votre navigateur.

✅ Vous devriez voir : **"Bienvenue sur Runique!"**

---

## Routing basique

### URLs avec paramètres

```rust
use runique::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index),
        path!("hello/<name>", hello),
        path!("user/<id>", user_detail),
    ]
}

async fn index() -> &'static str {
    "Page d'accueil"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Bonjour, {} !", name)
}

async fn user_detail(Path(id): Path<i32>) -> String {
    format!("Détails de l'utilisateur #{}", id)
}
```

**Test :**
- `GET /` → "Page d'accueil"
- `GET /hello/Alice` → "Bonjour, Alice !"
- `GET /user/42` → "Détails de l'utilisateur #42"

### Noms de routes (reverse routing)

```rust
use runique::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", index, "index"),
        path!("posts/", list_posts, "post_list"),
        path!("posts/<id>/", detail_post, "post_detail"),
    ]
}
```

---

## Templates avec Tera

### 1. Structure des dossiers

```
mon_app/
├── Cargo.toml
├── .env
├── src/
│   └── main.rs
└── templates/
    ├── base.html
    └── index.html
```

### 2. Template de base (templates/base.html)

```html
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}Mon App{% endblock %}</title>
</head>
<body>
    <header>
        <h1>Mon Application Runique</h1>
        <nav>
            <a href="{% link 'index' %}">Accueil</a>
        </nav>
    </header>

    <main>
        {% block content %}{% endblock %}
    </main>

    <footer>
        <p>© 2026 Mon App</p>
    </footer>
</body>
</html>
```

### 3. Template de page (templates/index.html)

```html
{% extends "base.html" %}

{% block title %}Accueil{% endblock %}

{% block content %}
<h2>Bienvenue {{ username }} !</h2>
<p>Vous êtes connecté depuis le {{ date }}.</p>
{% endblock %}
```

### 4. Utilisation dans un handler

```rust
use runique::prelude::*;

async fn index(template: Template) -> Response {
    template.render("index.html", context! {
        username: "Alice",
        date: chrono::Utc::now().format("%d/%m/%Y").to_string(),
    })
}
```

---

## Base de données

### 1. Configuration

Ajoutez dans `.env` :

```env
# PostgreSQL
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# SQLite (alternative)
# DB_ENGINE=sqlite
# DB_NAME=database.sqlite
```

### 2. Définir un modèle

Créez `src/models.rs` :

```rust
use runique::prelude::*;
use sea_orm::entity::prelude::*;
use runique::impl_objects;

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

// Active l'API Django-like
impl_objects!(Entity);
```

### 3. Connexion à la base

```rust
use runique::prelude::*;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RuniqueApp::new(settings).await?
        .with_database(db)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### 4. Utilisation dans un handler

```rust
use runique::prelude::*;
use crate::models::{users, Entity as User};

async fn list_users(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // API Django-like
    let users = User::objects
        .filter(users::Column::IsActive.eq(true))
        .order_by_asc(users::Column::Username)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.render("users.html", context! {
        users: users,
    })
}
```

---

## Formulaires

### 1. Définir un formulaire

Créez `src/forms.rs` :

```rust
use runique::prelude::*;
use runique::forms::prelude::*;

#[runique_form]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactForm {
    #[field(required = true)]
    pub name: CharField,

    #[field(required = true)]
    pub email: EmailField,

    #[field(required = true)]
    pub subject: CharField,

    #[field(widget = "textarea", required = true)]
    pub message: CharField,
}
```

### 2. Affichage du formulaire

```rust
use runique::prelude::*;
use crate::forms::ContactForm;

async fn contact_view(template: Template) -> Response {
    let form = ContactForm::new();
    template.render("contact.html", context! {
        form: form,
    })
}
```

Template `templates/contact.html` :

```html
{% extends "base.html" %}

{% block content %}
<h2>Contactez-nous</h2>

<form method="post">
    {% csrf %}
    {{ form }}
    <button type="submit">Envoyer</button>
</form>
{% endblock %}
```

### 3. Traitement du formulaire

```rust
use runique::prelude::*;
use crate::forms::ContactForm;

async fn contact_submit(
    Form(form): Form<ContactForm>,
    template: Template,
    mut message: Message,
) -> Response {
    // Validation
    if !form.is_valid() {
        return template.render("contact.html", context! {
            form: form,
            errors: form.errors(),
        });
    }

    // Traitement (envoyer email, etc.)
    success!(message, "Message envoyé avec succès !");

    redirect("/")
}
```

---

## Middleware et sécurité

### Configuration recommandée

```rust
use runique::prelude::*;
use runique::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RuniqueApp::new(settings).await?
        // Sécurité
        .middleware(CsrfMiddleware::new())
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(AllowedHostsMiddleware)
        .middleware(XssSanitizerMiddleware)

        // Fonctionnalités
        .middleware(FlashMiddleware)
        .middleware(MessageMiddleware)

        // Routes
        .routes(routes())

        // Lancement
        .run()
        .await?;

    Ok(())
}
```

### Protection CSRF

Automatique avec `CsrfMiddleware` :

```html
<form method="post">
    {% csrf %}
    <!-- Le token est vérifié automatiquement -->
</form>
```

### Content Security Policy

```rust
use runique::prelude::*;
use runique::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    use_nonce: true,
    ..Default::default()
};

RuniqueApp::new(settings).await?
    .middleware(CspMiddleware::new(csp_config))
    .routes(routes())
    .run()
    .await?;
```

---

## Fichiers statiques

### 1. Configuration (.env)

```env
STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/
```

### 2. Structure des dossiers

```
mon_app/
├── static/
│   ├── css/
│   │   └── style.css
│   └── js/
│       └── app.js
└── media/
    └── uploads/
```

### 3. Utilisation dans les templates

```html
<!-- Fichiers statiques -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src="{% static 'js/app.js' %}"></script>

<!-- Fichiers media (uploadés) -->
<img src="{% media user.avatar %}" alt="Avatar">
```

---

## Messages flash

### 1. Activation

```rust
use runique::prelude::*;

RuniqueApp::new(settings).await?
    .middleware(FlashMiddleware)
    .middleware(MessageMiddleware)
    .routes(routes())
    .run()
    .await?;
```

### 2. Utilisation dans les handlers

```rust
use runique::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        let _ = message.error("Données invalides").await;
        return redirect("/register");
    }

    // Créer l'utilisateur...

    let _ = message.success("Compte créé avec succès !").await;
    redirect("/dashboard")
}
```

### 3. Affichage dans les templates

```html
{% messages %}
```

Ou manuellement :

```html
{% for msg in get_messages() %}
<div class="alert alert-{{ msg.level }}">
    {{ msg.message }}
</div>
{% endfor %}
```

### 4. Macros utilitaires

Pour simplifier l'envoi de messages, Runiquefournit des macros :

```rust
use runique::prelude::*;

async fn create_user(
    Form(form): Form<UserForm>,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        // Macro error! - plus concis
        error!(message, "Données invalides");
        return redirect("/register");
    }

    // Créer l'utilisateur...

    // Macro success! - plus concis
    success!(message, "Compte créé avec succès !");
    redirect("/dashboard")
}
```

**Macros disponibles :**

| Macro | Équivalent | Utilisation |
|-------|-----------|-------------|
| `success!(msg, "text")` | `msg.success("text").await.unwrap()` | Messages de succès |
| `error!(msg, "text")` | `msg.error("text").await.unwrap()` | Messages d'erreur |
| `info!(msg, "text")` | `msg.info("text").await.unwrap()` | Messages d'information |
| `warning!(msg, "text")` | `msg.warning("text").await.unwrap()` | Messages d'avertissement |

**Plusieurs messages en une fois :**

```rust
// Envoyer plusieurs messages successifs
success!(
    message,
    "Utilisateur créé",
    "Email envoyé",
    "Bienvenue !"
);

// Ou de manière plus lisible
success!(message, "Utilisateur créé");
info!(message, "Vérifiez votre email");
warning!(message, "Pensez à valider votre compte");
```

**Avantages des macros :**
- ✅ Syntaxe plus concise
- ✅ Gestion automatique du `.await.unwrap()`
- ✅ Support de messages multiples
- ✅ Code plus lisible

---

## Exemple complet : Blog simple

### Structure

```
blog/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── models.rs
│   ├── forms.rs
│   └── views.rs
├── templates/
│   ├── base.html
│   ├── posts/
│   │   ├── list.html
│   │   ├── detail.html
│   │   └── create.html
└── static/
    └── css/
        └── style.css
```

### Modèle (src/models.rs)

```rust
use runique::prelude::*;
use sea_orm::entity::prelude::*;
use runique::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub published: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
```

### Formulaire (src/forms.rs)

```rust
use runique::prelude::*;
use runique::forms::prelude::*;

#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "crate::models::Model", entity = "crate::models::Entity")]
pub struct PostForm {
    #[field(required = true)]
    pub title: CharField,

    #[field(required = true)]
    pub slug: CharField,

    #[field(widget = "textarea", required = true)]
    pub content: CharField,

    #[field(default = "false")]
    pub published: BooleanField,
}
```

### Vues (src/views.rs)

```rust
use runique::prelude::*;
use crate::models::{posts, Entity as Post};
use crate::forms::PostForm;

// Liste des articles
pub async fn list_posts(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let posts = Post::objects
        .filter(posts::Column::Published.eq(true))
        .order_by_desc(posts::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap_or_default();

    template.render("posts/list.html", context! {
        posts: posts,
    })
}

// Détail d'un article
pub async fn detail_post(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    let post = match Post::objects.get(&*db, id).await {
        Ok(p) => p,
        Err(_) => return (StatusCode::NOT_FOUND, "Article introuvable").into_response(),
    };

    template.render("posts/detail.html", context! {
        post: post,
    })
}

// Formulaire de création
pub async fn create_post_view(template: Template) -> Response {
    let form = PostForm::new();
    template.render("posts/create.html", context! {
        form: form,
    })
}

// Traitement de création
pub async fn create_post_submit(
    Form(form): Form<PostForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("posts/create.html", context! {
            form: form,
        });
    }

    match form.save(&*db).await {
        Ok(post) => {
            success!(message, "Article créé avec succès !");
            redirect(&format!("/posts/{}/", post.id))
        }
        Err(_) => {
            error!(message, "Erreur lors de la création");
            template.render("posts/create.html", context! {
                form: form,
            })
        }
    }
}
```

### Routes (src/main.rs)

```rust
use runique::prelude::*;

mod models;
mod forms;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    RuniqueApp::new(settings).await?
        .with_database(db)
        .middleware(CsrfMiddleware::new())
        .middleware(SecurityHeadersMiddleware::new())
        .middleware(FlashMiddleware)
        .middleware(MessageMiddleware)
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        path!("", views::list_posts, "post_list"),
        path!("posts/<id>/", views::detail_post, "post_detail"),
        path!("posts/create/", views::create_post_view, "post_create"),
        path!("posts/create/submit/", views::create_post_submit),
    ]
}
```

### Template liste (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block title %}Articles{% endblock %}

{% block content %}
<h2>Tous les articles</h2>

{% messages %}

{% for post in posts %}
<article>
    <h3>{{ post.title }}</h3>
    <p>{{ post.content|truncate(200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Lire la suite</a>
</article>
{% endfor %}

<a href="{% link 'post_create' %}">Créer un article</a>
{% endblock %}
```

---

## Prochaines étapes

Maintenant que vous maîtrisez les bases, explorez :

1. **[Configuration avancée](CONFIGURATION.md)** - Variables d'environnement, settings
2. **[Base de données](DATABASE.md)** - Relations, transactions, migrations
3. **[Sécurité](SECURITY.md)** - CSP, CSRF, XSS, headers HTTP
4. **[Templates](TEMPLATES.md)** - Tags personnalisés, filtres, préprocessing
5. **[Déploiement](DEPLOIEMENT.md)** - Production, Docker, reverse proxy

---

## Besoin d'aide ?

- 📖 [Documentation complète](README.md)
- 🐛 [Signaler un bug](https://github.com/seb-alliot/runique/tree/issues)
- 💬 [Discord](https://discord.gg/Y5zW7rbt)

---

**Bon développement avec Runique! 🚀**

---

**Version:** 1.0.6.0 (Corrigée - 2 Janvier 2026)
**Licence:** MIT

*Documentation created with ❤️ by Claude for Itsuki*
