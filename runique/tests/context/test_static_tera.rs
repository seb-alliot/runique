//! Tests — context/tera/static_tera.rs
//! Couvre : register_asset_filters, mask_filter, csrf_filter, register_filter

use runique::context::register_asset_filters;
use runique::utils::env::css_token;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::{Context, Tera};

// ── Helper ────────────────────────────────────────────────────────────────────

fn make_tera() -> Tera {
    let mut tera = Tera::default();
    let registry: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    register_asset_filters(
        &mut tera,
        "/static".to_string(),
        "/media".to_string(),
        "/runique-static".to_string(),
        "/runique-media".to_string(),
        registry,
    );
    tera
}

// ═══════════════════════════════════════════════════════════════
// Filtre mask
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_mask_masque_caracteres_ascii() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ val | mask }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("val", "secret");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "••••••"); // 6 caractères → 6 bullets
}

#[test]
fn test_mask_chaine_vide() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ val | mask }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("val", "");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_mask_unicode_compte_codepoints() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ val | mask }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("val", "héllo"); // 5 codepoints Unicode
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "•••••");
}

#[test]
fn test_mask_un_seul_caractere() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ val | mask }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("val", "x");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "•");
}

// ═══════════════════════════════════════════════════════════════
// Filtre csrf_field
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_csrf_field_contient_input_hidden() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ token | csrf_field }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("token", "my_csrf_token");
    let result = tera.render("t", &ctx).unwrap();
    assert!(result.contains(r#"type="hidden""#));
}

#[test]
fn test_csrf_field_contient_nom_csrf_token() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ token | csrf_field }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("token", "my_csrf_token");
    let result = tera.render("t", &ctx).unwrap();
    assert!(result.contains(r#"name="csrf_token""#));
}

#[test]
fn test_csrf_field_contient_valeur_token() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ token | csrf_field }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("token", "abc123xyz");
    let result = tera.render("t", &ctx).unwrap();
    assert!(result.contains("abc123xyz"));
}

#[test]
fn test_csrf_field_format_complet() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ token | csrf_field }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("token", "TOKEN");
    let result = tera.render("t", &ctx).unwrap();
    let expected = r#"<input type="hidden" name="csrf_token" value="TOKEN">"#;
    assert_eq!(result, expected);
}

// ═══════════════════════════════════════════════════════════════
// Filtre static
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_static_filter_genere_url_correcte() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | static }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "style.css");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, format!("/static/style.css?v={}", css_token()));
}

#[test]
fn test_static_filter_fichier_avec_slash_initial() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | static }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "/style.css");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, format!("/static/style.css?v={}", css_token()));
}

#[test]
fn test_static_filter_chemin_avec_sous_dossier() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | static }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "css/main.css");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, format!("/static/css/main.css?v={}", css_token()));
}

// ═══════════════════════════════════════════════════════════════
// Filtre media
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_media_filter_genere_url_correcte() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | media }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "photo.jpg");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "/media/photo.jpg");
}

// ═══════════════════════════════════════════════════════════════
// Filtre runique_static
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_static_filter_genere_url_correcte() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | runique_static }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "app.js");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, format!("/runique-static/app.js?v={}", css_token()));
}

// ═══════════════════════════════════════════════════════════════
// Filtre runique_media
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_media_filter_genere_url_correcte() {
    let mut tera = make_tera();
    tera.add_raw_template("t", "{{ file | runique_media }}")
        .unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "video.mp4");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, "/runique-media/video.mp4");
}

// ═══════════════════════════════════════════════════════════════
// register_asset_filters — base_url trimming
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_static_base_url_trailing_slash_normalise() {
    // base_url "/static/" (avec slash final) doit donner le même résultat
    let mut tera = Tera::default();
    let registry: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    register_asset_filters(
        &mut tera,
        "/static/".to_string(), // trailing slash
        "/media/".to_string(),
        "/runique-static/".to_string(),
        "/runique-media/".to_string(),
        registry,
    );
    tera.add_raw_template("t", "{{ file | static }}").unwrap();
    let mut ctx = Context::new();
    ctx.insert("file", "style.css");
    let result = tera.render("t", &ctx).unwrap();
    assert_eq!(result, format!("/static/style.css?v={}", css_token()));
}
