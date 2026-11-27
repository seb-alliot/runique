//! Alone in its process: the mailer is process-wide, and the admin's other
//! tests set one. Here none is ever set, and the environment gives none.
#![cfg(feature = "admin")]
#[allow(dead_code)]
#[path = "helpers/mod.rs"]
mod helpers;

use axum::extract::Path;
use axum::routing::get;
use helpers::db;
use helpers::framework_tables::{
    GROUPES_DDL, GROUPES_DROITS_DDL, HISTORY_DDL, RESET_TOKENS_DDL, SESSIONS_DDL, USERS_DDL,
    USERS_GROUPES_DDL,
};
use helpers::pk::{pk, pk_sql_literal};
use runique::app::RuniqueApp;
use runique::auth::{BuiltinUserEntity, login};
use runique::config::RuniqueConfig;
use runique::flash::Message;
use runique::http::request::Request;

const SUPERUSER: u32 = 1;

async fn sign_in(Path(n): Path<u32>, req: Request) -> &'static str {
    let user = BuiltinUserEntity::find_by_id(&req.engine.db, pk(n))
        .await
        .expect("seeded account");
    login(&req.session, &user, None, false)
        .await
        .expect("login");
    "ok"
}

async fn flashes(notices: Message) -> String {
    notices
        .get_all()
        .await
        .into_iter()
        .map(|m| m.content)
        .collect::<Vec<_>>()
        .join("|")
}

/// The account the test tries to create: its username, or `none`.
async fn created_account(req: Request) -> String {
    BuiltinUserEntity::find_by_email(&req.engine.db, "newbie@example.com")
        .await
        .map_or("none".to_string(), |user| user.username)
}

/// An app with the admin and no mailer: neither set by code nor given by the
/// environment.
async fn spawn() -> String {
    let dbc = db::fresh_db().await;
    for ddl in [
        USERS_DDL,
        GROUPES_DDL,
        GROUPES_DROITS_DDL,
        USERS_GROUPES_DDL,
        HISTORY_DDL,
        RESET_TOKENS_DDL,
        SESSIONS_DDL,
    ] {
        db::exec(&dbc, ddl).await;
    }
    // Never verified: the test signs in through its own route, not a form.
    db::exec(
        &dbc,
        &format!(
            "INSERT INTO eihwaz_users \
             (id, username, email, password, is_active, is_admin, is_superuser, activated_at) \
             VALUES ({}, 'carol', 'carol@example.com', 'seeded-never-verified', 1, 0, 1, '2026-01-01 00:00:00')",
            pk_sql_literal(SUPERUSER)
        ),
    )
    .await;

    let mut config = RuniqueConfig::from_env();
    config.dev = true;
    // The `.env` is loaded by now: whatever this machine's says of a mailer
    // is taken out, so the test reads the same everywhere.
    for key in ["EMAIL_BACKEND", "SMTP_HOST", "SMTP_USER", "SMTP_PASS"] {
        // Safe here: this binary's only test, nothing else reads the
        // environment while it runs.
        unsafe { std::env::remove_var(key) }
    }
    let routes = runique::urlpatterns! {
        "/sign-in/{n}" => get(sign_in), name = "no_mailer_sign_in",
        "/flashes" => get(flashes), name = "no_mailer_flashes",
        "/probe/account" => get(created_account), name = "no_mailer_created_account",
    };
    let app = RuniqueApp::builder(config)
        .with_database(dbc)
        .routes(routes)
        .static_files(|s| s.enabled(false))
        .with_admin(|a| a.path("/secret-admin").site(|s| s))
        .build()
        .await
        .expect("app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app.router).await.unwrap() });
    format!("http://{addr}")
}

fn location(resp: &reqwest::Response) -> Option<&str> {
    resp.headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
}

async fn text(client: &reqwest::Client, url: String) -> String {
    client.get(url).send().await.unwrap().text().await.unwrap()
}

fn hidden_value(body: &str, name: &str) -> Option<String> {
    let after_name = &body[body.find(&format!("name=\"{name}\""))?..];
    let start = after_name.find("value=\"")? + "value=\"".len();
    let end = after_name[start..].find('"')? + start;
    Some(after_name[start..end].to_string())
}

/// The staff never holds a password: an account created from the admin is
/// opened by the link mailed to its owner. Without a mailer that link can't
/// leave, so the admin creates nothing rather than an account nobody opens.
#[tokio::test]
async fn without_a_mailer_the_admin_creates_no_account() {
    assert!(
        !runique::utils::mailer_configured(),
        "this binary must start without a mailer"
    );
    let base = spawn().await;
    let staff = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    assert_eq!(
        text(&staff, format!("{base}/sign-in/{SUPERUSER}")).await,
        "ok"
    );
    let list = Some("/secret-admin/users/list");
    let url = format!("{base}/secret-admin/users/create");

    // The page itself: no form to fill in vain.
    let page = staff.get(&url).send().await.unwrap();
    assert_eq!(page.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(location(&page), list);
    let notice = text(&staff, format!("{base}/flashes")).await;
    assert!(notice.contains("configur"), "the reason is said: {notice}");

    // The form sent anyway, with a token of the admin: nothing is written.
    let dashboard = text(&staff, format!("{base}/secret-admin/")).await;
    let token = hidden_value(&dashboard, "csrf_token").unwrap_or_default();
    let sent = staff
        .post(&url)
        .form(&[
            ("csrf_token", token.as_str()),
            ("runique_form", "admin_create"),
            ("username", "newbie"),
            ("email", "newbie@example.com"),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(sent.status(), reqwest::StatusCode::SEE_OTHER);
    assert_eq!(location(&sent), list);
    assert_eq!(
        text(&staff, format!("{base}/probe/account")).await,
        "none",
        "no account without a link to open it"
    );
}
