// rusti/src/app.rs
// use std::any::type_name;
use std::sync::Arc;
use std::net::SocketAddr;
use std::error::Error;
use axum::{Router, middleware, Extension};
use axum::http::StatusCode;
// use sea_orm::sea_query::value;
use tower::ServiceBuilder;
use tower_http::{
    services::ServeDir,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions::cookie::time::Duration;
use tokio::signal;
use tokio::net::TcpListener;
use tera::Tera;
use tera::Context;


#[cfg(feature = "orm")]
use crate::settings::Settings;
use crate::middleware::error_handler::error_handler_middleware;
use crate::middleware::error_handler::render_index;
use crate::middleware::flash_message::flash_middleware;

use crate::response::render_simple_404;




/// Structure principale de l'application Rusti
///
/// Encapsule toute la configuration et l'état de l'application
pub struct RustiApp {
    router: Router,
    config: Arc<Settings>,
    addr: SocketAddr,
    tera: Arc<Tera>,
}

impl RustiApp {
    /// Crée une nouvelle instance de RustiApp
    ///
    /// # Arguments
    /// * `settings` - Configuration de l'application
    ///
    /// # Exemple
    /// ```rust,no_run
    /// use rusti::{RustiApp, Settings};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let settings = Settings::default_values();
    ///     let app = RustiApp::new(settings).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let config = Arc::new(settings);
        let addr = config.server.domain_server.parse()?;

        let mut tera = Tera::default();

        // 1. CHARGER D'ABORD LES TEMPLATES DU FRAMEWORK (embarqués, inclus ici de manière statique)

        const INDEX_DEFAULT_TEMPLATE: &str = include_str!("../templates/index.html");
        const MESSAGE_FLASH_TEMPLATE: &str = include_str!("../templates/message.html");
        const ERROR_DEBUG_TEMPLATE: &str = include_str!("../templates/errors/debug_error.html");
        const ERROR_500_TEMPLATE: &str = include_str!("../templates/errors/500.html");
        const ERROR_404_TEMPLATE: &str = include_str!("../templates/errors/404.html");

        tera.add_raw_template("index.html", INDEX_DEFAULT_TEMPLATE)?;
        tera.add_raw_template("message", MESSAGE_FLASH_TEMPLATE)?;
        tera.add_raw_template("errors/404.html", ERROR_404_TEMPLATE)?;
        tera.add_raw_template("errors/500.html", ERROR_500_TEMPLATE)?;
        tera.add_raw_template("errors/debug_error.html", ERROR_DEBUG_TEMPLATE)?;

        const HEADER_ERROR: &str = include_str!("../templates/errors/corps-error/header-error.html");
        const MESSAGE_ERROR: &str = include_str!("../templates/errors/corps-error/message-error.html");
        const TEMPLATE_INFO: &str = include_str!("../templates/errors/corps-error/template-info.html");
        const STACK_TRACE: &str = include_str!("../templates/errors/corps-error/stack-trace-error.html");
        const REQUEST_INFO: &str = include_str!("../templates/errors/corps-error/request-info.html");
        const ENVIRONMENT_INFO: &str = include_str!("../templates/errors/corps-error/environment-info.html");
        const STATUS_CODE_INFO: &str = include_str!("../templates/errors/corps-error/status-code-info.html");
        const FOOTER_ERROR: &str = include_str!("../templates/errors/corps-error/footer-error.html");

        tera.add_raw_template("errors/corps-error/header-error.html", HEADER_ERROR)?;
        tera.add_raw_template("errors/corps-error/message-error.html", MESSAGE_ERROR)?;
        tera.add_raw_template("errors/corps-error/template-info.html", TEMPLATE_INFO)?;
        tera.add_raw_template("errors/corps-error/stack-trace-error.html", STACK_TRACE)?;
        tera.add_raw_template("errors/corps-error/request-info.html", REQUEST_INFO)?;
        tera.add_raw_template("errors/corps-error/environment-info.html", ENVIRONMENT_INFO)?;
        tera.add_raw_template("errors/corps-error/status-code-info.html", STATUS_CODE_INFO)?;
        tera.add_raw_template("errors/corps-error/footer-error.html", FOOTER_ERROR)?;


        // 2. CHARGER LES TEMPLATES UTILISATEUR
        let pattern = format!("{}/**/*.html", config.templates_dir.join(","));
        match Tera::new(&pattern) {
            Ok(t) => {
                tera.extend(&t).expect("Failed to extend Tera with user templates");
            }
            Err(e) => {
                println!("No user templates found in {} ({})", pattern, e);
            }
        }

        let tera = Arc::new(tera);
        let router = Router::new();

        Ok(Self {
            router,
            config,
            addr,
            tera,
        })
    }

    /// Configure les routes de l'application
    /// # Exemple
    /// ```rust,no_run
    /// use rusti::{RustiApp, Settings, Router, get, post, put, delete, patch};
    ///
    /// async fn index() -> &'static str {
    ///     "Hello, World!"
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let settings = Settings::default_values();
    ///     let app = RustiApp::new(settings).await?
    ///         .routes(Router::new().route("/", get(index)));
    ///     Ok(())
    /// }
    /// ```
    pub fn routes(mut self, routes: Router) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    /// Configure la base de données (SeaORM)
    #[cfg(feature = "orm")]
    pub async fn with_database(mut self) -> Result<Self, Box<dyn Error>> {
        use crate::db::connect_db;

        let db = Arc::new(connect_db(&self.config).await?);
        self.router = self.router.layer(Extension(db));
        Ok(self)
        }

    /// Configure les fichiers statiques et médias
    pub fn with_static_files(mut self) -> Result<Self, Box<dyn Error>> {
        let conf = self.config.as_ref();

        // 1. Fichiers statiques utilisateur
        let static_files = ServeDir::new(&conf.staticfiles_dirs);
        self.router = self.router.nest_service(&conf.static_url, static_files);

        // 2. Fichiers media utilisateur
        let media_files = ServeDir::new(&conf.media_root);
        self.router = self.router.nest_service(&conf.media_url, media_files);

        // 3. Fichiers statiques du framework - AVEC DEBUG
        if !conf.static_rusti_path.is_empty() {
            let static_files = ServeDir::new(&conf.static_rusti_path);
            self.router = self.router.nest_service(&conf.static_rusti_url, static_files);
        }

        // 4. Fichiers media du framework
        if !conf.media_rusti_path.is_empty() {
            let media_files = ServeDir::new(&conf.media_rusti_path);
            self.router = self.router.nest_service(&conf.media_rusti_url, media_files);
        }
        Ok(self)
    }


    /// Configure les middlewares par défaut (erreurs, timeouts, etc.)
    pub fn with_default_middleware(mut self) -> Self {
        let tera_for_fallback = self.tera.clone();
        let config_for_fallback = self.config.clone();

        self.router = self.router
            // Fallback 404 et index par défaut
            .fallback(move |uri: axum::http::Uri| {
                let tera = tera_for_fallback.clone();
                let config = config_for_fallback.clone();
                async move {
                    if uri.path() == "/" {
                        let context = Context::new();
                        return render_index(&tera, &context, &config);
                    }
                    render_simple_404(&tera)
                }
            })
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, std::time::Duration::from_secs(10))))
            .layer(middleware::from_fn(error_handler_middleware));

        self
    }
    pub fn build(self) -> Router {
        self.router
    }
    pub fn with_flash_messages(mut self) -> Self {
    self.router = self.router.layer(middleware::from_fn(flash_middleware));
    self
    }
    /// Lance le serveur
    ///
    /// # Exemple
    /// ```rust,no_run
    /// use rusti::{RustiApp, Settings};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let settings = Settings::default_values();
    ///     let app = RustiApp::new(settings).await?;
    ///     app.run().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        println!("🦀 Rusti Framework v{}", crate::VERSION);
        println!("   Starting server at http://{}", self.addr);

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(!self.config.debug)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(86400)));

        let router_with_extensions = self.router
            .layer(Extension(self.config.clone()))
            .layer(Extension(self.tera.clone()))
            .layer(session_layer);

    let listener: TcpListener = TcpListener::bind(&self.addr).await?;
    let server = axum::serve(listener, router_with_extensions);
        // Arrêt propre avec Ctrl+C
        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    eprintln!("Server error: {}", e);
                }
            },
            _ = signal::ctrl_c() => {
                println!("\nShutdown signal received. Stopping server...");
            },
        }
        Ok(())
    }
}

/// Builder pattern pour construire facilement une application
/// # Exemple
///
/// ```rust,no_run
///
/// use rusti::{RustiApp, Settings, Router, get, post, put, delete, patch};
///
/// async fn index() -> &'static str { "Hello!" }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let settings = Settings::default_values();
///
///     RustiApp::new(settings).await?
///         .routes(Router::new().route("/", get(index)))
///         .with_static_files()?
///         .with_flash_messages()
///         .with_default_middleware()
///         .run()
///         .await?;
///         Ok(())
/// }
/// ```
impl RustiApp {
    /// Builder pattern - crée et configure l'app en une chaîne
    pub async fn builder(settings: Settings) -> RustiAppBuilder {
        RustiAppBuilder {
            settings,
            routes: None,
        }
    }
}

pub struct RustiAppBuilder {
    settings: Settings,
    routes: Option<Router>,
}

impl RustiAppBuilder {
    pub fn routes(mut self, routes: Router) -> Self {
        self.routes = Some(routes);
        self
    }

    pub async fn build(self) -> Result<RustiApp, Box<dyn Error>> {
        let mut app = RustiApp::new(self.settings).await?;
        if let Some(routes) = self.routes {
            app = app.routes(routes);
        }
        Ok(app)
    }
}
