//! Tests — middleware/auth/user.rs
//! Couvre : RuniqueUser trait, BuiltinUserEntity (DB), schema()

use crate::helpers::db;
use runique::auth::{BuiltinUserEntity, UserEntity, user::Model, user_trait::RuniqueUser};

// ─── Helper ──────────────────────────────────────────────────────────────────

fn make_model() -> Model {
    Model {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "hashed".to_string(),
        is_active: true,
        is_staff: false,
        is_superuser: false,
        created_at: None,
        updated_at: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// RuniqueUser trait
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_user_id() {
    let model = make_model();
    assert_eq!(model.user_id(), 1);
}

#[test]
fn test_runique_user_username() {
    let model = make_model();
    assert_eq!(model.username(), "alice");
}

#[test]
fn test_runique_user_email() {
    let model = make_model();
    assert_eq!(model.email(), "alice@example.com");
}

#[test]
fn test_runique_user_password_hash() {
    let model = make_model();
    assert_eq!(model.password_hash(), "hashed");
}

#[test]
fn test_runique_user_is_active() {
    let model = make_model();
    assert!(model.is_active());
}

#[test]
fn test_runique_user_is_staff_false() {
    let model = make_model();
    assert!(!model.is_staff());
}

#[test]
fn test_runique_user_is_superuser_false() {
    let model = make_model();
    assert!(!model.is_superuser());
}

#[test]
fn test_runique_user_is_superuser_true() {
    let mut model = make_model();
    model.is_superuser = true;
    assert!(model.is_superuser());
}

#[test]
fn test_runique_user_roles_default_vide() {
    // roles() retourne toujours vec![] (implémentation par défaut du trait)
    let model = make_model();
    assert!(model.roles().is_empty());
}

#[test]
fn test_runique_user_can_access_admin_false() {
    let model = make_model();
    assert!(!model.can_access_admin());
}

#[test]
fn test_runique_user_can_access_admin_staff() {
    let mut model = make_model();
    model.is_staff = true;
    assert!(model.can_access_admin());
}

// ═══════════════════════════════════════════════════════════════
// schema()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_table_name() {
    let schema = runique::auth::user::schema();
    assert_eq!(schema.table_name, "eihwaz_users");
}

#[test]
fn test_schema_has_required_columns() {
    let schema = runique::auth::user::schema();
    let col_names: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(col_names.contains(&"username"));
    assert!(col_names.contains(&"email"));
    assert!(col_names.contains(&"password"));
    assert!(col_names.contains(&"is_active"));
}

#[test]
fn test_schema_has_primary_key() {
    let schema = runique::auth::user::schema();
    assert!(schema.primary_key.is_some());
    assert_eq!(schema.primary_key.as_ref().unwrap().name, "id");
}

// ═══════════════════════════════════════════════════════════════
// BuiltinUserEntity — DB tests
// ═══════════════════════════════════════════════════════════════

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
        updated_at  TEXT
    )
";

const INSERT_ALICE: &str = "
    INSERT INTO eihwaz_users (username, email, password, is_active, is_staff, is_superuser)
    VALUES ('alice', 'alice@example.com', 'hash123', 1, 0, 0)
";

#[tokio::test]
async fn test_find_by_id_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    db::exec(&db, INSERT_ALICE).await;
    let user = BuiltinUserEntity::find_by_id(&db, 1).await;
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "alice");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let user = BuiltinUserEntity::find_by_id(&db, 999).await;
    assert!(user.is_none());
}

#[tokio::test]
async fn test_find_by_username_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    db::exec(&db, INSERT_ALICE).await;
    let user = BuiltinUserEntity::find_by_username(&db, "alice").await;
    assert!(user.is_some());
    assert_eq!(user.unwrap().email, "alice@example.com");
}

#[tokio::test]
async fn test_find_by_username_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let user = BuiltinUserEntity::find_by_username(&db, "nobody").await;
    assert!(user.is_none());
}

#[tokio::test]
async fn test_find_by_email_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    db::exec(&db, INSERT_ALICE).await;
    let user = BuiltinUserEntity::find_by_email(&db, "alice@example.com").await;
    assert!(user.is_some());
    assert_eq!(user.unwrap().username, "alice");
}

#[tokio::test]
async fn test_find_by_email_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let user = BuiltinUserEntity::find_by_email(&db, "ghost@example.com").await;
    assert!(user.is_none());
}

#[tokio::test]
async fn test_update_password_success() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    db::exec(&db, INSERT_ALICE).await;
    let result = BuiltinUserEntity::update_password(&db, "alice@example.com", "newhash").await;
    assert!(result.is_ok());
    let user = BuiltinUserEntity::find_by_email(&db, "alice@example.com")
        .await
        .unwrap();
    assert_eq!(user.password, "newhash");
}

#[tokio::test]
async fn test_update_password_not_found() {
    let db = db::fresh_db_with_schema(USERS_DDL).await;
    let result = BuiltinUserEntity::update_password(&db, "ghost@example.com", "hash").await;
    assert!(result.is_err());
}
