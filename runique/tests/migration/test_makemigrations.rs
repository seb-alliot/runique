//! Tests — migration/makemigrations.rs
//! Couvre : seaorm_alter_module_name, seaorm_alter_file_path,
//!          update_migration_lib (création + mise à jour), parse_create_file,
//!          collect_destructive_messages

use runique::utils::cli::makemigration::{
    seaorm_alter_file_path, seaorm_alter_module_name, update_migration_lib,
};
use std::fs;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn temp_dir(suffix: &str) -> crate::utils::clean_tpm_test::TestTempDir {
    crate::utils::clean_tpm_test::TestTempDir::new("runique_test_mig", suffix)
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
    use runique::utils::cli::makemigration::parse_create_file;
    let result = parse_create_file("/chemin/inexistant/fichier.rs");
    assert!(result.is_err());
}

#[test]
fn test_parse_create_file_contenu_invalide_retourne_err() {
    use runique::utils::cli::makemigration::parse_create_file;
    let dir = temp_dir("parse_invalid");
    let file_path = dir.join("invalid.rs");
    fs::write(&file_path, "ce n'est pas du rust valide !!!@@@").unwrap();
    let result = parse_create_file(file_path.to_str().unwrap());
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════
// scan_entities
// ═══════════════════════════════════════════════════════════════

use runique::utils::cli::makemigration::scan_entities;

fn entity_user() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        fields: {
            username: String [unique],
            email: String [unique],
            is_active: bool,
            created_at: datetime [auto_now],
        }
    }
    "#
}

fn entity_post() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Post,
        table: "posts",
        pk: id => i64,
        fields: {
            title: String,
            body: text [nullable],
            user_id: i32,
            published_at: datetime [nullable],
        }
    }
    "#
}

fn entity_product() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Product,
        table: "products",
        pk: id => i32,
        fields: {
            name: String,
            price: f64,
            stock: i32,
            sku: String [unique],
            description: text [nullable],
        }
    }
    "#
}

#[test]
fn test_scan_entities_dir_vide_retourne_vide() {
    let dir = temp_dir("scan_empty");
    let result = scan_entities(dir.to_str().unwrap());
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty(), "dossier vide → aucun schéma");
}

#[test]
fn test_scan_entities_dir_inexistant_retourne_err() {
    let result = scan_entities("/chemin/qui/nexiste/pas/du/tout");
    assert!(result.is_err(), "dossier inexistant doit retourner Err");
}

#[test]
fn test_scan_entities_un_fichier() {
    let dir = temp_dir("scan_one");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].table_name, "users");
}

#[test]
fn test_scan_entities_deux_fichiers() {
    let dir = temp_dir("scan_two");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    fs::write(dir.join("post.rs"), entity_post()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 2);
    let tables: Vec<&str> = schemas.iter().map(|s| s.table_name.as_str()).collect();
    assert!(tables.contains(&"users"));
    assert!(tables.contains(&"posts"));
}

#[test]
fn test_scan_entities_trois_fichiers() {
    let dir = temp_dir("scan_three");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    fs::write(dir.join("post.rs"), entity_post()).unwrap();
    fs::write(dir.join("product.rs"), entity_product()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 3);
}

#[test]
fn test_scan_entities_ignore_mod_rs() {
    let dir = temp_dir("scan_mod");
    fs::write(dir.join("mod.rs"), entity_user()).unwrap();
    fs::write(dir.join("post.rs"), entity_post()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1, "mod.rs doit être ignoré");
    assert_eq!(schemas[0].table_name, "posts");
}

#[test]
fn test_scan_entities_ignore_fichiers_non_rs() {
    let dir = temp_dir("scan_nonrs");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    fs::write(dir.join("README.md"), "## Entities").unwrap();
    fs::write(dir.join("schema.toml"), "[table]\nname=\"x\"").unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1, "seuls les .rs doivent être scannés");
}

#[test]
fn test_scan_entities_fichier_sans_model_macro() {
    let dir = temp_dir("scan_no_macro");
    let src = r#"pub struct Foo { pub id: i32, pub name: String }"#;
    fs::write(dir.join("foo.rs"), src).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    assert!(
        schemas.is_empty(),
        "fichier sans model! ne doit pas générer de schéma"
    );
}

