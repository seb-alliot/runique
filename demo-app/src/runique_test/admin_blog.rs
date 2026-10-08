//! The admin's generated data functions for `blog`, on the real model and the
//! real database, through the code `runique start` wrote in `src/admins/`.
//! Written from cargo-mutants survivors (2026-10-07): the framework's own tests
//! only mount built-in resources, never generated ones.
use crate::admins::admin::admin_register;
use crate::entities::blog::ActiveModel as BlogActiveModel;
use runique::admin::helper::{CountFn, GetFn, ListFn, ListParams, SortDir, UpdateFn};
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;
use serde_json::Value;

/// Every title starts with it, so a search on it sees only this test's rows.
const MARK: &str = "rqt-admin-blog";

/// The generated functions of the `blog` resource.
struct BlogFns {
    list: ListFn,
    count: CountFn,
    get: GetFn,
    partial: UpdateFn,
}

fn blog_fns() -> BlogFns {
    let registry = admin_register();
    let entry = registry
        .get("blog")
        .expect("the blog resource is registered");
    BlogFns {
        list: entry.list_fn.clone().expect("list_fn"),
        count: entry.count_fn.clone().expect("count_fn"),
        get: entry.get_fn.clone().expect("get_fn"),
        partial: entry.partial_update_fn.clone().expect("partial_update_fn"),
    }
}

async fn insert(db: &ADb, title: &str, email: &str) -> Result<i32, DbErr> {
    let row = BlogActiveModel {
        title: Set(format!("{MARK} {title}")),
        email: Set(email.to_string()),
        summary: Set("summary".to_string()),
        content: Set("<p>content</p>".to_string()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(row.id)
}

fn params(limit: u64, offset: u64) -> ListParams {
    ListParams {
        offset,
        limit,
        sort_by: Some("title".to_string()),
        sort_dir: SortDir::Asc,
        search: Some(MARK.to_string()),
        column_filters: Vec::new(),
        scope: None,
    }
}

fn titles(rows: &[Value]) -> Vec<String> {
    rows.iter()
        .filter_map(|r| r.get("title").and_then(Value::as_str))
        .map(|t| t.trim_start_matches(MARK).trim().to_string())
        .collect()
}

fn check(ok: bool, what: &str) -> Result<(), DbErr> {
    if ok {
        Ok(())
    } else {
        Err(DbErr::Custom(what.to_string()))
    }
}

/// The list and its count agree, page by page, with the search and a sidebar filter.
#[tokio::test]
async fn list_and_count_agree_on_search_filter_and_pages() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let BlogFns { list, count, .. } = blog_fns();
        for (title, email) in [
            ("a", "x@example.com"),
            ("b", "y@example.com"),
            ("c", "x@example.com"),
        ] {
            insert(db, title, email).await?;
        }

        let all = list(db.clone(), params(10, 0)).await?;
        check(
            titles(&all) == ["a", "b", "c"],
            "search finds the three rows, sorted",
        )?;
        let total = count(db.clone(), Some(MARK.into()), Vec::new(), None).await?;
        check(total == 3, "the count matches the list")?;

        let page2 = list(db.clone(), params(2, 2)).await?;
        check(
            titles(&page2) == ["c"],
            "offset and limit give the last page",
        )?;

        let filters = vec![("email".to_string(), "x@example.com".to_string())];
        let filtered = list(
            db.clone(),
            ListParams {
                column_filters: filters.clone(),
                ..params(10, 0)
            },
        )
        .await?;
        check(titles(&filtered) == ["a", "c"], "the email filter applies")?;
        let filtered_total = count(db.clone(), Some(MARK.into()), filters, None).await?;
        check(filtered_total == 2, "the count applies the same filter")
    })
    .await
}

/// Sorting and filtering only ever use the allow-listed columns: anything else
/// from the query string is ignored, never turned into SQL.
#[tokio::test]
async fn list_ignores_columns_outside_its_allow_list() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let list = blog_fns().list;
        insert(db, "a", "x@example.com").await?;
        insert(db, "b", "y@example.com").await?;

        // `view_count` isn't in FILTER_COLS: the filter is dropped, both rows come back.
        let rows = list(
            db.clone(),
            ListParams {
                column_filters: vec![("view_count".to_string(), "999".to_string())],
                ..params(10, 0)
            },
        )
        .await?;
        check(
            titles(&rows) == ["a", "b"],
            "a filter outside FILTER_COLS is ignored",
        )?;

        // An unknown sort column falls back to the default order, without failing.
        let rows = list(
            db.clone(),
            ListParams {
                sort_by: Some("password; DROP TABLE blog".to_string()),
                ..params(10, 0)
            },
        )
        .await?;
        check(rows.len() == 2, "an unknown sort column is ignored")
    })
    .await
}

/// A partial update (bulk, inline) writes only the keys it receives.
#[tokio::test]
async fn partial_update_only_touches_the_given_keys() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let BlogFns { partial, get, .. } = blog_fns();
        let id = insert(db, "before", "keep@example.com").await?;
        let data: StrMap = [("title".to_string(), format!("{MARK} after"))].into();
        partial(db.clone(), id.to_string(), data).await?;

        let row = get(db.clone(), id.to_string())
            .await?
            .ok_or_else(|| DbErr::Custom("row gone".into()))?;
        check(
            row.get("title").and_then(Value::as_str) == Some(&format!("{MARK} after")),
            "the given key is written",
        )?;
        check(
            row.get("email").and_then(Value::as_str) == Some("keep@example.com"),
            "a key not given is left as it was",
        )
    })
    .await
}
