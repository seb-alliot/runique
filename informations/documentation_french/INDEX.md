# 📚 Documentation Rusti Framework

Bienvenue dans la documentation complète de Rusti, un framework web moderne pour Rust inspiré de Django.

## 🎯 Navigation rapide

| Document | Description | Pour qui ? |
|----------|-------------|-----------|
| **[README](README.md)** | Vue d'ensemble et installation | Tous |
| **[GETTING_STARTED](GETTING_STARTED.md)** | Tutorial pas à pas | Débutants |
| **[TEMPLATES](TEMPLATES.md)** | Système de templates | Développeurs frontend |
| **[DATABASE](DATABASE.md)** | ORM et base de données | Développeurs backend |
| **[CONFIGURATION](CONFIGURATION.md)** | Configuration complète | DevOps / Prod |

---

## 📖 Parcours d'apprentissage

### 🌱 Niveau débutant

1. **[README](README.md)** - Comprendre ce qu'est Rusti
2. **[GETTING_STARTED](GETTING_STARTED.md)** - Créer votre première application
3. **[TEMPLATES](TEMPLATES.md)** - Maîtriser les templates

**Durée estimée :** 2-3 heures

### 🚀 Niveau intermédiaire

1. **[DATABASE](DATABASE.md)** - Utiliser l'ORM Django-like
2. **[CONFIGURATION](CONFIGURATION.md)** - Configurer votre application
3. Exemples dans `examples/demo-app`

**Durée estimée :** 4-6 heures

### ⚡ Niveau avancé

1. Middleware personnalisés
2. Optimisations de performance
3. Déploiement en production
4. Architecture multi-services

**Durée estimée :** Variable

---

## 🎓 Guide par tâche

### "Je veux créer une application web simple"

