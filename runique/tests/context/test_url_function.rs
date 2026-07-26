//! Tests — context/tera/url.rs
//! Couvre les chemins d'erreur manquants de LinkFunction (argument absent,
//! route introuvable) et les cas nominaux avec paramètres.

use crate::helpers::tera::{kwargs, no_kwargs};
use runique::context::tera::url::LinkFunction;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::{Context, Function, State, TeraResult, Value};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_registry(routes: &[(&str, &str)]) -> Arc<RwLock<HashMap<String, String>>> {
    let map: HashMap<String, String> = routes
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Arc::new(RwLock::new(map))
}

/// Appelle `link()` avec les arguments donnés sur un registre construit à la volée.
fn call_link<const N: usize>(
    routes: &[(&str, &str)],
    args: [(&'static str, Value); N],
) -> TeraResult<String> {
    let ctx = Context::new();
    let state = State::new(&ctx);
    LinkFunction {
        url_registry: make_registry(routes),
    }
    .call(kwargs(args), &state)
}

// ═══════════════════════════════════════════════════════════════
// Chemin nominal — résolution de routes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_link_url_simple() {
    let result = call_link(&[("home", "/")], [("link", Value::from("home"))]).unwrap();
    assert_eq!(result, "/");
}

#[test]
fn test_link_url_avec_chemin() {
    let result = call_link(&[("about", "/about/")], [("link", Value::from("about"))]).unwrap();
    assert_eq!(result, "/about/");
}

#[test]
fn test_link_substitution_parametre_nombre() {
    let result = call_link(
        &[("detail", "/items/{id}/")],
        [("link", Value::from("detail")), ("id", Value::from(42))],
    )
    .unwrap();
    assert_eq!(result, "/items/42/");
}

#[test]
fn test_link_substitution_parametre_string() {
    let result = call_link(
        &[("user", "/users/{username}/")],
        [
            ("link", Value::from("user")),
            ("username", Value::from("alice")),
        ],
    )
    .unwrap();
    assert_eq!(result, "/users/alice/");
}

#[test]
fn test_link_substitution_multiple_parametres() {
    let result = call_link(
        &[("user_post", "/users/{uid}/posts/{pid}/")],
        [
            ("link", Value::from("user_post")),
            ("uid", Value::from(5)),
            ("pid", Value::from(99)),
        ],
    )
    .unwrap();
    assert_eq!(result, "/users/5/posts/99/");
}

// ═══════════════════════════════════════════════════════════════
// Chemins d'erreur (couvre les branches manquantes à 0%)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_link_argument_manquant_retourne_erreur() {
    // Pas de clé "link" : depuis Tera 2, c'est `Kwargs::must_get` qui produit
    // l'erreur, plus un test manuel dans la fonction.
    let ctx = Context::new();
    let state = State::new(&ctx);
    let result = LinkFunction {
        url_registry: make_registry(&[("home", "/")]),
    }
    .call(no_kwargs(), &state);

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("link"), "message inattendu : {msg}");
}

#[test]
fn test_link_route_inexistante_retourne_erreur() {
    let result = call_link(&[], [("link", Value::from("inexistant"))]);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("inexistant"));
}

#[test]
fn test_link_registre_vide_retourne_erreur() {
    assert!(call_link(&[], [("link", Value::from("home"))]).is_err());
}

// ═══════════════════════════════════════════════════════════════
// Query string — branches manquantes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_link_query_string_raw() {
    let result = call_link(
        &[("list", "/items/")],
        [
            ("link", Value::from("list")),
            ("query", Value::from("page=2&q=rust")),
        ],
    )
    .unwrap();
    assert_eq!(result, "/items/?page=2&q=rust");
}

#[test]
fn test_link_query_object_builds_querystring() {
    let result = call_link(
        &[("search", "/search/")],
        [
            ("link", Value::from("search")),
            ("query", Value::from_serializable(&json!({ "page": 3 }))),
        ],
    )
    .unwrap();
    assert!(result.starts_with("/search/?"));
    assert!(result.contains("page=3"));
}

#[test]
fn test_link_query_object_encodes_string_value() {
    let result = call_link(
        &[("search", "/search/")],
        [
            ("link", Value::from("search")),
            (
                "query",
                Value::from_serializable(&json!({ "q": "hello world" })),
            ),
        ],
    )
    .unwrap();
    assert!(result.contains("hello%20world") || result.contains("hello+world"));
}

#[test]
fn test_link_query_empty_string_no_question_mark() {
    let result = call_link(
        &[("list", "/items/")],
        [("link", Value::from("list")), ("query", Value::from(""))],
    )
    .unwrap();
    assert_eq!(result, "/items/");
}

#[test]
fn test_link_query_other_type_appended() {
    // bool value → to_string() fallback
    let result = call_link(
        &[("list", "/items/")],
        [("link", Value::from("list")), ("query", Value::from(true))],
    )
    .unwrap();
    assert!(result.starts_with("/items/"));
}
