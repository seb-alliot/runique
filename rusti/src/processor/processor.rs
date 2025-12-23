// rusti/src/processor/message_processor.rs

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{Response},
    http::StatusCode,
};
use std::sync::Arc;
use tera::{Tera, Context};
use crate::middleware::flash_message::FlashMessage;
use crate::middleware::flash_message::FlashMessageSession;
use crate::settings::Settings;
pub struct Template {
    tera: Arc<Tera>,
    config: Arc<Settings>,
    context: Context,
}

// ... tes imports existants ...
use crate::middleware::csrf::CsrfToken; // Assure-toi d'importer ta struct CsrfToken

impl<S> FromRequestParts<S> for Template
where
    S: Send + Sync {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // 1. Récupération des extensions indispensables (Tera et Config)
        let tera = parts.extensions.get::<Arc<Tera>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.clone();
        let config = parts.extensions.get::<Arc<Settings>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.clone();

        let mut context = Context::new();


        // 2. Injection du CSRF Token
        if let Some(token) = parts.extensions.get::<CsrfToken>() {
            context.insert("csrf_token", &token.0);
        }

        // 3. Injection des Messages Flash
        if let Some(messages) = parts.extensions.get::<Vec<FlashMessage>>() {
            context.insert("messages", messages);
        }

        // 4. Injection du flag Debug
        context.insert("debug", &config.debug);

        Ok(Template { tera, config, context })
    }
}
impl Template {
    /// Render avec StatusCode::OK par défaut
    pub fn render(self, template_name: &str, user_context: &Context) -> Response {
        self.render_with_status(template_name, user_context, StatusCode::OK)
    }

    /// Render avec StatusCode custom
    pub fn render_with_status(
        mut self,
        template_name: &str,
        user_context: &Context,
        status: StatusCode,
    ) -> Response {
        // Merge user context avec le context auto-injecté
        self.context.extend(user_context.clone());

        // Utilise la fonction render_template existante pour la gestion d'erreur !
        crate::middleware::error_handler::render_template(
            &self.tera,
            template_name,
            &self.context,
            status,
            &self.config,
        )
    }
}

use tower_sessions::Session;

// L'Extractor Flash que vous utiliserez dans vos Handlers Axum.
// Il encapsule la Session pour y ajouter des méthodes d'insertion simplifiées.
pub struct Message(Session);

impl<S> FromRequestParts<S> for Message
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Récupérer la session depuis les extensions
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone();

        Ok(Message(session))
    }
}
impl Message {
    /// Ajoute un message flash de succès
    pub async fn success<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::success(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Ajoute un message flash d'erreur
    pub async fn error<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::error(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Ajoute un message flash d'information
    pub async fn info<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::info(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
