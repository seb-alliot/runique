use crate::context::error::ErrorContext;
/// Contexte centralisé pour un handler Axum / template Tera
/// Contient :
/// - Engine (config, Tera, etc.)
/// - Flash messages
/// - Token CSRF
/// - Nonce CSP
/// - Helpers pour rendre les templates et injecter dynamiquement des variables
use crate::engine::RuniqueEngine;
use crate::flash::{FlashMessage, Message};
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

pub struct AppError {
    context: ErrorContext,
}

/// Type alias pour les résultats avec AppError boxé (optimisation Clippy)
pub type AppResult<T> = Result<T, Box<AppError>>;

impl AppError {
    /// Créer depuis une erreur Tera
    pub fn from_tera(error: tera::Error, template_name: &str, tera: &tera::Tera) -> Self {
        Self {
            context: ErrorContext::from_tera_error(&error, template_name, tera),
        }
    }

    /// Créer depuis anyhow
    pub fn from_anyhow(error: anyhow::Error) -> Self {
        Self {
            context: ErrorContext::from_anyhow(&error),
        }
    }

    /// Créer depuis une erreur de base de données
    pub fn from_db(error: DbErr) -> Self {
        Self {
            context: ErrorContext::database(error),
        }
    }

    /// Créer une erreur interne générique
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            context: ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, &message.into()),
        }
    }

    /// Créer une erreur de validation
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            context: ErrorContext::new(
                crate::context::error::ErrorType::Validation,
                StatusCode::BAD_REQUEST,
                "Validation Error",
                &message.into(),
            ),
        }
    }

    /// Créer une 404
    pub fn not_found(path: &str) -> Self {
        Self {
            context: ErrorContext::not_found(path),
        }
    }
}

// Conversion automatique depuis anyhow::Error
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::from_anyhow(error)
    }
}

// Conversion automatique depuis DbErr (SeaORM)
impl From<DbErr> for AppError {
    fn from(error: DbErr) -> Self {
        Self::from_db(error)
    }
}

// Conversion automatique depuis anyhow::Error vers Box<AppError>
impl From<anyhow::Error> for Box<AppError> {
    fn from(error: anyhow::Error) -> Self {
        Box::new(AppError::from_anyhow(error))
    }
}

// Conversion automatique depuis DbErr vers Box<AppError>
impl From<DbErr> for Box<AppError> {
    fn from(error: DbErr) -> Self {
        Box::new(AppError::from_db(error))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.context.status_code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let mut res = status.into_response();
        // Injecte le ErrorContext complet dans les extensions
        res.extensions_mut().insert(Arc::new(self.context));
        res
    }
}

impl IntoResponse for Box<AppError> {
    fn into_response(self) -> Response {
        (*self).into_response()
    }
}

pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub session: Session,
    pub notices: Message,
    pub messages: Vec<FlashMessage>,
    pub csrf_token: CsrfToken,
    pub csp_nonce: String,
    pub context: Context,
}

impl<S> FromRequestParts<S> for TemplateContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        //  Récupération de l'Engine depuis les extensions
        let engine = parts
            .extensions
            .get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        //  Récupération du token CSRF depuis les extensions
        let csrf_token: CsrfToken = parts
            .extensions
            .get::<CsrfToken>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        //  Récupération du nonce CSP depuis les extensions
        let csp_nonce: String = parts
            .extensions
            .get::<CspNonce>()
            .map(|n| n.as_str().to_string())
            .unwrap_or_default();

        //  Récupération de la session et création du FlashManager
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let notices = Message {
            session: session.clone(),
        };

        // Récupère les messages pour le template
        let messages: Vec<FlashMessage> = notices.get_all().await;

        //  Initialiser le contexto Tera com as variáveis globais
        let mut context = Context::new();
        context.insert("debug", &engine.config.debug);
        context.insert("csrf_token", &csrf_token.masked().as_str());
        context.insert("csp_nonce", &csp_nonce);
        context.insert("static_runique", &engine.config.static_files);
        context.insert("messages", &messages);

        Ok(Self {
            engine: engine.clone(),
            session,
            notices,
            messages,
            csrf_token,
            csp_nonce,
            context,
        })
    }
}

impl Clone for TemplateContext {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            session: self.session.clone(),
            notices: self.notices.clone(),
            messages: self.messages.clone(),
            csrf_token: self.csrf_token.clone(),
            csp_nonce: self.csp_nonce.clone(),
            context: self.context.clone(),
        }
    }
}

impl TemplateContext {
    /// Rendu d'un template Tera avec capture complète des erreurs
    pub fn render(&self, template_route: &str) -> AppResult<Response> {
        match self.engine.tera.render(template_route, &self.context) {
            Ok(html) => Ok(Html(html).into_response()),
            Err(e) => {
                // Utilise le builder dédié qui capture toutes les infos Tera
                Err(Box::new(AppError::from_tera(
                    e,
                    template_route,
                    &self.engine.tera,
                )))
            }
        }
    }

    /// Helper pour insérer des données dans le contexte (renommé pour être plus idiomatique)
    pub fn insert(&mut self, key: &str, value: &impl serde::Serialize) -> &mut Self {
        self.context.insert(key, value);
        self
    }

    /// Helper chainable pour plusieurs insertions
    pub fn with_data(mut self, data: Vec<(&str, serde_json::Value)>) -> Self {
        for (key, value) in data {
            self.context.insert(key, &value);
        }
        self
    }

    /// Rendu direct avec les données injectées
    pub fn render_with(
        &mut self,
        template_route: &str,
        data: Vec<(&str, serde_json::Value)>,
    ) -> AppResult<Response> {
        for (key, value) in data {
            self.context.insert(key, &value);
        }
        self.render(template_route)
    }

    /// Crée un formulaire vide avec le token CSRF et le moteur Tera déjà injectés
    pub fn form<T: crate::forms::field::RuniqueForm>(&self) -> T {
        T::build(self.engine.tera.clone(), self.csrf_token.as_str())
    }
}
