# 🔧 Refactorings Proposés - Amélioration de l'Architecture Runique

## 1️⃣ Macros: Un vrai système d'exports

### Problème Actuel
```rust
// ❌ Peu ergonomique
#[macro_use]
extern crate runique;
```

### Solution: Module `macros` dédié

**Créer**: `runique/src/lib.rs` - Section Macros
```rust
/// Module centralisant tous les exports de macros
pub mod macros {
    pub use crate::impl_objects;
    pub use crate::success;
    pub use crate::error;
    pub use crate::warning;
    pub use crate::info;
    pub use crate::context;
    pub use crate::flash_now;
    pub use crate::urlpatterns;
    pub use crate::view;
}
```

**Usage clean**:
```rust
use runique::prelude::*;
use runique::macros::*;  // ✅ Clair et découvrable
```

---

## 2️⃣ Configuration: Builder Pattern Completo

### Problème Actuel
```rust
// ❌ from_env() peu flexible
let config = RuniqueConfig::from_env();
```

### Solution: Builder Complet

**Créer/Modifier**: `runique/src/config_runique/config_builder.rs`

```rust
use crate::config_runique::config_struct::RuniqueConfig;

pub struct ConfigBuilder {
    ip: String,
    port: u16,
    secret: String,
    debug: bool,
    base_dir: String,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            ip: std::env::var("IP_SERVER").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            secret: std::env::var("SECRET_KEY").unwrap_or_else(|_| "change_me".to_string()),
            debug: std::env::var("DEBUG")
                .ok()
                .and_then(|d| d.parse().ok())
                .unwrap_or(cfg!(debug_assertions)),
            base_dir: std::env::var("BASE_DIR").unwrap_or_else(|_| ".".to_string()),
        }
    }

    pub fn ip(mut self, ip: impl Into<String>) -> Self {
        self.ip = ip.into();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = secret.into();
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.base_dir = dir.into();
        self
    }

    pub fn build(self) -> RuniqueConfig {
        RuniqueConfig::from_env()  // Utilise les defaults, puis override
    }
}
```

**Usage**:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .ip("0.0.0.0")
        .port(8080)
        .debug(false)
        .build();

    // ou avec env vars (default)
    let config = ConfigBuilder::new().build();

    // ...
}
```

---

## 3️⃣ Router State: Injection Propre

### Problème Actuel
```rust
// ❌ Workaround: injection par middleware custom
.layer(axum::middleware::from_fn(move |mut req, next| {
    req.extensions_mut().insert(tera.clone());
    ...
}))
```

### Solution: Type State Tuple avec FromRef

**Créer**: `runique/src/runique_body/composant_app/router_state.rs`

```rust
use std::sync::Arc;
use axum::extract::FromRef;
use tera::Tera;
use crate::config_runique::config_struct::RuniqueConfig;
use crate::moteur_engine::engine_struct::RuniqueEngine;

/// State tuple pour le router Axum
#[derive(Clone)]
pub struct RouterState {
    pub tera: Arc<Tera>,
    pub config: Arc<RuniqueConfig>,
    pub engine: Arc<RuniqueEngine>,
}

// Impléments FromRef pour chaque composant
impl FromRef<RouterState> for Arc<Tera> {
    fn from_ref(state: &RouterState) -> Self {
        state.tera.clone()
    }
}

impl FromRef<RouterState> for Arc<RuniqueConfig> {
    fn from_ref(state: &RouterState) -> Self {
        state.config.clone()
    }
}

impl FromRef<RouterState> for Arc<RuniqueEngine> {
    fn from_ref(state: &RouterState) -> Self {
        state.engine.clone()
    }
}
```

**Modifier**: `builder_util.rs`

```rust
use crate::runique_body::composant_app::router_state::RouterState;

pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
    let tera = Arc::new(TemplateLoader::init(&self.config)?);
    let config = Arc::new(self.config);
    
    let engine = Arc::new(RuniqueEngine {
        config: (*config).clone(),
        tera: tera.clone(),
        #[cfg(feature = "orm")]
        db: Arc::new(self.db.expect("Database connection required")),
        garde: Default::default(),
        url_registry: self.url_registry.clone(),
        csp: Arc::new(Default::default()),
    });

    let state = RouterState { tera, config, engine: engine.clone() };

    let final_router = self.router
        .with_state(state)  // ✅ Propre!
        .layer(middleware::from_fn_with_state(engine.clone(), sanitize_middleware))
        .layer(middleware::from_fn_with_state(engine.clone(), csrf_middleware))
        .layer(middleware::from_fn(flash_middleware))
        .layer(middleware::from_fn(error_handler_middleware));

    Ok(RuniqueApp { engine, router: final_router })
}
```

**Modifier**: `extracteur.rs`

```rust
// ✅ Plus de workaround! Utilise From ref directement
impl<S, T> FromRequest<S> for ExtractForm<T>
where
    S: Send + Sync,
    T: RuniqueForm,
    Arc<Tera>: FromRef<S>,
    Arc<RuniqueConfig>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let tera = Arc::<Tera>::from_ref(state);
        let config = Arc::<RuniqueConfig>::from_ref(state);
        // ... reste du code
    }
}
```

---

## 4️⃣ Plugin System Basique

### Créer: `runique/src/plugins/mod.rs`

```rust
use async_trait::async_trait;
use axum::Router;
use crate::runique_body::composant_app::router_state::RouterState;

