# ⚙️ Configuration

## RuniqueConfig

Toute la configuration est facilitée via `.env` et est chargée dans une struct `RuniqueConfig`.

### Charger la configuration

```rust
use runique::config_runique::RuniqueConfig;

let config = RuniqueConfig::from_env()?;

// Accès aux variables:
println!("Debug: {}", config.debug);
println!("Port: {}", config.port);
println!("DB: {}", config.database_url);
```

---

## Variables d'Environnement

### Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | 127.0.0.1 | Adresse IP écoute |
| `PORT` | 3000 | Port serveur |
| `DEBUG` | true | Mode debug (templates, logs, etc.) |

**Exemple:**
```env
# Server Configuration
IP_SERVER=127.0.0.1
PORT=3000

DEBUG=true
# Database Configuration (SQLite par défaut)

# Secret key for csrf management
SECRETE_KEY=your_secret_key_here

# A completer pour toute bdd autre que SQLite
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# En option pas obligatoire hormis usage personnel
DATABASE_URL=postgresql://monuser:monmotdepasse@localhost:5432/mabase

# Allowed hosts for production
ALLOWED_HOSTS=exemple.com,www.exemple.com,.api.exemple.com,localhost,127.0.0.1
```

### Base de Données

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | - | Chaîne de connexion complète |
| `DB_ENGINE` | postgres | postgres, sqlite, mysql |
| `DB_USER` | postgres | Utilisateur DB |
| `DB_PASSWORD` | - | Mot de passe DB |
| `DB_HOST` | localhost | Host DB |
| `DB_PORT` | 5432 | Port DB |
| `DB_NAME` | runique | Nom base de données |

**PostgreSQL:**
```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev):**
```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Templates & Assets

| Variable | Défaut | Description |
|----------|--------|-------------|
| `TEMPLATES_DIR` | templates | Répertoire templates |
| `STATICFILES_DIRS` | static | Répertoire assets statiques |
| `MEDIA_ROOT` | media | Répertoire médias (uploads) |

**Exemple:**
```env
TEMPLATES_DIR=templates
STATICFILES_DIRS=static:demo-app/static
MEDIA_ROOT=uploads
```

### Sécurité

| Variable | Défaut | Description |
|----------|--------|-------------|
| `SECRETE_KEY` | - | Clé secrète CSRF (⚠️ CHANGE EN PROD!) |
| `ALLOWED_HOSTS` | * | Hosts autorisés (comma-separated) |

**Exemple:**
```env
SECRETE_KEY=your_secret_key_change_this_in_production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

**ALLOWED_HOSTS patterns:**
- `localhost` - Exact match
- `*` - Wildcard tous les hosts (DANGER en production!)
- `.example.com` - Match example.com et *.example.com

---

## Fichier .env Complet

```env
# ============================================================================
# SERVER CONFIGURATION
# ============================================================================
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# ============================================================================
# DATABASE CONFIGURATION
# ============================================================================
# PostgreSQL (Recommended for production)
DATABASE_URL=postgres://postgres:password@localhost:5432/runique
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# SQLite (Development only)
# DATABASE_URL=sqlite:runique.db?mode=rwc

# ============================================================================
# TEMPLATES & STATIC FILES
# ============================================================================
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# ============================================================================
# SECURITY
# ============================================================================
# IMPORTANT: Générer une nouvelle clé pour la production!
# python3 -c "import secrets; print(secrets.token_urlsafe(32))"
SECRETE_KEY=your_secret_key_here_change_in_production

# Format: comma-separated (no spaces)
# .example.com matches example.com and *.example.com
ALLOWED_HOSTS=localhost,127.0.0.1
```

---

## Configuration Avancée

### Mode Production

```env
DEBUG=false
PORT=443
IP_SERVER=0.0.0.0

# HTTPS
SECRETE_KEY=<généré dynamiquement>

# Hosts stricts
ALLOWED_HOSTS=example.com,www.example.com,.api.example.com

# DB externalisée
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Mode Développement

```env
DEBUG=true
PORT=3000
IP_SERVER=127.0.0.1

SECRETE_KEY=any_dev_key
ALLOWED_HOSTS=*

DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Mode Testing

```env
DEBUG=true
SECRETE_KEY=test_key
ALLOWED_HOSTS=localhost,127.0.0.1

# Base de données en mémoire (SQLite)
DATABASE_URL=sqlite::memory:
```

---

## Générer une clé secrète

```bash
# Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Rust
cargo run --example generate_secret

# OpenSSL
openssl rand -base64 32
```

---

## Accéder à la configuration dans le code

```rust
use runique::config_runique::RuniqueConfig;

async fn my_handler(template: TemplateContext) -> Response {
    let config = &template.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Allowed hosts: {:?}", config.security.allowed_hosts);
    println!("Secret key: {}", config.security.secrete_key);
}
```

### Configuration conditionnelle

```rust
if template.engine.config.debug {
    // Mode debug: logs détaillés, templates rechargés
} else {
    // Mode production: cache templates, pas de logs sensibles
}

if template.engine.config.security.allowed_hosts.contains("*") {
    // ⚠️ Attention: tous les hosts sont autorisés (danger en production!)
}
```

---

## Validation de configuration

La configuration est validée au startup:

```rust
let config = RuniqueConfig::from_env()
    .expect("Configuration invalide");

