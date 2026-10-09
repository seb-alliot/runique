//! The password reset targets an account: only from a resource on the accounts
//! table (the built-in `users`, or a model extending it), and the link goes to
//! the account's own email — never to a field of the object shown.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, build_admin_app_with_registry, login_as_superuser, seed_superuser_id_str,
};
use runique::admin::helper::DynForm;
use runique::admin::helper::resource_entry::{FormBuilder, GetFn, ResourceEntry};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use runique::forms::Forms;
use runique::prelude::async_trait;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection};
use runique::utils::aliases::ADb;
use runique::utils::reset_token::decrypt_email;
use serial_test::serial;
use std::sync::Arc;

struct EmptyForm(Forms);

#[async_trait]
impl DynForm for EmptyForm {
    async fn is_valid(&mut self) -> bool {
        true
    }
    async fn save(&mut self, _db: &ADb) -> Result<(), runique::sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &Forms {
        &self.0
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.0
    }
}

/// A resource on `table` whose every object shows `attacker@example.com`.
fn resource(key: &'static str, table: &'static str) -> ResourceEntry {
    let form: FormBuilder = Arc::new(|_, _, _, _, csrf, _| {
        Box::pin(async move { Box::new(EmptyForm(Forms::new(&csrf))) as Box<dyn DynForm> })
    });
    let get_fn: GetFn = Arc::new(|_, id| {
        Box::pin(async move {
            Ok(Some(
                serde_json::json!({ "id": id, "email": "attacker@example.com" }),
            ))
        })
    });
    ResourceEntry::new(AdminResource::new(key, "M", "F", key), form)
        .with_table(table)
        .with_get_fn(get_fn)
}

async fn serve() -> (String, reqwest::Client, DatabaseConnection) {
    let mut registry = AdminRegistry::new();
    registry.register(resource("contacts", "contacts"));
    registry.register(resource("profiles", "eihwaz_users"));
    let (router, db) = build_admin_app_with_registry(registry).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;
    (base, client, db)
}

async fn tokens(db: &DatabaseConnection) -> i64 {
    let row = db
        .query_one_raw(runique::sea_orm::Statement::from_string(
            db.get_database_backend(),
            "SELECT COUNT(*) AS n FROM eihwaz_reset_tokens",
        ))
        .await
        .unwrap()
        .unwrap();
    row.try_get::<i64>("", "n").unwrap()
}

/// Asks a reset for the superuser's id through `key`; returns the page the
/// action redirects to.
async fn reset(base: &str, client: &reqwest::Client, key: &str) -> String {
    let id = seed_superuser_id_str();
    let detail = format!("{base}{ADMIN_PREFIX}/{key}/{id}/detail");
    let token = client.get(&detail).send().await.unwrap().headers()["x-csrf-token"]
        .to_str()
        .unwrap()
        .to_string();
    client
        .post(format!("{base}{ADMIN_PREFIX}/{key}/{id}/reset-password"))
        .form(&[("csrf_token", token.as_str())])
        .send()
        .await
        .unwrap();
    client
        .get(&detail)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

// Another table: its id is no account's id. Nothing is created, nothing sent.
#[tokio::test]
#[serial]
async fn a_resource_on_another_table_cannot_reset_an_account() {
    let (base, client, db) = serve().await;
    let page = reset(&base, &client, "contacts").await;
    assert_eq!(
        tokens(&db).await,
        0,
        "no token for the account sharing the id"
    );
    assert!(!page.contains("/reset-password/"), "no link shown");
}

// The accounts table, through a model extending it: the reset works, and the
// link is the account's — the email the object shows plays no part.
#[tokio::test]
#[serial]
async fn an_extension_of_the_accounts_table_resets_to_the_account_email() {
    let (base, client, db) = serve().await;
    let page = reset(&base, &client, "profiles").await;
    assert_eq!(tokens(&db).await, 1);

    let link = regex::Regex::new(r#"/reset-password/([^/\s"<]+)/([^/\s"<]+)"#)
        .unwrap()
        .captures(&page)
        .unwrap_or_else(|| panic!("reset link shown: {page}"));
    let email = decrypt_email(&link[1], &link[2]).expect("email in the link");
    assert_eq!(email, "crawler@example.com", "the account's email");
}

// The button follows the server's rule: shown on the accounts table only.
#[tokio::test]
#[serial]
async fn the_reset_button_shows_on_the_accounts_table_only() {
    let (base, client, _db) = serve().await;
    let id = seed_superuser_id_str();
    for (key, shown) in [("profiles", true), ("contacts", false)] {
        let page = client
            .get(format!("{base}{ADMIN_PREFIX}/{key}/{id}/detail"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_eq!(
            page.contains(&format!("/{key}/{id}/reset-password\"")),
            shown,
            "{key}"
        );
    }
}
