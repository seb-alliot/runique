//! Main application builder and runner
//!
//! This module provides the `RuniqueApp` struct, which is the main entry point
//! for building and running a Runique web application. It handles template loading,
//! middleware configuration, routing, and server lifecycle.
//!
//! # Examples
//!
//! ```no_run
//! use runique::{RuniqueApp, Settings, DatabaseConfig};
//! use runique::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let settings = Settings::default_values();
//!     let db_config = DatabaseConfig::from_env()?.build();
//!     let db = db_config.connect().await?;
//!
//!     RuniqueApp::new(settings)
//!         .await?
//!         .with_database(db)
//!         .with_static_files()?
//!         .with_default_middleware()
//!         .run()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use axum::http::StatusCode;
use axum::{middleware, Extension, Router};
use glob;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};

use sea_orm::DatabaseConnection;

use crate::middleware::csp::{security_headers_middleware, CspConfig};
use crate::middleware::csrf::csrf_middleware;
use crate::middleware::error_handler::{error_handler_middleware, render_index};
use crate::middleware::flash_message::flash_middleware;
use crate::middleware::middleware_sanetiser::sanitize_middleware;
use crate::response::render_404;
use crate::settings::Settings;

/// Main Runique application
///
/// This is the central struct for building and configuring a Runique web application.
/// It uses a builder pattern to allow fluent configuration of routes, middleware,
/// database connections, and other features.
///
/// # Examples
///
/// ```no_run
/// use runique::{RuniqueApp, Settings};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let settings = Settings::default_values();
///
/// RuniqueApp::new(settings)
///     .await?
///     .with_default_middleware()
///     .run()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct RuniqueApp {
    router: Router<Arc<Tera>>,
    config: Arc<Settings>,
    addr: SocketAddr,
    tera: Arc<Tera>,
}

