//! Tests — FormRenderer
//! Couvre : add_js (validation de chemins), validate_js_path (règles de sécurité)
//! Accès via runique::forms::renderer::FormRenderer (méthode pub)

use runique::forms::renderer::FormRenderer;

/// Helper — crée un FormRenderer sans Tera (on ne teste que add_js)
fn renderer() -> FormRenderer {
    // FormRenderer::new attend un Arc<Tera>. On crée un Tera vide.
    let tera = std::sync::Arc::new(tera::Tera::default());
    FormRenderer::new(tera)
}

// ── add_js — fichiers valides ─────────────────────────────────────────────────

#[test]
fn test_add_js_valid_file() {
    let mut r = renderer();
    r.add_js(&["filepicker.js"]);
    assert_eq!(r.js_files, vec!["filepicker.js"]);
}

#[test]
fn test_add_js_multiple_valid_files() {
    let mut r = renderer();
    r.add_js(&["a.js", "b.js", "c.js"]);
    assert_eq!(r.js_files.len(), 3);
}

#[test]
fn test_add_js_nested_path_valid() {
    let mut r = renderer();
    r.add_js(&["js/widgets/filepicker.js"]);
    assert_eq!(r.js_files, vec!["js/widgets/filepicker.js"]);
}

// ── add_js — fichiers rejetés (règles de sécurité) ───────────────────────────

#[test]
fn test_add_js_rejects_non_js_extension() {
    let mut r = renderer();
    r.add_js(&["malicious.sh"]);
    assert!(
        r.js_files.is_empty(),
        "Les fichiers non-.js doivent être rejetés"
    );
}

#[test]
fn test_add_js_rejects_css_extension() {
    let mut r = renderer();
    r.add_js(&["style.css"]);
    assert!(r.js_files.is_empty());
}

#[test]
fn test_add_js_rejects_absolute_unix_path() {
    let mut r = renderer();
    r.add_js(&["/etc/passwd.js"]);
    assert!(
        r.js_files.is_empty(),
        "Les chemins absolus doivent être rejetés"
    );
}

#[test]
fn test_add_js_rejects_absolute_windows_path() {
    let mut r = renderer();
    r.add_js(&["\\windows\\system32\\evil.js"]);
    assert!(
        r.js_files.is_empty(),
        "Les chemins absolus Windows doivent être rejetés"
    );
}

#[test]
fn test_add_js_rejects_path_traversal() {
    let mut r = renderer();
    r.add_js(&["../../../etc/secret.js"]);
    assert!(r.js_files.is_empty(), "Le path traversal doit être rejeté");
}

#[test]
fn test_add_js_rejects_embedded_traversal() {
    let mut r = renderer();
    r.add_js(&["js/../../../evil.js"]);
    assert!(r.js_files.is_empty());
}

#[test]
fn test_add_js_mixed_valid_and_invalid() {
    let mut r = renderer();
    r.add_js(&["valid.js", "../evil.js", "also_valid.js"]);
    // Seuls les valides passent
    assert_eq!(r.js_files.len(), 2);
    assert!(r.js_files.contains(&"valid.js".to_string()));
    assert!(r.js_files.contains(&"also_valid.js".to_string()));
}

#[test]
fn test_add_js_no_duplicates_tracking() {
    // add_js n'impose pas de déduplication — deux appels ajoutent deux fois
    let mut r = renderer();
    r.add_js(&["widget.js"]);
    r.add_js(&["widget.js"]);
    assert_eq!(r.js_files.len(), 2);
}
