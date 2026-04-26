//! Tests — forms/field.rs
//! Couvre : cleaned_string, cleaned_i32, cleaned_i64, cleaned_u32, cleaned_u64,
//!          cleaned_f32, cleaned_f64, cleaned_bool, cleaned_uuid, cleaned_naive_date,
//!          cleaned_naive_time, cleaned_naive_datetime, cleaned_datetime_utc,
//!          cleaned_value (path/query fallback), clear()

use runique::forms::{field::RuniqueForm, fields::text::TextField, form::Forms};

// ── Helper : form de test ──────────────────────────────────────

struct TestForm {
    form: Forms,
}

impl RuniqueForm for TestForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("str_field"));
        form.field(&TextField::text("int_field"));
        form.field(&TextField::text("float_field"));
        form.field(&TextField::text("bool_field"));
        form.field(&TextField::text("uuid_field"));
        form.field(&TextField::text("date_field"));
        form.field(&TextField::text("time_field"));
        form.field(&TextField::text("datetime_field"));
        form.field(&TextField::text("rfc3339_field"));
    }
    fn from_form(form: Forms) -> Self {
        Self { form }
    }
    fn get_form(&self) -> &Forms {
        &self.form
    }
    fn get_form_mut(&mut self) -> &mut Forms {
        &mut self.form
    }
}

fn make_form() -> TestForm {
    let mut form = TestForm {
        form: Forms::new("csrf"),
    };
    TestForm::register_fields(&mut form.form);
    form
}

// ═══════════════════════════════════════════════════════════════
// cleaned_string
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_string_some() {
    let mut form = make_form();
    form.form.add_value("str_field", "hello");
    assert_eq!(form.cleaned_string("str_field"), Some("hello".to_string()));
}

#[test]
fn test_cleaned_string_empty_returns_none() {
    let form = make_form();
    assert_eq!(form.cleaned_string("str_field"), None);
}

#[test]
fn test_cleaned_string_unknown_field_returns_none() {
    let form = make_form();
    assert_eq!(form.cleaned_string("unknown"), None);
}

// ═══════════════════════════════════════════════════════════════
// cleaned_i32 / cleaned_i64 / cleaned_u32 / cleaned_u64
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_i32_valid() {
    let mut form = make_form();
    form.form.add_value("int_field", "42");
    assert_eq!(form.cleaned_i32("int_field"), Some(42));
}

#[test]
fn test_cleaned_i32_invalid_text_returns_none() {
    let mut form = make_form();
    form.form.add_value("int_field", "abc");
    assert_eq!(form.cleaned_i32("int_field"), None);
}

#[test]
fn test_cleaned_i64_valid() {
    let mut form = make_form();
    form.form.add_value("int_field", "9999999999");
    assert_eq!(form.cleaned_i64("int_field"), Some(9_999_999_999_i64));
}

#[test]
fn test_cleaned_u32_valid() {
    let mut form = make_form();
    form.form.add_value("int_field", "100");
    assert_eq!(form.cleaned_u32("int_field"), Some(100u32));
}

#[test]
fn test_cleaned_u64_valid() {
    let mut form = make_form();
    form.form.add_value("int_field", "500");
    assert_eq!(form.cleaned_u64("int_field"), Some(500u64));
}

// ═══════════════════════════════════════════════════════════════
// cleaned_f32 / cleaned_f64
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_f64_point_separator() {
    let mut form = make_form();
    form.form.add_value("float_field", "19.99");
    let v = form.cleaned_f64("float_field").unwrap();
    assert!((v - 19.99).abs() < 0.001);
}

#[test]
fn test_cleaned_f64_comma_separator() {
    let mut form = make_form();
    form.form.add_value("float_field", "19,99");
    let v = form.cleaned_f64("float_field").unwrap();
    assert!((v - 19.99).abs() < 0.001);
}

#[test]
fn test_cleaned_f32_valid() {
    let mut form = make_form();
    form.form.add_value("float_field", "3.14");
    let v = form.cleaned_f32("float_field").unwrap();
    assert!((v - 3.14_f32).abs() < 0.01);
}

#[test]
fn test_cleaned_f64_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("float_field", "not-a-number");
    assert_eq!(form.cleaned_f64("float_field"), None);
}

