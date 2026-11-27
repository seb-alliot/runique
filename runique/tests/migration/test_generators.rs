//! Tests — migration/generate/
//! Couvre : generate_create_file, generate_alter_file,
//!          generate_snapshot_file

use runique_cli::db::DatabaseEngine;
use runique_cli::migration::types::{Changes, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};
use runique_cli::migration::{
    EnumChange, EnumChanges, EnumPlace, generate_alter_file, generate_create_file,
    generate_enum_file, generate_snapshot_file,
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn col(name: &str, col_type: &str) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: runique_cli::migration::ColumnType::from_name(col_type)
            .expect("the name of a column type"),
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
        primary_key: Some(col("id", "Integer")),
        columns: vec![col("name", "String"), col_nullable("bio", "String")],
        foreign_keys: vec![],
        indexes: vec![],
    }
}

fn schema_with_fk() -> ParsedSchema {
    ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![col("title", "String"), col("user_id", "Integer")],
        foreign_keys: vec![ParsedFk {
            from_column: "user_id".to_string(),
            to_table: "users".to_string(),
            to_column: "id".to_string(),
            on_delete: runique_cli::migration::FkAction::Cascade,
            on_update: runique_cli::migration::FkAction::NoAction,
        }],
        indexes: vec![],
    }
}

fn schema_with_index() -> ParsedSchema {
    ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "Integer")),
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
        dropped_columns: vec![col("old_col", "Integer")],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    }
}

// ═══════════════════════════════════════════════════════════════
// generate_create_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_create_file_contient_nom_table() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema);
    assert!(
        content.contains("users"),
        "Le nom de la table doit apparaître"
    );
}

#[test]
fn test_create_file_contient_struct_migration() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema);
    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("impl MigrationTrait for Migration"));
}

#[test]
fn test_create_file_contient_up_et_down() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema);
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
}

#[test]
fn test_create_file_contient_colonnes() {
    let schema = simple_schema("users");
    let content = generate_create_file(&schema);
    assert!(
        content.contains("name"),
        "La colonne 'name' doit être présente"
    );
}

#[test]
fn test_create_file_avec_cle_etrangere() {
    let schema = schema_with_fk();
    let content = generate_create_file(&schema);
    assert!(
        content.contains("user_id"),
        "La FK 'user_id' doit apparaître"
    );
}

#[test]
fn test_create_file_avec_index() {
    let schema = schema_with_index();
    let content = generate_create_file(&schema);
    assert!(
        content.contains("idx_articles_slug"),
        "L'index doit apparaître"
    );
}

#[test]
fn test_create_file_schema_vide_colonnes() {
    let schema = ParsedSchema {
        table_name: "empty_table".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
    assert!(content.contains("empty_table"));
}

#[test]
fn test_create_file_sans_pk() {
    let schema = ParsedSchema {
        table_name: "junction_table".to_string(),
        primary_key: None,
        columns: vec![col("user_id", "Integer"), col("tag_id", "Integer")],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
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
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
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
            on_delete: runique_cli::migration::FkAction::Cascade,
            on_update: runique_cli::migration::FkAction::NoAction,
        }],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(content.contains("author_id") || content.contains("users"));
}

// ═══════════════════════════════════════════════════════════════
// generate_enum_file — une valeur renommée, une fois pour l'enum
// ═══════════════════════════════════════════════════════════════

/// Un enum `status` employé par la colonne `status` de `articles`.
fn status_enum() -> EnumChanges {
    EnumChanges {
        name: "status".to_string(),
        places: vec![EnumPlace {
            table: "articles".to_string(),
            column: "status".to_string(),
        }],
        ..EnumChanges::default()
    }
}

const ENGINES: [DatabaseEngine; 3] = [
    DatabaseEngine::PostgreSQL,
    DatabaseEngine::MySQL,
    DatabaseEngine::SQLite,
];

