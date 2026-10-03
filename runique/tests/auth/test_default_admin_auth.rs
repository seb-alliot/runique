//! Tests — auth/user.rs : authenticate_admin()

use crate::helpers::db;
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
        is_staff    INTEGER NOT NULL DEFAULT 0,
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
        is_staff    INTEGER NOT NULL DEFAULT 0,
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
    "id, username, email, password, is_active, is_staff, is_superuser, activated_at"
}
#[cfg(not(feature = "pk-uuid"))]
fn insert_user_columns() -> &'static str {
    "username, email, password, is_active, is_staff, is_superuser, activated_at"
}

#[cfg(feature = "pk-uuid")]
fn insert_user_id_prefix(n: u32) -> String {
    format!("{}, ", pk_sql_literal(n))
}
#[cfg(not(feature = "pk-uuid"))]
fn insert_user_id_prefix(_n: u32) -> String {
    String::new()
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — user inexistant
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_user_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "unknown",
        "password",
    )
    .await;
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — mauvais mot de passe
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_wrong_password() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let hash = runique::utils::hash("correct_password").unwrap();
    db::exec(
        &db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'admin', 'admin@example.com', '{hash}', 1, 1, 0, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(1),
        ),
    )
    .await;

    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "admin",
        "wrong_password",
    )
    .await;
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — utilisateur sans accès admin
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_no_admin_access() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let hash = runique::utils::hash("password123").unwrap();
    // is_staff=0, is_superuser=0 → can_access_admin() = false
    db::exec(
        &db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'regular', 'regular@example.com', '{hash}', 1, 0, 0, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(2),
        ),
    )
    .await;

    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "regular",
        "password123",
    )
    .await;
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — utilisateur inactif
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_inactive_user() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let hash = runique::utils::hash("password123").unwrap();
    // is_active=0 → can_access_admin() = false (is_active check)
    db::exec(
        &db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'inactive', 'inactive@example.com', '{hash}', 0, 1, 0, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(3),
        ),
    )
    .await;

    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "inactive",
        "password123",
    )
    .await;
    assert!(result.is_none());
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — succès (staff)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_success_staff() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let hash = runique::utils::hash("securepass1").unwrap();
    db::exec(
        &db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'staffuser', 'staff@example.com', '{hash}', 1, 1, 0, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(4),
        ),
    )
    .await;

    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "staffuser",
        "securepass1",
    )
    .await;
    assert!(result.is_some());
    let r = result.unwrap();
    assert_eq!(r.username, "staffuser");
    assert!(r.is_staff);
    assert!(!r.is_superuser);
}

// ═══════════════════════════════════════════════════════════════
// authenticate() — succès (superuser)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_authenticate_success_superuser() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let hash = runique::utils::hash("superpass1").unwrap();
    db::exec(
        &db,
        &format!(
            "INSERT INTO eihwaz_users ({}) VALUES ({}'superuser', 'super@example.com', '{hash}', 1, 0, 1, '2026-01-01 00:00:00')",
            insert_user_columns(),
            insert_user_id_prefix(5),
        ),
    )
    .await;

    let result = authenticate_admin(
        &runique::db::ADb::from_connection(db.clone()),
        "superuser",
        "superpass1",
    )
    .await;
    assert!(result.is_some());
    let r = result.unwrap();
    assert!(!r.is_staff);
    assert!(r.is_superuser);
}
