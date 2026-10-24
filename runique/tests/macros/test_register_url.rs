//! Tests — macros/routeur/register_url.rs
//! Couvre : register_pending, PENDING_URLS drain, reverse, reverse_with_parameters

use runique::macros::routeur::register_url::{
    UrlError, register_pending, reverse, reverse_with_parameters,
};
use runique::{config::app::RuniqueConfig, engine::RuniqueEngine};
use sea_orm::Database;
use std::sync::Arc;
use tera::Tera;

// ─── Helpers ────────────────────────────────────────────────────────────────

async fn make_engine() -> Arc<RuniqueEngine> {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("sqlite en mémoire");
    let config = RuniqueConfig::default();
    let tera = Tera::default();
    Arc::new(RuniqueEngine::new(config, tera, db))
}

// ═══════════════════════════════════════════════════════════════
// register_pending
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_register_pending_ne_panique_pas() {
    register_pending("home", "/");
    register_pending("users_list", "/users/");
}

// ═══════════════════════════════════════════════════════════════
// register_name_url + reverse
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_reverse_url_enregistre() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "home", "/");
    let url = reverse(&engine, "home");
    assert_eq!(url, Some("/".to_string()));
}

#[tokio::test]
async fn test_reverse_url_inexistante_retourne_none() {
    let engine = make_engine().await;
    let url = reverse(&engine, "url_qui_nexiste_pas");
    assert_eq!(url, Err(UrlError::UnknownRoute("inexistant".into())));
}

#[tokio::test]
async fn test_reverse_plusieurs_urls() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "list", "/items/");
    register_name_url(&engine, "detail", "/items/{id}/");
    assert_eq!(reverse(&engine, "list"), Some("/items/".to_string()));
    assert_eq!(reverse(&engine, "detail"), Some("/items/{id}/".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// reverse_with_parameters
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_reverse_with_parameters_substitue() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "detail", "/items/{id}/");
    let url = reverse_with_parameters(&engine, "detail", &[("id", "42")]);
    assert_eq!(url, Ok("/items/42/".to_string()));
}

#[tokio::test]
async fn test_reverse_with_parameters_multiple() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "user_post", "/users/{user_id}/posts/{post_id}/");
    let url = reverse_with_parameters(&engine, "user_post", &[("user_id", "5"), ("post_id", "99")]);
    assert_eq!(url, Ok("/users/5/posts/99/".to_string()));
}

#[tokio::test]
async fn test_reverse_with_parameters_url_inexistante() {
    let engine = make_engine().await;
    let url = reverse_with_parameters(&engine, "inexistant", &[("id", "1")]);
    assert_eq!(url, Err(UrlError::UnknownRoute("inexistant".into())));
}

#[tokio::test]
async fn test_reverse_with_parameters_sans_substitution() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "static_url", "/about/");
    let url = reverse_with_parameters(&engine, "static_url", &[]);
    assert_eq!(url, Ok("/about/".to_string()));
}

// A value never adds a segment, a query or a fragment.
#[tokio::test]
async fn test_reverse_with_parameters_encode_les_valeurs() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "detail", "/items/{id}/");
    let url = reverse_with_parameters(&engine, "detail", &[("id", "x/../admin?a=1#f")]);
    assert_eq!(url, Ok("/items/x%2F..%2Fadmin%3Fa%3D1%23f/".to_string()));
}

#[tokio::test]
async fn test_reverse_with_parameters_parametre_manquant() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "user_post", "/users/{user_id}/posts/{post_id}/");
    let url = reverse_with_parameters(&engine, "user_post", &[("user_id", "5")]);
    assert_eq!(
        url,
        Err(UrlError::MissingParam {
            route: "user_post".into(),
            param: "post_id".into(),
        })
    );
}

#[tokio::test]
async fn test_reverse_with_parameters_parametre_inconnu() {
    use runique::macros::routeur::register_url::register_name_url;
    let engine = make_engine().await;
    register_name_url(&engine, "detail", "/items/{id}/");
    let url = reverse_with_parameters(&engine, "detail", &[("id", "1"), ("ids", "2")]);
    assert_eq!(
        url,
        Err(UrlError::UnknownParam {
            route: "detail".into(),
            param: "ids".into(),
        })
    );
}
