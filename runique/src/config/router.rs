use crate::engine::RuniqueEngine;
use axum::Router;
use std::sync::Arc;

pub struct RuniqueRouter {
    pub core: Router<Arc<RuniqueEngine>>,
}

impl RuniqueRouter {
    pub fn new() -> Self {
        Self {
            core: Router::new(),
        }
    }

    pub fn add_route(
        mut self,
        path: &str,
        method: axum::routing::MethodRouter<Arc<RuniqueEngine>>,
    ) -> Self {
        self.core = self.core.route(path, method);
        self
    }

    /// Permet de fusionner un autre groupe de routes (Nest)
    pub fn nest(mut self, path: &str, router: Router<Arc<RuniqueEngine>>) -> Self {
        self.core = self.core.nest(path, router);
        self
    }
}
