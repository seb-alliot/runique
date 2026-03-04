//! Tests — Flux de migration complet (end-to-end)
//! Couvre : scan_entities → diff_schemas → generate_create_file / generate_alter_file
//!          → update_migration_lib → arborescence des fichiers produits
//!
//! Ces tests reproduisent manuellement ce que `makemigrations::run()` fait,
//! en utilisant uniquement des fonctions synchrones et des répertoires temporaires.

use runique::migration::makemigrations::{
    scan_entities, seaorm_alter_file_path, seaorm_alter_module_name, update_migration_lib,
};
use runique::migration::utils::{
    diff::{db_columns, diff_schemas},
    generators::{generate_alter_file, generate_create_file},
    paths::*,
    types::{Changes, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};
use std::fs;
use std::path::{Path, PathBuf};

// ── Helpers partagés ─────────────────────────────────────────────────────────

fn tmp(suffix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("runique_flow_{}", suffix));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn entities_dir(base: &Path) -> PathBuf {
    let d = base.join("entities");
    fs::create_dir_all(&d).unwrap();
    d
}

fn migrations_dir(base: &Path) -> PathBuf {
    let d = base.join("migrations");
    fs::create_dir_all(&d).unwrap();
    d
}

fn user_entity() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        User,
        table: "users",
        pk: id => i32,
        fields: {
            username: String [unique],
            email: String [unique],
            password: String,
            is_active: bool,
            bio: text [nullable],
            created_at: datetime [auto_now],
            updated_at: datetime [auto_now_update],
        }
    }
    "#
}

fn post_entity() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Post,
        table: "posts",
        pk: id => i64,
        fields: {
            title: String,
            body: text [nullable],
            slug: String [unique],
            user_id: i32,
            view_count: i32,
            published: bool,
            created_at: datetime [auto_now],
        }
    }
    "#
}

fn comment_entity() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Comment,
        table: "comments",
        pk: id => i32,
        fields: {
            content: text,
            user_id: i32,
            post_id: i32,
            created_at: datetime [auto_now],
        }
    }
    "#
}

fn tag_entity() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Tag,
        table: "tags",
        pk: id => i32,
        fields: {
            name: String [unique],
            slug: String [unique],
            description: text [nullable],
        },
        meta: {
            ordering: [name],
            verbose_name: "Tag",
        }
    }
    "#
}

fn product_entity() -> &'static str {
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
            is_available: bool,
            weight: f32 [nullable],
            created_at: datetime [auto_now],
        }
    }
    "#
}

// ── Helpers de construction ParsedSchema ─────────────────────────────────────

fn col(name: &str, ty: &str) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: ty.to_string(),
        nullable: false,
        unique: false,
        ignored: false,
        created_at: false,
        updated_at: false,
    }
}

fn col_opt(name: &str, ty: &str, nullable: bool, unique: bool, ignored: bool) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: ty.to_string(),
        nullable,
        unique,
        ignored,
        created_at: false,
        updated_at: false,
    }
}

fn schema_users() -> ParsedSchema {
    ParsedSchema {
        table_name: "users".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![
            col_opt("username", "String", false, true, false),
            col_opt("email", "String", false, true, false),
            col("password", "String"),
            col("is_active", "Boolean"),
            col_opt("bio", "String", true, false, false),
            col_opt("created_at", "DateTime", true, false, true),
            col_opt("updated_at", "DateTime", true, false, true),
        ],
        foreign_keys: vec![],
        indexes: vec![],
    }
}

fn schema_posts() -> ParsedSchema {
    ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "BigInteger")),
        columns: vec![
            col("title", "String"),
            col_opt("body", "String", true, false, false),
            col_opt("slug", "String", false, true, false),
            col("user_id", "Integer"),
            col("view_count", "Integer"),
            col("published", "Boolean"),
            col_opt("created_at", "DateTime", true, false, true),
        ],
        foreign_keys: vec![ParsedFk {
            from_column: "user_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
            on_delete: "Cascade".to_string(),
            on_update: "NoAction".to_string(),
        }],
        indexes: vec![ParsedIndex {
            name: "idx_posts_slug".to_string(),
            columns: vec!["slug".to_string()],
            unique: true,
        }],
    }
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 1 — Scan des entities
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_scan_un_modele() {
    let base = tmp("flow_scan_one");
    let ent = entities_dir(&base);
    fs::write(ent.join("user.rs"), user_entity()).unwrap();

    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].table_name, "users");
}