1. [Installation rapide](README.md#-installation)
2. [Première application](GETTING_STARTED.md#première-application)
3. [Ajouter des templates](TEMPLATES.md)
4. [Servir des fichiers statiques](GETTING_STARTED.md#fichiers-statiques)

### "Je veux ajouter une base de données"

1. [Configuration DB](DATABASE.md#configuration)
2. [Définir des modèles](DATABASE.md#définition-des-modèles)
3. [API Django-like](DATABASE.md#api-django-like)
4. [Migrations](DATABASE.md#migrations)

### "Je veux déployer en production"

1. [Configuration production](CONFIGURATION.md#production)
2. [Build optimisé](CONFIGURATION.md#build-optimisé)
3. [Sécurité](CONFIGURATION.md#sécurité)
4. [Checklist production](CONFIGURATION.md#checklist-de-production)

### "Je veux créer une API REST"

1. [Handlers JSON](GETTING_STARTED.md#routes-et-handlers)
2. [Validation des données](DATABASE.md)
3. [Gestion d'erreurs](CONFIGURATION.md#logging-et-tracing)

---

## 📂 Structure de la documentation

```
documentation/
├── README.md                # Vue d'ensemble du framework
├── INDEX.md                 # Ce fichier - Navigation
├── GETTING_STARTED.md       # Tutorial complet pas à pas
├── TEMPLATES.md             # Système de templates Tera
├── DATABASE.md              # ORM et bases de données
└── CONFIGURATION.md         # Configuration et production
```

---

## 🔑 Concepts clés

### RustiApp - Le cœur du framework

```rust
RustiApp::new(settings).await?
    .routes(routes)              // Ajouter les routes
    .with_database(db)           // Optionnel: DB
    .with_static_files()?        // Optionnel: fichiers statiques
    .with_default_middleware()   // Optionnel: middleware erreur
    .run().await?;               // Lancer le serveur
```

**Voir :** [Getting Started - Structure](GETTING_STARTED.md#structure-du-projet)

### Settings - Configuration flexible

```rust
// Builder pattern
Settings::builder()
    .debug(true)
    .server("127.0.0.1", 3000, "secret")
    .templates_dir(vec!["templates".to_string()])
    .build()
```

**Voir :** [Configuration - Settings](CONFIGURATION.md#settings)

### urlpatterns! - Routing Django-like

```rust
urlpatterns! {
    "/" => get(index), name = "home",
    "/user/{id}" => get(user_detail), name = "user_profile",
}
```

**Voir :** [Getting Started - Routes](GETTING_STARTED.md#routes-et-handlers)

### ORM Django-like

```rust
Entity::objects
    .filter(Column::Age.gte(18))
    .exclude(Column::IsBanned.eq(true))
    .order_by_desc(Column::CreatedAt)
    .limit(10)
    .all(&db)
    .await?
```

**Voir :** [Database - API Django-like](DATABASE.md#api-django-like)

---

## 🎨 Features principales

| Feature | Documentation | Exemple |
|---------|---------------|---------|
| **Templates Tera** | [TEMPLATES.md](TEMPLATES.md) | `{% static "file.css" %}` |
| **Balises personnalisées** | [TEMPLATES.md](TEMPLATES.md#balises-disponibles) | `{% csrf %}`, `{% messages %}` |
| **Reverse routing** | [TEMPLATES.md](TEMPLATES.md#-link-route_name-params) | `{% link "home" %}` |
| **Flash messages** | [GETTING_STARTED.md](GETTING_STARTED.md#routes-et-handlers) | `success!(message,"message");` |
| **Protection CSRF** | [CONFIGURATION.md](CONFIGURATION.md#middleware) | `.with_csrf_tokens()` |
| **ORM SeaORM** | [DATABASE.md](DATABASE.md) | `Entity::objects.all()` |
| **Migrations** | [DATABASE.md](DATABASE.md#migrations) | `sea-orm-cli migrate up` |
| **Sessions** | [CONFIGURATION.md](CONFIGURATION.md) | Automatique |
| **Pages debug** | [CONFIGURATION.md](CONFIGURATION.md#production) | Mode `debug = true` |

---

## 🛠️ Références rapides

### Commandes courantes

```bash
# Créer un projet
cargo new mon-app && cd mon-app
cargo add rusti tokio --features full

# Lancer en dev
cargo run

# Build de production
cargo build --release

# Tests
cargo test

# Documentation
cargo doc --open

# Migrations
sea-orm-cli migrate up
sea-orm-cli migrate down
```

### Fichiers importants

```
mon-projet/
├── src/
│   ├── main.rs          # Point d'entrée
│   ├── urls.rs          # Routes
│   └── views.rs         # Handlers
├── templates/           # Templates Tera
├── static/              # CSS, JS, images
├── media/               # Uploads
├── .env                 # Configuration
└── Cargo.toml
```

### Imports courants

```rust
use rusti::prelude::*;  // Import principal

// Ou spécifiques
use rusti::{
    RustiApp,
    Settings,
    Router,
    Context,
    Template,
    Message,
    Response,
    StatusCode,
    Extension,
    Path,
    Json,
};
```

---

## 🐛 Résolution de problèmes

### "Template not found"

**Solution :** Vérifiez `templates_dir` dans Settings
```rust
.templates_dir(vec!["templates".to_string()])
```

**Voir :** [Templates - Configuration](TEMPLATES.md#configuration)

### "CSRF token verification failed"

**Solution :** Activez le middleware CSRF
```rust
.with_csrf_tokens()
```

**Voir :** [Configuration - Middleware](CONFIGURATION.md#middleware)

### "Database connection failed"

**Solution :** Vérifiez votre `.env` et la feature Cargo
```toml
rusti = { version = "0.1", features = ["postgres"] }
```

**Voir :** [Database - Configuration](DATABASE.md#configuration)

### "Route not found avec {% link %}"

**Solution :** Ajoutez `name = "..."` à votre route
```rust
urlpatterns! {
    "/" => get(index), name = "home",  // ✅
}
```

**Voir :** [Templates - Link](TEMPLATES.md#-link-route_name-params)

---

## 💡 Exemples pratiques

### Exemple 1: Application minimale

```rust
use rusti::prelude::*;

async fn hello() -> &'static str {
    "Hello, Rusti!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RustiApp::new(Settings::default_values()).await?
        .routes(Router::new().route("/", get(hello)))
        .run()
        .await?;
    Ok(())
}
```

**Voir :** [Getting Started - Première app](GETTING_STARTED.md#première-application)

### Exemple 2: Avec templates et DB

**Voir :** [Getting Started - Exemple complet](GETTING_STARTED.md#exemple-complet)

### Exemple 3: API REST

**Voir :** [Getting Started - API JSON](GETTING_STARTED.md#routes-et-handlers)

---

## 📚 Ressources externes

### Documentation officielle

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [Tera Documentation](https://keats.github.io/tera/)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Tokio Documentation](https://tokio.rs/)

### Inspirations

- [Django](https://www.djangoproject.com/)
- [Actix-Web](https://actix.rs/)
- [Rocket](https://rocket.rs/)

---

## 🤝 Contribution

Vous souhaitez contribuer à Rusti ? Excellent !

1. Fork le projet
2. Créez une branche (`git checkout -b feature/AmazingFeature`)
3. Committez vos changements (`git commit -m 'Add AmazingFeature'`)
4. Push vers la branche (`git push origin feature/AmazingFeature`)
5. Ouvrez une Pull Request

---

## 📄 Licence

Ce projet est sous double licence MIT / Apache-2.0.

**Voir :** [LICENSE-MIT](../LICENSE-MIT)

---

## 📞 Support et communauté

- 📖 [Documentation complète](https://docs.rs/rusti)
- 💬 [GitHub Discussions](https://github.com/votre-repo/rusti/discussions)
- 🐛 [Reporter un bug](https://github.com/votre-repo/rusti/issues)
- ⭐ [Donner une étoile](https://github.com/votre-repo/rusti)

---

**Développé avec ❤️ en Rust par Itsuki**

**Bon développement avec Rusti ! 🦀**