#[test]
fn test_enum_file_rename_genere_un_appel_up() {
    let changes = EnumChanges {
        renamed_values: vec![("Ajoute".to_string(), "Ajouté".to_string())],
        ..status_enum()
    };
    for engine in &ENGINES {
        let content = generate_enum_file(&changes, engine);
        let up = content.split("async fn down").next().unwrap_or("");
        assert!(
            up.contains(
                r#"runtime::rename_enum_value(manager, "status", "Ajoute", "Ajouté", &[("articles", "status")]).await?;"#
            ),
            "UP renomme la valeur par le runtime, une fois pour l'enum : {content}"
        );
    }
}

#[test]
fn test_enum_file_rename_genere_l_appel_inverse_down() {
    let changes = EnumChanges {
        renamed_values: vec![("Ajoute".to_string(), "Ajouté".to_string())],
        ..status_enum()
    };
    for engine in &ENGINES {
        let content = generate_enum_file(&changes, engine);
        let down_section = content.split("async fn down").nth(1).unwrap_or("");
        assert!(
            down_section.contains(
                r#"runtime::rename_enum_value(manager, "status", "Ajouté", "Ajoute", &[("articles", "status")]).await?;"#
            ),
            "DOWN reprend l'ancien nom : {content}"
        );
    }
}

#[test]
fn test_enum_file_rename_nomme_chaque_endroit() {
    let mut changes = EnumChanges {
        renamed_values: vec![("old".to_string(), "new".to_string())],
        ..status_enum()
    };
    changes.places.push(EnumPlace {
        table: "articles_tags".to_string(),
        column: "value".to_string(),
    });
    let content = generate_enum_file(&changes, &DatabaseEngine::MySQL);
    assert!(
        content.contains(r#"&[("articles", "status"), ("articles_tags", "value")]"#),
        "Chaque colonne de l'enum est nommée, dans un seul appel : {content}"
    );
    assert_eq!(content.matches("rename_enum_value").count(), 2, "{content}");
}

// ═══════════════════════════════════════════════════════════════
// generate_create_file — branches Postgres / MySQL
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_create_file_ne_cree_pas_le_type_enum() {
    let schema = ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "status".to_string(),
            col_type: runique_cli::migration::ColumnType::String,
            enum_string_values: vec!["Draft".to_string(), "Published".to_string()],
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
    assert!(
        !content.contains("Type::create()") && !content.contains("create_enum_type"),
        "Le type est créé par le fichier de l'enum, pas par celui de la table : {content}"
    );
    // La colonne nomme toujours son type et ses valeurs.
    assert!(
        content.contains("Alias::new(\"Draft\")") && content.contains("Alias::new(\"Published\")")
    );
}

#[test]
fn test_enum_file_postgres_cree_le_type_avec_ses_valeurs() {
    let changes = EnumChanges {
        change: EnumChange::Created,
        values: vec!["Draft".to_string(), "Published".to_string()],
        ..status_enum()
    };
    let content = generate_enum_file(&changes, &DatabaseEngine::PostgreSQL);
    assert!(
        content.contains(
            r#"runtime::create_enum_type(manager, "status", &["Draft", "Published"]).await?;"#
        ),
        "{content}"
    );
    for engine in [DatabaseEngine::MySQL, DatabaseEngine::SQLite] {
        let content = generate_enum_file(&changes, &engine);
        assert!(!content.contains("create_enum_type"), "{content}");
    }
}

#[test]
fn test_create_file_escapes_enum_values_as_rust_literals_everywhere() {
    let value = "draft\"); compile_error!(\"injected\"); //";
    let schema = ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "status".to_string(),
            col_type: runique_cli::migration::ColumnType::String,
            enum_name: Some("Status".to_string()),
            enum_string_values: vec![value.to_string()],
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };

    let content = generate_create_file(&schema);
    let escaped_literal = format!("Alias::new({value:?}).into_iden()");
    assert!(
        content.contains(&escaped_literal),
        "every enum value inserted into generated Rust must be escaped as a string literal"
    );
}

