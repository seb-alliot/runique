// Tests pour csrf middleware

use runique::middleware::security::csrf::CsrfTokenFunction;
use std::collections::HashMap;
use tera::{Function, Value};

#[test]
fn test_csrf_token_function_html() {
    let func = CsrfTokenFunction;
    let mut args = HashMap::new();
    args.insert(
        "csrf_token".to_string(),
        Value::String("tok123".to_string()),
    );
    let html = func.call(&args).unwrap();
    assert!(html.as_str().unwrap().contains("csrf_token"));
    assert!(html.as_str().unwrap().contains("tok123"));
}

#[test]
fn test_csrf_token_function_html_empty() {
    let func = CsrfTokenFunction;
    let args = HashMap::new();
    let html = func.call(&args).unwrap();
    assert!(html.as_str().unwrap().contains("csrf_token"));
    assert!(html.as_str().unwrap().contains("value=\"\""));
}

#[test]
fn test_csrf_token_function_is_safe() {
    let func = CsrfTokenFunction;
    assert!(func.is_safe());
}

#[test]
fn test_csrf_token_function_html_structure() {
    let func = CsrfTokenFunction;
    let mut args = HashMap::new();
    args.insert(
        "csrf_token".to_string(),
        Value::String("abc_token".to_string()),
    );
    let html = func.call(&args).unwrap().as_str().unwrap().to_string();
    assert!(html.starts_with("<input"));
    assert!(html.contains(r#"type="hidden""#));
    assert!(html.contains(r#"name="csrf_token""#));
    assert!(html.contains(r#"value="abc_token""#));
}

// Pour tester la logique middleware (génération/validation de session), il faudrait
// un test d'intégration avec axum et session mockée.
