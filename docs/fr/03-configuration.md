# ⚙️ Configuration

## RuniqueConfig

Toute la configuration est facilitée via `.env` et est chargée dans une struct `RuniqueConfig`.

### Charger la configuration

```rust
use runique::prelude::*;

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
use runique::prelude::*;

pub async fn my_handler(request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Allowed hosts: {:?}", config.security.allowed_hosts);
    println!("Secret key: {}", config.security.secrete_key);
}
```

### Configuration conditionnelle

```rust
if request.engine.config.debug {
    // Mode debug: logs détaillés, templates rechargés
} else {
    // Mode production: cache templates, pas de logs sensibles
}

if request.engine.config.security.allowed_hosts.contains("*") {
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

En plus de la configuration via le fichier `.env`, le builder offre des méthodes pour personnaliser directement votre application.

### Builder classique

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)
    .with_csp(true)
    .with_allowed_hosts(true)
    .with_cache(true)
    .with_static_files()
    .build()
    .await?;

app.run().await?;
```

### Builder Intelligent (nouveau)

Le Builder Intelligent simplifie la configuration et gère automatiquement l'ordre des middlewares :

```rust
use runique::app::RuniqueAppBuilder as IntelligentBuilder;

let app = IntelligentBuilder::new(config)
    .routes(url::routes())
    .with_database(db)
    .statics()
    .build()
    .await?;

app.run().await?;
```

### Méthodes du Builder

#### 📦 Base de données

```rust
// Option 1 : connexion directe
let db_config = DatabaseConfig::from_env()?.build();
let db = db_config.connect().await?;

let app = RuniqueApp::builder(config)
    .with_database(db)
    .routes(router)
    .build()
    .await?;

// Option 2 : configuration déférée (Builder Intelligent)
let db_config = DatabaseConfig::from_env()?.build();

let app = IntelligentBuilder::new(config)
    .with_database_config(db_config)  // Connexion lors du .build()
    .routes(router)
    .build()
    .await?;
```

#### 🔄 Routes

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

#### ⏱️ Durée de session

```rust
use tower_sessions::cookie::time::Duration;

let app = RuniqueApp::builder(config)
    .with_session_duration(Duration::hours(2))  // Par défaut : 24h
    .routes(router)
    .build()
    .await?;
```

**Exemples de durées :**
```rust
Duration::hours(2)      // 2 heures
Duration::days(7)       // 7 jours
Duration::minutes(30)   // 30 minutes
```

#### 🛡️ Middlewares (Builder classique)

```rust
let app = RuniqueApp::builder(config)
    .with_error_handler(true)   // Capture des erreurs (défaut : true)
    .with_csp(true)             // CSP & headers sécurité (défaut : false en debug)
    .with_allowed_hosts(true)   // Validation des hosts (défaut : false en debug)
    .with_cache(true)           // No-cache en dev (défaut : true)
    .routes(router)
    .build()
    .await?;
```

#### 🛡️ Middlewares (Builder Intelligent)

Le Builder Intelligent utilise le **profil debug/production** pour les valeurs par défaut :

```rust
let app = IntelligentBuilder::new(config)
    .routes(router)
    .middleware(|m| {
        m.disable_csp();             // Désactiver CSP
        m.disable_host_validation(); // Désactiver la validation des hosts
    })
    .build()
    .await?;
```

> En mode debug, CSP et host validation sont désactivés par défaut. En production, tout est activé.

#### 📁 Fichiers statiques

```rust
// Builder classique
let app = RuniqueApp::builder(config)
    .with_static_files()
    .build()
    .await?;

// Builder Intelligent
let app = IntelligentBuilder::new(config)
    .statics()     // Active les fichiers statiques
    // ou
    .no_statics()  // Désactive explicitement
    .build()
    .await?;
```

### Valeurs par défaut

| Configuration | Défaut | Notes |
|--------------|--------|-------|
| **Session duration** | 24 heures | |
| **Session store** | `MemoryStore` | |
| **CSRF protection** | ✅ Toujours activé | Non désactivable |
| **Error handler** | ✅ Activé | |
| **CSP** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Host validation** | Debug: ❌ / Prod: ✅ | Selon le mode |
| **Cache control** | ✅ Activé | No-cache en debug |
| **Static files** | ❌ Désactivé | Appeler `.statics()` ou `.with_static_files()` |

---

## Prochaines étapes

← [**Architecture**](https://github.com/seb-alliot/runique/blob/main/docs/fr/02-architecture.md) | [**Routing**](https://github.com/seb-alliot/runique/blob/main/docs/fr/04-routing.md) →
