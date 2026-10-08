//! Tests — the bulk edit form (`gate.rs::bulk_form`) on a resource like the
//! generated ones, whose partial update writes whatever it receives.
//!
//! Written from cargo-mutants survivors (2026-10-07): on the built-in resources
//! the filter is doubled by other guards (no password in the user edit form,
//! rights ignore their keys), so loosening it went unnoticed. Here the update
//! function records what reaches it: only the fields the bulk form offers may.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, build_admin_app_with_registry, login_as_superuser,
};
use runique::admin::helper::DynForm;
use runique::admin::helper::resource_entry::{
    CountFn, FormBuilder, ListFn, ResourceEntry, UpdateFn,
};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use runique::forms::Forms;
use runique::forms::fields::file::FileField;
use runique::forms::fields::{HiddenField, TextField};
use runique::prelude::async_trait;
use runique::utils::aliases::{ADb, StrMap};
use serial_test::serial;
use std::sync::{Arc, Mutex};

struct ItemForm(Forms);

#[async_trait]
impl DynForm for ItemForm {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await.unwrap_or(false)
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

type Written = Arc<Mutex<Vec<StrMap>>>;

/// `items`: a plain field, a unique one, a password, a hidden one and an upload.
fn items_registry(written: Written) -> AdminRegistry {
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, csrf, _| {
        Box::pin(async move {
            let mut form = Forms::new(&csrf);
            form.field(&TextField::text("name"));
            form.field(&TextField::text("slug"));
            form.field(&TextField::password("secret"));
            form.field(&HiddenField::new("internal"));
            form.field(&FileField::any("doc"));
            Box::new(ItemForm(form)) as Box<dyn DynForm>
        })
    });
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "name": "one" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
    let partial: UpdateFn = Arc::new(move |_, _, data| {
        written.lock().unwrap().push(data);
        Box::pin(async { Ok(()) })
    });
    let entry = ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_partial_update_fn(partial)
        .with_unique_fields(&["slug"]);
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

#[tokio::test]
#[serial]
async fn the_bulk_form_offers_only_plain_fields() {
    let written: Written = Arc::default();
    let (router, _db) = build_admin_app_with_registry(items_registry(written.clone())).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;

    let bulk = |field: &'static str, value: &'static str| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let list = format!("{base}{ADMIN_PREFIX}/items/list");
            let token = client.get(&list).send().await.unwrap().headers()["x-csrf-token"]
                .to_str()
                .unwrap()
                .to_string();
            client
                .post(format!("{base}{ADMIN_PREFIX}/items/bulk"))
                .form(&[
                    ("ids", "1"),
                    ("bulk_action", "update-submit"),
                    (field, value),
                    ("csrf_token", token.as_str()),
                ])
                .send()
                .await
                .unwrap();
        }
    };

    for (field, value) in [
        ("slug", "same-for-every-row"),
        ("secret", "chosen-by-the-client"),
        ("internal", "tampered"),
        ("doc", "/etc/passwd"),
    ] {
        bulk(field, value).await;
        assert!(
            written
                .lock()
                .unwrap()
                .iter()
                .all(|d| !d.contains_key(field)),
            "`{field}` must never reach the update: {:?}",
            written.lock().unwrap()
        );
    }

    bulk("name", "renamed").await;
    assert!(
        written
            .lock()
            .unwrap()
            .iter()
            .any(|d| d.get("name").map(String::as_str) == Some("renamed")),
        "an offered field is written: {:?}",
        written.lock().unwrap()
    );
}

// ── Written from cargo-mutants survivors (2026-10-08): handle_bulk.rs ─────────

/// `items` whose partial update refuses two values: `dup` as a unique
/// violation, `refuse` as the resource's own message. `with_get` gives the
/// resource a `get_fn`, so the history can show the old value.
fn history_registry(with_get: bool) -> AdminRegistry {
    use runique::admin::helper::resource_entry::GetFn;
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, csrf, _| {
        Box::pin(async move {
            let mut form = Forms::new(&csrf);
            form.field(&TextField::text("name"));
            Box::new(ItemForm(form)) as Box<dyn DynForm>
        })
    });
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "name": "old" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
    let partial: UpdateFn = Arc::new(|_, _, data| {
        Box::pin(async move {
            match data.get("name").map(String::as_str) {
                Some("dup") => Err(runique::sea_orm::DbErr::Custom(
                    "UNIQUE constraint failed: items.name".into(),
                )),
                Some("refuse") => Err(runique::sea_orm::DbErr::Custom("refused by items".into())),
                _ => Ok(()),
            }
        })
    });
    let mut entry =
        ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
            .with_list_fn(list_fn)
            .with_count_fn(count_fn)
            .with_partial_update_fn(partial);
    if with_get {
        let get_fn: GetFn = Arc::new(|_, _| {
            Box::pin(async { Ok(Some(serde_json::json!({ "id": "1", "name": "old" }))) })
        });
        entry = entry.with_get_fn(get_fn);
    }
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

