//! `save` / `delete` on a validated form, outside the admin: the row comes
//! from the URL, SeaORM writes it in its own transaction, nested in the
//! test's (a commit that ended the test's transaction fails the run).
use crate::entities::blog::{
    ActiveModel as BlogActiveModel, Column as BlogColumn, Entity as BlogEntity,
};
use crate::formulaire::BlogForm;
use runique::config::app::RuniqueConfig;
use runique::context::template::Request;
use runique::engine::RuniqueEngine;
use runique::forms::Prisme;
use runique::middleware::config::MiddlewareConfig;
use runique::middleware::security::{allowed_hosts::HostPolicy, csp::SecurityPolicy};
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use runique::tower_sessions::{MemoryStore, Session};
use runique::utils::aliases::StrMap;
use runique::utils::constante::session_key::session::FORM_NAME_KEY;
use runique::utils::crypto::csrf::CsrfToken;
use sea_orm::{ColumnTrait, DbErr, PaginatorTrait, QueryFilter};
use std::sync::Arc;

const TITLE: &str = "runique_test form save";

/// A POST of the form `"blog"` with `body`, on a route whose parameters are `path`.
fn request(db: &ADb, path: &[(&str, &str)], body: &[(&str, &str)]) -> Request {
    named_request(db, "blog", path, body)
}

/// A POST of the form `name` with `body`, on a route whose parameters are `path`.
pub(super) fn named_request(
    db: &ADb,
    name: &str,
    path: &[(&str, &str)],
    body: &[(&str, &str)],
) -> Request {
    let engine = Arc::new(RuniqueEngine {
        config: RuniqueConfig::default(),
        tera: Arc::new(runique::tera::Tera::default()),
        db: db.clone(),
        url_registry: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        features: MiddlewareConfig::default(),
        security_csp: Arc::new(SecurityPolicy::default()),
        security_hosts: Arc::new(HostPolicy::new(vec![], true)),
        csrf_exempt_paths: Arc::new(vec![]),
        permissions_policy: Arc::new(runique::middleware::PermissionsPolicy::default()),
        trusted_proxies: Arc::new(runique::middleware::TrustedProxies::default()),
        session_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        session_db_store: std::sync::LazyLock::new(|| std::sync::RwLock::new(None)),
        extensions: std::collections::HashMap::new(),
    });
    let session = Session::new(None, Arc::new(MemoryStore::default()), None);
    let mut req = Request::new(
        engine,
        session,
        CsrfToken("runique-test".to_string()),
        Method::POST,
    );
    req.path_params = path
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    let mut data: StrMap = body
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect();
    data.insert(FORM_NAME_KEY.to_string(), name.to_string());
    req.prisme = Prisme::for_test(data, true);
    req
}

fn article(title: &str) -> Vec<(&str, &str)> {
    vec![
        ("title", title),
        ("email", "runique_test@example.com"),
        ("summary", "Written by a runique test."),
        ("content", "<p>Rolled back at the end of the test.</p>"),
    ]
}

async fn validated(req: &Request) -> Result<ValidationForm<BlogForm>, DbErr> {
    ValidationForm::try_form(req.form::<BlogForm>(), req, "blog")
        .await
        .map_err(|form| {
            DbErr::Custom(format!(
                "the form should be valid: {:?}",
                form.get_form().errors()
            ))
        })
}

async fn existing(db: &ADb, title: &str) -> Result<Pk, DbErr> {
    Ok(BlogActiveModel {
        title: Set(title.to_string()),
        email: Set("runique_test@example.com".to_string()),
        summary: Set("Written by a runique test.".to_string()),
        content: Set("<p>Rolled back at the end of the test.</p>".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?
    .id)
}

async fn titled(db: &ADb, title: &str) -> Result<u64, DbErr> {
    BlogEntity::find()
        .filter(BlogColumn::Title.eq(title))
        .count(db)
        .await
}

async fn title_of(db: &ADb, id: Pk) -> Result<Option<String>, DbErr> {
    Ok(BlogEntity::find_by_id(id).one(db).await?.map(|row| row.title))
}

fn fail(message: &str) -> DbErr {
    DbErr::Custom(message.to_string())
}

#[tokio::test]
async fn save_without_a_key_creates_the_row() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let req = request(db, &[], &article(TITLE));
        if validated(&req).await?.save(db).await.is_err() {
            return Err(fail("save should create the row"));
        }
        match titled(db, TITLE).await? {
            1 => Ok(()),
            n => Err(DbErr::Custom(format!("expected 1 row, found {n}"))),
        }
    })
    .await
}

#[tokio::test]
async fn save_with_the_id_updates_that_row_and_creates_none() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let id = existing(db, "runique_test before").await?;
        let key = id.to_string();
        let req = request(db, &[("id", &key)], &article(TITLE));
        if validated(&req).await?.save(db).await.is_err() {
            return Err(fail("save should update the row"));
        }
        if title_of(db, id).await?.as_deref() != Some(TITLE) {
            return Err(fail("the row of the URL should carry the new title"));
        }
        match titled(db, TITLE).await? {
            1 => Ok(()),
            n => Err(DbErr::Custom(format!("an update made {n} rows"))),
        }
    })
    .await
}

