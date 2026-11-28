//! Axum request extensions: injects the engine (which carries Tera and the config), CSRF, CSP nonce, and current user.
use crate::auth::current_user::CurrentUser;
use axum::{body::Body, http::Request};

use crate::{
    security::{csp_nonce::CspNonce, csrf::CsrfToken},
    utils::aliases::{AEngine, OAEngine, OCspNonce, OCsrfToken, OCurrentUser},
};

/// Structure containing all data to be injected into extensions.
pub struct RequestExtensions {
    pub engine: OAEngine,
    pub csrf_token: OCsrfToken,
    pub csp_nonce: OCspNonce,
    pub current_user: OCurrentUser,
}

impl RequestExtensions {
    /// Creates a new empty instance.
    pub fn new() -> Self {
        Self {
            engine: None,
            csrf_token: None,
            csp_nonce: None,
            current_user: None,
        }
    }

    /// Injects all data into a Request (wrapper for Axum).
    pub fn inject_request(&self, req: &mut Request<Body>) {
        // Uses the extensions_mut() method of the Request directly.
        let extensions = req.extensions_mut();

        if let Some(engine) = &self.engine {
            extensions.insert(engine.clone());
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
    /// Builder pattern - CSRF token
    pub fn with_csrf_token(mut self, csrf_token: CsrfToken) -> Self {
        self.csrf_token = Some(csrf_token);
        self
    }
    /// Builder pattern - Engine
    pub fn with_engine(mut self, engine: AEngine) -> Self {
        self.engine = Some(engine);
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
