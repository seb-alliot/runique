# Rusti

**Un framework web Rust inspiré de Django**

Rusti est un framework web moderne qui combine la sécurité et les performances de Rust avec l'ergonomie de Django. Il offre une expérience de développement familière aux développeurs Django tout en exploitant la puissance du système de types de Rust.

[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](https://github.com/seb-alliot/rusti)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🚀 Caractéristiques principales

### Architecture Django-like
- **Routing déclaratif** avec `urlpatterns!` macro
- **ORM intuitif** basé sur SeaORM avec API Django-style
- **Système de templates** Tera avec préprocessing personnalisé
- **Génération automatique de formulaires** via macros procédurales
- **Messages flash** entre requêtes
- **Gestion des fichiers statiques et media**

### Sécurité intégrée
- ✅ **Protection CSRF** (HMAC-SHA256)
- ✅ **Content Security Policy** (CSP) avec nonces
- ✅ **Sanitization XSS** (ammonia)
- ✅ **Security Headers** automatiques (HSTS, X-Frame-Options, etc.)
- ✅ **Validation ALLOWED_HOSTS**
- ✅ **Hachage Argon2id** intégré

### Support multi-bases de données
- PostgreSQL
- MySQL / MariaDB
- SQLite

### Développement moderne
- **Async/await** natif avec Tokio
- **Type-safe** grâce au système de types Rust
- **Zero-cost abstractions**
- **Hot reload** en développement
- **Documentation complète** en français et anglais

---

## 📦 Installation

### Prérequis

- Rust 1.75+ ([installer Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Ajouter Rusti à votre projet

```toml
# Cargo.toml

# Configuration minimale (SQLite par défaut)
[dependencies]
rusti = "1.0.0"

# Avec PostgreSQL
[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }

# Avec MySQL
[dependencies]
rusti = { version = "1.0.0", features = ["mysql"] }

# Avec MariaDB
[dependencies]
rusti = { version = "1.0.0", features = ["mariadb"] }

# Avec toutes les bases de données
[dependencies]
rusti = { version = "1.0.0", features = ["all-databases"] }
```

### Features Cargo disponibles

| Feature | Description | Par défaut |
|---------|-------------|------------|
| `default` | Active le support ORM avec SQLite | ✅ |
| `orm` | Active SeaORM | ✅ (inclus dans `default`) |
| `sqlite` | Driver SQLite | ✅ (inclus dans `orm`) |
| `postgres` | Driver PostgreSQL | ❌ |
| `mysql` | Driver MySQL | ❌ |
| `mariadb` | Driver MariaDB (utilise le driver MySQL) | ❌ |
| `all-databases` | Active tous les drivers simultanément | ❌ |

**Exemples de configuration :**

```toml
# SQLite uniquement (configuration par défaut)
[dependencies]
rusti = "1.0.0"

# PostgreSQL + MySQL
[dependencies]
rusti = { version = "1.0.0", features = ["postgres", "mysql"] }

# Toutes les bases de données
[dependencies]
rusti = { version = "1.0.0", features = ["all-databases"] }

# Sans ORM (framework minimal)
[dependencies]
rusti = { version = "1.0.0", default-features = false }
```

### Créer un nouveau projet

```bash
cargo new mon_app
cd mon_app
```

Ajoutez Rusti dans `Cargo.toml` :

```toml
[dependencies]
rusti = { version = "1.0.0", features = ["postgres"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## 🏁 Démarrage rapide

### Application minimale

```rust
// src/main.rs
use rusti::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RustiApp::new(settings).await?
        .routes(routes())
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router {
    urlpatterns![
        "/" => view!{
            GET => views::index
        },
        name ="index",

        "/hello" => view!{
            GET => views::hello
        },
        name ="hello",

        "/user" => view! {
            GET => views::user_profile,
            POST => views::user_profile_submit
        },
         name = "user_profile",
    ]
}

async fn index() -> &'static str {
    "Bienvenue sur Rusti !"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Bonjour, {} !", name)
}

pub async fn user_profile(
    template: Template,
    ExtractForm(form): ExtractForm<ModelForm>,
) -> Response {
    let ctx = context! {
        "title", "Profil Utilisateur";
        "form", form
    };
    template.render("profile/register_profile.html", &ctx)
}

pub async fn user_profile_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    // 1. Vérification de la validité du formulaire
    if user.is_valid() {
        match user.save(&db).await {
            Ok(created_user) => {
                success!(message, "Profil utilisateur créé avec succès !");
                
                // Génération de l'URL de redirection
                let target = reverse_with_parameters(
                    "user_profile",
                    &[
                        ("id", &created_user.id.to_string()),
                        ("name", &created_user.username),
                    ],
                ).unwrap();
                
                return Redirect::to(&target).into_response();
            }
            Err(err) => {
                // Gestion des erreurs d'unicité de la base de données
                let error_msg = if err.to_string().contains("unique") {
                    if err.to_string().contains("username") {
                        "Ce nom d'utilisateur est déjà pris !"
                    } else if err.to_string().contains("email") {
                        "Cette adresse email est déjà utilisée !"
                    } else {
                        "Cette valeur existe déjà dans la base de données."
                    }
                } else {
                    "Une erreur est survenue lors de l'enregistrement."
                };

                error!(message, error_msg);
                
                let ctx = context! {
                    "form", ModelForm::build();
                    "forms_errors", user.get_errors();
                    "title", "Profil";
                    "db_error", error_msg
                };
                return template.render("name.html", &ctx);
            }
        }
    }
    
    // 2. Cas d'erreur de validation (champs mal remplis)
    error!(message, "Le formulaire contient des erreurs de validation.");
    
    let ctx = context! {
        "form", ModelForm::build();
        "forms_errors", user.get_errors();
        "title", "Erreur de validation"
    };
    template.render("name.html", &ctx)
}
```

### Configuration (.env)

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=your-secret-key-here
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

# PostgreSQL
DB_ENGINE=postgres
DB_USER=user
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

### Lancement

```bash
cargo run
```

Ouvrez [http://localhost:8000](http://localhost:8000)

---

## 📚 Documentation complète

### Guides français

- [🚀 Guide de démarrage](docs/fr/GETTING_STARTED.md)
- [⚙️ Configuration](docs/fr/CONFIGURATION.md)
- [🗄️ Base de données](docs/fr/DATABASE.md)
- [📝 Formulaires](docs/fr/FORMULAIRE.md)
- [🎨 Templates](docs/fr/TEMPLATES.md)
- [🔒 Sécurité](docs/fr/SECURITY.md)
- [🛣️ Routing](docs/fr/ROUTING.md)
- [🔧 Middleware](docs/fr/MIDDLEWARE.md)
- [🚀 Déploiement](docs/fr/DEPLOIEMENT.md)

### English guides

- [🚀 Getting Started](docs/en/GETTING_STARTED.md)
- [⚙️ Configuration](docs/en/CONFIGURATION.md)
- [🗄️ Database](docs/en/DATABASE.md)
- [📝 Forms](docs/en/FORMS.md)
- [🎨 Templates](docs/en/TEMPLATES.md)
- [🔒 Security](docs/en/SECURITY.md)
- [🛣️ Routing](docs/en/ROUTING.md)
- [🔧 Middleware](docs/en/MIDDLEWARE.md)
- [🚀 Deployment](docs/en/DEPLOYMENT.md)

---

## 🎯 Exemple complet

### Structure du projet

```
my_app/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── models/
│   │   └── mod.rs
│   ├── views/
│   │   └── mod.rs
│   ├──  forms/
│   |   └── mod.rs
│   └── urls/
│       └── mod.rs
├── templates/
│   ├── base.html
│   └── index.html
└── static/
    ├── css/
    └── js/
```

### Modèle (models/mod.rs)

```rust
use sea_orm::entity::prelude::*;
use rusti::impl_objects;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// API Django-like
impl_objects!(Entity);
```

### Formulaire (forms/mod.rs)

```rust
use rusti::forms::prelude::*;

#[derive(DeriveModelForm, Debug, Clone, Serialize, Deserialize)]
#[sea_orm(model = "crate::models::Model", entity = "crate::models::Entity")]
pub struct PostForm {
    #[form_field(widget = "textarea", required = true)]
    pub title: CharField,

    #[form_field(widget = "textarea", required = true)]
    pub content: CharField,

    #[form_field(default = "false")]
    pub published: BooleanField,
}
```

### Vue (views/mod.rs)

```rust
use rusti::prelude::*;
use crate::models::{posts, Entity as Post};
use crate::forms::PostForm;

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

pub async fn create_post(
    Form(form): Form<PostForm>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
    mut message: Message,
) -> Response {
    if !form.is_valid() {
        return template.render("posts/create.html", context! { form });
    }

    match form.save(&*db).await {
        Ok(post) => {
            success!(message, "Article créé avec succès !");
            redirect(&format!("/posts/{}", post.id))
        }
        Err(_) => {
            error!(message, "Erreur lors de la création");
            template.render("posts/create.html", context! { form })
        }
    }
}

```

### Template (templates/posts/list.html)

```html
{% extends "base.html" %}

{% block content %}
<h1>Articles</h1>

{% for post in posts %}
<article>
    <h2>{{ post.title }}</h2>
    <p>{{ post.content|truncate(200) }}</p>
    <a href="{% link 'post_detail' id=post.id %}">Lire la suite</a>
</article>
{% endfor %}

<a href="{% link 'post_create' %}">Créer un article</a>
{% endblock %}
```

### Routes (main.rs)

```rust
use rusti::prelude::*;

fn routes() -> Router {
    urlpatterns![
        path!("", views::index, "index"),
        path!("posts/", views::list_posts, "post_list"),
        path!("posts/create/", views::create_post, "post_create"),
        path!("posts/<id>/", views::detail_post, "post_detail"),
    ]
}
```

---

## 🔒 Sécurité

Rusti intègre plusieurs couches de sécurité par défaut :

### Protection CSRF

```rust
RustiApp::new(settings).await?
    .middleware(CsrfMiddleware::new())
    .routes(routes())
    .run()
    .await?;
```

Dans les templates :
```html
<form method="post">
    {% csrf %}
    <!-- champs du formulaire -->
</form>
```

### Content Security Policy

```rust
use rusti::middleware::CspConfig;

let csp_config = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    use_nonce: true,
    ..Default::default()
};

