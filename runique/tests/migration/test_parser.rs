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
        {
            f_text: text [nullable],
            f_textarea: textarea [nullable],
            f_char: char [nullable],
            f_i8: i8 [nullable],
            f_i16: i16 [nullable],
            f_i32: int [nullable],
            f_i64: bigint [nullable],
            f_u32: u32 [nullable],
            f_u64: u64 [nullable],
            f_f32: f32 [nullable],
            f_f64: float [nullable],
            f_decimal: decimal [nullable],
            f_bool: bool [nullable],
            f_date: date [nullable],
            f_time: time [nullable],
            f_datetime: datetime [nullable],
            f_timestamp: timestamp [nullable],
            f_timestamp_tz: timestamp_tz [nullable],
            f_uuid: uuid [nullable],
            f_json: json [nullable],
            f_json_binary: json_binary [nullable],
            f_binary: binary [nullable],
            f_var_binary: var_binary [nullable],
            f_blob: blob [nullable],
            f_inet: ip [nullable],
            f_cidr: cidr [nullable],
            f_mac_address: mac_address [nullable],
            f_interval: interval [nullable],
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
        {
            required_field: text [required],
            nullable_field: text [nullable],
            unique_field: text [required, unique],
            both_field: text [nullable, unique],
            auto_now_field: datetime [auto_now],
            auto_now_update_field: datetime [auto_now_update],
            readonly_field: text [nullable, readonly],
            max_len_field: text [nullable, max_length: 255],
            min_len_field: text [nullable, min_length: 3],
            label_field: text [nullable, label: "Mon label"],
            renamed_field: text [nullable, renamed_from: "old_name"],
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
        {
            data: text [nullable],
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
        {
            title: text [nullable],
        }
    }
    "#
}

fn pk_generic_alias_source() -> &'static str {
    r#"
    use runique::prelude::*;
    model! {
        AliasTable,
        table: "alias_table",
        pk: id => Pk,
        {
            data: text [nullable],
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
        {
            title: text [required],
            user_id: int [required],
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
        {
            body: text [nullable],
            post_id: int [required],
            author_id: int [required],
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
        {
            title: text [required],
            slug: text [required, unique],
        },
        meta: {
            ordering: [title, -slug],
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
        {
            username: text [required, unique],
            email: email [required, unique],
            bio: textarea [nullable],
            age: int [required],
            score: float [required],
            is_active: bool [nullable, default: true],
            birth_date: date [nullable],
            created_at: datetime [auto_now],
            updated_at: datetime [auto_now_update],
            avatar_data: binary [nullable],
            metadata: json [nullable],
            ip_addr: ip [nullable],
            cache_key: text [nullable, readonly],
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
    assert!(
        parse_schema_from_source(full_types_source())
            .unwrap()
            .is_some()
    );
}

#[test]
fn test_parse_empty_returns_none() {
    assert!(parse_schema_from_source("").unwrap().is_none());
}

#[test]
fn test_parse_no_model_macro_returns_none() {
    let src = r#"pub struct Foo { pub id: i32 }"#;
    assert!(parse_schema_from_source(src).unwrap().is_none());
}

#[test]
fn test_parse_invalid_rust_is_an_error() {
    assert!(parse_schema_from_source("let x = !!@@@#").is_err());
}

// ═══════════════════════════════════════════════════════════════
// Nom de table et PK
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_table_name_preserved() {
    let s = parse_schema_from_source(full_types_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.table_name, "all_types");
}

#[test]
fn test_pk_name_i32() {
    let s = parse_schema_from_source(full_types_source())
        .unwrap()
        .unwrap()
        .1;
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "Integer");
}

#[test]
fn test_pk_name_i64() {
    let s = parse_schema_from_source(pk_i64_source())
        .unwrap()
        .unwrap()
        .1;
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "BigInteger");
}

#[test]
fn test_pk_uuid() {
    let s = parse_schema_from_source(pk_uuid_source())
        .unwrap()
        .unwrap()
        .1;
    let pk = s.primary_key.unwrap();
    assert_eq!(pk.name, "slug");
    assert_eq!(pk.col_type, "Uuid");
}

// `pk: id => Pk` defers to the global `runique::utils::config::Pk` alias — resolved column
// type must follow whichever of `big-pk`/`pk-uuid` is active (or plain i32 by default).
#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
#[test]
fn test_pk_generic_alias_default() {
    let s = parse_schema_from_source(pk_generic_alias_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.primary_key.unwrap().col_type, "Integer");
}

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
#[test]
fn test_pk_generic_alias_big_pk() {
    let s = parse_schema_from_source(pk_generic_alias_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.primary_key.unwrap().col_type, "BigInteger");
}

#[cfg(feature = "pk-uuid")]
#[test]
fn test_pk_generic_alias_pk_uuid() {
    let s = parse_schema_from_source(pk_generic_alias_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.primary_key.unwrap().col_type, "Uuid");
}

#[test]
fn test_pk_not_nullable() {
    let s = parse_schema_from_source(pk_i64_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(!s.primary_key.unwrap().nullable);
}

// ═══════════════════════════════════════════════════════════════
// Mapping des types de champs — types scalar classiques
// ═══════════════════════════════════════════════════════════════

fn col_type(src: &str, field: &str) -> String {
    let s = parse_schema_from_source(src).unwrap().unwrap().1;
    s.columns
        .iter()
        .find(|c| c.name == field)
        .unwrap_or_else(|| panic!("champ '{field}' introuvable"))
        .col_type
        .clone()
}

#[test]
fn test_type_textarea_maps_to_text() {
    assert_eq!(col_type(full_types_source(), "f_textarea"), "Text");
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
fn test_type_i64_maps_to_biginteger() {
    assert_eq!(col_type(full_types_source(), "f_i64"), "BigInteger");
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
fn test_type_var_binary_maps_to_varbinary() {
    assert_eq!(col_type(full_types_source(), "f_var_binary"), "VarBinary");
}

#[test]
fn test_type_blob_maps_to_blob() {
    assert_eq!(col_type(full_types_source(), "f_blob"), "Blob");
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
    let s = parse_schema_from_source(full_types_source())
        .unwrap()
        .unwrap()
        .1;
    // 28 champs déclarés
    assert_eq!(s.columns.len(), 28);
}

#[test]
fn test_field_count_options_model() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    // 11 champs déclarés
    assert_eq!(s.columns.len(), 11);
}

// ═══════════════════════════════════════════════════════════════
// Options des champs — nullable, unique, ignored
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_option_required_field_not_nullable() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "nullable_field")
        .unwrap();
    assert!(f.nullable, "nullable_field doit être nullable");
}

#[test]
fn test_option_unique_field() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    let f = s.columns.iter().find(|c| c.name == "unique_field").unwrap();
    assert!(f.unique, "unique_field doit être unique");
    assert!(!f.nullable);
}

#[test]
fn test_option_nullable_and_unique() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    let f = s.columns.iter().find(|c| c.name == "both_field").unwrap();
    assert!(f.nullable);
    assert!(f.unique);
}

#[test]
fn test_option_auto_now_is_ignored_and_datetime() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "readonly_field")
        .unwrap();
    assert!(f.ignored, "readonly doit marquer le champ ignored");
}

