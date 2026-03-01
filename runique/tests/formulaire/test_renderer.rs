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