RustiApp::new(settings).await?
    .middleware(CspMiddleware::new(csp_config))
    .routes(routes())
    .run()
    .await?;
```

### Security Headers

```rust
RustiApp::new(settings).await?
    .middleware(SecurityHeadersMiddleware::new())
    .routes(routes())
    .run()
    .await?;
```

Headers configurés automatiquement :
- `Strict-Transport-Security`
- `X-Content-Type-Options`
- `X-Frame-Options`
- `X-XSS-Protection`
- `Referrer-Policy`
- `Permissions-Policy`

---

## 🗄️ Base de données

### API Django-like

```rust
use crate::models::{users, Entity as User};

// Récupération
let all_users = User::objects.all().all(&db).await?;
let user = User::objects.get(&db, 1).await?;

// Filtrage
let active_users = User::objects
    .filter(users::Column::IsActive.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&db)
    .await?;

// Tri et pagination
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// Comptage
let count = User::objects.count(&db).await?;
```

### Migrations

Utilisez `sea-orm-cli` pour les migrations :

```bash
cargo install sea-orm-cli

# Créer une migration
sea-orm-cli migrate generate create_users_table

# Appliquer
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down
```

---

## 🎨 Templates

### Tags personnalisés

```html
<!-- Fichiers statiques -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src="{% static 'js/app.js' %}"></script>

