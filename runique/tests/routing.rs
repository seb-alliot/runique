use runique::{app_state::AppState, register_name_url, reverse, reverse_with_parameters, Settings};
use sea_orm::Database;
use std::sync::Arc;
use tera::Tera;

// Fonction utilitaire mise à jour
async fn setup_test_state() -> Arc<AppState> {
    let settings = Settings::default_values();
    let tera = Arc::new(Tera::default());

    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Échec de la création de la DB de test");

    Arc::new(AppState::new(tera, Arc::new(settings), db))
}

#[tokio::test]
async fn test_register_name_url() {
    let state = setup_test_state().await;

    // On passe maintenant 'state' aux fonctions
    register_name_url(&state, "home", "/");
    register_name_url(&state, "about", "/about");
    register_name_url(&state, "user", "/user/{id}");

    assert_eq!(reverse(&state, "home"), Some("/".to_string()));
    assert_eq!(reverse(&state, "about"), Some("/about".to_string()));
}

#[tokio::test]
async fn test_reverse_with_parameters() {
    let state = setup_test_state().await;
    register_name_url(&state, "user", "/user/{id}");
    register_name_url(&state, "post", "/post/{slug}/comment/{comment_id}");

    // Test avec un paramètre
    let user_url = reverse_with_parameters(&state, "user", &[("id", "123")]);
    assert_eq!(user_url, Some("/user/123".to_string()));

    // Test avec plusieurs paramètres
    let post_url = reverse_with_parameters(
        &state,
        "post",
        &[("slug", "my-post"), ("comment_id", "456")],
    );
    assert_eq!(post_url, Some("/post/my-post/comment/456".to_string()));
}

#[tokio::test]
async fn test_reverse_nonexistent_route() {
    let state = setup_test_state().await;
    let url = reverse(&state, "nonexistent");
    assert_eq!(url, None);
}

#[tokio::test]
async fn test_reverse_with_parameters_missing() {
    let state = setup_test_state().await;
    register_name_url(&state, "user", "/user/{id}");

    let url = reverse_with_parameters(&state, "user", &[("wrong_param", "123")]);
    // Retourne la route brute car {id} n'a pas été trouvé dans les paramètres fournis
    assert_eq!(url, Some("/user/{id}".to_string()));
}
