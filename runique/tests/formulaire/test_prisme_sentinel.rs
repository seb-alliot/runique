//! Tests — forms/prisme/sentinel.rs
//! Couvre : les règles lues dans les extensions, le `CurrentUser` placé par
//! `auth_middleware`, et les groupes chargés depuis la base pour `roles`.

use crate::helpers::framework_tables::{
    GROUPES_DDL, GROUPES_DROITS_DDL, USERS_DDL, USERS_GROUPES_DDL,
};
use crate::helpers::pk::{pk, pk_sql_literal};
use crate::helpers::server::build_engine;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use runique::auth::current_user::CurrentUser;
use runique::auth::permissions::Groupe;
use runique::auth::permissions::rang::Rang;
use runique::forms::prisme::rules::GuardRules;
use runique::forms::prisme::sentinel::sentinel;
use runique::sea_orm::ConnectionTrait;
use runique::utils::aliases::AEngine;

fn user(id: u32, groupes: &[&str]) -> CurrentUser {
    CurrentUser {
        id: pk(id),
        username: format!("user{id}"),
        rang: Rang::Lambda,
        groupes: groupes
            .iter()
            .enumerate()
            .map(|(i, nom)| Groupe {
                id: i as i32 + 1,
                nom: nom.to_string(),
                is_staff: false,
                permissions: vec![],
            })
            .collect(),
        droits: Vec::new(),
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

/// Engine whose database holds the group `editeur`, joined by account 1 only,
/// and the staff group `equipe`, joined by account 3 only; both answered for
/// by the dirigeant account 9.
async fn engine_with_editeur() -> AEngine {
    let engine = build_engine().await;
    let responsable = pk_sql_literal(9);
    for sql in [
        USERS_DDL.to_string(),
        GROUPES_DDL.to_string(),
        GROUPES_DROITS_DDL.to_string(),
        USERS_GROUPES_DDL.to_string(),
        format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_dirigeant) \
             VALUES ({responsable}, 'dirigeant', 'dirigeant@example.com', 'x', 1)"
        ),
        format!(
            "INSERT INTO eihwaz_groupes (id, nom, is_staff, responsable_id) \
             VALUES (1, 'editeur', 0, {responsable}), (2, 'equipe', 1, {responsable})"
        ),
        format!(
            "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({}, 1), ({}, 2)",
            pk_sql_literal(1),
            pk_sql_literal(3)
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

// Staff access comes from the groups: read from the database for an account
// without a rank, never for one whose rank already passes.
#[tokio::test]
async fn test_staff_reads_the_staff_flag_of_the_groups() {
    let engine = engine_with_editeur().await;
    let rules = || Some(GuardRules::staff());
    assert_eq!(
        status(request(rules(), Some(user(3, &[])), Some(engine.clone()))).await,
        StatusCode::OK
    );
    assert_eq!(
        status(request(rules(), Some(user(1, &[])), Some(engine))).await,
        StatusCode::FORBIDDEN,
        "a group that doesn't open the admin"
    );
}

#[tokio::test]
async fn test_staff_lets_a_rank_through_without_reading_the_groups() {
    // No engine: a lookup would find nothing, so passing proves none was needed.
    let mut admin = user(1, &[]);
    admin.rang = Rang::Admin;
    assert_eq!(
        status(request(Some(GuardRules::staff()), Some(admin), None)).await,
        StatusCode::OK
    );
}
