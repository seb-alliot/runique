//! Tests — Migration Parser DSL (model!)
//! Couvre : parse_schema_from_source — tous les types de champs, toutes les options,
//!          types de PK, blocs relations/meta, cas invalides.

use runique::migration::utils::parser_builder::parse_schema_from_source;

// ═══════════════════════════════════════════════════════════════
// Sources DSL de référence
// ═══════════════════════════════════════════════════════════════

fn full_types_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        AllTypes,
        table: "all_types",
        pk: id => i32,
        fields: {
            f_string: String,
            f_text: text,
            f_char: char,
            f_varchar: varchar,
            f_i8: i8,
            f_i16: i16,
            f_i32: i32,
            f_integer: integer,
            f_i64: i64,
            f_big_integer: big_integer,
            f_u32: u32,
            f_u64: u64,
            f_f32: f32,
            f_f64: f64,
            f_decimal: decimal,
            f_bool: bool,
            f_date: date,
            f_time: time,
            f_datetime: datetime,
            f_timestamp: timestamp,
            f_timestamp_tz: timestamp_tz,
            f_uuid: uuid,
            f_json: json,
            f_json_binary: json_binary,
            f_binary: binary,
            f_blob: blob,
            f_inet: inet,
            f_cidr: cidr,
            f_mac_address: mac_address,
            f_interval: interval,
        }
    }
    "#
}

fn options_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        OptionsModel,
        table: "options_table",
        pk: id => i32,
        fields: {
            required_field: String,
            nullable_field: String [nullable],
            unique_field: String [unique],
            both_field: String [nullable, unique],
            auto_now_field: datetime [auto_now],
            auto_now_update_field: datetime [auto_now_update],
            readonly_field: String [readonly],
            max_len_field: String [max_len(255)],
            min_len_field: String [min_len(3)],
            select_as_field: String [select_as(some_expr)],
            label_field: String [label(Mon_Label)],
            help_field: String [help(Aide_Saisie)],
        }
    }
    "#
}

fn pk_i64_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        BigTable,
        table: "big_table",
        pk: id => i64,
        fields: {
            data: String,
        }
    }
    "#
}

fn pk_uuid_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        UuidTable,
        table: "uuid_table",
        pk: slug => uuid,
        fields: {
            title: String,
        }
    }
    "#
}

fn relations_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Post,
        table: "posts",
        pk: id => i32,
        fields: {
            title: String,
            user_id: i32,
        },
        relations: {
            belongs_to: User via user_id,
            has_many: Comment,
        }
    }
    "#
}

fn relations_cascade_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Comment,
        table: "comments",
        pk: id => i32,
        fields: {
            body: String,
            post_id: i32,
            author_id: i32,
        },
        relations: {
            belongs_to: Post via post_id [cascade],
            belongs_to: EihwazUsers via author_id [cascade, restrict],
            has_one: CommentMeta,
        }
    }
    "#
}

fn meta_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        Article,
        table: "articles",
        pk: id => i32,
        fields: {
            title: String,
            slug: String [unique],
        },
        meta: {
            ordering: [title, -created_at],
            unique_together: [(title, slug)],
            verbose_name: "Article",
        }
    }
    "#
}

fn full_model_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        UserProfile,
        table: "user_profiles",
        pk: id => i32,
        fields: {
            username: String [unique],
            email: String [unique],
            bio: text [nullable],
            age: i32,
            score: f64,
            is_active: bool,
            birth_date: date [nullable],
            created_at: datetime [auto_now],
            updated_at: datetime [auto_now_update],
            avatar_data: binary [nullable],
            metadata: json [nullable],
            ip_addr: inet [nullable],
            cache_key: String [readonly],
        },
        relations: {
            has_many: Post,
        },
        meta: {
            ordering: [-created_at],
            verbose_name: "User Profile",
        }
    }
    "#
}

