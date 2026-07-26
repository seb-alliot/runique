// Tests pour csrf middleware

use crate::helpers::tera::{kwargs, no_kwargs};
use runique::middleware::security::csrf::CsrfTokenFunction;
use tera::{Context, Function, State, Value};

#[test]
fn test_csrf_token_function_html() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let func = CsrfTokenFunction;

    let html = func
        .call(kwargs([("csrf_token", Value::from("tok123"))]), &state)
        .unwrap();

    assert!(html.as_str().unwrap().contains("csrf_token"));
    assert!(html.as_str().unwrap().contains("tok123"));
}

#[test]
fn test_csrf_token_function_html_empty() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let func = CsrfTokenFunction;

    let html = func.call(no_kwargs(), &state).unwrap();

    assert!(html.as_str().unwrap().contains("csrf_token"));
    assert!(html.as_str().unwrap().contains("value=\"\""));
}

#[test]
fn test_csrf_token_function_is_safe() {
    let func = CsrfTokenFunction;
    assert!(func.is_safe());
}

/// The function declares `is_safe()`, so its output is emitted verbatim. This is
/// what makes escaping the token mandatory rather than cosmetic: a token carrying
/// a quote must not be able to close the `value="…"` attribute and inject markup.
#[test]
fn test_csrf_token_function_escapes_the_token() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let func = CsrfTokenFunction;

    let html = func
        .call(
            kwargs([("csrf_token", Value::from(r#""><script>alert(1)</script>"#))]),
            &state,
        )
        .unwrap();
    let html = html.as_str().unwrap();

    assert!(
        !html.contains("<script>"),
        "the token escaped its attribute: {html}"
    );
    assert!(html.contains("&quot;"), "quote was not escaped: {html}");
    assert!(html.contains("&lt;script&gt;"));
}

#[test]
fn test_csrf_token_function_html_structure() {
    let ctx = Context::new();
    let state = State::new(&ctx);
    let func = CsrfTokenFunction;

    let html = func
        .call(kwargs([("csrf_token", Value::from("abc_token"))]), &state)
        .unwrap();
    let html = html.as_str().unwrap();

    assert!(html.starts_with("<input"));
    assert!(html.contains(r#"type="hidden""#));
    assert!(html.contains(r#"name="csrf_token""#));
    assert!(html.contains(r#"value="abc_token""#));
}

// Pour tester la logique middleware (génération/validation de session), il faudrait
// un test d'intégration avec axum et session mockée.
