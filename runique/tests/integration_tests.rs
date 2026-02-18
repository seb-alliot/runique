//! Integration tests for the Runique framework
//! Tests the main features of the framework:
//! - All field types (17 types)
//! - Validation (required, format, limits)
//! - Builder pattern (label, placeholder, etc.)
//! - Forms
//! - Special fields (CSRF, HiddenField)

use runique::forms::field::FormField;
use runique::forms::fields::choice::{CheckboxField, ChoiceOption, RadioField};
use runique::forms::fields::datetime::{DateField, DurationField, TimeField};
use runique::forms::fields::special::{IPAddressField, UUIDField};
use runique::prelude::*;

// ============================================================================
// TEXTFIELD — Constructors
// ============================================================================

#[test]
fn test_text_field_creation() {
    let field = TextField::text("username");
    assert_eq!(field.name(), "username");
    assert_eq!(field.field_type(), "text");
}

#[test]
fn test_email_field() {
    let field = TextField::email("email");
    assert_eq!(field.field_type(), "email");
}

#[test]
fn test_password_field() {
    let field = TextField::password("password");
    assert_eq!(field.field_type(), "password");
}

#[test]
fn test_textarea_field() {
    let field = TextField::textarea("content");
    assert_eq!(field.field_type(), "textarea");
}

#[test]
fn test_richtext_field() {
    let field = TextField::richtext("description");
    assert_eq!(field.field_type(), "richtext");
}

#[test]
fn test_url_field() {
    let field = TextField::url("website");
    assert_eq!(field.field_type(), "url");
}

// ============================================================================
// TEXTFIELD — Builder & Validation
// ============================================================================

#[test]
fn test_text_field_builder() {
    let field = TextField::text("name")
        .label("Nom complet")
        .placeholder("Entrez votre nom")
        .required();
    assert_eq!(field.name(), "name");
    assert!(field.base.is_required.choice);
}

#[test]
fn test_text_field_required_empty_fails() {
    let mut field = TextField::text("name").required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Ce champ est obligatoire");
}

#[test]
fn test_text_field_required_filled_passes() {
    let mut field = TextField::text("name").required();
    field.set_value("Alice");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_text_field_min_length() {
    let mut field = TextField::text("pseudo").min_length(3, "");
    field.set_value("ab");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("min 3"));
}

#[test]
fn test_text_field_max_length() {
    let mut field = TextField::text("pseudo").max_length(5, "Trop long !");
    field.set_value("abcdef");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Trop long !");
}

