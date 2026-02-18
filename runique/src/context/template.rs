use crate::app::templates::TemplateLoader;
use crate::errors::error::ErrorContext;
use crate::flash::Message;
use crate::impl_from_error;
use crate::utils::aliases::{AEngine, AppResult};
use crate::utils::{csp_nonce::CspNonce, csrf::CsrfToken};
use axum::{
    extract::FromRequestParts,
    http::{method::Method, request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
};
use sea_orm::DbErr;
use std::sync::Arc;
use tera::Context;
use tower_sessions::Session;
use tracing::error;

// --- ERROR HANDLING ---

pub struct AppError {
    pub context: ErrorContext,
}

impl AppError {
    pub fn new(context: ErrorContext) -> Self {
        Self { context }
    }

    // Generic helper to map known errors
    pub fn map_tera(e: tera::Error, route: &str, tera: &tera::Tera) -> Box<Self> {
        // Log the detailed error in the console
        error!(
            template = route,
            error = ?e,
            "Template rendering error"
        );

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

        //  Log the error in detail
        error!(
            status = status.as_u16(),
            error_type = ?self.context.error_type,
            message = %self.context.message,
            "AppError occurred"
        );

        let mut res = status.into_response();
        //  Insert ErrorContext so that the middleware can retrieve it
        res.extensions_mut().insert(Arc::new(self.context));
        res
    }
}

impl IntoResponse for Box<AppError> {
    fn into_response(self) -> Response {
        (*self).into_response()
    }
}

// --- TEMPLATE CONTEXT ---

#[derive(Clone)]
pub struct Request {
    pub engine: AEngine,
    pub session: Session,
    pub notices: Message,
    pub csrf_token: CsrfToken,
    pub context: Context,
    pub method: Method,
}

impl<S> FromRequestParts<S> for Request
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ex = &parts.extensions;

        // Fast extraction
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
        // Inject current_user if available
        if let Some(current_user) = ex.get::<crate::middleware::auth::CurrentUser>() {
            context.insert("current_user", current_user);
        }
        Ok(Self {
            engine,
            session,
            notices,
            csrf_token,
            context,
            method: parts.method.clone(),
        })
    }
}

impl Request {
    pub fn new(engine: AEngine, session: Session, csrf_token: CsrfToken, method: Method) -> Self {
        let mut context = tera::Context::new();
        // mod reload for templates in debug mode
        // The backend cannot be reloaded here because it is shared between requests
        context.insert("debug", &engine.config.debug);
        context.insert("static_runique", &engine.config.static_files.static_url);
        context.insert("csrf_token", &csrf_token.masked().as_str());

        Self {
            engine,
            session: session.clone(),
            notices: Message { session },
            csrf_token,
            context,
            method,
        }
    }

    pub fn is_get(&self) -> bool {
        self.method == Method::GET
    }

    pub fn is_post(&self) -> bool {
        self.method == Method::POST
    }

    pub fn is_put(&self) -> bool {
        self.method == Method::PUT
    }

    pub fn is_delete(&self) -> bool {
        self.method == Method::DELETE
    }
    /// Unique generic rendering to avoid duplication
    pub fn render(&mut self, template: &str) -> AppResult<Response> {
        let html_result = if self.engine.config.debug {
            // In debug mode, Tera is fully reinitialized with the Loader
            // This applies Regex on {% messages %}, {% form.xxx %}, etc.
            match TemplateLoader::init(&self.engine.config, self.engine.url_registry.clone()) {
                Ok(mut dev_tera) => {
                    dev_tera.autoescape_on(vec!["html", "xml"]);

                    let res = dev_tera.render(template, &self.context);
                    if let Err(ref e) = res {
                        // Detailed log of the Tera error with all sources
                        error!(
                            template = template,
                            error_kind = ?e.kind,
                            error_message = %e,
                            "Tera rendering failed in debug mode"
                        );

                        // Log the full error chain (source)
                        // Uses the source() method from the std::error::Error trait
                        use std::error::Error as StdError;
                        if let Some(source) = e.source() {
                            error!(
                                source_error = %source,
                                "Tera error source"
                            );
                        }
                    }
                    res
                }
                Err(e) => {
                    error!(
                        template = template,
                        error = %e,
                        "Failed to initialize TemplateLoader in debug mode"
                    );
                    return Err(AppError::map_tera(
                        tera::Error::msg(e.to_string()),
                        template,
                        &self.engine.tera,
                    ));
                }
            }
        } else {
            // Production mode: uses the already transformed instance
            self.engine.tera.render(template, &self.context)
        };

        html_result
            .map(|html| Html(html).into_response())
            .map_err(|e| AppError::map_tera(e, template, &self.engine.tera))
    }

    /// Fluent insertion with builder pattern
    pub fn insert(mut self, key: &str, value: impl serde::Serialize) -> Self {
        self.context.insert(key, &value);
        self
    }

    /// Immediate rendering with additional data
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
