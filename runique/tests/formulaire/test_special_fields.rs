// Tests for special fields: ColorField, SlugField, UUIDField, JSONField, IPAddressField

use runique::forms::base::FormField;
use runique::forms::fields::special::{
    ColorField, IPAddressField, JSONField, SlugField, UUIDField,
};

// ═══════════════════════════════════════════════════════════════
// ColorField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_color_field_new_structure() {
    let field = ColorField::new("couleur");
    assert_eq!(field.base.name, "couleur");
    assert_eq!(field.base.type_field, "color");
}

#[test]
fn test_color_field_valide_rrggbb() {
    let mut field = ColorField::new("couleur");
    field.set_value("#FF5733");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_color_field_valide_rgb_court() {
    let mut field = ColorField::new("couleur");
    field.set_value("#F53");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_color_field_invalide_sans_diese() {
    let mut field = ColorField::new("couleur");
    field.set_value("FF5733");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_color_field_invalide_longueur_incorrecte() {
    let mut field = ColorField::new("couleur");
    field.set_value("#FF573");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_color_field_invalide_caracteres_non_hex() {
    let mut field = ColorField::new("couleur");
    field.set_value("#ZZZZZZ");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_color_field_requis_vide_echoue() {
    let mut field = ColorField::new("couleur").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_color_field_optionnel_vide_passe() {
    let mut field = ColorField::new("couleur");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_color_field_default_color_valide() {
    let field = ColorField::new("couleur").default_color("#336699");
    assert_eq!(field.base.value, "#336699");
}

#[test]
fn test_color_field_default_color_invalide_ignore() {
    let field = ColorField::new("couleur").default_color("invalide");
    assert_eq!(field.base.value, ""); // Couleur invalide ignorée
}

#[test]
fn test_color_field_label() {
    let field = ColorField::new("couleur").label("Ma couleur");
    assert_eq!(field.base.label, "Ma couleur");
}

// ═══════════════════════════════════════════════════════════════
// SlugField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_slug_field_new_structure() {
    let field = SlugField::new("slug");
    assert_eq!(field.base.name, "slug");
    assert!(!field.allow_unicode);
}

#[test]
fn test_slug_field_ascii_valide() {
    let mut field = SlugField::new("slug");
    field.set_value("mon-article-123");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_slug_field_avec_underscore() {
    let mut field = SlugField::new("slug");
    field.set_value("mon_article_123");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_slug_field_commence_par_tiret_echoue() {
    let mut field = SlugField::new("slug");
    field.set_value("-mon-article");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_slug_field_finit_par_tiret_echoue() {
    let mut field = SlugField::new("slug");
    field.set_value("mon-article-");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_slug_field_caracteres_speciaux_echoue() {
    let mut field = SlugField::new("slug");
    field.set_value("mon article");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_slug_field_accents_sans_unicode_echoue() {
    let mut field = SlugField::new("slug");
    field.set_value("éléphant");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_slug_field_unicode_autorise() {
    let mut field = SlugField::new("slug").allow_unicode();
    field.set_value("éléphant");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_slug_field_requis_vide_echoue() {
    let mut field = SlugField::new("slug");
    field.set_required(true, None);
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_slug_field_optionnel_vide_passe() {
    let mut field = SlugField::new("slug");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_slug_field_label_et_placeholder() {
    let field = SlugField::new("slug")
        .label("Identifiant URL")
        .placeholder("mon-article");
    assert_eq!(field.base.label, "Identifiant URL");
    assert_eq!(field.base.placeholder, "mon-article");
}

// ═══════════════════════════════════════════════════════════════
// UUIDField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_uuid_field_new_structure() {
    let field = UUIDField::new("id");
    assert_eq!(field.base.name, "id");
    assert_eq!(field.base.type_field, "text");
}

#[test]
fn test_uuid_field_valide() {
    let mut field = UUIDField::new("id");
    field.set_value("550e8400-e29b-41d4-a716-446655440000");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_uuid_field_invalide_format() {
    let mut field = UUIDField::new("id");
    field.set_value("pas-un-uuid");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_uuid_field_invalide_incomplet() {
    let mut field = UUIDField::new("id");
    field.set_value("550e8400-e29b-41d4");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_uuid_field_requis_vide_echoue() {
    let mut field = UUIDField::new("id").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_uuid_field_optionnel_vide_passe() {
    let mut field = UUIDField::new("id");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_uuid_field_label_et_placeholder() {
    let field = UUIDField::new("id")
        .label("Identifiant")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx");
    assert_eq!(field.base.label, "Identifiant");
    assert_eq!(
        field.base.placeholder,
        "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    );
}

// ═══════════════════════════════════════════════════════════════
// JSONField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_json_field_new_structure() {
    let field = JSONField::new("donnees");
    assert_eq!(field.base.name, "donnees");
    assert_eq!(field.base.type_field, "textarea");
}

#[test]
fn test_json_field_objet_valide() {
    let mut field = JSONField::new("donnees");
    field.set_value(r#"{"clef": "valeur", "nombre": 42}"#);
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_json_field_tableau_valide() {
    let mut field = JSONField::new("donnees");
    field.set_value(r#"[1, 2, 3]"#);
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_json_field_valeur_primitive_valide() {
    let mut field = JSONField::new("donnees");
    field.set_value("42");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_json_field_invalide() {
    let mut field = JSONField::new("donnees");
    field.set_value("{clef: valeur}");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_json_field_requis_vide_echoue() {
    let mut field = JSONField::new("donnees").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_json_field_optionnel_vide_passe() {
    let mut field = JSONField::new("donnees");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_json_field_label_et_placeholder() {
    let field = JSONField::new("donnees")
        .label("Données JSON")
        .placeholder("{\"clef\": \"valeur\"}");
    assert_eq!(field.base.label, "Données JSON");
}

// ═══════════════════════════════════════════════════════════════
// IPAddressField
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_ip_field_new_structure() {
    let field = IPAddressField::new("adresse");
    assert_eq!(field.base.name, "adresse");
    assert!(!field.ipv4_only);
    assert!(!field.ipv6_only);
}

#[test]
fn test_ip_field_ipv4_valide() {
    let mut field = IPAddressField::new("adresse");
    field.set_value("192.168.1.1");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_ip_field_ipv6_valide() {
    let mut field = IPAddressField::new("adresse");
    field.set_value("2001:db8::1");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_ip_field_invalide() {
    let mut field = IPAddressField::new("adresse");
    field.set_value("999.999.999.999");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_ip_field_pas_une_ip() {
    let mut field = IPAddressField::new("adresse");
    field.set_value("pas-une-ip");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_ip_field_ipv4_only_rejette_ipv6() {
    let mut field = IPAddressField::new("adresse").ipv4_only();
    field.set_value("2001:db8::1");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_ip_field_ipv4_only_accepte_ipv4() {
    let mut field = IPAddressField::new("adresse").ipv4_only();
    field.set_value("10.0.0.1");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_ip_field_ipv6_only_rejette_ipv4() {
    let mut field = IPAddressField::new("adresse").ipv6_only();
    field.set_value("192.168.1.1");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_ip_field_ipv6_only_accepte_ipv6() {
    let mut field = IPAddressField::new("adresse").ipv6_only();
    field.set_value("::1");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_ip_field_requis_vide_echoue() {
    let mut field = IPAddressField::new("adresse").required();
    field.set_value("");
    assert!(!field.validate());
    assert!(field.error().is_some());
}

#[test]
fn test_ip_field_optionnel_vide_passe() {
    let mut field = IPAddressField::new("adresse");
    field.set_value("");
    assert!(field.validate());
    assert!(field.error().is_none());
}

#[test]
fn test_ip_field_label() {
    let field = IPAddressField::new("adresse").label("Adresse IP");
    assert_eq!(field.base.label, "Adresse IP");
}

#[test]
fn test_ip_field_ipv4_only_mutual_exclusion() {
    let field = IPAddressField::new("adresse").ipv4_only();
    assert!(field.ipv4_only);
    assert!(!field.ipv6_only);
}

#[test]
fn test_ip_field_ipv6_only_mutual_exclusion() {
    let field = IPAddressField::new("adresse").ipv6_only();
    assert!(field.ipv6_only);
    assert!(!field.ipv4_only);
}