#[test]
fn test_flow_scan_plusieurs_modeles() {
    let base = tmp("flow_scan_multi");
    let ent = entities_dir(&base);
    fs::write(ent.join("user.rs"), user_entity()).unwrap();
    fs::write(ent.join("post.rs"), post_entity()).unwrap();
    fs::write(ent.join("comment.rs"), comment_entity()).unwrap();
    fs::write(ent.join("tag.rs"), tag_entity()).unwrap();

    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 4);

    let tables: std::collections::HashSet<&str> =
        schemas.iter().map(|s| s.table_name.as_str()).collect();
    assert!(tables.contains("users"));
    assert!(tables.contains("posts"));
    assert!(tables.contains("comments"));
    assert!(tables.contains("tags"));
}

#[test]
fn test_flow_scan_modele_avec_meta() {
    let base = tmp("flow_scan_meta");
    let ent = entities_dir(&base);
    fs::write(ent.join("tag.rs"), tag_entity()).unwrap();

    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].table_name, "tags");

    let name_col = schemas[0]
        .columns
        .iter()
        .find(|c| c.name == "name")
        .unwrap();
    assert!(name_col.unique, "name doit être unique dans tags");
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 2 — Génération du fichier CREATE
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_generate_create_users() {
    let schema = schema_users();
    let content = generate_create_file(&schema);

    assert!(content.contains("users"), "nom de table présent");
    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
    assert!(content.contains("username"));
    assert!(content.contains("email"));
    assert!(content.contains("password"));
    assert!(content.contains("is_active"));
    // champs ignored (auto_now) ne doivent pas apparaître dans le CREATE
    // Note: dépend de si generate_create_file filtre les ignored
}

#[test]
fn test_flow_generate_create_posts_avec_fk() {
    let schema = schema_posts();
    let content = generate_create_file(&schema);

    assert!(content.contains("posts"), "nom de table présent");
    assert!(content.contains("user_id"), "colonne FK présente");
    assert!(
        content.contains("users"),
        "référence à la table users présente"
    );
}

#[test]
fn test_flow_generate_create_posts_avec_index() {
    let schema = schema_posts();
    let content = generate_create_file(&schema);

    assert!(content.contains("idx_posts_slug"), "nom de l'index présent");
}

#[test]
fn test_flow_generate_create_contient_pk_auto_increment() {
    let schema = schema_users();
    let content = generate_create_file(&schema);

    assert!(
        content.contains("auto_increment"),
        "PK entière → auto_increment"
    );
}

#[test]
fn test_flow_generate_create_pk_uuid_sans_auto_increment() {
    let schema = ParsedSchema {
        table_name: "sessions".to_string(),
        primary_key: Some(col("token", "Uuid")),
        columns: vec![col("user_id", "Integer")],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);

    assert!(content.contains("sessions"));
    // UUID PKs ne doivent pas avoir auto_increment
    assert!(
        !content.contains("auto_increment"),
        "PK UUID ne doit pas avoir auto_increment"
    );
}

#[test]
fn test_flow_generate_create_colonne_nullable() {
    let schema = ParsedSchema {
        table_name: "profiles".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![
            col("name", "String"),
            col_opt("bio", "String", true, false, false),
        ],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
    assert!(content.contains(".null()"), "colonne nullable → .null()");
    assert!(
        content.contains(".not_null()"),
        "colonne requise → .not_null()"
    );
}

#[test]
fn test_flow_generate_create_colonne_unique() {
    let schema = ParsedSchema {
        table_name: "accounts".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![col_opt("email", "String", false, true, false)],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
    assert!(
        content.contains(".unique_key()"),
        "colonne unique → .unique_key()"
    );
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 3 — Diff de schémas
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_diff_schema_inchange() {
    let s = schema_users();
    let changes = diff_schemas(&s, &s);
    assert!(changes.is_empty(), "schéma identique → aucun changement");
}

#[test]
fn test_flow_diff_ajout_colonne() {
    let old = schema_users();
    let mut new = schema_users();
    new.columns.push(col("phone", "String"));

    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.added_columns.len(), 1);
    assert_eq!(changes.added_columns[0].name, "phone");
}

#[test]
fn test_flow_diff_suppression_colonne() {
    let old = schema_users();
    let mut new = schema_users();
    // Supprimer bio
    new.columns.retain(|c| c.name != "bio");

    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.dropped_columns.len(), 1);
    assert_eq!(changes.dropped_columns[0].name, "bio");
}

#[test]
fn test_flow_diff_modification_type() {
    let old = schema_users();
    let mut new = schema_users();
    // Changer le type de "view_count" de Integer → BigInteger
    if let Some(c) = new.columns.iter_mut().find(|c| c.name == "is_active") {
        c.col_type = "String".to_string();
    }
    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.modified_columns.len(), 1);
    let (old_col, new_col) = &changes.modified_columns[0];
    assert_eq!(old_col.col_type, "Boolean");
    assert_eq!(new_col.col_type, "String");
}

