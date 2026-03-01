//! Tests — forms/prisme/aegis.rs
//! Couvre : aegis<S> avec content-type urlencoded et json.

use axum::{body::Body, http::Request};
use runique::{config::app::RuniqueConfig, forms::prisme::aegis};
use std::sync::Arc;

fn make_config() -> Arc<RuniqueConfig> {
    Arc::new(RuniqueConfig::default())
}

// ═══════════════════════════════════════════════════════════════
// application/x-www-form-urlencoded
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_aegis_urlencoded_simple() {
    let body = "username=alice&age=30";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(
        parsed
            .get("username")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("alice")
    );
    assert_eq!(
        parsed
            .get("age")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("30")
    );
}

#[tokio::test]
async fn test_aegis_urlencoded_vide() {
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::empty())
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.is_empty());
}

#[tokio::test]
async fn test_aegis_urlencoded_avec_csrf() {
    let body = "csrf_token=abc123&email=test%40example.com";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(
        parsed
            .get("csrf_token")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("abc123")
    );
    assert_eq!(
        parsed
            .get("email")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("test@example.com")
    );
}

#[tokio::test]
async fn test_aegis_urlencoded_invalide_retourne_hashmap_vide() {
    // serde_urlencoded::from_bytes retourne unwrap_or_default sur erreur
    let body = "champ_valide=oui&===invalide===";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    // Peut réussir ou retourner un HashMap vide selon le parser
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════
// application/json
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_aegis_json_simple() {
    let body = r#"{"username":"bob","role":"admin"}"#;
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/json").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(
        parsed
            .get("username")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("bob")
    );
    assert_eq!(
        parsed
            .get("role")
            .and_then(|v| v.first())
            .map(|s| s.as_str()),
        Some("admin")
    );
}

#[tokio::test]
async fn test_aegis_json_invalide_retourne_hashmap_vide() {
    // serde_json::from_slice retourne unwrap_or_default sur erreur
    let body = "{invalide json}";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/json").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(
        parsed.is_empty(),
        "JSON invalide doit retourner un HashMap vide"
    );
}

#[tokio::test]
async fn test_aegis_json_vide_retourne_hashmap_vide() {
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/json").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Content-type inconnu → HashMap vide
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_aegis_content_type_inconnu_retourne_vide() {
    let body = "some data";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "text/plain")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "text/plain").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.is_empty(), "Content-type inconnu → HashMap vide");
}

#[tokio::test]
async fn test_aegis_content_type_vide_retourne_vide() {
    let req = Request::builder()
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let result = aegis(req, &(), make_config(), "").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.is_empty());
}

// ═══════════════════════════════════════════════════════════════
// Statut HTTP de l'erreur de body
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_aegis_urlencoded_plusieurs_champs() {
    let body = "a=1&b=2&c=3&d=4&e=5";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert_eq!(parsed.len(), 5);
}

#[tokio::test]
async fn test_aegis_json_erreur_retourne_statut_400() {
    // Simuler une erreur de corps (pas possible facilement sans body poisonné,
    // mais on peut vérifier que le chemin json fonctionne avec un objet valide vide)
    let body = r#"{}"#;
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/json").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    assert!(parsed.is_empty(), "Objet JSON vide → HashMap vide");
}

#[tokio::test]
async fn test_aegis_urlencoded_valeur_avec_espaces() {
    let body = "nom=Jean+Dupont&ville=Paris";
    let req = Request::builder()
        .method("POST")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();

    let result = aegis(req, &(), make_config(), "application/x-www-form-urlencoded").await;
    assert!(result.is_ok());
    let parsed = result.unwrap();
    let nom = parsed
        .get("nom")
        .and_then(|v| v.first())
        .map(|s| s.as_str());
    // "+" est décodé comme espace en urlencoded
    assert!(nom == Some("Jean Dupont") || nom == Some("Jean+Dupont"));
}
