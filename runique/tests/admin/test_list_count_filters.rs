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
