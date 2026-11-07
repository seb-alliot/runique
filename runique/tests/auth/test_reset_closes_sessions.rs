//! A completed password reset closes every session of the account, on every
//! device — a stolen one included — and nobody else's. It closes the browser
//! it ran in too, whoever was signed in there, and only once it commits: a
//! revocation the database refuses changes nothing. A logout or a sign-in
//! that can't revoke the previous session says so instead of pretending.

use crate::helpers::db;
use crate::helpers::framework_tables::{RESET_TOKENS_DDL, SESSIONS_DDL, USERS_DDL};
use crate::helpers::pk::{pk, pk_sql_literal};
use axum::{Router, extract::Path, routing::get};
use runique::app::RuniqueApp;
use runique::auth::{BuiltinUserEntity, authenticate_user, login, logout};
use runique::config::RuniqueConfig;
use runique::context::template::Request;
use runique::db::ADb;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use runique::utils::password::PasswordConfig;
use runique::utils::reset_token;
use serial_test::serial;
use std::time::Duration;

const NEW_PASSWORD: &str = "Brand-new passw0rd!";

async fn sign_in(Path(id): Path<u32>, req: Request) -> &'static str {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(id))
        .await
        .expect("seeded account");
    login(&req.session, &user, None, false)
        .await
        .expect("login");
    "ok"
}

async fn try_sign_in(Path(id): Path<u32>, req: Request) -> &'static str {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(id))
        .await
        .expect("seeded account");
    match login(&req.session, &user, None, false).await {
        Ok(()) => "ok",
        Err(_) => "refused",
    }
}

async fn sign_out(req: Request) -> &'static str {
    match logout(&req.session, None).await {
        Ok(()) => "out",
        Err(_) => "failed",
    }
}

async fn whoami(req: Request) -> String {
    req.user.clone().map_or("anonymous".into(), |u| u.username)
}

async fn close_all(Path(id): Path<u32>, req: Request) -> &'static str {
    req.engine
        .close_user_sessions(pk(id))
        .await
        .expect("sessions closed");
    "closed"
}

fn test_routes() -> Router {
    Router::new()
        .route("/sign-in/{id}", get(sign_in))
        .route("/try-sign-in/{id}", get(try_sign_in))
        .route("/sign-out", get(sign_out))
        .route("/whoami", get(whoami))
        .route("/close/{id}", get(close_all))
}

async fn spawn() -> (String, DatabaseConnection) {
    spawn_with_password_config(PasswordConfig::auto()).await
}

async fn seeded_db() -> DatabaseConnection {
    let dbc = db::fresh_db().await;
    for ddl in [USERS_DDL, RESET_TOKENS_DDL, SESSIONS_DDL] {
        db::exec(&dbc, ddl).await;
    }
    let hash = runique::utils::password::hash("old password 123").unwrap();
    for (n, name) in [(1, "alice"), (2, "bob")] {
        db::exec(
            &dbc,
            &format!(
                "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_superuser, activated_at) \
                 VALUES ({}, '{name}', '{name}@example.com', '{hash}', 1, 0, '2026-01-01 00:00:00')",
                pk_sql_literal(n)
            ),
        )
        .await;
    }
    dbc
}

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

async fn spawn_with_password_config(password: PasswordConfig) -> (String, DatabaseConnection) {
    let dbc = seeded_db().await;
    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    config.password = password;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(test_routes())
        .with_password_reset(|pr| pr)
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .expect("app");
    (serve(app.router).await, dbc)
}

/// The sessions live in a store Runique can't search by user, as with Redis.
async fn spawn_with_custom_store() -> (String, DatabaseConnection) {
    let dbc = seeded_db().await;
    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(test_routes())
        .with_password_reset(|pr| pr)
        .middleware(|m| m.with_session_store(tower_sessions::MemoryStore::default()))
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .expect("app");
    (serve(app.router).await, dbc)
}

