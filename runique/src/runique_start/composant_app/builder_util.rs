use axum::{middleware, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tower_http::services::ServeDir;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

use crate::config_runique::config_struct::RuniqueConfig;
use crate::gardefou::composant_middleware::{
    csrf_middleware::csrf_middleware, error_handler::error_handler_middleware,
    middleware_sanitiser::sanitize_middleware,
};
use crate::macro_runique::router::flush_pending_urls;
use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::runique_start::composant_app::template_engine::TemplateLoader;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Structure unique de l'application
pub struct RuniqueApp {
    pub engine: Arc<RuniqueEngine>,
    pub router: Router,
}

impl RuniqueApp {
    /// Créer un builder pour configurer l'app
    pub fn builder(config: RuniqueConfig) -> RuniqueAppBuilder {
        RuniqueAppBuilder::new(config)
    }

    /// Unique méthode run pour lancer le serveur
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!(
            "{}:{}",
            self.engine.config.server.ip_server, self.engine.config.server.port
        );

        println!("🦀 Runique Framework opérationnel");
        println!("   Serveur lancé sur http://{}", addr);
        let moteur_db = self.engine.db.get_database_backend();
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
        println!("   Connected to database {:?} -> {} ", moteur_db, db_name);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        axum::serve(listener, self.router)
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.expect("Erreur signal Ctrl+C");
                println!("\nArrêt du serveur Runique...");
            })
            .await?;

        Ok(())
    }
}

/// Unique Builder pour assembler les pièces
pub struct RuniqueAppBuilder {
    config: RuniqueConfig,
    router: Router,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
    #[cfg(feature = "orm")]
    db: Option<DatabaseConnection>,
}

impl RuniqueAppBuilder {
    pub fn new(config: RuniqueConfig) -> Self {
        Self {
            config,
            router: Router::new(),
            url_registry: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "orm")]
            db: None,
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

    fn static_runique(mut router: Router, config: &RuniqueConfig) -> Router {
        // --- 1. DOSSIERS DU DÉVELOPPEUR (Projet) ---
        // Le dev garde le contrôle via sa config (ex: /static)
        router = router
            .nest_service(
                &config.static_files.static_url,
                ServeDir::new(&config.static_files.staticfiles_dirs),
            )
            .nest_service(
                &config.static_files.media_url,
                ServeDir::new(&config.static_files.media_root),
            );

        // --- 2. DOSSIERS DU FRAMEWORK (Runique) ---
        // Injection automatique des ressources internes
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

    pub fn with_static_files(mut self) -> Self {
        let config = &self.config;

        // Ajouter les routes pour les fichiers statiques du projet
        self.router = self
            .router
            .nest_service(
                &config.static_files.static_url,
                ServeDir::new(&config.static_files.staticfiles_dirs),
            )
            .nest_service(
                &config.static_files.media_url,
                ServeDir::new(&config.static_files.media_root),
            );

        self
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        // 1. Init Tera avec le url_registry qui sera utilisé par l'engine
        let url_registry = self.url_registry.clone();
        let tera = TemplateLoader::init(&self.config, url_registry)?;
        let tera = Arc::new(tera);

        // 2. Config Arc
        let config = Arc::new(self.config);

        // 3. Engine complet
        let engine = Arc::new(RuniqueEngine {
            config: (*config).clone(),
            tera: tera.clone(),
            #[cfg(feature = "orm")]
            db: Arc::new(self.db.expect("Database connection required")),
            garde: Default::default(),
            url_registry: self.url_registry.clone(),
            csp: Arc::new(Default::default()),
        });

        // 3b. Transférer les URLs en attente depuis la macro vers l'engine
        flush_pending_urls(&engine);

        let engine_ext = engine.clone();

        // Créer le session layer avec configuration sécurisée
        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(!config.debug)
            .with_http_only(!config.debug)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(86400)));

        let mut final_router = self
            .router
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                sanitize_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                csrf_middleware,
            ))
            .layer(session_layer)
            .layer(middleware::from_fn(error_handler_middleware))
            .layer(axum::middleware::from_fn(
                move |mut req: axum::http::Request<axum::body::Body>,
                      next: axum::middleware::Next| {
                    // Injecter Tera, Config, Engine - Session est automatiquement disponible via tower_sessions
                    req.extensions_mut().insert(tera.clone());
                    req.extensions_mut().insert(config.clone());
                    req.extensions_mut().insert(engine_ext.clone());
                    async { next.run(req).await }
                },
            ));
        final_router = Self::static_runique(final_router, &engine.config);
        Ok(RuniqueApp {
            engine,
            router: final_router,
        })
    }
}
