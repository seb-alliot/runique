//! Tests supplémentaires — column/mod.rs
//! Couvre : to_form_field (types manquants), format_label, postgres types,
//!          to_sea_column avec default, binary/char/var_binary

use runique::forms::base::FormField;
use runique::migration::column::ColumnDef;
use sea_query::ColumnType;

// ═══════════════════════════════════════════════════════════════
// Types Postgres
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_inet() {
    let col = ColumnDef::new("ip").inet();
    assert!(matches!(col.col_type, ColumnType::Inet));
}

#[test]
fn test_column_cidr() {
    let col = ColumnDef::new("net").cidr();
    assert!(matches!(col.col_type, ColumnType::Cidr));
}

#[test]
fn test_column_mac_address() {
    let col = ColumnDef::new("mac").mac_address();
    assert!(matches!(col.col_type, ColumnType::MacAddr));
}

#[test]
fn test_column_interval() {
    let col = ColumnDef::new("duree").interval();
    assert!(matches!(col.col_type, ColumnType::Interval(_, _)));
}

// ═══════════════════════════════════════════════════════════════
// Types char / var_binary / blob
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_column_char() {
    let col = ColumnDef::new("code").char();
    assert!(matches!(col.col_type, ColumnType::Char(_)));
}

#[test]
fn test_column_char_len() {
    let col = ColumnDef::new("code").char_len(3);
    assert!(matches!(col.col_type, ColumnType::Char(Some(3))));
}

#[test]
fn test_column_var_binary() {
    let col = ColumnDef::new("data").var_binary(128);
    assert!(matches!(col.col_type, ColumnType::VarBinary(_)));
}

#[test]
fn test_column_blob() {
    let col = ColumnDef::new("payload").blob();
    assert!(matches!(col.col_type, ColumnType::Blob));
}

#[test]
fn test_column_unsigned() {
    let col = ColumnDef::new("count").unsigned();
    assert!(matches!(col.col_type, ColumnType::Unsigned));
}

#[test]
fn test_column_big_unsigned() {
    let col = ColumnDef::new("big").big_unsigned();
    assert!(matches!(col.col_type, ColumnType::BigUnsigned));
}

// ═══════════════════════════════════════════════════════════════
// to_sea_column avec valeur par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_sea_column_avec_default() {
    let col = ColumnDef::new("actif")
        .boolean()
        .default(sea_query::Value::Bool(Some(true)));
    let _ = col.to_sea_column();
}

#[test]
fn test_to_sea_column_nullable() {
    let col = ColumnDef::new("bio").text().nullable();
    let _ = col.to_sea_column();
}

#[test]
fn test_to_sea_column_unique() {
    let col = ColumnDef::new("email").string().unique();
    let _ = col.to_sea_column();
}

// ═══════════════════════════════════════════════════════════════
// to_form_field — branches manquantes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_form_field_password_name() {
    let col = ColumnDef::new("password").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_password_suffixe() {
    let col = ColumnDef::new("user_password").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_url_name() {
    let col = ColumnDef::new("url").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_website_name() {
    let col = ColumnDef::new("website").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_slug_name() {
    let col = ColumnDef::new("slug").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_color_name() {
    let col = ColumnDef::new("color").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_ip_name() {
    let col = ColumnDef::new("ip").string();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_text_description() {
    let col = ColumnDef::new("description").text();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_text_bio() {
    let col = ColumnDef::new("bio").text();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_text_content() {
    let col = ColumnDef::new("content").text();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_text_generic() {
    let col = ColumnDef::new("remarque").text();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_big_integer() {
    let col = ColumnDef::new("counter").big_integer();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_tiny_integer() {
    let col = ColumnDef::new("score").tiny_integer();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_float() {
    let col = ColumnDef::new("prix").float();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_double() {
    let col = ColumnDef::new("ratio").double();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_decimal() {
    let col = ColumnDef::new("montant").decimal();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_date() {
    let col = ColumnDef::new("naissance").date();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_time() {
    let col = ColumnDef::new("heure").time();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_datetime() {
    let col = ColumnDef::new("created_at").datetime();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_timestamp() {
    let col = ColumnDef::new("updated_at").timestamp();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_json_binary() {
    let col = ColumnDef::new("data").json_binary();
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_char() {
    let col = ColumnDef::new("code").char_len(3);
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_enum() {
    let col = ColumnDef::new("status").enum_type(
        "post_status",
        vec!["draft".to_string(), "published".to_string()],
    );
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_string_avec_max_len() {
    let col = ColumnDef::new("titre").string().max_len(255);
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_email_avec_max_len() {
    let col = ColumnDef::new("email").string().max_len(100);
    assert!(col.to_form_field().is_some());
}

#[test]
fn test_to_form_field_required_quand_non_nullable() {
    let col = ColumnDef::new("nom").string();
    assert!(!col.nullable);
    let field = col.to_form_field().unwrap();
    assert!(field.required());
}

#[test]
fn test_to_form_field_optionnel_quand_nullable() {
    let col = ColumnDef::new("bio").text().nullable();
    let field = col.to_form_field().unwrap();
    assert!(!field.required());
}

// ═══════════════════════════════════════════════════════════════
// format_label (via to_form_field — le label est auto-calculé)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_format_label_simple() {
    let col = ColumnDef::new("name").string();
    let field = col.to_form_field().unwrap();
    // "name" → "Name"
    let label = field.label();
    assert_eq!(label, "Name");
}

#[test]
fn test_format_label_snake_case() {
    let col = ColumnDef::new("first_name").string();
    let field = col.to_form_field().unwrap();
    // "first_name" → "First Name"
    let label = field.label();
    assert_eq!(label, "First Name");
}

#[test]
fn test_format_label_triple() {
    let col = ColumnDef::new("date_of_birth").date();
    let field = col.to_form_field().unwrap();
    // "date_of_birth" → "Date Of Birth"
    let label = field.label();
    assert_eq!(label, "Date Of Birth");
}
