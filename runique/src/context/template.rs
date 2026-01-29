use crate::app::templates::TemplateLoader;
use crate::context::error::ErrorContext;
use crate::flash::Message;
use crate::impl_from_error;
use crate::utils::aliases::{AEngine, AppResult};
use crate::utils::{csp_nonce::CspNonce, csrf::CsrfToken};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
};
use sea_orm::DbErr;
use std::sync::Arc;
use tera::Context;
use tower_sessions::Session;
// --- GESTION DES ERREURS ---

pub struct AppError {
    pub context: ErrorContext,
}

impl AppError {
    pub fn new(context: ErrorContext) -> Self {
        Self { context }
    }

    // Helper générique pour mapper les erreurs connues
    pub fn map_tera(e: tera::Error, route: &str, tera: &tera::Tera) -> Box<Self> {
        Box::new(Self {
            context: ErrorContext::from_tera_error(&e, route, tera),
        })
    }
}

impl_from_error!(anyhow::Error => from_anyhow, DbErr => database);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.context.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut res = status.into_response();
        res.extensions_mut().insert(Arc::new(self.context));
        res
    }
}

impl IntoResponse for Box<AppError> {
    fn into_response(self) -> Response {
        (*self).into_response()
    }
}

// --- CONTEXTE DE TEMPLATE ---

#[derive(Clone)]
pub struct TemplateContext {
    pub engine: AEngine,
    pub session: Session,
    pub notices: Message,
    pub csrf_token: CsrfToken,
    pub context: Context,
}

impl<S> FromRequestParts<S> for TemplateContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ex = &parts.extensions;

        // Extraction rapide
        let engine = ex
            .get::<AEngine>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let csrf_token = ex
            .get::<CsrfToken>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let session = ex
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let nonce = ex.get::<CspNonce>().map(|n| n.as_str()).unwrap_or_default();

        let notices = Message {
            session: session.clone(),
        };
        let messages = notices.get_all().await;

        let mut context = Context::new();
        context.insert("debug", &engine.config.debug);
        context.insert("csrf_token", &csrf_token.masked().as_str());
        context.insert("csp_nonce", nonce);
        context.insert("static_runique", &engine.config.static_files);
        context.insert("messages", &messages);

        Ok(Self {
            engine,
            session,
            notices,
            csrf_token,
            context,
        })
    }
}

impl TemplateContext {
    pub fn new(engine: AEngine, session: Session, csrf_token: CsrfToken) -> Self {
        let mut context = tera::Context::new();
        // mod reload pour les templates en debug
        // Le backend ne peux être reloadé ici car il est partagé entre les requêtes
        context.insert("debug", &engine.config.debug);
        context.insert("static_runique", &engine.config.static_files.static_url);
        context.insert("csrf_token", &csrf_token.masked().as_str());

        Self {
            engine,
            session: session.clone(),
            notices: Message { session },
            csrf_token,
            context,
        }
    }
    /// Rendu générique unique pour éviter la duplication
    pub fn render(&mut self, template: &str) -> AppResult<Response> {
        let html_result = if self.engine.config.debug {
            // En mode debug, on réinitialise Tera complètement via ton Loader
            // Cela applique les Regex sur {% messages %}, {% form.xxx %}, etc.
            match TemplateLoader::init(&self.engine.config, self.engine.url_registry.clone()) {
                Ok(mut dev_tera) => {
                    dev_tera.autoescape_on(vec!["html", "xml"]);

                    let res = dev_tera.render(template, &self.context);
                    if let Err(ref e) = res {
                        println!("[Erreur rendu Runique (Debug)]: {:?}", e);
                    }
                    res
                }
                Err(e) => {
                    println!("[Erreur Initialisation Loader (Debug)]: {:?}", e);
                    return Err(AppError::map_tera(
                        tera::Error::msg(e.to_string()),
                        template,
                        &self.engine.tera,
                    ));
                }
            }
        } else {
            // Mode Production : utilise l'instance déjà transformée
            self.engine.tera.render(template, &self.context)
        };

        html_result
            .map(|html| Html(html).into_response())
            .map_err(|e| AppError::map_tera(e, template, &self.engine.tera))
    }

    /// Insertion fluide avec pattern builder
    pub fn insert(mut self, key: &str, value: impl serde::Serialize) -> Self {
        self.context.insert(key, &value);
        self
    }

    /// Rendu immédiat avec données additionnelles
    pub fn render_with(
        mut self,
        template: &str,
        data: Vec<(&str, serde_json::Value)>,
    ) -> AppResult<Response> {
        for (k, v) in data {
            self.context.insert(k, &v);
        }
        self.render(template)
    }

    pub fn form<T: crate::forms::field::RuniqueForm>(&self) -> T {
        T::build(self.engine.tera.clone(), self.csrf_token.as_str())
    }
}
