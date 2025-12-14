// rusti/src/processor/message_processor.rs

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{Response},
    http::StatusCode,
};
use std::sync::Arc;
use tera::{Tera, Context};
use crate::middleware::flash_message::Message;
use crate::settings::Settings;

pub struct Template {
    tera: Arc<Tera>,
    config: Arc<Settings>,
    context: Context,
}

impl<S> FromRequestParts<S> for Template
where
    S: Send + Sync {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Récupérer Tera
        let tera = parts
            .extensions
            .get::<Arc<Tera>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone();

        // Récupérer Config
        let config = parts
            .extensions
            .get::<Arc<Settings>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone();

        let mut context = Context::new();

        // AUTO-INJECT messages
        if let Some(messages) = parts.extensions.get::<Vec<Message>>() {

            context.insert("messages", messages);
        }

        // AUTO-INJECT debug mode
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