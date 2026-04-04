//! Wrapper autour du router Axum pour la déclaration des routes utilisateur.
use crate::utils::aliases::AEngine;
use axum::Router;

/// Router de l'application wrappant [`axum::Router`] avec le state [`AEngine`].
#[derive(Debug, Clone)]
pub struct RuniqueRouter {
    /// Router Axum interne.
    pub core: Router<AEngine>,
}

impl Default for RuniqueRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl RuniqueRouter {
    /// Crée un router vide.
    pub fn new() -> Self {
        Self {
            core: Router::new(),
        }
    }

    /// Ajoute une route avec son handler.
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