impl RuniqueApp {
    /// Creates a new Runique application
    ///
    /// Initializes the application with the provided settings, sets up the
    /// template engine, and loads both internal and user templates.
    ///
    /// # Template Processing
    ///
    /// This method automatically processes templates to support Django-style syntax:
    /// - `{% csrf %}` → CSRF token inclusion
    /// - `{% messages %}` → Flash message display
    /// - `{% static "path" %}` → Static file URL
    /// - `{% media "path" %}` → Media file URL
    /// - `{% link "name" %}` → Named URL reverse lookup
    ///
    /// # Arguments
    ///
    /// * `settings` - Application settings
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::builder()
    ///     .debug(true)
    ///     .server("127.0.0.1", 8000, "secret-key")
    ///     .build();
    ///
    /// let app = RuniqueApp::new(settings).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template loading fails
    /// - Invalid socket address in settings
    /// - Template directory is inaccessible
    pub async fn new(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let config = Arc::new(settings);
        let addr = config.server.domain_server.parse()?;

        let mut tera = Tera::default();
        Self::load_internal_templates(&mut tera)?;

        // 2. Traitement des templates utilisateurs (Regex tags)
        let mut all_templates = Vec::new();
        let balise_link =
            regex::Regex::new(r#"\{%\s*(?P<tag>static|media)\s*['"](?P<link>[^'"]+)['"]\s*%}"#)
                .unwrap();
        let link_regex = regex::Regex::new(
            r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s*(?:,\s*)?(?P<params>[^%]*?)\s*%}"#,
        )
        .unwrap();
        let re =
            regex::Regex::new(r#"\{%\s*form\.([a-zA-Z0-9_]+)(?:\.([a-zA-Z0-9_]+))?\s*%}"#).unwrap();

        for dir_string in &config.templates_dir {
            let template_dir = std::path::Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let mut content = std::fs::read_to_string(&entry)?;

                    content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
                    content = content.replace("{% messages %}", r#"{% include "message" %}"#);
                    content = content.replace("{{ csp }}", r#"{% include "csp" %}"#);

                    // Transformation pour les formulaire
                    content = re
                        .replace_all(&content, |caps: &regex::Captures| {
                            let form_name = &caps[1];

                            match caps.get(2) {
                                // Cas A : {% form.user.email %} -> rendu d'un champ seul
                                Some(field_name) => {
                                    format!(
                                        "{{{{ {} | form(field='{}') | safe }}}}",
                                        form_name,
                                        field_name.as_str()
                                    )
                                }
                                // Cas B : {% form.user %} -> rendu du formulaire complet
                                None => {
                                    format!("{{{{ {} | form | safe }}}}", form_name)
                                }
                            }
                        })
                        .to_string();

                    // Transformation unifiée pour {% link "name" %}
                    content = link_regex
                        .replace_all(&content, |caps: &regex::Captures| {
                            let name = &caps["name"];
                            let params = caps
                                .name("params")
                                .map(|m| m.as_str().trim())
                                .filter(|s| !s.is_empty());

                            if let Some(params) = params {
                                format!(r#"{{{{ link(link='{}', {}) }}}}"#, name, params)
                            } else {
                                format!(r#"{{{{ link(link='{}') }}}}"#, name)
                            }
                        })
                        .to_string();

                    content = balise_link
                        .replace_all(&content, |caps: &regex::Captures| {
                            let tag = &caps["tag"];
                            let link = &caps["link"];
                            format!(r#"{{{{ "{}" | {} }}}}"#, link, tag)
                        })
                        .to_string();

                    let name = entry
                        .strip_prefix(template_dir)?
                        .to_string_lossy()
                        .replace("\\", "/");

                    all_templates.push((name, content));
                }
            }
        }

        tera.add_raw_templates(all_templates)?;

        crate::tera_function::static_balise::register_all_asset_filters(
            &mut tera,
            config.static_url.clone(),
            config.media_url.clone(),
            config.static_runique_url.clone(),
            config.media_runique_url.clone(),
        );

        crate::tera_function::url_balise::register_url(&mut tera);

        let tera = Arc::new(tera);
        let router = Router::new().with_state(tera.clone());

        Ok(Self {
            router,
            config,
            addr,
            tera,
        })
    }

    /// Loads internal Runique templates
    ///
    /// Embeds and loads built-in templates for error pages, CSRF tokens,
    /// flash messages, and other framework features.
    fn load_internal_templates(tera: &mut Tera) -> Result<(), Box<dyn Error>> {
        // Templates principales
        tera.add_raw_template(
            "base_index",
            include_str!("../templates/runique_index/base_index.html"),
        )?;
        tera.add_raw_template("message", include_str!("../templates/message/message.html"))?;
        tera.add_raw_template("404", include_str!("../templates/errors/404.html"))?;
        tera.add_raw_template("500", include_str!("../templates/errors/500.html"))?;
        tera.add_raw_template(
            "debug",
            include_str!("../templates/errors/debug_error.html"),
        )?;
        // Templates de sécurité
        tera.add_raw_template("csrf", include_str!("../templates/csrf/csrf.html"))?;
        tera.add_raw_template("csp", include_str!("../templates/csp/csp.html"))?;

        // Corps des pages d'erreurs détaillées
        const ERROR_CORPS: [(&str, &str); 8] = [
            (
                "errors/corps-error/header-error.html",
                include_str!("../templates/errors/corps-error/header-error.html"),
            ),
            (
                "errors/corps-error/message-error.html",
                include_str!("../templates/errors/corps-error/message-error.html"),
            ),
            (
                "errors/corps-error/template-info.html",
                include_str!("../templates/errors/corps-error/template-info.html"),
            ),
            (
                "errors/corps-error/stack-trace-error.html",
                include_str!("../templates/errors/corps-error/stack-trace-error.html"),
            ),
            (
                "errors/corps-error/request-info.html",
                include_str!("../templates/errors/corps-error/request-info.html"),
            ),
            (
                "errors/corps-error/environment-info.html",
                include_str!("../templates/errors/corps-error/environment-info.html"),
            ),
            (
                "errors/corps-error/status-code-info.html",
                include_str!("../templates/errors/corps-error/status-code-info.html"),
            ),
            (
                "errors/corps-error/footer-error.html",
                include_str!("../templates/errors/corps-error/footer-error.html"),
            ),
        ];

        for (name, content) in ERROR_CORPS {
            tera.add_raw_template(name, content)?;
        }

        // Champs de formulaire
        tera.add_raw_template(
            "checkbox",
            include_str!("../templates/formulaire/checkboxfield.html"),
        )?;
        tera.add_raw_template(
            "text",
            include_str!("../templates/formulaire/charfield.html"),
        )?;
        tera.add_raw_template(
            "number",
            include_str!("../templates/formulaire/numberfield.html"),
        )?;
        tera.add_raw_template(
            "date",
            include_str!("../templates/formulaire/datefield.html"),
        )?;
        tera.add_raw_template(
            "email",
            include_str!("../templates/formulaire/emailfield.html"),
        )?;
        tera.add_raw_template(
            "file",
            include_str!("../templates/formulaire/filefield.html"),
        )?;
        tera.add_raw_template(
            "json",
            include_str!("../templates/formulaire/jsonfield.html"),
        )?;
        tera.add_raw_template("url", include_str!("../templates/formulaire/urlfield.html"))?;
        tera.add_raw_template(
            "password",
            include_str!("../templates/formulaire/passwordfield.html"),
        )?;
        tera.add_raw_template(
            "slug",
            include_str!("../templates/formulaire/slugfield.html"),
        )?;
        tera.add_raw_template(
            "textarea",
            include_str!("../templates/formulaire/textarea.html"),
        )?;
        tera.add_raw_template(
            "select",
            include_str!("../templates/formulaire/selectfield.html"),
        )?;
        tera.add_raw_template(
            "hidden",
            include_str!("../templates/formulaire/hiddenfield.html"),
        )?;
        tera.add_raw_template(
            "datetime-local",
            include_str!("../templates/formulaire/datetime-local.html"),
        )?;
        tera.add_raw_template(
            "ipaddress",
            include_str!("../templates/formulaire/ipfield.html"),
        )?;

        // Nouveaux champs
        tera.add_raw_template("color", include_str!("../templates/formulaire/color.html"))?;
        tera.add_raw_template("time", include_str!("../templates/formulaire/time.html"))?;
        tera.add_raw_template("tel", include_str!("../templates/formulaire/telephon.html"))?;
        tera.add_raw_template("range", include_str!("../templates/formulaire/range.html"))?;
        tera.add_raw_template("radio", include_str!("../templates/formulaire/radio.html"))?;
        tera.add_raw_template(
            "select-multiple",
            include_str!("../templates/formulaire/select-multiple.html"),
        )?;
        tera.add_raw_template(
            "file-multiple",
            include_str!("../templates/formulaire/multiple-file.html"),
        )?;

        Ok(())
    }

    /// Adds routes to the application
    ///
    /// Merges the provided router with the application's existing routes.
    ///
    /// # Arguments
    ///
    /// * `routes` - Router containing application routes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings, Router, get};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// async fn index() -> &'static str { "Hello" }
    ///
    /// let settings = Settings::default_values();
    /// let routes = Router::new().route("/", get(index));
    ///
    /// RuniqueApp::new(settings)
    ///     .await?
    ///     .routes(routes)
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn routes(mut self, routes: Router<Arc<Tera>>) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    /// Adds database connection to the application
    ///
    /// Makes the database connection available as a shared extension
    /// throughout the application via `Extension<Arc<DatabaseConnection>>`.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings, DatabaseConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::default_values();
    /// let db_config = DatabaseConfig::from_env()?.build();
    /// let db = db_config.connect().await?;
    ///
    /// RuniqueApp::new(settings)
    ///     .await?
    ///     .with_database(db)
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        let shared_db = Arc::new(db);
        self.router = self.router.layer(Extension(shared_db));
        self
    }

    /// Configures static and media file serving
    ///
    /// Sets up routes to serve static assets, media uploads, and Runique's
    /// internal assets based on the paths configured in settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let settings = Settings::builder()
    ///     .static_url("/static")
    ///     .media_url("/media")
    ///     .build();
    ///
    /// RuniqueApp::new(settings)
    ///     .await?
    ///     .with_static_files()?
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if static file directories are inaccessible
    pub fn with_static_files(mut self) -> Result<Self, Box<dyn Error>> {
        let conf = self.config.as_ref();
        self.router = self
            .router
            .nest_service(&conf.static_url, ServeDir::new(&conf.staticfiles_dirs))
            .nest_service(&conf.media_url, ServeDir::new(&conf.media_root));

        if !conf.static_runique_path.is_empty() {
            self.router = self.router.nest_service(
                &conf.static_runique_url,
                ServeDir::new(&conf.static_runique_path),
            );
        }
        Ok(self)
    }

    /// Adds default middleware stack
    ///
    /// Configures the following middleware:
    /// - Error handler with custom error pages
    /// - Flash message support
    /// - CSRF token generation and validation
    /// - Request tracing
    /// - Request timeout (10 seconds)
    /// - 404 handler
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_default_middleware()
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_default_middleware(mut self) -> Self {
        let tera = self.tera.clone();
        let config = self.config.clone();

        self.router = self
            .router
            .fallback(move |uri: axum::http::Uri| {
                let t = tera.clone();
                let c = config.clone();
                async move {
                    if uri.path() == "/" {
                        return render_index(&t, &Context::new(), &c);
                    }
                    render_404(&t)
                }
            })
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(TimeoutLayer::with_status_code(
                        StatusCode::REQUEST_TIMEOUT,
                        std::time::Duration::from_secs(10),
                    )),
            )
            .layer(middleware::from_fn(csrf_middleware))
            .layer(middleware::from_fn(flash_middleware))
            .layer(middleware::from_fn(error_handler_middleware));
        self
    }

    /// Builds and returns the final Router
    ///
    /// Consumes the `RuniqueApp` and returns the configured Axum router.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .build();
    ///
    /// // Use router with custom server configuration
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Router<Arc<Tera>> {
        self.router
    }

    /// Adds flash message middleware
    ///
    /// Enables flash messages for one-time notifications across requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_flash_messages()
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_flash_messages(mut self) -> Self {
        self.router = self.router.layer(middleware::from_fn(flash_middleware));
        self
    }
    /// Crée un nouveau builder d'application (méthode statique).
    /// Permet d'utiliser la syntaxe RuniqueApp::builder(settings) dans les tests et l'API.
    pub async fn builder(settings: Settings) -> RuniqueAppBuilder {
        crate::app::builder(settings).await
    }
    /// Adds CSRF protection middleware
    ///
    /// Generates and validates CSRF tokens for forms.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_csrf_tokens()
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_csrf_tokens(mut self) -> Self {
        self.router = self.router.layer(middleware::from_fn(csrf_middleware));
        self
    }

    /// Enables automatic text input sanitization
    ///
    /// When enabled, automatically sanitizes user input to prevent XSS attacks.
    ///
    /// # Arguments
    ///
    /// * `enable` - Whether to enable sanitization
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_sanitize_text_inputs(true)
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_sanitize_text_inputs(mut self, enable: bool) -> Self {
        if !enable {
            return self;
        }
        let config = self.config.clone();
        self.router = self
            .router
            .layer(middleware::from_fn_with_state(config, sanitize_middleware));
        self
    }

    /// Enables allowed hosts validation (ALLOWED_HOSTS)
    ///
    /// Validates incoming requests against a whitelist of allowed host headers.
    /// This is a security feature to prevent Host header attacks.
    ///
    /// # Arguments
    ///
    /// * `hosts` - Optional list of allowed hosts. If None, uses hosts from settings.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Use hosts from settings
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_allowed_hosts(None)
    ///     .run()
    ///     .await?;
    ///
    /// // Or provide custom hosts
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_allowed_hosts(Some(vec!["example.com".to_string()]))
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_allowed_hosts(mut self, hosts: Option<Vec<String>>) -> Self {
        let allowed_hosts = hosts.unwrap_or_else(|| self.config.allowed_hosts.clone());

        let mut config = (*self.config).clone();
        config.allowed_hosts = allowed_hosts;
        self.config = Arc::new(config);

        self.router = self.router.layer(middleware::from_fn(
            crate::middleware::allowed_hosts::allowed_hosts_middleware,
        ));
        self
    }

    /// Enables Content Security Policy
    ///
    /// Adds CSP headers to protect against XSS and injection attacks.
    ///
    /// # Arguments
    ///
    /// * `config` - CSP configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings, CspConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_csp(CspConfig::strict())
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_csp(self, config: CspConfig) -> Self {
        let router = self.router.layer(middleware::from_fn_with_state(
            config,
            crate::middleware::csp::csp_middleware,
        ));

        Self {
            router,
            config: self.config,
            addr: self.addr,
            tera: self.tera,
        }
    }

    /// Enables all security headers
    ///
    /// Adds comprehensive security headers including CSP, X-Frame-Options,
    /// X-Content-Type-Options, and more.
    ///
    /// # Arguments
    ///
    /// * `config` - CSP configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings, CspConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_security_headers(CspConfig::strict())
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_security_headers(self, config: CspConfig) -> Self {
        let router = self.router.layer(middleware::from_fn_with_state(
            config,
            security_headers_middleware,
        ));

        Self {
            router,
            config: self.config,
            addr: self.addr,
            tera: self.tera,
        }
    }

    /// Enables CSP in report-only mode
    ///
    /// Adds CSP headers in report-only mode for testing without blocking content.
    ///
    /// # Arguments
    ///
    /// * `config` - CSP configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings, CspConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_csp_report_only(CspConfig::strict())
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_csp_report_only(self, config: CspConfig) -> Self {
        let router = self.router.layer(middleware::from_fn_with_state(
            config,
            crate::middleware::csp::csp_report_only_middleware,
        ));

        Self {
            router,
            config: self.config,
            addr: self.addr,
            tera: self.tera,
        }
    }

    /// Runs the application server
    ///
    /// Starts the HTTP server and listens for incoming requests.
    /// Blocks until the server is shut down (via Ctrl+C).
    ///
    /// # Session Management
    ///
    /// Automatically configures session management with:
    /// - In-memory session store
    /// - Secure cookies in production
    /// - 24-hour session expiry
    ///
    /// # Graceful Shutdown
    ///
    /// The server listens for SIGINT (Ctrl+C) and shuts down gracefully,
    /// allowing in-flight requests to complete.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use runique::{RuniqueApp, Settings};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// RuniqueApp::new(Settings::default_values())
    ///     .await?
    ///     .with_default_middleware()
    ///     .run()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Unable to bind to the configured address/port
    /// - Server encounters a fatal error
    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        println!("🦀 Runique Framework v {}", crate::VERSION);
        println!("   Starting server at http://{}", self.addr);

        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(!self.config.debug)
            .with_http_only(!self.config.debug)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(86400)));

