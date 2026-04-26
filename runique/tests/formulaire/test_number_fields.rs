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

#[test]
fn test_integer_vide_non_requis() {
    let mut field = NumericField::integer("age");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_integer_vide_requis() {
    let mut field = NumericField::integer("age");
    field.set_required(true, None);
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_integer_requis_message_custom() {
    let mut field = NumericField::integer("age");
    field.set_required(true, Some("Âge requis"));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Âge requis");
}

#[test]
fn test_integer_valide() {
    let mut field = NumericField::integer("age");
    field.set_value("42");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_integer_valide_negatif() {
    let mut field = NumericField::integer("solde");
    field.set_value("-10");
    assert!(field.validate());
}

#[test]
fn test_integer_invalide_decimal() {
    let mut field = NumericField::integer("age");
    field.set_value("3.14");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_integer_invalide_texte() {
    let mut field = NumericField::integer("age");
    field.set_value("abc");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_integer_min_respecte() {
    let mut field = NumericField::integer("age").min(18.0, "");
    field.set_value("25");
    assert!(field.validate());
}

#[test]
fn test_integer_min_viole() {
    let mut field = NumericField::integer("age").min(18.0, "");
    field.set_value("10");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_integer_max_respecte() {
    let mut field = NumericField::integer("age").max(120.0, "");
    field.set_value("80");
    assert!(field.validate());
}

#[test]
fn test_integer_max_viole() {
    let mut field = NumericField::integer("age").max(120.0, "");
    field.set_value("200");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_integer_min_max_message_custom() {
    let mut field = NumericField::integer("age")
        .min(18.0, "Trop jeune")
        .max(99.0, "Trop vieux");
    field.set_value("10");
    assert!(!field.validate());
    field.set_value("200");
    assert!(!field.validate());
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

#[test]
fn test_float_vide_non_requis() {
    let mut field = NumericField::float("prix");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_float_valide() {
    let mut field = NumericField::float("prix");
    field.set_value("9.99");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_float_valide_virgule() {
    let mut field = NumericField::float("prix");
    field.set_value("9,99");
    assert!(field.validate());
}

#[test]
fn test_float_invalide_texte() {
    let mut field = NumericField::float("prix");
    field.set_value("pas-un-float");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_float_min_viole() {
    let mut field = NumericField::float("prix").min(0.0, "");
    field.set_value("-1.5");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_float_max_viole() {
    let mut field = NumericField::float("prix").max(100.0, "");
    field.set_value("150.0");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_float_min_max_respectes() {
    let mut field = NumericField::float("prix").min(0.0, "").max(100.0, "");
    field.set_value("50.5");
    assert!(field.validate());
}

// ═══════════════════════════════════════════════════════════════
// Decimal
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_decimal_new() {
    let field = NumericField::decimal("montant");
    assert_eq!(field.base.name, "montant");
}

#[test]
fn test_decimal_valide() {
    let mut field = NumericField::decimal("montant");
    field.set_value("1234.56");
    assert!(field.validate());
}

#[test]
fn test_decimal_invalide() {
    let mut field = NumericField::decimal("montant");
    field.set_value("abc");
    assert!(!field.validate());
}

#[test]
fn test_decimal_digits_min_viole() {
    let mut field = NumericField::decimal("montant").digits(2, 4);
    field.set_value("1234.5"); // 1 chiffre après virgule < min 2
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_decimal_digits_max_viole() {
    let mut field = NumericField::decimal("montant").digits(0, 2);
    field.set_value("1234.567"); // 3 chiffres > max 2
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_decimal_digits_respectes() {
    let mut field = NumericField::decimal("montant").digits(2, 4);
    field.set_value("1234.56");
    assert!(field.validate());
}

// ═══════════════════════════════════════════════════════════════
// Percent
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_percent_new() {
    let field = NumericField::percent("taux");
    assert_eq!(field.base.name, "taux");
}

#[test]
fn test_percent_valide() {
    let mut field = NumericField::percent("taux");
    field.set_value("50");
    assert!(field.validate());
}

#[test]
fn test_percent_sous_zero() {
    let mut field = NumericField::percent("taux");
    field.set_value("-1");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_percent_au_dessus_100() {
    let mut field = NumericField::percent("taux");
    field.set_value("101");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_percent_invalide_texte() {
    let mut field = NumericField::percent("taux");
    field.set_value("abc");
    assert!(!field.validate());
}

#[test]
fn test_percent_limites() {
    let mut field = NumericField::percent("taux");
    field.set_value("0");
    assert!(field.validate());
    field.set_value("100");
    assert!(field.validate());
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

#[test]
fn test_range_valide() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("75");
    assert!(field.validate());
}

#[test]
fn test_range_sous_min() {
    let mut field = NumericField::range("volume", 10.0, 100.0, 50.0);
    field.set_value("5");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_range_au_dessus_max() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("150");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_range_invalide_texte() {
    let mut field = NumericField::range("volume", 0.0, 100.0, 50.0);
    field.set_value("abc");
    assert!(!field.validate());
}

#[test]
fn test_range_step() {
    let field = NumericField::range("volume", 0.0, 100.0, 50.0).step(5.0);
    assert_eq!(field.base.name, "volume");
}
