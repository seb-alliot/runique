//! `auto_now` / `auto_now_update` are kept by the entity itself
//! (`ActiveModelBehavior::before_save`), the same on every engine — they used
//! to rely on a Postgres trigger (and `ON UPDATE` on MySQL), SQLite had nothing.
use crate::helpers::db;
use runique::sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, IntoActiveModel};

mod stamps {
    use runique::prelude::*;

    model! {
        Stamp,
        table: "auto_now_stamps",
        pk: id => i32,
        {
            label: text [required],
            created_at: datetime [auto_now],
            updated_at: datetime [auto_now_update],
        }
    }
}

const DDL: &str = "CREATE TABLE auto_now_stamps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)";

fn old() -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(2000, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

#[tokio::test]
async fn an_insert_stamps_both_and_an_update_only_updated_at() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let created = stamps::ActiveModel {
        label: Set("a".into()),
        ..Default::default()
    }
    .insert(&conn)
    .await
    .expect("insert");
    assert!(created.created_at > old() && created.updated_at > old());

    // Back-date both, then edit: only `updated_at` moves.
    let mut row = created.into_active_model();
    row.created_at = Set(old());
    row.updated_at = Set(old());
    let backdated = row.update(&conn).await.expect("update");
    assert_eq!(
        backdated.created_at,
        old(),
        "auto_now is set on insert only"
    );
    assert!(
        backdated.updated_at > old(),
        "auto_now_update on every save"
    );

    let mut row = backdated.into_active_model();
    row.label = Set("b".into());
    let edited = row.update(&conn).await.expect("update");
    assert_eq!(edited.created_at, old());
    assert!(edited.updated_at > old());

    let stored = stamps::Entity::find_by_id(edited.id)
        .one(&conn)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.label, "b");
}

#[tokio::test]
async fn an_explicit_creation_date_is_kept() {
    let conn = db::fresh_db_with_schema(DDL).await;
    let row = stamps::ActiveModel {
        label: Set("seeded".into()),
        created_at: Set(old()),
        ..Default::default()
    }
    .insert(&conn)
    .await
    .expect("insert");
    assert_eq!(row.created_at, old());
}

mod marks {
    use runique::prelude::*;

    model! {
        Mark,
        table: "auto_now_marks",
        pk: id => i32,
        {
            day: date [auto_now],
            at: time [auto_now],
            tz: timestamp_tz [auto_now],
        }
    }
}

// `auto_now` alone (no `auto_now_update`), on the other date types: the NOT
// NULL columns make the insert fail if one of them isn't stamped.
#[tokio::test]
async fn auto_now_alone_stamps_a_date_a_time_and_a_timestamp_tz() {
    let conn = db::fresh_db_with_schema(
        "CREATE TABLE auto_now_marks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            day TEXT NOT NULL,
            at TEXT NOT NULL,
            tz TEXT NOT NULL
        )",
    )
    .await;
    let before = chrono::Utc::now();
    let row = marks::ActiveModel::default()
        .insert(&conn)
        .await
        .expect("every column stamped");
    assert!(row.tz >= before - chrono::Duration::seconds(1));
    assert!(row.day >= before.date_naive());
}