        // Clonage de l'état Tera pour l'injection
        let state = self.tera.clone();

        // On construit le router final
        // .with_state() est déjà appliqué au début, mais on s'assure que
        // les extensions globales sont ajoutées ici.
        let router = self
            .router
            .layer(Extension(self.config.clone()))
            .layer(Extension(self.tera.clone())) // Pour compatibilité ancienne
            .layer(session_layer)
            .with_state(state);

        let listener = TcpListener::bind(&self.addr).await?;

        // Axum::serve attend un Router<()> ou une méthode qui gère l'état.
        // Puisque nous avons déjà injecté l'état via with_state(),
        // le Router est compatible.
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
                println!("\nShutdown signal received. Stopping server...");
            })
            .await?;

        Ok(())
    }
}

/// Creates a new application builder
///
/// Alternative API using the builder pattern for application configuration.
///
/// # Arguments
///
/// * `settings` - Application settings
///
/// # Examples
///
/// ```no_run
/// use runique::{RuniqueApp, Settings, Router, get};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// async fn index() -> &'static str { "Hello" }
///
/// let settings = Settings::default_values();
/// let routes = Router::new().route("/", get(index));
///
/// let app = RuniqueApp::builder(settings)
///     .await
///     .routes(routes)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub async fn builder(settings: Settings) -> RuniqueAppBuilder {
    RuniqueAppBuilder {
        settings,
        routes: None,
    }
}

/// Builder for RuniqueApp
///
/// Provides an alternative API for building Runique applications
/// using the builder pattern.
///
/// # Examples
///
/// ```no_run
/// use runique::{RuniqueApp, Settings};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let app = RuniqueApp::builder(Settings::default_values())
///     .await
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct RuniqueAppBuilder {
    settings: Settings,
    routes: Option<Router<Arc<Tera>>>,
}

impl RuniqueAppBuilder {
    /// Adds routes to the builder
    ///
    /// # Arguments
    ///
    /// * `routes` - Router containing application routes
    pub fn routes(mut self, routes: Router<Arc<Tera>>) -> Self {
        self.routes = Some(routes);
        self
    }

    /// Builds the final RuniqueApp
    ///
    /// # Errors
    ///
    /// Returns an error if application initialization fails
    pub async fn build(self) -> Result<RuniqueApp, Box<dyn Error>> {
        let mut app = RuniqueApp::new(self.settings).await?;
        if let Some(routes) = self.routes {
            app = app.routes(routes);
        }
        Ok(app)
    }
}