// ═══════════════════════════════════════════════════════════════
// cleaned_bool
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_bool_true_values() {
    for val in &["true", "1", "on", "TRUE", "ON"] {
        let mut form = make_form();
        form.form.add_value("bool_field", val);
        assert_eq!(
            form.cleaned_bool("bool_field"),
            Some(true),
            "expected true for {val}"
        );
    }
}

#[test]
fn test_cleaned_bool_false_value() {
    let mut form = make_form();
    form.form.add_value("bool_field", "false");
    assert_eq!(form.cleaned_bool("bool_field"), Some(false));
}

#[test]
fn test_cleaned_bool_empty_returns_none() {
    let form = make_form();
    assert_eq!(form.cleaned_bool("bool_field"), None);
}

// ═══════════════════════════════════════════════════════════════
// cleaned_uuid
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_uuid_valid() {
    let mut form = make_form();
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    form.form.add_value("uuid_field", uuid);
    assert!(form.cleaned_uuid("uuid_field").is_some());
}

#[test]
fn test_cleaned_uuid_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("uuid_field", "not-a-uuid");
    assert_eq!(form.cleaned_uuid("uuid_field"), None);
}

// ═══════════════════════════════════════════════════════════════
// cleaned_naive_date / cleaned_naive_time / cleaned_naive_datetime
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_naive_date_valid() {
    let mut form = make_form();
    form.form.add_value("date_field", "2024-06-15");
    assert!(form.cleaned_naive_date("date_field").is_some());
}

#[test]
fn test_cleaned_naive_date_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("date_field", "15/06/2024");
    assert_eq!(form.cleaned_naive_date("date_field"), None);
}

#[test]
fn test_cleaned_naive_time_valid() {
    let mut form = make_form();
    form.form.add_value("time_field", "14:30");
    assert!(form.cleaned_naive_time("time_field").is_some());
}

#[test]
fn test_cleaned_naive_time_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("time_field", "14h30");
    assert_eq!(form.cleaned_naive_time("time_field"), None);
}

#[test]
fn test_cleaned_naive_datetime_valid() {
    let mut form = make_form();
    form.form.add_value("datetime_field", "2024-06-15T14:30");
    assert!(form.cleaned_naive_datetime("datetime_field").is_some());
}

#[test]
fn test_cleaned_naive_datetime_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("datetime_field", "invalid");
    assert_eq!(form.cleaned_naive_datetime("datetime_field"), None);
}

// ═══════════════════════════════════════════════════════════════
// cleaned_datetime_utc
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_datetime_utc_valid() {
    let mut form = make_form();
    form.form.add_value("rfc3339_field", "2024-06-15T14:30:00Z");
    assert!(form.cleaned_datetime_utc("rfc3339_field").is_some());
}

#[test]
fn test_cleaned_datetime_utc_invalid_returns_none() {
    let mut form = make_form();
    form.form.add_value("rfc3339_field", "not-a-date");
    assert_eq!(form.cleaned_datetime_utc("rfc3339_field"), None);
}

// ═══════════════════════════════════════════════════════════════
// path/query param fallback dans cleaned_value
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cleaned_string_falls_back_to_path_param() {
    let mut form = make_form();
    let path =
        std::collections::HashMap::from([("str_field".to_string(), "from-path".to_string())]);
    let query = std::collections::HashMap::new();
    form.form.set_url_params(&path, &query);
    // champ vide mais path param présent
    assert_eq!(
        form.cleaned_string("str_field"),
        Some("from-path".to_string())
    );
}

#[test]
fn test_cleaned_string_falls_back_to_query_param() {
    let mut form = make_form();
    let path = std::collections::HashMap::new();
    let query =
        std::collections::HashMap::from([("str_field".to_string(), "from-query".to_string())]);
    form.form.set_url_params(&path, &query);
    assert_eq!(
        form.cleaned_string("str_field"),
        Some("from-query".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════
// clear()
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_clear_resets_field_values() {
    let mut form = make_form();
    form.form.add_value("str_field", "Alice");
    form.clear();
    assert_eq!(form.cleaned_string("str_field"), None);
}
