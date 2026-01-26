use crate::context::error::ErrorContext;
use crate::engine::RuniqueEngine;
use crate::flash::Message;
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

pub type AppResult<T> = Result<T, Box<AppError>>;

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

// Factorisation des conversions avec une macro interne simple
macro_rules! impl_from_error {
    ($($err:ty => $method:ident),*) => {
        $(
            impl From<$err> for AppError {
                fn from(err: $err) -> Self { Self { context: ErrorContext::$method(&err) } }
            }
            impl From<$err> for Box<AppError> {
                fn from(err: $err) -> Self { Box::new(AppError::from(err)) }
            }
        )*
    };
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
    pub engine: Arc<RuniqueEngine>,
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
            .get::<Arc<RuniqueEngine>>()
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
    /// Rendu générique unique pour éviter la duplication
    pub fn render(&mut self, template: &str) -> AppResult<Response> {
        self.engine
            .tera
            .render(template, &self.context)
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