// ═══════════════════════════════════════════════════════════════
// Parsing de base
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_returns_some_for_valid_dsl() {
    assert!(parse_schema_from_source(full_types_source()).is_some());
}

#[test]
fn test_parse_empty_returns_none() {
    assert!(parse_schema_from_source("").is_none());
}

#[test]
fn test_parse_no_model_macro_returns_none() {
    let src = r#"pub struct Foo { pub id: i32 }"#;
    assert!(parse_schema_from_source(src).is_none());
}

#[test]
fn test_parse_invalid_rust_returns_none() {
    assert!(parse_schema_from_source("let x = !!@@@#").is_none());
}

// ═══════════════════════════════════════════════════════════════
// Nom de table et PK
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_table_name_preserved() {
    let s = parse_schema_from_source(full_types_source()).unwrap();
    assert_eq!(s.table_name, "all_types");
}

#[test]
fn test_pk_name_i32() {
    let s = parse_schema_from_source(full_types_source()).unwrap();
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "Integer");
}

#[test]
fn test_pk_name_i64() {
    let s = parse_schema_from_source(pk_i64_source()).unwrap();
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "BigInteger");
}

#[test]
fn test_pk_uuid() {
    let s = parse_schema_from_source(pk_uuid_source()).unwrap();
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "slug");
    assert_eq!(pk.col_type, "Uuid");
}

#[test]
fn test_pk_not_nullable() {
    let s = parse_schema_from_source(pk_i64_source()).unwrap();
    assert!(!s.primary_key.unwrap().nullable);
}

// ═══════════════════════════════════════════════════════════════
// Mapping des types de champs — types scalar classiques
// ═══════════════════════════════════════════════════════════════

fn col_type(src: &str, field: &str) -> String {
    let s = parse_schema_from_source(src).unwrap();
    s.columns
        .iter()
        .find(|c| c.name == field)
        .unwrap_or_else(|| panic!("champ '{field}' introuvable"))
        .col_type
        .clone()
}

#[test]
fn test_type_string_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_string"), "String");
}

#[test]
fn test_type_text_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_text"), "String");
}

#[test]
fn test_type_char_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_char"), "String");
}

#[test]
fn test_type_varchar_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_varchar"), "String");
}

#[test]
fn test_type_i8_maps_to_tinyinteger() {
    assert_eq!(col_type(full_types_source(), "f_i8"), "TinyInteger");
}

#[test]
fn test_type_i16_maps_to_smallinteger() {
    assert_eq!(col_type(full_types_source(), "f_i16"), "SmallInteger");
}

#[test]
fn test_type_i32_maps_to_integer() {
    assert_eq!(col_type(full_types_source(), "f_i32"), "Integer");
}

#[test]
fn test_type_integer_maps_to_integer() {
    assert_eq!(col_type(full_types_source(), "f_integer"), "Integer");
}

#[test]
fn test_type_i64_maps_to_biginteger() {
    assert_eq!(col_type(full_types_source(), "f_i64"), "BigInteger");
}

#[test]
fn test_type_big_integer_maps_to_biginteger() {
    assert_eq!(col_type(full_types_source(), "f_big_integer"), "BigInteger");
}

#[test]
fn test_type_u32_maps_to_unsigned() {
    assert_eq!(col_type(full_types_source(), "f_u32"), "Unsigned");
}

#[test]
fn test_type_u64_maps_to_bigunsigned() {
    assert_eq!(col_type(full_types_source(), "f_u64"), "BigUnsigned");
}

#[test]
fn test_type_f32_maps_to_float() {
    assert_eq!(col_type(full_types_source(), "f_f32"), "Float");
}

#[test]
fn test_type_f64_maps_to_double() {
    assert_eq!(col_type(full_types_source(), "f_f64"), "Double");
}

#[test]
fn test_type_decimal_maps_to_decimal() {
    assert_eq!(col_type(full_types_source(), "f_decimal"), "Decimal");
}

