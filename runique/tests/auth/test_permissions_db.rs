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
use crate::helpers::pk::pk;
use serial_test::serial;

// `#[serial]` : ces tests partagent la même base Postgres Docker que
// `test_sea_migrate.rs`/`test_migrate.rs` (`migrate fresh` y droppe/recrée
// toutes les tables). Sans serialisation, un `migrate fresh` concurrent peut
// dropper `eihwaz_users_groupes` entre le CREATE et l'INSERT d'un test d'ici —
// "relation does not exist" observé en pratique. Même convention que
// `tests/db/test_postgres.rs`/`test_mariadb.rs`.

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

// Ce fichier ne peut pas supposer que `eihwaz_users` existe déjà (créée par un
// AUTRE test ailleurs) : `eihwaz_users_groupes` porte une FK dessus, et son
// CREATE échouait silencieusement (`let _ = ...`) si `eihwaz_users` manquait —
// l'échec ne se voyait qu'au prochain INSERT, en un message sans rapport avec
// la vraie cause. Ce fichier crée maintenant sa propre `eihwaz_users` et
// n'avale plus les erreurs de CREATE.
async fn setup_rbac_tables(db: &runique::sea_orm::DatabaseConnection) {
    use runique::admin::table_admin::migrations_table::{
        create_eihwaz_groupes_droits_table, create_eihwaz_groupes_table,
        create_eihwaz_users_groupes_table, create_eihwaz_users_table,
    };
    use runique::sea_orm::ConnectionTrait;
    use runique::sea_orm::sea_query::PostgresQueryBuilder;

    db.execute_unprepared(&create_eihwaz_users_table().to_string(PostgresQueryBuilder))
        .await
        .expect("create eihwaz_users");
    db.execute_unprepared(&create_eihwaz_groupes_table().to_string(PostgresQueryBuilder))
        .await
        .expect("create eihwaz_groupes");
    db.execute_unprepared(&create_eihwaz_groupes_droits_table().to_string(PostgresQueryBuilder))
        .await
        .expect("create eihwaz_groupes_droits");
    db.execute_unprepared(&create_eihwaz_users_groupes_table().to_string(PostgresQueryBuilder))
        .await
        .expect("create eihwaz_users_groupes");
}

// `CASCADE` : cette base Postgres Docker est partagée avec `test_sea_migrate.rs`,
// qui y fait tourner la vraie migration complète (FK réelles sur ces tables,
// ex. `eihwaz_users_groupes` → `eihwaz_users`). Un DROP nu échoue dès que ce
// schéma plus large traîne encore — CASCADE emporte les dépendants avec.
async fn teardown(db: &runique::sea_orm::DatabaseConnection) {
    use runique::sea_orm::ConnectionTrait;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_users_groupes CASCADE")
        .await;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_groupes_droits CASCADE")
        .await;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_users CASCADE")
        .await;
    let _ = db
        .execute_unprepared("DROP TABLE IF EXISTS eihwaz_groupes CASCADE")
        .await;
}

/// `eihwaz_users_groupes.user_id` porte une FK vers `eihwaz_users` — il faut
/// une ligne réelle pour l'id utilisé, sinon l'INSERT échoue en violation de
/// contrainte.
async fn seed_user(db: &runique::sea_orm::DatabaseConnection, id: runique::utils::config::Pk) {
    use runique::auth::user;
    use runique::sea_orm::ActiveModelTrait;
    use runique::sea_orm::ActiveValue::Set;

    let am = user::ActiveModel {
        id: Set(id),
        username: Set(format!("user-{id}")),
        email: Set(format!("user-{id}@example.com")),
        password: Set("hash123".to_string()),
        is_active: Set(true),
        is_staff: Set(false),
        is_superuser: Set(false),
        created_at: Set(None),
        updated_at: Set(None),
    };
    am.insert(db).await.expect("seed user");
}

// ═══════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_pull_groupes_db_retourne_permissions() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;
    seed_user(&db, pk(42)).await;

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

    let groupes = runique::admin::permissions::pull_groupes_db(&db, pk(42)).await;

    assert_eq!(groupes.len(), 1);
    assert_eq!(groupes[0].nom, "moderateur");
    assert_eq!(groupes[0].permissions.len(), 1);
    assert_eq!(groupes[0].permissions[0].resource_key, "articles");
    assert!(groupes[0].permissions[0].can_read);
    assert!(!groupes[0].permissions[0].can_create);

    teardown(&db).await;
}

#[tokio::test]
#[serial]
async fn test_pull_groupes_db_multi_ressources() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;
    seed_user(&db, pk(43)).await;

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

    let groupes = runique::admin::permissions::pull_groupes_db(&db, pk(43)).await;

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
#[serial]
async fn test_refresh_cache_puis_clear() {
    use runique::admin::permissions::refresh_cache_for_user;
    use runique::auth::guard::{clear_cache, get_permissions};

    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;
    seed_user(&db, pk(44)).await;

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
    refresh_cache_for_user(&db, pk(44)).await;
    assert!(get_permissions(pk(44)).is_some());

    // Invalide tout le cache (simule une modif admin)
    clear_cache();
    assert!(get_permissions(pk(44)).is_none());

    teardown(&db).await;
}

#[tokio::test]
#[serial]
async fn test_pull_groupes_db_user_sans_groupe() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };

    teardown(&db).await;
    setup_rbac_tables(&db).await;

    // User 99 n'appartient à aucun groupe
    let groupes = runique::admin::permissions::pull_groupes_db(&db, pk(99)).await;
    assert!(groupes.is_empty());

    teardown(&db).await;
}
