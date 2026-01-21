# ⚙️ Configuration

## RuniqueConfig

Toute la configuration se fait via `.env` et est chargée dans une struct `RuniqueConfig`.

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
IP_SERVER=0.0.0.0
PORT=8000
DEBUG=false
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
DB_PASSWORD=Studietudiant1.
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

async fn my_handler(ctx: RuniqueContext) -> Response {
    let config = &ctx.engine.config;
    
    println!("Debug mode: {}", config.debug);
    println!("Database: {}", config.database_url);
    println!("Secret key: {}", config.secret_key);
    println!("Allowed hosts: {:?}", config.allowed_hosts);
}
```

### Configuration conditionnelle

```rust
if ctx.engine.config.debug {
    // Mode debug: logs détaillés, templates rechargés
} else {
    // Mode production: cache templates, pas de logs sensibles
}

if ctx.engine.config.allowed_hosts.contains("*") {
    // ⚠️ Attention: tous les hosts sont autorisés
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

## Prochaines étapes

→ [**Routage**](./04-routing.md)
