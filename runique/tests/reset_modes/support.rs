//! An app whose `build()` is the first to set the global password config
//! (`password_init`, first call wins): each binary using it tests one mode,
//! alone in its process.
use crate::helpers::db;
use crate::helpers::framework_tables::{RESET_TOKENS_DDL, SESSIONS_DDL, USERS_DDL};
use crate::helpers::pk::{pk, pk_sql_literal};
use runique::app::RuniqueApp;
use runique::auth::BuiltinUserEntity;
use runique::config::RuniqueConfig;
use runique::db::ADb;
use runique::sea_orm::DatabaseConnection;
use runique::utils::password::PasswordConfig;

pub const NEW_PASSWORD: &str = "Brand-new passw0rd!";

/// The seeded account's password, never verified: hashing a real one here
/// would set the global config before `build()` does.
pub const SEEDED_HASH: &str = "seeded-password-never-verified";

pub async fn spawn(password: PasswordConfig) -> (String, DatabaseConnection) {
    let dbc = db::fresh_db().await;
    for ddl in [USERS_DDL, RESET_TOKENS_DDL, SESSIONS_DDL] {
        db::exec(&dbc, ddl).await;
    }
    db::exec(
        &dbc,
        &format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_superuser, activated_at) \
             VALUES ({}, 'alice', 'alice@example.com', '{SEEDED_HASH}', 1, 0, '2026-01-01 00:00:00')",
            pk_sql_literal(1)
        ),
    )
    .await;
    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    config.password = password;
    let app = RuniqueApp::builder(config)
        .with_database(dbc.clone())
        .routes(axum::Router::new())
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

pub async fn submit_reset(base: &str, token: &str) {
    let url = format!("{base}/reset-password/{token}");
    let browser = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
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
        .unwrap();
}

pub async fn stored_password(db: &DatabaseConnection) -> String {
    let adb = ADb::from_connection(db.clone());
    BuiltinUserEntity::find_by_id(&adb, pk(1))
        .await
        .expect("seeded account")
        .password
}