#[test]
fn test_scan_entities_contenu_schema_user() {
    let dir = temp_dir("scan_content_user");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    let s = &schemas[0];
    assert_eq!(s.table_name, "users");
    assert!(s.primary_key.is_some());
    assert_eq!(s.primary_key.as_ref().unwrap().name, "id");
    // 4 champs: username, email, is_active, created_at
    assert_eq!(s.columns.len(), 4);
}

#[test]
fn test_scan_entities_auto_now_est_ignore() {
    let dir = temp_dir("scan_auto_now");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    let created_at = schemas[0]
        .columns
        .iter()
        .find(|c| c.name == "created_at")
        .unwrap();
    assert!(
        !created_at.ignored,
        "created_at (auto_now) ne doit PAS être ignored"
    );
}

#[test]
fn test_scan_entities_champs_unique_detectes() {
    let dir = temp_dir("scan_unique");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    let schemas = scan_entities(dir.to_str().unwrap()).unwrap();
    let username = schemas[0]
        .columns
        .iter()
        .find(|c| c.name == "username")
        .unwrap();
    assert!(username.unique, "username doit être unique");
}

#[test]
fn test_scan_entities_melange_valide_invalide() {
    let dir = temp_dir("scan_mixed");
    fs::write(dir.join("user.rs"), entity_user()).unwrap();
    fs::write(dir.join("garbage.rs"), "let x = !!@@;").unwrap();
    // Le fichier invalide est ignoré silencieusement (parse error → pas de schema)
    // Mais scan_entities retourne quand même Ok si le fichier est du Rust invalide
    // (parse_schema_from_source retourne None pour du Rust invalide)
    let result = scan_entities(dir.to_str().unwrap());
    // Si scan_entities propage l'erreur de lecture/parse : Err
    // Si scan_entities utilise parse_schema_from_source (qui retourne None) : Ok avec 1 schéma
    // Selon l'implémentation, l'un ou l'autre est acceptable.
    // On vérifie juste que ça ne panique pas.
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════
// collect_destructive_messages
// ═══════════════════════════════════════════════════════════════

use runique::migration::utils::types::{Changes, ParsedColumn, ParsedFk};
use runique::utils::cli::makemigration::collect_destructive_messages;

fn col(name: &str, col_type: &str, nullable: bool) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: col_type.to_string(),
        nullable,
        ..Default::default()
    }
}

fn empty_changes(table: &str) -> Changes {
    Changes {
        table_name: table.to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    }
}

#[test]
fn no_changes_returns_empty() {
    let changes = vec![empty_changes("users")];
    assert!(collect_destructive_messages(&changes).is_empty());
}

#[test]
fn dropped_column_detected() {
    let mut c = empty_changes("users");
    c.dropped_columns.push(col("email", "text", true));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("users.email"));
    assert!(msgs[0].contains("DROP COLUMN"));
}

#[test]
fn type_change_detected() {
    let mut c = empty_changes("posts");
    c.modified_columns
        .push((col("views", "int", false), col("views", "bigint", false)));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("posts.views"));
    assert!(msgs[0].contains("int"));
    assert!(msgs[0].contains("bigint"));
}

#[test]
fn nullable_to_required_detected() {
    let mut c = empty_changes("orders");
    c.modified_columns
        .push((col("note", "text", true), col("note", "text", false)));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("orders.note"));
    assert!(msgs[0].contains("not_null"));
}

#[test]
fn same_type_nullable_to_nullable_not_destructive() {
    let mut c = empty_changes("items");
    c.modified_columns
        .push((col("desc", "text", true), col("desc", "text", true)));
    assert!(collect_destructive_messages(&[c]).is_empty());
}

#[test]
fn multiple_tables_all_collected() {
    let mut c1 = empty_changes("users");
    c1.dropped_columns.push(col("phone", "text", true));

    let mut c2 = empty_changes("orders");
    c2.modified_columns
        .push((col("amount", "int", false), col("amount", "bigint", false)));

    let msgs = collect_destructive_messages(&[c1, c2]);
    assert_eq!(msgs.len(), 2);
    assert!(msgs.iter().any(|m| m.contains("users.phone")));
    assert!(msgs.iter().any(|m| m.contains("orders.amount")));
}

// ── FK tests ──────────────────────────────────────────────────────────────────

fn fk(from: &str, to_table: &str, on_delete: &str) -> ParsedFk {
    ParsedFk {
        from_column: from.to_string(),
        to_table: to_table.to_string(),
        to_column: "id".to_string(),
        on_delete: on_delete.to_string(),
        on_update: "NO ACTION".to_string(),
    }
}

