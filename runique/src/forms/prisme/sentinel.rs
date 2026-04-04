//! Sentinel Prisme : évalue les règles d'accès avant l'extraction du corps de requête.
use crate::config::RuniqueConfig;
use crate::forms::prisme::rules::{GuardContext, GuardRules, evaluate_rules};
use axum::{body::Body, http::Request, response::Response};

/// Sentinel : point d'entrée pour les règles d'accès (login, rôle, feature flags).
/// Les règles peuvent être injectées dans les extensions via `GuardRules` et le contexte via `GuardContext`.
pub fn sentinel(req: &Request<Body>, _config: &RuniqueConfig) -> Result<(), Box<Response>> {
    let rules: Option<&GuardRules> = req.extensions().get::<GuardRules>();
    if let Some(rules) = rules {
        let ctx: Option<&GuardContext> = req.extensions().get::<GuardContext>();
        evaluate_rules(rules, ctx)
    } else {
        Ok(())
    }
}