/// Trait pour les plugins/extensions
#[async_trait]
pub trait RuniquePlugin: Send + Sync {
    /// Nom du plugin
    fn name(&self) -> &str;

    /// Initialiser le plugin
    async fn init(&self) -> Result<(), Box<dyn std::error::Error>>;

    /// Optionnellement modifier les routes
    fn routes(&self, router: Router<RouterState>) -> Router<RouterState> {
        router
    }

    /// Optionnellement ajouter des middlewares
    fn middlewares(&self) -> Vec<Box<dyn Fn(Router<RouterState>) -> Router<RouterState>>> {
        vec![]
    }
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn RuniquePlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: vec![],
        }
    }

    pub fn register(&mut self, plugin: Box<dyn RuniquePlugin>) {
        self.plugins.push(plugin);
    }

    pub async fn init_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in &self.plugins {
            plugin.init().await?;
        }
        Ok(())
    }
}
```

### Exemple d'Usage:

```rust
// Plugin utilisateur personnalisé
struct APIDocsPlugin;

#[async_trait]
impl RuniquePlugin for APIDocsPlugin {
    fn name(&self) -> &str {
        "API Docs"
    }

    async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("API Docs plugin initialized");
        Ok(())
    }

    fn routes(&self, router: Router<RouterState>) -> Router<RouterState> {
        router.route("/api/docs", get(|| async { "OpenAPI Spec" }))
    }
}

// Utilisation
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugins = PluginRegistry::new();
    plugins.register(Box::new(APIDocsPlugin));
    plugins.init_all().await?;

    // ...
}
```

---

## 5️⃣ Middleware Builder

### Créer: `runique/src/gardefou/middleware_builder.rs`

```rust
use axum::Router;
use crate::runique_body::composant_app::router_state::RouterState;

/// Builder pour les middlewares
pub struct MiddlewareBuilder {
    enable_csrf: bool,
    enable_sanitize: bool,
    enable_csp: bool,
    enable_security_headers: bool,
}

impl Default for MiddlewareBuilder {
    fn default() -> Self {
        Self {
            enable_csrf: true,
            enable_sanitize: true,
            enable_csp: true,
            enable_security_headers: true,
        }
    }
}

impl MiddlewareBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn csrf(mut self, enable: bool) -> Self {
        self.enable_csrf = enable;
        self
    }

    pub fn sanitize(mut self, enable: bool) -> Self {
        self.enable_sanitize = enable;
        self
    }

    pub fn csp(mut self, enable: bool) -> Self {
        self.enable_csp = enable;
        self
    }

    pub fn security_headers(mut self, enable: bool) -> Self {
        self.enable_security_headers = enable;
        self
    }

    pub fn apply(self, mut router: Router<RouterState>) -> Router<RouterState> {
        if self.enable_sanitize {
            router = router.layer(/* sanitize middleware */);
        }
        if self.enable_csrf {
            router = router.layer(/* csrf middleware */);
        }
        if self.enable_csp {
            router = router.layer(/* csp middleware */);
        }
        if self.enable_security_headers {
            router = router.layer(/* security headers middleware */);
        }
        router
    }
}
```

**Usage**:
```rust
let router = url::routes()
    .layer(MiddlewareBuilder::new()
        .csp(false)  // Disable CSP
        .build()
    );
```

---

## 📋 Plan d'Implémentation

1. **Phase 1** (Facile): Macros module + ConfigBuilder
2. **Phase 2** (Moyen): RouterState avec FromRef
3. **Phase 3** (Avancé): Plugin system + MiddlewareBuilder

**Priorité**: Phase 1 + 2 amélioreront déjà **massif** l'ergonomie

---

## 📊 Avant/Après

### Avant
```rust
#[macro_use]
extern crate runique;

use runique::prelude::*;

let config = RuniqueConfig::from_env();
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .build()
    .await?;
```

### Après
```rust
use runique::prelude::*;
use runique::macros::*;

let config = ConfigBuilder::new()
    .port(8080)
    .build();

let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .middlewares(MiddlewareBuilder::new().csp(false))
    .plugins(plugins)
    .build()
    .await?;
```

**Gains**:
- ✅ Imports clairs et découvrables
- ✅ Configuration programmatique flexible
- ✅ State propre (plus de workaround)
- ✅ Extensibilité via plugins
- ✅ Middlewares optionnels/configurables

---

Veux-tu que j'implémente une de ces phases?
