//! Tests — forms/renderer.rs
//! Couvre : FormRenderer::new, add_js (validation des chemins JS),
//!          render_field (sans tera pour les tests unitaires)

use std::sync::Arc;
use tera::Tera;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn make_renderer() -> runique::forms::renderer::FormRenderer {
    let tera = Arc::new(Tera::default());
    runique::forms::renderer::FormRenderer::new(tera)
}

fn make_renderer_with_templates() -> runique::forms::renderer::FormRenderer {
    let mut tera = Tera::default();
    // Ajouter un template js_files minimal pour tester render_js
    tera.add_raw_template(
        "js_files",
        "{% for f in js_files %}<script src=\"{{ f }}\"></script>{% endfor %}",
    )
    .unwrap();
    // Ajouter un template base_string pour render_field via TextField
    tera.add_raw_template("base_string", "<input name=\"{{ field.name }}\">")
        .unwrap();
    tera.add_raw_template(
        "csrf",
        "<input type=\"hidden\" name=\"csrf_token\" value=\"{{ field.value }}\">",
    )
    .unwrap();
    runique::forms::renderer::FormRenderer::new(Arc::new(tera))
}

// ═══════════════════════════════════════════════════════════════
// Création
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_form_renderer_new_js_files_vide() {
    let renderer = make_renderer();
    assert!(renderer.js_files.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// add_js — chemins valides
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_js_fichier_valide() {
    let mut renderer = make_renderer();
    renderer.add_js(&["components/datepicker.js"]);
    assert_eq!(renderer.js_files.len(), 1);
    assert_eq!(renderer.js_files[0], "components/datepicker.js");
}

#[test]
fn test_add_js_plusieurs_fichiers_valides() {
    let mut renderer = make_renderer();
    renderer.add_js(&["a.js", "b.js", "c.js"]);
    assert_eq!(renderer.js_files.len(), 3);
}

#[test]
fn test_add_js_fichier_sous_dossier() {
    let mut renderer = make_renderer();
    renderer.add_js(&["assets/js/form.js"]);
    assert_eq!(renderer.js_files.len(), 1);
}

// ═══════════════════════════════════════════════════════════════
// add_js — chemins invalides (doivent être ignorés)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_js_sans_extension_js_ignore() {
    let mut renderer = make_renderer();
    renderer.add_js(&["fichier.css"]);
    assert!(
        renderer.js_files.is_empty(),
        "Un fichier .css doit être ignoré"
    );
}

#[test]
fn test_add_js_chemin_absolu_ignore() {
    let mut renderer = make_renderer();
    renderer.add_js(&["/absolute/path.js"]);
    assert!(
        renderer.js_files.is_empty(),
        "Les chemins absolus doivent être ignorés"
    );
}

#[test]
fn test_add_js_chemin_absolu_backslash_ignore() {
    let mut renderer = make_renderer();
    renderer.add_js(&["\\absolute\\path.js"]);
    assert!(
        renderer.js_files.is_empty(),
        "Les chemins avec \\ absolu doivent être ignorés"
    );
}

#[test]
fn test_add_js_traversal_ignore() {
    let mut renderer = make_renderer();
    renderer.add_js(&["../secret.js"]);
    assert!(
        renderer.js_files.is_empty(),
        "Le path traversal ../ doit être ignoré"
    );
}

#[test]
fn test_add_js_traversal_milieu_ignore() {
    let mut renderer = make_renderer();
    renderer.add_js(&["assets/../secret.js"]);
    assert!(
        renderer.js_files.is_empty(),
        "Le path traversal dans le chemin doit être ignoré"
    );
}

#[test]
fn test_add_js_melange_valide_invalide() {
    let mut renderer = make_renderer();
    renderer.add_js(&["valid.js", "/invalid.js", "../evil.js", "also_valid.js"]);
    assert_eq!(
        renderer.js_files.len(),
        2,
        "Seuls les fichiers valides doivent être ajoutés"
    );
    assert!(renderer.js_files.contains(&"valid.js".to_string()));
    assert!(renderer.js_files.contains(&"also_valid.js".to_string()));
}

#[test]
fn test_add_js_liste_vide() {
    let mut renderer = make_renderer();
    renderer.add_js(&[]);
    assert!(renderer.js_files.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// render — erreurs globales sans tera
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_render_sans_champs_ni_erreurs() {
    use indexmap::IndexMap;
    let renderer = make_renderer();
    let fields = IndexMap::new();
    let result = renderer.render(&fields, &[]);
    // Sans champs ni erreurs, le rendu doit réussir (chaîne vide ou minimale)
    assert!(result.is_ok());
}

#[test]
fn test_render_avec_erreurs_globales() {
    use indexmap::IndexMap;
    let renderer = make_renderer();
    let fields = IndexMap::new();
    let errors = vec!["Erreur globale".to_string()];
    let result = renderer.render(&fields, &errors);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Erreur globale"));
}

#[test]
fn test_render_plusieurs_erreurs_globales() {
    use indexmap::IndexMap;
    let renderer = make_renderer();
    let fields = IndexMap::new();
    let errors = vec!["Erreur 1".to_string(), "Erreur 2".to_string()];
    let result = renderer.render(&fields, &errors);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("Erreur 1"));
    assert!(html.contains("Erreur 2"));
}

// ═══════════════════════════════════════════════════════════════
// render_field — rendu d'un champ individuel
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_render_field_avec_template_valide() {
    use runique::forms::fields::TextField;
    let renderer = make_renderer_with_templates();
    let field = TextField::text("username");
    let result = renderer.render_field(&field);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("username"));
}

#[test]
fn test_render_field_sans_template_retourne_erreur() {
    use runique::forms::fields::TextField;
    // make_renderer() n'a pas de template base_string
    let renderer = make_renderer();
    let field = TextField::text("champ");
    let result = renderer.render_field(&field);
    // Doit retourner une erreur car le template base_string est manquant
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════
// render_js — avec des fichiers JS
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_render_avec_js_files_et_template() {
    use indexmap::IndexMap;
    let mut renderer = make_renderer_with_templates();
    renderer.add_js(&["form.js", "datepicker.js"]);

    let fields = IndexMap::new();
    let result = renderer.render(&fields, &[]);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("form.js"));
    assert!(html.contains("datepicker.js"));
}

#[test]
fn test_render_js_sans_template_retourne_erreur() {
    // make_renderer() n'a pas de template js_files
    let mut renderer = make_renderer();
    renderer.add_js(&["script.js"]);

    use indexmap::IndexMap;
    let fields = IndexMap::new();
    let result = renderer.render(&fields, &[]);
    // Doit échouer car le template js_files est absent
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing template: js_files"));
}

#[test]
fn test_render_avec_champ_et_js() {
    use indexmap::IndexMap;
    use runique::forms::fields::TextField;

    let mut renderer = make_renderer_with_templates();
    renderer.add_js(&["ui.js"]);

    let field = TextField::text("nom");
    let mut fields: indexmap::IndexMap<String, Box<dyn runique::forms::base::FormField>> =
        IndexMap::new();
    fields.insert("nom".to_string(), Box::new(field));

    let result = renderer.render(&fields, &[]);
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("ui.js"));
    assert!(html.contains("nom"));
}
