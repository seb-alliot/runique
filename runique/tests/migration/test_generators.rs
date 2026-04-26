//! Tests — migration/utils/generators.rs
//! Couvre : generate_create_file, generate_alter_file,
//!          generate_batch_up_file, generate_batch_down_file

use runique::migration::utils::{
    generators::{
        generate_alter_file, generate_batch_down_file, generate_batch_up_file, generate_create_file,
    },
    types::{Changes, DbKind, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn col(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: col_type.to_string(),
        ..ParsedColumn::default()
    }
}

fn col_nullable(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        nullable: true,
        ..col(name, col_type)
    }
}

fn simple_schema(table: &str) -> ParsedSchema {
    ParsedSchema {
        table_name: table.to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![col("name", "String"), col_nullable("bio", "String")],
        foreign_keys: vec![],
        indexes: vec![],
    }
}

fn schema_with_fk() -> ParsedSchema {
    ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![col("title", "String"), col("user_id", "i32")],
        foreign_keys: vec![ParsedFk {
            from_column: "user_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
            on_delete: "Cascade".to_string(),
            on_update: "NoAction".to_string(),
        }],
        indexes: vec![],
    }
}

fn schema_with_index() -> ParsedSchema {
    ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![col("slug", "String")],
        foreign_keys: vec![],
        indexes: vec![ParsedIndex {
            name: "idx_articles_slug".to_string(),
            columns: vec!["slug".to_string()],
            unique: true,
        }],
    }
}

fn simple_changes(table: &str) -> Changes {
    Changes {
        table_name: table.to_string(),
        added_columns: vec![col("new_col", "String")],
        dropped_columns: vec![col("old_col", "i32")],
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

// ═══════════════════════════════════════════════════════════════
// generate_create_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_create_file_contient_nom_table() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains("users"),
        "Le nom de la table doit apparaître"
    );
}

#[test]
fn test_create_file_contient_struct_migration() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("impl MigrationTrait for Migration"));
}

#[test]
fn test_create_file_contient_up_et_down() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
}

#[test]
fn test_create_file_contient_colonnes() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains("name"),
        "La colonne 'name' doit être présente"
    );
}

#[test]
fn test_create_file_avec_cle_etrangere() {
    let schema = schema_with_fk();
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains("user_id"),
        "La FK 'user_id' doit apparaître"
    );
}

#[test]
fn test_create_file_avec_index() {
    let schema = schema_with_index();
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains("idx_articles_slug"),
        "L'index doit apparaître"
    );
}

#[test]
fn test_create_file_schema_vide_colonnes() {
    let schema = ParsedSchema {
        table_name: "empty_table".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(content.contains("empty_table"));
}

#[test]
fn test_create_file_sans_pk() {
    let schema = ParsedSchema {
        table_name: "junction_table".to_string(),
        primary_key: None,
        columns: vec![col("user_id", "i32"), col("tag_id", "i32")],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(content.contains("junction_table"));
}

// ═══════════════════════════════════════════════════════════════
// generate_alter_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_contient_struct_migration() {
    let changes = simple_changes("users");
    let content = generate_alter_file(&changes);
    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("impl MigrationTrait for Migration"));
}

#[test]
fn test_alter_file_contient_up_et_down() {
    let changes = simple_changes("users");
    let content = generate_alter_file(&changes);
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
}

#[test]
fn test_alter_file_sans_changements() {
    let changes = Changes {
        table_name: "users".to_string(),
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
    };
    let content = generate_alter_file(&changes);
    assert!(content.contains("pub struct Migration"));
}

#[test]
fn test_alter_file_avec_ajout_fk() {
    let changes = Changes {
        table_name: "posts".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![ParsedFk {
            from_column: "author_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
            on_delete: "Cascade".to_string(),
            on_update: "NoAction".to_string(),
        }],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(content.contains("author_id") || content.contains("users"));
}

// ═══════════════════════════════════════════════════════════════
// generate_alter_file — enum_renames → UPDATE SQL
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_enum_rename_genere_update_up() {
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![(
            "status".to_string(),
            "Ajoute".to_string(),
            "Ajouté".to_string(),
        )],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("Ajoute") && content.contains("Ajouté"),
        "UP doit contenir les deux valeurs"
    );
    assert!(content.contains("UPDATE"), "UP doit générer un UPDATE SQL");
}