async fn session_version_of(db: &DatabaseConnection, n: u32) -> i32 {
    let adb = ADb::from_connection(db.clone());
    BuiltinUserEntity::find_by_id(&adb, pk(n))
        .await
        .expect("seeded account")
        .session_version
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn text(client: &reqwest::Client, url: String) -> String {
    client.get(url).send().await.unwrap().text().await.unwrap()
}

async fn sessions_of(db: &DatabaseConnection, n: u32) -> i64 {
    db.query_one_raw(runique::sea_orm::Statement::from_string(
        db.get_database_backend(),
        format!(
            "SELECT COUNT(*) AS n FROM eihwaz_sessions WHERE user_id = {}",
            pk_sql_literal(n)
        ),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

/// Sets alice's new password through the emailed link, from a fresh client.
async fn complete_reset(base: &str, db: &DatabaseConnection) {
    let adb = ADb::from_connection(db.clone());
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();
    let url = format!("{base}/reset-password/{token}");
    let reset = client();
    let page = reset.get(&url).send().await.unwrap();
    let csrf = page.headers()["x-csrf-token"].to_str().unwrap().to_string();
    let resp = reset
        .post(&url)
        .form(&[
            ("token", token.as_str()),
            ("email", "alice@example.com"),
            ("password", NEW_PASSWORD),
            ("confirm", NEW_PASSWORD),
            ("csrf_token", csrf.as_str()),
            ("runique_form", "reset_password"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    assert!(
        authenticate_user(&adb, "alice", NEW_PASSWORD)
            .await
            .is_some(),
        "the new password is set"
    );
}

/// Posts alice's new password from `browser`, whatever account it is signed in as.
async fn post_reset(base: &str, browser: &reqwest::Client, token: &str) -> u16 {
    let url = format!("{base}/reset-password/{token}");
    let page = browser.get(&url).send().await.unwrap();
    let csrf = page.headers()["x-csrf-token"].to_str().unwrap().to_string();
    browser
        .post(&url)
        .form(&[
            ("token", token),
            ("email", "alice@example.com"),
            ("password", NEW_PASSWORD),
            ("confirm", NEW_PASSWORD),
            ("csrf_token", csrf.as_str()),
            ("runique_form", "reset_password"),
        ])
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

const REJECT_SESSION_DELETE: &str = "CREATE TRIGGER reject_session_delete BEFORE DELETE ON eihwaz_sessions \
     BEGIN SELECT RAISE(FAIL, 'simulated session delete failure'); END";

#[tokio::test]
#[serial]
async fn a_completed_reset_signs_the_account_out_on_every_device() {
    let (base, db) = spawn().await;
    let (laptop, phone, other) = (client(), client(), client());
    for (c, id) in [(&laptop, 1), (&phone, 1), (&other, 2)] {
        text(c, format!("{base}/sign-in/{id}")).await;
    }
    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "alice");
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "alice");

    complete_reset(&base, &db).await;

    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(sessions_of(&db, 1).await, 0, "none left in the database");
    assert_eq!(
        text(&other, format!("{base}/whoami")).await,
        "bob",
        "another account keeps its session"
    );

    // Signing in again after the reset gives a session that stays.
    text(&phone, format!("{base}/sign-in/1")).await;
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "alice");
}

#[tokio::test]
#[serial]
async fn failed_password_update_does_not_consume_the_reset_token() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();
    let url = format!("{base}/reset-password/{token}");
    let reset = client();
    let page = reset.get(&url).send().await.unwrap();
    let csrf = page.headers()["x-csrf-token"].to_str().unwrap().to_string();

    db::exec(
        &db,
        "CREATE TRIGGER reject_password_update BEFORE UPDATE OF password ON eihwaz_users \
         BEGIN SELECT RAISE(FAIL, 'simulated password update failure'); END",
    )
    .await;

    let _response = reset
        .post(&url)
        .form(&[
            ("token", token.as_str()),
            ("email", "alice@example.com"),
            ("password", NEW_PASSWORD),
            ("confirm", NEW_PASSWORD),
            ("csrf_token", csrf.as_str()),
            ("runique_form", "reset_password"),
        ])
        .send()
        .await
        .unwrap();

    assert!(reset_token::peek(&adb, &token).await);
    assert!(
        authenticate_user(&adb, "alice", "old password 123")
            .await
            .is_some(),
        "a failed password update must leave the previous credential usable"
    );
}

#[tokio::test]
#[serial]
async fn a_wrong_email_does_not_consume_the_reset_token() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db);
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();
    let url = format!("{base}/reset-password/{token}");
    let reset = client();
    let page = reset.get(&url).send().await.unwrap();
    assert_eq!(page.status().as_u16(), 200);
    let csrf = page.headers()["x-csrf-token"].to_str().unwrap().to_string();

    let response = reset
        .post(&url)
        .form(&[
            ("token", token.as_str()),
            ("email", "bob@example.com"),
            ("password", NEW_PASSWORD),
            ("confirm", NEW_PASSWORD),
            ("csrf_token", csrf.as_str()),
            ("runique_form", "reset_password"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    assert!(reset_token::peek(&adb, &token).await);
    assert!(
        authenticate_user(&adb, "alice", "old password 123")
            .await
            .is_some(),
        "a mismatched email cannot change the account password"
    );
    assert!(
        authenticate_user(&adb, "alice", NEW_PASSWORD)
            .await
            .is_none()
    );
}

/// `request.user` is the account as the database has it now, like Django's:
/// a rename shows on the very next request, nothing is kept in the session.
#[tokio::test]
#[serial]
async fn request_user_follows_a_rename_at_once() {
    let (base, db) = spawn().await;
    let alice = client();
    assert_eq!(text(&alice, format!("{base}/sign-in/1")).await, "ok");
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "alice");

    db::exec(
        &db,
        &format!(
            "UPDATE eihwaz_users SET username = 'alicia' WHERE id = {}",
            pk_sql_literal(1)
        ),
    )
    .await;
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "alicia");
    assert_eq!(text(&client(), format!("{base}/whoami")).await, "anonymous");
}

/// A store that can't be searched by user: the version alone closes its
/// sessions, those of this account and no other.
#[tokio::test]
#[serial]
async fn a_completed_reset_closes_the_sessions_of_a_custom_store() {
    let (base, db) = spawn_with_custom_store().await;
    let (laptop, phone, other) = (client(), client(), client());
    for (c, id) in [(&laptop, 1), (&phone, 1), (&other, 2)] {
        text(c, format!("{base}/sign-in/{id}")).await;
    }
    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "alice");

    complete_reset(&base, &db).await;

    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(text(&other, format!("{base}/whoami")).await, "bob");

    // Signing in again reads the new version: that session stays.
    text(&phone, format!("{base}/sign-in/1")).await;
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "alice");
    assert_eq!(text(&phone, format!("{base}/whoami")).await, "alice");
}

#[tokio::test]
#[serial]
async fn close_user_sessions_reaches_a_custom_store() {
    let (base, db) = spawn_with_custom_store().await;
    let (alice, bob) = (client(), client());
    text(&alice, format!("{base}/sign-in/1")).await;
    text(&bob, format!("{base}/sign-in/2")).await;

    assert_eq!(text(&bob, format!("{base}/close/1")).await, "closed");

    assert_eq!(session_version_of(&db, 1).await, 1);
    assert_eq!(session_version_of(&db, 2).await, 0);
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(text(&bob, format!("{base}/whoami")).await, "bob");
}

/// Signing in rehashes an outdated password through `update_password_by_id`:
/// that must sign nobody out.
#[tokio::test]
#[serial]
async fn updating_the_stored_hash_closes_no_session() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    let alice = client();
    text(&alice, format!("{base}/sign-in/1")).await;

    let hash = runique::utils::password::hash("another password 456").unwrap();
    BuiltinUserEntity::update_password_by_id(&adb, pk(1), &hash)
        .await
        .unwrap();

    assert_eq!(session_version_of(&db, 1).await, 0);
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "alice");
}

