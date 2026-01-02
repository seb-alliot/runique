// rusti/src/app.rs
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

pub struct RustiApp {
    router: Router,
    config: Arc<Settings>,
    addr: SocketAddr,
    tera: Arc<Tera>,
}

impl RustiApp {
    pub async fn new(settings: Settings) -> Result<Self, Box<dyn Error>> {
        let config = Arc::new(settings);
        let addr = config.server.domain_server.parse()?;

        let mut tera = Tera::default();
        // 1. Templates internes
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
        for dir_string in &config.templates_dir {
            let template_dir = std::path::Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let mut content = std::fs::read_to_string(&entry)?;

                    content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
                    content = content.replace("{% messages %}", r#"{% include "message" %}"#);
                    content = content.replace("{{ csp }}", r#"{% include "csp" %}"#);

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
            config.static_rusti_url.clone(),
            config.media_rusti_url.clone(),
        );

        crate::tera_function::url_balise::register_url(&mut tera);

        let tera = Arc::new(tera);
        let router = Router::new();

        Ok(Self {
            router,
            config,
            addr,
            tera,
        })
    }

    fn load_internal_templates(tera: &mut Tera) -> Result<(), Box<dyn Error>> {
        tera.add_raw_template("base_index", include_str!("../templates/base_index.html"))?;
        tera.add_raw_template("message", include_str!("../templates/message.html"))?;
        tera.add_raw_template("404", include_str!("../templates/errors/404.html"))?;
        tera.add_raw_template("500", include_str!("../templates/errors/500.html"))?;
        tera.add_raw_template(
            "debug",
            include_str!("../templates/errors/debug_error.html"),
        )?;
        tera.add_raw_template("csrf", include_str!("../templates/csrf/csrf.html"))?;
        tera.add_raw_template("csp", include_str!("../templates/csp/csp.html"))?;

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
        Ok(())
    }

    pub fn routes(mut self, routes: Router) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    pub fn with_database(mut self, db: DatabaseConnection) -> Self {
        let shared_db = Arc::new(db);
        self.router = self.router.layer(Extension(shared_db));
        self
    }

    pub fn with_static_files(mut self) -> Result<Self, Box<dyn Error>> {
        let conf = self.config.as_ref();
        self.router = self
            .router
            .nest_service(&conf.static_url, ServeDir::new(&conf.staticfiles_dirs))
            .nest_service(&conf.media_url, ServeDir::new(&conf.media_root));

        if !conf.static_rusti_path.is_empty() {
            self.router = self.router.nest_service(
                &conf.static_rusti_url,
                ServeDir::new(&conf.static_rusti_path),
            );
        }
        Ok(self)
    }

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

    pub fn build(self) -> Router {
        self.router
    }

    pub fn with_flash_messages(mut self) -> Self {
        self.router = self.router.layer(middleware::from_fn(flash_middleware));
        self
    }

    pub fn with_csrf_tokens(mut self) -> Self {
        self.router = self.router.layer(middleware::from_fn(csrf_middleware));
        self
    }

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

    /// Active la validation des hosts autorisés (ALLOWED_HOSTS)
    ///
    /// # Exemple
    /// ```no_run
    /// # use rusti::app::RustiApp;
    /// # async fn test(settings: rusti::Settings) -> Result<(), Box<dyn std::error::Error>> {
    /// RustiApp::new(settings).await?
    ///     .with_allowed_hosts(None)
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

    /// Active la Content Security Policy
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

    /// Active tous les en-têtes de sécurité
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

    /// Active la CSP en mode report-only
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

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        println!("🦀 Rusti Framework v{}", crate::VERSION);
        println!("   Starting server at http://{}", self.addr);

        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(!self.config.debug)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(86400)));

        let router = self
            .router
            .layer(Extension(self.config.clone()))
            .layer(Extension(self.tera.clone()))
            .layer(session_layer);

        let listener = TcpListener::bind(&self.addr).await?;
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
                println!("\nShutdown signal received. Stopping server...");
            })
            .await?;

        Ok(())
    }

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
