//! Tests — migration/makemigrations.rs
//! Couvre : seaorm_alter_module_name, seaorm_alter_file_path,
//!          update_migration_lib (création + mise à jour), parse_create_file

use runique::migration::makemigrations::{
    seaorm_alter_file_path, seaorm_alter_module_name, update_migration_lib,
};
use std::fs;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn temp_dir(suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("runique_test_mig_{}", suffix));
    fs::create_dir_all(&dir).ok();
    dir
}

// ═══════════════════════════════════════════════════════════════
// seaorm_alter_module_name
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_module_name_format() {
    let name = seaorm_alter_module_name("20260228_120000", "users");
    assert_eq!(name, "m20260228_120000_alter_users_table");
}

#[test]
fn test_alter_module_name_different_tables() {
    assert_eq!(
        seaorm_alter_module_name("20260101_000000", "posts"),
        "m20260101_000000_alter_posts_table"
    );
    assert_eq!(
        seaorm_alter_module_name("20260101_000000", "user_profiles"),
        "m20260101_000000_alter_user_profiles_table"
    );
}

// ═══════════════════════════════════════════════════════════════
// seaorm_alter_file_path
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_path_format() {
    let path = seaorm_alter_file_path("/migrations", "20260228_120000", "users");
    assert_eq!(path, "/migrations/m20260228_120000_alter_users_table.rs");
}

#[test]
fn test_alter_file_path_contient_timestamp() {
    let path = seaorm_alter_file_path("/some/path", "20260101_120000", "products");
    assert!(path.contains("20260101_120000"));
    assert!(path.contains("products"));
    assert!(path.ends_with(".rs"));
}

// ═══════════════════════════════════════════════════════════════
// update_migration_lib — création de lib.rs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_update_migration_lib_cree_le_fichier() {
    let dir = temp_dir("create_lib");
    let module = "m20260228_120000_create_users_table";
    let result = update_migration_lib(dir.to_str().unwrap(), module);
    assert!(result.is_ok(), "Doit réussir : {:?}", result);
    let lib = dir.join("lib.rs");
    assert!(lib.exists(), "lib.rs doit être créé");
    let content = fs::read_to_string(&lib).unwrap();
    assert!(content.contains(module));
}

#[test]
fn test_update_migration_lib_contenu_valide() {
    let dir = temp_dir("lib_content");
    let module = "m20260228_130000_create_posts_table";
    update_migration_lib(dir.to_str().unwrap(), module).unwrap();
    let content = fs::read_to_string(dir.join("lib.rs")).unwrap();
    assert!(content.contains("use sea_orm_migration::prelude::*;"));
    assert!(content.contains("pub struct Migrator;"));
    assert!(content.contains("impl MigratorTrait for Migrator"));
    assert!(content.contains(&format!("mod {};", module)));
    assert!(content.contains(&format!("Box::new({}::Migration)", module)));
}

#[test]
fn test_update_migration_lib_idempotent() {
    let dir = temp_dir("lib_idempotent");
    let module = "m20260228_140000_create_items_table";
    update_migration_lib(dir.to_str().unwrap(), module).unwrap();
    // Deuxième appel avec le même module ne doit pas dupliquer
    update_migration_lib(dir.to_str().unwrap(), module).unwrap();
    let content = fs::read_to_string(dir.join("lib.rs")).unwrap();
    let count = content.matches(&format!("mod {};", module)).count();
    assert_eq!(count, 1, "Le module ne doit apparaître qu'une seule fois");
}

#[test]
fn test_update_migration_lib_ajoute_second_module() {
    let dir = temp_dir("lib_second_module");
    let m1 = "m20260228_100000_create_users_table";
    let m2 = "m20260228_110000_create_posts_table";
    update_migration_lib(dir.to_str().unwrap(), m1).unwrap();
    update_migration_lib(dir.to_str().unwrap(), m2).unwrap();
    let content = fs::read_to_string(dir.join("lib.rs")).unwrap();
    assert!(content.contains(m1));
    assert!(content.contains(m2));
}

// ═══════════════════════════════════════════════════════════════
// parse_create_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_create_file_fichier_inexistant_retourne_err() {
    use runique::migration::makemigrations::parse_create_file;
    let result = parse_create_file("/chemin/inexistant/fichier.rs");
    assert!(result.is_err());
}

#[test]
fn test_parse_create_file_contenu_invalide_retourne_err() {
    use runique::migration::makemigrations::parse_create_file;
    let dir = temp_dir("parse_invalid");
    let file_path = dir.join("invalid.rs");
    fs::write(&file_path, "ce n'est pas du rust valide !!!@@@").unwrap();
    let result = parse_create_file(file_path.to_str().unwrap());
    assert!(result.is_err());
}
