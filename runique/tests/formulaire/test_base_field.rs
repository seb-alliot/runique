//! Tests — forms/base.rs (trait FormField — implémentations par défaut)
//! Couvre : set_name, set_label, set_placeholder, set_readonly, set_disabled,
//!          clear_error, set_html_attribute, to_json_value, to_json_readonly,
//!          to_json_disabled, to_json_attributes, to_json_meta, finalize.

use runique::forms::base::FormField;
use runique::forms::fields::TextField;

// Helper: accède aux getters via le trait (UFCS) pour éviter l'ambiguïté
// avec les builder methods de TextField (label/placeholder prennent un &str)
fn trait_label(f: &dyn FormField) -> &str {
    f.label()
}
fn trait_placeholder(f: &dyn FormField) -> &str {
    f.placeholder()
}
fn trait_name(f: &dyn FormField) -> &str {
    f.name()
}

fn make_field() -> TextField {
    TextField::text("test_field")
}

// ═══════════════════════════════════════════════════════════════
// set_name
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_name_modifie_le_nom() {
    let mut field = make_field();
    field.set_name("nouveau_nom");
    assert_eq!(trait_name(&field), "nouveau_nom");
}

// ═══════════════════════════════════════════════════════════════
// set_label
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_label_modifie_le_label() {
    let mut field = make_field();
    field.set_label("Mon Label");
    assert_eq!(trait_label(&field), "Mon Label");
}

#[test]
fn test_set_label_vide() {
    let mut field = make_field();
    field.set_label("");
    assert_eq!(trait_label(&field), "");
}

// ═══════════════════════════════════════════════════════════════
// set_placeholder
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_placeholder() {
    let mut field = make_field();
    field.set_placeholder("Saisissez une valeur");
    assert_eq!(trait_placeholder(&field), "Saisissez une valeur");
}

// ═══════════════════════════════════════════════════════════════
// set_readonly
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_readonly_true_avec_message() {
    let mut field = make_field();
    field.set_readonly(true, Some("Champ en lecture seule"));
    let json = field.to_json_readonly();
    assert_eq!(json["choice"], serde_json::json!(true));
    assert_eq!(json["message"], serde_json::json!("Champ en lecture seule"));
}

#[test]
fn test_set_readonly_false() {
    let mut field = make_field();
    field.set_readonly(false, None);
    let json = field.to_json_readonly();
    assert_eq!(json["choice"], serde_json::json!(false));
    assert_eq!(json["message"], serde_json::json!(null));
}

#[test]
fn test_set_readonly_defaut_sans_appel() {
    let field = make_field();
    // Sans appel à set_readonly, to_json_readonly doit retourner choice=false
    let json = field.to_json_readonly();
    assert_eq!(json["choice"], serde_json::json!(false));
}

// ═══════════════════════════════════════════════════════════════
// set_disabled
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_disabled_true_avec_message() {
    let mut field = make_field();
    field.set_disabled(true, Some("Champ désactivé"));
    let json = field.to_json_disabled();
    assert_eq!(json["choice"], serde_json::json!(true));
    assert_eq!(json["message"], serde_json::json!("Champ désactivé"));
}

#[test]
fn test_set_disabled_false() {
    let mut field = make_field();
    field.set_disabled(false, None);
    let json = field.to_json_disabled();
    assert_eq!(json["choice"], serde_json::json!(false));
}

#[test]
fn test_set_disabled_defaut_sans_appel() {
    let field = make_field();
    let json = field.to_json_disabled();
    assert_eq!(json["choice"], serde_json::json!(false));
}

// ═══════════════════════════════════════════════════════════════
// clear_error
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_clear_error_supprime_erreur_existante() {
    let mut field = make_field();
    field.set_error("Une erreur".to_string());
    assert!(field.error().is_some());
    field.clear_error();
    assert!(field.error().is_none());
}

