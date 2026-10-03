//! "Forgot password" by account state — a link for a pending account (its lost
//! activation email) and for an active one, nothing for a blocked one, and the
//! same answer on the page whatever the address.

use crate::helpers::admin_server::{RESET_TOKENS_DDL, USERS_DDL};
use crate::helpers::db;
use crate::helpers::pk::pk_sql_literal;
use axum::{Router, routing::get};
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use serial_test::serial;

async fn spawn() -> (String, DatabaseConnection) {
    let dbc = db::fresh_db().await;
    db::exec(&dbc, USERS_DDL).await;
    db::exec(&dbc, RESET_TOKENS_DDL).await;
    for (n, name, active, activated) in [
        (1, "pending", 0, "NULL"),
        (2, "active", 1, "'2026-01-01 00:00:00'"),
        (3, "blocked", 0, "'2026-01-01 00:00:00'"),
    ] {
        db::exec(
            &dbc,
            &format!(
                "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
                 VALUES ({}, '{name}', '{name}@example.com', 'h', {active}, 0, 0, {activated})",
                pk_sql_literal(n)
            ),
        )
        .await;
    }

    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(Router::new().route("/", get(|| async { "ok" })))
        // Several requests in a row from one address: above the default rate limit.
        .with_password_reset(|mut pr| {
            pr.max_requests = 100;
            pr
        })
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .expect("app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app.router).await.unwrap() });
    (format!("http://{addr}"), dbc)
}

async fn tokens(db: &DatabaseConnection) -> i64 {
    db.query_one_raw(runique::sea_orm::Statement::from_string(
        db.get_database_backend(),
        "SELECT COUNT(*) AS n FROM eihwaz_reset_tokens".to_string(),
    ))
    .await
    .unwrap()
    .unwrap()
    .try_get("", "n")
    .unwrap()
}

/// Asks for a link for `email`; returns the response status and target.
async fn forgot(base: &str, email: &str) -> (u16, String) {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let url = format!("{base}/forgot-password");
    let page = client.get(&url).send().await.unwrap();
    let token = page.headers()["x-csrf-token"].to_str().unwrap().to_string();
    let resp = client
        .post(&url)
        .form(&[("email", email), ("csrf_token", token.as_str())])
        .send()
        .await
        .unwrap();
    let location = resp
        .headers()
        .get("location")
        .map_or(String::new(), |l| l.to_str().unwrap().to_string());
    (resp.status().as_u16(), location)
}

#[tokio::test]
#[serial]
async fn only_pending_and_active_accounts_get_a_link_and_the_page_never_tells() {
    let (base, db) = spawn().await;
    let mut answers = Vec::new();

    answers.push(forgot(&base, "blocked@example.com").await);
    assert_eq!(tokens(&db).await, 0, "a blocked account gets no link");
    answers.push(forgot(&base, "nobody@example.com").await);
    assert_eq!(tokens(&db).await, 0);

    answers.push(forgot(&base, "pending@example.com").await);
    assert_eq!(
        tokens(&db).await,
        1,
        "a lost activation email can be asked again"
    );
    answers.push(forgot(&base, "active@example.com").await);
    assert_eq!(tokens(&db).await, 2);

    assert!(
        answers.windows(2).all(|w| w[0] == w[1]),
        "same answer for every address: {answers:?}"
    );
}