#[test]
fn test_text_field_min_max_valid() {
    let mut field = TextField::text("pseudo")
        .min_length(2, "")
        .max_length(10, "");
    field.set_value("hello");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_email_validation_valid() {
    let mut field = TextField::email("email");
    field.set_value("test@example.com");
    assert!(field.validate());
    // The email must be converted to lowercase
    assert_eq!(field.value(), "test@example.com");
}

#[test]
fn test_email_validation_uppercase_normalized() {
    let mut field = TextField::email("email");
    field.set_value("Test@EXAMPLE.Com");
    assert!(field.validate());
    assert_eq!(field.value(), "test@example.com");
}

#[test]
fn test_email_validation_invalid() {
    let mut field = TextField::email("email");
    field.set_value("pas-un-email");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Format d'adresse email invalide");
}

#[test]
fn test_url_validation_valid() {
    let mut field = TextField::url("site");
    field.set_value("https://example.com");
    assert!(field.validate());
}

#[test]
fn test_url_validation_invalid() {
    let mut field = TextField::url("site");
    field.set_value("pas-une-url");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Veuillez entrer une URL valide");
}

#[test]
fn test_password_hash_and_verify() {
    let mut field = TextField::password("pass");
    field.set_value("MonMotDePasse123");
    let hash = field.hash_password().unwrap();
    assert!(hash.starts_with("$argon2"));
    assert!(TextField::verify_password("MonMotDePasse123", &hash));
    assert!(!TextField::verify_password("mauvais", &hash));
}

#[test]
fn test_text_field_not_required_empty_passes() {
    let mut field = TextField::text("optionnel");
    field.set_value("");
    assert!(field.validate());
}

// ============================================================================
// NUMERICFIELD
// ============================================================================

#[test]
fn test_numeric_field_integer() {
    let field = NumericField::integer("age");
    assert_eq!(field.name(), "age");
    assert_eq!(field.field_type(), "number");
}

#[test]
fn test_numeric_field_float() {
    let field = NumericField::float("ratio");
    assert_eq!(field.name(), "ratio");
    assert_eq!(field.field_type(), "number");
}

#[test]
fn test_numeric_field_decimal() {
    let field = NumericField::decimal("price");
    assert_eq!(field.name(), "price");
    assert_eq!(field.field_type(), "number");
}

#[test]
fn test_numeric_integer_valid() {
    let mut field = NumericField::integer("age");
    field.set_value("25");
    assert!(field.validate());
}

#[test]
fn test_numeric_integer_invalid() {
    let mut field = NumericField::integer("age");
    field.set_value("abc");
    assert!(!field.validate());
}

#[test]
fn test_numeric_integer_min_max() {
    let mut field = NumericField::integer("age").min(18.0, "").max(99.0, "");
    field.set_value("17");
    assert!(!field.validate());

    field.set_value("50");
    field.set_error("".into()); // reset
    assert!(field.validate());
}

#[test]
fn test_numeric_float_valid() {
    let mut field = NumericField::float("ratio");
    field.set_value("3.14");
    assert!(field.validate());
}

#[test]
fn test_numeric_comma_to_dot() {
    // The engine replaces , with . for normalization
    let mut field = NumericField::float("price");
    field.set_value("19,99");
    assert!(field.validate());
}

#[test]
fn test_numeric_required_empty() {
    let mut field = NumericField::integer("qte");
    field.set_required(true, None);
    field.set_value("");
    assert!(!field.validate());
}

// ============================================================================
// BOOLEANFIELD
// ============================================================================

#[test]
fn test_boolean_field_checkbox() {
    let field = BooleanField::new("accept");
    assert_eq!(field.name(), "accept");
    assert_eq!(field.field_type(), "checkbox");
}

#[test]
fn test_boolean_field_radio() {
    let field = BooleanField::radio("newsletter");
    assert_eq!(field.name(), "newsletter");
    assert_eq!(field.field_type(), "radio");
}

#[test]
fn test_boolean_checked_unchecked() {
    let field = BooleanField::new("cgu").checked();
    assert_eq!(field.value(), "true");

    let field = BooleanField::new("cgu").unchecked();
    assert_eq!(field.value(), "false");
}

#[test]
fn test_boolean_required_not_checked_fails() {
    let mut field = BooleanField::new("cgu").required();
    field.set_value("false");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Vous devez accepter ce champ");
}

#[test]
fn test_boolean_required_checked_passes() {
    let mut field = BooleanField::new("cgu").required();
    field.set_value("true");
    assert!(field.validate());
}

#[test]
fn test_boolean_not_required_passes() {
    let mut field = BooleanField::new("optionnel");
    field.set_value("false");
    assert!(field.validate());
}

// ============================================================================
// CHOICEFIELD (Select)
// ============================================================================

#[test]
fn test_choice_field_creation() {
    let field = ChoiceField::new("pays")
        .add_choice("fr", "France")
        .add_choice("de", "Allemagne")
        .label("Pays");
    assert_eq!(field.name(), "pays");
    assert_eq!(field.field_type(), "select");
}

#[test]
fn test_choice_field_valid_selection() {
    let mut field = ChoiceField::new("pays")
        .add_choice("fr", "France")
        .add_choice("de", "Allemagne");
    field.set_value("fr");
    assert!(field.validate());
}

#[test]
fn test_choice_field_invalid_selection() {
    let mut field = ChoiceField::new("pays")
        .add_choice("fr", "France")
        .add_choice("de", "Allemagne");
    field.set_value("xx");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Choix invalide");
}

#[test]
fn test_choice_field_required_empty() {
    let mut field = ChoiceField::new("pays")
        .add_choice("fr", "France")
        .required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Veuillez sélectionner une option");
}

#[test]
fn test_choice_field_multiple() {
    let field = ChoiceField::new("tags").multiple();
    assert_eq!(field.field_type(), "select-multiple");
}

// ============================================================================
// RADIOFIELD
// ============================================================================

#[test]
fn test_radio_field_creation() {
    let field = RadioField::new("genre")
        .add_choice("m", "Masculin")
        .add_choice("f", "Féminin")
        .label("Genre");
    assert_eq!(field.name(), "genre");
    assert_eq!(field.field_type(), "radio");
}

#[test]
fn test_radio_field_valid() {
    let mut field = RadioField::new("genre")
        .add_choice("m", "Masculin")
        .add_choice("f", "Féminin");
    field.set_value("m");
    assert!(field.validate());
}

#[test]
fn test_radio_field_invalid() {
    let mut field = RadioField::new("genre")
        .add_choice("m", "Masculin")
        .add_choice("f", "Féminin");
    field.set_value("x");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Choix invalide");
}

#[test]
fn test_radio_field_required_empty() {
    let mut field = RadioField::new("genre")
        .add_choice("m", "Masculin")
        .required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Veuillez sélectionner une option");
}

// ============================================================================
// CHECKBOXFIELD (multi-selection)
// ============================================================================

#[test]
fn test_checkbox_field_creation() {
    let field = CheckboxField::new("langs")
        .add_choice("rust", "Rust")
        .add_choice("python", "Python")
        .add_choice("go", "Go");
    assert_eq!(field.name(), "langs");
    assert_eq!(field.field_type(), "checkbox");
}

#[test]
fn test_checkbox_field_valid_multi() {
    let mut field = CheckboxField::new("langs")
        .add_choice("rust", "Rust")
        .add_choice("python", "Python")
        .add_choice("go", "Go");
    field.set_value("rust,python");
    assert!(field.validate());
}

#[test]
fn test_checkbox_field_invalid_choice() {
    let mut field = CheckboxField::new("langs")
        .add_choice("rust", "Rust")
        .add_choice("python", "Python");
    field.set_value("rust,java");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Choix invalide: java"));
}

#[test]
fn test_checkbox_field_required_empty() {
    let mut field = CheckboxField::new("langs")
        .add_choice("rust", "Rust")
        .required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(
        field.error().unwrap(),
        "Veuillez sélectionner au moins une option"
    );
}

#[test]
fn test_checkbox_set_value_marks_selected() {
    let mut field = CheckboxField::new("langs")
        .add_choice("rust", "Rust")
        .add_choice("python", "Python")
        .add_choice("go", "Go");
    field.set_value("rust,go");
    // Checks that the choices are marked
    assert!(field.choices[0].selected); // rust
    assert!(!field.choices[1].selected); // python
    assert!(field.choices[2].selected); // go
}

// ============================================================================
// CHOICE OPTION
// ============================================================================

#[test]
fn test_choice_option_builder() {
    let opt = ChoiceOption::new("val", "Label").selected();
    assert_eq!(opt.value, "val");
    assert_eq!(opt.label, "Label");
    assert!(opt.selected);
}

// ============================================================================
// DATEFIELD
// ============================================================================

#[test]
fn test_date_field_creation() {
    let field = DateField::new("birthday").label("Date de naissance");
    assert_eq!(field.name(), "birthday");
    assert_eq!(field.field_type(), "date");
}

#[test]
fn test_date_field_valid() {
    let mut field = DateField::new("birthday");
    field.set_value("2000-01-15");
    assert!(field.validate());
}

#[test]
fn test_date_field_invalid_format() {
    let mut field = DateField::new("birthday");
    field.set_value("15/01/2000");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Format de date invalide"));
}

#[test]
fn test_date_field_required_empty() {
    let mut field = DateField::new("birthday").required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Ce champ est obligatoire");
}

#[test]
fn test_date_field_min_max() {
    use chrono::NaiveDate;
    let min = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
    let max = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let mut field = DateField::new("event")
        .min(min, "Trop ancien")
        .max(max, "Trop loin");
    field.set_value("2019-06-15");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop ancien"));
}

// ============================================================================
// TIMEFIELD
// ============================================================================

#[test]
fn test_time_field_creation() {
    let field = TimeField::new("heure");
    assert_eq!(field.name(), "heure");
    assert_eq!(field.field_type(), "time");
}

#[test]
fn test_time_field_valid() {
    let mut field = TimeField::new("heure");
    field.set_value("14:30");
    assert!(field.validate());
}

#[test]
fn test_time_field_invalid() {
    let mut field = TimeField::new("heure");
    field.set_value("25:00");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Format de temps invalide"));
}

// ============================================================================
// DATETIMEFIELD
// ============================================================================

#[test]
fn test_datetime_field_creation() {
    let field = DateTimeField::new("rendez_vous");
    assert_eq!(field.name(), "rendez_vous");
    assert_eq!(field.field_type(), "datetime-local");
}

#[test]
fn test_datetime_field_valid() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("2025-06-15T14:30");
    assert!(field.validate());
}