async fn serve(
    registry: AdminRegistry,
) -> (
    String,
    reqwest::Client,
    runique::sea_orm::DatabaseConnection,
) {
    let (router, db) = build_admin_app_with_registry(registry).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;
    (base, client, db)
}

/// Submits the bulk edit of row 1 with `name`, returns the page it lands on.
async fn bulk_name(base: &str, client: &reqwest::Client, name: &str) -> String {
    let list = format!("{base}{ADMIN_PREFIX}/items/list");
    let token = client.get(&list).send().await.unwrap().headers()["x-csrf-token"]
        .to_str()
        .unwrap()
        .to_string();
    client
        .post(format!("{base}{ADMIN_PREFIX}/items/bulk"))
        .form(&[
            ("ids", "1"),
            ("bulk_action", "update-submit"),
            ("name", name),
            ("csrf_token", token.as_str()),
        ])
        .send()
        .await
        .unwrap();
    // The client doesn't follow redirects: the notice shows on the next page.
    client
        .get(&list)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

async fn last_summary(db: &runique::sea_orm::DatabaseConnection) -> Option<String> {
    use runique::sea_orm::{ConnectionTrait, Statement};
    let row = db
        .query_one_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT summary FROM eihwaz_history WHERE resource_key = 'items' ORDER BY id DESC LIMIT 1",
        ))
        .await
        .unwrap()?;
    row.try_get_by_index::<Option<String>>(0).unwrap()
}

/// The history of a bulk edit records the change: old and new value when the
/// resource can read the row, the new value alone otherwise.
#[tokio::test]
#[serial]
async fn bulk_edit_history_records_old_and_new_values() {
    let (base, client, db) = serve(history_registry(true)).await;
    bulk_name(&base, &client, "renamed").await;
    let summary: serde_json::Value =
        serde_json::from_str(&last_summary(&db).await.expect("a summary")).unwrap();
    assert_eq!(
        summary,
        serde_json::json!({ "name": { "old": "old", "new": "renamed" } })
    );

    let (base, client, db) = serve(history_registry(false)).await;
    bulk_name(&base, &client, "renamed").await;
    let summary: serde_json::Value =
        serde_json::from_str(&last_summary(&db).await.expect("a summary")).unwrap();
    assert_eq!(summary, serde_json::json!({ "name": { "new": "renamed" } }));
}

/// A unique violation shows the generic duplicate message; the resource's own
/// refusal shows its own message.
#[tokio::test]
#[serial]
async fn bulk_edit_errors_show_the_right_message() {
    let (base, client, _db) = serve(history_registry(false)).await;
    // As the page shows it: autoescaped (`'` in the French text).
    let unique = runique::utils::trad::t("forms.unique_constraint_violated")
        .replace('&', "&amp;")
        .replace('\'', "&#x27;");

    let html = bulk_name(&base, &client, "dup").await;
    assert!(html.contains(&unique), "duplicate → generic message");
    assert!(
        !html.contains("UNIQUE constraint failed"),
        "never the raw database text"
    );

    let html = bulk_name(&base, &client, "refuse").await;
    assert!(
        html.contains("refused by items"),
        "the resource's own message"
    );
    assert!(!html.contains(&unique));
}