/// The increase is part of the reset's transaction: refused, it leaves the
/// version where it was.
#[tokio::test]
#[serial]
async fn a_refused_reset_leaves_the_session_version() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    // The trigger only fires on a row: without a session there is nothing to refuse.
    text(&client(), format!("{base}/sign-in/1")).await;
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();
    db::exec(&db, REJECT_SESSION_DELETE).await;

    post_reset(&base, &client(), &token).await;

    assert!(
        reset_token::peek(&adb, &token).await,
        "the reset was really refused"
    );
    assert_eq!(session_version_of(&db, 1).await, 0);
}

#[tokio::test]
#[serial]
async fn each_increase_counts_and_touches_one_account() {
    let (_, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    for _ in 0..2 {
        BuiltinUserEntity::increase_session_version(&adb, pk(1))
            .await
            .unwrap();
    }
    assert_eq!(session_version_of(&db, 1).await, 2);
    assert_eq!(session_version_of(&db, 2).await, 0);
}

/// Opening the emailed link only shows the form: nobody is signed out by a
/// GET, so another site linking to the page can't log users out.
#[tokio::test]
#[serial]
async fn showing_the_reset_form_signs_nobody_out() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    let browser = client();
    text(&browser, format!("{base}/sign-in/2")).await;
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();

    for url in [
        format!("{base}/reset-password/{token}"),
        format!("{base}/reset-password/made-up-token"),
    ] {
        browser.get(url).send().await.unwrap();
        assert_eq!(text(&browser, format!("{base}/whoami")).await, "bob");
    }
    assert_eq!(sessions_of(&db, 2).await, 1);
}