#[test]
fn test_datetime_field_invalid() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("2025-06-15 14:30");
    assert!(!field.validate());
    assert!(field
        .error()
        .unwrap()
        .contains("Format de date/temps invalide"));
}

// ============================================================================
// DURATIONFIELD
// ============================================================================

#[test]
fn test_duration_field_creation() {
    let field = DurationField::new("duree");
    assert_eq!(field.name(), "duree");
    assert_eq!(field.field_type(), "number");
}

#[test]
fn test_duration_field_valid() {
    let mut field = DurationField::new("duree");
    field.set_value("3600");
    assert!(field.validate());
}

#[test]
fn test_duration_field_invalid() {
    let mut field = DurationField::new("duree");
    field.set_value("abc");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Durée invalide"));
}

#[test]
fn test_duration_field_required_empty() {
    let mut field = DurationField::new("duree").required();
    field.set_value("");
    assert!(!field.validate());
}

// ============================================================================
// FILEFIELD
// ============================================================================

#[test]
fn test_file_field_image() {
    let field = FileField::image("avatar");
    assert_eq!(field.name(), "avatar");
    assert_eq!(field.field_type(), "file");
}

#[test]
fn test_file_field_document() {
    let field = FileField::document("cv");
    assert_eq!(field.name(), "cv");
    assert_eq!(field.field_type(), "file");
}

