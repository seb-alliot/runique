# 📚 Rusti Framework - Index de documentation

Bienvenue dans la documentation du framework Rusti !

##  Structure du projet

```
rusti-framework/
├── rusti/                    #  Le framework (bibliothèque)
│   ├── src/
│   │   ├── lib.rs           # Point d'entrée, exports publics
│   │   ├── app.rs           # RustiApp - structure principale
│   │   ├── settings.rs      # Configuration (Settings, builder)
│   │   ├── db.rs            # Connexion base de données (feature orm)
│   │   ├── error.rs         # Structures d'erreur (ErrorContext, etc.)
│   │   ├── response.rs      # Helpers de réponse (JSON, HTML, redirect)
│   │   ├── middleware.rs    # Module middleware
│   │   └── middleware/
│   │       ├── error_handler.rs  # Middleware de gestion d'erreur
│   │       └── tera_ext.rs      # Extension trait TeraSafe pour Tera
│   └── Cargo.toml
│
├── examples/
│   └── demo-app/            # 🎯 Application exemple complète
│       ├── src/
│       │   ├── main.rs      # Point d'entrée de l'app
│       │   └── views.rs     # Handlers/views
│       ├── templates/       # Templates Tera
│       │   ├── index.html
│       │   ├── about.html
│       │   └── errors/
│       │       ├── 404.html
│       │       └── 500.html
│       ├── static/          # Fichiers statiques (CSS, JS)
│       │   └── css/
│       │       └── main.css
│       ├── media/           # Fichiers média (uploads, images)
│       ├── .env.example     # Exemple de configuration
│       └── Cargo.toml
│
├── Cargo.toml              # Workspace root
├── .gitignore
├── LICENSE-MIT
│
└── Documentation/
    ├── README.md           # 📖 Documentation principale
    ├── QUICKSTART.md       # 🚀 Guide de démarrage rapide
    ├── MIGRATION.md        # 📦 Guide de migration depuis ton code
    ├── TEMPLATES.md        # 🎨 Templates de projets
    └── INDEX.md            # 📚 Ce fichier
```

## 📖 Guides de lecture

### Pour démarrer rapidement
1. **README.md** - Vue d'ensemble, features, installation
2. **QUICKSTART.md** - Premier projet en 5 minutes
3. **examples/demo-app** - Application exemple fonctionnelle

### Pour migrer ton code existant
1. **MIGRATION.md** - Guide étape par étape
2. Comparer ton code avec `examples/demo-app`
3. Utiliser les templates dans **TEMPLATES.md**

### Pour approfondir
1. Lire les commentaires dans `rusti/src/lib.rs`
2. Explorer chaque module dans `rusti/src/`
3. Consulter **TEMPLATES.md** pour différents cas d'usage

## 🎯 Modules du framework

### Core (`rusti/src/`)

| Fichier | Description | Responsabilité |
|---------|-------------|----------------|
| `lib.rs` | Point d'entrée | Exports publics, macro `routes!`, version |
| `app.rs` | RustiApp | Structure principale, builder pattern, lancement serveur |
| `settings.rs` | Configuration | Settings, ServerSettings, DatabaseSettings, builder |
| `db.rs` | Base de données | Connexion SeaORM, validation, masquage mot de passe |
| `error.rs` | Gestion erreurs | ErrorContext, ErrorType, structures de debug |
| `response.rs` | Helpers réponse | json_response, html_response, redirect, 404 |
| `middleware.rs` | Middleware | Module principal, re-exports |

### Middleware (`rusti/src/middleware/`)

| Fichier | Description | Exports |
|---------|-------------|---------|
| `error_handler.rs` | Gestion erreurs HTTP | error_handler_middleware, render_safe |
| `tera_ext.rs` | Extension Tera | Trait TeraSafe avec render_safe() |

## 🔑 Concepts clés

### 1. RustiApp - Le cœur du framework

```rust
RustiApp::new(settings).await?
    .routes(routes)              // Ajouter les routes
    .with_database().await?      // Optionnel: DB
    .with_static_files()?        // Optionnel: fichiers statiques
    .with_sessions()             // Optionnel: sessions
    .with_default_middleware()   // Optionnel: middleware erreur/timeout
    .run().await?;               // Lancer le serveur
```

### 2. Settings - Configuration flexible

```rust
// Trois façons de configurer
Settings::default_values()           // Défauts
Settings::from_env()                 // Depuis .env
Settings::builder()                  // Builder pattern
    .debug(true)
    .server("0.0.0.0", 8080)
    .build()
```

### 3. TeraSafe - Extension trait pour Tera

```rust
use rusti::middleware::TeraSafe;

// Au lieu de gérer manuellement les erreurs
tera.render_safe(template, context, status, config)
// Remplace:
// return_render(&tera, template, context, status, config)
```

### 4. Middleware d'erreur intégré

