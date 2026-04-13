//! Wrapper around the Axum router for user route declarations.
use crate::utils::aliases::AEngine;
use axum::Router;

/// Application router wrapping [`axum::Router`] with state [`AEngine`].
#[derive(Debug, Clone)]
pub struct RuniqueRouter {
    /// Internal Axum router.
    pub core: Router<AEngine>,
}

impl Default for RuniqueRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueRouter {
    /// Creates an empty router.
    pub fn new() -> Self {
        Self {
            core: Router::new(),
        }
    }

    /// Adds a route with its handler.
    pub fn add_route(mut self, path: &str, method: axum::routing::MethodRouter<AEngine>) -> Self {
        self.core = self.core.route(path, method);
        self
    }

    /// Allows merging another group of routes (Nest).
    pub fn nest(mut self, path: &str, router: Router<AEngine>) -> Self {
        self.core = self.core.nest(path, router);
        self
    }
}