#[tokio::test]
async fn the_key_named_after_the_form_wins_over_the_plain_one() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let named = existing(db, "runique_test named").await?;
        let plain = existing(db, "runique_test plain").await?;
        let (named_key, plain_key) = (named.to_string(), plain.to_string());
        let req = request(
            db,
            &[("blog_id", &named_key), ("id", &plain_key)],
            &article(TITLE),
        );
        if validated(&req).await?.save(db).await.is_err() {
            return Err(fail("save should update the row"));
        }
        if title_of(db, named).await?.as_deref() != Some(TITLE) {
            return Err(fail("{blog_id} should be the row written"));
        }
        if title_of(db, plain).await?.as_deref() != Some("runique_test plain") {
            return Err(fail("{id} should be left alone"));
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn an_unknown_or_unreadable_key_writes_nothing() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        for key in ["2147483000", "abc", "", "1 OR 1=1"] {
            let req = request(db, &[("id", key)], &article(TITLE));
            let form = match validated(&req).await?.save(db).await {
                Ok(_) => return Err(DbErr::Custom(format!("key {key:?} was saved"))),
                Err(form) => form,
            };
            if form.get_form().errors.is_empty() {
                return Err(DbErr::Custom(format!("key {key:?}: no error on the form")));
            }
        }
        match titled(db, TITLE).await? {
            0 => Ok(()),
            n => Err(DbErr::Custom(format!("a refused key wrote {n} rows"))),
        }
    })
    .await
}

#[tokio::test]
async fn delete_removes_the_row_of_the_url() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let id = existing(db, TITLE).await?;
        let key = id.to_string();
        let req = request(db, &[("id", &key)], &article(TITLE));
        if validated(&req).await?.delete(db).await.is_err() {
            return Err(fail("delete should remove the row"));
        }
        match title_of(db, id).await? {
            None => Ok(()),
            Some(_) => Err(fail("the row is still there")),
        }
    })
    .await
}

#[tokio::test]
async fn delete_without_a_key_removes_nothing() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        existing(db, TITLE).await?;
        let req = request(db, &[], &article(TITLE));
        let form = match validated(&req).await?.delete(db).await {
            Ok(_) => return Err(fail("delete without a key should be refused")),
            Err(form) => form,
        };
        if form.get_form().errors.is_empty() {
            return Err(fail("no error on the form"));
        }
        match titled(db, TITLE).await? {
            1 => Ok(()),
            n => Err(DbErr::Custom(format!("expected the row kept, found {n}"))),
        }
    })
    .await
}

#[tokio::test]
async fn user_id_is_none_without_a_login() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let req = request(db, &[], &[]);
        match req.user_id().await {
            None => Ok(()),
            Some(id) => Err(DbErr::Custom(format!("anonymous request gave {id}"))),
        }
    })
    .await
}

/// The form `"blog"` with an empty title: refused by `try_form`, with the
/// error on `title`. `Ok` carries the form, never saved.
async fn refused_on_title(req: &Request) -> Result<(), DbErr> {
    match ValidationForm::try_form(req.form::<BlogForm>(), req, "blog").await {
        Ok(_) => Err(fail("an empty title should not validate")),
        Err(form) if form.get_form().errors().contains_key("title") => Ok(()),
        Err(form) => Err(DbErr::Custom(format!(
            "the error should be on title: {:?}",
            form.get_form().errors()
        ))),
    }
}

#[tokio::test]
async fn an_empty_title_creates_nothing() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let before = BlogEntity::find().count(db).await?;
        for title in ["", "   "] {
            let req = request(db, &[], &article(title));
            refused_on_title(&req).await?;
        }
        match BlogEntity::find().count(db).await? {
            n if n == before => Ok(()),
            n => Err(DbErr::Custom(format!("{before} rows before, {n} after"))),
        }
    })
    .await
}

#[tokio::test]
async fn an_empty_title_leaves_the_row_of_the_url_unchanged() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let id = existing(db, TITLE).await?;
        let key = id.to_string();
        for title in ["", "   "] {
            let req = request(db, &[("id", &key)], &article(title));
            refused_on_title(&req).await?;
        }
        match title_of(db, id).await?.as_deref() {
            Some(TITLE) => Ok(()),
            other => Err(DbErr::Custom(format!("the title became {other:?}"))),
        }
    })
    .await
}

async fn website_of(db: &ADb, id: Pk) -> Result<Option<String>, DbErr> {
    Ok(BlogEntity::find_by_id(id)
        .one(db)
        .await?
        .and_then(|row| row.website))
}

/// `website` written on the create, replaced by a second save on the same
/// row, then emptied: NULL, since the form carries the field.
#[tokio::test]
async fn website_is_set_then_replaced_then_emptied() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let mut body = article(TITLE);
        body.push(("website", "https://first.example.com"));
        let req = request(db, &[], &body);
        if validated(&req).await?.save(db).await.is_err() {
            return Err(fail("the create should go through"));
        }
        let Some(row) = BlogEntity::find()
            .filter(BlogColumn::Title.eq(TITLE))
            .one(db)
            .await?
        else {
            return Err(fail("the created row should be found"));
        };
        if row.website.as_deref() != Some("https://first.example.com") {
            return Err(DbErr::Custom(format!("created with {:?}", row.website)));
        }

        let key = row.id.to_string();
        for (sent, expected) in [
            ("https://second.example.com", Some("https://second.example.com")),
            ("", None),
        ] {
            let mut body = article(TITLE);
            body.push(("website", sent));
            let req = request(db, &[("id", &key)], &body);
            if validated(&req).await?.save(db).await.is_err() {
                return Err(DbErr::Custom(format!("the update with {sent:?} failed")));
            }
            let website = website_of(db, row.id).await?;
            if website.as_deref() != expected {
                return Err(DbErr::Custom(format!(
                    "sent {sent:?}, expected {expected:?}, got {website:?}"
                )));
            }
        }
        match titled(db, TITLE).await? {
            1 => Ok(()),
            n => Err(DbErr::Custom(format!("the updates made {n} rows"))),
        }
    })
    .await
}
