//! Tests — URL Resolution (LinkFunction Tera)
//! Couvre : reverse via LinkFunction, substitution de paramètres dynamiques
//! Bug fix vérifié : link() ignorait les paramètres dynamiques {id}, {slug}, etc.
//!
//! On teste LinkFunction directement (le lieu du bug fix) sans avoir besoin
//! d'instancier RuniqueEngine complet.

use crate::helpers::tera::kwargs;
use runique::context::tera::url::LinkFunction;
use runique::utils::aliases::new_registry;
use tera::{Context, Function, State, TeraResult, Value};

/// Insère une route dans un registre
fn with_route(name: &str, path: &str) -> LinkFunction {
    let reg = new_registry();
    reg.write()
        .unwrap()
        .insert(name.to_string(), path.to_string());
    LinkFunction { url_registry: reg }
}

/// Appelle link() avec les args donnés
fn call<const N: usize>(f: &LinkFunction, args: [(&'static str, Value); N]) -> TeraResult<String> {
    let ctx = Context::new();
    let state = State::new(&ctx);
    f.call(kwargs(args), &state)
}

// ── Résolution simple ──────────────────────────────────────────────────────────

#[test]
fn test_link_simple_route() {
    let f = with_route("index", "/");
    assert_eq!(call(&f, [("link", Value::from("index"))]).unwrap(), "/");
}

#[test]
fn test_link_named_route() {
    let f = with_route("login", "/login");
    assert_eq!(
        call(&f, [("link", Value::from("login"))]).unwrap(),
        "/login"
    );
}

#[test]
fn test_link_unknown_route_returns_error() {
    let f = LinkFunction {
        url_registry: new_registry(),
    };
    assert!(call(&f, [("link", Value::from("ghost"))]).is_err());
}

#[test]
fn test_link_without_link_arg_returns_error() {
    let f = with_route("index", "/");
    assert!(call(&f, [("other_key", Value::from("index"))]).is_err());
}

// ── Substitution de paramètres dynamiques (bug fix) ───────────────────────────

#[test]
fn test_link_with_integer_id_parameter() {
    let f = with_route("blog_detail", "/blog/{id}");
    let result = call(
        &f,
        [
            ("link", Value::from("blog_detail")),
            ("id", Value::from(42)),
        ],
    )
    .unwrap();
    assert_eq!(result, "/blog/42");
}

#[test]
fn test_link_with_string_id_parameter() {
    let f = with_route("blog_detail", "/blog/{id}");
    let result = call(
        &f,
        [
            ("link", Value::from("blog_detail")),
            ("id", Value::from("99")),
        ],
    )
    .unwrap();
    assert_eq!(result, "/blog/99");
}

#[test]
fn test_link_with_slug_parameter() {
    let f = with_route("article", "/articles/{slug}");
    let result = call(
        &f,
        [
            ("link", Value::from("article")),
            ("slug", Value::from("my-first-post")),
        ],
    )
    .unwrap();
    assert_eq!(result, "/articles/my-first-post");
}

#[test]
fn test_link_with_multiple_parameters() {
    let f = with_route("user_post", "/users/{user_id}/posts/{post_id}");
    let url = call(
        &f,
        [
            ("link", Value::from("user_post")),
            ("user_id", Value::from(7)),
            ("post_id", Value::from(99)),
        ],
    )
    .unwrap();
    assert!(url.contains("/users/7/posts/99"), "URL: {}", url);
}

#[test]
fn test_link_no_placeholder_ignores_extra_params() {
    let f = with_route("about", "/about");
    let result = call(
        &f,
        [("link", Value::from("about")), ("id", Value::from(42))],
    )
    .unwrap();
    // Les params supplémentaires ne modifient pas la route sans placeholder
    assert_eq!(result, "/about");
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

    let r1 = call(&f, [("link", Value::from("index"))]).unwrap();
    let r2 = call(
        &f,
        [("link", Value::from("profile")), ("id", Value::from(5))],
    )
    .unwrap();

    assert_eq!(r1, "/");
    assert_eq!(r2, "/profil/5");
}
