# Guide de configuration - Runique Framework

Runique utilise un système de configuration centralisé via la struct `Settings` et le fichier `.env`.

## Table des matières

1. [Structure Settings](#structure-settings)
2. [Configuration via .env](#configuration-via-env)
3. [Configuration programmatique](#configuration-programmatique)
4. [Variables d'environnement](#variables-denvironnement)
5. [Sécurité](#sécurité)
6. [Middleware](#middleware)

---

## Structure Settings

La struct `Settings` centralise toute la configuration de votre application Runique.

### Définition

```rust
pub struct Settings {
    pub server: ServerSettings,
    pub base_dir: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    pub installed_apps: Vec<String>,
    pub middleware: Vec<String>,
    pub root_urlconf: String,
    pub static_runique_path: String,
    pub static_runique_url: String,
    pub media_runique_path: String,
    pub media_runique_url: String,
    pub templates_runique: String,
    pub templates_dir: Vec<String>,
    pub staticfiles_dirs: String,
    pub media_root: String,
    pub static_url: String,
    pub media_url: String,
    pub staticfiles_storage: String,
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
    pub sanitize_inputs: bool,
    pub strict_csp: bool,
    pub rate_limiting: bool,
    pub enforce_https: bool,
    pub auth_password_validators: Vec<String>,
    pub password_hashers: Vec<String>,
    pub default_auto_field: String,
    pub logging_config: String,
}

pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}
```

### Chargement depuis `.env`

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charge automatiquement depuis .env ou utilise les valeurs par défaut
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## Configuration via .env

Créez un fichier `.env` à la racine de votre projet :

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000

# Sécurité
SECRET_KEY=votre-cle-secrete-tres-longue-et-aleatoire
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Base de données (PostgreSQL)
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# Fichiers statiques
STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

# Templates
TEMPLATES_DIR=templates/

# Sessions
SESSION_COOKIE_NAME=sessionid
SESSION_COOKIE_SECURE=false
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Lax

# CSRF
CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken

# Placeholder (non implémenté)
RATE_LIMITING=false
```

---

## Configuration programmatique

### Configuration manuelle

```rust
use runique::prelude::*;

let mut settings = Settings::default_values();

// Modifier après chargement
settings.server.port = 9000;
settings.allowed_hosts.push("api.example.com".to_string());

RuniqueApp::new(settings).await?
    .routes(routes())
    .with_default_middleware()
    .run()
    .await?;
```

### Modification des valeurs par défaut

```rust
let mut settings = Settings::default_values();

// Modifier après chargement
settings.server.port = 9000;
settings.allowed_hosts.push("api.example.com".to_string());

RuniqueApp::new(settings).await?
    .routes(routes())
    .with_default_middleware()
    .run()
    .await?;
```

---

## Variables d'environnement

### Serveur

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `IP_SERVER` | String | `127.0.0.1` | Adresse d'écoute du serveur |
| `PORT` | u16 | `3000` | Port d'écoute |

**Exemple :**

```env
IP_SERVER=0.0.0.0
PORT=3000
```

### Sécurité

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `SECRET_KEY` | String | **Requis** | Clé secrète pour CSRF/sessions (min 32 caractères) |
| `ALLOWED_HOSTS` | Vec | `[]` | Liste des domaines autorisés (séparés par virgule) |
| `DEBUG` | bool | `false` | Mode debug (affiche les erreurs détaillées) |

**Exemple :**

```env
SECRET_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,*.example.com
DEBUG=false
```

**⚠️ IMPORTANT :**
- `SECRET_KEY` doit faire **minimum 32 caractères**
- Générez-la avec : `openssl rand -base64 32`
- Ne commitez **JAMAIS** votre `.env` dans Git
- En production : `DEBUG=false` obligatoire

### Base de données

Voir [Guide de la base de données](DATABASE.md) pour la configuration complète.

```env
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb
```

### Fichiers statiques et media

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `STATIC_URL` | String | `/static/` | URL de base pour les fichiers statiques |
| `STATIC_ROOT` | Path | `static/` | Chemin physique des fichiers statiques |
| `MEDIA_URL` | String | `/media/` | URL de base pour les fichiers uploadés |
| `MEDIA_ROOT` | Path | `media/` | Chemin physique des fichiers uploadés |

**Exemple :**

```env
STATIC_URL=/static/
STATIC_ROOT=/var/www/myapp/static/
MEDIA_URL=/media/
MEDIA_ROOT=/var/www/myapp/media/
```

### Templates

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `TEMPLATES_DIR` | Path | `templates/` | Répertoire des templates Tera |

**Exemple :**

```env
TEMPLATES_DIR=templates/
```

### Sessions

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `SESSION_COOKIE_NAME` | String | `sessionid` | Nom du cookie de session |
| `SESSION_COOKIE_SECURE` | bool | `false` | Cookie uniquement en HTTPS |
| `SESSION_COOKIE_HTTPONLY` | bool | `true` | Cookie non accessible en JavaScript |
| `SESSION_COOKIE_SAMESITE` | String | `Lax` | Politique SameSite (`Strict`, `Lax`, `None`) |

**Exemple (production) :**

```env
SESSION_COOKIE_NAME=sessionid
SESSION_COOKIE_SECURE=true
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Strict
```

### CSRF

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `CSRF_COOKIE_NAME` | String | `csrftoken` | Nom du cookie CSRF |
| `CSRF_HEADER_NAME` | String | `X-CSRFToken` | Header HTTP pour les requêtes AJAX |

**Exemple :**

```env
CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken
```

### Rate Limiting

| Variable | Type | Défaut | Description |
|----------|------|--------|-------------|
| `RATE_LIMITING` | bool | `false` | ⚠️ **Placeholder non implémenté** |

**⚠️ IMPORTANT : Fonctionnalité non implémentée**

Le flag `RATE_LIMITING` existe dans la configuration mais **aucun middleware de rate limiting n'est actuellement implémenté dans Runique**.

**Si vous avez besoin de rate limiting :**

Vous pouvez intégrer manuellement la bibliothèque [tower-governor](https://crates.io/crates/tower-governor) :

```rust
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use std::time::Duration;

// Configuration : 10 requêtes par minute par IP
let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap(),
);

let governor_limiter = governor_conf.limiter().clone();
let governor_layer = GovernorLayer {
    config: Box::leak(governor_conf),
};

// Ajout au RuniqueApp
RuniqueApp::new(settings).await?
    .middleware(governor_layer)  // ✅ Rate limiting actif
    .routes(routes())
    .run()
    .await?;
```

**Roadmap future :**

Cette fonctionnalité est prévue pour une future version de Runique sous forme de middleware intégré. En attendant, utilisez `tower-governor` directement.

---

## Sécurité

### Génération de SECRET_KEY

```bash
# Méthode 1 : OpenSSL
openssl rand -base64 32

# Méthode 2 : Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# Méthode 3 : Rust
cargo add rand
```

```rust
use rand::Rng;
use rand::distributions::Alphanumeric;

fn generate_secret_key() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}
```

### Configuration ALLOWED_HOSTS

**Syntaxe :**

```env
# Domaines exacts
ALLOWED_HOSTS=example.com,www.example.com

# Wildcard pour sous-domaines
ALLOWED_HOSTS=*.example.com

# Localhost + production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Tous les sous-domaines ET domaine principal
ALLOWED_HOSTS=example.com,*.example.com
```

**⚠️ Sécurité :**
- Ne jamais utiliser `*` seul en production
- Toujours lister explicitement les domaines autorisés
- Les wildcards ne matchent qu'un seul niveau : `*.example.com` match `api.example.com` mais pas `v1.api.example.com`

### Mode DEBUG

```env
# Développement
DEBUG=true

# Production
DEBUG=false
```

**En mode DEBUG=true :**
- Affiche les stack traces complètes
- Logs verbeux
- Messages d'erreur détaillés

**En mode DEBUG=false (production) :**
- Erreurs génériques pour l'utilisateur
- Logs uniquement dans les fichiers
- Pas de stack traces exposées

---

## Middleware

### Configuration via RuniqueApp

```rust
use runique::prelude::*;
use runique::middleware::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    RuniqueApp::new(settings).await?
        .with_default_middleware()
        .routes(routes())
        .run()
        .await?;

    Ok(())
}
```

### Middleware disponibles

| Middleware | Description | Requis |
|------------|-------------|--------|
| `CsrfMiddleware` | Protection CSRF via token HMAC-SHA256 | ✅ Recommandé |
| `SecurityHeadersMiddleware` | Headers de sécurité HTTP | ✅ Recommandé |
| `AllowedHostsMiddleware` | Validation Host header | ✅ Recommandé |
| `FlashMiddleware` | Messages flash entre requêtes | Optionnel |
| `MessageMiddleware` | Messages utilisateur | Optionnel |
| `XssSanitizerMiddleware` | Sanitization XSS (ammonia) | ✅ Recommandé |
| `CspMiddleware` | Content Security Policy | ✅ Recommandé |

Voir [Guide de Sécurité](informations/documentation_french/CSP.md) pour les détails complets.

---

## Exemples de configuration

### Configuration développement

```env
# .env.development
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=dev-secret-key-change-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

DB_ENGINE=sqlite
DB_NAME=dev.sqlite

STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

TEMPLATES_DIR=templates/

SESSION_COOKIE_SECURE=false
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Lax
```

### Configuration production

```env
# .env.production
IP_SERVER=0.0.0.0
PORT=3000
SECRET_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0
ALLOWED_HOSTS=example.com,www.example.com,api.example.com

DB_ENGINE=postgres
DB_USER=produser
DB_PASSWORD=secure-password-here
DB_HOST=db.internal.example.com
DB_PORT=5432
DB_NAME=proddb

STATIC_URL=/static/
STATIC_ROOT=/var/www/example.com/static/
MEDIA_URL=/media/
MEDIA_ROOT=/var/www/example.com/media/

TEMPLATES_DIR=/var/www/example.com/templates/

SESSION_COOKIE_SECURE=true
SESSION_COOKIE_HTTPONLY=true
SESSION_COOKIE_SAMESITE=Strict

CSRF_COOKIE_NAME=csrftoken
CSRF_HEADER_NAME=X-CSRFToken
```

### Configuration Docker

```env
# .env.docker
IP_SERVER=0.0.0.0
PORT=3000
SECRET_KEY=${SECRET_KEY}
ALLOWED_HOSTS=localhost,app

DB_ENGINE=postgres
DB_USER=${POSTGRES_USER}
DB_PASSWORD=${POSTGRES_PASSWORD}
DB_HOST=postgres
DB_PORT=5432
DB_NAME=${POSTGRES_DB}

STATIC_URL=/static/
STATIC_ROOT=/app/static/
MEDIA_URL=/media/
MEDIA_ROOT=/app/media/

TEMPLATES_DIR=/app/templates/
```

---

## Bonnes pratiques

### 1. Ne jamais commiter le fichier .env

```gitignore
# .gitignore
.env
.env.*
!.env.example
```

### 2. Créer un .env.example

```env
# .env.example
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=change-me-in-production
ALLOWED_HOSTS=localhost,127.0.0.1

DB_ENGINE=postgres
DB_USER=your_user
DB_PASSWORD=your_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your_database

STATIC_URL=/static/
STATIC_ROOT=static/
MEDIA_URL=/media/
MEDIA_ROOT=media/

TEMPLATES_DIR=templates/
```

### 3. Utiliser différents fichiers .env par environnement

```bash
# Structure recommandée
.
├── .env                    # Ignoré par Git
├── .env.example           # Template committé
├── .env.development       # Config dev (ignoré)
├── .env.production        # Config prod (ignoré)
└── .env.docker           # Config Docker (ignoré)
```

### 4. Valider la configuration au démarrage

```rust
use runique::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default_values();

    // Validations
    assert!(settings.server.secret_key.len() >= 32, "SECRET_KEY trop courte");
    assert!(!settings.allowed_hosts.is_empty(), "ALLOWED_HOSTS vide");

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

### 5. Utiliser des secrets managés en production

```rust
// Exemple avec AWS Secrets Manager, Vault, etc.
use aws_sdk_secretsmanager::Client;

async fn load_secret_key() -> String {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    let response = client
        .get_secret_value()
        .secret_id("myapp/secret_key")
        .send()
        .await
        .unwrap();

    response.secret_string().unwrap().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = Settings::default_values();
    settings.server.secret_key = load_secret_key().await;

    RuniqueApp::new(settings).await?
        .routes(routes())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
```

---

## Voir aussi

- [Guide de démarrage](informations/documentation_french/GETTING_STARTED.md)
- [Sécurité](informations/documentation_french/CSP.md)
- [Base de données](informations/documentation_french/DATABASE.md)

Configurez Runique de manière sécurisée et efficace !

---

**Version:** 1.0.87 (17 Janvier 2026)
**Licence:** MIT