#[test]
fn test_file_field_any() {
    let field = FileField::any("import");
    assert_eq!(field.name(), "import");
    assert_eq!(field.field_type(), "file");
}

#[test]
fn test_file_field_required_empty() {
    let mut field = FileField::image("photo").required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(
        field.error().unwrap(),
        "Veuillez sélectionner au moins un fichier"
    );
}

#[test]
fn test_file_field_invalid_extension() {
    let mut field = FileField::image("photo");
    field.set_value("malware.exe");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("non autorisé"));
}

#[test]
fn test_file_field_valid_extension() {
    let mut field = FileField::document("cv");
    field.set_value("cv.pdf");
    assert!(field.validate());
}

#[test]
fn test_file_field_svg_always_blocked() {
    let mut field = FileField::any("file");
    field.set_value("image.svg");
    assert!(!field.validate());
}

// ============================================================================
// COLORFIELD
// ============================================================================

#[test]
fn test_color_field_creation() {
    let field = ColorField::new("couleur")
        .label("Couleur préférée")
        .default_color("#ff0000");
    assert_eq!(field.name(), "couleur");
    assert_eq!(field.field_type(), "color");
    assert_eq!(field.value(), "#ff0000");
}

#[test]
fn test_color_field_valid_hex() {
    let mut field = ColorField::new("c");
    field.set_value("#aabbcc");
    assert!(field.validate());
}

