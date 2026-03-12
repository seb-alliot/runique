//! Tests — context/tera/url.rs
//! Couvre les chemins d'erreur manquants de LinkFunction (argument absent,
//! route introuvable) et les cas nominaux avec paramètres.

use runique::context::tera::url::LinkFunction;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Function;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn make_registry(routes: &[(&str, &str)]) -> Arc<RwLock<HashMap<String, String>>> {
    let map: HashMap<String, String> = routes
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Arc::new(RwLock::new(map))
}

fn make_args(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

// ═══════════════════════════════════════════════════════════════
// Chemin nominal — résolution de routes
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_link_url_simple() {
    let func = LinkFunction {
        url_registry: make_registry(&[("home", "/")]),
    };
    let args = make_args(&[("link", json!("home"))]);
    let result = func.call(&args).unwrap();
    assert_eq!(result, Value::String("/".to_string()));
}

#[test]
fn test_link_url_avec_chemin() {
    let func = LinkFunction {
        url_registry: make_registry(&[("about", "/about/")]),
    };
    let args = make_args(&[("link", json!("about"))]);
    let result = func.call(&args).unwrap();
    assert_eq!(result, Value::String("/about/".to_string()));
}

#[test]
fn test_link_substitution_parametre_nombre() {
    let func = LinkFunction {
        url_registry: make_registry(&[("detail", "/items/{id}/")]),
    };
    let args = make_args(&[("link", json!("detail")), ("id", json!(42))]);
    let result = func.call(&args).unwrap();
    assert_eq!(result, Value::String("/items/42/".to_string()));
}

#[test]
fn test_link_substitution_parametre_string() {
    let func = LinkFunction {
        url_registry: make_registry(&[("user", "/users/{username}/")]),
    };
    let args = make_args(&[("link", json!("user")), ("username", json!("alice"))]);
    let result = func.call(&args).unwrap();
    assert_eq!(result, Value::String("/users/alice/".to_string()));
}

#[test]
fn test_link_substitution_multiple_parametres() {
    let func = LinkFunction {
        url_registry: make_registry(&[("user_post", "/users/{uid}/posts/{pid}/")]),
    };
    let args = make_args(&[
        ("link", json!("user_post")),
        ("uid", json!(5)),
        ("pid", json!(99)),
    ]);
    let result = func.call(&args).unwrap();
    assert_eq!(result, Value::String("/users/5/posts/99/".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// Chemins d'erreur (couvre les branches manquantes à 0%)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_link_argument_manquant_retourne_erreur() {
    let func = LinkFunction {
        url_registry: make_registry(&[("home", "/")]),
    };
    let args: HashMap<String, Value> = HashMap::new(); // pas de clé "link"
    let result = func.call(&args);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("link"));
}

#[test]
fn test_link_route_inexistante_retourne_erreur() {
    let func = LinkFunction {
        url_registry: make_registry(&[]), // registre vide
    };
    let args = make_args(&[("link", json!("inexistant"))]);
    let result = func.call(&args);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("inexistant"));
}

#[test]
fn test_link_registre_vide_retourne_erreur() {
    let func = LinkFunction {
        url_registry: make_registry(&[]),
    };
    let args = make_args(&[("link", json!("home"))]);
    assert!(func.call(&args).is_err());
}
