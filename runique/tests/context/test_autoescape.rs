//! Tests — contrat d'échappement des filtres (Tera 2)
//!
//! Depuis la migration Tera 2, la marque « ce HTML est sûr » ne vient plus d'un
//! `| safe` injecté par regex dans le source des templates, mais de `is_safe()` +
//! `Value::safe_string()` portés par le filtre lui-même.
//!
//! Ces tests rendent de vrais templates via Tera : c'est le seul niveau où la
//! différence se voit. Un test qui inspecte uniquement le `Value` retourné par un
//! filtre ne dit rien sur ce qui part réellement au navigateur.

use runique::context::register_asset_filters;
use runique::utils::aliases::new_registry;
use tera::{Context, Tera};

const XSS: &str = r#"<script>alert(1)</script>"#;

/// Instance Tera configurée comme celle du framework : autoescape sur `.html`,
/// filtres Runique enregistrés.
fn tera_with_filters(template: &str) -> (Tera, Context) {
    let mut tera = Tera::default();
    tera.autoescape_on(vec!["html", "xml"]);
    register_asset_filters(
        &mut tera,
        "/static".to_string(),
        "/media".to_string(),
        "/runique/static".to_string(),
        "/runique/media".to_string(),
        new_registry(),
    );
    tera.add_raw_template("t.html", template).unwrap();
    (tera, Context::new())
}

fn render(template: &str, value: &str) -> String {
    let (tera, mut ctx) = tera_with_filters(template);
    ctx.insert("input", value);
    tera.render("t.html", &ctx).unwrap()
}

// ── Le défaut : tout est échappé ──────────────────────────────────────────────

/// L'assertion de référence. Si elle tombe, l'autoescape est cassé et tout le
/// reste de ce fichier ne veut plus rien dire.
#[test]
fn test_plain_variable_is_escaped() {
    let out = render("{{ input }}", XSS);
    assert!(!out.contains("<script>"), "sortie non échappée : {out}");
    assert!(out.contains("&lt;script&gt;"));
}

// ── Filtres qui restent échappés ──────────────────────────────────────────────

/// `plaintext` retire les balises puis décode les entités : un `&lt;` stocké
/// redevient un vrai `<`, que Tera doit ré-échapper une fois. Le filtre ne
/// déclare donc pas `is_safe()` — sinon il émettrait comme HTML le balisage
/// qu'on vient justement de lui demander de neutraliser.
#[test]
fn test_plaintext_stays_escaped() {
    let out = render(
        "{{ input | plaintext }}",
        "&lt;script&gt;alert(1)&lt;/script&gt;",
    );
    assert!(
        !out.contains("<script>"),
        "plaintext a émis du HTML : {out}"
    );
    assert!(out.contains("&lt;script&gt;"));
}

#[test]
fn test_mask_stays_escaped() {
    let out = render("{{ input | mask }}", XSS);
    assert!(!out.contains("<script>"));
}

#[test]
fn test_humanize_stays_escaped() {
    let out = render("{{ input | humanize }}", "<b>a_b</b>");
    assert!(!out.contains("<b>"), "humanize a émis du HTML : {out}");
}

/// Une URL d'asset finit dans un attribut `href`/`src` : si un chemin contrôlé
/// pouvait en sortir sans échappement, il fermerait l'attribut.
#[test]
fn test_static_filter_stays_escaped() {
    let out = render(r#"<img src="{{ input | static }}">"#, r#"a" onerror="x"#);
    assert!(!out.contains(r#"onerror="x""#), "attribut refermé : {out}");
    assert!(out.contains("&quot;"));
}

// ── Filtres qui émettent du HTML (is_safe) ────────────────────────────────────

#[test]
fn test_markdown_emits_html() {
    let out = render("{{ input | markdown }}", "**gras**");
    assert!(
        out.contains("<strong>gras</strong>"),
        "markdown doit sortir en HTML sans `| safe` : {out}"
    );
}

/// Le pendant du test précédent : émettre sans échappement n'autorise pas à
/// émettre n'importe quoi. Le XSS est retiré par l'assainissement, pas par
/// l'échappement.
#[test]
fn test_markdown_strips_xss() {
    let out = render("{{ input | markdown }}", XSS);
    assert!(!out.contains("<script>"), "XSS conservé : {out}");
    assert!(!out.contains("alert(1)") || !out.contains("<script"));
}

#[test]
fn test_sanitize_keeps_allowed_tags_and_drops_scripts() {
    let out = render(
        "{{ input | sanitize }}",
        "<b>ok</b><script>alert(1)</script>",
    );
    assert!(out.contains("<b>ok</b>"), "balise autorisée perdue : {out}");
    assert!(!out.contains("<script"), "script conservé : {out}");
}

/// Sans `| markdown`, la même valeur ressort échappée : la sûreté est portée par
/// le filtre, pas par la variable ni par le template.
#[test]
fn test_same_value_is_escaped_without_the_filter() {
    let markdown = render("{{ input | markdown }}", "**gras**");
    let raw = render("{{ input }}", "**gras**");

    assert!(markdown.contains("<strong>"));
    assert!(!raw.contains("<strong>"));
}
