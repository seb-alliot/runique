// Tests pour ModelSchema et SchemaDiff

use runique::forms::Forms;
use runique::migration::{
    column::ColumnDef,
    foreign_key::ForeignKeyDef,
    hooks::HooksDef,
    index::IndexDef,
    primary_key::PrimaryKeyDef,
    relation::RelationDef,
    schema::{ModelSchema, SchemaDiff},
};

// ═══════════════════════════════════════════════════════════════
// ModelSchema::new() — conversion PascalCase → snake_case
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_new_pascal_case_simple() {
    let s = ModelSchema::new("User");
    assert_eq!(s.model_name, "User");
    assert_eq!(s.table_name, "user");
}

#[test]
fn test_schema_new_pascal_case_compose() {
    let s = ModelSchema::new("BlogPost");
    assert_eq!(s.model_name, "BlogPost");
    assert_eq!(s.table_name, "blog_post");
}

#[test]
fn test_schema_new_defauts() {
    let s = ModelSchema::new("Article");
    assert!(s.primary_key.is_none());
    assert!(s.columns.is_empty());
    assert!(s.foreign_keys.is_empty());
    assert!(s.relations.is_empty());
    assert!(s.indexes.is_empty());
    assert!(s.hooks.is_none());
    assert!(s.schema.is_none());
}

// ═══════════════════════════════════════════════════════════════
// Builders — table_name, schema
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_table_name_override() {
    let s = ModelSchema::new("User").table_name("custom_users");
    assert_eq!(s.table_name, "custom_users");
}

#[test]
fn test_schema_set_schema() {
    let s = ModelSchema::new("User").schema("public");
    assert_eq!(s.schema.as_deref(), Some("public"));
}

// ═══════════════════════════════════════════════════════════════
// Builders — primary_key, column, foreign_key, relation, index, hooks
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_primary_key() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id"));
    assert!(s.primary_key.is_some());
    assert_eq!(s.primary_key.unwrap().name, "id");
}

#[test]
fn test_schema_column_ajout() {
    let s = ModelSchema::new("User").column(ColumnDef::new("username").string());
    assert_eq!(s.columns.len(), 1);
    assert_eq!(s.columns[0].name, "username");
}

#[test]
fn test_schema_multi_columns() {
    let s = ModelSchema::new("Post")
        .column(ColumnDef::new("title").string())
        .column(ColumnDef::new("body").text());
    assert_eq!(s.columns.len(), 2);
}

#[test]
fn test_schema_foreign_key_ajout() {
    let s = ModelSchema::new("Post").foreign_key(ForeignKeyDef::new("user_id").references("users"));
    assert_eq!(s.foreign_keys.len(), 1);
    assert_eq!(s.foreign_keys[0].from_column, "user_id");
}

#[test]
fn test_schema_relation_ajout() {
    let s = ModelSchema::new("Post").relation(RelationDef::has_one("profile"));
    assert_eq!(s.relations.len(), 1);
}

#[test]
fn test_schema_index_ajout() {
    let s = ModelSchema::new("User").index(IndexDef::new(vec!["email"]).unique());
    assert_eq!(s.indexes.len(), 1);
    assert!(s.indexes[0].unique);
}

#[test]
fn test_schema_hooks_ajout() {
    let s = ModelSchema::new("User").hooks(HooksDef::new().before_save(0, "handler"));
    assert!(s.hooks.is_some());
    assert_eq!(s.hooks.unwrap().hooks.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// build()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_build_sans_pk_retourne_err() {
    let result = ModelSchema::new("User").build();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing primary key"));
}

#[test]
fn test_schema_build_avec_pk_retourne_ok() {
    let result = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .build();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().model_name, "User");
}

// ═══════════════════════════════════════════════════════════════
// diff()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_identiques_est_vide() {
    let s1 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let s2 = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = s1.diff(&s2);
    assert!(diff.is_empty());
}

