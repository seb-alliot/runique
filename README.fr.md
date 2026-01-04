# Runique

**Un framework web Rust inspiré de Django**

Runique est un framework web moderne qui combine la sécurité et les performances de Rust avec l'ergonomie de Django. Il offre une expérience de développement familière pour les développeurs Django tout en tirant parti de la puissance du système de types de Rust.

[![Version](https://img.shields.io/badge/version-1.0.3-blue.svg)](https://crates.io/crates/runique)
[![docs.rs](https://img.shields.io/docsrs/runique)](https://docs.rs/runique)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

---

## 🤔 Pourquoi Runique ?

- **Pour les développeurs Django** : API et patterns familiers avec les performances et la sécurité de Rust
- **Pour les développeurs Rust** : Ergonomie inspirée de Django sans sacrifier la sécurité des types
- **Pour tout le monde** : Sécurité intégrée dès le départ, pas ajoutée après coup

---

## 🚀 Fonctionnalités Principales

### Architecture Similaire à Django
- **Routing déclaratif** avec la macro `urlpatterns!`
- **ORM intuitif** basé sur SeaORM avec une API dans le style Django
- **Système de templates** Tera avec préprocessing personnalisé
- **Génération automatique de formulaires** via macros procédurales
- **Messages flash** entre les requêtes
- **Gestion des fichiers statiques et médias**

### Sécurité Intégrée
- ✅ **Protection CSRF** (HMAC-SHA256)
- ✅ **Content Security Policy** (CSP) avec nonces
- ✅ **Sanitisation XSS** (ammonia)
- ✅ **Headers de Sécurité Automatiques** (HSTS, X-Frame-Options, etc.)
- ✅ **Validation ALLOWED_HOSTS**
- ✅ **Hachage Argon2id Intégré**

### Support Multi-bases de Données
- PostgreSQL
- MySQL / MariaDB
- SQLite

### Développement Moderne
- **Async/await natif** avec Tokio
- **Type-safe** grâce au système de types de Rust
- **Abstractions sans coût**
- **Hot reload** en développement
- **Documentation complète**

---

## 📦 Installation

### Prérequis

- Rust 1.75+ ([installer Rust](https://www.rust-lang.org/tools/install))
- Cargo

### Ajouter Runique à Votre Projet

```toml
# Cargo.toml

# Configuration minimale (SQLite par défaut)
[dependencies]
runique = "1.0.3"

# Avec PostgreSQL
[dependencies]
runique = { version = "1.0.3", features = ["postgres"] }

# Avec MySQL
[dependencies]
runique = { version = "1.0.3", features = ["mysql"] }

# Avec MariaDB
[dependencies]
runique = { version = "1.0.3", features = ["mariadb"] }

# Avec toutes les bases de données
[dependencies]
runique = { version = "1.0.3", features = ["all-databases"] }
```

### Features Cargo Disponibles

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
runique = "1.0.3"

# PostgreSQL + MySQL
[dependencies]
runique = { version = "1.0.3", features = ["postgres", "mysql"] }

# Toutes les bases de données
[dependencies]
runique = { version = "1.0.3", features = ["all-databases"] }

# Sans ORM (framework minimal)
[dependencies]
runique = { version = "1.0.3", default-features = false }
```

### Créer un Nouveau Projet

```bash
cargo install runique
runique new mon_app
cd mon_app
```

Ajouter Runique au `Cargo.toml` :

```toml
[dependencies]
runique = { version = "1.0.3", features = ["sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

---

## 🏁 Démarrage Rapide

### Application Minimale

```rust
// src/main.rs
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::from_env();

    RuniqueApp::new(settings).await?
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
        name = "index",

        "/hello/:name" => view!{
            GET => views::hello
        },
        name = "hello",
    ]
}

async fn index() -> &'static str {
    "Bienvenue sur Runique ! 🚀"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Bonjour, {} !", name)
}
```

### Configuration (.env)

```env
HOST=127.0.0.1
PORT=8000
SECRET_KEY=votre-cle-secrete-ici
ALLOWED_HOSTS=localhost,127.0.0.1
DEBUG=true

# PostgreSQL (optionnel)
DB_ENGINE=postgres
DB_USER=utilisateur
DB_PASSWORD=motdepasse
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mabase
```

### Lancement

```bash
cargo run
```

Ouvrir [http://localhost:8000](http://localhost:8000)

**Pour des exemples plus avancés, voir la section [Exemple Complet](#-exemple-complet) ci-dessous.**

---

## 📚 Documentation

- [🚀 Démarrage](informations/documentation_french/GETTING_STARTED.md)
- [⚙️ Configuration](informations/documentation_french/CONFIGURATION.md)
- [🗄️ Base de Données](informations/documentation_french/DATABASE.md)
- [📝 Formulaires](informations/documentation_french/FORMULAIRE.md)
- [🎨 Templates](informations/documentation_french/TEMPLATES.md)
- [🔒 Sécurité](informations/documentation_french/CSP.md)
- [🛣️ Macros](informations/documentation_french/MACRO_CONTEXT.md)
- [🔧 Changelog](informations/documentation_french/CHANGELOG.md)
- [🚀 Contribuer](informations/documentation_french/CONTRIBUTING.md)
- [🆕 Nouveau projet](informations/documentation_french/NOUVEAU_PROJET.md)
- [📖 Documentation API](https://docs.rs/runique)

---

## 🎯 Exemple Complet

### Structure du Projet
### Vous pouvez utiliser : `cargo install runique` → `runique new nom_projet`

```
mon_app/
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

### Handler Avancé avec Validation de Formulaire

```rust
use runique::prelude::*;

// Handler de formulaire avec validation
pub async fn profil_utilisateur(
    template: Template,
    ExtractForm(form): ExtractForm<ModelForm>,
) -> Response {
    let ctx = context! {
        "title", "Profil Utilisateur";
        "form", form
    };
    template.render("profile/register_profile.html", &ctx)
}

// Soumission de formulaire avec gestion d'erreurs
pub async fn profil_utilisateur_submit(
    Extension(db): Extension<Arc<DatabaseConnection>>,
    mut message: Message,
    template: Template,
    ExtractForm(user): ExtractForm<ModelForm>,
) -> Response {
    if user.is_valid() {
        match user.save(&db).await {
            Ok(created_user) => {
                success!(message, "Profil utilisateur créé avec succès !");
                let target = reverse_with_parameters(
                    "user_profile",
                    &[
                        ("id", &created_user.id.to_string()),
                        ("name", &created_user.username),
                    ],
                )
                .unwrap();
                return Redirect::to(&target).into_response();
            }
            Err(err) => {
                // Gestion des erreurs de contrainte unique
                let error_msg = if err.to_string().contains("unique") {
                    if err.to_string().contains("username") {
                        "Ce nom d'utilisateur est déjà pris !"
                    } else if err.to_string().contains("email") {
                        "Cet email est déjà utilisé !"
                    } else {
                        "Cette valeur existe déjà dans la base de données"
                    }
                } else {
                    "Erreur lors de la sauvegarde"
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

    // Scénarios d'erreur de validation
    error!(message, "Erreur de validation du formulaire");

    let ctx = context! {
        "form", ModelForm::build();
        "forms_errors", user.get_errors();
        "title", "Erreur de Validation"
    };
    template.render("name.html", &ctx)
}
```

---

## 🔒 Sécurité

### Protection CSRF

La protection CSRF est automatiquement activée lors de l'utilisation de `.with_default_middleware()`.

```rust
use runique::prelude::*;

RuniqueApp::new(settings).await?
    .with_default_middleware()  // Inclut la protection CSRF
    .routes(routes())
    .run()
    .await?;
```

Dans vos templates :

```html
<form method="post">
    {% csrf %}
    <!-- champs du formulaire -->
</form>
```

### Content Security Policy

```rust
use runique::prelude::*;

RuniqueApp::new(settings).await?
    .with_security_headers(CspConfig::strict())
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

### Headers de Sécurité

```rust
RuniqueApp::new(settings).await?
    .with_static_files()?
    .with_allowed_hosts(
        env::var("ALLOWED_HOSTS")
        .ok()
        .map(|s| s.split(',').map(|h| h.to_string()).collect()),
    )
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

Headers automatiquement configurés :
- `Strict-Transport-Security`
- `X-Content-Type-Options`
- `X-Frame-Options`
- `X-XSS-Protection`
- `Referrer-Policy`
- `Permissions-Policy`

---

## 🗄️ Base de Données

### Configuration

```rust
RuniqueApp::new(settings).await?
    .with_database(db)
    .with_static_files()?
    .with_allowed_hosts(
        env::var("ALLOWED_HOSTS")
        .ok()
        .map(|s| s.split(',').map(|h| h.to_string()).collect()),
    )
    .with_sanitize_text_inputs(false)
    .with_default_middleware()
    .routes(routes())
    .run()
    .await?;
```

### API dans le Style Django

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

Utiliser `sea-orm-cli` pour les migrations :

```bash
cargo install sea-orm-cli

# Créer une migration
sea-orm-cli migrate generate create_users_table

# Appliquer
sea-orm-cli migrate up

# Annuler
sea-orm-cli migrate down
```

---

## 🎨 Templates

### Tags Personnalisés

```html
<!-- Fichiers statiques -->
<link rel="stylesheet" href="{% static 'css/style.css' %}">
<script src='{% static "js/main.js" %}'></script>

<!-- Fichiers médias -->
<img src='{% media "media.jpg" %}' alt="Avatar">

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

## 📦 Macros Utilitaires

Runique fournit des macros pour simplifier les opérations courantes.

### Messages Flash

```rust
use runique::prelude::*;

async fn mon_handler(mut message: Message) -> Response {
    // Note : Il faut utiliser `mut` pour message, sinon ça ne fonctionnera pas
    // Messages simples
    success!(message, "Opération réussie !");
    error!(message, "Une erreur s'est produite");
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
- Support des messages multiples
- Code plus lisible et maintenable

**Macros disponibles :**
- `success!(message, "texte")` - Messages de succès
- `error!(message, "texte")` - Messages d'erreur
- `info!(message, "texte")` - Messages d'information
- `warning!(message, "texte")` - Messages d'avertissement

---

## 🚀 Performance

Runique exploite les performances de Rust et Tokio :

- **Abstractions sans coût** : Aucune surcharge à l'exécution
- **Async/await natif** : Concurrence efficace avec Tokio
- **Pool de connexions** : Gestion optimisée des connexions DB
- **Compilation optimisée** : Binaire hautement optimisé

### Benchmark (indicatif)

```
Configuration : Machine de développement locale
Requêtes/sec : ~50 000
Latence p50 : ~1ms
Latence p99 : ~5ms
Mémoire : ~20MB
```

*Note : Les performances réelles dépendent de votre matériel et de la complexité de votre application. Effectuez vos propres benchmarks pour des estimations en production.*

---

## 🛠️ Développement

### Tests

```bash
# Lancer tous les tests
cargo test

# Lancer les tests d'intégration
cargo test --test integration

# Lancer les doc tests
cargo test --doc
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
# Générer et ouvrir la documentation
cargo doc --open

# Tester les exemples de documentation
cargo test --doc
```

---

## 🤝 Contribuer

Les contributions sont les bienvenues ! Voici comment contribuer :

1. Fork le projet
2. Créer une branche (`git checkout -b feature/fonctionnalite-incroyable`)
3. Committer vos changements (`git commit -m 'Ajout d'une fonctionnalité incroyable'`)
4. Pusher vers la branche (`git push origin feature/fonctionnalite-incroyable`)
5. Ouvrir une Pull Request

### Directives

- Écrire des tests pour les nouvelles fonctionnalités
- Suivre les conventions de code Rust (rustfmt)
- Documenter les APIs publiques
- Ajouter des exemples si pertinent

Voir [CONTRIBUTING.md](informations/documentation_french/CONTRIBUTING.md) pour plus de détails.

---

## 📝 Roadmap

### Version 1.1 (T1 2026)

- [ ] Système d'authentification intégré
- [ ] Panel d'administration auto-généré
- [ ] Middleware de limitation de débit
- [ ] Support WebSocket
- [ ] Couche de cache (Redis)

### Version 1.2 (T2 2026)

- [x] CLI pour le scaffolding
- [ ] Hot reload amélioré
- [ ] Support GraphQL
- [ ] Tâches en arrière-plan (Tokio tasks)

### Version 2.0 (T3 2026)

- [ ] Système de plugins
- [ ] Multi-tenancy
- [ ] Internationalisation (i18n)
- [ ] Fonctionnalités ORM avancées

---

## 📄 Licence

Ce projet est sous licence MIT. Voir le fichier [LICENSE-MIT](LICENSE-MIT) pour plus de détails.

---

## 🙏 Remerciements

Runique s'appuie sur d'excellentes bibliothèques de l'écosystème Rust :

- [Axum](https://github.com/tokio-rs/axum) - Framework web
- [Tokio](https://tokio.rs/) - Runtime asynchrone
- [SeaORM](https://www.sea-ql.org/SeaORM/) - ORM
- [Tera](https://keats.github.io/tera/) - Moteur de templates
- [Tower](https://github.com/tower-rs/tower) - Middleware
- [Argon2](https://github.com/RustCrypto/password-hashes) - Hachage de mots de passe
- [ammonia](https://github.com/rust-ammonia/ammonia) - Sanitisation HTML

Merci spécial à tous les contributeurs et à la communauté Rust !

---

## 📧 Contact

- **GitHub Issues** : [Signaler des bugs ou demander des fonctionnalités](https://github.com/seb-alliot/runique/tree/issues)
- **Discord** : [Rejoindre notre communauté](https://discord.gg/Y5zW7rbt)
- **Email** : alliotsebastien04@gmail.com
- **Crates.io** : [Voir sur crates.io](https://crates.io/crates/runique)
- **Docs.rs** : [Lire la documentation de l'API](https://docs.rs/runique)

---

## ⭐ Soutenir le Projet

Si Runique vous est utile, pensez à :

- ⭐ [Mettre une étoile sur GitHub](https://github.com/seb-alliot/runique)
- 🐛 Signaler des bugs
- 💡 Suggérer des fonctionnalités
- 📖 Améliorer la documentation
- 🤝 Contribuer du code
- 💬 Rejoindre notre communauté Discord

---

**Construisez des applications web sécurisées et performantes avec Runique !** 🚀

---

**Version :** 1.0.3
**Licence :** MIT
**Statut :** Stable

*Fait avec ❤️ et 🦀 par la communauté Runique*