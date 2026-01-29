use axum::Router;
use tokio::signal;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, SessionStore};

use crate::aliases::*;
use crate::app::templates::TemplateLoader;
use crate::config::RuniqueConfig;
use crate::context::RequestExtensions;
use crate::engine::RuniqueEngine;
use crate::macros::add_urls;
use crate::middleware::session::SessionConfig;
use crate::middleware::{HostPolicy, MiddlewareConfig, SecurityPolicy};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

pub struct RuniqueApp {
    pub engine: AEngine,
    pub router: Router,
}

impl RuniqueApp {
    pub fn builder(config: RuniqueConfig) -> RuniqueAppBuilder {
        RuniqueAppBuilder::new(config)
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!(
            "{}:{}",
            self.engine.config.server.ip_server, self.engine.config.server.port
        );

        println!("🦀 Runique Framework is operational");
        println!("   Server launched on http://{}", addr);

        #[cfg(feature = "orm")]
        {
            let moteur_db = self.engine.db.get_database_backend();
            let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
            println!("   Connected to database {:?} -> {} ", moteur_db, db_name);
        }

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(listener, self.router)
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.expect("Error signal Ctrl+C");
                println!("\nShutting down Runique server...");
            })
            .await?;

        Ok(())
    }
}

pub struct RuniqueAppBuilder {
    config: RuniqueConfig,
    router: Router,
    url_registry: ARlockmap,
    #[cfg(feature = "orm")]
    db: Bdd,
    features: MiddlewareConfig,
    session_config: SessionConfig,
}

pub struct RuniqueAppBuilderWithStore<Store: SessionStore + Clone> {
    base: RuniqueAppBuilder,
    session_store: Store,
}

impl RuniqueAppBuilder {
    pub fn new(config: RuniqueConfig) -> Self {
        let features = if config.debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        Self {
            config,
            router: Router::new(),
            url_registry: new_registry(),
            #[cfg(feature = "orm")]
            db: None,
            features,
            session_config: SessionConfig::default(),
        }
    }

    #[cfg(feature = "orm")]
    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    pub fn routes(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.session_config = self.session_config.with_duration(duration);
        self
    }

    pub fn with_error_handler(mut self, enable: bool) -> Self {
        self.features.enable_debug_errors = enable;
        self
    }

    pub fn with_session_store<S: SessionStore + Clone>(
        self,
        store: S,
    ) -> RuniqueAppBuilderWithStore<S> {
        RuniqueAppBuilderWithStore {
            base: self,
            session_store: store,
        }
    }

    fn static_runique(mut router: Router, config: &RuniqueConfig) -> Router {
        router = router
            .nest_service(
                &config.static_files.static_url,
                new_serve(&config.static_files.staticfiles_dirs),
            )
            .nest_service(
                &config.static_files.media_url,
                new_serve(&config.static_files.media_root),
            );

        if !config.static_files.static_runique_url.is_empty() {
            router = router.nest_service(
                &config.static_files.static_runique_url,
                new_serve(&config.static_files.static_runique_path),
            );
        }

        router
    }

    fn build_pipeline<S: SessionStore + Clone + Send + Sync + 'static>(
        router: Router,
        config: ARuniqueConfig,
        session_layer: Session<S>,
        engine: AEngine,
        tera: ATera,
    ) -> Router {
        let mut app_router = router;

        // ═══════════════════════════════════════════════════════════════
        // Superposition des middlewares => extremement important
        // ═══════════════════════════════════════════════════════════════

        // ÉTAPE 1 (Dernier exécuté) - Middlewares de sécurité
        app_router = RuniqueEngine::attach_middlewares(engine.clone(), app_router);

        // ÉTAPE 2 - Session
        app_router = app_router.layer(session_layer);

        // ÉTAPE 3 (Premier exécuté) - Injection Extensions
        // DOIT être EN DERNIER dans le code = PREMIER exécuté
        let engine_ext: AEngine = engine.clone();
        app_router = app_router.layer(axum::middleware::from_fn(
            move |mut req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let extensions = RequestExtensions::new()
                    .with_tera(tera.clone())
                    .with_config(config.clone())
                    .with_engine(engine_ext.clone());

                extensions.inject_request(&mut req);
                async move { next.run(req).await }
            },
        ));

        // Statiques - En dehors de la pile
        Self::static_runique(app_router, &engine.config)
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        let tera = new(TemplateLoader::init(
            &self.config,
            self.url_registry.clone(),
        )?);
        let config = new(self.config);

        let engine = new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: new(self.db.expect("Database required")),
            features: self.features.clone(),
            url_registry: self.url_registry.clone(),
            security_csp: new(SecurityPolicy::from_env()),
            security_hosts: new(HostPolicy::from_env()),
        });

        add_urls(&engine);

        let session_layer = Session::new(MemoryStore::default())
            .with_secure(!config.debug)
            .with_http_only(!config.debug)
            .with_expiry(Expiry::OnInactivity(self.session_config.duration));

        let router = Self::build_pipeline(self.router, config, session_layer, engine.clone(), tera);

        Ok(RuniqueApp { engine, router })
    }
}

impl<Store: SessionStore + Clone> RuniqueAppBuilderWithStore<Store> {
    pub fn routes(mut self, router: Router) -> Self {
        self.base.router = router;
        self
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        let base = self.base;
        let tera = new(TemplateLoader::init(
            &base.config,
            base.url_registry.clone(),
        )?);
        let config = new(base.config);

        let engine = new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: new(base.db.expect("Database required")),
            features: base.features.clone(),
            url_registry: base.url_registry.clone(),
            security_csp: new(SecurityPolicy::from_env()),
            security_hosts: new(HostPolicy::from_env()),
        });

        add_urls(&engine);

        let session_layer = SessionManagerLayer::new(self.session_store)
            .with_secure(!config.debug)
            .with_http_only(!config.debug)
            .with_expiry(Expiry::OnInactivity(base.session_config.duration));

        let router = RuniqueAppBuilder::build_pipeline(
            base.router,
            config,
            session_layer,
            engine.clone(),
            tera,
        );

        Ok(RuniqueApp { engine, router })
    }
}
