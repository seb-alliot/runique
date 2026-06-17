//! Tests — Form filter (injection CSRF et accesseur js)
//! Comportement vérifié :
//!   - CSRF injecté sur le premier champ par index (rendu champ par champ)
//!   - Scripts via l'accesseur explicite `{% form.x.js %}` (field='js') → bloc pré-rendu
//!     (`rendered_js`), nonce CSP réel, jamais de tags Tera littéraux
//!   - Le rendu d'un champ (même le dernier) n'auto-injecte plus les scripts

use runique::context::tera::form::form_filter;
use std::collections::HashMap;
use tera::Value;

/// Construit un Value simulant la sérialisation d'un Forms avec N champs
fn make_form_value(field_names_with_index: &[(&str, u64)], with_scripts: bool) -> Value {
    use serde_json::json;

    let mut fields = serde_json::Map::new();
    fields.insert(
        "csrf_token".to_string(),
        json!({ "name": "csrf_token", "index": 0u64 }),
    );
    for (name, idx) in field_names_with_index {
        fields.insert(name.to_string(), json!({ "name": name, "index": idx + 1 }));
    }

    let mut rendered = serde_json::Map::new();
    rendered.insert(
        "csrf_token".to_string(),
        Value::String(r#"<input type="hidden" name="csrf_token" value="tok123">"#.to_string()),
    );
    for (name, _) in field_names_with_index {
        rendered.insert(
            name.to_string(),
            Value::String(format!("<input name=\"{}\">", name)),
        );
    }

    // `rendered_js` is what `renderer::render_js` produces (real nonce + resolved
    // static URL via the `js.html` template). The `{% form.x.js %}` accessor reads it
    // verbatim — the filter never rebuilds script tags from `js_files`.
    let (js_files, rendered_js) = if with_scripts {
        (
            json!(["filepicker.js"]),
            Value::String(
                r#"<script nonce="n0nc3" src="/static/filepicker.js" defer></script>"#.to_string(),
            ),
        )
    } else {
        (json!([]), Value::String(String::new()))
    };

    json!({
        "fields": Value::Object(fields),
        "rendered_fields": Value::Object(rendered),
        "js_files": js_files,
        "rendered_js": rendered_js,
    })
}

/// Construit les args pour form_filter (HashMap<String, Value>)
fn args_field(field_name: &str) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    map.insert("field".to_string(), Value::String(field_name.to_string()));
    map
}

fn args_empty() -> HashMap<String, Value> {
    HashMap::new()
}

// ── CSRF injecté sur le premier champ (index min) ─────────────────────────────

#[test]
fn test_form_filter_csrf_injected_on_first_field() {
    let form = make_form_value(&[("username", 0), ("email", 1)], false);
    let args = args_field("username");

    let result = form_filter(&form, &args).unwrap();
    let html = result.as_str().unwrap();

    assert!(
        html.contains("csrf_token"),
        "Le CSRF doit être injecté sur le premier champ (username, index=0). HTML: {}",
        html
    );
    assert!(html.contains("<input name=\"username\">"));
}

#[test]
fn test_form_filter_csrf_not_injected_on_non_first_field() {
    let form = make_form_value(&[("username", 0), ("email", 1)], false);
    let args = args_field("email");

    let result = form_filter(&form, &args).unwrap();
    let html = result.as_str().unwrap();

    assert!(
        !html.contains("csrf_token"),
        "Le CSRF ne doit PAS être injecté sur email (pas le premier champ). HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_csrf_injected_on_first_field_with_three_fields() {
    let form = make_form_value(&[("username", 0), ("email", 1), ("password", 2)], false);
    let result = form_filter(&form, &args_field("username")).unwrap();
    assert!(result.as_str().unwrap().contains("csrf_token"));
}

#[test]
fn test_form_filter_csrf_not_injected_on_middle_or_last() {
    let form = make_form_value(&[("username", 0), ("email", 1), ("password", 2)], false);

    for field_name in ["email", "password"] {
        let result = form_filter(&form, &args_field(field_name)).unwrap();
        let html = result.as_str().unwrap();
        assert!(
            !html.contains("csrf_token"),
            "CSRF ne doit pas apparaître sur '{}'. HTML: {}",
            field_name,
            html
        );
    }
}

// ── Accesseur `{% form.x.js %}` (field='js') ──────────────────────────────────

#[test]
fn test_form_filter_js_accessor_returns_prerendered_block() {
    let form = make_form_value(&[("username", 0), ("email", 1)], true);
    let result = form_filter(&form, &args_field("js")).unwrap();
    let html = result.as_str().unwrap();

    assert!(
        html.contains("filepicker.js") && html.contains(r#"nonce="n0nc3""#),
        "L'accesseur js doit renvoyer le bloc pré-rendu (nonce réel + src résolue). HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_js_accessor_no_literal_tera_tags() {
    let form = make_form_value(&[("username", 0)], true);
    let result = form_filter(&form, &args_field("js")).unwrap();
    let html = result.as_str().unwrap();

    assert!(
        !html.contains("{% static") && !html.contains("{% csp %}"),
        "L'accesseur js ne doit jamais émettre de tags Tera littéraux. HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_js_accessor_empty_when_no_scripts() {
    let form = make_form_value(&[("username", 0), ("email", 1)], false);
    let result = form_filter(&form, &args_field("js")).unwrap();
    assert!(
        result.as_str().unwrap().is_empty(),
        "Sans js_files, l'accesseur js renvoie une chaîne vide"
    );
}

// ── Plus d'auto-injection des scripts via le rendu d'un champ ──────────────────

#[test]
fn test_form_filter_scripts_not_auto_injected_on_last_field() {
    // L'ancienne heuristique injectait les scripts après le dernier champ.
    // Désormais le js passe uniquement par l'accesseur explicite `{% form.x.js %}`.
    let form = make_form_value(&[("username", 0), ("email", 1)], true);
    let result = form_filter(&form, &args_field("email")).unwrap();
    let html = result.as_str().unwrap();

    assert!(
        !html.contains("filepicker.js") && !html.contains("<script"),
        "Le rendu d'un champ (même le dernier) ne doit plus auto-injecter les scripts. HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_scripts_not_injected_on_non_last_field() {
    let form = make_form_value(&[("username", 0), ("email", 1)], true);
    let result = form_filter(&form, &args_field("username")).unwrap();
    assert!(!result.as_str().unwrap().contains("filepicker.js"));
}

// ── Rendu complet (sans field arg) ───────────────────────────────────────────

#[test]
fn test_form_filter_full_render_without_field_arg() {
    use serde_json::json;
    let form = json!({
        "html": "<div class=\"form\">username + email</div>"
    });
    let result = form_filter(&form, &args_empty()).unwrap();
    assert!(result.as_str().unwrap().contains("form"));
}

// ── Champ inconnu → erreur ─────────────────────────────────────────────────────

#[test]
fn test_form_filter_unknown_field_returns_error() {
    let form = make_form_value(&[("username", 0)], false);
    let result = form_filter(&form, &args_field("nonexistent_field"));
    assert!(
        result.is_err(),
        "Un champ inconnu doit retourner une erreur"
    );
}
