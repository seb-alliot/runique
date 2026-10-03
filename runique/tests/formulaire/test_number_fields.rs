// Tests — NumericField (integer, float, decimal, percent, range)

use runique::forms::base::FormField;
use runique::forms::fields::number::NumericField;

// ═══════════════════════════════════════════════════════════════
// Integer
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_integer_new() {
    let field = NumericField::integer("age");
    assert_eq!(field.base.name, "age");
    assert_eq!(field.base.type_field, "number");
}

#[tokio::test]
async fn test_integer_vide_non_requis() {
    let mut field = NumericField::integer("age");
    field.set_value("");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_integer_vide_requis() {
    let mut field = NumericField::integer("age");
    field.set_required(true, None);
    field.set_value("");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_integer_requis_message_custom() {
    let mut field = NumericField::integer("age");
    field.set_required(true, Some("Âge requis"));
    field.set_value("");
    assert!(!field.validate().await);
    assert_eq!(field.error().unwrap(), "Âge requis");
}

#[tokio::test]
async fn test_integer_valide() {
    let mut field = NumericField::integer("age");
    field.set_value("42");
    assert!(field.validate().await);
    assert!(field.error().is_none());
}

#[tokio::test]
async fn test_integer_valide_negatif() {
    let mut field = NumericField::integer("solde");
    field.set_value("-10");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_integer_invalide_decimal() {
    let mut field = NumericField::integer("age");
    field.set_value("3.14");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_integer_invalide_texte() {
    let mut field = NumericField::integer("age");
    field.set_value("abc");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_integer_min_respecte() {
    let mut field = NumericField::integer("age").min(18.0, "");
    field.set_value("25");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_integer_min_viole() {
    let mut field = NumericField::integer("age").min(18.0, "");
    field.set_value("10");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_integer_max_respecte() {
    let mut field = NumericField::integer("age").max(120.0, "");
    field.set_value("80");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_integer_max_viole() {
    let mut field = NumericField::integer("age").max(120.0, "");
    field.set_value("200");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_integer_min_max_message_custom() {
    let mut field = NumericField::integer("age")
        .min(18.0, "Trop jeune")
        .max(99.0, "Trop vieux");
    field.set_value("10");
    assert!(!field.validate().await);
    field.set_value("200");
    assert!(!field.validate().await);
}

#[test]
fn test_integer_label() {
    let field = NumericField::integer("age").label("Âge");
    assert_eq!(field.base.label, "Âge");
}

#[test]
fn test_integer_placeholder() {
    let field = NumericField::integer("age").placeholder("Ex: 25");
    assert_eq!(field.base.placeholder, "Ex: 25");
}

// ═══════════════════════════════════════════════════════════════
// Float
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_float_new() {
    let field = NumericField::float("prix");
    assert_eq!(field.base.name, "prix");
}

#[tokio::test]
async fn test_float_vide_non_requis() {
    let mut field = NumericField::float("prix");
    field.set_value("");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_float_valide() {
    let mut field = NumericField::float("prix");
    field.set_value("9.99");
    assert!(field.validate().await);
    assert!(field.error().is_none());
}

#[tokio::test]
async fn test_float_valide_virgule() {
    let mut field = NumericField::float("prix");
    field.set_value("9,99");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_float_invalide_texte() {
    let mut field = NumericField::float("prix");
    field.set_value("pas-un-float");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_float_min_viole() {
    let mut field = NumericField::float("prix").min(0.0, "");
    field.set_value("-1.5");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_float_max_viole() {
    let mut field = NumericField::float("prix").max(100.0, "");
    field.set_value("150.0");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_float_min_max_respectes() {
    let mut field = NumericField::float("prix").min(0.0, "").max(100.0, "");
    field.set_value("50.5");
    assert!(field.validate().await);
}

// ═══════════════════════════════════════════════════════════════
// Decimal
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_decimal_new() {
    let field = NumericField::decimal("montant");
    assert_eq!(field.base.name, "montant");
}

#[tokio::test]
async fn test_decimal_valide() {
    let mut field = NumericField::decimal("montant");
    field.set_value("1234.56");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_decimal_invalide() {
    let mut field = NumericField::decimal("montant");
    field.set_value("abc");
    assert!(!field.validate().await);
}

#[tokio::test]
async fn test_decimal_digits_min_viole() {
    let mut field = NumericField::decimal("montant").digits(2, 4);
    field.set_value("1234.5"); // 1 chiffre après virgule < min 2
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_decimal_digits_max_viole() {
    let mut field = NumericField::decimal("montant").digits(0, 2);
    field.set_value("1234.567"); // 3 chiffres > max 2
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_decimal_digits_respectes() {
    let mut field = NumericField::decimal("montant").digits(2, 4);
    field.set_value("1234.56");
    assert!(field.validate().await);
}

// ═══════════════════════════════════════════════════════════════
// Percent
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_percent_new() {
    let field = NumericField::percent("taux");
    assert_eq!(field.base.name, "taux");
}

#[tokio::test]
async fn test_percent_valide() {
    let mut field = NumericField::percent("taux");
    field.set_value("50");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_percent_sous_zero() {
    let mut field = NumericField::percent("taux");
    field.set_value("-1");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_percent_au_dessus_100() {
    let mut field = NumericField::percent("taux");
    field.set_value("101");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_percent_invalide_texte() {
    let mut field = NumericField::percent("taux");
    field.set_value("abc");
    assert!(!field.validate().await);
}

#[tokio::test]
async fn test_percent_limites() {
    let mut field = NumericField::percent("taux");
    field.set_value("0");
    assert!(field.validate().await);
    field.set_value("100");
    assert!(field.validate().await);
}

// ═══════════════════════════════════════════════════════════════
// Range
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_range_new() {
    let field = NumericField::range("volume", 0.0, 100.0, 50.0);
    assert_eq!(field.base.name, "volume");
    assert_eq!(field.base.value, "50");
}

#[tokio::test]
async fn test_range_valide() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("75");
    assert!(field.validate().await);
}

#[tokio::test]
async fn test_range_sous_min() {
    let mut field = NumericField::range("volume", 10.0, 100.0, 50.0);
    field.set_value("5");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_range_au_dessus_max() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("150");
    assert!(!field.validate().await);
    assert!(field.error().is_some());
}

#[tokio::test]
async fn test_range_invalide_texte() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("abc");
    assert!(!field.validate().await);
}

#[test]
fn test_range_step() {
    let field = NumericField::range("volume", 0.0, 100.0, 50.0).step(5.0);
    assert_eq!(field.base.name, "volume");
}

// Written from cargo-mutants survivors (2026-10-02): the message given to
// `min` / `max` was stored but never shown — the default always won.
async fn error_for(mut field: NumericField, value: &str) -> Option<String> {
    field.set_value(value);
    if field.validate().await {
        None
    } else {
        field.error().cloned()
    }
}

#[tokio::test]
async fn test_min_max_custom_messages_replace_the_default() {
    let int = || {
        NumericField::integer("n")
            .min(1.0, "Trop petit")
            .max(9.0, "Trop grand")
    };
    assert_eq!(error_for(int(), "0").await.as_deref(), Some("Trop petit"));
    assert_eq!(error_for(int(), "10").await.as_deref(), Some("Trop grand"));
    assert_eq!(error_for(int(), "5").await, None);

    let dec = || {
        NumericField::decimal("d")
            .min(0.5, "Min 0,5")
            .max(2.5, "Max 2,5")
    };
    assert_eq!(error_for(dec(), "0.4").await.as_deref(), Some("Min 0,5"));
    assert_eq!(error_for(dec(), "2.6").await.as_deref(), Some("Max 2,5"));

    let pct = || {
        NumericField::percent("p")
            .min(10.0, "Au moins 10")
            .max(90.0, "Au plus 90")
    };
    assert_eq!(error_for(pct(), "5").await.as_deref(), Some("Au moins 10"));
    assert_eq!(error_for(pct(), "95").await.as_deref(), Some("Au plus 90"));
}

#[tokio::test]
async fn test_min_max_without_message_keep_the_default() {
    let field = || NumericField::integer("n").min(1.0, "").max(9.0, "");
    let low = error_for(field(), "0").await.expect("refused");
    let high = error_for(field(), "10").await.expect("refused");
    assert!(!low.is_empty() && !high.is_empty());
    assert_ne!(low, high, "each bound its own default message");
    assert!(!field().base.extra_context.contains_key("min_message"));
}

#[tokio::test]
async fn test_digits_upper_bound_is_inclusive() {
    let field = || NumericField::decimal("d").digits(0, 2);
    assert_eq!(error_for(field(), "1.25").await, None);
    assert!(error_for(field(), "1.255").await.is_some());
}

#[tokio::test]
async fn test_min_max_bounds_are_inclusive() {
    let int = || NumericField::integer("n").min(1.0, "").max(9.0, "");
    assert_eq!(error_for(int(), "1").await, None);
    assert_eq!(error_for(int(), "9").await, None);
    let typed = || NumericField::integer_in("n", -128, 127);
    assert_eq!(error_for(typed(), "-128").await, None);
    assert_eq!(error_for(typed(), "127").await, None);
    assert!(error_for(typed(), "128").await.is_some());
    assert!(error_for(typed(), "-129").await.is_some());
    let dec = || NumericField::decimal("d").min(0.5, "").max(2.5, "");
    assert_eq!(error_for(dec(), "0.5").await, None);
    assert_eq!(error_for(dec(), "2.5").await, None);
    let pct = || NumericField::percent("p").min(10.0, "").max(90.0, "");
    assert_eq!(error_for(pct(), "10").await, None);
    assert_eq!(error_for(pct(), "90").await, None);
}