#[test]
fn test_color_field_valid_short_hex() {
    let mut field = ColorField::new("c");
    field.set_value("#abc");
    assert!(field.validate());
}

#[test]
fn test_color_field_invalid_no_hash() {
    let mut field = ColorField::new("c");
    field.set_value("ff0000");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("doit commencer par #"));
}

#[test]
fn test_color_field_invalid_length() {
    let mut field = ColorField::new("c");
    field.set_value("#abcd");
    assert!(!field.validate());
    assert!(field
        .error()
        .unwrap()
        .contains("Format de couleur invalide"));
}

#[test]
fn test_color_field_invalid_chars() {
    let mut field = ColorField::new("c");
    field.set_value("#gghhii");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("hexadécimaux"));
}

// ============================================================================
// SLUGFIELD
// ============================================================================

#[test]
fn test_slug_field_creation() {
    let field = SlugField::new("slug");
    assert_eq!(field.name(), "slug");
    assert_eq!(field.field_type(), "text");
}

#[test]
fn test_slug_field_valid() {
    let mut field = SlugField::new("slug");
    field.set_value("mon-article-123");
    assert!(field.validate());
}

#[test]
fn test_slug_field_invalid_special_chars() {
    let mut field = SlugField::new("slug");
    field.set_value("mon article!");
    assert!(!field.validate());
}

#[test]
fn test_slug_field_no_leading_trailing_dash() {
    let mut field = SlugField::new("slug");
    field.set_value("-invalid-slug-");
    assert!(!field.validate());
    assert!(field
        .error()
        .unwrap()
        .contains("commencer ou finir par un tiret"));
}

#[test]
fn test_slug_field_unicode() {
    let mut field = SlugField::new("slug").allow_unicode();
    field.set_value("mon-éditeur-42");
    assert!(field.validate());
}

// ============================================================================
// UUIDFIELD
// ============================================================================

#[test]
fn test_uuid_field_creation() {
    let field = UUIDField::new("token");
    assert_eq!(field.name(), "token");
    assert_eq!(field.field_type(), "text");
}

#[test]
fn test_uuid_field_valid() {
    let mut field = UUIDField::new("token");
    field.set_value("550e8400-e29b-41d4-a716-446655440000");
    assert!(field.validate());
}

#[test]
fn test_uuid_field_invalid() {
    let mut field = UUIDField::new("token");
    field.set_value("pas-un-uuid");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Format UUID invalide"));
}

#[test]
fn test_uuid_field_required_empty() {
    let mut field = UUIDField::new("token").required();
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Ce champ est obligatoire");
}

// ============================================================================
// JSONFIELD
// ============================================================================

#[test]
fn test_json_field_creation() {
    let field = JSONField::new("config").rows(10);
    assert_eq!(field.name(), "config");
    assert_eq!(field.field_type(), "textarea");
}