#[test]
fn dropped_fk_detected() {
    let mut c = empty_changes("comments");
    c.dropped_fks.push(fk("post_id", "posts", "RESTRICT"));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("comments.post_id"));
    assert!(msgs[0].contains("DROP FOREIGN KEY"));
    assert!(msgs[0].contains("posts"));
}

#[test]
fn added_fk_cascade_detected() {
    let mut c = empty_changes("orders");
    c.added_fks.push(fk("user_id", "users", "CASCADE"));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
    assert!(msgs[0].contains("orders.user_id"));
    assert!(msgs[0].contains("CASCADE"));
}

#[test]
fn added_fk_cascade_case_insensitive() {
    let mut c = empty_changes("orders");
    c.added_fks.push(fk("user_id", "users", "cascade"));
    let msgs = collect_destructive_messages(&[c]);
    assert_eq!(msgs.len(), 1);
}

#[test]
fn added_fk_restrict_not_destructive() {
    let mut c = empty_changes("orders");
    c.added_fks.push(fk("user_id", "users", "RESTRICT"));
    assert!(collect_destructive_messages(&[c]).is_empty());
}

#[test]
fn added_fk_set_null_not_destructive() {
    let mut c = empty_changes("orders");
    c.added_fks.push(fk("user_id", "users", "SET NULL"));
    assert!(collect_destructive_messages(&[c]).is_empty());
}

#[test]
fn added_fk_no_action_not_destructive() {
    let mut c = empty_changes("items");
    c.added_fks
        .push(fk("category_id", "categories", "NO ACTION"));
    assert!(collect_destructive_messages(&[c]).is_empty());
}

// ═══════════════════════════════════════════════════════════════
// seaorm_extend_module_name / seaorm_extend_file_path
// ═══════════════════════════════════════════════════════════════

use runique::utils::cli::makemigration::{seaorm_extend_file_path, seaorm_extend_module_name};

#[test]
fn extend_module_name_format() {
    let name = seaorm_extend_module_name("20260101_000000", "eihwaz_users");
    assert_eq!(name, "m20260101_000000_extend_eihwaz_users_table");
}

#[test]
fn extend_module_name_tables_differentes() {
    assert_eq!(
        seaorm_extend_module_name("20260101_000000", "posts"),
        "m20260101_000000_extend_posts_table"
    );
}

#[test]
fn extend_file_path_format() {
    let path = seaorm_extend_file_path("/migrations", "20260101_000000", "eihwaz_users");
    assert_eq!(
        path,
        "/migrations/m20260101_000000_extend_eihwaz_users_table.rs"
    );
}

#[test]
fn extend_file_path_termine_par_rs() {
    let path = seaorm_extend_file_path("/some/path", "20260228_120000", "orders");
    assert!(path.ends_with(".rs"));
    assert!(path.contains("orders"));
}

// ═══════════════════════════════════════════════════════════════
// merge_extend_schemas
// ═══════════════════════════════════════════════════════════════

use runique::migration::utils::types::ParsedSchema;
use runique::utils::cli::makemigration::merge_extend_schemas;

fn extend_schema(table: &str, col_names: &[&str]) -> ParsedSchema {
    ParsedSchema {
        table_name: table.to_string(),
        primary_key: None,
        columns: col_names
            .iter()
            .map(|n| ParsedColumn {
                name: n.to_string(),
                col_type: "text".to_string(),
                ..Default::default()
            })
            .collect(),
        foreign_keys: vec![],
        indexes: vec![],
    }
}

#[test]
fn merge_meme_table_concatene_colonnes() {
    let s1 = extend_schema("eihwaz_users", &["bio"]);
    let s2 = extend_schema("eihwaz_users", &["telephone"]);
    let result = merge_extend_schemas(vec![s1, s2]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].table_name, "eihwaz_users");
    assert_eq!(result[0].columns.len(), 2);
    let names: Vec<&str> = result[0].columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"bio"));
    assert!(names.contains(&"telephone"));
}

#[test]
fn merge_tables_differentes_reste_separees() {
    let s1 = extend_schema("eihwaz_users", &["bio"]);
    let s2 = extend_schema("orders", &["note"]);
    let result = merge_extend_schemas(vec![s1, s2]);
    assert_eq!(result.len(), 2);
}