#[test]
fn test_enum_file_postgres_down_supprime_le_type() {
    let schema = ParsedSchema {
        table_name: "articles".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "status".to_string(),
            col_type: runique_cli::migration::ColumnType::String,
            enum_string_values: vec!["Draft".to_string()],
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    // Le fichier de la table ne supprime plus le type : c'est celui de
    // l'enum qui le défait, après les tables.
    assert!(!generate_create_file(&schema).contains("drop_enum_type"));

    let changes = EnumChanges {
        change: EnumChange::Created,
        values: vec!["Draft".to_string()],
        ..status_enum()
    };
    let content = generate_enum_file(&changes, &DatabaseEngine::PostgreSQL);
    let down = content.split("async fn down").nth(1).unwrap_or("");
    assert!(
        down.contains(r#"runtime::drop_enum_type(manager, "status").await?;"#),
        "Postgres down doit supprimer le type : {content}"
    );
}

#[test]
fn test_enum_file_escapes_added_enum_values_as_rust_literals() {
    let value = "draft\"); compile_error!(\"injected\"); //";
    let changes = EnumChanges {
        added_values: vec![value.to_string()],
        renamed_values: vec![(value.to_string(), "safe".to_string())],
        ..status_enum()
    };

    let content = generate_enum_file(&changes, &DatabaseEngine::PostgreSQL);
    let added = format!(r#"add_enum_value(manager, "status", {value:?}).await?;"#);
    let renamed = format!(r#"rename_enum_value(manager, "status", {value:?}, "safe","#);
    assert!(
        content.contains(&added) && content.contains(&renamed),
        "enum values must be escaped before generating Rust code: {content}"
    );
    assert!(
        !content.contains("compile_error!(\"injected\")"),
        "{content}"
    );
}

#[test]
fn test_create_file_postgres_updated_at_has_no_trigger() {
    let schema = ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "updated_at".to_string(),
            col_type: runique_cli::migration::ColumnType::DateTime,
            has_default_now: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    // `updated_at` is the entity's job (`[auto_now_update]` → `before_save`):
    // no trigger in the migration any more.
    let content = generate_create_file(&schema);
    assert!(!content.contains("TRIGGER"), "{content}");
    assert!(!content.contains("execute_unprepared"), "{content}");
}

#[test]
fn test_create_file_mysql_updated_at_has_no_on_update() {
    let schema = ParsedSchema {
        table_name: "posts".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "updated_at".to_string(),
            col_type: runique_cli::migration::ColumnType::DateTime,
            has_default_now: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    // Same on MySQL: no `ON UPDATE` clause, the entity keeps `updated_at`.
    let content = generate_create_file(&schema);
    assert!(
        !content.contains("ON UPDATE CURRENT_TIMESTAMP"),
        "{content}"
    );
}

#[test]
fn test_create_file_col_nullable_unique_default_now() {
    let schema = ParsedSchema {
        table_name: "items".to_string(),
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "created_at".to_string(),
            col_type: runique_cli::migration::ColumnType::DateTime,
            nullable: true,
            unique: true,
            has_default_now: true,
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
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
        primary_key: Some(col("id", "Integer")),
        columns: vec![ParsedColumn {
            name: "kind".to_string(),
            col_type: runique_cli::migration::ColumnType::String,
            enum_string_values: vec!["A".to_string(), "B".to_string()],
            enum_name: Some("event_kind".to_string()),
            ..ParsedColumn::default()
        }],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
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
            col_type: runique_cli::migration::ColumnType::String,
            ..ParsedColumn::default()
        }),
        columns: vec![],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let content = generate_create_file(&schema);
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
        col_type: runique_cli::migration::ColumnType::Integer,
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "age".to_string(),
        col_type: runique_cli::migration::ColumnType::BigInteger,
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
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
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
        col_type: runique_cli::migration::ColumnType::String,
        nullable: true,
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "bio".to_string(),
        col_type: runique_cli::migration::ColumnType::String,
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
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("modify_column"),
        "nullable→not_null doit générer modify_column"
    );
}

#[test]
fn test_enum_file_value_adds() {
    let changes = EnumChanges {
        name: "article_status".to_string(),
        added_values: vec!["Archived".to_string()],
        ..status_enum()
    };
    let content = generate_enum_file(&changes, &DatabaseEngine::PostgreSQL);
    assert!(
        content
            .contains(r#"runtime::add_enum_value(manager, "article_status", "Archived").await?;"#),
        "une valeur ajoutée est donnée au type par le runtime : {content}"
    );
    // Sous MySQL et SQLite la colonne est du texte : rien à lancer.
    for engine in [DatabaseEngine::MySQL, DatabaseEngine::SQLite] {
        let content = generate_enum_file(&changes, &engine);
        assert!(!content.contains("add_enum_value"), "{content}");
    }
}

#[test]
fn test_enum_file_value_drops_warning() {
    let changes = EnumChanges {
        name: "article_status".to_string(),
        dropped_values: vec!["OldVal".to_string()],
        ..status_enum()
    };
    for engine in &ENGINES {
        let content = generate_enum_file(&changes, engine);
        let up = content.split("async fn down").next().unwrap_or("");
        assert!(
            up.contains("WARNING"),
            "une valeur retirée doit générer un WARNING dans up"
        );
        assert!(up.contains("OldVal"));
    }
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
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
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
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("create_index"),
        "UP doit contenir create_index pour index ajouté"
    );
    assert!(content.contains("idx_posts_title"));
    assert!(
        content.contains(".unique()"),
        "Index unique doit contenir .unique()"
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
            on_delete: runique_cli::migration::FkAction::Cascade,
            on_update: runique_cli::migration::FkAction::NoAction,
        }],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("drop_foreign_key"),
        "UP doit contenir drop_foreign_key pour FK supprimée"
    );
}

// ═══════════════════════════════════════════════════════════════
// generate_alter_file — enum CREATE TYPE sur ALTER Postgres
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_enum_add_column_leaves_the_type_to_the_enum_file() {
    let enum_col = ParsedColumn {
        name: "status".to_string(),
        col_type: runique_cli::migration::ColumnType::Enum,
        nullable: true,
        enum_name: Some("article_status".to_string()),
        enum_string_values: vec!["draft".to_string(), "published".to_string()],
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![enum_col],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    let up = content.split("async fn down").next().unwrap_or("");
    assert!(
        !up.contains("create_enum_type") && !up.contains("Type::create()"),
        "le type est créé par le fichier de l'enum, plus tôt dans le lancement : {content}"
    );
    assert!(
        up.contains("Alias::new(\"draft\")") && up.contains("Alias::new(\"published\")"),
        "la colonne ajoutée nomme les valeurs de son enum"
    );
    let down = content.split("async fn down").nth(1).unwrap_or("");
    assert!(
        !down.contains("drop_enum_type"),
        "le type est défait par le fichier de l'enum : {content}"
    );
}

#[test]
fn test_alter_file_enum_add_column_type_is_chosen_at_run_time() {
    let enum_col = ParsedColumn {
        name: "status".to_string(),
        col_type: runique_cli::migration::ColumnType::Enum,
        nullable: true,
        enum_name: Some("article_status".to_string()),
        enum_string_values: vec!["draft".to_string(), "published".to_string()],
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "articles".to_string(),
        added_columns: vec![enum_col],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    assert!(
        content.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "le type de la colonne est choisi au lancement : {content}"
    );
    assert!(!content.contains("Type::create()"));
}

// ═══════════════════════════════════════════════════════════════
// generate_alter_file — colonne string devenant un enum (bug J, 2026-09-01)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_alter_file_string_to_enum_postgres_casts_to_the_enum() {
    let old_col = ParsedColumn {
        name: "block_type".to_string(),
        col_type: runique_cli::migration::ColumnType::String,
        nullable: false,
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "block_type".to_string(),
        col_type: runique_cli::migration::ColumnType::String,
        nullable: false,
        enum_name: Some("CourBlockType".to_string()),
        enum_string_values: vec!["text".to_string(), "code".to_string()],
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "cour_block".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old_col, new_col)],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);

    // Never the broken old behaviour: no manual-migration warning, no bare
    // `ALTER TYPE ... ADD VALUE` on a type that doesn't exist yet.
    assert!(!content.contains("Manual migration required"));
    assert!(!content.contains("ADD VALUE"));

    let up = content.split("async fn down").next().unwrap_or("");
    assert!(
        !up.contains("create_enum_type") && !up.contains(".as_enum("),
        "le type est créé par le fichier de l'enum, plus tôt dans le lancement"
    );
    assert!(up.contains("Alias::new(\"text\")") && up.contains("Alias::new(\"code\")"));
    assert!(
        up.contains(
            ".using(Expr::col(Alias::new(\"block_type\")).cast_as(Alias::new(\"CourBlockType\")))"
        ),
        "UP doit caster la colonne existante vers le nouvel enum"
    );

    let down = content.split("async fn down").nth(1).unwrap_or("");
    assert!(
        down.contains(
            ".using(Expr::col(Alias::new(\"block_type\")).cast_as(Alias::new(\"text\")))"
        ),
        "DOWN doit recaster vers text en repassant en string"
    );
    assert!(
        down.contains("runtime::drop_enum_type(manager, \"courblocktype\").await?;"),
        "DOWN doit supprimer le type après avoir déplacé la colonne dessus"
    );
}