#[test]
fn test_json_field_valid() {
    let mut field = JSONField::new("data");
    field.set_value(r#"{"key": "value", "count": 42}"#);
    assert!(field.validate());
}

#[test]
fn test_json_field_valid_array() {
    let mut field = JSONField::new("list");
    field.set_value(r#"[1, 2, 3]"#);
    assert!(field.validate());
}

#[test]
fn test_json_field_invalid() {
    let mut field = JSONField::new("data");
    field.set_value("{key: broken}");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "JSON invalide");
}

#[test]
fn test_json_field_required_empty() {
    let mut field = JSONField::new("data").required();
    field.set_value("");
    assert!(!field.validate());
}

// ============================================================================
// IPADDRESSFIELD
// ============================================================================

#[test]
fn test_ip_field_creation() {
    let field = IPAddressField::new("server_ip").label("IP du serveur");
    assert_eq!(field.name(), "server_ip");
    assert_eq!(field.field_type(), "text");
}

#[test]
fn test_ip_field_valid_ipv4() {
    let mut field = IPAddressField::new("ip");
    field.set_value("192.168.1.1");
    assert!(field.validate());
}

#[test]
fn test_ip_field_valid_ipv6() {
    let mut field = IPAddressField::new("ip");
    field.set_value("::1");
    assert!(field.validate());
}

#[test]
fn test_ip_field_invalid() {
    let mut field = IPAddressField::new("ip");
    field.set_value("999.999.999.999");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Adresse IP invalide");
}

#[test]
fn test_ip_field_ipv4_only_rejects_ipv6() {
    let mut field = IPAddressField::new("ip").ipv4_only();
    field.set_value("::1");
    assert!(!field.validate());
    assert_eq!(
        field.error().unwrap(),
        "Seules les adresses IPv4 sont acceptées"
    );
}

#[test]
fn test_ip_field_ipv6_only_rejects_ipv4() {
    let mut field = IPAddressField::new("ip").ipv6_only();
    field.set_value("192.168.1.1");
    assert!(!field.validate());
    assert_eq!(
        field.error().unwrap(),
        "Seules les adresses IPv6 sont acceptées"
    );
}

#[test]
fn test_ip_field_ipv4_only_accepts_ipv4() {
    let mut field = IPAddressField::new("ip").ipv4_only();
    field.set_value("10.0.0.1");
    assert!(field.validate());
}

// ============================================================================
// HIDDENFIELD (CSRF)
// Since the refactor, HiddenField uses a real FieldConfig (base).
// Validation via validate() works as expected.
// ============================================================================

#[test]
fn test_hidden_field_csrf_construction() {
    let field = HiddenField::new_csrf();
    assert_eq!(field.name(), "csrf_token");
    assert_eq!(field.field_type(), "hidden");
    assert_eq!(field.template_name(), "csrf");
    assert!(field.expected_value.is_none());
    assert!(field.error().is_none());
}

#[test]
fn test_hidden_field_csrf_set_value() {
    let mut field = HiddenField::new_csrf();
    field.set_value("abc123");
    assert_eq!(field.value(), "abc123");
}

#[test]
fn test_hidden_field_csrf_expected_value() {
    let mut field = HiddenField::new_csrf();
    field.set_expected_value("token_secret");
    assert_eq!(field.expected_value.as_deref(), Some("token_secret"));
}

#[test]
fn test_hidden_field_csrf_validate_matching() {
    let mut field = HiddenField::new_csrf();
    field.set_value("token_ok");
    field.set_expected_value("token_ok");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_hidden_field_csrf_validate_mismatch() {
    let mut field = HiddenField::new_csrf();
    field.set_value("mauvais");
    field.set_expected_value("attendu");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Token CSRF invalide");
}

#[test]
fn test_hidden_field_csrf_validate_empty() {
    let mut field = HiddenField::new_csrf();
    field.set_value("");
    field.set_expected_value("attendu");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Token CSRF manquant");
}

#[test]
fn test_hidden_field_generic() {
    let field = HiddenField::new("user_id").label("ID");
    assert_eq!(field.name(), "user_id");
    assert_eq!(field.field_type(), "hidden");
}

// ============================================================================
// FORMS — Form manager
// ============================================================================

#[test]
fn test_forms_new() {
    let form = Forms::new("csrf_token_123");
    assert!(form.fields.contains_key("csrf_token"));
}

#[test]
fn test_forms_add_field() {
    let mut form = Forms::new("csrf_token");
    let field = TextField::text("username");
    form.field(&field);
    assert!(form.fields.contains_key("username"));
}

#[test]
fn test_forms_fill_data() {
    let mut form = Forms::new("csrf");
    let mut field = TextField::text("name");
    field.set_value("John Doe");
    form.field(&field);

    if let Some(f) = form.fields.get("name") {
        assert_eq!(f.value(), "John Doe");
    }
}

#[test]
fn test_complex_form_creation() {
    let mut form = Forms::new("csrf_token");

    form.field(
        &TextField::text("username")
            .label("Nom d'utilisateur")
            .placeholder("Entrez un pseudo"),
    );
    form.field(&TextField::email("email").label("Email"));
    form.field(
        &TextField::password("password")
            .label("Mot de passe")
            .placeholder("Entrez votre mot de passe"),
    );
    form.field(&NumericField::integer("age").label("Âge"));

    assert!(form.fields.contains_key("username"));
    assert!(form.fields.contains_key("email"));
    assert!(form.fields.contains_key("password"));
    assert!(form.fields.contains_key("age"));
    assert_eq!(form.fields.len(), 5); // csrf + 4 fields
}

#[test]
fn test_forms_all_field_types() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("nom"));
    form.field(&NumericField::integer("age"));
    form.field(&BooleanField::new("cgu"));
    form.field(&ChoiceField::new("pays").add_choice("fr", "France"));
    form.field(&DateTimeField::new("rdv"));
    form.field(&FileField::image("avatar"));
    form.field(&ColorField::new("theme"));
    form.field(&JSONField::new("config"));
    form.field(&SlugField::new("slug"));

    assert_eq!(form.fields.len(), 10); // csrf + 9 fields
}

