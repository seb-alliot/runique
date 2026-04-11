//! Tests d'intégration — permissions chargées depuis PostgreSQL (Docker requis).
//!
//! Nécessite : `DATABASE_URL_PG` dans `.env.test` et container actif.
//! Si absent, les tests sont ignorés silencieusement.
//!
//! Ce que ces tests vérifient :
//! - `pull_groupes_db` charge correctement les groupes et permissions depuis la DB
//! - `refresh_cache_for_user` met le cache à jour
//! - Après `clear_cache`, `get_permissions` retourne None

use crate::helpers::db_postgres;

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

async fn setup_rbac_tables(db: &runique::sea_orm::DatabaseConnection) {
    use runique::admin::table_admin::migrations_table::{
        create_eihwaz_groupes_droits_table, create_eihwaz_groupes_table,
        create_eihwaz_users_groupes_table,
    };
    use runique::sea_orm::ConnectionTrait;
    use runique::sea_orm::sea_query::PostgresQueryBuilder;

    let _ = db
        .execute_unprepared(&create_eihwaz_groupes_table().to_string(PostgresQueryBuilder))
        .await;
    let _ = db
        .execute_unprepared(&create_eihwaz_groupes_droits_table().to_string(PostgresQueryBuilder))
        .await;
    let _ = db
        .execute_unprepared(&create_eihwaz_users_groupes_table().to_string(PostgresQueryBuilder))
        .await;
}

async fn teardown(db: &runique::sea_orm::DatabaseConnection) {
    use runique::sea_orm::ConnectionTrait;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_users_groupes")
        .await;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_groupes_droits")
        .await;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_groupes")
        .await;
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_pull_groupes_db_retourne_permissions() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;

    // Insère un groupe avec une permission
    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (1, 'moderateur')",
    )
    .await;
    db_postgres::exec(&db, "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_read, can_create, can_update, can_delete, can_update_own, can_delete_own) VALUES (1, 'articles', true, false, false, false, false, false)").await;
    // Lie l'user 42 au groupe
    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES (42, 1)",
    )
    .await;

    let groupes = runique::admin::permissions::pull_groupes_db(&db, 42).await;

    assert_eq!(groupes.len(), 1);
    assert_eq!(groupes[0].nom, "moderateur");
    assert_eq!(groupes[0].permissions.len(), 1);
    assert_eq!(groupes[0].permissions[0].resource_key, "articles");
    assert!(groupes[0].permissions[0].can_read);
    assert!(!groupes[0].permissions[0].can_create);

    teardown(&db).await;
}

#[tokio::test]
async fn test_pull_groupes_db_multi_ressources() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;

    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (2, 'admin')",
    )
    .await;
    db_postgres::exec(&db, "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_read, can_create, can_update, can_delete, can_update_own, can_delete_own) VALUES (2, 'articles', true, true, true, true, false, false)").await;
    db_postgres::exec(&db, "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_read, can_create, can_update, can_delete, can_update_own, can_delete_own) VALUES (2, 'users', true, false, false, false, false, false)").await;
    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES (43, 2)",
    )
    .await;

    let groupes = runique::admin::permissions::pull_groupes_db(&db, 43).await;

    assert_eq!(groupes.len(), 1);
    assert_eq!(groupes[0].permissions.len(), 2);

    let keys: Vec<&str> = groupes[0]
        .permissions
        .iter()
        .map(|p| p.resource_key.as_str())
        .collect();
    assert!(keys.contains(&"articles"));
    assert!(keys.contains(&"users"));

    teardown(&db).await;
}

#[tokio::test]
async fn test_refresh_cache_puis_clear() {
    use runique::admin::permissions::refresh_cache_for_user;
    use runique::middleware::auth::permissions_cache::{clear_cache, get_permissions};

    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;

    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_groupes (id, nom) VALUES (3, 'editeur')",
    )
    .await;
    db_postgres::exec(&db, "INSERT INTO eihwaz_groupes_droits (groupe_id, resource_key, can_read, can_create, can_update, can_delete, can_update_own, can_delete_own) VALUES (3, 'blog', true, true, false, false, false, false)").await;
    db_postgres::exec(
        &db,
        "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) VALUES (44, 3)",
    )
    .await;

    // Charge en cache
    refresh_cache_for_user(&db, 44).await;
    assert!(get_permissions(44).is_some());

    // Invalide tout le cache (simule une modif admin)
    clear_cache();
    assert!(get_permissions(44).is_none());

    teardown(&db).await;
}

#[tokio::test]
async fn test_pull_groupes_db_user_sans_groupe() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;

    // User 99 n'appartient à aucun groupe
    let groupes = runique::admin::permissions::pull_groupes_db(&db, 99).await;
    assert!(groupes.is_empty());

    teardown(&db).await;
}