#[test]
fn test_alter_file_string_to_enum_column_type_is_runtime_guarded() {
    let old_col = ParsedColumn {
        name: "block_type".to_string(),
        col_type: runique_cli::migration::ColumnType::String,
        nullable: false,
        ..ParsedColumn::default()
    };
    let new_col = ParsedColumn {
        name: "block_type".to_string(),
        col_type: runique_cli::migration::ColumnType::String,
        nullable: false,
        enum_name: Some("CourBlockType".to_string()),
        enum_string_values: vec!["text".to_string(), "code".to_string()],
        ..ParsedColumn::default()
    };
    let changes = Changes {
        table_name: "cour_block".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![(old_col, new_col)],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        table: runique_cli::migration::TableChange::Altered,
        renames: Default::default(),
        renamed_columns: vec![],
    };
    let content = generate_alter_file(&changes);
    // Un seul fichier généré sert les 3 moteurs désormais — le CREATE TYPE est
    // toujours présent dans le texte, gardé par un check du backend réel à
    // l'exécution (MySQL ignore la branche, pas de type nommé séparé
    // nécessaire). Le `modify_column` via `.using()` est lui aussi gardé, mais à
    // l'exclusion de SQLite (sea-query panique sur tout modify_column sur ce
    // backend) — pas de perte fonctionnelle : SQLite n'a pas d'enum natif.
    assert!(
        content.contains("get_database_backend() == sea_orm::DbBackend::Postgres"),
        "doit checker le backend réel à l'exécution : {content}"
    );
    assert!(!content.contains("Type::create()"));
    assert!(!content.contains("Manual migration required"));
    assert!(content.contains("ColumnType::Enum"));
}

