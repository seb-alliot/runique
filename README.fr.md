# Runique

**Un framework web Rust inspiré de Django**

Runique est un framework web moderne qui combine la sécurité et les performances de Rust avec l'ergonomie de Django. Il offre une expérience de développement familière aux développeurs Django tout en tirant parti de la puissance du système de types de Rust.

[![Version](https://img.shields.io/badge/version-0.1.86-blue.svg)](https://crates.io/crates/runique)
[![docs.rs](https://img.shields.io/docsrs/runique)](https://docs.rs/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🤔 Pourquoi Runique ?

- **Pour les développeurs Django** : API et patterns familiers avec les performances et la sécurité de Rust
- **Pour les développeurs Rust** : Ergonomie inspirée de Django sans sacrifier la sécurité des types
- **Pour tous** : Sécurité intégrée dès le départ, pas ajoutée après coup

---

## 🚀 Fonctionnalités principales

### Architecture à la Django
- **Routage déclaratif** avec la macro `urlpatterns!`
- **ORM intuitif** basé sur SeaORM avec une API style Django (`User::objects.filter(...)`)
- **Système de templates** Tera avec prétraitement personnalisé et tags Django-like
- **Génération automatique de formulaires** via macros procédurales (`#[derive(DeriveModelForm)]`)
- **Messages flash** entre les requêtes avec sécurité des types
- **Gestion des fichiers statiques et médias**

### Sécurité intégrée
- ✅ **Protection CSRF** (HMAC-SHA256 avec masquage de token contre les attaques BREACH)
- ✅ **Content Security Policy** (CSP) avec génération automatique de nonce
- ✅ **Sanitisation XSS** avec sanitisation automatique des entrées
- ✅ **En-têtes de sécurité automatiques** (HSTS, X-Frame-Options, etc.)
- ✅ **Validation ALLOWED_HOSTS** avec support des sous-domaines wildcard
- ✅ **Hachage Argon2id intégré** pour les mots de passe
- ✅ **Middleware Login Required** pour la protection par authentification

### Système de formulaires avancé
- **Génération HTML automatique** depuis les modèles
- **Validation intégrée** avec règles personnalisées
- **Types de champs** : CharField, EmailField, PasswordField, IntegerField, DateField, URLField, SlugField, FileField, SelectField, et plus
- **Intégration SeaORM** avec conversion automatique des modèles
- **Gestion des erreurs** avec détection des contraintes de base de données
- **Protection CSRF** intégrée dans les formulaires

### Support multi-bases de données
- PostgreSQL
- MySQL / MariaDB
- SQLite
- Pool de connexions et configuration des timeouts
- Détection automatique du driver depuis l'URL
- Changement facile de base de données via variables d'environnement

### Développement moderne
- **Async/await natif** avec Tokio
- **Type-safe** grâce au système de types de Rust
- **Abstractions à coût zéro**
- **Outil CLI** pour la génération de projets
- **Hot reload** en développement
- **Documentation complète** avec exemples

---

## 📦 Installation

### Prérequis

- Rust 1.75+ ([installer Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Ajouter Runique à votre projet

```toml
# Cargo.toml

# Configuration minimale (SQLite)
[dependencies]
runique = { version = "0.1.86", features = ["sqlite"] }

# Avec PostgreSQL
[dependencies]
runique = { version = "0.1.86", features = ["postgres"] }

# Avec MySQL
[dependencies]
runique = { version = "0.1.86", features = ["mysql"] }

# Avec MariaDB
[dependencies]
runique = { version = "0.1.86", features = ["mariadb"] }

# Avec plusieurs bases de données (PostgreSQL + SQLite)
[dependencies]
runique = { version = "0.1.86", features = ["postgres", "sqlite"] }

# Avec toutes les bases de données
[dependencies]
runique = { version = "0.1.86", features = ["all-databases"] }
```

### Features Cargo disponibles

| Feature | Description | Défaut |
|---------|-------------|--------|
| `orm` | Active SeaORM | ✅ |
| `sqlite` | Driver SQLite | ❌ (doit être activé explicitement) |
| `postgres` | Driver PostgreSQL | ❌ (doit être activé explicitement) |
| `mysql` | Driver MySQL | ❌ (doit être activé explicitement) |
| `mariadb` | Driver MariaDB (utilise le driver MySQL) | ❌ (doit être activé explicitement) |
| `all-databases` | Active tous les drivers simultanément | ❌ (doit être activé explicitement) |

**Note :** Vous devez explicitement spécifier au moins un driver de base de données.

**Exemples de configuration :**

```toml
# SQLite uniquement
[dependencies]
runique = { version = "0.1.86", features = ["sqlite"] }

# PostgreSQL uniquement
[dependencies]
runique = { version = "0.1.86", features = ["postgres"] }

# PostgreSQL + MySQL
[dependencies]
runique = { version = "0.1.86", features = ["postgres", "mysql"] }

# Toutes les bases de données
[dependencies]
runique = { version = "0.1.86", features = ["all-databases"] }

# Sans ORM (framework minimal)
[dependencies]
runique = { version = "0.1.86", default-features = false }
```

### Créer un nouveau projet avec le CLI

```bash
# Installer le CLI Runique
cargo install runique

# Créer un nouveau projet (génère une structure complète)
runique new mon_app
cd mon_app

# Lancer le projet
cargo run
```

Le CLI génère une structure de projet complète avec :
- `Cargo.toml` pré-configuré
- Modèle utilisateur avec authentification
- Formulaires d'inscription et de connexion
- Fichiers statiques (CSS avec thème sombre)
- Templates avec design responsive
- Migrations de base de données prêtes
- Configuration d'environnement

---

## 🛠️ Outil CLI

Runique fournit un outil CLI puissant pour créer des projets avec une structure complète et prête pour la production.

### Créer un nouveau projet

```bash
# Installer le CLI (si pas déjà installé)
cargo install runique

# Créer un nouveau projet
runique new my_app

# Naviguer vers le projet
cd my_app

# Lancer l'application
cargo run
```

### Structure du projet généré

```
my_app/
├── Cargo.toml (pré-configuré avec Runique)
├── .env (configuration base de données)
├── .gitignore
├── README.md
├── src/
│   ├── main.rs (point d'entrée)
│   ├── forms.rs (définitions de formulaires)
│   ├── url.rs (patterns d'URL)
│   ├── views.rs (gestionnaires de vues)
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs (exemple de modèle User)
│   ├── static/
│   │   ├── css/ (thème sombre responsive inclus)
│   │   │   ├── main.css
│   │   │   ├── variables.css
│   │   │   ├── about.css
│   │   │   ├── register-form.css
│   │   │   └── search-user.css
│   │   ├── js/
│   │   └── images/
│   └── media/
│       ├── favicon/
│       │   └── favicon.ico
│       └── toshiro.jpg (image d'exemple)
└── templates/
    ├── index.html
    ├── about/
    │   └── about.html
    └── profile/
        ├── register_user.html
        └── view_user.html
```

Le projet généré inclut :
- ✅ Exemple CRUD complet avec modèle User
- ✅ Validation de formulaire et gestion des erreurs
- ✅ CSS responsive avec thème sombre
- ✅ Protection CSRF activée
- ✅ Messages flash configurés
- ✅ Migrations de base de données prêtes
- ✅ Exemples de middleware d'authentification

---

## 🏁 Démarrage rapide

### Application minimale

```rust
// src/main.rs
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}

fn routes() -> Router<Arc<Tera>> {
    urlpatterns![
        "/" => get(index), name = "index",
        "/hello/{name}" => get(hello), name = "hello"
    ]
}

async fn index(template: Template) -> Response {
    let ctx = context!();
    template.render("index.html", &ctx)
}

async fn hello(
    Path(name): Path<String>,
    template: Template
) -> Response {
    let ctx = context! {
        "name", name
    };
    template.render("hello.html", &ctx)
}
```

### Configuration (.env)

```env
# Configuration serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=votre-cle-secrete-a-changer-en-production
ALLOWED_HOSTS=localhost,127.0.0.1

# Mode Debug (désactiver en production)
DEBUG=true

# Configuration base de données (exemple PostgreSQL)
DB_ENGINE=postgres
DB_USER=monuser
DB_PASSWORD=monmotdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mabase

# Ou SQLite (par défaut)
DB_ENGINE=sqlite
DB_NAME=app.db
```

### Lancement

```bash
cargo run
```

Ouvrir [http://localhost:3000](http://localhost:3000)

---

## 📚 Documentation

- [🚀 Guide de démarrage](informations/documentation_french/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_french/CONFIGURATION.md)
- [🗄️ Base de données](informations/documentation_french/DATABASE.md)
- [📝 Formulaires](informations/documentation_french/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_french/TEMPLATES.md)
- [🔒 Sécurité](informations/documentation_french/CSP.md)
- [🛣️ Macros](informations/documentation_french/MACRO_CONTEXT.md)
- [🔧 Changelog](informations/documentation_french/CHANGELOG.md)
- [🚀 Contribuer](informations/documentation_french/CONTRIBUTING.md)
- [🆕 Nouveau projet](informations/documentation_french/NEW_PROJECT.md)
- [📖 Documentation API](https://docs.rs/runique)

---

## 🎯 Exemple complet

### Structure du projet
**Générée automatiquement avec `runique new nom_projet`**

```
mon_app/
├── Cargo.toml
├── .env
├── .gitignore
├── README.md
├── src/
│   ├── main.rs
│   ├── forms.rs
│   ├── url.rs
│   ├── views.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── users.rs
│   ├── static/
│   │   ├── css/
│   │   │   ├── main.css
│   │   │   ├── variables.css
│   │   │   ├── register-form.css
│   │   │   ├── search-user.css
│   │   │   └── about.css
│   │   ├── js/
│   │   └── images/
│   └── media/
│       └── favicon/
│           └── favicon.ico
└── templates/
    ├── index.html
    ├── about/
    │   └── about.html
    └── profile/
        ├── register_user.html
        └── view_user.html
```

### Définition de modèle avec SeaORM

```rust
// src/models/users.rs
use sea_orm::entity::prelude::*;
use runique::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    #[sea_orm(unique)]
    pub username: String,

    #[sea_orm(unique)]
    pub email: String,

    pub password: String,
    pub age: i32,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Ajouter les méthodes ORM style Django
impl_objects!(Entity);
```

### Génération automatique de formulaire

```rust
// src/forms.rs
use runique::prelude::*;
use crate::models::users;

// Générer le formulaire automatiquement depuis le modèle
#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// Le formulaire inclura :
// - username (CharField)
// - email (EmailField - auto-détecté)
// - password (PasswordField - auto-haché avec Argon2)
// - age (IntegerField)
// - Protection CSRF token
// - Validation automatique
// - Gestion des erreurs
```

### Handler avancé avec formulaire

```rust
// src/views.rs
use runique::prelude::*;
use crate::forms::UserForm;
use crate::models::{users, Entity as User};

// Afficher le formulaire (GET)
pub async fn register_form(template: Template) -> Response {
    let form = UserForm::build(template.tera.clone());

    let ctx = context! {
        "title", "Inscription utilisateur";
        "form", form
    };

    template.render("profile/register_user.html", &ctx)
}

// Gérer la soumission du formulaire (POST)
pub async fn register(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    // Validation automatique
    if form.is_valid() {
        match form.save(&db).await {
            Ok(user) => {
                success!(message, "Inscription réussie ! Bienvenue !");

                let url = reverse_with_parameters(
                    "user_profile",
                    &[("id", &user.id.to_string())]
                ).unwrap();

                return Redirect::to(&url).into_response();
            }
            Err(err) => {
                // Détection automatique des erreurs de base de données
                let mut form = form;
                form.get_form_mut().handle_database_error(&err);

                let ctx = context! {
                    "title", "Erreur d'inscription";
                    "form", form;
                    "messages", flash_now!(error, "Une erreur s'est produite")
                };

                return template.render("profile/register_user.html", &ctx);
            }
        }
    }

    // Erreurs de validation
    let ctx = context! {
        "title", "Erreur de validation";
        "form", form;
        "messages", flash_now!(error, "Veuillez corriger les erreurs")
    };

    template.render("profile/register_user.html", &ctx)
}

// Afficher le profil utilisateur
pub async fn user_profile(
    Path(id): Path<i32>,
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Requête style Django avec gestion d'erreur
    match User::objects.get_or_404(&db, id, &template, "Utilisateur introuvable").await {
        Ok(user) => {
            let ctx = context! {
                "title", "Profil utilisateur";
                "user", user
            };
            template.render("profile/view_user.html", &ctx)
        }
        Err(response) => response
    }
}

// Lister les utilisateurs avec filtrage
pub async fn user_list(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    template: Template,
) -> Response {
    // Requête ORM style Django
    let users = User::objects
        .filter(users::Column::Age.gte(18))
        .order_by_desc(users::Column::CreatedAt)
        .limit(20)
        .all(&db)
        .await
        .unwrap_or_default();

    let ctx = context! {
        "title", "Liste des utilisateurs";
        "users", users
    };

    template.render("profile/user_list.html", &ctx)
}
```

### Templates avec syntaxe Django-like

```html
<!-- templates/profile/register_user.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{{ title }}</title>
    <link rel="stylesheet" href="{% static 'css/main.css' %}">
    <link rel="stylesheet" href="{% static 'css/register-form.css' %}">
</head>
<body>
    <div class="container">
        <h1>{{ title }}</h1>

        <!-- Messages flash -->
        {% messages %}

        <!-- Formulaire avec protection CSRF automatique -->
        <form method="post" action="{% link 'register' %}">
            {% csrf %}

            <!-- Rendu automatique du formulaire -->
            {% form.register_form %}

            <!-- Ou rendre des champs spécifiques -->
            {% form.register_form.username %}
            {% form.register_form.email %}
            {% form.register_form.password %}
            {% form.register_form.age %}

            <button type="submit">S'inscrire</button>
        </form>

        <p>
            Déjà inscrit ?
            <a href="{% link 'login' %}">Se connecter</a>
        </p>
    </div>

    <!-- JavaScript conforme CSP -->
    <script {{ csp }}>
        console.log('Formulaire d\'inscription chargé');
    </script>
</body>
</html>
```

### Configuration du routage

```rust
// src/url.rs
use runique::prelude::*;
use crate::views;

pub fn routes() -> Router<Arc<Tera>> {
    urlpatterns![
        // Routes publiques
        "/" => get(views::index), name = "index",
        "/about" => get(views::about), name = "about",

        // Authentification
        "/register" => get(views::register_form)
                      .post(views::register),
                      name = "register",

        "/login" => get(views::login_form)
                   .post(views::login),
                   name = "login",

        "/logout" => post(views::logout), name = "logout",

        // Routes protégées (avec middleware login_required)
        "/profile/{id}" => get(views::user_profile)
                          .layer(middleware::from_fn(login_required)),
                          name = "user_profile",

        "/users" => get(views::user_list)
                   .layer(middleware::from_fn(login_required)),
                   name = "user_list"
    ]
}
```

### Configuration de l'application principale

```rust
// src/main.rs
use runique::prelude::*;

mod models;
mod forms;
mod views;
mod url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charger les paramètres
    let settings = Settings::builder()
        .debug(true)
        .server("127.0.0.1", 3000, "cle-secrete")
        .sanitize_inputs(true)
        .build();

    // Connexion base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Construire et lancer l'application
    RuniqueApp::new(settings).await?
        .with_database(db)
        .with_static_files()?
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .routes(url::routes())
        .run()
        .await?;

    Ok(())
}
```

---

## 🗄️ Base de données

### Configuration avec le pattern Builder

```rust
use runique::prelude::*;

// Depuis les variables d'environnement
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

// Ou avec configuration personnalisée
let db_config = DatabaseConfig::from_url("sqlite://app.db")?
    .max_connections(50)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(10))
    .logging(true)
    .build();
```

### API ORM style Django

```rust
use crate::models::{users, Entity as User};

// Tous les enregistrements
let all_users = User::objects.all().all(&db).await?;

// Récupérer par ID
let user = User::objects.get(&db, 1).await?;

// Récupérer par ID (retourne Option)
let user: Option<Model> = User::objects.get_optional(&db, 1).await?;

// Récupérer ou 404 (réponse d'erreur automatique)
let user = User::objects.get_or_404(
    &db,
    1,
    &template,
    "Utilisateur introuvable"
).await?;

// Filtrage
let active_users = User::objects
    .filter(users::Column::IsActive.eq(true))
    .filter(users::Column::Age.gte(18))
    .all(&db)
    .await?;

// Exclusion
let non_admin_users = User::objects
    .exclude(users::Column::Role.eq("admin"))
    .all(&db)
    .await?;

// Tri
let recent_users = User::objects
    .order_by_desc(users::Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?;

// Pagination
let page_2 = User::objects
    .order_by_asc(users::Column::Username)
    .limit(20)
    .offset(20)
    .all(&db)
    .await?;

// Compter
let total = User::objects.count(&db).await?;

// Obtenir le premier résultat
let first_user = User::objects
    .order_by_asc(users::Column::CreatedAt)
    .first(&db)
    .await?;

// Query Builder avec get_or_404
let user = User::objects
    .filter(users::Column::Username.eq("admin"))
    .get_or_404(&db, &template, "Utilisateur admin introuvable")
    .await?;

// Requêtes complexes
let filtered = User::objects
    .filter(users::Column::Age.gte(18))
    .exclude(users::Column::Status.eq("banned"))
    .order_by_desc(users::Column::CreatedAt)
    .limit(50)
    .all(&db)
    .await?;
```

### Méthodes ORM avancées

```rust
// Méthodes RuniqueQueryBuilder
let query = User::objects
    .filter(users::Column::Age.gte(18))
    .order_by_desc(users::Column::CreatedAt);

// Obtenir tous les résultats
let users: Vec<Model> = query.clone().all(&db).await?;

// Obtenir le premier résultat
let first: Option<Model> = query.clone().first(&db).await?;

// Compter les résultats
let count: u64 = query.clone().count(&db).await?;

// Obtenir le premier ou 404
let user: Model = query
    .get_or_404(&db, &template, "Aucun utilisateur correspondant")
    .await?;
```

### Migrations avec SeaORM CLI

```bash
# Installer le CLI
cargo install sea-orm-cli

# Initialiser les migrations
sea-orm-cli migrate init

# Créer une migration
sea-orm-cli migrate generate create_users_table

# Appliquer les migrations
sea-orm-cli migrate up

# Rollback
sea-orm-cli migrate down

# Vérifier le statut
sea-orm-cli migrate status

# Générer les entités depuis une base de données existante
sea-orm-cli generate entity \
    --database-url "sqlite://app.db" \
    --output-dir src/models

# Pour PostgreSQL
sea-orm-cli generate entity \
    --database-url "postgres://user:password@localhost/mydb" \
    --output-dir src/models

# Pour MySQL
sea-orm-cli generate entity \
    --database-url "mysql://user:password@localhost/mydb" \
    --output-dir src/models
```

**Après avoir généré les entités, n'oubliez pas de :**

1. Ajouter la macro `impl_objects!` pour activer l'ORM style Django :
```rust
// Dans votre fichier d'entité généré (ex: src/models/users.rs)
use runique::prelude::*;

// Après la définition de Entity, ajoutez :
impl_objects!(Entity);
```

2. Générer automatiquement les formulaires depuis vos modèles :
```rust
// Dans src/forms.rs
use runique::prelude::*;

#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// Le formulaire est maintenant prêt avec :
// - Détection automatique des champs
// - Validation intégrée
// - Protection CSRF
// - Gestion des erreurs
// - Intégration base de données
```

### Génération automatique de formulaires

Runique fournit un système puissant de génération de formulaires qui crée automatiquement des formulaires depuis vos modèles SeaORM.

#### Utilisation basique

```rust
use runique::prelude::*;

// Votre modèle SeaORM
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub age: i32,
}

// Générer le formulaire automatiquement
#[derive(DeriveModelForm)]
#[model_form(model = "users::Model")]
pub struct UserForm;

// C'est tout ! Le formulaire inclut maintenant :
// ✅ Détection automatique du type de champ (CharField, EmailField, PasswordField, IntegerField)
// ✅ Génération HTML pour chaque champ
// ✅ Validation intégrée
// ✅ Protection CSRF
// ✅ Gestion des erreurs avec messages conviviaux
// ✅ Intégration SeaORM (sauvegarde directe en base)
```

#### Détection des types de champs

Le générateur de formulaires détecte automatiquement les types de champs selon :

1. **Noms des champs** (détection intelligente) :
   - `email` → EmailField (avec validation email)
   - `password`, `pwd` → PasswordField (haché automatiquement avec Argon2)
   - `url`, `link`, `website` → URLField
   - `slug` → SlugField
   - `description`, `bio`, `content`, `text` → TextField (textarea)

2. **Types Rust** :
   - `String` → CharField
   - `i32`, `i64` → IntegerField
   - `f32`, `f64` → FloatField
   - `bool` → BooleanField (case à cocher)
   - `NaiveDate` → DateField
   - `NaiveDateTime`, `DateTime` → DateTimeField
   - `IpAddr` → IPAddressField
   - `Value`, `Json` → JSONField

3. **Champs optionnels** (`Option<T>`) :
   - Automatiquement détectés comme optionnels
   - Pas d'erreur de validation si laissé vide

#### Utilisation du formulaire

```rust
// Afficher le formulaire (requête GET)
pub async fn register_form(template: Template) -> Response {
    let form = UserForm::build(template.tera.clone());

    let ctx = context! {
        "form", form
    };

    template.render("register.html", &ctx)
}

// Gérer la soumission (requête POST)
pub async fn register(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    ExtractForm(form): ExtractForm<UserForm>,
) -> Response {
    if form.is_valid() {
        // Sauvegarder directement en base de données
        match form.save(&db).await {
            Ok(user) => {
                // Utilisateur créé avec succès
                Redirect::to("/success").into_response()
            }
            Err(err) => {
                // Gérer les erreurs de base de données
                let mut form = form;
                form.get_form_mut().handle_database_error(&err);
                // Re-rendre avec les erreurs
            }
        }
    }

    // Re-rendre avec les erreurs de validation
}
```

#### Rendu dans les templates

```html
<!-- Rendre le formulaire complet -->
{% form.user_form %}

<!-- Ou rendre des champs spécifiques -->
<div class="form-group">
    {% form.user_form.username %}
</div>
<div class="form-group">
    {% form.user_form.email %}
</div>
<div class="form-group">
    {% form.user_form.password %}
</div>
```

#### Validation personnalisée

```rust
// Ajouter une logique de validation personnalisée
impl UserForm {
    pub fn validate_custom(&mut self) -> bool {
        let form = self.get_form_mut();

        // Accéder aux valeurs des champs
        if let Some(age) = form.get_value::<i64>("age") {
            if age < 18 {
                form.add_error("age", "Doit avoir 18 ans ou plus");
                return false;
            }
        }

        self.is_valid()
    }
}
```

#### Gestion des erreurs de base de données

Le système de formulaires détecte automatiquement les erreurs courantes de base de données :

```rust
// Gère automatiquement :
// ✅ Violations de contraintes uniques
// ✅ Erreurs spécifiques aux champs (username, email, etc.)
// ✅ Messages d'erreur conviviaux

match form.save(&db).await {
    Ok(user) => { /* Succès */ }
    Err(err) => {
        form.get_form_mut().handle_database_error(&err);
        // Message d'erreur comme "Ce nom d'utilisateur est déjà utilisé"
        // automatiquement ajouté à form.errors
    }
}
```

#### Fonctionnalités avancées

```rust
// Accès manuel aux champs
let username: Option<String> = form.get_value("username");
let age: Option<i64> = form.get_value("age");

// Vérifier les erreurs de champs spécifiques
if let Some(error) = form.get_errors().get("email") {
    println!("Erreur email : {}", error);
}

// Ajouter des erreurs manuellement
form.get_form_mut().add_error("custom_field", "Message d'erreur personnalisé");

// Convertir en ActiveModel pour opérations avancées
let active_model = form.to_active_model();
```

---

## 🎨 Templates

### Tags de template Django-like

```html
<!-- Fichiers statiques -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src="{% static 'js/main.js' %}"></script>

<!-- Fichiers médias (uploads utilisateur) -->
<img src="{% media 'avatars/user.jpg' %}" alt="Avatar">

<!-- Assets internes Runique -->
<link rel="stylesheet" href="{% runique_static 'css/error.css' %}">

<!-- Token CSRF (protection automatique) -->
<form method="post">
    {% csrf %}
    <!-- champs du formulaire -->
</form>

<!-- Messages flash -->
{% messages %}

<!-- Inversion d'URL -->
<a href="{% link 'home' %}">Accueil</a>
<a href="{% link 'user_profile' id=user.id %}">Profil</a>
<a href="{% link 'post_detail' slug=post.slug id=post.id %}">Lire la suite</a>

<!-- Nonce CSP pour scripts inline -->
<script {{ csp }}>
    // Ce script est conforme CSP
    console.log('JavaScript sécurisé');
</script>

<!-- Formulaires (rendu automatique) -->
{% form.user_form %}

<!-- Ou rendre des champs spécifiques -->
{% form.user_form.username %}
{% form.user_form.email %}
```

### Contexte de template depuis un handler

```rust
use runique::prelude::*;

async fn mon_handler(template: Template) -> Response {
    let ctx = context! {
        "title", "Ma Page";
        "user", user;
        "count", 42;
        "items", vec!["a", "b", "c"]
    };

    template.render("mon_template.html", &ctx)
}
```

### Template Processor (Extractor)

L'extracteur `Template` injecte automatiquement des variables communes dans vos templates :

```rust
use runique::prelude::*;

async fn handler(template: Template) -> Response {
    // Déjà disponible dans les templates sans insertion manuelle :
    // - csrf_token (protection CSRF)
    // - messages (messages flash)
    // - debug (indicateur mode debug)
    // - csp_nonce (nonce CSP pour scripts inline)
    // - static_runique (URL statique interne de Runique)

    let ctx = context! { "user", user };
    template.render("profile.html", &ctx)
}

// Codes de statut personnalisés
async fn not_found(template: Template) -> Response {
    let ctx = context! { "reason", "Page non trouvée" };
    template.render_with_status("404.html", &ctx, StatusCode::NOT_FOUND)
}

// Méthodes helper
async fn error_handler(template: Template) -> Response {
    template.render_404("Cette ressource n'existe pas")
    // ou
    template.render_500("Une erreur est survenue")
}
```

### Message Extractor (Messages Flash)

L'extracteur `Message` fournit une API pratique pour les messages flash :

```rust
use runique::prelude::*;

async fn create_user(mut message: Message) -> Response {
    // Envoyer un message de succès
    message.success("Utilisateur créé avec succès").await?;

    // Ou envoyer plusieurs messages
    message.success("Utilisateur créé").await?;
    message.info("Email de vérification envoyé").await?;

    Redirect::to("/users").into_response()
}

async fn handle_form(mut message: Message, form: ExtractForm<UserForm>) -> Response {
    if form.is_valid() {
        message.success("Formulaire enregistré !").await?;
    } else {
        message.error("Données de formulaire invalides").await?;
        message.warning("Veuillez vérifier votre saisie").await?;
    }

    Redirect::to("/form").into_response()
}
```

### Filtres et fonctions Tera

Runique fournit des filtres et fonctions Tera personnalisés :

```html
<!-- Filtres -->
{{ "style.css" | static }}           <!-- /static/style.css -->
{{ "avatar.jpg" | media }}           <!-- /media/avatar.jpg -->
{{ "error.css" | runique_static }}   <!-- /runique/static/error.css -->

<!-- Rendu de formulaire -->
{{ user_form | form }}               <!-- Rend le formulaire complet -->
{{ user_form | form(field='email') }}  <!-- Rend un seul champ -->

<!-- Reversement d'URL avec paramètres -->
{{ link(link='user_detail', id=123) }}
{{ link(link='post_detail', slug='my-post', id=456) }}

<!-- Nonce CSP pour scripts inline -->
<script {{ csp }}>
    console.log('Script conforme CSP');
</script>
```

---

## 📦 Macros utilitaires

### Messages flash

```rust
use runique::prelude::*;

async fn mon_handler(mut message: Message) -> Response {
    // Messages simples
    success!(message, "Opération réussie !");
    error!(message, "Une erreur s'est produite");
    info!(message, "Information importante");
    warning!(message, "Avertissement");

    // Messages multiples
    success!(
        message,
        "Utilisateur créé",
        "Email envoyé",
        "Bienvenue !"
    );

    Redirect::to("/").into_response()
}

// Ou utiliser flash_now! pour affichage immédiat
async fn afficher_erreur(template: Template) -> Response {
    let ctx = context! {
        "messages", flash_now!(error, "Identifiants invalides")
    };
    template.render("login.html", &ctx)
}
```

### Macro Context

```rust
// Paires clé-valeur simples
let ctx = context! {
    "name", "Jean";
    "age", 30;
    "active", true
};

// Fonctionne avec tout type Serialize
let ctx = context! {
    "user", user_model;
    "posts", posts_vec;
    "metadata", json!({"key": "value"})
};

// Contexte vide
let ctx = context!();
```

### Inversion d'URL

```rust
// URL simple
let url = reverse("home").unwrap();

// URL avec paramètres
let url = reverse_with_parameters(
    "user_profile",
    &[("id", "123")]
).unwrap();

// Paramètres multiples
let url = reverse_with_parameters(
    "post_detail",
    &[
        ("slug", "mon-article"),
        ("id", "456")
    ]
).unwrap();
```

---

## 🔒 Sécurité

### Fonctionnalités de sécurité intégrées

Runique inclut des fonctionnalités de sécurité complètes activées par défaut :

#### Protection CSRF

```rust
// Automatiquement activé avec le middleware par défaut
RuniqueApp::new(settings).await?
    .with_default_middleware()
    .run()
    .await?;

// Configuration manuelle
RuniqueApp::new(settings).await?
    .with_csrf_tokens()
    .run()
    .await?;
```

Les templates incluent automatiquement les tokens CSRF :
```html
<form method="post">
    {% csrf %}  <!-- Token CSRF automatique -->
    <!-- champs du formulaire -->
</form>
```

#### Content Security Policy

```rust
use runique::prelude::*;

// CSP strict (recommandé pour la production)
RuniqueApp::new(settings).await?
    .with_security_headers(CspConfig::strict())
    .run()
    .await?;

// CSP permissif (pour le développement)
RuniqueApp::new(settings).await?
    .with_csp(CspConfig::permissive())
    .run()
    .await?;

// CSP personnalisé
let csp = CspConfig {
    default_src: vec!["'self'".to_string()],
    script_src: vec!["'self'".to_string()],
    style_src: vec!["'self'".to_string(), "'unsafe-inline'".to_string()],
    img_src: vec!["'self'".to_string(), "https:".to_string()],
    use_nonce: true,
    ..Default::default()
};

RuniqueApp::new(settings).await?
    .with_security_headers(csp)
    .run()
    .await?;
```

#### Validation ALLOWED_HOSTS

```rust
// Depuis .env
// ALLOWED_HOSTS=example.com,www.example.com,.api.example.com

let settings = Settings::from_env();

RuniqueApp::new(settings).await?
    .with_allowed_hosts(None)  // Utilise .env
    .run()
    .await?;

// Ou par programmation
RuniqueApp::new(settings).await?
    .with_allowed_hosts(Some(vec![
        "example.com".to_string(),
        ".api.example.com".to_string()  // Correspond à tous les sous-domaines
    ]))
    .run()
    .await?;
```

#### Sanitisation des entrées

```rust
// Activer la sanitisation automatique
RuniqueApp::new(settings).await?
    .with_sanitize_text_inputs(true)
    .run()
    .await?;
```

Sanitise automatiquement :
- Attaques XSS (tags `<script>`)
- Gestionnaires d'événements JavaScript (`onclick=`, etc.)
- Protocole `javascript:`
- Tags HTML dans les entrées texte
- Préserve le formatage (sauts de ligne, espaces)

#### Middleware d'authentification

```rust
use runique::prelude::*;

// Protéger des routes
let protected_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route("/profile", get(profile))
    .layer(middleware::from_fn(login_required));

// Rediriger les utilisateurs authentifiés
let public_routes = Router::new()
    .route("/login", get(login_form).post(login))
    .layer(middleware::from_fn(redirect_if_authenticated));
```

#### Hachage de mots de passe

```rust
// Automatique avec PasswordField
use runique::formulaire::field::PasswordField;

let field = PasswordField;
let hashed = field.process("user_password").unwrap();
// Retourne un hash Argon2id

// Hachage manuel
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, rand_core::OsRng};

let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let hash = argon2.hash_password(b"password", &salt)
    .unwrap()
    .to_string();
```

### En-têtes de sécurité

Tous les en-têtes de sécurité activés avec `.with_security_headers()` :

- ✅ Content-Security-Policy
- ✅ X-Content-Type-Options: nosniff
- ✅ X-Frame-Options: DENY
- ✅ X-XSS-Protection: 1; mode=block
- ✅ Referrer-Policy: strict-origin-when-cross-origin
- ✅ Permissions-Policy
- ✅ Cross-Origin-Embedder-Policy
- ✅ Cross-Origin-Opener-Policy
- ✅ Cross-Origin-Resource-Policy

---

## 🔐 Authentification & Autorisation

### Middleware d'authentification

Runique fournit des middleware intégrés pour protéger les routes :

```rust
use runique::prelude::*;

// Routes protégées (nécessitent authentification)
let protected_routes = Router::new()
    .route("/dashboard", get(dashboard))
    .route("/profile", get(profile))
    .layer(middleware::from_fn(login_required));

// Routes publiques (redirige les utilisateurs authentifiés)
let public_routes = Router::new()
    .route("/login", get(login_page))
    .route("/register", get(register_page))
    .layer(middleware::from_fn(redirect_if_authenticated));
```

### Gestion des sessions

```rust
use runique::prelude::*;
use runique::middleware::login_requiert::*;

// Connecter un utilisateur
async fn login(session: Session, form: ExtractForm<LoginForm>) -> Response {
    if let Some(user) = authenticate_user(&form).await {
        login_user(&session, user.id, &user.username).await?;
        Redirect::to("/dashboard").into_response()
    } else {
        // Gérer l'erreur
    }
}

// Déconnecter un utilisateur
async fn logout(session: Session) -> Response {
    logout(&session).await?;
    Redirect::to("/").into_response()
}

// Vérifier si authentifié
async fn check_auth(session: Session) -> Response {
    if is_authenticated(&session).await {
        // L'utilisateur est connecté
    }
}

// Obtenir les infos utilisateur
async fn get_info(session: Session) -> Response {
    if let Some(user_id) = get_user_id(&session).await {
        if let Some(username) = get_username(&session).await {
            // Utiliser les infos utilisateur
        }
    }
}
```

### Extracteur CurrentUser

Utilisez `load_user_middleware` pour injecter automatiquement les informations utilisateur :

```rust
use runique::prelude::*;
use runique::middleware::login_requiert::{load_user_middleware, CurrentUser};

// Configurer le middleware
let app = Router::new()
    .route("/dashboard", get(dashboard))
    .layer(middleware::from_fn(load_user_middleware));

// Accéder à l'utilisateur actuel dans les handlers
async fn dashboard(Extension(user): Extension<CurrentUser>) -> Response {
    // user.id et user.username sont disponibles
    let ctx = context! {
        "user_id", user.id;
        "username", &user.username
    };

    template.render("dashboard.html", &ctx)
}
```

### Vérification des permissions (Stub)

```rust
use runique::middleware::login_requiert::has_permission;

async fn admin_page(session: Session) -> Response {
    if has_permission(&session, "admin").await {
        // L'utilisateur a la permission admin
    } else {
        // Accès refusé
    }
}
```

**Note** : `has_permission` est actuellement un stub. Vous devrez implémenter la logique complète des permissions avec votre base de données.

---

## 🛡️ Middleware avancés

### Middleware disponibles

Runique fournit plusieurs composants middleware :

```rust
use runique::prelude::*;
use runique::middleware::*;

let app = RuniqueApp::new(settings).await?
    .routes(routes)
    // Gestion des erreurs avec pages 404/500 personnalisées
    .layer(middleware::from_fn(error_handler_middleware))

    // Support des messages flash
    .layer(middleware::from_fn(flash_middleware))

    // Protection CSRF
    .layer(middleware::from_fn(csrf_middleware))

    // Sanitisation des entrées (si activée dans settings)
    .layer(middleware::from_fn_with_state(
        settings.clone(),
        sanitize_middleware
    ))

    // Validation ALLOWED_HOSTS
    .layer(middleware::from_fn(allowed_hosts_middleware))

    // En-têtes de sécurité (CSP, HSTS, etc.)
    .layer(middleware::from_fn_with_state(
        CspConfig::strict(),
        security_headers_middleware
    ))

    // Authentification
    .layer(middleware::from_fn(login_required))

    // Auto-injection de CurrentUser
    .layer(middleware::from_fn(load_user_middleware))

    .run()
    .await?;
```

### Middleware Error Handler

Intercepte automatiquement les erreurs 404 et 500 :

```rust
// Configuré automatiquement avec .with_default_middleware()
// Ou manuellement :
.layer(middleware::from_fn(error_handler_middleware))

// En mode debug : affiche des pages d'erreur détaillées
// En production : affiche les templates 404.html et 500.html personnalisés
```

### Middleware de sanitisation

Sanitise automatiquement les entrées de formulaire pour prévenir les XSS :

```rust
let settings = Settings::builder()
    .sanitize_inputs(true)  // Activer auto-sanitisation
    .build();

// Le middleware sanitise automatiquement :
// - application/x-www-form-urlencoded (formulaires)
// - application/json (APIs)
// - Ignore les champs sensibles (password, token, secret, key)
```

### Fonctions de token CSRF

Gestion avancée des tokens CSRF :

```rust
use runique::utils::*;

// Générer token masqué (protection contre attaque BREACH)
let masked_token = mask_csrf_token(&raw_token);

// Démasquer token pour validation
let raw_token = unmask_csrf_token(&masked_token)?;

// Générer token spécifique à l'utilisateur
let user_token = generate_user_token(&secret_key, &user_id.to_string());
```

---

## 🚀 Performances

Runique tire parti de Rust et Tokio pour des performances exceptionnelles :

- **Abstractions à coût zéro** : Aucun surcoût à l'exécution
- **Async/await natif** : Concurrence efficace avec Tokio
- **Pool de connexions** : Gestion optimisée des connexions DB
- **Compilation optimisée** : Binaire hautement optimisé avec LTO
- **Sécurité mémoire** : Pas de garbage collector, performances prévisibles

### Benchmark (indicatif)

```
Configuration : AMD Ryzen 7 5800X, 32GB RAM
Requêtes/sec : ~60,000
Latence p50 : ~0.8ms
Latence p99 : ~3ms
Mémoire : ~15MB (au repos)
```

*Note : Les performances réelles dépendent de votre matériel et de la complexité de l'application.*

---

## 🛠️ Développement

### Exécuter les tests

```bash
# Lancer tous les tests
cargo test

# Tests avec features spécifiques
cargo test --features sqlite
cargo test --features postgres

# Tests d'intégration
cargo test --test integration

# Tests de documentation
cargo test --doc

# Avec sortie complète
cargo test -- --nocapture
```

### Qualité du code

```bash
# Linting
cargo clippy

# Formatage
cargo fmt

# Vérification sans compilation
cargo check

# Audit de sécurité
cargo audit
```

### Documentation

```bash
# Générer et ouvrir la documentation
cargo doc --open --no-deps

# Tester les exemples de documentation
cargo test --doc

# Vérifier la couverture documentation
cargo doc --document-private-items
```

### Benchmarking

```bash
# Lancer les benchmarks (nécessite nightly)
cargo +nightly bench

# Avec features spécifiques
cargo +nightly bench --features all-databases
```

---

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voici comment contribuer :

1. **Fork le projet**
2. **Créer une branche de fonctionnalité**
   ```bash
   git checkout -b feature/fonctionnalite-incroyable
   ```
3. **Commiter vos changements**
   ```bash
   git commit -m 'Ajout d\'une fonctionnalité incroyable'
   ```
4. **Pousser vers la branche**
   ```bash
   git push origin feature/fonctionnalite-incroyable
   ```
5. **Ouvrir une Pull Request**

### Directives

- ✅ Écrire des tests pour les nouvelles fonctionnalités
- ✅ Suivre les conventions Rust (`cargo fmt`)
- ✅ Documenter les APIs publiques avec exemples
- ✅ Mettre à jour le CHANGELOG.md
- ✅ Ajouter des exemples si pertinent
- ✅ S'assurer que tous les tests passent
- ✅ Lancer `cargo clippy` avant de soumettre

### Configuration développement

```bash
# Cloner le dépôt
git clone https://github.com/seb-alliot/runique.git
cd runique

# Installer les dépendances de développement
cargo install cargo-watch
cargo install cargo-edit
cargo install sea-orm-cli

# Lancer les tests en mode watch
cargo watch -x test

# Lancer avec hot reload
cargo watch -x run
```

Voir [CONTRIBUTING.md](informations/documentation_french/CONTRIBUTING.md) pour plus de détails.

---

## 📝 Feuille de route

### Version 1.1 (Actuelle)
- [x] Outil CLI pour génération de projets
- [x] Système de formulaires complet avec validation
- [x] Protection CSRF avec masquage de token
- [x] CSP avec génération de nonce
- [x] Sanitisation automatique des entrées
- [x] Middleware login/logout
- [ ] Améliorations gestion de session
- [ ] Middleware de limitation de débit

### Version 1.2
- [ ] Générateur de panneau d'administration
- [ ] Support WebSocket
- [ ] Tâches en arrière-plan avec Tokio
- [ ] Couche de cache (Redis)
- [ ] Gestion des uploads de fichiers
- [ ] Intégration email

### Version 2.0
- [ ] Support GraphQL
- [ ] Système de plugins
- [ ] Multi-tenancy
- [ ] Internationalisation (i18n)
- [ ] Fonctionnalités ORM avancées (relations, agrégation)
- [ ] Fonctionnalités temps réel
- [ ] Support microservices

---

## 📚 Référence API

### Macros utilitaires

```rust
// get_or_return! - Unwrap ou retour anticipé
let value = get_or_return!(some_result);
// Équivalent à :
let value = match some_result {
    Ok(v) => v,
    Err(e) => return e,
};

// view! - Routing GET/POST combiné
let route = view!(
    GET => get_handler,
    POST => post_handler
);
```

### Utilitaires de formulaire

```rust
use runique::formulaire::*;

// Vérifier si une valeur contient du contenu dangereux
if is_dangerous("<script>alert('xss')</script>") {
    // Gérer l'entrée dangereuse
}

// Vérifier si un champ est sensible (password, token, secret, key)
if is_sensitive_field("password") {
    // Ne pas sanitiser ce champ
}

// Sanitisation manuelle
let clean = auto_sanitize("<script>alert('xss')</script>");
// Résultat : "alert('xss')"
```

### Helpers de réponse

```rust
use runique::response::*;

// Réponse JSON
let response = json_response(
    StatusCode::OK,
    json!({ "status": "success", "data": data })
);

// Réponse HTML
let response = html_response(
    StatusCode::OK,
    "<h1>Bonjour le monde</h1>"
);

// Redirection
let response = redirect("/dashboard");

// Page 404 de secours (quand le template n'est pas trouvé)
let response = fallback_404_html();
```

### Configuration Settings

Tous les champs settings disponibles :

```rust
let settings = Settings::builder()
    // Serveur
    .server("0.0.0.0", 8000, "clé-secrète")

    // Sécurité
    .debug(false)
    .allowed_hosts(vec!["example.com".to_string()])
    .sanitize_inputs(true)
    .strict_csp(true)
    .rate_limiting(true)
    .enforce_https(true)

    // Chemins
    .templates_dir(vec!["templates".to_string()])
    .staticfiles_dirs("static")
    .media_root("media")
    .static_url("/static")
    .media_url("/media")

    // Chemins internes Runique (généralement pas besoin de changer)
    .static_runique_path("chemin/vers/runique/static")
    .static_runique_url("/runique/static")
    .media_runique_path("chemin/vers/runique/media")
    .media_runique_url("/runique/media")
    .templates_runique("chemin/vers/runique/templates")

    .build();

// Champs additionnels disponibles dans la struct Settings :
// - installed_apps: Vec<String>
// - middleware: Vec<String>
// - root_urlconf: String
// - staticfiles_storage: String
// - language_code: String (défaut : "en-us")
// - time_zone: String (défaut : "UTC")
// - use_i18n: bool
// - use_tz: bool
// - auth_password_validators: Vec<String>
// - password_hashers: Vec<String>
// - default_auto_field: String
// - logging_config: String
```

### Contexte d'erreur

```rust
use runique::error::*;

// Créer depuis une erreur Tera
let ctx = ErrorContext::from_tera_error(&error, "template.html", &tera);

// Créer depuis une erreur anyhow
let ctx = ErrorContext::from_anyhow(&error);

// Créer une erreur 404
let ctx = ErrorContext::not_found("/page-manquante");

// Créer une erreur générique
let ctx = ErrorContext::generic(StatusCode::BAD_REQUEST, "Entrée invalide")
    .with_request(&request)
    .with_details("JSON attendu, XML reçu");

// Champs disponibles dans ErrorContext :
// - status_code: u16
// - error_type: ErrorType (Template, NotFound, Internal, Database, Validation)
// - timestamp: String (ISO 8601)
// - title: String
// - message: String
// - details: Option<String>
// - template_info: Option<TemplateInfo>
// - request_info: Option<RequestInfo>
// - stack_trace: Vec<StackFrame>
// - environment: EnvironmentInfo
```

### Traits de session

```rust
use tower_sessions::Session;
use runique::middleware::csrf::CsrfSession;
use runique::middleware::flash_message::FlashMessageSession;

// Gestion des tokens CSRF
let token = session.get_csrf_token().await?;

// Messages flash
session.insert_message(FlashMessage::success("Terminé !")).await?;
session.insert_message(FlashMessage::error("Échec !")).await?;
session.insert_message(FlashMessage::info("Note")).await?;
session.insert_message(FlashMessage::warning("Attention")).await?;
```

### Gestion des tokens CSRF

```rust
use runique::utils::*;

// Générer un token sécurisé
let token = generate_token("clé_secrète", "id_session");

// Générer un token spécifique à l'utilisateur
let user_token = generate_user_token("clé_secrète", &user_id.to_string());

// Masquer le token (protection contre attaque BREACH)
let masked = mask_csrf_token(&token);

// Démasquer le token pour validation
let original = unmask_csrf_token(&masked)?;
```

---

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE-MIT](LICENSE-MIT) pour plus de détails.

---

## 🙏 Remerciements

Runique s'appuie sur d'excellentes bibliothèques de l'écosystème Rust :

- [Axum](https://github.com/tokio-rs/axum) - Fondation du framework web
- [Tokio](https://tokio.rs/) - Runtime asynchrone
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM avec excellente expérience développeur
- [Tera](https://keats.github.io/tera/) - Moteur de templates inspiré de Django
- [Tower](https://github.com/tower-rs/tower) - Abstractions middleware et service
- [Argon2](https://github.com/RustCrypto/password-hashes) - Hachage sécurisé de mots de passe
- [Serde](https://serde.rs/) - Framework de sérialisation

Remerciements spéciaux à :
- Le projet Django pour l'inspiration
- La communauté Rust pour les outils incroyables
- Tous les contributeurs qui aident à améliorer Runique

---

## 📧 Contact

- **GitHub** : [seb-alliot/runique](https://github.com/seb-alliot/runique)
- **Issues** : [Signaler des bugs ou demander des fonctionnalités](https://github.com/seb-alliot/runique/issues)
- **Discord** : [Rejoindre notre communauté](https://discord.gg/Y5zW7rbt)
- **Email** : alliotsebastien04@gmail.com
- **Crates.io** : [runique](https://crates.io/crates/runique)
- **Docs.rs** : [Documentation API](https://docs.rs/runique)

---

## ⭐ Soutenir le projet

Si Runique vous aide à construire de meilleures applications web, considérez :

- ⭐ [Mettre une étoile sur GitHub](https://github.com/seb-alliot/runique)
- 🐛 [Signaler des bugs et problèmes](https://github.com/seb-alliot/runique/issues)
- 💡 [Suggérer de nouvelles fonctionnalités](https://github.com/seb-alliot/runique/issues/new)
- 📖 [Améliorer la documentation](https://github.com/seb-alliot/runique/tree/main/informations/documentation_french)
- 🤝 [Contribuer du code](https://github.com/seb-alliot/runique/pulls)
- 💬 [Rejoindre notre communauté Discord](https://discord.gg/Y5zW7rbt)
- 📢 Partager Runique avec d'autres

---

## 🌟 Projets remarquables

Projets construits avec Runique :

- **Bientôt disponible !** Soyez le premier à présenter votre projet

Vous voulez ajouter votre projet ? [Contactez-moi](mailto:alliotsebastien04@gmail.com) ou soumettez une PR !

---

**Construisez des applications web sécurisées et performantes avec Runique !** 🦀

---

**Version actuelle :** 0.1.86
**Licence :** MIT
**Statut :** Développement actif
**Version Rust :** 1.75+

*Fait avec ❤️ et 🦀 par la communauté Runique*