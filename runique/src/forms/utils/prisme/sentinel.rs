use crate::config::RuniqueConfig;
use crate::forms::utils::prisme::rules::{evaluate_rules, GuardContext, GuardRules};
use axum::{body::Body, http::Request, response::Response};

/// Sentinel : point d'entrée pour les règles d'accès (login, rôle, feature flags).
/// Les règles peuvent être injectées dans les extensions via `GuardRules` et le contexte via `GuardContext`.
pub fn sentinel(req: &Request<Body>, _config: &RuniqueConfig) -> Result<(), Response> {
    let rules = req.extensions().get::<GuardRules>();
    if let Some(rules) = rules {
        let ctx = req.extensions().get::<GuardContext>();
        evaluate_rules(rules, ctx)
    } else {
        Ok(())
    }
}
