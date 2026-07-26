//! Tests — Form filter (injection CSRF et accesseur js)
//! Comportement vérifié :
//!   - CSRF injecté sur le premier champ par index (rendu champ par champ)
//!   - Scripts via l'accesseur explicite `{% form.x.js %}` (field='js') → bloc pré-rendu
//!     (`rendered_js`), nonce CSP réel, jamais de tags Tera littéraux
//!   - Le rendu d'un champ (même le dernier) n'auto-injecte plus les scripts

use crate::helpers::tera::{kwargs, no_kwargs};
use runique::context::tera::form::FormFilter;
use serde_json::json;
use tera::{Context, Filter, State, Value};

/// Construit un Value simulant la sérialisation d'un Forms avec N champs.
///
/// La structure est bâtie en `serde_json` (comme le fait le vrai code de rendu)
/// puis convertie une fois : le filtre reçoit un `tera::Value`.
fn make_form_value(field_names_with_index: &[(&str, u64)], with_scripts: bool) -> Value {
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
        json!(r#"<input type="hidden" name="csrf_token" value="tok123">"#),
    );
    for (name, _) in field_names_with_index {
        rendered.insert(name.to_string(), json!(format!("<input name=\"{name}\">")));
    }

    // `rendered_js` is what `renderer::render_js` produces (real nonce + resolved
    // static URL via the `js.html` template). The `{% form.x.js %}` accessor reads it
    // verbatim — the filter never rebuilds script tags from `js_files`.
    let (js_files, rendered_js) = if with_scripts {
        (
            json!(["filepicker.js"]),
            json!(r#"<script nonce="n0nc3" src="/static/filepicker.js" defer></script>"#),
        )
    } else {
        (json!([]), json!(""))
    };

    Value::from_serializable(&json!({
        "fields": serde_json::Value::Object(fields),
        "rendered_fields": serde_json::Value::Object(rendered),
        "js_files": js_files,
        "rendered_js": rendered_js,
    }))
}

/// Rend un champ nommé et renvoie le HTML produit.
fn render_field(form: &Value, field_name: &'static str) -> String {
    let ctx = Context::new();
    let state = State::new(&ctx);
    FormFilter
        .call(form, kwargs([("field", Value::from(field_name))]), &state)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

// ── CSRF injecté sur le premier champ (index min) ─────────────────────────────

#[test]
fn test_form_filter_csrf_injected_on_first_field() {
    let form = make_form_value(&[("username", 0), ("email", 1)], false);
    let html = render_field(&form, "username");

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
    let html = render_field(&form, "email");

    assert!(
        !html.contains("csrf_token"),
        "Le CSRF ne doit PAS être injecté sur email (pas le premier champ). HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_csrf_injected_on_first_field_with_three_fields() {
    let form = make_form_value(&[("username", 0), ("email", 1), ("password", 2)], false);
    assert!(render_field(&form, "username").contains("csrf_token"));
}

#[test]
fn test_form_filter_csrf_not_injected_on_middle_or_last() {
    let form = make_form_value(&[("username", 0), ("email", 1), ("password", 2)], false);

    for field_name in ["email", "password"] {
        let html = render_field(&form, field_name);
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
    let html = render_field(&form, "js");

    assert!(
        html.contains("filepicker.js") && html.contains(r#"nonce="n0nc3""#),
        "L'accesseur js doit renvoyer le bloc pré-rendu (nonce réel + src résolue). HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_js_accessor_no_literal_tera_tags() {
    let form = make_form_value(&[("username", 0)], true);
    let html = render_field(&form, "js");

    assert!(
        !html.contains("{% static") && !html.contains("{% csp %}"),
        "L'accesseur js ne doit jamais émettre de tags Tera littéraux. HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_js_accessor_empty_when_no_scripts() {
    let form = make_form_value(&[("username", 0), ("email", 1)], false);
    assert!(
        render_field(&form, "js").is_empty(),
        "Sans js_files, l'accesseur js renvoie une chaîne vide"
    );
}

// ── Plus d'auto-injection des scripts via le rendu d'un champ ──────────────────

#[test]
fn test_form_filter_scripts_not_auto_injected_on_last_field() {
    // L'ancienne heuristique injectait les scripts après le dernier champ.
    // Désormais le js passe uniquement par l'accesseur explicite `{% form.x.js %}`.
    let form = make_form_value(&[("username", 0), ("email", 1)], true);
    let html = render_field(&form, "email");

    assert!(
        !html.contains("filepicker.js") && !html.contains("<script"),
        "Le rendu d'un champ (même le dernier) ne doit plus auto-injecter les scripts. HTML: {}",
        html
    );
}

#[test]
fn test_form_filter_scripts_not_injected_on_non_last_field() {
    let form = make_form_value(&[("username", 0), ("email", 1)], true);
    assert!(!render_field(&form, "username").contains("filepicker.js"));
}

// ── Rendu complet (sans field arg) ───────────────────────────────────────────

#[test]
fn test_form_filter_full_render_without_field_arg() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let form = Value::from_serializable(&json!({
        "html": "<div class=\"form\">username + email</div>"
    }));

    let result = FormFilter.call(&form, no_kwargs(), &state).unwrap();
    assert!(result.as_str().unwrap().contains("form"));
}

// ── Champ inconnu → erreur ─────────────────────────────────────────────────────

#[test]
fn test_form_filter_unknown_field_returns_error() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let form = make_form_value(&[("username", 0)], false);

    let result = FormFilter.call(
        &form,
        kwargs([("field", Value::from("nonexistent_field"))]),
        &state,
    );
    assert!(
        result.is_err(),
        "Un champ inconnu doit retourner une erreur"
    );
}

// ── Contrat d'échappement ─────────────────────────────────────────────────────

/// Le filtre déclare `is_safe()` : sa sortie est émise sans échappement. C'est ce
/// qui remplace le `| safe` que le préprocesseur ajoutait à `{% form.x %}`. Si
/// cette déclaration saute, tout le HTML des formulaires ressort échappé et visible.
#[test]
fn test_form_filter_is_safe() {
    assert!(FormFilter.is_safe());
}