#[test]
fn test_flow_diff_ajout_fk() {
    let old = schema_users();
    let mut new = schema_users();
    new.foreign_keys.push(ParsedFk {
        from_column: "manager_id".to_string(),
        to_table: "users".to_string(),
        to_column: "id".to_string(),
        on_delete: "SetNull".to_string(),
        on_update: "NoAction".to_string(),
    });
    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.added_fks.len(), 1);
    assert_eq!(changes.added_fks[0].from_column, "manager_id");
}

#[test]
fn test_flow_diff_suppression_fk() {
    let old = schema_posts(); // a une FK user_id
    let mut new = schema_posts();
    new.foreign_keys.clear();

    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.dropped_fks.len(), 1);
    assert_eq!(changes.dropped_fks[0].from_column, "user_id");
}

#[test]
fn test_flow_diff_ajout_index() {
    let old = schema_users();
    let mut new = schema_users();
    new.indexes.push(ParsedIndex {
        name: "idx_users_email".to_string(),
        columns: vec!["email".to_string()],
        unique: true,
    });
    let changes = diff_schemas(&old, &new);
    assert_eq!(changes.added_indexes.len(), 1);
    assert_eq!(changes.added_indexes[0].name, "idx_users_email");
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 4 — Génération du fichier ALTER
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_generate_alter_ajout_colonne() {
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![col("phone", "String")],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    };
    let content = generate_alter_file(&changes);

    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("phone"), "nouvelle colonne dans ALTER");
    assert!(content.contains("add_column") || content.contains("alter_table"));
}

#[test]
fn test_flow_generate_alter_suppression_colonne() {
    let old_bio = col_opt("bio", "String", true, false, false);
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![],
        dropped_columns: vec![old_bio],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    };
    let content = generate_alter_file(&changes);

    assert!(content.contains("bio"), "colonne supprimée dans ALTER");
    assert!(content.contains("drop_column") || content.contains("alter_table"));
}

#[test]
fn test_flow_generate_alter_up_et_down_sont_inverses() {
    let added = col("phone", "String");
    let dropped = col_opt("old_field", "String", true, false, false);

    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![added.clone()],
        dropped_columns: vec![dropped.clone()],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    };
    let content = generate_alter_file(&changes);

    // Les deux colonnes doivent apparaître dans la migration
    assert!(content.contains("phone"));
    assert!(content.contains("old_field"));

    // up() et down() sont présents
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
}

#[test]
fn test_flow_generate_alter_type_change_commente() {
    let old = col("count", "Integer");
    let new_col = col("count", "BigInteger");

    let changes = Changes {
        table_name: "stats".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old, new_col)],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    };
    let content = generate_alter_file(&changes);

    assert!(
        content.contains("WARNING") || content.contains("Manual migration required"),
        "type change doit générer un avertissement"
    );
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 5 — update_migration_lib + structure de fichiers
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_update_lib_cree_lib_rs() {
    let base = tmp("flow_lib_create");
    let mig = migrations_dir(&base);
    let module = "m20260301_120000_create_users_table";

    update_migration_lib(mig.to_str().unwrap(), module).unwrap();

    let lib = mig.join("lib.rs");
    assert!(lib.exists(), "lib.rs doit être créé");

    let content = fs::read_to_string(&lib).unwrap();
    assert!(content.contains("use sea_orm_migration::prelude::*;"));
    assert!(content.contains("pub struct Migrator;"));
    assert!(content.contains("impl MigratorTrait for Migrator"));
    assert!(content.contains(&format!("mod {};", module)));
    assert!(content.contains(&format!("Box::new({}::Migration)", module)));
}

#[test]
fn test_flow_update_lib_ajoute_plusieurs_modules() {
    let base = tmp("flow_lib_multi");
    let mig = migrations_dir(&base);

    let modules = [
        "m20260301_100000_create_users_table",
        "m20260301_110000_create_posts_table",
        "m20260301_120000_create_comments_table",
    ];

    for m in &modules {
        update_migration_lib(mig.to_str().unwrap(), m).unwrap();
    }

    let content = fs::read_to_string(mig.join("lib.rs")).unwrap();
    for m in &modules {
        assert!(
            content.contains(&format!("mod {};", m)),
            "module {} absent",
            m
        );
        assert!(
            content.contains(&format!("Box::new({}::Migration)", m)),
            "Box::new pour {} absent",
            m
        );
    }
}

