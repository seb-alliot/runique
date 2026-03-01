// Tests — ChoiceField, RadioField, CheckboxField

use runique::forms::base::FormField;
use runique::forms::fields::choice::{CheckboxField, ChoiceField, ChoiceOption, RadioField};

fn options() -> Vec<ChoiceOption> {
    vec![
        ChoiceOption::new("a", "Option A"),
        ChoiceOption::new("b", "Option B"),
        ChoiceOption::new("c", "Option C"),
    ]
}

// ═══════════════════════════════════════════════════════════════
// ChoiceOption
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_choice_option_new() {
    let opt = ChoiceOption::new("val", "Label");
    assert_eq!(opt.value, "val");
    assert_eq!(opt.label, "Label");
    assert!(!opt.selected);
}

#[test]
fn test_choice_option_selected() {
    let opt = ChoiceOption::new("val", "Label").selected();
    assert!(opt.selected);
}

// ═══════════════════════════════════════════════════════════════
// ChoiceField (select)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_choice_field_new() {
    let field = ChoiceField::new("pays");
    assert_eq!(field.base.name, "pays");
    assert_eq!(field.base.type_field, "select");
    assert!(!field.multiple);
}

#[test]
fn test_choice_field_multiple() {
    let field = ChoiceField::new("pays").multiple();
    assert!(field.multiple);
    assert_eq!(field.base.type_field, "select-multiple");
}

#[test]
fn test_choice_field_vide_non_requis() {
    let mut field = ChoiceField::new("pays").choices(options());
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_choice_field_vide_requis() {
    let mut field = ChoiceField::new("pays").choices(options()).required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_choice_field_requis_message_custom() {
    let mut field = ChoiceField::new("pays").choices(options());
    field.set_required(true, Some("Veuillez choisir".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Veuillez choisir");
}

#[test]
fn test_choice_field_valide() {
    let mut field = ChoiceField::new("pays").choices(options());
    field.set_value("a");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_choice_field_choix_invalide() {
    let mut field = ChoiceField::new("pays").choices(options());
    field.set_value("z");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_choice_field_add_choice() {
    let field = ChoiceField::new("pays")
        .add_choice("fr", "France")
        .add_choice("en", "Angleterre");
    assert_eq!(field.choices.len(), 2);
}

#[test]
fn test_choice_field_label() {
    let field = ChoiceField::new("pays").label("Pays");
    assert_eq!(field.base.label, "Pays");
}

// ═══════════════════════════════════════════════════════════════
// RadioField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_radio_field_new() {
    let field = RadioField::new("genre");
    assert_eq!(field.base.name, "genre");
    assert_eq!(field.base.type_field, "radio");
}

#[test]
fn test_radio_field_vide_non_requis() {
    let mut field = RadioField::new("genre").choices(options());
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_radio_field_vide_requis() {
    let mut field = RadioField::new("genre").choices(options()).required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_radio_field_requis_message_custom() {
    let mut field = RadioField::new("genre").choices(options());
    field.set_required(true, Some("Choix requis".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Choix requis");
}

#[test]
fn test_radio_field_valide() {
    let mut field = RadioField::new("genre").choices(options());
    field.set_value("b");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_radio_field_choix_invalide() {
    let mut field = RadioField::new("genre").choices(options());
    field.set_value("x");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_radio_field_add_choice() {
    let field = RadioField::new("genre")
        .add_choice("m", "Masculin")
        .add_choice("f", "Féminin");
    assert_eq!(field.choices.len(), 2);
}

#[test]
fn test_radio_field_label() {
    let field = RadioField::new("genre").label("Genre");
    assert_eq!(field.base.label, "Genre");
}

// ═══════════════════════════════════════════════════════════════
// CheckboxField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_checkbox_field_new() {
    let field = CheckboxField::new("interets");
    assert_eq!(field.base.name, "interets");
    assert_eq!(field.base.type_field, "checkbox");
}

#[test]
fn test_checkbox_field_vide_non_requis() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("");
    assert!(field.validate());
}

#[test]
fn test_checkbox_field_vide_requis() {
    let mut field = CheckboxField::new("interets").choices(options()).required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_checkbox_field_requis_message_custom() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_required(true, Some("Sélectionnez au moins un".into()));
    field.set_value("");
    assert!(!field.validate());
    assert_eq!(field.error().unwrap(), "Sélectionnez au moins un");
}

#[test]
fn test_checkbox_field_une_valeur_valide() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("a");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_checkbox_field_plusieurs_valides() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("a,b,c");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_checkbox_field_valeur_invalide() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("z");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_checkbox_field_mix_valide_invalide() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("a,z");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_checkbox_field_set_value_marque_selected() {
    let mut field = CheckboxField::new("interets").choices(options());
    field.set_value("a,c");
    let selected: Vec<_> = field.choices.iter().filter(|c| c.selected).collect();
    assert_eq!(selected.len(), 2);
    assert!(field.choices[0].selected); // "a"
    assert!(!field.choices[1].selected); // "b"
    assert!(field.choices[2].selected); // "c"
}

#[test]
fn test_checkbox_field_add_choice() {
    let field = CheckboxField::new("interets")
        .add_choice("sport", "Sport")
        .add_choice("musique", "Musique");
    assert_eq!(field.choices.len(), 2);
}

#[test]
fn test_checkbox_field_label() {
    let field = CheckboxField::new("interets").label("Centres d'intérêt");
    assert_eq!(field.base.label, "Centres d'intérêt");
}
