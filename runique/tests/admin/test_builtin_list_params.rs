//! The built-in resources (users, groups, rights) only sort by the columns
//! their list shows, and declare no sidebar filter: any other `sort_by` or
//! `filter_*` from the query string is ignored, never turned into SQL.

use crate::helpers::admin_server::{ADMIN_PREFIX, admin_server_addr, login_as_superuser};
use serial_test::serial;

const SEEDED_EMAIL: &str = "crawler@example.com";

async fn list(path_and_query: &str) -> (u16, String) {
    let base = format!("http://{}", admin_server_addr());
    let client = login_as_superuser(&base).await;
    let resp = client
        .get(format!("{base}{ADMIN_PREFIX}/{path_and_query}"))
        .send()
        .await
        .expect("GET list");
    (
        resp.status().as_u16(),
        resp.text().await.unwrap_or_default(),
    )
}

// Before: `filter_email=…` became `WHERE email = …` and hid the seeded account;
// `filter_password=…` would have compared password hashes the same way.
#[tokio::test]
#[serial]
async fn an_undeclared_filter_is_ignored() {
    for filter in ["filter_email=nobody@example.com", "filter_password=x"] {
        let (status, body) = list(&format!("users/list?{filter}")).await;
        assert_eq!(status, 200, "{filter}");
        assert!(body.contains(SEEDED_EMAIL), "{filter} filtered the list");
    }
}

// Before: `ORDER BY "id"` on `eihwaz_groupes_droits`, which has no `id` column —
// refused by Postgres and MariaDB (a 500). SQLite, which runs this test, takes an
// unknown quoted identifier for a string: here the test only guards the 200.
#[tokio::test]
#[serial]
async fn sorting_by_a_column_that_is_not_shown_is_ignored() {
    let (status, _) = list("droits/list?sort_by=id").await;
    assert_eq!(status, 200);
    let (status, body) = list("users/list?sort_by=password&sort_dir=desc").await;
    assert_eq!(status, 200);
    assert!(body.contains(SEEDED_EMAIL));
}

#[tokio::test]
#[serial]
async fn sorting_by_a_shown_column_still_works() {
    for path in [
        "users/list?sort_by=username&sort_dir=desc",
        "groupes/list?sort_by=nom",
        "droits/list?sort_by=resource_key",
    ] {
        let (status, _) = list(path).await;
        assert_eq!(status, 200, "{path}");
    }
}