#[test]
fn test_type_bool_maps_to_boolean() {
    assert_eq!(col_type(full_types_source(), "f_bool"), "Boolean");
}

#[test]
fn test_type_date_maps_to_date() {
    assert_eq!(col_type(full_types_source(), "f_date"), "Date");
}

#[test]
fn test_type_time_maps_to_time() {
    assert_eq!(col_type(full_types_source(), "f_time"), "Time");
}

#[test]
fn test_type_datetime_maps_to_datetime() {
    assert_eq!(col_type(full_types_source(), "f_datetime"), "DateTime");
}

#[test]
fn test_type_timestamp_maps_to_datetime() {
    assert_eq!(col_type(full_types_source(), "f_timestamp"), "DateTime");
}

#[test]
fn test_type_uuid_maps_to_uuid() {
    assert_eq!(col_type(full_types_source(), "f_uuid"), "Uuid");
}

#[test]
fn test_type_json_maps_to_json() {
    assert_eq!(col_type(full_types_source(), "f_json"), "Json");
}

#[test]
fn test_type_binary_maps_to_binary() {
    assert_eq!(col_type(full_types_source(), "f_binary"), "Binary");
}

#[test]
fn test_type_blob_maps_to_binary() {
    assert_eq!(col_type(full_types_source(), "f_blob"), "Binary");
}

// ═══════════════════════════════════════════════════════════════
// Nouveaux types AST (PostgreSQL / spéciaux)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_type_timestamp_tz_maps_to_timestampwithtimezone() {
    assert_eq!(
        col_type(full_types_source(), "f_timestamp_tz"),
        "TimestampWithTimeZone"
    );
}

#[test]
fn test_type_json_binary_maps_to_json() {
    assert_eq!(col_type(full_types_source(), "f_json_binary"), "Json");
}

#[test]
fn test_type_inet_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_inet"), "String");
}

#[test]
fn test_type_cidr_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_cidr"), "String");
}

#[test]
fn test_type_mac_address_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_mac_address"), "String");
}

#[test]
fn test_type_interval_maps_to_string() {
    assert_eq!(col_type(full_types_source(), "f_interval"), "String");
}

// ═══════════════════════════════════════════════════════════════
// Comptage des champs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_field_count_all_types() {
    let s = parse_schema_from_source(full_types_source()).unwrap();
    // 30 champs déclarés
    assert_eq!(s.columns.len(), 30);
}

#[test]
fn test_field_count_options_model() {
    let s = parse_schema_from_source(options_source()).unwrap();
    // 12 champs déclarés
    assert_eq!(s.columns.len(), 12);
}

// ═══════════════════════════════════════════════════════════════
// Options des champs — nullable, unique, ignored
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_option_required_field_not_nullable() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "required_field")
        .unwrap();
    assert!(!f.nullable);
    assert!(!f.unique);
    assert!(!f.ignored);
}

#[test]
fn test_option_nullable_field() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "nullable_field")
        .unwrap();
    assert!(f.nullable, "nullable_field doit être nullable");
}

#[test]
fn test_option_unique_field() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s.columns.iter().find(|c| c.name == "unique_field").unwrap();
    assert!(f.unique, "unique_field doit être unique");
    assert!(!f.nullable);
}

#[test]
fn test_option_nullable_and_unique() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s.columns.iter().find(|c| c.name == "both_field").unwrap();
    assert!(f.nullable);
    assert!(f.unique);
}

#[test]
fn test_option_auto_now_is_ignored_and_datetime() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "auto_now_field")
        .unwrap();
    assert!(!f.ignored, "auto_now ne doit plus marquer le champ ignored");
    assert_eq!(f.col_type, "DateTime");
    // auto_now n'implique pas forcément nullable, on ne teste plus ce point
}