#[test]
fn test_schema_diff_colonne_ajoutee() {
    let old = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let new = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.added_columns.len(), 1);
    assert_eq!(diff.added_columns[0].name, "email");
    assert!(diff.dropped_columns.is_empty());
}

#[test]
fn test_schema_diff_colonne_supprimee() {
    let old = ModelSchema::new("User")
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("email").string());
    let new = ModelSchema::new("User").column(ColumnDef::new("name").string());
    let diff = old.diff(&new);
    assert!(!diff.is_empty());
    assert_eq!(diff.dropped_columns.len(), 1);
    assert_eq!(diff.dropped_columns[0], "email");
    assert!(diff.added_columns.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// SchemaDiff
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_diff_new_est_vide() {
    let diff = SchemaDiff::new("users");
    assert_eq!(diff.table_name, "users");
    assert!(diff.is_empty());
    assert!(diff.added_columns.is_empty());
    assert!(diff.dropped_columns.is_empty());
    assert!(diff.modified_columns.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// to_migration() — ne panique pas
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_migration_compile() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string());
    let _ = s.to_migration();
}

#[test]
fn test_schema_to_migration_avec_fk() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string())
        .foreign_key(ForeignKeyDef::new("user_id").references("users"));
    let _ = s.to_migration();
}

// ═══════════════════════════════════════════════════════════════
// to_model() — contenu de la chaîne générée
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_contient_struct_model() {
    let s = ModelSchema::new("Article").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(code.contains("pub struct Model"));
}

#[test]
fn test_schema_to_model_contient_table_name() {
    let s = ModelSchema::new("BlogPost").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(
        code.contains("blog_post"),
        "doit contenir le nom de table snake_case"
    );
}

#[test]
fn test_schema_to_model_pk_i32() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id").i32());
    let code = s.to_model();
    assert!(code.contains("i32"));
}

#[test]
fn test_schema_to_model_pk_i64() {
    let s = ModelSchema::new("BigTable").primary_key(PrimaryKeyDef::new("id").i64());
    let code = s.to_model();
    assert!(code.contains("i64"));
}

#[test]
fn test_schema_to_model_colonne_nullable() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("bio").text().nullable());
    let code = s.to_model();
    assert!(
        code.contains("Option<"),
        "colonne nullable doit générer Option<T>"
    );
}

#[test]
fn test_schema_to_model_colonne_ignoree_absente() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("internal_cache").string().ignore());
    let code = s.to_model();
    assert!(
        !code.contains("internal_cache"),
        "champ ignoré ne doit pas apparaître"
    );
}

#[test]
fn test_schema_to_model_contient_active_model_behavior() {
    let s = ModelSchema::new("User").primary_key(PrimaryKeyDef::new("id"));
    let code = s.to_model();
    assert!(code.contains("ActiveModelBehavior"));
}

// ═══════════════════════════════════════════════════════════════
// Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_clone() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("name").string());
    let cloned = s.clone();
    assert_eq!(cloned.model_name, "User");
    assert_eq!(cloned.table_name, "user");
    assert_eq!(cloned.columns.len(), 1);
    assert!(cloned.primary_key.is_some());
}

// ═══════════════════════════════════════════════════════════════
// to_model() — relations
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_belongs_to() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::belongs_to("User", "user_id", "id"));
    let code = s.to_model();
    assert!(
        code.contains("belongs_to"),
        "BelongsTo doit générer belongs_to"
    );
    assert!(code.contains("user_id") || code.contains("UserId"));
}

#[test]
fn test_schema_to_model_has_many() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::has_many("post"));
    let code = s.to_model();
    assert!(code.contains("has_many"), "HasMany doit générer has_many");
}

#[test]
fn test_schema_to_model_has_one() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::has_one("profile"));
    let code = s.to_model();
    assert!(code.contains("has_many") || code.contains("has_one") || code.contains("Profile"));
}

#[test]
fn test_schema_to_model_many_to_many() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .relation(RelationDef::many_to_many("tag", "post_tag"));
    let code = s.to_model();
    assert!(
        code.contains("many_to_many") || code.contains("via"),
        "ManyToMany doit générer via"
    );
}

