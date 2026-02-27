//! Tests — URL Resolution (LinkFunction Tera)
//! Couvre : reverse via LinkFunction, substitution de paramètres dynamiques
//! Bug fix vérifié : link() ignorait les paramètres dynamiques {id}, {slug}, etc.
//!
//! On teste LinkFunction directement (le lieu du bug fix) sans avoir besoin
//! d'instancier RuniqueEngine complet.

use runique::context::tera::url::LinkFunction;
use runique::utils::aliases::new_registry;
use std::collections::HashMap;
use tera::{Function, Value};

/// Insère une route dans un registre
fn with_route(name: &str, path: &str) -> LinkFunction {
    let reg = new_registry();
    reg.write()
        .unwrap()
        .insert(name.to_string(), path.to_string());
    LinkFunction { url_registry: reg }
}

/// Appelle link() avec les args donnés
fn call(f: &LinkFunction, args: &[(&str, Value)]) -> tera::Result<Value> {
    let map: HashMap<String, Value> = args
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    f.call(&map)
}

// ── Résolution simple ──────────────────────────────────────────────────────────

#[test]
fn test_link_simple_route() {
    let f = with_route("index", "/");
    let result = call(&f, &[("link", Value::String("index".into()))]).unwrap();
    assert_eq!(result.as_str().unwrap(), "/");
}

#[test]
fn test_link_named_route() {
    let f = with_route("login", "/login");
    let result = call(&f, &[("link", Value::String("login".into()))]).unwrap();
    assert_eq!(result.as_str().unwrap(), "/login");
}

#[test]
fn test_link_unknown_route_returns_error() {
    let f = LinkFunction {
        url_registry: new_registry(),
    };
    let result = call(&f, &[("link", Value::String("ghost".into()))]);
    assert!(result.is_err());
}

#[test]
fn test_link_without_link_arg_returns_error() {
    let f = with_route("index", "/");
    let result = call(&f, &[("other_key", Value::String("index".into()))]);
    assert!(result.is_err());
}

// ── Substitution de paramètres dynamiques (bug fix) ───────────────────────────

#[test]
fn test_link_with_integer_id_parameter() {
    let f = with_route("blog_detail", "/blog/{id}");
    let result = call(
        &f,
        &[
            ("link", Value::String("blog_detail".into())),
            ("id", Value::Number(42.into())),
        ],
    )
    .unwrap();
    assert_eq!(result.as_str().unwrap(), "/blog/42");
}

#[test]
fn test_link_with_string_id_parameter() {
    let f = with_route("blog_detail", "/blog/{id}");
    let result = call(
        &f,
        &[
            ("link", Value::String("blog_detail".into())),
            ("id", Value::String("99".into())),
        ],
    )
    .unwrap();
    assert_eq!(result.as_str().unwrap(), "/blog/99");
}

#[test]
fn test_link_with_slug_parameter() {
    let f = with_route("article", "/articles/{slug}");
    let result = call(
        &f,
        &[
            ("link", Value::String("article".into())),
            ("slug", Value::String("my-first-post".into())),
        ],
    )
    .unwrap();
    assert_eq!(result.as_str().unwrap(), "/articles/my-first-post");
}

#[test]
fn test_link_with_multiple_parameters() {
    let f = with_route("user_post", "/users/{user_id}/posts/{post_id}");
    let result = call(
        &f,
        &[
            ("link", Value::String("user_post".into())),
            ("user_id", Value::Number(7.into())),
            ("post_id", Value::Number(99.into())),
        ],
    )
    .unwrap();
    let url = result.as_str().unwrap();
    assert!(url.contains("/users/7/posts/99"), "URL: {}", url);
}

#[test]
fn test_link_no_placeholder_ignores_extra_params() {
    let f = with_route("about", "/about");
    let result = call(
        &f,
        &[
            ("link", Value::String("about".into())),
            ("id", Value::Number(42.into())),
        ],
    )
    .unwrap();
    // Les params supplémentaires ne modifient pas la route sans placeholder
    assert_eq!(result.as_str().unwrap(), "/about");
}

#[test]
fn test_link_multiple_routes_independent() {
    let reg = new_registry();
    {
        let mut map = reg.write().unwrap();
        map.insert("index".to_string(), "/".to_string());
        map.insert("profile".to_string(), "/profil/{id}".to_string());
    }
    let f = LinkFunction { url_registry: reg };

    let r1 = call(&f, &[("link", Value::String("index".into()))]).unwrap();
    let r2 = call(
        &f,
        &[
            ("link", Value::String("profile".into())),
            ("id", Value::Number(5.into())),
        ],
    )
    .unwrap();

    assert_eq!(r1.as_str().unwrap(), "/");
    assert_eq!(r2.as_str().unwrap(), "/profil/5");
}
