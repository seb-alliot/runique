use crate::moteur_engine::engine_struct::RuniqueEngine;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use std::sync::Arc;
use tera::Context;

pub struct RequestContext {
    pub engine: Arc<RuniqueEngine>,
    pub csrf_token: String,
    pub context: Context,
}

impl RequestContext {
    /// Crée un nouveau contexte et injecte les variables globales (Static, CSRF)
    pub fn new(engine: Arc<RuniqueEngine>, csrf_token: String) -> Self {
        let mut context = Context::new();

        // Injection issue de ton ancien mod.rs
        context.insert("static_runique", &engine.config.static_files);
        context.insert("csrf_token", &csrf_token);

        context.insert("debug", &engine.config.debug);

        Self {
            engine,
            csrf_token,
            context,
        }
    }

    /// Rendu d'un template Tera
    pub fn render(self, template_name: &str) -> Response {
        match self.engine.tera.render(template_name, &self.context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                tracing::error!("Erreur de rendu Tera: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de rendu interne").into_response()
            }
        }
    }

    /// Helper pour insérer des données manuellement dans la vue
    pub fn insert<T: serde::Serialize>(&mut self, key: &str, value: T) {
        self.context.insert(key, &value);
    }
}