#[test]
fn test_option_max_len_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(s.columns.iter().any(|c| c.name == "max_len_field"));
}

#[test]
fn test_option_min_len_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(s.columns.iter().any(|c| c.name == "min_len_field"));
}

#[test]
fn test_option_label_does_not_break_parsing() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(s.columns.iter().any(|c| c.name == "label_field"));
}

#[test]
fn test_option_renamed_from_is_read() {
    let s = parse_schema_from_source(options_source())
        .unwrap()
        .unwrap()
        .1;
    let f = s
        .columns
        .iter()
        .find(|c| c.name == "renamed_field")
        .unwrap();
    assert_eq!(f.renamed_from.as_deref(), Some("old_name"));
}

#[test]
fn test_unknown_type_is_an_error_with_its_position() {
    let src = "model! {\n    Post,\n    table: \"posts\",\n    pk: id => i32,\n    { title: texte [required] }\n}";
    let err = parse_schema_from_source(src).unwrap_err().to_string();
    assert!(err.starts_with("5:"), "ligne de l'erreur : {err}");
    assert!(err.contains("texte"), "{err}");
}

#[test]
fn test_unknown_fk_action_is_an_error() {
    let src = r#"model! { Post, table: "posts", pk: id => i32, { user_id: int [required] }, relations: { belongs_to: User via user_id [cascad] } }"#;
    assert!(parse_schema_from_source(src).is_err());
}

#[test]
fn test_two_models_in_one_file_is_an_error() {
    let src = r#"
        model! { A, table: "a", pk: id => i32, { x: text } }
        model! { B, table: "b", pk: id => i32, { y: text } }
    "#;
    assert!(parse_schema_from_source(src).is_err());
}

// ═══════════════════════════════════════════════════════════════
// Bloc relations — parsing sans crash
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_with_relations_returns_some() {
    assert!(
        parse_schema_from_source(relations_source())
            .unwrap()
            .is_some(),
        "un modèle avec relations doit parser correctement"
    );
}

#[test]
fn test_parse_relations_table_name() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.table_name, "posts");
}