#[test]
fn test_option_auto_now_update_is_ignored_and_datetime() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "auto_now_update_field")
        .unwrap();
    assert!(
        !f.ignored,
        "auto_now_update ne doit plus marquer le champ ignored"
    );
    assert_eq!(f.col_type, "DateTime");
    // auto_now_update n'implique pas forcément nullable, on ne teste plus ce point
}

#[test]
fn test_option_readonly_is_ignored() {
    let s = parse_schema_from_source(options_source()).unwrap();
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "readonly_field")
        .unwrap();
    assert!(f.ignored, "readonly doit marquer le champ ignored");
}

#[test]
fn test_option_max_len_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "max_len_field"));
}

#[test]
fn test_option_min_len_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "min_len_field"));
}

#[test]
fn test_option_select_as_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "select_as_field"));
}

#[test]
fn test_option_label_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "label_field"));
}

#[test]
fn test_option_help_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "help_field"));
}

// ═══════════════════════════════════════════════════════════════
// Bloc relations — parsing sans crash
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_with_relations_returns_some() {
    assert!(
        parse_schema_from_source(relations_source()).is_some(),
        "un modèle avec relations doit parser correctement"
    );
}

#[test]
fn test_parse_relations_table_name() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    assert_eq!(s.table_name, "posts");
}

#[test]
fn test_parse_relations_fields_intact() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "title"));
    assert!(s.columns.iter().any(|c| c.name == "user_id"));
}

// ═══════════════════════════════════════════════════════════════
// Bloc meta — parsing sans crash
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_with_meta_returns_some() {
    assert!(
        parse_schema_from_source(meta_source()).is_some(),
        "un modèle avec meta doit parser correctement"
    );
}

#[test]
fn test_parse_meta_table_name() {
    let s = parse_schema_from_source(meta_source()).unwrap();
    assert_eq!(s.table_name, "articles");
}

#[test]
fn test_parse_meta_fields_intact() {
    let s = parse_schema_from_source(meta_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "title"));
    let slug = s.columns.iter().find(|c| c.name == "slug").unwrap();
    assert!(slug.unique);
}

// ═══════════════════════════════════════════════════════════════
// Modèle complet (relations + meta + tous les cas)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_full_model_parses_successfully() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    assert_eq!(s.table_name, "user_profiles");
}

#[test]
fn test_full_model_pk() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    let pk = s.primary_key.as_ref().unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "Integer");
}

#[test]
fn test_full_model_unique_fields() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    let email = s.columns.iter().find(|c| c.name == "email").unwrap();
    assert!(email.unique);
    let username = s.columns.iter().find(|c| c.name == "username").unwrap();
    assert!(username.unique);
}

#[test]
fn test_full_model_nullable_fields() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    for name in &["bio", "birth_date", "avatar_data", "metadata", "ip_addr"] {
        let f = s.columns.iter().find(|c| c.name == *name).unwrap();
        assert!(f.nullable, "{} doit être nullable", name);
    }
}

#[test]
fn test_full_model_ignored_fields() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    {
        let name = &"cache_key";
        let f = s.columns.iter().find(|c| c.name == *name).unwrap();
        assert!(f.ignored, "{} doit être ignored", name);
    }
    for name in &["created_at", "updated_at"] {
        let f = s.columns.iter().find(|c| c.name == *name).unwrap();
        assert!(!f.ignored, "{} ne doit PAS être ignored", name);
    }
}

#[test]
fn test_full_model_type_mappings() {
    let s = parse_schema_from_source(full_model_source()).unwrap();
    let age = s.columns.iter().find(|c| c.name == "age").unwrap();
    assert_eq!(age.col_type, "Integer");
    let score = s.columns.iter().find(|c| c.name == "score").unwrap();
    assert_eq!(score.col_type, "Double");
    let is_active = s.columns.iter().find(|c| c.name == "is_active").unwrap();
    assert_eq!(is_active.col_type, "Boolean");
    let metadata = s.columns.iter().find(|c| c.name == "metadata").unwrap();
    assert_eq!(metadata.col_type, "Json");
    let ip_addr = s.columns.iter().find(|c| c.name == "ip_addr").unwrap();
    assert_eq!(ip_addr.col_type, "String"); // inet → String
}

