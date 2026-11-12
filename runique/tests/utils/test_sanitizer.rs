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
// sanitize_strict — le texte reste tel que tapé, seules les balises partent
// ═══════════════════════════════════════════════════════════════

/// `javascript:`, `data:`, `file:` are words in a text. The value of a link
/// is checked where it is a link: `TextField::url`.
#[test]
fn test_strict_garde_les_mots_qui_ressemblent_a_un_protocole() {
    for texte in [
        "data: 2024",
        "voir file: rapport final",
        "javascript:alert(1)",
        "vbscript:x",
        "Mon profile: dev",
        "x,data:text/html",
    ] {
        assert_eq!(sanitize_strict(texte), texte);
    }
}

#[test]
fn test_strict_retire_toujours_les_balises_autour() {
    assert_eq!(
        sanitize_strict("data:text/html,<h1>x</h1>"),
        "data:text/html,x"
    );
    assert_eq!(
        sanitize_strict("<a href=\"javascript:alert(1)\">clic</a>"),
        "clic"
    );
}

#[test]
fn test_strict_caractere_multi_octets_ne_panique_pas() {
    // `K` (U+212A, 3 octets) et `İ` (U+0130) changent de longueur en minuscules.
    assert_eq!(
        sanitize_strict("\u{212A}javascript:alert(1)"),
        "\u{212A}javascript:alert(1)"
    );
    assert_eq!(
        sanitize_strict("\u{130}\u{130}JavaScript:x"),
        "\u{130}\u{130}JavaScript:x"
    );
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