#[test]
fn test_alter_file_enum_rename_genere_update_down() {
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![(
            "status".to_string(),
            "Ajoute".to_string(),
            "Ajouté".to_string(),
        )],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    // DOWN doit inverser : SET 'Ajoute' WHERE 'Ajouté'
    let down_section = content.split("async fn down").nth(1).unwrap_or("");
    assert!(
        down_section.contains("Ajoute") || down_section.contains("Ajouté"),
        "DOWN doit aussi contenir un UPDATE inversé"
    );
}

#[test]
fn test_alter_file_enum_rename_contient_nom_table() {
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![("status".to_string(), "old".to_string(), "new".to_string())],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("articles"),
        "Le nom de la table doit apparaître dans l'UPDATE"
    );
}

// ═══════════════════════════════════════════════════════════════
// generate_batch_up_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_batch_up_contient_timestamp() {
    let changes = simple_changes("users");
    let ts = "20260228_120000";
    let content = generate_batch_up_file(&[&changes], ts);
    assert!(content.contains(ts));
}

#[test]
fn test_batch_up_contient_nom_table() {
    let changes = simple_changes("users");
    let content = generate_batch_up_file(&[&changes], "20260228_120000");
    assert!(content.contains("users"));
}

#[test]
fn test_batch_up_plusieurs_tables() {
    let c1 = simple_changes("users");
    let c2 = simple_changes("posts");
    let content = generate_batch_up_file(&[&c1, &c2], "20260228_120000");
    assert!(content.contains("users"));
    assert!(content.contains("posts"));
}

#[test]
fn test_batch_up_contient_struct_migration() {
    let changes = simple_changes("users");
    let content = generate_batch_up_file(&[&changes], "20260228_120000");
    assert!(content.contains("pub struct Migration"));
}

// ═══════════════════════════════════════════════════════════════
// generate_batch_down_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_batch_down_contient_timestamp() {
    let changes = simple_changes("users");
    let ts = "20260228_120000";
    let content = generate_batch_down_file(&[&changes], ts);
    assert!(content.contains(ts));
}

#[test]
fn test_batch_down_contient_nom_table() {
    let changes = simple_changes("users");
    let content = generate_batch_down_file(&[&changes], "20260228_120000");
    assert!(content.contains("users"));
}

#[test]
fn test_batch_down_contient_struct_migration() {
    let changes = simple_changes("users");
    let content = generate_batch_down_file(&[&changes], "20260228_120000");
    assert!(content.contains("pub struct Migration"));
}

#[test]
fn test_batch_down_plusieurs_tables() {
    let c1 = simple_changes("users");
    let c2 = simple_changes("posts");
    let content = generate_batch_down_file(&[&c1, &c2], "20260228_120000");
    assert!(content.contains("users"));
    assert!(content.contains("posts"));
}

// ═══════════════════════════════════════════════════════════════
// generate_create_file — branches Postgres / MySQL
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_create_file_postgres_enum_stmts() {
    let schema = ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "status".to_string(),
            col_type: "String".to_string(),
            enum_string_values: vec!["Draft".to_string(), "Published".to_string()],
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        content.contains("CREATE TYPE"),
        "Postgres doit créer un type enum"
    );
    assert!(content.contains("'Draft'") && content.contains("'Published'"));
}

#[test]
fn test_create_file_postgres_enum_drops() {
    let schema = ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "status".to_string(),
            col_type: "String".to_string(),
            enum_string_values: vec!["Draft".to_string()],
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        content.contains("DROP TYPE IF EXISTS"),
        "Postgres down doit supprimer le type"
    );
}

#[test]
fn test_create_file_postgres_updated_at_trigger() {
    let schema = ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "updated_at".to_string(),
            col_type: "DateTime".to_string(),
            updated_at: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Postgres);
    assert!(
        content.contains("CREATE TRIGGER"),
        "Postgres doit créer un trigger updated_at"
    );
    assert!(
        content.contains("DROP TRIGGER IF EXISTS"),
        "Down doit supprimer le trigger"
    );
}

#[test]
fn test_create_file_mysql_updated_at_on_update() {
    let schema = ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "updated_at".to_string(),
            col_type: "DateTime".to_string(),
            updated_at: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Mysql);
    assert!(
        content.contains("ON UPDATE CURRENT_TIMESTAMP"),
        "MySQL doit utiliser ON UPDATE CURRENT_TIMESTAMP"
    );
}

