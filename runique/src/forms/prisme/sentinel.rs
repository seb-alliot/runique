//! Prisme Sentinel: evaluates access rules before extracting the request body.
use crate::auth::permissions::{groupe, pull_groupes_db};
use crate::auth::session::CurrentUser;
use crate::errors::error::RuniqueError;
use crate::forms::prisme::rules::{GuardRules, evaluate_rules};
use crate::utils::aliases::{ADb, AEngine};
use crate::utils::trad::tf;
use axum::{
    body::Body,
    http::Request,
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};

/// Sentinel: entry point for the `GuardRules` a route placed in extensions,
/// checked against the `CurrentUser` that `auth_middleware` read from the
/// database for this request.
///
/// Takes what it needs from `req` up front: the request (its body isn't
/// `Sync`) can't be held across the group lookup.
pub fn sentinel(
    req: &Request<Body>,
) -> impl Future<Output = Result<(), Box<Response>>> + Send + 'static {
    let rules = req.extensions().get::<GuardRules>().cloned();
    let user = rules
        .as_ref()
        .and_then(|_| req.extensions().get::<CurrentUser>().cloned());
    let engine = req.extensions().get::<AEngine>().cloned();
    async move {
        let Some(rules) = rules else {
            return Ok(());
        };
        let groupes = match &user {
            Some(user) if !rules.roles.is_empty() && !user.is_superuser => {
                group_names(user, engine.as_ref()).await
            }
            _ => Vec::new(),
        };
        let refusal = evaluate_rules(&rules, user.as_ref(), &groupes);
        // A group name that doesn't exist refuses everyone but superusers,
        // silently: told apart from a plain refusal — only once refused, so
        // granted requests pay nothing — and raised as an error, shown on the
        // debug page in debug mode and as a plain 500 otherwise.
        if refusal.is_err()
            && user.is_some()
            && !rules.roles.is_empty()
            && let Some(engine) = &engine
        {
            let unknown = unknown_groups(&engine.db, &rules.roles).await;
            if !unknown.is_empty() {
                let message = tf("forms.unknown_group", &[unknown.join(", ")]);
                return Err(Box::new(
                    RuniqueError::Custom {
                        message,
                        source: None,
                    }
                    .into_response(),
                ));
            }
        }
        refusal
    }
}

/// The account's group names: those already loaded on `user`, else read from
/// the database. A failed read gives none, so the role check refuses.
async fn group_names(user: &CurrentUser, engine: Option<&AEngine>) -> Vec<String> {
    let groupes = if !user.groupes.is_empty() {
        user.groupes.clone()
    } else if let Some(engine) = engine {
        pull_groupes_db(&engine.db, user.id).await
    } else {
        Vec::new()
    };
    groupes.into_iter().map(|g| g.nom).collect()
}

/// The names in `roles` with no group in `eihwaz_groupes`. A failed read
/// reports none: the request is then refused as it was.
async fn unknown_groups(db: &ADb, roles: &[String]) -> Vec<String> {
    let existing: Vec<String> = match groupe::Entity::find()
        .select_only()
        .column(groupe::Column::Nom)
        .filter(groupe::Column::Nom.is_in(roles.iter().cloned()))
        .into_tuple()
        .all(db)
        .await
    {
        Ok(existing) => existing,
        Err(e) => {
            tracing::error!(error = %e, "GuardRules: checking the group names failed");
            return Vec::new();
        }
    };
    roles
        .iter()
        .filter(|role| !existing.contains(role))
        .cloned()
        .collect()
}
