# Configuration Programmatique — Builder

`RuniqueApp::builder(config)` retourne un `RuniqueAppBuilder`. C'est le seul builder — il n'y a pas deux versions séparées.

## Exemple minimal

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .build()
    .await?;

app.run().await?;
```

---

## Méthodes disponibles

### Base de données

```rust
// Option 1 : connexion directe (DatabaseConnection)
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

let app = RuniqueApp::builder(config)
    .with_database(db)       // prend une DatabaseConnection
    .routes(router)
    .build()
    .await?;

// Option 2 : connexion déférée (DatabaseConfig)
let db_config = DatabaseConfig::from_env()?.build();

let app = RuniqueApp::builder(config)
    .with_database_config(db_config)  // connexion lors du .build()
    .routes(router)
    .build()
    .await?;
```

### Routes

```rust
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",
        "/about" => view!{ views::about }, name = "about",
    }
}

let app = RuniqueApp::builder(config)
    .routes(routes())
    .build()
    .await?;
```

### Gestion des erreurs

```rust
let app = RuniqueApp::builder(config)
    .with_error_handler(true)   // Active le handler d'erreurs (défaut : true)
    .routes(router)
    .build()
    .await?;
```

### Middlewares

Toute la configuration des middlewares passe par `.middleware(|m| { ... })` où `m` est un `MiddlewareStaging` :

```rust
let app = RuniqueApp::builder(config)
    .routes(router)
    .middleware(|m| {
        m.with_csp(|c| c.with_header_security(true))          // Active le CSP
         .with_allowed_hosts(|h| h.enabled(true).host("mondomaine.fr"))  // Active la validation des hosts
         .with_cache(true)              // Active le no-cache en dev
         .with_debug_errors(true)       // Active les erreurs détaillées
    })
    .build()
    .await?;
```

La validation des hosts s'active via `.with_allowed_hosts(|h| h.enabled(true).host("..."))` dans le builder — sans cet appel, la validation est désactivée. Aucune variable `.env` ne contrôle ce comportement.

> **`is_debug()`** — helper global disponible via `use runique::prelude::*`. Retourne `true` si `DEBUG=true` dans `.env`. Lu une seule fois au démarrage (`LazyLock`), disponible partout sans paramètre.

### Durée de session

```rust
use tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Par défaut : 24h
    .routes(router)
    .build()
    .await?;
```

Ou via `.middleware()` pour les options avancées :

```rust
.middleware(|m| {
    m.with_session_duration(Duration::hours(2))
     .with_anonymous_session_duration(Duration::minutes(5))
     .with_session_memory_limit(128 * 1024 * 1024, 256 * 1024 * 1024)
})
```

### Logs framework

`RuniqueLog` centralise toute la configuration des logs : le niveau du subscriber tracing global **et** les catégories internes du framework.

Le subscriber est initialisé automatiquement par `build()` — **aucun appel à `init_logging()` n'est nécessaire dans `main.rs`**.

Tout passe par `.with_log(|l| ...)` — la closure reçoit un `RuniqueLog` vide et retourne la configuration finale.

```rust
use tracing::Level;

// Contrôle fin par catégorie
RuniqueApp::builder(config)
    .with_log(|l| l
        .csrf(Level::WARN)
        .session(Level::WARN)
        .db(Level::INFO)
    )
    .routes(router)
    .build()
    .await?;
```

#### `subscriber_level` — niveau du subscriber

Par défaut : `"debug"` si `DEBUG=true` dans `.env`, sinon `"warn"`. La variable `RUST_LOG` a toujours la priorité.

```rust
RuniqueApp::builder(config)
    .with_log(|l| l.subscriber_level("info"))
    .routes(router)
    .build()
    .await?;
```

#### `.dev()` — tout activer en développement

Preset qui active toutes les catégories au niveau `DEBUG`. Sans effet si `DEBUG` n'est pas `true` dans `.env` — peut être utilisé inconditionnellement.

```rust
// Dev uniquement (no-op si DEBUG != true)
RuniqueApp::builder(config)
    .with_log(|l| l.dev())
    .routes(router)
    .build()
    .await?;

// Dev avec surcharge du niveau subscriber
RuniqueApp::builder(config)
    .with_log(|l| l.dev().subscriber_level("info").db(Level::INFO))
    .routes(router)
    .build()
    .await?;
```

#### Catégories disponibles

| Catégorie        | Ce qui est journalisé                                      |
| ---------------- | ---------------------------------------------------------- |
| `csrf`           | Token CSRF détecté dans une URL GET (nettoyage silencieux) |
| `exclusive_login`| Sessions invalidées lors d'une connexion exclusive         |
| `filter_fn`      | Échec d'une `filter_fn` dans la vue liste admin            |
| `roles`          | Erreurs d'accès au registre des rôles admin                |
| `password_init`  | `password_init()` appelé plusieurs fois                    |
| `session`        | Watermarks mémoire, records volumineux, erreurs cleanup    |
| `db`             | Connexion DB en cours / connexion établie                  |

### Fichiers statiques

```rust
let app = RuniqueApp::builder(config)
    .statics()     // Active les fichiers statiques
    // ou
    .no_statics()  // Désactive explicitement
    .build()
    .await?;
```

---

## Valeurs par défaut

| Configuration | Défaut | Notes |
| ------------ | ------ | ----- |
| **Session duration** | 24 heures | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Toujours activé | Non désactivable |
| **Error handler** | ✅ Activé | |
| **CSP** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Cache control** | ✅ Activé | No-cache en debug |
| **Static files** | ❌ Désactivé | Appeler `.statics()` |
| **Hot reload admin** | Selon `DEBUG` | Automatique via `is_debug()` |
| **Logs framework** | ❌ Désactivés | Activer via `.with_log(\|l\| ...)` |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](/docs/fr/configuration/variables) | Toutes les variables `.env` |
| [Accès dans le code](/docs/fr/configuration/code) | `RuniqueConfig`, validation |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
