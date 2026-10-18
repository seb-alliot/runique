//! Tests — auth/user.rs : authenticate_admin()

use crate::helpers::db;
use crate::helpers::framework_tables::{GROUPES_DDL, GROUPES_DROITS_DDL, USERS_GROUPES_DDL};
#[cfg(feature = "pk-uuid")]
use crate::helpers::pk::pk_sql_literal;
use runique::auth::authenticate_admin;

// ─── DDL ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "pk-uuid")]
const USERS_DDL: &str = "
    CREATE TABLE eihwaz_users (
        id          BLOB PRIMARY KEY,
        username    TEXT NOT NULL UNIQUE,
        email       TEXT NOT NULL UNIQUE,
        password    TEXT NOT NULL,
        is_active   INTEGER NOT NULL DEFAULT 1,
        is_dirigeant INTEGER NOT NULL DEFAULT 0,
        is_admin    INTEGER NOT NULL DEFAULT 0,
        is_superuser INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT,
        updated_at  TEXT,
        activated_at TEXT
    )
";

#[cfg(not(feature = "pk-uuid"))]
const USERS_DDL: &str = "
    CREATE TABLE eihwaz_users (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        username    TEXT NOT NULL UNIQUE,
        email       TEXT NOT NULL UNIQUE,
        password    TEXT NOT NULL,
        is_active   INTEGER NOT NULL DEFAULT 1,
        is_dirigeant INTEGER NOT NULL DEFAULT 0,
        is_admin    INTEGER NOT NULL DEFAULT 0,
        is_superuser INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT,
        updated_at  TEXT,
        activated_at TEXT
    )
";

// Sous pk-uuid, l'id n'est jamais généré côté DB (pas d'auto-increment possible
// pour un Uuid) — chaque INSERT doit fournir sa propre valeur explicite.
#[cfg(feature = "pk-uuid")]
fn insert_user_columns() -> &'static str {
    "id, username, email, password, is_active, is_dirigeant, is_admin, is_superuser, activated_at"
}
#[cfg(not(feature = "pk-uuid"))]
fn insert_user_columns() -> &'static str {
    "username, email, password, is_active, is_dirigeant, is_admin, is_superuser, activated_at"
}

#[cfg(feature = "pk-uuid")]
fn insert_user_id_prefix(n: u32) -> String {
    format!("{}, ", pk_sql_literal(n))
}
#[cfg(not(feature = "pk-uuid"))]
fn insert_user_id_prefix(_n: u32) -> String {
    String::new()
}

/// One account; `flags` = (is_active, is_dirigeant, is_admin, is_superuser).
async fn insert_user(
    db: &runique::sea_orm::DatabaseConnection,
    n: u32,
    username: &str,
    password: &str,
    flags: (u8, u8, u8, u8),
) {
    let hash = runique::utils::hash(password).unwrap();
    let (active, dirigeant, admin, superuser) = flags;
    db::exec(
        db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'{username}', '{username}@example.com', '{hash}', {active}, {dirigeant}, {admin}, {superuser}, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(n),
        ),
    )
    .await;
}

/// Group tables, and `username` in one group whose `is_staff` is `staff`.
async fn join_group(db: &runique::sea_orm::DatabaseConnection, username: &str, staff: bool) {
    for ddl in [GROUPES_DDL, GROUPES_DROITS_DDL, USERS_GROUPES_DDL] {
        db::exec(db, ddl).await;
    }
    db::exec(
        db,
        &format!(
            "INSERT INTO eihwaz_groupes (id, nom, is_staff) VALUES (1, 'equipe', {})",
            u8::from(staff)
        ),
    )
    .await;
    db::exec(
        db,
        &format!(
            "INSERT INTO eihwaz_users_groupes (user_id, groupe_id) \
             SELECT id, 1 FROM eihwaz_users WHERE username = '{username}'"
        ),
    )
    .await;
}

async fn authenticate(
    db: &runique::sea_orm::DatabaseConnection,
    username: &str,
    password: &str,
) -> Option<runique::auth::user::Model> {
    authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        username,
        password,
    )
    .await
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — user inexistant
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_user_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    assert!(authenticate(&db, "unknown", "password").await.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — mauvais mot de passe
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_wrong_password() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 1, "admin", "correct_password", (1, 0, 1, 0)).await;
    assert!(authenticate(&db, "admin", "wrong_password").await.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — utilisateur sans accès admin
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_no_admin_access() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 2, "regular", "password123", (1, 0, 0, 0)).await;
    join_group(&db, "nobody", true).await;
    assert!(authenticate(&db, "regular", "password123").await.is_none());
}

#[tokio::test]
async fn test_authenticate_group_that_does_not_open_the_admin() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 2, "regular", "password123", (1, 0, 0, 0)).await;
    join_group(&db, "regular", false).await;
    assert!(authenticate(&db, "regular", "password123").await.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — utilisateur inactif
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_inactive_user() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 3, "inactive", "password123", (0, 0, 1, 0)).await;
    assert!(authenticate(&db, "inactive", "password123").await.is_none());
}

#[tokio::test]
async fn test_authenticate_inactive_staff_member() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 3, "inactive", "password123", (0, 0, 0, 0)).await;
    join_group(&db, "inactive", true).await;
    assert!(authenticate(&db, "inactive", "password123").await.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — succès (groupe staff)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_success_staff() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    insert_user(&db, 4, "staffuser", "securepass1", (1, 0, 0, 0)).await;
    join_group(&db, "staffuser", true).await;
    let r = authenticate(&db, "staffuser", "securepass1")
        .await
        .expect("a staff group opens the admin");
    assert_eq!(r.username, "staffuser");
    assert!(!r.is_superuser);
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — succès (rang, sans groupe)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_success_every_rank_without_group() {
    for (n, username, flags) in [
        (5, "admin", (1, 0, 1, 0)),
        (6, "dirigeant", (1, 1, 0, 0)),
        (7, "superuser", (1, 0, 0, 1)),
    ] {
        let db = db::fresh_db_with_schema(USERS_DDL).await;
        insert_user(&db, n, username, "rankpass1", flags).await;
        let r = authenticate(&db, username, "rankpass1").await;
        assert!(r.is_some(), "{username}");
    }
}