// ═══════════════════════════════════════════════════════════════
// to_model() — col_to_rust_type() variants
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_model_float_col() {
    let s = ModelSchema::new("Metrics")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("score").float());
    let code = s.to_model();
    assert!(code.contains("f32"), "float doit générer f32");
}

#[test]
fn test_schema_to_model_double_col() {
    let s = ModelSchema::new("Metrics")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("lat").double());
    let code = s.to_model();
    assert!(code.contains("f64"), "double doit générer f64");
}

#[test]
fn test_schema_to_model_boolean_col() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("active").boolean());
    let code = s.to_model();
    assert!(code.contains("bool"), "boolean doit générer bool");
}

#[test]
fn test_schema_to_model_date_col() {
    let s = ModelSchema::new("Event")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("event_date").date());
    let code = s.to_model();
    assert!(code.contains("NaiveDate"), "date doit générer NaiveDate");
}

#[test]
fn test_schema_to_model_uuid_col() {
    let s = ModelSchema::new("Token")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("uuid_val").uuid());
    let code = s.to_model();
    assert!(code.contains("Uuid"), "uuid doit générer Uuid");
}

#[test]
fn test_schema_to_model_json_col() {
    let s = ModelSchema::new("Config")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("data").json());
    let code = s.to_model();
    assert!(
        code.contains("serde_json::Value"),
        "json doit générer serde_json::Value"
    );
}

#[test]
fn test_schema_to_model_pk_uuid() {
    let s = ModelSchema::new("Token").primary_key(PrimaryKeyDef::new("id").uuid());
    let code = s.to_model();
    assert!(code.contains("Uuid"), "PK uuid doit générer Uuid");
}

// ═══════════════════════════════════════════════════════════════
// auto_now_columns / auto_now_update_columns / has_auto_timestamps
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_auto_now_columns() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now())
        .column(ColumnDef::new("updated_at").datetime().auto_now_update())
        .column(ColumnDef::new("title").string());
    let auto = s.auto_now_columns();
    assert_eq!(auto.len(), 1);
    assert_eq!(auto[0].name, "created_at");
}

#[test]
fn test_schema_auto_now_update_columns() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now())
        .column(ColumnDef::new("updated_at").datetime().auto_now_update());
    let auto_update = s.auto_now_update_columns();
    assert_eq!(auto_update.len(), 1);
    assert_eq!(auto_update[0].name, "updated_at");
}

#[test]
fn test_schema_has_auto_timestamps_true() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("created_at").datetime().auto_now());
    assert!(s.has_auto_timestamps());
}

#[test]
fn test_schema_has_auto_timestamps_false() {
    let s = ModelSchema::new("Post")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("title").string());
    assert!(!s.has_auto_timestamps());
}

// ═══════════════════════════════════════════════════════════════
// fill_form()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_fill_form_all_fields() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, None);
    // 2 colonnes ajoutées (PK exclue automatiquement)
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_exclude() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("password").string());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, None, Some(&["password"]));
    assert_eq!(form.fields.len() - before, 2);
}

#[test]
fn test_schema_fill_form_with_whitelist() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("username").string())
        .column(ColumnDef::new("email").string())
        .column(ColumnDef::new("bio").text());
    let mut form = Forms::new("dummy_token");
    let before = form.fields.len();
    s.fill_form(&mut form, Some(&["username", "email"]), None);
    assert_eq!(form.fields.len() - before, 2);
}

// ═══════════════════════════════════════════════════════════════
// to_migration() — avec colonnes ignorées
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_schema_to_migration_ignored_col_skipped() {
    let s = ModelSchema::new("User")
        .primary_key(PrimaryKeyDef::new("id"))
        .column(ColumnDef::new("name").string())
        .column(ColumnDef::new("cache_internal").string().ignore());
    // Ne doit pas paniquer et ignorer le champ
    let _ = s.to_migration();
}
