// Tests pour ColumnDef

use runique::migration::column::ColumnDef;
use sea_query::ColumnType;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_def_new_defauts() {
    let col = ColumnDef::new("titre");
    assert_eq!(col.name, "titre");
    assert!(!col.nullable);
    assert!(!col.unique);
    assert!(!col.ignored);
    assert!(!col.auto_now);
    assert!(!col.auto_now_update);
    assert!(col.default.is_none());
    assert!(col.enum_variants.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Types de colonnes — principaux
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_string() {
    let col = ColumnDef::new("nom").string();
    assert!(matches!(col.col_type, ColumnType::String(_)));
}

#[test]
fn test_column_varchar() {
    let col = ColumnDef::new("code").varchar(50);
    assert!(matches!(col.col_type, ColumnType::String(_)));
}

#[test]
fn test_column_text() {
    let col = ColumnDef::new("contenu").text();
    assert!(matches!(col.col_type, ColumnType::Text));
}

#[test]
fn test_column_integer() {
    let col = ColumnDef::new("age").integer();
    assert!(matches!(col.col_type, ColumnType::Integer));
}

#[test]
fn test_column_big_integer() {
    let col = ColumnDef::new("counter").big_integer();
    assert!(matches!(col.col_type, ColumnType::BigInteger));
}

#[test]
fn test_column_float() {
    let col = ColumnDef::new("prix").float();
    assert!(matches!(col.col_type, ColumnType::Float));
}

#[test]
fn test_column_double() {
    let col = ColumnDef::new("coords").double();
    assert!(matches!(col.col_type, ColumnType::Double));
}

#[test]
fn test_column_boolean() {
    let col = ColumnDef::new("actif").boolean();
    assert!(matches!(col.col_type, ColumnType::Boolean));
}

#[test]
fn test_column_datetime() {
    let col = ColumnDef::new("created_at").datetime();
    assert!(matches!(col.col_type, ColumnType::DateTime));
}

#[test]
fn test_column_date() {
    let col = ColumnDef::new("naissance").date();
    assert!(matches!(col.col_type, ColumnType::Date));
}

#[test]
fn test_column_time() {
    let col = ColumnDef::new("heure").time();
    assert!(matches!(col.col_type, ColumnType::Time));
}

#[test]
fn test_column_uuid() {
    let col = ColumnDef::new("token").uuid();
    assert!(matches!(col.col_type, ColumnType::Uuid));
}

#[test]
fn test_column_json() {
    let col = ColumnDef::new("meta").json();
    assert!(matches!(col.col_type, ColumnType::Json));
}

#[test]
fn test_column_json_binary() {
    let col = ColumnDef::new("data").json_binary();
    assert!(matches!(col.col_type, ColumnType::JsonBinary));
}

#[test]
fn test_column_decimal() {
    let col = ColumnDef::new("montant").decimal();
    assert!(matches!(col.col_type, ColumnType::Decimal(_)));
}

#[test]
fn test_column_decimal_len() {
    let col = ColumnDef::new("montant").decimal_len(10, 2);
    assert!(matches!(col.col_type, ColumnType::Decimal(Some((10, 2)))));
}

#[test]
fn test_column_binary() {
    let col = ColumnDef::new("hash").binary();
    assert!(matches!(col.col_type, ColumnType::Binary(255)));
}

#[test]
fn test_column_binary_len() {
    let col = ColumnDef::new("hash").binary_len(64);
    assert!(matches!(col.col_type, ColumnType::Binary(64)));
}

#[test]
fn test_column_tiny_integer() {
    let col = ColumnDef::new("score").tiny_integer();
    assert!(matches!(col.col_type, ColumnType::TinyInteger));
}

#[test]
fn test_column_small_integer() {
    let col = ColumnDef::new("port").small_integer();
    assert!(matches!(col.col_type, ColumnType::SmallInteger));
}

#[test]
fn test_column_timestamp() {
    let col = ColumnDef::new("ts").timestamp();
    assert!(matches!(col.col_type, ColumnType::Timestamp));
}

#[test]
fn test_column_timestamp_tz() {
    let col = ColumnDef::new("ts_tz").timestamp_tz();
    assert!(matches!(col.col_type, ColumnType::TimestampWithTimeZone));
}

// ═══════════════════════════════════════════════════════════════
// Enum
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_enum_type_stocke_variants() {
    let variants = vec![
        "draft".to_string(),
        "published".to_string(),
        "archived".to_string(),
    ];
    let col = ColumnDef::new("status").enum_type("post_status", variants.clone());
    assert_eq!(col.enum_variants, variants);
    assert!(matches!(col.col_type, ColumnType::Enum { .. }));
}

// ═══════════════════════════════════════════════════════════════
// Modifiers
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_nullable() {
    let col = ColumnDef::new("bio").text().nullable();
    assert!(col.nullable);
}

#[test]
fn test_column_required() {
    let col = ColumnDef::new("email").string().nullable().required();
    assert!(!col.nullable);
}

#[test]
fn test_column_unique() {
    let col = ColumnDef::new("email").string().unique();
    assert!(col.unique);
}

#[test]
fn test_column_ignore() {
    let col = ColumnDef::new("computed").integer().ignore();
    assert!(col.ignored);
}

#[test]
fn test_column_auto_now() {
    let col = ColumnDef::new("created_at").auto_now();
    assert!(col.auto_now);
    assert!(matches!(col.col_type, ColumnType::DateTime));
}

#[test]
fn test_column_auto_now_update() {
    let col = ColumnDef::new("updated_at").auto_now_update();
    assert!(col.auto_now_update);
    assert!(matches!(col.col_type, ColumnType::DateTime));
}

#[test]
fn test_column_max_len() {
    let col = ColumnDef::new("slug").string().max_len(100);
    assert_eq!(col.max_length, Some(100));
}

#[test]
fn test_column_min_len() {
    let col = ColumnDef::new("password").string().min_len(8);
    assert_eq!(col.min_length, Some(8));
}

#[test]
fn test_column_max_i64() {
    let col = ColumnDef::new("score").integer().max_i64(1000);
    assert_eq!(col.max_value, Some(1000));
}

#[test]
fn test_column_min_i64() {
    let col = ColumnDef::new("score").integer().min_i64(-100);
    assert_eq!(col.min_value, Some(-100));
}

#[test]
fn test_column_max_f64() {
    let col = ColumnDef::new("ratio").float().max_f64(1.0);
    assert_eq!(col.max_float, Some(1.0));
}

#[test]
fn test_column_min_f64() {
    let col = ColumnDef::new("ratio").float().min_f64(0.0);
    assert_eq!(col.min_float, Some(0.0));
}

#[test]
fn test_column_select_as() {
    let col = ColumnDef::new("col").string().select_as("alias");
    assert_eq!(col.select_as, Some("alias".to_string()));
}

#[test]
fn test_column_save_as() {
    let col = ColumnDef::new("col").string().save_as("db_col");
    assert_eq!(col.save_as, Some("db_col".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// to_sea_column (ne panique pas)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_to_sea_column_compile() {
    let col = ColumnDef::new("email").string().unique().nullable();
    let _ = col.to_sea_column();
}

// ═══════════════════════════════════════════════════════════════
// to_form_field — dispatch par type
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_form_field_auto_now_retourne_none() {
    let col = ColumnDef::new("created_at").auto_now();
    assert!(col.to_form_field().is_none());
}

#[test]
fn test_to_form_field_auto_now_update_retourne_none() {
    let col = ColumnDef::new("updated_at").auto_now_update();
    assert!(col.to_form_field().is_none());
}

#[test]
fn test_to_form_field_ignored_retourne_none() {
    let col = ColumnDef::new("computed").integer().ignore();
    assert!(col.to_form_field().is_none());
}

#[test]
fn test_to_form_field_string_retourne_some() {
    let col = ColumnDef::new("titre").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_email_name() {
    // Le nom "email" déclenche la branche email dans to_form_field
    let col = ColumnDef::new("email").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_boolean() {
    let col = ColumnDef::new("actif").boolean();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_integer() {
    let col = ColumnDef::new("age").integer();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_uuid() {
    let col = ColumnDef::new("token").uuid();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_json() {
    let col = ColumnDef::new("meta").json();
    assert!(col.to_form_field().is_some());
}