// Retourne Err() si:
// - DATABASE_URL manquant
// - SECRETE_KEY manquant
// - Variables invalides
```

---

## Configuration Programmatique (Hors .env)

En plus de la configuration via le fichier `.env`, le builder `RuniqueApp` offre des méthodes pour personnaliser directement votre application sans toucher aux variables d'environnement.

### Méthodes du Builder

#### 📦 Base de données

```rust
use sea_orm::Database;

let db = Database::connect("postgresql://localhost/mydb").await?;

let app = RuniqueApp::builder(config)
    .with_database(db)
    .routes(router)
    .build()
    .await?;
```

#### 🔄 Routes

```rust
let router = Router::new()
    .route("/", get(home))
    .route("/about", get(about));

let app = RuniqueApp::builder(config)
    .routes(router)  // Définir les routes
    .build()
    .await?;
```

#### ⏱️ Durée de session

```rust
use tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Par défaut: 24h
    .routes(router)
    .build()
    .await?;
```

**Exemples de durées:**
```rust
Duration::hours(2)      // 2 heures
Duration::days(7)       // 7 jours
Duration::minutes(30)   // 30 minutes
Duration::seconds(3600) // 1 heure
```

#### 💾 Session store personnalisé

Par défaut, Runique utilise `MemoryStore`. Pour la production, utilisez Redis, PostgreSQL, ou autre:

```rust
use tower_sessions::RedisStore;

let redis_pool = /* votre pool Redis */;
let session_store = RedisStore::new(redis_pool);

let app = RuniqueApp::builder(config)
    .with_session_store(session_store)  // ⚠️ Retourne RuniqueAppBuilderWithStore
    .with_session_duration(Duration::hours(12))
    .routes(router)
    .build()
    .await?;
```

**Note:** `with_session_store()` retourne un type différent (`RuniqueAppBuilderWithStore<Store>`), mais vous pouvez continuer à chaîner les méthodes normalement.

#### 🛡️ Middlewares

La protection CSRF est toujours activée (et non désactivable) pour garantir le bon fonctionnement des formulaires. Vous pouvez toutefois ajuster d'autres middlewares :

```rust
let app = RuniqueApp::builder(config)
    .with_sanitize(false)      // Désactiver sanitization (par défaut: true)
    .with_error_handler(false) // Désactiver error handler (par défaut: true)
    .routes(router)
    .build()
    .await?;
```

**Cas d'usage:**
- `with_sanitize(false)` - Validation custom des inputs
- `with_error_handler(false)` - Gestion d'erreurs personnalisée

#### 📁 Fichiers statiques

```rust
let app = RuniqueApp::builder(config)
    .with_static_files()  // Active le service de fichiers statiques
    .routes(router)
    .build()
    .await?;
```

### Exemples complets

#### Configuration minimale (développement)

```rust
use runique::{RuniqueApp, config_runique::RuniqueConfig};
use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env()?;
    let db = Database::connect(&config.database_url).await?;

    let router = Router::new()
        .route("/", get(home));

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(router)
        .build()
        .await?;

    app.run().await
}
```

#### Configuration production avec Redis

```rust
use tower_sessions::cookie::time::Duration;
use tower_sessions::RedisStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RuniqueConfig::from_env()?;
    let db = Database::connect(&config.database_url).await?;

    // Session store Redis pour production
    let redis_url = std::env::var("REDIS_URL")?;
    let redis_pool = redis::Client::open(redis_url)?;
    let session_store = RedisStore::new(redis_pool);

    let router = Router::new()
        .route("/", get(home))
        .route("/login", post(login));

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .with_session_store(session_store)
        .with_session_duration(Duration::hours(6))  // Session 6h
        .routes(router)
        .with_static_files()
        .build()
        .await?;

    app.run().await
}
```

#### Configuration de test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_config() {
        let config = RuniqueConfig::from_env().unwrap();
        let db = Database::connect("sqlite::memory:").await.unwrap();

        let app = RuniqueApp::builder(config)
            .with_database(db)
            .with_session_duration(Duration::minutes(5))  // Sessions courtes
            .with_error_handler(false)  // Erreurs explicites en test
            .routes(test_router())
            .build()
            .await
            .unwrap();

        // Tests...
    }
}
```

### Ordre d'appel recommandé

```rust
RuniqueApp::builder(config)
    // 1. Database
    .with_database(db)

    // 2. Session (optionnel)
    .with_session_store(store)  // ⚠️ Si utilisé, doit être avant les autres méthodes
    .with_session_duration(Duration::hours(2))

    // 3. Middlewares (optionnel)
    // CSRF est toujours activé par défaut (non désactivable)
    .with_sanitize(true)
    .with_error_handler(true)

    // 4. Routes (requis)
    .routes(router)

    // 5. Fichiers statiques (optionnel)
    .with_static_files()

    // 6. Build (requis)
    .build()
    .await?
```

### Valeurs par défaut

Si vous ne configurez rien, voici les valeurs par défaut:

| Configuration | Défaut |
|--------------|--------|
| **Session duration** | 24 heures |
| **Session store** | `MemoryStore` |
| **CSRF protection** | ✅ Activé (non désactivable) |
| **Sanitize** | ✅ Activé |
| **Error handler** | ✅ Activé |
| **Static files** | ❌ Désactivé (appeler `.with_static_files()`) |

---

## Prochaines étapes

→ [**Routage**](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md)
