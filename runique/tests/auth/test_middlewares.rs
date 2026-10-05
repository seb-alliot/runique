//! Tests — the app-wide auth middleware: `req.user` is the signed-in account
//! as the database knows it now, not the copy taken into the session at login.

use crate::helpers::admin_server::USERS_DDL;
use crate::helpers::db;
use crate::helpers::pk::{pk, pk_sql_literal};
use axum::{Router, routing::get};
use runique::app::RuniqueApp;
use runique::auth::{BuiltinUserEntity, login};
use runique::config::RuniqueConfig;
use runique::context::template::Request;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use serial_test::serial;

async fn sign_in(req: Request) -> &'static str {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(1))
        .await
        .expect("seeded account");
    login(&req.session, &user, None, false)
        .await
        .expect("login");
    "ok"
}

async fn whoami(req: Request) -> String {
    req.user.map_or("anonymous".into(), |u| {
        format!("{}|{}", u.username, u.is_staff)
    })
}

async fn spawn() -> (String, DatabaseConnection) {
    let dbc = db::fresh_db().await;
    db::exec(&dbc, USERS_DDL).await;
    db::exec(
        &dbc,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, 'alice', 'alice@example.com', 'x', 1, 1, 0, '2026-01-01 00:00:00')",
            pk_sql_literal(1)
        ),
    )
    .await;

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(
            Router::new()
                .route("/sign-in", get(sign_in))
                .route("/whoami", get(whoami)),
        )
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .expect("app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app.router).await.unwrap() });
    (format!("http://{addr}"), dbc)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn get_text(client: &reqwest::Client, url: String) -> String {
    client.get(url).send().await.unwrap().text().await.unwrap()
}

#[tokio::test]
#[serial]
async fn an_anonymous_visitor_has_no_user() {
    let (base, _db) = spawn().await;
    assert_eq!(
        get_text(&client(), format!("{base}/whoami")).await,
        "anonymous"
    );
}

#[tokio::test]
#[serial]
async fn the_user_reflects_the_database_on_every_request() {
    let (base, db) = spawn().await;
    let c = client();
    assert_eq!(get_text(&c, format!("{base}/sign-in")).await, "ok");
    assert_eq!(get_text(&c, format!("{base}/whoami")).await, "alice|true");

    db.execute_unprepared("UPDATE eihwaz_users SET is_staff = 0")
        .await
        .unwrap();
    assert_eq!(
        get_text(&c, format!("{base}/whoami")).await,
        "alice|false",
        "read from the database, not from the session"
    );
}

#[tokio::test]
#[serial]
async fn a_deactivated_account_is_signed_out_on_its_next_request() {
    let (base, db) = spawn().await;
    let c = client();
    get_text(&c, format!("{base}/sign-in")).await;
    assert_eq!(get_text(&c, format!("{base}/whoami")).await, "alice|true");

    db.execute_unprepared("UPDATE eihwaz_users SET is_active = 0")
        .await
        .unwrap();
    assert_eq!(get_text(&c, format!("{base}/whoami")).await, "anonymous");

    // Closed, not just hidden: reactivating doesn't bring the session back.
    db.execute_unprepared("UPDATE eihwaz_users SET is_active = 1")
        .await
        .unwrap();
    assert_eq!(get_text(&c, format!("{base}/whoami")).await, "anonymous");
}
