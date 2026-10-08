//! Create and edit handlers on a resource like the generated ones. Written
//! from cargo-mutants survivors (2026-10-08): admin_main/handle_crud.rs.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, build_admin_app_with_registry, login_as_superuser,
};
use crate::utils::env::{del_env, set_env};
use runique::admin::helper::DynForm;
use runique::admin::helper::resource_entry::{
    CountFn, CreateFn, FormBuilder, GetFn, ListFn, ResourceEntry, UpdateFn,
};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use runique::forms::Forms;
use runique::forms::base::FormField as _;
use runique::forms::fields::TextField;
use runique::forms::fields::file::FileField;
use runique::prelude::async_trait;
use runique::utils::aliases::ADb;
use serial_test::serial;
use std::sync::{Arc, Mutex};

/// Valid whatever it holds: these tests are about what the handler does next.
struct AlwaysValid(Forms);

#[async_trait]
impl DynForm for AlwaysValid {
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

type Calls = Arc<Mutex<Vec<&'static str>>>;

/// A `title` text field and an `attachment` upload, filled from the request,
/// with one script asset.
fn form_builder() -> FormBuilder {
    Arc::new(|_, _, data, tera, csrf, _| {
        Box::pin(async move {
            let mut form = Forms::new(&csrf);
            form.set_renderer(runique::forms::renderer::FormRenderer::new(tera));
            form.add_js(&["js/widget.js"]);
            let mut title = TextField::text("title");
            title.set_value(data.get("title").map_or("", String::as_str));
            let mut attachment = FileField::any("attachment");
            attachment.set_value(data.get("attachment").map_or("", String::as_str));
            form.field(&title);
            form.field(&attachment);
            Box::new(AlwaysValid(form)) as Box<dyn DynForm>
        })
    })
}

/// `docs`: row 1 holds a path in its text field and an upload. `create`
/// refuses `dup` (unique violation) and `boom` (any other error).
fn docs_registry(calls: Calls, custom_edit_form: bool) -> AdminRegistry {
    let list_fn: ListFn = Arc::new(|_, _| Box::pin(async { Ok(vec![]) }));
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(0) }));
    let get_fn: GetFn = Arc::new(|_, _| {
        Box::pin(async {
            Ok(Some(serde_json::json!({
                "id": "1", "title": "docs/keep.txt", "attachment": "docs/old.pdf"
            })))
        })
    });
    // Database errors (not `Custom`, which is the resource's own refusal).
    let create_fn: CreateFn = Arc::new(|_, data| {
        Box::pin(async move {
            use runique::sea_orm::{DbErr, RuntimeErr};
            match data.get("title").map(String::as_str) {
                Some("dup") => Err(DbErr::Exec(RuntimeErr::Internal(
                    "UNIQUE constraint failed: docs.title".into(),
                ))),
                Some("boom") => Err(DbErr::Exec(RuntimeErr::Internal("disk full".into()))),
                _ => Ok(()),
            }
        })
    });
    let record = |name: &'static str, calls: Calls| -> UpdateFn {
        Arc::new(move |_, _, _| {
            calls.lock().unwrap().push(name);
            Box::pin(async { Ok(()) })
        })
    };
    let mut entry =
        ResourceEntry::new(AdminResource::new("docs", "M", "F", "Docs"), form_builder())
            .with_list_fn(list_fn)
            .with_count_fn(count_fn)
            .with_get_fn(get_fn)
            .with_create_fn(create_fn)
            .with_update_fn(record("update", calls.clone()))
            .with_partial_update_fn(record("partial", calls));
    if custom_edit_form {
        entry = entry.with_edit_form_builder(form_builder());
    }
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

async fn serve(registry: AdminRegistry) -> (String, reqwest::Client) {
    let (router, _db) = build_admin_app_with_registry(registry).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;
    (base, client)
}