#[test]
fn test_parse_relations_fields_intact() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(s.columns.iter().any(|c| c.name == "title"));
    assert!(s.columns.iter().any(|c| c.name == "user_id"));
}

// ═══════════════════════════════════════════════════════════════
// Bloc meta — parsing sans crash
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_with_meta_returns_some() {
    assert!(
        parse_schema_from_source(meta_source()).unwrap().is_some(),
        "un modèle avec meta doit parser correctement"
    );
}

#[test]
fn test_parse_meta_table_name() {
    let s = parse_schema_from_source(meta_source()).unwrap().unwrap().1;
    assert_eq!(s.table_name, "articles");
}

#[test]
fn test_parse_meta_fields_intact() {
    let s = parse_schema_from_source(meta_source()).unwrap().unwrap().1;
    assert!(s.columns.iter().any(|c| c.name == "title"));
    let slug = s.columns.iter().find(|c| c.name == "slug").unwrap();
    assert!(slug.unique);
}

// ═══════════════════════════════════════════════════════════════
// Modèle complet (relations + meta + tous les cas)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_full_model_parses_successfully() {
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.table_name, "user_profiles");
}

#[test]
fn test_full_model_pk() {
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
    let pk = s.primary_key.as_ref().unwrap();
    assert_eq!(pk.name, "id");
    assert_eq!(pk.col_type, "Integer");
}

#[test]
fn test_full_model_unique_fields() {
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
    let email = s.columns.iter().find(|c| c.name == "email").unwrap();
    assert!(email.unique);
    let username = s.columns.iter().find(|c| c.name == "username").unwrap();
    assert!(username.unique);
}

#[test]
fn test_full_model_nullable_fields() {
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
    for name in &["bio", "birth_date", "avatar_data", "metadata", "ip_addr"] {
        let f = s.columns.iter().find(|c| c.name == *name).unwrap();
        assert!(f.nullable, "{} doit être nullable", name);
    }
}

#[test]
fn test_full_model_ignored_fields() {
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(full_model_source())
        .unwrap()
        .unwrap()
        .1;
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
    let a = parse_schema_from_source(pk_i64_source())
        .unwrap()
        .unwrap()
        .1;
    let b = parse_schema_from_source(pk_uuid_source())
        .unwrap()
        .unwrap()
        .1;
    assert_ne!(a.table_name, b.table_name);
    assert_ne!(
        a.primary_key.unwrap().col_type,
        b.primary_key.unwrap().col_type
    );
}

#[test]
fn test_multiple_calls_same_source_return_equal_results() {
    let s1 = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    let s2 = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s1.table_name, s2.table_name);
    assert_eq!(s1.columns.len(), s2.columns.len());
}

// ═══════════════════════════════════════════════════════════════
// Relations — FK générées depuis belongs_to
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_belongs_to_genere_une_fk() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(
        s.foreign_keys.len(),
        1,
        "belongs_to doit générer exactement 1 FK"
    );
}

#[test]
fn test_belongs_to_from_column() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.from_column, "user_id");
}

#[test]
fn test_belongs_to_to_table_pascal_vers_snake() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    let fk = &s.foreign_keys[0];
    // User → user
    assert_eq!(fk.to_table, "user");
}

#[test]
fn test_belongs_to_to_column_defaut_id() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.to_column, "id");
}

#[test]
fn test_belongs_to_on_delete_no_action_par_defaut() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    let fk = &s.foreign_keys[0];
    assert_eq!(fk.on_delete, "NoAction");
    assert_eq!(fk.on_update, "NoAction");
}

#[test]
fn test_has_many_ne_genere_pas_de_fk() {
    let s = parse_schema_from_source(relations_source())
        .unwrap()
        .unwrap()
        .1;
    // only belongs_to generates FK — has_many does not
    assert!(
        s.foreign_keys
            .iter()
            .all(|fk| fk.from_column != "comment_id")
    );
}

#[test]
fn test_belongs_to_cascade_on_delete() {
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
    assert_eq!(s.foreign_keys.len(), 2, "2 belongs_to → 2 FK");
}

#[test]
fn test_belongs_to_table_cible_pascal_to_snake_composee() {
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
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
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
    // has_one: CommentMeta ne doit pas créer de FK
    assert!(
        s.foreign_keys
            .iter()
            .all(|fk| fk.to_table != "comment_meta")
    );
}

#[test]
fn test_relations_champs_intacts_avec_cascade() {
    let s = parse_schema_from_source(relations_cascade_source())
        .unwrap()
        .unwrap()
        .1;
    assert!(s.columns.iter().any(|c| c.name == "body"));
    assert!(s.columns.iter().any(|c| c.name == "post_id"));
    assert!(s.columns.iter().any(|c| c.name == "author_id"));
}