#[test]
fn test_flow_update_lib_idempotent() {
    let base = tmp("flow_lib_idempotent");
    let mig = migrations_dir(&base);
    let module = "m20260301_100000_create_users_table";

    update_migration_lib(mig.to_str().unwrap(), module).unwrap();
    update_migration_lib(mig.to_str().unwrap(), module).unwrap();
    update_migration_lib(mig.to_str().unwrap(), module).unwrap();

    let content = fs::read_to_string(mig.join("lib.rs")).unwrap();
    let count = content.matches(&format!("mod {};", module)).count();
    assert_eq!(count, 1, "Le module ne doit apparaître qu'une fois");
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 6 — Flux complet : scan → generate → écriture fichiers
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_complet_premiere_migration() {
    let base = tmp("flow_complet_init");
    let ent = entities_dir(&base);
    let mig = migrations_dir(&base);
    let ts = "20260301_120000";

    // 1. Écriture des entités
    fs::write(ent.join("user.rs"), user_entity()).unwrap();
    fs::write(ent.join("post.rs"), post_entity()).unwrap();

    // 2. Scan
    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 2);

    // 3. Pour chaque schéma (nouvelle table) → génère le fichier CREATE
    fs::create_dir_all(snapshot_dir(mig.to_str().unwrap())).unwrap();

    for schema in &schemas {
        let module = seaorm_create_module_name(ts, &schema.table_name);
        let file_path = seaorm_create_file_path(mig.to_str().unwrap(), ts, &schema.table_name);
        let snap_path = snapshot_file_path(mig.to_str().unwrap(), &schema.table_name);

        let content = generate_create_file(schema);
        fs::write(&file_path, &content).unwrap();
        fs::write(&snap_path, &content).unwrap();

        update_migration_lib(mig.to_str().unwrap(), &module).unwrap();
    }

    // 4. Vérifications — lib.rs
    let lib_content = fs::read_to_string(mig.join("lib.rs")).unwrap();
    assert!(lib_content.contains("create_users_table"));
    assert!(lib_content.contains("create_posts_table"));

    // 5. Vérifications — fichiers migration
    let users_file = mig.join(format!("m{}_create_users_table.rs", ts));
    let posts_file = mig.join(format!("m{}_create_posts_table.rs", ts));
    assert!(users_file.exists(), "fichier migration users doit exister");
    assert!(posts_file.exists(), "fichier migration posts doit exister");

    // 6. Vérifications — snapshots
    let snap_users = mig.join("snapshots").join("users.rs");
    let snap_posts = mig.join("snapshots").join("posts.rs");
    assert!(snap_users.exists(), "snapshot users doit exister");
    assert!(snap_posts.exists(), "snapshot posts doit exister");

    // 7. Contenu des fichiers
    let users_content = fs::read_to_string(&users_file).unwrap();
    assert!(users_content.contains("users"));
    assert!(users_content.contains("MigrationTrait"));

    let posts_content = fs::read_to_string(&posts_file).unwrap();
    assert!(posts_content.contains("posts"));
}

