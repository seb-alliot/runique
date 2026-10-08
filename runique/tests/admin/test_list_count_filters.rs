//! The list's total — hence its pages — counts what the list shows: the count
//! receives the same column filters as the list.

use crate::helpers::admin_server::{
    ADMIN_PREFIX, build_admin_app_with_registry, login_as_superuser,
};
use runique::admin::helper::resource_entry::{CountFn, FormBuilder, ListFn, ResourceEntry};
use runique::admin::registry::AdminRegistry;
use runique::admin::resource::AdminResource;
use serial_test::serial;
use std::sync::{Arc, Mutex};

type Seen = Arc<Mutex<Vec<Vec<(String, String)>>>>;

/// A resource whose count records the column filters it receives.
fn recording_registry(seen: Seen) -> AdminRegistry {
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "color": "red" })]) })
    });
    let count_fn: CountFn = Arc::new(move |_, _, column_filters, _| {
        seen.lock().unwrap().push(column_filters);
        Box::pin(async { Ok(1) })
    });
    let entry = ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn);
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

#[tokio::test]
#[serial]
async fn the_count_receives_the_column_filters_of_the_list() {
    let seen: Seen = Arc::default();
    let (router, _db) = build_admin_app_with_registry(recording_registry(seen.clone())).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    let client = login_as_superuser(&base).await;
    let resp = client
        .get(format!("{base}{ADMIN_PREFIX}/items/list?filter_color=red"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let seen = seen.lock().unwrap();
    assert!(
        seen.iter()
            .any(|f| f == &vec![("color".to_string(), "red".to_string())]),
        "the list's count got: {seen:?}"
    );
}

/// A list of 30 `items` (one row returned per page, enough for the links).
fn thirty_items_registry() -> AdminRegistry {
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "7", "color": "red" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(30) }));
    let entry = ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn);
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

