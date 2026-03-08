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
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view!{ GET => views::about }, name = "about",
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
        m.with_csp(true)                // Active le Content Security Policy
         .with_host_validation(true)    // Active la validation des hosts
         .with_cache(true)              // Active le no-cache en dev
         .with_debug_errors(true)       // Active les erreurs détaillées
    })
    .build()
    .await?;
```

En mode `DEBUG=true`, `with_csp` et `with_host_validation` sont **désactivés par défaut**. En production, tout est activé. Les variables `RUNIQUE_ENABLE_*` du `.env` sont prioritaires sur les défauts.

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
|--------------|--------|-------|
| **Session duration** | 24 heures | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Toujours activé | Non désactivable |
| **Error handler** | ✅ Activé | |
| **CSP** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Cache control** | ✅ Activé | No-cache en debug |
| **Static files** | ❌ Désactivé | Appeler `.statics()` |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/variables/variables.md) | Toutes les variables `.env` |
| [Accès dans le code](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/code/code.md) | `RuniqueConfig`, validation |

## Retour au sommaire

- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)
