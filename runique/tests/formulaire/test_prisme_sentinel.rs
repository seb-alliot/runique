//! Tests — forms/prisme/sentinel.rs
//! Couvre : les règles lues dans les extensions, le `CurrentUser` placé par
//! `auth_middleware`, et les groupes chargés depuis la base pour `roles`.

use crate::helpers::admin_server::{GROUPES_DDL, GROUPES_DROITS_DDL, USERS_GROUPES_DDL};
use crate::helpers::pk::{pk, pk_sql_literal};
use crate::helpers::server::build_engine;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use runique::auth::permissions::Groupe;
use runique::auth::session::CurrentUser;
use runique::forms::prisme::rules::GuardRules;
use runique::forms::prisme::sentinel::sentinel;
use runique::sea_orm::ConnectionTrait;
use runique::utils::aliases::AEngine;

fn user(id: u32, groupes: &[&str]) -> CurrentUser {
    CurrentUser {
        id: pk(id),
        username: format!("user{id}"),
        is_staff: false,
        is_superuser: false,
        groupes: groupes
            .iter()
            .enumerate()
            .map(|(i, nom)| Groupe {
                id: i as i32 + 1,
                nom: nom.to_string(),
                permissions: vec![],
            })
            .collect(),
    }
}

fn request(
    rules: Option<GuardRules>,
    user: Option<CurrentUser>,
    engine: Option<AEngine>,
) -> Request<Body> {
    let mut req = Request::builder().body(Body::empty()).unwrap();
    if let Some(rules) = rules {
        req.extensions_mut().insert(rules);
    }
    if let Some(user) = user {
        req.extensions_mut().insert(user);
    }
    if let Some(engine) = engine {
        req.extensions_mut().insert(engine);
    }
    req
}

async fn status(req: Request<Body>) -> StatusCode {
    match sentinel(&req).await {
        Ok(()) => StatusCode::OK,
        Err(resp) => resp.status(),
    }
}

/// Engine whose database holds the group `editeur`, joined by account 1 only.
async fn engine_with_editeur() -> AEngine {
    let engine = build_engine().await;
    for sql in [
        GROUPES_DDL.to_string(),
        GROUPES_DROITS_DDL.to_string(),
        USERS_GROUPES_DDL.to_string(),
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (1, 'editeur')".to_string(),
        format!(
            "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({}, 1)",
            pk_sql_literal(1)
        ),
    ] {
        engine.db.execute_unprepared(&sql).await.unwrap();
    }
    engine
}

#[tokio::test]
async fn test_without_rules_everyone_passes() {
    assert_eq!(status(request(None, None, None)).await, StatusCode::OK);
}

#[tokio::test]
async fn test_login_required_reads_the_current_user() {
    let rules = || Some(GuardRules::login_required());
    assert_eq!(
        status(request(rules(), None, None)).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(request(rules(), Some(user(1, &[])), None)).await,
        StatusCode::OK
    );
}

#[tokio::test]
async fn test_roles_uses_the_groups_already_loaded() {
    let rules = || Some(GuardRules::roles(["editeur"]));
    assert_eq!(
        status(request(rules(), Some(user(1, &["editeur"])), None)).await,
        StatusCode::OK
    );
    assert_eq!(
        status(request(rules(), Some(user(1, &["lecteur"])), None)).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_roles_reads_the_groups_from_the_database() {
    let engine = engine_with_editeur().await;
    let rules = || Some(GuardRules::roles(["editeur"]));
    assert_eq!(
        status(request(rules(), Some(user(1, &[])), Some(engine.clone()))).await,
        StatusCode::OK
    );
    assert_eq!(
        status(request(rules(), Some(user(2, &[])), Some(engine))).await,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_roles_refuses_when_the_groups_cannot_be_read() {
    // No engine to read them from, and none loaded: refused, never let through.
    assert_eq!(
        status(request(
            Some(GuardRules::roles(["editeur"])),
            Some(user(1, &[])),
            None
        ))
        .await,
        StatusCode::FORBIDDEN
    );
}

// A misspelt group refuses everyone silently: raised as an error naming it, so
// the developer sees it on the debug page.
#[tokio::test]
async fn test_roles_names_a_group_that_does_not_exist() {
    let engine = engine_with_editeur().await;
    let req = request(
        Some(GuardRules::roles(["editeru"])),
        Some(user(1, &[])),
        Some(engine),
    );
    let resp = sentinel(&req).await.expect_err("refused");
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let error = resp
        .extensions()
        .get::<std::sync::Arc<runique::errors::error::RuniqueError>>()
        .expect("an error the error page can show");
    assert!(error.to_string().contains("editeru"), "{error}");
}

// Only a refusal looks the names up: a member of one of the groups passes.
#[tokio::test]
async fn test_roles_lets_a_member_through_despite_another_unknown_name() {
    let engine = engine_with_editeur().await;
    let rules = Some(GuardRules::roles(["editeur", "editeru"]));
    assert_eq!(
        status(request(rules, Some(user(1, &[])), Some(engine))).await,
        StatusCode::OK
    );
}