#[test]
fn test_flow_complet_migration_alter() {
    let base = tmp("flow_complet_alter");
    let mig = migrations_dir(&base);

    // Ancienne version : juste username + email
    let old_schema = ParsedSchema {
        table_name: "users".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![
            col_opt("username", "String", false, true, false),
            col_opt("email", "String", false, true, false),
        ],
        foreign_keys: vec![],
        indexes: vec![],
    };

    // Nouvelle version : on ajoute phone et on supprime email
    let new_schema = ParsedSchema {
        table_name: "users".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![
            col_opt("username", "String", false, true, false),
            col("phone", "String"),
            col_opt("is_active", "Boolean", false, false, false),
        ],
        foreign_keys: vec![],
        indexes: vec![],
    };

    let ts = "20260301_130000";
    let changes = diff_schemas(&old_schema, &new_schema);

    assert!(!changes.is_empty(), "des changements doivent être détectés");
    assert_eq!(changes.added_columns.len(), 2, "phone et is_active ajoutés");
    assert_eq!(changes.dropped_columns.len(), 1, "email supprimé");

    // Génère le fichier ALTER
    let alter_content = generate_alter_file(&changes);
    let module = seaorm_alter_module_name(ts, "users");
    let alter_path = seaorm_alter_file_path(mig.to_str().unwrap(), ts, "users");

    fs::write(&alter_path, &alter_content).unwrap();
    update_migration_lib(mig.to_str().unwrap(), &module).unwrap();

    // Vérifications
    assert!(
        std::path::Path::new(&alter_path).exists(),
        "fichier ALTER doit exister"
    );

    let lib_content = fs::read_to_string(mig.join("lib.rs")).unwrap();
    assert!(lib_content.contains("alter_users_table"));

    let content = fs::read_to_string(&alter_path).unwrap();
    assert!(content.contains("phone"), "nouvelle colonne dans ALTER");
    assert!(
        content.contains("email"),
        "colonne supprimée dans ALTER down"
    );
    assert!(content.contains("MigrationTrait"));
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 7 — Nommage des modules (seaorm_create / seaorm_alter)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_module_name_create() {
    assert_eq!(
        seaorm_create_module_name("20260301_120000", "users"),
        "m20260301_120000_create_users_table"
    );
}

#[test]
fn test_flow_module_name_alter() {
    assert_eq!(
        seaorm_alter_module_name("20260301_130000", "users"),
        "m20260301_130000_alter_users_table"
    );
}

#[test]
fn test_flow_file_path_create_correspond_module() {
    let ts = "20260301_120000";
    let table = "user_profiles";
    let module = seaorm_create_module_name(ts, table);
    let path = seaorm_create_file_path("/migrations", ts, table);
    assert!(
        path.ends_with(&format!("{}.rs", module)),
        "le chemin doit se terminer par le nom du module + .rs"
    );
}

#[test]
fn test_flow_chemins_snapshot_coherents() {
    let mig = "/project/migration/src";
    let table = "users";
    let snap = snapshot_file_path(mig, table);
    let snap_dir = snapshot_dir(mig);
    assert!(
        snap.starts_with(&snap_dir),
        "snapshot doit être dans le dossier snapshots"
    );
    assert!(snap.ends_with("users.rs"));
}

// ═══════════════════════════════════════════════════════════════
// ÉTAPE 8 — db_columns : colonnes effectives en base
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_db_columns_exclut_pk() {
    let schema = schema_users();
    let cols = db_columns(&schema);
    assert!(
        !cols.iter().any(|c| c.name == "id"),
        "la PK ne doit pas être dans db_columns"
    );
}

#[test]
fn test_flow_db_columns_exclut_ignored() {
    let schema = schema_users();
    let cols = db_columns(&schema);
    assert!(
        !cols.iter().any(|c| c.name == "created_at"),
        "created_at (ignored) ne doit pas être dans db_columns"
    );
    assert!(
        !cols.iter().any(|c| c.name == "updated_at"),
        "updated_at (ignored) ne doit pas être dans db_columns"
    );
}

#[test]
fn test_flow_db_columns_inclut_colonnes_normales() {
    let schema = schema_users();
    let cols = db_columns(&schema);
    let names: Vec<&str> = cols.iter().map(|c| c.name.as_str()).collect();

    assert!(names.contains(&"username"));
    assert!(names.contains(&"email"));
    assert!(names.contains(&"password"));
    assert!(names.contains(&"is_active"));
    assert!(names.contains(&"bio"));
}

// ═══════════════════════════════════════════════════════════════
// Types float (f32, f64) — scan + génération
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_flow_scan_modele_avec_float() {
    let base = tmp("flow_float_scan");
    let ent = entities_dir(&base);
    fs::write(ent.join("product.rs"), product_entity()).unwrap();

    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].table_name, "products");

    let weight = schemas[0]
        .columns
        .iter()
        .find(|c| c.name == "weight")
        .unwrap();
    assert!(weight.nullable, "weight (f32 nullable) doit être nullable");

    let sku = schemas[0].columns.iter().find(|c| c.name == "sku").unwrap();
    assert!(sku.unique, "sku doit être unique");
}

#[test]
fn test_flow_generate_create_float_types() {
    let base = tmp("flow_float_gen");
    let ent = entities_dir(&base);
    fs::write(ent.join("product.rs"), product_entity()).unwrap();

    let schemas = scan_entities(ent.to_str().unwrap()).unwrap();
    let content = generate_create_file(&schemas[0]);

    assert!(content.contains("products"), "table products présente");
    assert!(content.contains("price"), "colonne price présente");
    assert!(content.contains("weight"), "colonne weight présente");
    assert!(content.contains(".null()"), "weight nullable → .null()");
    assert!(content.contains("sku"), "colonne sku présente");
    assert!(
        content.contains(".unique_key()"),
        "sku unique → .unique_key()"
    );
}