<!-- Fichiers media -->
<img src="{% media user.avatar %}" alt="Avatar">

<!-- Token CSRF -->
<form method="post">
    {% csrf %}
    <!-- ... -->
</form>

<!-- Messages flash -->
{% messages %}

<!-- Liens avec reverse routing -->
<a href="{% link 'post_detail' id=post.id %}">Détails</a>

<!-- CSP nonce (si activé) -->
<script {{ csp }}>
    // Code JavaScript
</script>
```

---

## 📦 Macros utilitaires

Rusti fournit des macros pour simplifier les opérations courantes.

### Messages Flash

```rust
use rusti::prelude::*;

async fn my_handler(mut message: Message) -> Response {
    // Messages simples
    success!(message, "Opération réussie !");
    error!(message, "Une erreur est survenue");
    info!(message, "Information importante");
    warning!(message, "Attention");

    // Messages multiples
    success!(
        message,
        "Utilisateur créé",
        "Email envoyé",
        "Bienvenue !"
    );

    redirect("/")
}
```

**Avantages :**
- Syntaxe concise et expressive
- Gestion automatique de `.await.unwrap()`
- Support de messages multiples
- Code plus lisible et maintenable

**Macros disponibles :**
- `success!(message, "text")` - Messages de succès
- `error!(message, "text")` - Messages d'erreur
- `info!(message, "text")` - Messages d'information
- `warning!(message, "text")` - Messages d'avertissement

---

## 🚀 Performance

Rusti exploite les performances de Rust et Tokio :

- **Zéro-cost abstractions** : Aucun overhead à l'exécution
- **Async/await natif** : Concurrence efficace avec Tokio
- **Connection pooling** : Gestion optimisée des connexions DB
- **Compilation optimisée** : Binaire hautement optimisé

### Benchmark (exemple)

```
Requêtes/sec : ~50,000
Latence p50 : ~1ms
Latence p99 : ~5ms
Mémoire : ~20MB
```

---

## 🛠️ Développement

### Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatage

```bash
cargo fmt
```

### Documentation

```bash
cargo doc --open
```

---

## 🤝 Contribution

Les contributions sont les bienvenues ! Voici comment contribuer :

1. Fork le projet
2. Créez une branche (`git checkout -b feature/amazing-feature`)
3. Committez vos changements (`git commit -m 'Add amazing feature'`)
4. Push vers la branche (`git push origin feature/amazing-feature`)
5. Ouvrez une Pull Request

### Directives

- Écrivez des tests pour les nouvelles fonctionnalités
- Suivez les conventions de code Rust (rustfmt)
- Documentez les API publiques
- Ajoutez des exemples si pertinent

---

## 📝 Roadmap

### Version 1.1 (Q1 2026)

- [ ] Authentication system intégré
- [ ] Admin panel auto-généré
- [ ] Rate limiting middleware
- [ ] WebSocket support
- [ ] Cache layer (Redis)

### Version 1.2 (Q2 2026)

- [ ] CLI pour scaffolding
- [ ] Hot reload amélioré
- [ ] GraphQL support
- [ ] Background jobs (Tokio tasks)

### Version 2.0 (Q3 2026)

- [ ] Plugin system
- [ ] Multi-tenancy
- [ ] Internationalization (i18n)
- [ ] Advanced ORM features

---

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE](LICENSE) pour plus de détails.

---

## 🙏 Remerciements

Rusti s'appuie sur d'excellentes bibliothèques de l'écosystème Rust :

- [Axum](https://github.com/tokio-rs/axum) - Framework web
- [Tokio](https://tokio.rs/) - Runtime async
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM
- [Tera](https://keats.github.io/tera/) - Moteur de templates
- [Tower](https://github.com/tower-rs/tower) - Middleware
- [Argon2](https://github.com/RustCrypto/password-hashes) - Hachage de mots de passe
- [ammonia](https://github.com/rust-ammonia/ammonia) - Sanitization HTML

---

## 📧 Contact

- **GitHub Issues** : [github.com/votre-username/rusti/issues](https://github.com/votre-username/rusti/issues)
- **Discord** : [Rejoindre le serveur](#)
- **Email** : contact@rusti-framework.dev

---

## ⭐ Soutenez le projet

Si Rusti vous est utile, pensez à :

- ⭐ Mettre une étoile sur GitHub
- 🐛 Signaler des bugs
- 💡 Proposer des fonctionnalités
- 📖 Améliorer la documentation
- 🤝 Contribuer au code

---

**Développez des applications web sécurisées et performantes avec Rusti !**

---

**Version:** 1.0.0 (Corrigée - 2 Janvier 2026)
**Licence:** MIT