// ═══════════════════════════════════════════════════════════════
// generate_snapshot_file
// ═══════════════════════════════════════════════════════════════

#[test]
fn snapshot_contient_nom_table() {
    let content = generate_snapshot_file(&simple_schema("articles"));
    assert!(content.contains("articles"));
}

#[test]
fn snapshot_contient_struct_migration() {
    let content = generate_snapshot_file(&simple_schema("articles"));
    assert!(content.contains("pub struct Migration"));
    assert!(content.contains("impl MigrationTrait for Migration"));
}

#[test]
fn snapshot_contient_up_et_down() {
    let content = generate_snapshot_file(&simple_schema("articles"));
    assert!(content.contains("async fn up("));
    assert!(content.contains("async fn down("));
}

#[test]
fn snapshot_contient_colonnes() {
    let content = generate_snapshot_file(&simple_schema("articles"));
    assert!(content.contains("name"));
    assert!(content.contains("bio"));
}

#[test]
fn snapshot_avec_fk_contient_add_fk() {
    let content = generate_snapshot_file(&schema_with_fk());
    assert!(content.contains("user_id"));
    assert!(content.contains("users"));
}

// ── Foreign key indexes on MySQL (fixed 2026-10-07) ──────────────────
// InnoDB indexes every foreign key column itself; an explicit index replaced
// that one and then couldn't be dropped while the key existed (error 1553,
// `migrate down`/`reset` failed on MySQL). A foreign key's own index is now
// skipped there, checked at run time.

fn fk_index(table: &str, col: &str) -> ParsedIndex {
    ParsedIndex {
        name: format!("idx_{table}_{col}"),
        columns: vec![col.to_string()],
        unique: false,
    }
}

