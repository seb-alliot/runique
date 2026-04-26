// Tests pour sanitize_strict, sanitize_rich, sanitize
// (complement to inline tests in sanitizer.rs)

use runique::utils::forms::sanitizer::{sanitize, sanitize_rich, sanitize_strict};

// ═══════════════════════════════════════════════════════════════
// sanitize_strict — comportements de base
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_strict_texte_vide_retourne_vide() {
    assert_eq!(sanitize_strict(""), "");
}

#[test]
fn test_strict_convertit_en_minuscules() {
    assert_eq!(sanitize_strict("BONJOUR"), "BONJOUR");
}

#[test]
fn test_strict_retire_balise_bold() {
    assert_eq!(sanitize_strict("<b>test</b>"), "test");
}

#[test]
fn test_strict_retire_balise_paragraphe() {
    let out = sanitize_strict("<p>Hello</p>");
    assert!(!out.contains("<p>"));
    assert!(out.contains("Hello"));
}

#[test]
fn test_strict_texte_normal_conserve() {
    assert_eq!(sanitize_strict("bonjour le monde"), "bonjour le monde");
}

// ═══════════════════════════════════════════════════════════════
// sanitize_strict — neutralisation des protocoles dangereux
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_strict_retire_javascript_protocol() {
    let out = sanitize_strict("javascript:alert(1)");
    assert!(!out.contains("javascript:"));
}

#[test]
fn test_strict_retire_vbscript_protocol() {
    let out = sanitize_strict("vbscript:alert(1)");
    assert!(!out.contains("vbscript:"));
}

#[test]
fn test_strict_retire_data_protocol() {
    let out = sanitize_strict("data:text/html,<h1>x</h1>");
    assert!(!out.contains("data:"));
}

#[test]
fn test_strict_retire_file_protocol() {
    let out = sanitize_strict("file:///etc/passwd");
    assert!(!out.contains("file:"));
}

// ═══════════════════════════════════════════════════════════════
// sanitize_rich — comportements de base
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_rich_texte_vide_retourne_vide() {
    assert_eq!(sanitize_rich(""), "");
}

#[test]
fn test_rich_conserve_balise_p_et_strong() {
    let out = sanitize_rich("<p><strong>texte</strong></p>");
    assert!(out.contains("<p>"));
    assert!(out.contains("<strong>"));
}

#[test]
fn test_rich_retire_balise_script() {
    let out = sanitize_rich("<script>alert(1)</script>texte");
    assert!(!out.contains("<script>"));
    assert!(!out.contains("script"));
}

#[test]
fn test_rich_retire_attribut_onclick() {
    let out = sanitize_rich("<p onclick=\"alert(1)\">test</p>");
    assert!(!out.contains("onclick"));
}

#[test]
fn test_rich_autorise_lien_https() {
    let out = sanitize_rich("<a href=\"https://example.com\">lien</a>");
    assert!(out.contains("https://example.com"));
}

#[test]
fn test_rich_retire_lien_javascript() {
    let out = sanitize_rich("<a href=\"javascript:alert(1)\">clique</a>");
    assert!(!out.contains("javascript:"));
}

#[test]
fn test_rich_conserve_balise_code_et_pre() {
    let out = sanitize_rich("<pre><code>let x = 1;</code></pre>");
    assert!(out.contains("<pre>"));
    assert!(out.contains("<code>"));
}

// ═══════════════════════════════════════════════════════════════
// sanitize — dispatch selon le nom du champ
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sanitize_champ_riche_conserve_html() {
    // "description" est dans RICH_CONTENT_FIELDS
    let out = sanitize("description", "<b>gras</b>");
    assert!(out.contains("<b>"));
}

#[test]
fn test_sanitize_champ_riche_content_conserve_html() {
    let out = sanitize("content", "<p>paragraphe</p>");
    assert!(out.contains("<p>"));
}

#[test]
fn test_sanitize_champ_non_riche_supprime_html() {
    let out = sanitize("username", "<b>gras</b>");
    assert!(!out.contains("<b>"));
}

#[test]
fn test_sanitize_champ_non_riche_retire_script() {
    let out = sanitize("email", "<script>alert(1)</script>");
    assert!(!out.contains("script"));
}

#[test]
fn test_sanitize_champ_riche_bloque_quand_meme_script() {
    // Even rich fields should not let scripts pass through
    let out = sanitize("content", "<script>alert(1)</script>texte");
    assert!(!out.contains("<script>"));
}