/// The links of a list page keep its whole state — sort, direction, search,
/// filter and page — so coming back from a detail lands on the same view.
/// Written from cargo-mutants survivors (2026-10-07): the query strings
/// `handle_list` builds were never read back.
#[tokio::test]
#[serial]
async fn list_links_keep_sort_search_filter_and_page() {
    let (router, _db) = build_admin_app_with_registry(thirty_items_registry()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;

    let html = client
        .get(format!(
            "{base}{ADMIN_PREFIX}/items/list?sort_by=color&sort_dir=desc&search=x&filter_color=red&page=2"
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let expected =
        urlencoding::encode("sort_by=color&sort_dir=desc&search=x&filter_color=red&page=2");
    assert!(
        html.contains(&format!("return_qs={expected}")),
        "the detail link carries the full state"
    );

    // An unknown sort column never reaches the links.
    let html = client
        .get(format!(
            "{base}{ADMIN_PREFIX}/items/list?sort_by=secret&page=1"
        ))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        !html.contains("sort_by=secret"),
        "an unknown column is dropped"
    );
    assert!(
        !html.contains("return_qs=page"),
        "page 1 is the default: not written"
    );
}

type SeenParams = Arc<Mutex<Vec<(Option<String>, bool, Option<String>)>>>;

/// Written from cargo-mutants survivors (2026-10-08): the list function gets
/// the sort column, the direction and the search from the query string; an
/// empty value means "none", and anything but `desc` means ascending.
#[tokio::test]
#[serial]
async fn the_list_receives_sort_direction_and_search() {
    use runique::admin::helper::SortDir;
    let seen: SeenParams = Arc::default();
    let record = seen.clone();
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
    let list_fn: ListFn = Arc::new(move |_, p| {
        record.lock().unwrap().push((
            p.sort_by.clone(),
            matches!(p.sort_dir, SortDir::Desc),
            p.search.clone(),
        ));
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "color": "red" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
    let mut registry = AdminRegistry::new();
    registry.register(
        ResourceEntry::new(AdminResource::new("items", "M", "F", "Items"), form_builder)
            .with_list_fn(list_fn)
            .with_count_fn(count_fn),
    );
    let (router, _db) = build_admin_app_with_registry(registry).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;

    for qs in [
        "sort_by=color&sort_dir=desc&search=x",
        "sort_by=&sort_dir=asc&search=",
    ] {
        let resp = client
            .get(format!("{base}{ADMIN_PREFIX}/items/list?{qs}"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }
    let seen = seen.lock().unwrap();
    assert_eq!(
        *seen,
        vec![
            (Some("color".to_string()), true, Some("x".to_string())),
            (None, false, None),
        ]
    );
}

/// `items` with two sidebar filters, 2 values per page: `color` has 5
/// distinct values (3 pages), `size` 4 (2 pages).
fn sidebar_registry() -> AdminRegistry {
    use runique::admin::helper::resource_entry::FilterFn;
    use runique::admin::resource::DisplayConfig;
    let form_builder: FormBuilder = Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
    let list_fn: ListFn = Arc::new(|_, _| {
        Box::pin(async { Ok(vec![serde_json::json!({ "id": "1", "color": "red" })]) })
    });
    let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
    let filter_fn: FilterFn = Arc::new(|_, _| {
        Box::pin(async {
            Ok([
                ("color".to_string(), (vec!["red".into(), "blue".into()], 5)),
                ("size".to_string(), (vec!["s".into(), "m".into()], 4)),
            ]
            .into())
        })
    });
    let resource = AdminResource::new("items", "M", "F", "Items").display(
        DisplayConfig::new().list_filter(vec![("color", "Color", 2), ("size", "Size", 2)]),
    );
    let entry = ResourceEntry::new(resource, form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_filter_fn(filter_fn);
    let mut registry = AdminRegistry::new();
    registry.register(entry);
    registry
}

/// The sidebar filters page on their own: each one's arrows move only its
/// page and keep the other's, page 0 is never written, the first page has no
/// "previous" and the last no "next". Written from cargo-mutants survivors
/// (2026-10-08): handle_list.rs:175, 216, 217, 222, 227.
#[tokio::test]
#[serial]
async fn sidebar_filters_page_independently() {
    let (router, _db) = build_admin_app_with_registry(sidebar_registry()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;
    let page = |qs: &'static str| {
        let client = client.clone();
        let url = format!("{base}{ADMIN_PREFIX}/items/list?{qs}");
        async move { client.get(url).send().await.unwrap().text().await.unwrap() }
    };
    let arrows = |html: &str| html.matches("admin-filter__page-btn").count();
    let fp_links = |html: &str| -> Vec<String> {
        html.split("href=\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .filter(|h| h.contains("fp_"))
            .map(str::to_string)
            .collect()
    };
    let disabled = |html: &str| html.matches("admin-filter__page-btn disabled").count();

    // color on its middle page, size on its last.
    let html = page("fp_color=1&fp_size=1").await;
    assert!(html.contains("2 / 3") && html.contains("2 / 2"), "{html}");
    assert!(
        html.contains(r#"href="?fp_size=1&page=1""#),
        "color ‹ : page 0 not written, size kept: {:?}",
        fp_links(&html)
    );
    assert!(
        html.contains(r#"href="?fp_size=1&amp;fp_color=2&page=1""#),
        "color › : next page, size kept"
    );
    assert!(
        html.contains(r#"href="?fp_color=1&page=1""#),
        "size ‹ : color kept"
    );
    assert_eq!(
        (arrows(&html), disabled(&html)),
        (4, 1),
        "size is on its last page: no ›"
    );
    assert!(
        html.contains("&amp;fp_color=1"),
        "the other links carry the filter pages"
    );

    // Both on their first page: no ‹ anywhere, page 0 written nowhere.
    let html = page("fp_color=0&fp_size=0").await;
    assert!(html.contains("1 / 3") && html.contains("1 / 2"), "{html}");
    assert_eq!(disabled(&html), 2, "first pages: no ‹");
    assert!(html.contains(r#"href="?fp_color=1&page=1""#), "color ›");
    assert!(html.contains(r#"href="?fp_size=1&page=1""#), "size ›");
    assert!(
        !html.contains("fp_color=0") && !html.contains("fp_size=0"),
        "page 0 is the default"
    );
}

/// Three views of the same rows: every column, some excluded, some picked
/// with their own labels.
fn columns_registry() -> AdminRegistry {
    use runique::admin::resource::DisplayConfig;
    let mut registry = AdminRegistry::new();
    for (key, display) in [
        ("all_cols", DisplayConfig::new()),
        (
            "some_cols",
            DisplayConfig::new().columns_exclude(vec!["secret"]),
        ),
        (
            "picked_cols",
            DisplayConfig::new()
                .columns_include(vec![("can_read", "Own label"), ("color", "Colour")]),
        ),
    ] {
        let form_builder: FormBuilder =
            Arc::new(|_, _, _, _, _, _| Box::pin(async { unreachable!() }));
        let list_fn: ListFn = Arc::new(|_, _| {
            Box::pin(async {
                Ok(vec![serde_json::json!({
                    "id": "1", "color": "red", "secret": "s3", "can_read": "yes",
                    "password_hash": "$argon2id$x"
                })])
            })
        });
        let count_fn: CountFn = Arc::new(|_, _, _, _| Box::pin(async { Ok(1) }));
        registry.register(
            ResourceEntry::new(
                AdminResource::new(key, "M", "F", key).display(display),
                form_builder,
            )
            .with_list_fn(list_fn)
            .with_count_fn(count_fn),
        );
    }
    registry
}

/// The list shows the configured columns under the right labels: an explicit
/// label wins over the translation, a column without one keeps its name, and
/// a password column never shows. Written from cargo-mutants survivors
/// (2026-10-08): handle_list.rs:345, 352, 355 (resolve_columns).
#[tokio::test]
#[serial]
async fn list_columns_and_labels_follow_the_display_config() {
    let (router, _db) = build_admin_app_with_registry(columns_registry()).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = login_as_superuser(&base).await;
    let labels = |key: &'static str| {
        let client = client.clone();
        let url = format!("{base}{ADMIN_PREFIX}/{key}/list");
        async move {
            let html = client.get(url).send().await.unwrap().text().await.unwrap();
            let mut found: Vec<String> = html
                .split("admin-table__td-data")
                .skip(1)
                .filter_map(|s| s.split("data-label=\"").nth(1)?.split('"').next())
                .map(str::to_string)
                .collect();
            found.sort();
            found
        }
    };
    let read = runique::utils::trad::t("permission.col.can_read").into_owned();
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };

    assert_eq!(
        labels("all_cols").await,
        sorted(vec![read.clone(), "color".into(), "secret".into()]),
        "translated when a translation exists, the name otherwise, never the password"
    );
    assert_eq!(
        labels("some_cols").await,
        sorted(vec![read.clone(), "color".into()]),
        "an excluded column is gone, the others stay"
    );
    assert_eq!(
        labels("picked_cols").await,
        sorted(vec!["Colour".into(), "Own label".into()]),
        "explicit labels win over the translation"
    );
}