// ============================================================================
// PRELUDE — Vérification des exports
// ============================================================================

#[test]
fn test_prelude_exports() {
    // Types principaux accessibles via le prelude
    let _form = Forms::new("test");
    let _text = TextField::text("t");
    let _numeric = NumericField::integer("n");
    let _boolean = BooleanField::new("b");
    let _choice = ChoiceField::new("c");
    let _datetime = DateTimeField::new("dt");
    let _file = FileField::image("f");
    let _hidden = HiddenField::new_csrf();
    let _color = ColorField::new("c");
    let _json = JSONField::new("j");
    let _slug = SlugField::new("s");
}

#[test]
fn test_non_prelude_imports() {
    // Types qui nécessitent un import explicite
    let _radio = RadioField::new("r");
    let _checkbox = CheckboxField::new("c");
    let _date = DateField::new("d");
    let _time = TimeField::new("t");
    let _duration = DurationField::new("d");
    let _uuid = UUIDField::new("u");
    let _ip = IPAddressField::new("i");
    let _opt = ChoiceOption::new("v", "l");
}

#[test]
fn test_field_types_available() {
    // Tous les constructeurs de TextField
    let _text = TextField::text("f");
    let _email = TextField::email("f");
    let _password = TextField::password("f");
    let _textarea = TextField::textarea("f");
    let _richtext = TextField::richtext("f");
    let _url = TextField::url("f");
    // Tous les constructeurs de NumericField
    let _int = NumericField::integer("f");
    let _float = NumericField::float("f");
    let _decimal = NumericField::decimal("f");
    let _percent = NumericField::percent("f");
    let _range = NumericField::range("f", 0.0, 100.0, 50.0);
}

// ============================================================================
// HELPERS DE CONVERSION TYPÉE — Forms::get_*()
// ============================================================================

#[test]
fn test_get_string() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("name"));
    form.add_value("name", "Alice");
    assert_eq!(form.get_string("name"), "Alice");
    // Champ inexistant → String vide
    assert_eq!(form.get_string("inexistant"), "");
}

