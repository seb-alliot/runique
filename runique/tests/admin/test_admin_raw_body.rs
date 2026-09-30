//! Tests — the admin only saves what its form accepted.
//!
//! The data handed to a resource's create/update function used to be the raw
//! request body, with only the form's own fields overwritten after validation:
//! any other key the client added reached the database as is. For the builtin
//! `users` resource, that let a staff member create an account that the docs
//! promise is always inactive, with a random password and a reset email, as
//! active and with a password of their choosing.
//!
//! Each test runs its own admin server on its own runtime, to query the
//! database directly (see `test_admin_password_security.rs` for why).

use crate::helpers::admin_server::{self, ADMIN_PREFIX};
use runique::sea_orm::{
    ConnectionTrait, DatabaseConnection,
    sea_query::{Alias, Expr, ExprTrait, Query},
};
use serial_test::serial;

const SUPERUSER_USERNAME: &str = "crawler";
const SUPERUSER_PASSWORD: &str = "crawler_password_123";

async fn spawn_local_admin_server() -> (String, DatabaseConnection) {
    let (router, dbc) = admin_server::build_admin_app().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local admin server");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve admin app");
    });
    (format!("http://{addr}"), dbc)
}

fn local_client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("reqwest client")
}

async fn csrf_token(client: &reqwest::Client, url: &str) -> String {
    let resp = client.get(url).send().await.expect("GET for csrf token");
    assert_eq!(resp.status(), 200, "GET {url} should render 200");
    resp.headers()
        .get("x-csrf-token")
        .expect("x-csrf-token header missing")
        .to_str()
        .expect("x-csrf-token not UTF-8")
        .to_string()
}

async fn login_as_superuser(base: &str) -> reqwest::Client {
    let client = local_client();
    let login_url = format!("{base}{ADMIN_PREFIX}/login");
    let token = csrf_token(&client, &login_url).await;
    let resp = client
        .post(&login_url)
        .form(&[
            ("username", SUPERUSER_USERNAME),
            ("password", SUPERUSER_PASSWORD),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST admin/login");
    assert!(resp.status().is_redirection(), "login should redirect");
    client
}

/// `(is_active, password)` of the user with this email.
async fn stored_user(db: &DatabaseConnection, email: &str) -> (bool, String) {
    let stmt = Query::select()
        .columns([Alias::new("is_active"), Alias::new("password")])
        .from(Alias::new("eihwaz_users"))
        .and_where(Expr::col(Alias::new("email")).eq(email))
        .to_owned();
    let row = db
        .query_one(&stmt)
        .await
        .expect("query user")
        .unwrap_or_else(|| panic!("no user with email {email}"));
    (
        row.try_get::<bool>("", "is_active").expect("is_active"),
        row.try_get::<String>("", "password").expect("password"),
    )
}

#[tokio::test]
#[serial]
async fn created_user_ignores_fields_the_form_does_not_have() {
    let (base, db) = spawn_local_admin_server().await;
    let client = login_as_superuser(&base).await;

    // A password hash the submitter knows the password of.
    let chosen_hash = runique::utils::password::hash("known_password_123").expect("hash");
    let email = "raw_body_victim@example.com";

    let create_url = format!("{base}{ADMIN_PREFIX}/users/create");
    let token = csrf_token(&client, &create_url).await;
    let resp = client
        .post(&create_url)
        .form(&[
            ("username", "raw_body_victim"),
            ("email", email),
            ("is_active", "true"),
            ("password", chosen_hash.as_str()),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .expect("POST users/create");
    assert!(
        resp.status().is_redirection(),
        "create should redirect, got {} — {}",
        resp.status(),
        resp.text().await.unwrap_or_default()
    );

    let (is_active, password) = stored_user(&db, email).await;
    assert_ne!(
        password, chosen_hash,
        "the password must be the random one the admin generates, not the submitted one"
    );
    assert!(
        !runique::utils::password::verify("known_password_123", &password),
        "the submitter must not know the new account's password"
    );
    assert!(
        !is_active,
        "an account created from the admin must start inactive"
    );
}
