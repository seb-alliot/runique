// Tests — DateField, TimeField, DateTimeField, DurationField

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use runique::forms::base::FormField;
use runique::forms::fields::datetime::{DateField, DateTimeField, DurationField, TimeField};

// ═══════════════════════════════════════════════════════════════
// DateField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_date_field_new() {
    let field = DateField::new("naissance");
    assert_eq!(field.base.name, "naissance");
    assert_eq!(field.base.type_field, "date");
    assert!(field.min_date.is_none());
    assert!(field.max_date.is_none());
}

#[test]
fn test_date_field_vide_non_requis() {
    let mut field = DateField::new("date");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_date_field_vide_requis() {
    let mut field = DateField::new("date").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_date_field_requis_message_custom() {
    let mut field = DateField::new("date");
    field.set_required(true, Some("Date obligatoire".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Date obligatoire");
}

#[test]
fn test_date_field_valide() {
    let mut field = DateField::new("date");
    field.set_value("2024-06-15");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_date_field_format_invalide() {
    let mut field = DateField::new("date");
    field.set_value("15/06/2024");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_date_field_format_invalide_texte() {
    let mut field = DateField::new("date");
    field.set_value("pas-une-date");
    assert!(!field.validate());
}

#[test]
fn test_date_field_min_respecte() {
    let min = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let mut field = DateField::new("date").min(min, "");
    field.set_value("2024-06-15");
    assert!(field.validate());
}

#[test]
fn test_date_field_min_viole() {
    let min = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let mut field = DateField::new("date").min(min, "");
    field.set_value("2024-01-01");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_date_field_min_message_custom() {
    let min = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
    let mut field = DateField::new("date").min(min, "Trop tôt");
    field.set_value("2024-01-01");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop tôt"));
}

#[test]
fn test_date_field_max_respecte() {
    let max = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let mut field = DateField::new("date").max(max, "");
    field.set_value("2024-06-15");
    assert!(field.validate());
}

#[test]
fn test_date_field_max_viole() {
    let max = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let mut field = DateField::new("date").max(max, "");
    field.set_value("2025-06-15");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_date_field_max_message_custom() {
    let max = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let mut field = DateField::new("date").max(max, "Trop tard");
    field.set_value("2025-06-15");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop tard"));
}

#[test]
fn test_date_field_label() {
    let field = DateField::new("date").label("Date de naissance");
    assert_eq!(field.base.label, "Date de naissance");
}

#[test]
fn test_date_field_placeholder() {
    let field = DateField::new("date").placeholder("YYYY-MM-DD");
    assert_eq!(field.base.placeholder, "YYYY-MM-DD");
}

// ═══════════════════════════════════════════════════════════════
// TimeField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_time_field_new() {
    let field = TimeField::new("heure");
    assert_eq!(field.base.name, "heure");
    assert_eq!(field.base.type_field, "time");
    assert!(field.min_time.is_none());
    assert!(field.max_time.is_none());
}

#[test]
fn test_time_field_vide_non_requis() {
    let mut field = TimeField::new("heure");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_time_field_vide_requis() {
    let mut field = TimeField::new("heure").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_time_field_requis_message_custom() {
    let mut field = TimeField::new("heure");
    field.set_required(true, Some("Heure obligatoire".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Heure obligatoire");
}

#[test]
fn test_time_field_valide() {
    let mut field = TimeField::new("heure");
    field.set_value("14:30");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_time_field_format_invalide() {
    let mut field = TimeField::new("heure");
    field.set_value("14h30");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_time_field_format_invalide_texte() {
    let mut field = TimeField::new("heure");
    field.set_value("pas-une-heure");
    assert!(!field.validate());
}

#[test]
fn test_time_field_min_respecte() {
    let min = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let mut field = TimeField::new("heure").min(min, "");
    field.set_value("09:00");
    assert!(field.validate());
}

#[test]
fn test_time_field_min_viole() {
    let min = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let mut field = TimeField::new("heure").min(min, "");
    field.set_value("06:00");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_time_field_min_message_custom() {
    let min = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let mut field = TimeField::new("heure").min(min, "Trop tôt");
    field.set_value("06:00");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop tôt"));
}

#[test]
fn test_time_field_max_respecte() {
    let max = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    let mut field = TimeField::new("heure").max(max, "");
    field.set_value("14:00");
    assert!(field.validate());
}

#[test]
fn test_time_field_max_viole() {
    let max = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    let mut field = TimeField::new("heure").max(max, "");
    field.set_value("20:00");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_time_field_max_message_custom() {
    let max = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    let mut field = TimeField::new("heure").max(max, "Trop tard");
    field.set_value("20:00");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop tard"));
}

#[test]
fn test_time_field_label() {
    let field = TimeField::new("heure").label("Heure de début");
    assert_eq!(field.base.label, "Heure de début");
}

// ═══════════════════════════════════════════════════════════════
// DateTimeField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_datetime_field_new() {
    let field = DateTimeField::new("rdv");
    assert_eq!(field.base.name, "rdv");
    assert_eq!(field.base.type_field, "datetime-local");
}

#[test]
fn test_datetime_field_vide_non_requis() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_datetime_field_vide_requis() {
    let mut field = DateTimeField::new("rdv").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_datetime_field_requis_message_custom() {
    let mut field = DateTimeField::new("rdv");
    field.set_required(true, Some("RDV obligatoire".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "RDV obligatoire");
}

#[test]
fn test_datetime_field_valide() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("2024-06-15T14:30");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_datetime_field_format_invalide() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("2024-06-15 14:30");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_datetime_field_format_invalide_texte() {
    let mut field = DateTimeField::new("rdv");
    field.set_value("pas-un-datetime");
    assert!(!field.validate());
}

#[test]
fn test_datetime_field_min_respecte() {
    let min = NaiveDateTime::parse_from_str("2024-01-01T00:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").min(min, "");
    field.set_value("2024-06-15T14:30");
    assert!(field.validate());
}

#[test]
fn test_datetime_field_min_viole() {
    let min = NaiveDateTime::parse_from_str("2025-01-01T00:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").min(min, "");
    field.set_value("2024-06-15T14:30");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_datetime_field_min_message_custom() {
    let min = NaiveDateTime::parse_from_str("2025-01-01T00:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").min(min, "Date trop ancienne");
    field.set_value("2024-06-15T14:30");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Date trop ancienne"));
}

#[test]
fn test_datetime_field_max_respecte() {
    let max = NaiveDateTime::parse_from_str("2025-12-31T23:59", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").max(max, "");
    field.set_value("2024-06-15T14:30");
    assert!(field.validate());
}

#[test]
fn test_datetime_field_max_viole() {
    let max = NaiveDateTime::parse_from_str("2023-01-01T00:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").max(max, "");
    field.set_value("2024-06-15T14:30");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_datetime_field_max_message_custom() {
    let max = NaiveDateTime::parse_from_str("2023-01-01T00:00", "%Y-%m-%dT%H:%M").unwrap();
    let mut field = DateTimeField::new("rdv").max(max, "Trop tard");
    field.set_value("2024-06-15T14:30");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Trop tard"));
}

#[test]
fn test_datetime_field_label() {
    let field = DateTimeField::new("rdv").label("Rendez-vous");
    assert_eq!(field.base.label, "Rendez-vous");
}

// ═══════════════════════════════════════════════════════════════
// DurationField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_duration_field_new() {
    let field = DurationField::new("duree");
    assert_eq!(field.base.name, "duree");
    assert_eq!(field.base.type_field, "number");
    assert!(field.min_seconds.is_none());
    assert!(field.max_seconds.is_none());
}

#[test]
fn test_duration_field_vide_non_requis() {
    let mut field = DurationField::new("duree");
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_duration_field_vide_requis() {
    let mut field = DurationField::new("duree").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_duration_field_requis_message_custom() {
    let mut field = DurationField::new("duree");
    field.set_required(true, Some("Durée obligatoire".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Durée obligatoire");
}

#[test]
fn test_duration_field_valide() {
    let mut field = DurationField::new("duree");
    field.set_value("3600");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_duration_field_invalide_texte() {
    let mut field = DurationField::new("duree");
    field.set_value("une-heure");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_duration_field_invalide_negatif() {
    let mut field = DurationField::new("duree");
    field.set_value("-60");
    assert!(!field.validate());
}

#[test]
fn test_duration_field_min_respecte() {
    let mut field = DurationField::new("duree").min_seconds(60, "");
    field.set_value("120");
    assert!(field.validate());
}

#[test]
fn test_duration_field_min_viole() {
    let mut field = DurationField::new("duree").min_seconds(3600, "");
    field.set_value("60");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_duration_field_min_message_custom() {
    let mut field = DurationField::new("duree").min_seconds(3600, "Au moins 1 heure");
    field.set_value("60");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Au moins 1 heure"));
}

#[test]
fn test_duration_field_max_respecte() {
    let mut field = DurationField::new("duree").max_seconds(7200, "");
    field.set_value("3600");
    assert!(field.validate());
}

#[test]
fn test_duration_field_max_viole() {
    let mut field = DurationField::new("duree").max_seconds(60, "");
    field.set_value("3600");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_duration_field_max_message_custom() {
    let mut field = DurationField::new("duree").max_seconds(60, "Maximum 1 minute");
    field.set_value("3600");
    assert!(!field.validate());
    assert!(field.error().unwrap().contains("Maximum 1 minute"));
}

#[test]
fn test_duration_field_label() {
    let field = DurationField::new("duree").label("Durée (secondes)");
    assert_eq!(field.base.label, "Durée (secondes)");
}

#[test]
fn test_duration_field_zero_valide() {
    let mut field = DurationField::new("duree");
    field.set_value("0");
    assert!(field.validate());
}
