use axum::{middleware, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tower_http::services::ServeDir;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, SessionStore};

use crate::app::templates::TemplateLoader;
use crate::config::RuniqueConfig;
use crate::context::RequestExtensions;
use crate::engine::RuniqueEngine;
use crate::macros::router::add_urls;
use crate::middleware::session::SessionConfig;
use crate::middleware::{csrf_middleware, error_handler_middleware, MiddlewareConfig};

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

pub struct RuniqueApp {
    pub engine: Arc<RuniqueEngine>,
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
        let moteur_db = self.engine.db.get_database_backend();
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
        println!("   Connected to database {:?} -> {} ", moteur_db, db_name);
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
    url_registry: Arc<RwLock<HashMap<String, String>>>,
    #[cfg(feature = "orm")]
    db: Option<DatabaseConnection>,
    middleware_config: MiddlewareConfig,
    session_config: SessionConfig,
}

pub struct RuniqueAppBuilderWithStore<Store: SessionStore + Clone> {
    base: RuniqueAppBuilder,
    session_store: Store,
}

impl RuniqueAppBuilder {
    pub fn new(config: RuniqueConfig) -> Self {
        let middleware_config = if config.debug {
            MiddlewareConfig::development()
        } else {
            MiddlewareConfig::production()
        };

        Self {
            config,
            router: Router::new(),
            url_registry: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "orm")]
            db: None,
            middleware_config,
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
        self.middleware_config = self.middleware_config.with_error_handler(enable);
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
                ServeDir::new(&config.static_files.staticfiles_dirs),
            )
            .nest_service(
                &config.static_files.media_url,
                ServeDir::new(&config.static_files.media_root),
            );

        if !config.static_files.static_runique_url.is_empty() {
            router = router.nest_service(
                &config.static_files.static_runique_url,
                ServeDir::new(&config.static_files.static_runique_path),
            );
        }

        if !config.static_files.media_runique_url.is_empty() {
            router = router.nest_service(
                &config.static_files.media_runique_url,
                ServeDir::new(&config.static_files.media_runique_path),
            );
        }

        router
    }

    fn build_pipeline<S: SessionStore + Clone + Send + Sync + 'static>(
        mut router: Router,
        engine: Arc<RuniqueEngine>,
        tera: Arc<tera::Tera>,
        config: Arc<RuniqueConfig>,
        middleware_config: MiddlewareConfig,
        session_layer: SessionManagerLayer<S>,
    ) -> Router {
        let engine_ext = engine.clone();

        router = router.layer(middleware::from_fn_with_state(
            engine.clone(),
            csrf_middleware,
        ));

        router = router.layer(session_layer);

        if middleware_config.enable_error_handler {
            router = router.layer(middleware::from_fn(error_handler_middleware));
        }

        router = router.layer(axum::middleware::from_fn(
            move |mut req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let extensions = RequestExtensions::new()
                    .with_tera(tera.clone())
                    .with_config(config.clone())
                    .with_engine(engine_ext.clone());

                extensions.inject_request(&mut req);
                async move { next.run(req).await }
            },
        ));

        Self::static_runique(router, &engine.config)
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        let tera = Arc::new(TemplateLoader::init(
            &self.config,
            self.url_registry.clone(),
        )?);

        let config = Arc::new(self.config);

        let engine = Arc::new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: Arc::new(self.db.expect("Database required")),
            middleware_config: self.middleware_config.clone(),
            url_registry: self.url_registry.clone(),
            csp: Arc::new(Default::default()),
        });

        add_urls(&engine);

        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(!config.debug)
            .with_http_only(!config.debug)
            .with_expiry(Expiry::OnInactivity(self.session_config.duration));

        let router = Self::build_pipeline(
            self.router,
            engine.clone(),
            tera,
            config,
            self.middleware_config,
            session_layer,
        );

        Ok(RuniqueApp { engine, router })
    }
}

impl<Store: SessionStore + Clone> RuniqueAppBuilderWithStore<Store> {
    pub fn routes(mut self, router: Router) -> Self {
        self.base.router = router;
        self
    }

    pub fn with_session_duration(mut self, duration: Duration) -> Self {
        self.base.session_config = self.base.session_config.with_duration(duration);
        self
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        let base = self.base;

        let tera = Arc::new(TemplateLoader::init(
            &base.config,
            base.url_registry.clone(),
        )?);

        let config = Arc::new(base.config);

        let engine = Arc::new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: Arc::new(base.db.expect("Database required")),
            middleware_config: base.middleware_config.clone(),
            url_registry: base.url_registry.clone(),
            csp: Arc::new(Default::default()),
        });

        add_urls(&engine);

        let session_layer = SessionManagerLayer::new(self.session_store)
            .with_secure(!config.debug)
            .with_http_only(!config.debug)
            .with_expiry(Expiry::OnInactivity(base.session_config.duration));

        let router = RuniqueAppBuilder::build_pipeline(
            base.router,
            engine.clone(),
            tera,
            config,
            base.middleware_config,
            session_layer,
        );

        Ok(RuniqueApp { engine, router })
    }
}