/// The reset closes the session of the browser it ran in, signed in as another
/// account (a shared computer), and leaves that account's other devices alone.
#[tokio::test]
#[serial]
async fn a_reset_signs_out_the_browser_it_ran_in_whoever_was_signed_in() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    let (shared, bobs_phone) = (client(), client());
    text(&shared, format!("{base}/sign-in/2")).await;
    text(&bobs_phone, format!("{base}/sign-in/2")).await;
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();

    assert_eq!(post_reset(&base, &shared, &token).await, 200);

    assert_eq!(text(&shared, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(
        text(&bobs_phone, format!("{base}/whoami")).await,
        "bob",
        "only the shared browser's session is closed, not bob's account"
    );
    assert_eq!(sessions_of(&db, 2).await, 1);
}

/// The session revocation is part of the reset's transaction: if the database
/// refuses it, the token stays usable, the old password stays, and every
/// session lives on.
#[tokio::test]
#[serial]
async fn a_refused_session_revocation_undoes_the_whole_reset() {
    let (base, db) = spawn().await;
    let adb = ADb::from_connection(db.clone());
    let laptop = client();
    text(&laptop, format!("{base}/sign-in/1")).await;
    let token = reset_token::generate(&adb, pk(1), Duration::from_secs(600))
        .await
        .unwrap();
    db::exec(&db, REJECT_SESSION_DELETE).await;

    post_reset(&base, &client(), &token).await;

    assert!(
        reset_token::peek(&adb, &token).await,
        "the link still works"
    );
    assert!(
        authenticate_user(&adb, "alice", "old password 123")
            .await
            .is_some(),
        "the password change is rolled back with the revocation"
    );
    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "alice");
    assert_eq!(sessions_of(&db, 1).await, 1);

    db::exec(&db, "DROP TRIGGER reject_session_delete").await;
    post_reset(&base, &client(), &token).await;
    assert_eq!(text(&laptop, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(sessions_of(&db, 1).await, 0);
}

/// A logout the database refuses reports it, and the session stays whole,
/// cookie included, so the user can try again; once it goes through, the
/// session is gone.
#[tokio::test]
#[serial]
async fn a_refused_logout_keeps_the_session_and_says_so() {
    let (base, db) = spawn().await;
    let alice = client();
    text(&alice, format!("{base}/sign-in/1")).await;
    db::exec(&db, REJECT_SESSION_DELETE).await;

    assert_eq!(text(&alice, format!("{base}/sign-out")).await, "failed");
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "alice");
    assert_eq!(sessions_of(&db, 1).await, 1);

    db::exec(&db, "DROP TRIGGER reject_session_delete").await;
    assert_eq!(text(&alice, format!("{base}/sign-out")).await, "out");
    assert_eq!(text(&alice, format!("{base}/whoami")).await, "anonymous");
    assert_eq!(sessions_of(&db, 1).await, 0);
}

/// Signing in as another account first closes the previous one. If that
/// can't be done, the sign-in is refused: the previous session would
/// otherwise survive, a stolen copy of its cookie with it.
#[tokio::test]
#[serial]
async fn switching_account_is_refused_while_the_previous_session_cannot_be_closed() {
    let (base, db) = spawn().await;
    let browser = client();
    text(&browser, format!("{base}/sign-in/2")).await;
    db::exec(&db, REJECT_SESSION_DELETE).await;

    assert_eq!(
        text(&browser, format!("{base}/try-sign-in/1")).await,
        "refused"
    );
    assert_eq!(text(&browser, format!("{base}/whoami")).await, "bob");

    db::exec(&db, "DROP TRIGGER reject_session_delete").await;
    assert_eq!(text(&browser, format!("{base}/try-sign-in/1")).await, "ok");
    assert_eq!(text(&browser, format!("{base}/whoami")).await, "alice");
    assert_eq!(
        sessions_of(&db, 2).await,
        0,
        "bob's previous session is gone"
    );
}