async fn post(
    base: &str,
    client: &reqwest::Client,
    path: &str,
    fields: &[(&str, &str)],
) -> reqwest::Response {
    let page = format!("{base}{ADMIN_PREFIX}/docs/list");
    let token = client.get(&page).send().await.unwrap().headers()["x-csrf-token"]
        .to_str()
        .unwrap()
        .to_string();
    let mut form: Vec<(&str, &str)> = fields.to_vec();
    form.push(("csrf_token", &token));
    client
        .post(format!("{base}{ADMIN_PREFIX}/docs/{path}"))
        .form(&form)
        .send()
        .await
        .unwrap()
}

/// Replacing an upload removes the old file; keeping it, clearing it, or a
/// text field that happens to hold a path never removes anything.
#[tokio::test]
#[serial]
async fn edit_removes_only_a_replaced_upload() {
    let media = std::env::temp_dir().join(format!("rq_crud_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(media.join("docs")).unwrap();
    std::fs::write(media.join("docs/old.pdf"), "old").unwrap();
    std::fs::write(media.join("docs/keep.txt"), "keep").unwrap();
    set_env("MEDIA_ROOT", media.to_str().unwrap());

    let (base, client) = serve(docs_registry(Arc::default(), false)).await;
    for attachment in ["docs/old.pdf", ""] {
        post(
            &base,
            &client,
            "1/edit",
            &[("title", "x"), ("attachment", attachment)],
        )
        .await;
        assert!(
            media.join("docs/old.pdf").exists(),
            "kept with {attachment:?}"
        );
    }
    post(
        &base,
        &client,
        "1/edit",
        &[("title", "x"), ("attachment", "docs/new.pdf")],
    )
    .await;
    let old_gone = !media.join("docs/old.pdf").exists();
    let keep_kept = media.join("docs/keep.txt").exists();

    del_env("MEDIA_ROOT");
    let _ = std::fs::remove_dir_all(&media);
    assert!(old_gone, "the replaced upload is removed");
    assert!(
        keep_kept,
        "a text field's old value is never treated as a file"
    );
}

/// A narrower custom edit form saves through the partial update (only what it
/// shows); the default form through the full update.
#[tokio::test]
#[serial]
async fn edit_uses_the_partial_update_behind_a_custom_edit_form() {
    for (custom, expected) in [(true, "partial"), (false, "update")] {
        let calls: Calls = Arc::default();
        let (base, client) = serve(docs_registry(calls.clone(), custom)).await;
        post(
            &base,
            &client,
            "1/edit",
            &[("title", "x"), ("attachment", "")],
        )
        .await;
        assert_eq!(
            *calls.lock().unwrap(),
            [expected],
            "custom edit form: {custom}"
        );
    }
}

/// On create, a duplicate shows the form again for correction; any other
/// database error is a server error.
#[tokio::test]
#[serial]
async fn create_redisplays_the_form_only_for_a_duplicate() {
    let (base, client) = serve(docs_registry(Arc::default(), false)).await;
    let dup = post(
        &base,
        &client,
        "create",
        &[("title", "dup"), ("attachment", "")],
    )
    .await;
    assert_eq!(dup.status(), 200);
    assert!(
        dup.text().await.unwrap().contains("name=\"title\""),
        "the form, again"
    );
    let boom = post(
        &base,
        &client,
        "create",
        &[("title", "boom"), ("attachment", "")],
    )
    .await;
    assert_eq!(boom.status(), 500);
}

/// The form's scripts carry the page's CSP nonce.
#[tokio::test]
#[serial]
async fn form_scripts_carry_the_csp_nonce() {
    let (base, client) = serve(docs_registry(Arc::default(), false)).await;
    let html = client
        .get(format!("{base}{ADMIN_PREFIX}/docs/create"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let tag = html
        .split("<script")
        .find(|s| s.contains("js/widget.js"))
        .expect("the form's script");
    assert!(tag.contains("nonce=\""), "{tag}");
}
