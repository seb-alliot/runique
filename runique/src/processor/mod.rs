// runique/src/processor/message_processor.rs

use crate::context;
use crate::middleware::flash_message::FlashMessage;
use crate::middleware::flash_message::FlashMessageSession;
use crate::settings::Settings;
use axum::{extract::FromRequestParts, http::StatusCode, http::request::Parts, response::Response};
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone)]
pub struct Template {
    tera: Arc<Tera>,
    config: Arc<Settings>,
    context: Context,
}

use crate::middleware::csrf::CsrfToken;

impl<S> FromRequestParts<S> for Template
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tera_arc = parts
            .extensions
            .get::<Arc<Tera>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let config_arc = parts
            .extensions
            .get::<Arc<Settings>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut context = Context::new();

        // 1. Injection automatique des URLs statiques
        context.insert("static_runique", &config_arc.static_runique_url);

        // 2. Injection du CSRF Token
        if let Some(token) = parts.extensions.get::<CsrfToken>() {
            context.insert("csrf_token", &token.0);
        }

        // 3. Injection des Messages Flash
        if let Some(messages) = parts.extensions.get::<Vec<FlashMessage>>() {
            context.insert("messages", messages);
        }

        // 4. Injection du flag Debug
        context.insert("debug", &config_arc.debug);

        // 5. Injection du nonce CSP
        if let Some(csp_nonce) = parts.extensions.get::<String>() {
            context.insert("csp_nonce", csp_nonce);
        }

        Ok(Template {
            tera: Arc::clone(tera_arc),
            config: Arc::clone(config_arc),
            context,
        })
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
        // On étend le contexte existant avec celui de l'utilisateur
        // .extend() est plus efficace ici car 'self' est consommé (ownership)
        self.context.extend(user_context.clone());

        // Appel à ton error_handler optimisé
        crate::middleware::error_handler::render_template(
            &self.tera,
            template_name,
            &self.context,
            status,
            &self.config,
        )
    }

    pub fn render_404(&self, message: &str) -> Response {
        let ctx = context! {
            "title", "Page non trouvée";
            "error_message", message
        };
        // Utilisation de self.clone() car la méthode render() consomme le Template
        self.clone().render("404", &ctx)
    }

    pub fn render_500(&self, message: &str) -> Response {
        let ctx = context! {
            "title", "Erreur serveur";
            "error_message", message
        };
        self.clone().render("500", &ctx)
    }
}

use tower_sessions::Session;

pub struct Message(Session);

impl<S> FromRequestParts<S> for Message
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // On clone la session (qui est un wrapper vers un Arc en interne dans tower-sessions)
        Ok(Message(session.clone()))
    }
}

impl Message {
    pub async fn success<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::success(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn error<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::error(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn info<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::info(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub async fn warning<S: Into<String>>(&mut self, content: S) -> Result<(), StatusCode> {
        self.0
            .insert_message(FlashMessage::warning(content.into()))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
