//! At boot, rights on a resource the admin doesn't register are deleted: a
//! group can't keep a grant on a key that no model answers to any more (a
//! renamed or removed table), which a later model reusing the key would
//! inherit.
use crate::helpers::db::{exec, fresh_db_with_schema};
use crate::helpers::framework_tables::GROUPES_DROITS_DDL;
use axum::{Router, routing::get};
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use runique::sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serial_test::serial;

mod notes {
    use runique::prelude::*;

    model! {
        Note,
        table: "prune_notes",
        pk: id => i32,
        {
            title: text [required],
        }
    }
}

async fn resource_keys(db: &DatabaseConnection) -> Vec<String> {
    let rows = db
        .query_all_raw(Statement::from_string(
            db.get_database_backend(),
            "SELECT resource_key FROM eihwaz_groupes_droits ORDER BY resource_key",
        ))
        .await
        .unwrap();
    rows.iter()
        .map(|r| r.try_get::<String>("", "resource_key").unwrap())
        .collect()
}

async fn boot(db: DatabaseConnection, register: bool) {
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    RuniqueApp::builder(config)
        .with_database(db)
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .with_admin(|a| {
            let a = a.path("/admin");
            if register {
                a.site(|s| {
                    s.register(
                        runique::admin::refonte_admin::builder::admin_builder::ModelAdmin::<
                            notes::Entity,
                        >::new(),
                    )
                })
            } else {
                a
            }
        })
        .build()
        .await
        .unwrap();
}

async fn seeded() -> DatabaseConnection {
    let db = fresh_db_with_schema(GROUPES_DROITS_DDL).await;
    exec(
        &db,
        "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_read) VALUES \
         (1, 'prune_notes', 1), (1, 'ghost_resource', 1)",
    )
    .await;
    db
}

#[tokio::test]
#[serial]
async fn a_right_on_an_unregistered_resource_is_pruned_at_boot() {
    let db = seeded().await;
    boot(db.clone(), true).await;
    assert_eq!(resource_keys(&db).await, ["prune_notes"]);
}

/// No registered model at all: nothing to compare with, nothing is deleted.
#[tokio::test]
#[serial]
async fn without_any_registered_model_no_right_is_touched() {
    let db = seeded().await;
    boot(db.clone(), false).await;
    assert_eq!(resource_keys(&db).await, ["ghost_resource", "prune_notes"]);
}