#[test]
fn test_clear_error_sans_erreur_ne_panique_pas() {
    let mut field = make_field();
    assert!(field.error().is_none());
    field.clear_error();
    assert!(field.error().is_none());
}

// ═══════════════════════════════════════════════════════════════
// set_html_attribute
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_html_attribute_ajoute_attribut() {
    let mut field = make_field();
    field.set_html_attribute("data-id", "42");
    let attrs = field.to_json_attributes();
    assert_eq!(attrs["data-id"], serde_json::json!("42"));
}

#[test]
fn test_set_html_attribute_plusieurs() {
    let mut field = make_field();
    field.set_html_attribute("class", "form-input");
    field.set_html_attribute("autocomplete", "off");
    let attrs = field.to_json_attributes();
    assert_eq!(attrs["class"], serde_json::json!("form-input"));
    assert_eq!(attrs["autocomplete"], serde_json::json!("off"));
}

#[test]
fn test_set_html_attribute_ecrase_valeur_existante() {
    let mut field = make_field();
    field.set_html_attribute("maxlength", "50");
    field.set_html_attribute("maxlength", "100");
    let attrs = field.to_json_attributes();
    assert_eq!(attrs["maxlength"], serde_json::json!("100"));
}

// ═══════════════════════════════════════════════════════════════
// to_json_value
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_json_value_retourne_valeur_courante() {
    let mut field = make_field();
    field.set_value("bonjour");
    let v = field.to_json_value();
    assert_eq!(v, serde_json::json!("bonjour"));
}

#[test]
fn test_to_json_value_vide_par_defaut() {
    let field = make_field();
    let v = field.to_json_value();
    assert_eq!(v, serde_json::json!(""));
}

// ═══════════════════════════════════════════════════════════════
// to_json_attributes — sans attributs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_json_attributes_vide_par_defaut() {
    let field = make_field();
    let attrs = field.to_json_attributes();
    assert!(
        attrs.as_object().map(|m| m.is_empty()).unwrap_or(false),
        "Sans attributs HTML, to_json_attributes doit retourner un objet vide"
    );
}

// ═══════════════════════════════════════════════════════════════
// to_json_meta
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_to_json_meta_retourne_objet_vide() {
    let field = make_field();
    let meta = field.to_json_meta();
    assert_eq!(meta, serde_json::json!({}));
}

// ═══════════════════════════════════════════════════════════════
// finalize — implémentation par défaut (Ok(()))
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_finalize_retourne_ok_par_defaut() {
    let mut field = TextField::text("champ");
    let result = field.finalize();
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════
// Builder methods via TextField (readonly / disabled)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_builder_readonly_via_builder_method() {
    let field = TextField::text("champ").readonly("Ce champ est en lecture seule");
    let json = field.to_json_readonly();
    assert_eq!(json["choice"], serde_json::json!(true));
    assert_eq!(
        json["message"],
        serde_json::json!("Ce champ est en lecture seule")
    );
}

#[test]
fn test_builder_disabled_via_builder_method() {
    let field = TextField::text("champ").disabled("Fonctionnalité désactivée");
    let json = field.to_json_disabled();
    assert_eq!(json["choice"], serde_json::json!(true));
    assert_eq!(
        json["message"],
        serde_json::json!("Fonctionnalité désactivée")
    );
}

// ═══════════════════════════════════════════════════════════════
// FieldConfig::new — couverture directe
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_field_config_new_valeurs_par_defaut() {
    use runique::forms::base::FieldConfig;
    let config = FieldConfig::new("mon_champ", "text", "base_string");
    assert_eq!(config.name, "mon_champ");
    assert_eq!(config.type_field, "text");
    assert_eq!(config.template_name, "base_string");
    assert_eq!(config.label, "");
    assert_eq!(config.value, "");
    assert!(config.error.is_none());
    assert!(config.html_attributes.is_empty());
    assert!(config.extra_context.is_empty());
}