#[test]
fn test_create_file_col_nullable_unique_default_now() {
    let schema = ParsedSchema {
        table_name: "items".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "created_at".to_string(),
            col_type: "DateTime".to_string(),
            nullable: true,
            unique: true,
            has_default_now: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains(".null()"),
        "Colonne nullable doit contenir .null()"
    );
    assert!(
        content.contains(".unique_key()"),
        "Colonne unique doit contenir .unique_key()"
    );
    assert!(
        content.contains("Expr::current_timestamp()"),
        "default_now doit utiliser current_timestamp"
    );
}

#[test]
fn test_create_file_col_enum_values_columndef_with_type() {
    let schema = ParsedSchema {
        table_name: "events".to_string(),
        primary_key: Some(col("id", "i32")),
        columns: vec![ParsedColumn {
            name: "kind".to_string(),
            col_type: "String".to_string(),
            enum_string_values: vec!["A".to_string(), "B".to_string()],
            enum_name: Some("event_kind".to_string()),
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        content.contains("ColumnDef::new_with_type"),
        "Colonne enum doit utiliser new_with_type"
    );
}

#[test]
fn test_create_file_pk_non_integer_no_autoinc() {
    let schema = ParsedSchema {
        table_name: "tokens".to_string(),
        primary_key: Some(ParsedColumn {
            name: "token".to_string(),
            col_type: "String".to_string(),
            ..ParsedColumn::default()
        }),
        columns: vec![],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema, &DbKind::Other);
    assert!(
        !content.contains(".auto_increment()"),
        "PK String ne doit pas avoir auto_increment"
    );
    assert!(
        content.contains(".primary_key()"),
        "PK doit avoir .primary_key()"
    );
}

// ═══════════════════════════════════════════════════════════════
// generate_alter_file — branches supplémentaires
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_type_change_generates_warning() {
    let old_col = ParsedColumn {
        name: "age".to_string(),
        col_type: "Integer".to_string(),
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "age".to_string(),
        col_type: "BigInteger".to_string(),
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old_col, new_col)],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("WARNING"),
        "Changement de type doit générer un avertissement"
    );
    assert!(content.contains("Manual migration required"));
}

#[test]
fn test_alter_file_nullable_to_not_null_modify_column() {
    let old_col = ParsedColumn {
        name: "bio".to_string(),
        col_type: "String".to_string(),
        nullable: true,
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "bio".to_string(),
        col_type: "String".to_string(),
        nullable: false,
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old_col, new_col)],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("modify_column"),
        "nullable→not_null doit générer modify_column"
    );
}

#[test]
fn test_alter_file_enum_value_adds() {
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![(
            "status".to_string(),
            "article_status".to_string(),
            "Archived".to_string(),
        )],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("ADD VALUE IF NOT EXISTS"),
        "enum_value_adds doit générer ALTER TYPE ADD VALUE"
    );
    assert!(content.contains("Archived"));
}

#[test]
fn test_alter_file_enum_value_drops_warning() {
    let changes = Changes {
        table_name: "articles".to_string(),
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
        enum_value_drops: vec![(
            "status".to_string(),
            "article_status".to_string(),
            "OldVal".to_string(),
        )],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("WARNING"),
        "enum_value_drops doit générer un WARNING dans up"
    );
    assert!(content.contains("OldVal"));
}

#[test]
fn test_alter_file_drop_index_in_up() {
    let changes = Changes {
        table_name: "posts".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![ParsedIndex {
            name: "idx_posts_slug".to_string(),
            columns: vec!["slug".to_string()],
            unique: false,
        }],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("drop_index"),
        "UP doit contenir drop_index pour index supprimé"
    );
    assert!(content.contains("idx_posts_slug"));
}

#[test]
fn test_alter_file_add_index_in_up() {
    let changes = Changes {
        table_name: "posts".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![ParsedIndex {
            name: "idx_posts_title".to_string(),
            columns: vec!["title".to_string()],
            unique: true,
        }],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("create_index"),
        "UP doit contenir create_index pour index ajouté"
    );
    assert!(content.contains("idx_posts_title"));
    assert!(
        content.contains("unique_key"),
        "Index unique doit contenir unique_key"
    );
}

#[test]
fn test_alter_file_drop_fk_in_up() {
    let changes = Changes {
        table_name: "comments".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![ParsedFk {
            from_column: "post_id".to_string(),
            to_table: "posts".to_string(),
            to_column: "id".to_string(),
            on_delete: "Cascade".to_string(),
            on_update: "NoAction".to_string(),
        }],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
        enum_renames: vec![],
        enum_value_adds: vec![],
        enum_value_drops: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("drop_foreign_key"),
        "UP doit contenir drop_foreign_key pour FK supprimée"
    );
}
