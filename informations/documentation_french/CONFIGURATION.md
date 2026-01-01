# ⚙️ Configuration - Rusti Framework

Guide complet de configuration de votre application Rusti.

## Table des matières

1. [Méthodes de configuration](#méthodes-de-configuration)
2. [Settings](#settings)
3. [Variables d'environnement](#variables-denvironnement)
4. [Configuration du serveur](#configuration-du-serveur)
5. [Fichiers statiques et média](#fichiers-statiques-et-média)
6. [Middleware](#middleware)
7. [Production](#production)

---

## Méthodes de configuration

Rusti propose 3 façons de configurer votre application :

### 1. Valeurs par défaut

```rust
use rusti::Settings;

let settings = Settings::default_values();
```

### 2. Depuis variables d'environnement

```rust
let settings = Settings::from_env();
```

### 3. Builder pattern (recommandé)

```rust
let settings = Settings::builder()
    .debug(true)
    .templates_dir(vec!["templates".to_string()])
    .server("127.0.0.1", 3000, "secret-key")
    .build();
```

---

## Settings

### Structure complète

```rust
pub struct Settings {
    // Serveur
    pub server: ServerSettings,
    
    // Projet
    pub base_dir: String,
    pub debug: bool,
    pub allowed_hosts: Vec<String>,
    
    // Templates
    pub templates_dir: Vec<String>,
    
    // Fichiers statiques (projet)
    pub staticfiles_dirs: String,
    pub static_url: String,
    
    // Fichiers média (uploads)
    pub media_root: String,
    pub media_url: String,
    
    // Framework Rusti (interne)
    pub static_rusti_path: String,
    pub static_rusti_url: String,
    pub media_rusti_path: String,
    pub media_rusti_url: String,
    
    // Internationalisation
    pub language_code: String,
    pub time_zone: String,
    pub use_i18n: bool,
    pub use_tz: bool,
}
```

### Paramètres du serveur

```rust
pub struct ServerSettings {
    pub ip_server: String,
    pub domain_server: String,
    pub port: u16,
    pub secret_key: String,
}
```

---

## Variables d'environnement

### Fichier `.env`

Créez un fichier `.env` à la racine de votre projet :

```env
# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=changez-cette-clef-en-production

# Base de données PostgreSQL
DB_ENGINE=postgres
DB_USER=myuser
DB_PASSWORD=mypassword
DB_HOST=localhost
DB_PORT=5432
DB_NAME=mydb

# Base de données MySQL
# DB_ENGINE=mysql
# DB_USER=root
# DB_PASSWORD=secret
# DB_HOST=localhost
# DB_PORT=3306
# DB_NAME=mydb

# Base de données SQLite
# DB_ENGINE=sqlite
# DB_NAME=database.sqlite
```

### Fichier `.env.example`

Créez un template pour les autres développeurs :

```env
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=your-secret-key-here

DB_ENGINE=postgres
DB_USER=your-db-user
DB_PASSWORD=your-db-password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=your-db-name
```

### Charger les variables

```rust
use rusti::Settings;

// Charge automatiquement depuis .env
let settings = Settings::from_env();
```

---

## Configuration du serveur

### IP et Port

```rust
let settings = Settings::builder()
    .server("127.0.0.1", 3000, "secret")
    .build();

// Ou depuis .env
// IP_SERVER=0.0.0.0
// PORT=8080
```

### Clef secrète

La clef secrète est utilisée pour :
- Génération des tokens CSRF
- Signature des sessions
- Cryptage des cookies

**⚠️ IMPORTANT :** Changez la clef secrète en production !

```rust
// ❌ Mauvais - clef par défaut
.server("127.0.0.1", 3000, "default_secret_key")

// ✅ Bon - clef unique et longue
.server("127.0.0.1", 3000, "8k2jF9mN4pQr7sW1xY5zA3bC6eD8gH0j")

// ✅ Meilleur - depuis variable d'environnement
let secret = std::env::var("SECRET_KEY")?;
.server("127.0.0.1", 3000, &secret)
```

### Générer une clef secrète

```bash
# Linux/Mac
openssl rand -hex 32

# Ou en Rust
use rand::Rng;
let key: String = rand::thread_rng()
    .sample_iter(&rand::distributions::Alphanumeric)
    .take(64)
    .map(char::from)
    .collect();
```

---

## Fichiers statiques et média

### Configuration

```rust
let settings = Settings::builder()
    // Fichiers statiques (CSS, JS, images du projet)
    .staticfiles_dirs("static")
    .static_url("/static")
    
    // Fichiers média (uploads utilisateurs)
    .media_root("media")
    .media_url("/media")
    
    .build();
```

### Structure recommandée

```
mon-projet/
├── static/              # Fichiers statiques du projet
│   ├── css/
│   │   └── main.css
│   ├── js/
│   │   └── app.js
│   └── images/
│       └── logo.png
│
└── media/               # Fichiers uploadés
    ├── avatars/
    ├── documents/
    └── uploads/
```

### Utilisation dans les templates

```html
<!-- Fichiers statiques du projet -->
<link rel="stylesheet" href='{% static "css/main.css" %}'>
<script src='{% static "js/app.js" %}'></script>
<img src='{% static "images/logo.png" %}' alt="Logo">

<!-- Fichiers uploadés -->
<img src='{% media "avatars/user-123.jpg" %}' alt="Avatar">
<a href='{% media "documents/report.pdf" %}'>Télécharger</a>
```

### Servir les fichiers

```rust
RustiApp::new(settings).await?
    .routes(routes())
    .with_static_files()? // ✅ Active le service des fichiers
    .run()
    .await?;
```

---

## Middleware

### Middleware disponibles

```rust
RustiApp::new(settings).await?
    .routes(routes())
    
    // Fichiers statiques
    .with_static_files()?
    
    // Flash messages
    .with_flash_messages()
    
    // Protection CSRF
    .with_csrf_tokens()
    
    // Middleware par défaut (erreurs + timeout)
    .with_default_middleware()
    
    .run()
    .await?;
```

### Middleware personnalisé

```rust
use axum::middleware::{Next, from_fn};
use axum::extract::Request;
use axum::response::Response;

async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {
    // Logique avant la requête
    println!("Requête: {} {}", request.method(), request.uri());
    
    let response = next.run(request).await;
    
    // Logique après la requête
    println!("Status: {}", response.status());
    
    response
}

// Ajouter le middleware
let app = RustiApp::new(settings).await?
    .routes(routes())
    .build()
    .layer(from_fn(my_middleware));
```

---

## Templates

### Configuration

```rust
let settings = Settings::builder()
    .templates_dir(vec![
        "templates".to_string(),
        "custom_templates".to_string(),
    ])
    .build();
```

### Plusieurs répertoires

Rusti cherche les templates dans l'ordre des répertoires :

```rust
.templates_dir(vec![
    "templates".to_string(),       // Priorité 1
    "shared/templates".to_string(), // Priorité 2
    "vendor/templates".to_string(), // Priorité 3
])
```

---

## Production

### Configuration de production

```rust
let settings = Settings::builder()
    .debug(false) // ✅ Désactiver le mode debug
    .server("0.0.0.0", 8080, &env::var("SECRET_KEY")?)
    .build();
```

### Variables d'environnement en production

```env
# .env.production
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=votre-tres-longue-clef-secrete-unique

DB_ENGINE=postgres
DB_URL=postgresql://user:pass@prod-host:5432/prod_db
```

### Sécurité

#### 1. Toujours désactiver le mode debug

```rust
// ❌ Danger en production
.debug(true)

// ✅ Bon
.debug(false)
```

#### 2. Utiliser HTTPS

```nginx
# nginx.conf
server {
    listen 443 ssl http2;
    server_name monapp.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

#### 3. Restrictions CORS

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("https://monapp.com".parse::<HeaderValue>()?)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any);

let app = RustiApp::new(settings).await?
    .routes(routes())
    .build()
    .layer(cors);
```

#### 4. Rate limiting

```rust
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap()
);

let app = RustiApp::new(settings).await?
    .routes(routes())
    .build()
    .layer(GovernorLayer { config: governor_conf });
```

### Build optimisé

```bash
# Build de production
cargo build --release

# Stripping des symboles de debug
strip target/release/mon-app

# Avec optimisations LTO
RUSTFLAGS="-C lto=fat" cargo build --release
```

### Fichier `Cargo.toml` optimisé

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

---

## Logging et tracing

### Configuration basique

```rust
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // ...
}
```

### Configuration avancée

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

### Niveaux de log

```env
# .env
RUST_LOG=debug           # Tout en debug
RUST_LOG=info            # Info et au-dessus
RUST_LOG=warn            # Warnings et erreurs seulement
RUST_LOG=error           # Erreurs seulement

# Spécifique par module
RUST_LOG=mon_app=debug,rusti=info,sea_orm=warn
```

---

## Exemple complet

### `src/main.rs`

```rust
use rusti::prelude::*;
use std::env;

mod urls;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // Charger .env
    dotenvy::dotenv().ok();
    
    // Configuration
    let is_production = env::var("PRODUCTION")
        .unwrap_or_else(|_| "false".to_string())
        == "true";
    
    let settings = Settings::builder()
        .debug(!is_production)
        .templates_dir(vec!["templates".to_string()])
        .staticfiles_dirs("static")
        .media_root("media")
        .server(
            &env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string()),
            env::var("PORT")?.parse()?,
            &env::var("SECRET_KEY")?,
        )
        .build();
    
    // Base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    
    tracing::info!("🦀 Starting Rusti application");
    
    // Lancer l'application
    RustiApp::new(settings).await?
        .with_database(db)
        .routes(urls::routes())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;
    
    Ok(())
}
```

### `.env`

```env
# Mode
PRODUCTION=false

# Serveur
IP_SERVER=127.0.0.1
PORT=3000
SECRET_KEY=dev-secret-key-change-in-production

# Base de données
DB_ENGINE=postgres
DB_USER=dev_user
DB_PASSWORD=dev_password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=dev_db

# Logs
RUST_LOG=debug
```

### `.env.production`

```env
# Mode
PRODUCTION=true

# Serveur
IP_SERVER=0.0.0.0
PORT=8080
SECRET_KEY=prod-very-long-secret-key-change-me

# Base de données
DB_URL=postgresql://user:pass@prod-host/prod_db

# Logs
RUST_LOG=info
```

---

## Checklist de production

- [ ] Mode debug désactivé (`debug = false`)
- [ ] Clef secrète unique et sécurisée
- [ ] HTTPS configuré (via nginx/Caddy)
- [ ] Base de données de production configurée
- [ ] Logs configurés (niveau INFO ou WARN)
- [ ] CORS configuré selon vos besoins
- [ ] Rate limiting activé
- [ ] Build en mode `--release`
- [ ] Variables d'environnement sécurisées
- [ ] Sauvegardes automatiques de la DB
- [ ] Monitoring (Prometheus, Grafana, etc.)

---

## Voir aussi

- 🚀 [Guide de démarrage](GETTING_STARTED.md)
- 🗄️ [Base de données](DATABASE.md)
- 📖 [Templates](TEMPLATES.md)

**Configurez efficacement votre application Rusti ! 🦀**
