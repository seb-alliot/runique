//! Prisme Sentinel: evaluates access rules before extracting the request body.
use crate::config::RuniqueConfig;
use crate::forms::prisme::rules::{GuardContext, GuardRules, evaluate_rules};
use axum::{body::Body, http::Request, response::Response};

/// Sentinel: entry point for access rules (login, role, feature flags).
/// Rules can be injected into extensions via `GuardRules` and context via `GuardContext`.
pub fn sentinel(req: &Request<Body>, _config: &RuniqueConfig) -> Result<(), Box<Response>> {
    let rules: Option<&GuardRules> = req.extensions().get::<GuardRules>();
    if let Some(rules) = rules {
        let ctx: Option<&GuardContext> = req.extensions().get::<GuardContext>();
        evaluate_rules(rules, ctx)
    } else {
        Ok(())
    }
}