- Mode debug: Pages détaillées avec stack trace
- Mode production: Pages simples et élégantes
- Personnalisable via templates dans `templates/errors/`

## 🎨 Patterns d'utilisation

### Pattern 1: Application minimale
```rust
// 10 lignes pour un serveur web !
let settings = Settings::default_values();
RustiApp::new(settings).await?
    .routes(Router::new().route("/", get(handler)))
    .run().await?;
```

### Pattern 2: Application avec templates
```rust
// Structure classique avec views
let routes = Router::new()
    .route("/", get(views::index));

RustiApp::new(settings).await?
    .routes(routes)
    .with_static_files()?
    .with_default_middleware()
    .run().await?;
```

### Pattern 3: API REST
```rust
// JSON API
let routes = Router::new()
    .route("/api/users", get(api::list))
    .route("/api/users", post(api::create));

RustiApp::new(settings).await?
    .routes(routes)
    .run().await?;
```

### Pattern 4: Fullstack (Web + API + DB)
```rust
// Application complète
RustiApp::new(settings).await?
    .with_database().await?
    .routes(routes)
    .with_static_files()?
    .with_sessions()
    .with_default_middleware()
    .run().await?;
```

## 🔧 Features disponibles

| Feature | Description | Par défaut |
|---------|-------------|------------|
| `orm` | Support SeaORM pour la base de données | ✅ Oui |

```toml
# Avec ORM (défaut)
rusti = "0.1"

# Sans ORM
rusti = { version = "0.1", default-features = false }
```

## 🚀 Commandes rapides

```bash
# Créer un nouveau projet
cargo new mon-app && cd mon-app
cargo add rusti tokio --features full

# Lancer l'exemple
cd examples/demo-app
cargo run

# Tests
cargo test

# Build release
cargo build --release

# Watch mode (avec cargo-watch)
cargo watch -x run
```

## 📊 Comparaison avec ton code d'origine

| Aspect | Avant | Après (Rusti) |
|--------|-------|---------------|
| Fichiers | ~15 fichiers Rust | 3 fichiers (main, views, models) |
| Configuration | Code manuel | Builder pattern ou .env |
| Gestion erreur | Middleware custom | Intégré avec debug pages |
| Routing | Construction manuelle | Déclaratif avec macro `routes!` |
| Templates | Helper function | Trait extension `.render_safe()` |
| Base de données | Configuration manuelle | `.with_database()` |
| Dépendances | ~15 crates | 1 crate (+ tokio, serde) |

## 🎓 Exemples d'utilisation

### Exemple 1: Hello World
```rust
use rusti::{RustiApp, Settings, Router, routing::get};

async fn hello() -> &'static str { "Hello!" }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RustiApp::new(Settings::default_values()).await?
        .routes(Router::new().route("/", get(hello)))
        .run().await?;
    Ok(())
}
```

### Exemple 2: Avec template
Voir `examples/demo-app/src/main.rs` et `views.rs`

### Exemple 3: API JSON
Voir section "Template API REST" dans `TEMPLATES.md`

### Exemple 4: Avec base de données
Voir section "Template avec base de données" dans `TEMPLATES.md`

## 🐛 Débogage

### Mode debug activé
- Pages d'erreur détaillées automatiques
- Stack trace complète
- Informations de requête
- Templates disponibles listés

### Logs
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

## 🤝 Contribution

Le framework est structuré pour être facilement extensible :

1. **Ajouter un middleware** → `rusti/src/middleware/`
2. **Ajouter des helpers** → `rusti/src/response.rs`
3. **Étendre Settings** → `rusti/src/settings.rs`
4. **Nouveaux templates d'erreur** → `templates/errors/`

## 📝 TODO / Roadmap

- [ ] Tests unitaires et d'intégration
- [ ] Middleware d'authentification
- [ ] Support de migrations DB (SeaORM migrations)
- [ ] CLI pour scaffolding de projets
- [ ] Documentation API complète (docs.rs)
- [ ] Exemples additionnels (WebSocket, GraphQL)
- [ ] Benchmarks de performance

## 🔗 Ressources

### Documentation Rust
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Docs](https://docs.rs/axum/)
- [Tera Docs](https://keats.github.io/tera/)
- [SeaORM Docs](https://www.sea-ql.org/SeaORM/)

### Inspiration
- [Django](https://www.djangoproject.com/)
- [Actix-Web](https://actix.rs/)
- [Rocket](https://rocket.rs/)

## 💡 Prochaines étapes

1. ✅ Lire **README.md** pour une vue d'ensemble
2. ✅ Suivre **QUICKSTART.md** pour ton premier projet
3. ✅ Lancer `examples/demo-app` pour voir le framework en action
4. ✅ Consulter **MIGRATION.md** pour migrer ton code existant
5. ✅ Utiliser **TEMPLATES.md** comme référence rapide

---

**Bon dev avec Rusti ! 🦀**
