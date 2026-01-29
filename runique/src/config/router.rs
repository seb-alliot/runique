use crate::utils::aliases::AEngine;
use axum::Router;

#[derive(Debug, Clone)]
pub struct RuniqueRouter {
    pub core: Router<AEngine>,
}

impl Default for RuniqueRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueRouter {
    pub fn new() -> Self {
        Self {
            core: Router::new(),
        }
    }

    pub fn add_route(mut self, path: &str, method: axum::routing::MethodRouter<AEngine>) -> Self {
        self.core = self.core.route(path, method);
        self
    }

    /// Permet de fusionner un autre groupe de routes (Nest)
    pub fn nest(mut self, path: &str, router: Router<AEngine>) -> Self {
        self.core = self.core.nest(path, router);
        self
    }
}
