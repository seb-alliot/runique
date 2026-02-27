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

// Pour tester la logique middleware, il faudrait un test d'intégration avec axum et session mockée.
// On vérifie ici la structure de CsrfTokenFunction et la robustesse de call.