// ═══════════════════════════════════════════════════════════════
// Isolation entre modèles
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_two_models_are_independent() {
    let a = parse_schema_from_source(pk_i64_source()).unwrap();
    let b = parse_schema_from_source(pk_uuid_source()).unwrap();
    assert_ne!(a.table_name, b.table_name);
    assert_ne!(
        a.primary_key.unwrap().col_type,
        b.primary_key.unwrap().col_type
    );
}

#[test]
fn test_multiple_calls_same_source_return_equal_results() {
    let s1 = parse_schema_from_source(relations_source()).unwrap();
    let s2 = parse_schema_from_source(relations_source()).unwrap();
    assert_eq!(s1.table_name, s2.table_name);
    assert_eq!(s1.columns.len(), s2.columns.len());
}

// ═══════════════════════════════════════════════════════════════
// Relations — FK générées depuis belongs_to
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_belongs_to_genere_une_fk() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    assert_eq!(
        s.foreign_keys.len(),
        1,
        "belongs_to doit générer exactement 1 FK"
    );
}

#[test]
fn test_belongs_to_from_column() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.from_column, "user_id");
}

#[test]
fn test_belongs_to_to_table_pascal_vers_snake() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    let fk = &s.foreign_keys[0];
    // User → user
    assert_eq!(fk.to_table, "user");
}

#[test]
fn test_belongs_to_to_column_defaut_id() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.to_column, "id");
}

#[test]
fn test_belongs_to_on_delete_no_action_par_defaut() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.on_delete, "NoAction");
    assert_eq!(fk.on_update, "NoAction");
}

#[test]
fn test_has_many_ne_genere_pas_de_fk() {
    let s = parse_schema_from_source(relations_source()).unwrap();
    // only belongs_to generates FK — has_many does not
    assert!(
        s.foreign_keys
            .iter()
            .all(|fk| fk.from_column != "comment_id")
    );
}

#[test]
fn test_belongs_to_cascade_on_delete() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    let fk = s
        .foreign_keys
        .iter()
        .find(|fk| fk.from_column == "post_id")
        .unwrap();
    assert_eq!(fk.on_delete, "Cascade");
    assert_eq!(fk.on_update, "NoAction");
}

#[test]
fn test_belongs_to_cascade_et_restrict() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    let fk = s
        .foreign_keys
        .iter()
        .find(|fk| fk.from_column == "author_id")
        .unwrap();
    assert_eq!(fk.on_delete, "Cascade");
    assert_eq!(fk.on_update, "Restrict");
}

#[test]
fn test_plusieurs_belongs_to_generent_plusieurs_fk() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    assert_eq!(s.foreign_keys.len(), 2, "2 belongs_to → 2 FK");
}

#[test]
fn test_belongs_to_table_cible_pascal_to_snake_composee() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    let fk = s
        .foreign_keys
        .iter()
        .find(|fk| fk.from_column == "author_id")
        .unwrap();
    // EihwazUsers → eihwaz_users
    assert_eq!(fk.to_table, "eihwaz_users");
}

#[test]
fn test_has_one_ne_genere_pas_de_fk() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    // has_one: CommentMeta ne doit pas créer de FK
    assert!(
        s.foreign_keys
            .iter()
            .all(|fk| fk.to_table != "comment_meta")
    );
}

#[test]
fn test_relations_champs_intacts_avec_cascade() {
    let s = parse_schema_from_source(relations_cascade_source()).unwrap();
    assert!(s.columns.iter().any(|c| c.name == "body"));
    assert!(s.columns.iter().any(|c| c.name == "post_id"));
    assert!(s.columns.iter().any(|c| c.name == "author_id"));
}
