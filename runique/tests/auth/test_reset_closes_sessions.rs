//! A completed password reset closes every session of the account, on every
//! device — a stolen one included — and nobody else's.

use crate::helpers::admin_server::{RESET_TOKENS_DDL, SESSIONS_DDL, USERS_DDL};
use crate::helpers::db;
use crate::helpers::pk::{pk, pk_sql_literal};
use axum::{Router, extract::Path, routing::get};
use runique::app::RuniqueApp;
use runique::auth::{BuiltinUserEntity, authenticate_user, login};
use runique::config::RuniqueConfig;
use runique::context::template::Request;
use runique::db::ADb;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
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

async fn whoami(req: Request) -> String {
    req.user.map_or("anonymous".into(), |u| u.username)
}

async fn spawn() -> (String, DatabaseConnection) {
    let dbc = db::fresh_db().await;
    for ddl in [USERS_DDL, RESET_TOKENS_DDL, SESSIONS_DDL] {
        db::exec(&dbc, ddl).await;
    }
    let hash = runique::utils::password::hash("old password 123").unwrap();
    for (n, name) in [(1, "alice"), (2, "bob")] {
        db::exec(
            &dbc,
            &format!(
                "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
                 VALUES ({}, '{name}', '{name}@example.com', '{hash}', 1, 0, 0, '2026-01-01 00:00:00')",
                pk_sql_literal(n)
            ),
        )
        .await;
    }

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(
            Router::new()
                .route("/sign-in/{id}", get(sign_in))
                .route("/whoami", get(whoami)),
        )
        .with_password_reset(|pr| pr)
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
    let encrypted = reset_token::encrypt_email(&token, "alice@example.com");
    let url = format!("{base}/reset-password/{token}/{encrypted}");
    let reset = client();
    let page = reset.get(&url).send().await.unwrap();
    let csrf = page.headers()["x-csrf-token"].to_str().unwrap().to_string();
    let resp = reset
        .post(&url)
        .form(&[
            ("token", token.as_str()),
            ("encrypted_email", encrypted.as_str()),
            ("email", "alice@example.com"),
            ("password", NEW_PASSWORD),
            ("confirm", NEW_PASSWORD),
            ("csrf_token", csrf.as_str()),
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