#[test]
fn alter_wraps_a_foreign_key_index_in_the_mysql_check() {
    let mut changes = simple_changes("posts");
    changes.added_indexes = vec![fk_index("posts", "user_id")];
    let out = generate_alter_file(&changes);
    let up = &out[..out.find("async fn down").expect("down")];
    let down = &out[out.find("async fn down").expect("down")..];
    for (part, op) in [(up, "create_index"), (down, "drop_index")] {
        let guard = part.find("KEY_COLUMN_USAGE").expect("checked on MySQL");
        let stmt = part.find(op).expect(op);
        assert!(guard < stmt, "the check comes before {op}");
        assert!(part.contains(r#"eq("user_id")"#) && part.contains("if !backs_fk"));
    }
}

#[test]
fn other_indexes_are_not_wrapped() {
    let mut changes = simple_changes("posts");
    changes.added_indexes = vec![
        ParsedIndex {
            name: "idx_posts_user_id_title".to_string(),
            columns: vec!["user_id".to_string(), "title".to_string()],
            unique: false,
        },
        ParsedIndex {
            name: "idx_posts_slug".to_string(),
            columns: vec!["slug".to_string()],
            unique: true,
        },
    ];
    let out = generate_alter_file(&changes);
    assert!(out.contains("idx_posts_user_id_title") && out.contains("idx_posts_slug"));
    assert!(!out.contains("KEY_COLUMN_USAGE"), "{out}");
}

#[test]
fn create_file_wraps_its_foreign_key_index_but_the_snapshot_does_not() {
    let mut schema = schema_with_fk();
    schema.indexes = vec![fk_index("posts", "user_id")];
    assert!(generate_create_file(&schema).contains("KEY_COLUMN_USAGE"));
    assert!(
        !generate_snapshot_file(&schema).contains("KEY_COLUMN_USAGE"),
        "snapshots stay plain statements for parser_seaorm"
    );
}

// ═══════════════════════════════════════════════════════════════
// Written from cargo-mutants survivors (2026-10-08)
// ═══════════════════════════════════════════════════════════════

fn down_part(file: &str) -> &str {
    file.split("async fn down").nth(1).expect("a down() body")
}

#[test]
fn test_snapshot_down_drops_its_foreign_keys_and_indexes() {
    let mut schema = schema_with_fk();
    schema.indexes = vec![ParsedIndex {
        name: "idx_posts_title".to_string(),
        columns: vec!["title".to_string()],
        unique: false,
    }];
    let down = down_part(&generate_snapshot_file(&schema)).to_string();
    assert!(
        down.contains(".drop_foreign_key(") && down.contains(".name(\"posts_user_id_users_fkey\")"),
        "{down}"
    );
    assert!(
        down.contains(
            ".drop_index(Index::drop().name(\"idx_posts_title\").table(Alias::new(\"posts\"))"
        ),
        "{down}"
    );

    let plain = down_part(&generate_snapshot_file(&simple_schema("plain"))).to_string();
    assert!(
        !plain.contains("drop_foreign_key") && !plain.contains("drop_index"),
        "{plain}"
    );
}

#[test]
fn test_create_in_cycle_keeps_a_forward_key_out_of_the_table_body() {
    use runique_cli::migration::{CycleKeys, generate_create_file_in_cycle};
    let schema = schema_with_fk();
    let fk_name = ".name(\"posts_user_id_users_fkey\")";
    let sqlite_only = "== sea_orm::DbBackend::Sqlite";

    // Forward key: only in the SQLite branch, never also inline in CREATE TABLE.
    let forward = generate_create_file_in_cycle(
        &schema,
        &CycleKeys {
            forward: vec![&schema.foreign_keys[0]],
            closing: vec![],
        },
    );
    assert_eq!(forward.matches(fk_name).count(), 1, "{forward}");
    assert!(
        forward.find(sqlite_only).unwrap() < forward.find(fk_name).unwrap(),
        "{forward}"
    );

    // Not part of a cycle: inlined in CREATE TABLE, no SQLite branch.
    let inline = generate_create_file_in_cycle(&schema, &CycleKeys::default());
    assert_eq!(inline.matches(fk_name).count(), 1, "{inline}");
    assert!(!inline.contains(sqlite_only), "{inline}");
}
