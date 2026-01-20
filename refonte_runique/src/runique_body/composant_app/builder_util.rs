use axum::{Router, middleware, Extension};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::signal;
use sea_orm::DatabaseConnection;

use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::config_runique::config_struct::RuniqueConfig;
use crate::runique_body::composant_app::template_engine::TemplateLoader;
use crate::data_base_runique::DatabaseConfig;
use crate::gardefou::composant_middleware::{
    csrf::csrf_middleware,
    middleware_sanitiser::sanitize_middleware,
    flash_message::flash_middleware,
    error_handler::error_handler_middleware,
};

/// Structure unique de l'application
pub struct RuniqueApp {
    pub engine: Arc<RuniqueEngine>,
    pub router: Router, // L'état est déjà consommé ici
}

impl RuniqueApp {
    /// Unique méthode run pour lancer le serveur
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}",
            self.engine.config.server.ip_server,
            self.engine.config.server.port
        );

        println!("🦀 Runique Framework opérationnel");
        println!("   Serveur lancé sur http://{}", addr);
        // let moteur_db = self.engine.db.get_database_backend();
        let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "runique_db".to_string());
        println!("   Connected to database: {} ", db_name);
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
    db: Option<DatabaseConnection>,
    router: Router<Arc<RuniqueEngine>>,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl RuniqueAppBuilder {
    pub fn new(config: RuniqueConfig) -> Self {
        Self {
            config,
            db: None,
            router: Router::new(),
            url_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    pub fn add_routes(mut self, routes: Router<Arc<RuniqueEngine>>) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    pub async fn build(self) -> Result<RuniqueApp, Box<dyn std::error::Error>> {
        // 1. Initialisation de Tera
        let tera = TemplateLoader::init(&self.config)?;

        // 2. Création de l'Engine
        let engine = Arc::new(RuniqueEngine {
            config: self.config,
            tera: Arc::new(tera),
            db,
            url_registry: self.url_registry.clone(),
            garde: Default::default(),
        });

        let final_router = self.router
            .layer(Extension(engine.clone()))
            .layer(middleware::from_fn_with_state(engine.clone(), sanitize_middleware))
            .layer(middleware::from_fn_with_state(engine.clone(), csrf_middleware))
            .layer(middleware::from_fn(flash_middleware))
            .layer(middleware::from_fn(error_handler_middleware))
            .with_state(engine.clone());

        Ok(RuniqueApp {
            engine,
            router: final_router,
        })
    }
}