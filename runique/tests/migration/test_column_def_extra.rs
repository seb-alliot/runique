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

// Written from cargo-mutants survivors (2026-10-02): the `is_some()` tests
// above pass whatever field the name leads to. A column with no DSL type
// (builtin tables, `extend!{}`) gets its widget from its name — each
// alternative of each rule, alone, must lead to that widget.
async fn guessed(col: ColumnDef) -> String {
    let mut field = col.to_form_field().expect("field");
    let (ty, tpl) = (
        field.field_type().to_string(),
        field.template_name().to_string(),
    );
    if (ty.as_str(), tpl.as_str()) == ("text", "base_special.html") {
        field.set_value("a-b");
        return if field.validate().await { "slug" } else { "ip" }.to_string();
    }
    if tpl == "base_string.html" {
        ty
    } else {
        format!("{ty}@{tpl}")
    }
}

#[tokio::test]
async fn a_column_without_a_dsl_type_gets_its_widget_from_its_name() {
    let string = |n: &str| ColumnDef::new(n).string();
    let cases: &[(&str, &str)] = &[
        ("email", "email"),
        ("contact_email", "email"),
        ("password", "password"),
        ("user_password", "password"),
        ("pin_pwd", "password"),
        ("url", "url"),
        ("home_url", "url"),
        ("website", "url"),
        ("company_website", "url"),
        ("http_link", "url"),
        ("slug", "slug"),
        ("post_slug", "slug"),
        ("color", "color@base_color.html"),
        ("bg_color", "color@base_color.html"),
        ("colour", "color@base_color.html"),
        ("bg_colour", "color@base_color.html"),
        ("ip", "ip"),
        ("client_ip", "ip"),
        ("last_ip_address", "ip"),
        ("title", "text"),
    ];
    for (name, widget) in cases {
        assert_eq!(
            guessed(string(name)).await,
            *widget,
            "string column `{name}`"
        );
    }

    for name in [
        "description",
        "short_bio",
        "content",
        "message",
        "summary",
        "richtext_body",
    ] {
        assert_eq!(
            guessed(ColumnDef::new(name).text()).await,
            "richtext",
            "text column `{name}`"
        );
    }
    assert_eq!(guessed(ColumnDef::new("notes").text()).await, "textarea");
}