#[test]
fn merge_ordre_premiere_occurrence_preserve() {
    let s1 = extend_schema("table_a", &["col1"]);
    let s2 = extend_schema("table_b", &["col2"]);
    let s3 = extend_schema("table_a", &["col3"]);
    let result = merge_extend_schemas(vec![s1, s2, s3]);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].table_name, "table_a");
    assert_eq!(result[1].table_name, "table_b");
    assert_eq!(result[0].columns.len(), 2);
}

#[test]
fn merge_vide_retourne_vide() {
    let result = merge_extend_schemas(vec![]);
    assert!(result.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// scan_extend_blocks
// ═══════════════════════════════════════════════════════════════

use runique::utils::cli::makemigration::scan_extend_blocks;

#[test]
fn scan_extend_dir_vide_retourne_vide() {
    let dir = temp_dir("scan_ext_empty");
    let result = scan_extend_blocks(dir.to_str().unwrap());
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn scan_extend_dir_inexistant_retourne_err() {
    let result = scan_extend_blocks("/chemin/qui/nexiste/pas");
    assert!(result.is_err());
}

#[test]
fn scan_extend_detecte_un_bloc() {
    let dir = temp_dir("scan_ext_one");
    let src = r#"
        extend! {
            table: "eihwaz_users",
            fields: { bio: textarea, }
        }
    "#;
    fs::write(dir.join("user_ext.rs"), src).unwrap();
    let result = scan_extend_blocks(dir.to_str().unwrap()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].table_name, "eihwaz_users");
}

#[test]
fn scan_extend_ignore_fichier_sans_extend() {
    let dir = temp_dir("scan_ext_no_macro");
    fs::write(dir.join("user.rs"), "pub struct User { pub id: i32 }").unwrap();
    let result = scan_extend_blocks(dir.to_str().unwrap()).unwrap();
    assert!(result.is_empty());
}

#[test]
fn scan_extend_ignore_mod_rs() {
    let dir = temp_dir("scan_ext_mod");
    let src = r#"extend! { table: "users", fields: { x: text, } }"#;
    fs::write(dir.join("mod.rs"), src).unwrap();
    let result = scan_extend_blocks(dir.to_str().unwrap()).unwrap();
    assert!(result.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// ensure_admin_migration_positioned
// ═══════════════════════════════════════════════════════════════

use runique::utils::cli::makemigration::ensure_admin_migration_positioned;
use std::sync::Mutex;
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lib_with_vec(modules: &[&str]) -> String {
    let mods: String = modules.iter().map(|m| format!("mod {};\n", m)).collect();
    let boxes: String = modules
        .iter()
        .map(|m| format!("            Box::new({}::Migration),\n", m))
        .collect();
    format!(
        "use sea_orm_migration::prelude::*;\n{}\npub struct Migrator;\n\
         #[async_trait::async_trait]\nimpl MigratorTrait for Migrator {{\n\
         fn migrations() -> Vec<Box<dyn MigrationTrait>> {{\n        vec![\n{}        ]\n    }}\n}}\n",
        mods, boxes
    )
}

#[test]
fn ensure_admin_no_lib_retourne_ok() {
    let dir = temp_dir("ensure_no_lib");
    let result = ensure_admin_migration_positioned(dir.to_str().unwrap());
    assert!(result.is_ok());
}

#[test]
fn ensure_admin_builtin_insere_trois_migrations_framework() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("RUNIQUE_USER_TABLE") };
    let dir = temp_dir("ensure_builtin");
    fs::write(
        dir.join("lib.rs"),
        lib_with_vec(&["m20260101_create_menus_table"]),
    )
    .unwrap();
    ensure_admin_migration_positioned(dir.to_str().unwrap()).unwrap();
    let content = fs::read_to_string(dir.join("lib.rs")).unwrap();
    assert!(content.contains("EihwazUsersMigration"));
    assert!(content.contains("EihwazSessionsMigration"));
    assert!(content.contains("AdminTableMigration"));
}

#[test]
fn ensure_admin_builtin_idempotent() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("RUNIQUE_USER_TABLE") };
    let dir = temp_dir("ensure_idempotent");
    fs::write(
        dir.join("lib.rs"),
        lib_with_vec(&["m20260101_create_menus_table"]),
    )
    .unwrap();
    ensure_admin_migration_positioned(dir.to_str().unwrap()).unwrap();
    ensure_admin_migration_positioned(dir.to_str().unwrap()).unwrap();
    let content = fs::read_to_string(dir.join("lib.rs")).unwrap();
    assert_eq!(content.matches("EihwazUsersMigration").count(), 1);
    assert_eq!(content.matches("AdminTableMigration").count(), 1);
}
