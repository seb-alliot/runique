// rusti/src/app.rs
use std::sync::Arc;
use std::net::SocketAddr;
use std::error::Error;
use axum::{Router, middleware, Extension};
use axum::http::StatusCode;
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
use tera::{Tera, Context};
use anyhow::Result;
use glob;

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

use crate::settings::Settings;
use crate::middleware::error_handler::{error_handler_middleware, render_index};
use crate::middleware::flash_message::flash_middleware;
use crate::middleware::csrf::csrf_middleware;
use crate::response::render_simple_404;

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
        let re_tag_with_link = regex::Regex::new(
            r#"\{%\s*(?P<tag>static|media)\s*['"](?P<link>[^'"]+)['"]\s*%}"#
        ).unwrap();

        for dir_string in &config.templates_dir {
            let template_dir = std::path::Path::new(dir_string);
            let pattern = format!("{}/**/*.html", template_dir.display());

            if let Ok(paths) = glob::glob(&pattern) {
                for entry in paths.flatten() {
                    let mut content = std::fs::read_to_string(&entry)?;

                    content = content.replace("{% csrf %}", r#"{% include "csrf" %}"#);
                    content = content.replace("{% messages %}", r#"{% include "message" %}"#);

                    // Transformation {% link "name" %}
                    let re_link_simple = regex::Regex::new(
                        r#"\{%\s*link\s+['"](?P<name>[^'"]+)['"]\s*%}"#
                    ).unwrap();

                    let re_link_params = regex::Regex::new(
                        r#"\{%\s*link\s+['"](?P<name>[^'"]+)['"],\s*(?P<params>[^%]+)%}"#
                    ).unwrap();

                    // Appliquer les transformations
                    content = re_link_simple.replace_all(&content, |caps: &regex::Captures| {
                        let name = &caps["name"];
                        format!(r#"{{{{ link(link='{}') }}}}"#, name)
                    }).to_string();

                    content = re_link_params.replace_all(&content, |caps: &regex::Captures| {
                        let name = &caps["name"];
                        let params = &caps["params"].trim();
                        format!(r#"{{{{ link(link='{}', {}) }}}}"#, name, params)
                    }).to_string();
                    content = re_tag_with_link.replace_all(&content, |caps: &regex::Captures| {
                        let tag = &caps["tag"];
                        let link = &caps["link"];
                        format!(r#"{{{{ "{}" | {} }}}}"#, link, tag)
                    }).to_string();
                    // Dans RustiApp::new(), après les transformations existantes

                    // Transformation {% link "name" %}
                    let re_link_simple = regex::Regex::new(
                        r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s*%}"#
                    ).unwrap();

                    let re_link_with_params = regex::Regex::new(
                        r#"\{%\s*link\s*['"](?P<name>[^'"]+)['"]\s+(?P<params>[^%]+)%}"#
                    ).unwrap();

                    // Transformer {% link "about" %} → {{ link(link='about') }}
                    content = re_link_simple.replace_all(&content, |caps: &regex::Captures| {
                        let name = &caps["name"];
                        format!(r#"{{{{ link(link='{}') }}}}"#, name)
                    }).to_string();

                    // Transformer {% link "user_profile" id=66 name='sebastien' %}
                    // → {{ link(link='user_profile', id=66, name='sebastien') }}
                    content = re_link_with_params.replace_all(&content, |caps: &regex::Captures| {
                        let name = &caps["name"];
                        let params = &caps["params"];
                        format!(r#"{{{{ link(link='{}', {}) }}}}"#, name, params.trim())
                    }).to_string();

                    let name = entry.strip_prefix(template_dir)?
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

        Ok(Self { router, config, addr, tera })
    }

    fn load_internal_templates(tera: &mut Tera) -> Result<(), Box<dyn Error>> {
        tera.add_raw_template("base_index.html", include_str!("../templates/base_index.html"))?;
        tera.add_raw_template("message", include_str!("../templates/message.html"))?;
        tera.add_raw_template("errors/404.html", include_str!("../templates/errors/404.html"))?;
        tera.add_raw_template("errors/500.html", include_str!("../templates/errors/500.html"))?;
        tera.add_raw_template("errors/debug_error.html", include_str!("../templates/errors/debug_error.html"))?;
        tera.add_raw_template("csrf", include_str!("../templates/csrf/csrf.html"))?;

        const ERROR_CORPS: [(&str, &str); 8] = [
            ("errors/corps-error/header-error.html", include_str!("../templates/errors/corps-error/header-error.html")),
            ("errors/corps-error/message-error.html", include_str!("../templates/errors/corps-error/message-error.html")),
            ("errors/corps-error/template-info.html", include_str!("../templates/errors/corps-error/template-info.html")),
            ("errors/corps-error/stack-trace-error.html", include_str!("../templates/errors/corps-error/stack-trace-error.html")),
            ("errors/corps-error/request-info.html", include_str!("../templates/errors/corps-error/request-info.html")),
            ("errors/corps-error/environment-info.html", include_str!("../templates/errors/corps-error/environment-info.html")),
            ("errors/corps-error/status-code-info.html", include_str!("../templates/errors/corps-error/status-code-info.html")),
            ("errors/corps-error/footer-error.html", include_str!("../templates/errors/corps-error/footer-error.html")),
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

    #[cfg(feature = "orm")]
    pub fn with_database_custom(mut self, db: DatabaseConnection) -> Self {
        self.router = self.router.layer(Extension(Arc::new(db)));
        self
    }

    pub fn with_static_files(mut self) -> Result<Self, Box<dyn Error>> {
        let conf = self.config.as_ref();
        self.router = self.router
            .nest_service(&conf.static_url, ServeDir::new(&conf.staticfiles_dirs))
            .nest_service(&conf.media_url, ServeDir::new(&conf.media_root));

        if !conf.static_rusti_path.is_empty() {
            self.router = self.router.nest_service(&conf.static_rusti_url, ServeDir::new(&conf.static_rusti_path));
        }
        Ok(self)
    }

    pub fn with_default_middleware(mut self) -> Self {
        let tera = self.tera.clone();
        let config = self.config.clone();

        self.router = self.router
            .fallback(move |uri: axum::http::Uri| {
                let t = tera.clone();
                let c = config.clone();
                async move {
                    if uri.path() == "/" {
                        return render_index(&t, &Context::new(), &c);
                    }
                    render_simple_404(&t)
                }
            })
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(TimeoutLayer::with_status_code(
                        StatusCode::REQUEST_TIMEOUT,
                        std::time::Duration::from_secs(10)
                    ))
            )
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

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        println!("🦀 Rusti Framework v{}", crate::VERSION);
        println!("   Starting server at http://{}", self.addr);

        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(!self.config.debug)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(86400)));

        let router = self.router
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