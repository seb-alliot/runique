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
    created_at TEXT,
    updated_at TEXT
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
    assert!(created.created_at.is_some() && created.updated_at.is_some());

    // Back-date both, then edit: only `updated_at` moves.
    let mut row = created.into_active_model();
    row.created_at = Set(Some(old()));
    row.updated_at = Set(Some(old()));
    let backdated = row.update(&conn).await.expect("update");
    assert_eq!(
        backdated.created_at,
        Some(old()),
        "auto_now is set on insert only"
    );
    assert!(
        backdated.updated_at > Some(old()),
        "auto_now_update on every save"
    );

    let mut row = backdated.into_active_model();
    row.label = Set("b".into());
    let edited = row.update(&conn).await.expect("update");
    assert_eq!(edited.created_at, Some(old()));
    assert!(edited.updated_at > Some(old()));

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
        created_at: Set(Some(old())),
        ..Default::default()
    }
    .insert(&conn)
    .await
    .expect("insert");
    assert_eq!(row.created_at, Some(old()));
}