/// The bulk edit page counts the selected rows, ignoring empty ids.
#[tokio::test]
#[serial]
async fn bulk_edit_page_counts_the_selected_rows() {
    let (base, client, _db) = serve(history_registry(false)).await;
    let html = client
        .get(format!("{base}{ADMIN_PREFIX}/items/bulk?ids=1,2,"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(html.contains("(2)"), "two rows, not three: {html}");
}

/// `items` whose form has a text field, a select without placeholder and a
/// select with its own.
fn select_registry() -> AdminRegistry {
    let form_builder: FormBuilder = Arc::new(|_, _, _, tera, csrf, _| {
        Box::pin(async move {
            use runique::forms::base::FormField as _;
            use runique::forms::fields::{ChoiceField, ChoiceOption};
            let mut form = Forms::new(&csrf);
            // The app's Tera, so the bulk page shows the fields.
            form.set_renderer(runique::forms::renderer::FormRenderer::new(tera));
            form.field(&TextField::text("name"));
            let options = || vec![ChoiceOption::new("a", "A"), ChoiceOption::new("b", "B")];
            form.field(&ChoiceField::new("status").choices(options()));
            let mut kind = ChoiceField::new("kind").choices(options());
            kind.set_placeholder("Pick one");
            form.field(&kind);
            Box::new(ItemForm(form)) as Box<dyn DynForm>
        })
    });
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "name": "old" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
    let partial: UpdateFn = Arc::new(|_, _, _| Box::pin(async { Ok(()) }));
    let mut registry = AdminRegistry::new();
    registry.register(
        ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
            .with_list_fn(list_fn)
            .with_count_fn(count_fn)
            .with_partial_update_fn(partial),
    );
    registry
}

/// A select without its own placeholder offers "no change"; one with a
/// placeholder keeps it, and a text field gets none. Written from
/// cargo-mutants survivors (2026-10-08): gate.rs:170.
#[tokio::test]
#[serial]
async fn bulk_edit_selects_offer_no_change() {
    let (base, client, _db) = serve(select_registry()).await;
    let html = client
        .get(format!("{base}{ADMIN_PREFIX}/items/bulk?ids=1"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(html.matches("— sans changement —").count(), 1, "{html}");
    assert!(
        html.contains(">— sans changement —</option>"),
        "on the select: {html}"
    );
    assert!(
        !html.contains(r#"placeholder="— sans changement —""#),
        "not on the text field"
    );
    assert!(html.contains("Pick one"));
}

/// `items` whose rows label differently: 1 has an empty `name` and a `title`,
/// 2 a numeric `name`, 3 a plain `name`.
fn labelled_registry() -> AdminRegistry {
    use runique::admin::helper::resource_entry::GetFn;
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, csrf, _| {
        Box::pin(async move { Box::new(ItemForm(Forms::new(&csrf))) as Box<dyn DynForm> })
    });
    let get_fn: GetFn = Arc::new(|_, id| {
        Box::pin(async move {
            Ok(match id.as_str() {
                "1" => Some(serde_json::json!({ "id": "1", "name": "", "title": "Fallback" })),
                "2" => Some(serde_json::json!({ "id": "2", "name": 42 })),
                "3" => Some(serde_json::json!({ "id": "3", "name": "Named" })),
                _ => None,
            })
        })
    });
    let mut registry = AdminRegistry::new();
    registry.register(
        ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
            .with_get_fn(get_fn),
    );
    registry
}

/// History rows are labelled from their object: an empty value is skipped for
/// the next field, a number is shown as is; a batch names each object.
/// Written from cargo-mutants survivors (2026-10-08): admin_router.rs:470-473
/// (object_label_from_json), 957, 960 (extract_display_name).
#[tokio::test]
#[serial]
async fn history_labels_come_from_the_object() {
    use runique::sea_orm::ConnectionTrait;
    let (base, client, db) = serve(labelled_registry()).await;
    db.execute_unprepared(
        "INSERT INTO eihwaz_history (resource_key, object_pk, action, user_id, username, created_at, summary, batch_id) VALUES \
         ('items', '1', 'edit', 1, 'crawler', '2026-07-30T00:00:00', NULL, NULL), \
         ('items', '2', 'edit', 1, 'crawler', '2026-07-30T00:01:00', NULL, NULL), \
         ('items', '3', 'edit', 1, 'crawler', '2026-07-30T00:02:00', NULL, 'ib'), \
         ('items', '1', 'edit', 1, 'crawler', '2026-07-30T00:03:00', NULL, 'ib')",
    )
    .await
    .unwrap();
    let page = |path: &str| {
        let url = format!("{base}{ADMIN_PREFIX}{path}");
        let client = client.clone();
        async move { client.get(url).send().await.unwrap().text().await.unwrap() }
    };

    let html = page("/history").await;
    assert!(
        html.contains("<span title=\"#1\">Fallback</span>"),
        "empty name skipped"
    );
    assert!(
        html.contains("<span title=\"#2\">42</span>"),
        "a number is a label"
    );

    let html = page("/history/batch/ib").await;
    assert!(html.contains("batch-obj-name\">Named<"), "{html}");
    assert!(
        html.contains("batch-obj-name\">Fallback<"),
        "empty name skipped"
    );
}
