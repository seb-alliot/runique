//! Tests — `BuiltinUserEntity::find_by_id` sur les 3 moteurs (SQLite, Postgres,
//! MariaDB via Docker), avec la MÊME rigueur d'assertion partout : égalité
//! champ à champ (`assert_eq!(found, alice)`), pas juste `username`.
//! `test_user_model.rs` a déjà des tests SQLite mais ne vérifie que
//! `username` — ce fichier ajoute l'équivalent complet pour SQLite (parité,
//! pas de doublon fonctionnel) et couvre Postgres/MariaDB qui n'avaient
//! aucun test.
//!
//! Objectif : vérifier que l'insertion puis la relecture des données passe
//! identiquement sur les 3 moteurs — c'est exactement le chemin qu'exerce
//! `get_fn` dans l'admin (cf. `admin/builtin/user.rs`, correctif id invalide
//! → `Ok(None)` au lieu d'une `DbErr`, 2026-08-31). La partie "id malformé"
//! du correctif est indépendante du moteur DB (`id.parse::<Pk>()` est du Rust
//! pur) — déjà couverte côté HTTP par `test_admin_password_security.rs`. Ce
//! fichier couvre l'autre moitié : les données sont-elles bien écrites et
//! relues à l'identique, sur les 3 moteurs.
//!
//! Postgres/MariaDB nécessitent `docker compose up -d` et
//! `DATABASE_URL_PG`/`DATABASE_URL_MARIADB` dans `.env.test`. Si absentes,
//! chaque test retourne immédiatement (skip implicite) — même convention que
//! `tests/db/test_postgres.rs`/`test_mariadb.rs`. SQLite (`:memory:`) tourne
//! toujours, sans dépendance externe.
//!
//! Table créée via `Schema::create_table_from_entity(user::Entity)` plutôt que
//! du DDL écrit à la main : le type de colonne `id` (SERIAL/UUID sous Postgres,
//! AUTO_INCREMENT/CHAR sous MariaDB…) suit alors automatiquement les attributs
//! réels de l'entité (et les features `big-pk`/`pk-uuid`), sans avoir à
//! réécrire à la main le mapping par moteur.

use crate::helpers::{db, db_mariadb, db_postgres, pk::pk};
use runique::auth::{BuiltinUserEntity, UserEntity, user};
use runique::sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseConnection, DbBackend, Schema,
};
use serial_test::serial;

// `eihwaz_users` a des dépendants FK ailleurs sur cette même base Docker partagée
// (`test_sea_migrate.rs` y fait tourner la vraie migration : `eihwaz_users_groupes`,
// `eihwaz_sessions`, `eihwaz_reset_tokens`, `contributions`... → `eihwaz_users`).
// Un DROP nu échoue si ce schéma plus large traîne encore. Postgres exige CASCADE
// explicite ; MariaDB n'a pas cette syntaxe et refuse le DROP tant que
// `foreign_key_checks` reste actif — désactivé le temps du DROP. SQLite n'a ni
// CASCADE ni FK actives par défaut, un DROP nu suffit.
async fn recreate_users_table(db: &DatabaseConnection) {
    let backend = db.get_database_backend();
    match backend {
        DbBackend::Postgres => {
            db.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users CASCADE")
                .await
                .expect("drop eihwaz_users");
        }
        DbBackend::MySql => {
            db.execute_unprepared("SET FOREIGN_KEY_CHECKS=0")
                .await
                .expect("disable FK checks");
            db.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users")
                .await
                .expect("drop eihwaz_users");
            db.execute_unprepared("SET FOREIGN_KEY_CHECKS=1")
                .await
                .expect("re-enable FK checks");
        }
        DbBackend::Sqlite => {
            db.execute_unprepared("DROP TABLE IF EXISTS eihwaz_users")
                .await
                .expect("drop eihwaz_users");
        }
        other => unimplemented!("moteur non couvert par ce test : {other:?}"),
    }
    let schema = Schema::new(backend);
    let stmt = schema.create_table_from_entity(user::Entity);
    db.execute(&stmt).await.expect("create eihwaz_users");
}

async fn insert_alice(db: &DatabaseConnection) -> user::Model {
    #[allow(unused_mut)]
    let mut am = user::ActiveModel {
        username: Set("alice".to_string()),
        email: Set("alice@example.com".to_string()),
        password: Set("hash123".to_string()),
        is_active: Set(true),
        is_staff: Set(false),
        is_superuser: Set(false),
        created_at: Set(None),
        updated_at: Set(None),
        ..Default::default()
    };
    // Uuid PKs ne sont jamais auto-increment côté DB — générées côté
    // application, comme `create_fn` dans `admin/builtin/user.rs`.
    #[cfg(feature = "pk-uuid")]
    {
        am.id = Set(pk(1));
    }
    am.insert(db).await.expect("insert alice")
}

// ═══════════════════════════════════════════════════════════════
// SQLite
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_find_by_id_found_sqlite() {
    let db = db::fresh_db().await;
    recreate_users_table(&db).await;
    let alice = insert_alice(&db).await;

    let found = BuiltinUserEntity::find_by_id(&db, alice.id).await;

    let found = found.expect("alice doit être relue depuis SQLite");
    assert_eq!(found, alice);
}

#[tokio::test]
async fn test_find_by_id_not_found_sqlite() {
    let db = db::fresh_db().await;
    recreate_users_table(&db).await;
    insert_alice(&db).await;

    let other = pk(999);
    let found = BuiltinUserEntity::find_by_id(&db, other).await;
    assert!(found.is_none());
}

// ═══════════════════════════════════════════════════════════════
// PostgreSQL
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_find_by_id_found_pg() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    let alice = insert_alice(&db).await;

    let found = BuiltinUserEntity::find_by_id(&db, alice.id).await;

    let found = found.expect("alice doit être relue depuis Postgres");
    // Comparaison champ à champ complète (id inclus) contre ce qui a été
    // inséré — pas juste un sous-ensemble des colonnes.
    assert_eq!(found, alice);

    db_postgres::exec(&db, "DROP TABLE IF EXISTS eihwaz_users CASCADE").await;
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found_pg() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    insert_alice(&db).await;

    // Un id bien typé mais qui ne correspond à aucune ligne doit rester
    // `None` — jamais une erreur DB, sur aucun moteur.
    let other = pk(999);
    let found = BuiltinUserEntity::find_by_id(&db, other).await;
    assert!(found.is_none());

    db_postgres::exec(&db, "DROP TABLE IF EXISTS eihwaz_users CASCADE").await;
}

// ═══════════════════════════════════════════════════════════════
// MariaDB
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_find_by_id_found_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    let alice = insert_alice(&db).await;

    let found = BuiltinUserEntity::find_by_id(&db, alice.id).await;

    let found = found.expect("alice doit être relue depuis MariaDB");
    // Comparaison champ à champ complète (id inclus) contre ce qui a été
    // inséré — pas juste un sous-ensemble des colonnes.
    assert_eq!(found, alice);

    db_mariadb::exec(&db, "DROP TABLE IF EXISTS eihwaz_users").await;
}

#[tokio::test]
#[serial]
async fn test_find_by_id_not_found_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    recreate_users_table(&db).await;
    insert_alice(&db).await;

    let other = pk(999);
    let found = BuiltinUserEntity::find_by_id(&db, other).await;
    assert!(found.is_none());

    db_mariadb::exec(&db, "DROP TABLE IF EXISTS eihwaz_users").await;
}