#[test]
fn test_get_option() {
    let mut form = Forms::new("csrf");
    form.field(&TextField::text("bio"));
    form.field(&TextField::text("vide"));

    form.add_value("bio", "Développeuse Rust");
    form.add_value("vide", "");

    assert_eq!(
        form.get_option("bio"),
        Some("Développeuse Rust".to_string())
    );
    assert_eq!(form.get_option("vide"), None); // vide → None
    assert_eq!(form.get_option("inexistant"), None);
}

#[test]
fn test_get_i32() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("age"));
    form.field(&TextField::text("texte"));

    form.add_value("age", "25");
    form.add_value("texte", "pas un nombre");

    assert_eq!(form.get_i32("age"), 25);
    assert_eq!(form.get_i32("texte"), 0); // parse échoué → 0
    assert_eq!(form.get_i32("inexistant"), 0);
}

#[test]
fn test_get_i64() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("big"));
    form.add_value("big", "9999999999");
    assert_eq!(form.get_i64("big"), 9_999_999_999i64);
}

#[test]
fn test_get_u32() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("count"));
    form.add_value("count", "42");
    assert_eq!(form.get_u32("count"), 42u32);
    // Négatif → 0 (parse échoue pour u32)
    form.add_value("count", "-5");
    assert_eq!(form.get_u32("count"), 0);
}

#[test]
fn test_get_u64() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("id"));
    form.add_value("id", "18446744073709551615");
    assert_eq!(form.get_u64("id"), u64::MAX);
}

#[test]
fn test_get_f32() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::float("ratio"));
    form.add_value("ratio", &std::f32::consts::PI.to_string());
    let val = form.get_f32("ratio");
    assert!((val - std::f32::consts::PI).abs() < f32::EPSILON);
}

#[test]
fn test_get_f64() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::float("price"));
    form.add_value("price", "19.99");
    assert_eq!(form.get_f64("price"), 19.99);
}

#[test]
fn test_get_f64_comma_to_dot() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::float("price"));
    form.add_value("price", "19,99");
    assert_eq!(form.get_f64("price"), 19.99);
}

#[test]
fn test_get_bool() {
    let mut form = Forms::new("csrf");
    form.field(&BooleanField::new("cgu"));
    form.field(&BooleanField::new("news"));
    form.field(&BooleanField::new("html"));
    form.field(&BooleanField::new("off"));

    form.add_value("cgu", "true");
    form.add_value("news", "1");
    form.add_value("html", "on");
    form.add_value("off", "false");

    assert!(form.get_bool("cgu"));
    assert!(form.get_bool("news"));
    assert!(form.get_bool("html"));
    assert!(!form.get_bool("off"));
    assert!(!form.get_bool("inexistant"));
}

#[test]
fn test_get_option_i32() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("age"));
    form.field(&TextField::text("vide"));

    form.add_value("age", "30");
    form.add_value("vide", "");

    assert_eq!(form.get_option_i32("age"), Some(30));
    assert_eq!(form.get_option_i32("vide"), None);
    assert_eq!(form.get_option_i32("inexistant"), None);
}

#[test]
fn test_get_option_i64() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::integer("score"));
    form.add_value("score", "999");
    assert_eq!(form.get_option_i64("score"), Some(999i64));
}

#[test]
fn test_get_option_f64() {
    let mut form = Forms::new("csrf");
    form.field(&NumericField::float("note"));
    form.field(&TextField::text("vide"));

    form.add_value("note", "18,5");
    form.add_value("vide", "");

    assert_eq!(form.get_option_f64("note"), Some(18.5));
    assert_eq!(form.get_option_f64("vide"), None);
}

#[test]
fn test_get_option_bool() {
    let mut form = Forms::new("csrf");
    form.field(&BooleanField::new("active"));
    form.field(&TextField::text("vide"));

    form.add_value("active", "true");
    form.add_value("vide", "");

    assert_eq!(form.get_option_bool("active"), Some(true));
    assert_eq!(form.get_option_bool("vide"), None);
}
