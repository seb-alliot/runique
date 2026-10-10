//! A superuser's account can only be changed by a superuser. A staff member
//! with full rights on `users` could otherwise put their own email on a
//! superuser's account, ask a password reset, and sign in as that superuser.
//! Reading the account stays open.
use crate::helpers::admin_server::{self, ADMIN_PREFIX, SEED_GROUPE_ID, seed_superuser_id_str};
use crate::helpers::pk::{pk, pk_sql_literal};
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serial_test::serial;

const MANAGER: &str = "manager";
const MANAGER_PASSWORD: &str = "manager_password_123";

async fn spawn() -> (String, DatabaseConnection) {
    let (router, db) = admin_server::build_admin_app().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    // A staff member, not a superuser, with every right on `users`.
    let hash = runique::utils::password::hash(MANAGER_PASSWORD).unwrap();
    for sql in [
        format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, '{MANAGER}', 'manager@example.com', '{hash}', 1, 1, 0, '2026-01-01 00:00:00')",
            pk_sql_literal(60)
        ),
        format!(
            "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES ({}, {SEED_GROUPE_ID})",
            pk_sql_literal(60)
        ),
        format!(
            "INSERT INTO eihwaz_groupes_droits \
             (groupe_id, resource_key, can_create, can_read, can_update, can_delete, can_update_own, can_delete_own) \
             VALUES ({SEED_GROUPE_ID}, 'users', 1, 1, 1, 1, 0, 0)"
        ),
        // An ordinary account the manager may change.
        format!(
            "INSERT INTO eihwaz_users (id, username, email, password, is_active, is_staff, is_superuser, activated_at) \
             VALUES ({}, 'member', 'member@example.com', '{hash}', 1, 0, 0, '2026-01-01 00:00:00')",
            pk_sql_literal(61)
        ),
    ] {
        db.execute_unprepared(&sql).await.unwrap();
    }
    (format!("http://{addr}"), db)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn csrf(client: &reqwest::Client, url: &str) -> String {
    let resp = client.get(url).send().await.unwrap();
    resp.headers()["x-csrf-token"].to_str().unwrap().to_string()
}

async fn login(base: &str, username: &str, password: &str) -> reqwest::Client {
    let client = client();
    let url = format!("{base}{ADMIN_PREFIX}/login");
    let token = csrf(&client, &url).await;
    let resp = client
        .post(&url)
        .form(&[
            ("username", username),
            ("password", password),
            ("csrf_token", &token),
        ])
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_redirection(), "{username} signs in");
    client
}

async fn scalar(db: &DatabaseConnection, sql: &str) -> String {
    let row = db
        .query_one_raw(Statement::from_string(db.get_database_backend(), sql))
        .await
        .unwrap();
    row.and_then(|r| r.try_get_by_index::<String>(0).ok())
        .unwrap_or_default()
}

async fn email_of(db: &DatabaseConnection, username: &str) -> String {
    scalar(
        db,
        &format!("SELECT email FROM eihwaz_users WHERE username = '{username}'"),
    )
    .await
}

async fn count(db: &DatabaseConnection, sql: &str) -> i64 {
    let row = db
        .query_one_raw(Statement::from_string(db.get_database_backend(), sql))
        .await
        .unwrap()
        .unwrap();
    row.try_get_by_index::<i64>(0).unwrap()
}

async fn edit_email(client: &reqwest::Client, base: &str, id: &str, username: &str, email: &str) {
    let url = format!("{base}{ADMIN_PREFIX}/users/{id}/edit");
    let token = csrf(client, &url).await;
    let _ = client
        .post(&url)
        .form(&[
            ("username", username),
            ("email", email),
            ("is_staff", "true"),
            ("is_active", "true"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .unwrap();
}

async fn post_action(client: &reqwest::Client, base: &str, id: &str, action: &str) {
    let detail = format!("{base}{ADMIN_PREFIX}/users/{id}/detail");
    let token = csrf(client, &detail).await;
    let _ = client
        .post(format!("{base}{ADMIN_PREFIX}/users/{id}/{action}"))
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn a_staff_member_cannot_change_a_superusers_email() {
    let (base, db) = spawn().await;
    let manager = login(&base, MANAGER, MANAGER_PASSWORD).await;
    let su = seed_superuser_id_str();
    edit_email(&manager, &base, &su, "crawler", "manager@evil.example").await;
    assert_eq!(email_of(&db, "crawler").await, "crawler@example.com");
}

#[tokio::test]
#[serial]
async fn a_staff_member_cannot_reset_or_delete_a_superuser() {
    let (base, db) = spawn().await;
    let manager = login(&base, MANAGER, MANAGER_PASSWORD).await;
    let su = seed_superuser_id_str();

    post_action(&manager, &base, &su, "reset-password").await;
    assert_eq!(
        count(&db, "SELECT COUNT(*) FROM eihwaz_reset_tokens").await,
        0,
        "no reset token for the superuser"
    );

    post_action(&manager, &base, &su, "delete").await;
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM eihwaz_users WHERE is_superuser = 1"
        )
        .await,
        1,
        "the superuser is still there"
    );
}

#[tokio::test]
#[serial]
async fn a_bulk_action_touching_a_superuser_is_refused_as_a_whole() {
    let (base, db) = spawn().await;
    let manager = login(&base, MANAGER, MANAGER_PASSWORD).await;
    let ids = format!("{},{}", seed_superuser_id_str(), pk(61));
    let list = format!("{base}{ADMIN_PREFIX}/users/list");
    let token = csrf(&manager, &list).await;
    let _ = manager
        .post(format!("{base}{ADMIN_PREFIX}/users/bulk"))
        .form(&[
            ("ids", ids.as_str()),
            ("bulk_action", "delete"),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(
        count(
            &db,
            "SELECT COUNT(*) FROM eihwaz_users WHERE username IN ('crawler', 'member')"
        )
        .await,
        2,
        "nothing deleted, not even the ordinary account"
    );
}

#[tokio::test]
#[serial]
async fn a_staff_member_can_still_read_a_superuser_and_change_an_ordinary_account() {
    let (base, db) = spawn().await;
    let manager = login(&base, MANAGER, MANAGER_PASSWORD).await;
    let detail = format!(
        "{base}{ADMIN_PREFIX}/users/{}/detail",
        seed_superuser_id_str()
    );
    assert_eq!(manager.get(&detail).send().await.unwrap().status(), 200);

    let member = pk(61).to_string();
    edit_email(&manager, &base, &member, "member", "renamed@example.com").await;
    assert_eq!(email_of(&db, "member").await, "renamed@example.com");
}

#[tokio::test]
#[serial]
async fn a_superuser_can_change_a_superusers_account() {
    let (base, db) = spawn().await;
    let su_client = login(
        &base,
        admin_server::SUPERUSER_USERNAME,
        admin_server::SUPERUSER_PASSWORD,
    )
    .await;
    let su = seed_superuser_id_str();
    edit_email(&su_client, &base, &su, "crawler", "new@example.com").await;
    assert_eq!(email_of(&db, "crawler").await, "new@example.com");
}
