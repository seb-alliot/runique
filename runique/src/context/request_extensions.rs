use crate::config::RuniqueConfig;
use crate::engine::RuniqueEngine;
use crate::middleware::auth::CurrentUser;
use crate::utils::csp_nonce::CspNonce;
use crate::utils::csrf::CsrfToken;
use axum::body::Body;
/// Module centralisé pour injecter toutes les données dans les extensions Axum
use axum::http::request::Parts;
use axum::http::Request;
use std::sync::Arc;
use tera::Tera;

/// Structure contenant toutes les données à injecter dans les extensions
pub struct RequestExtensions {
    pub engine: Option<Arc<RuniqueEngine>>,
    pub tera: Option<Arc<Tera>>,
    pub config: Option<Arc<RuniqueConfig>>,
    pub csrf_token: Option<CsrfToken>,
    pub csp_nonce: Option<CspNonce>,
    pub current_user: Option<CurrentUser>,
}

impl RequestExtensions {
    /// Crée une nouvelle instance vide
    pub fn new() -> Self {
        Self {
            engine: None,
            tera: None,
            config: None,
            csrf_token: None,
            csp_nonce: None,
            current_user: None,
        }
    }

    /// Injecte toutes les données présentes dans les extensions
    pub fn inject(&self, parts: &mut Parts) {
        if let Some(engine) = &self.engine {
            parts.extensions.insert(engine.clone());
        }
        if let Some(tera) = &self.tera {
            parts.extensions.insert(tera.clone());
        }
        if let Some(config) = &self.config {
            parts.extensions.insert(config.clone());
        }
        if let Some(csrf_token) = &self.csrf_token {
            parts.extensions.insert(csrf_token.clone());
        }
        if let Some(csp_nonce) = &self.csp_nonce {
            parts.extensions.insert(csp_nonce.clone());
        }
        if let Some(current_user) = &self.current_user {
            parts.extensions.insert(current_user.clone());
        }
    }

    /// Injecte toutes les données dans une Request (wrapper pour Axum)
    pub fn inject_request(&self, req: &mut Request<Body>) {
        // Utilise directement la méthode extensions_mut() de la Request
        let extensions = req.extensions_mut();

        if let Some(engine) = &self.engine {
            extensions.insert(engine.clone());
        }
        if let Some(tera) = &self.tera {
            extensions.insert(tera.clone());
        }
        if let Some(config) = &self.config {
            extensions.insert(config.clone());
        }
        if let Some(csrf_token) = &self.csrf_token {
            extensions.insert(csrf_token.clone());
        }
        if let Some(csp_nonce) = &self.csp_nonce {
            extensions.insert(csp_nonce.clone());
        }
        if let Some(current_user) = &self.current_user {
            extensions.insert(current_user.clone());
        }
    }

    /// Builder pattern - Engine
    pub fn with_engine(mut self, engine: Arc<RuniqueEngine>) -> Self {
        self.engine = Some(engine);
        self
    }

    /// Builder pattern - Tera
    pub fn with_tera(mut self, tera: Arc<Tera>) -> Self {
        self.tera = Some(tera);
        self
    }

    /// Builder pattern - Config
    pub fn with_config(mut self, config: Arc<RuniqueConfig>) -> Self {
        self.config = Some(config);
        self
    }

    /// Builder pattern - CSRF Token
    pub fn with_csrf_token(mut self, csrf_token: CsrfToken) -> Self {
        self.csrf_token = Some(csrf_token);
        self
    }

    /// Builder pattern - CSP Nonce
    pub fn with_csp_nonce(mut self, csp_nonce: CspNonce) -> Self {
        self.csp_nonce = Some(csp_nonce);
        self
    }

    /// Builder pattern - Current User
    pub fn with_current_user(mut self, current_user: CurrentUser) -> Self {
        self.current_user = Some(current_user);
        self
    }
}

impl Default for RequestExtensions {
    fn default() -> Self {
        Self::new()
    }
}
