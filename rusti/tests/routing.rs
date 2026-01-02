use rusti::{
    reverse,
    reverse_with_parameters,
    register_name_url,
};


#[test]
fn test_register_name_url() {
    // Test l'enregistrement d'une route nommée
    register_name_url("home", "/");
    register_name_url("about", "/about");
    register_name_url("user", "/user/{id}");

    // Vérifier que reverse fonctionne
    let home_url = reverse("home");
    assert_eq!(home_url, Some("/".to_string()));

    let about_url = reverse("about");
    assert_eq!(about_url, Some("/about".to_string()));
}

#[test]
fn test_reverse_with_parameters() {
    register_name_url("user", "/user/{id}");
    register_name_url("post", "/post/{slug}/comment/{comment_id}");

    // Test avec un paramètre
    let user_url = reverse_with_parameters("user", &[("id", "123")]);
    assert_eq!(user_url, Some("/user/123".to_string()));

    // Test avec plusieurs paramètres
    let post_url = reverse_with_parameters("post", &[
        ("slug", "my-post"),
        ("comment_id", "456"),
    ]);
    assert_eq!(post_url, Some("/post/my-post/comment/456".to_string()));
}

#[test]
fn test_reverse_simple() {
    register_name_url("home", "/");
    register_name_url("contact", "/contact");

    assert_eq!(reverse("home"), Some("/".to_string()));
    assert_eq!(reverse("contact"), Some("/contact".to_string()));
}

// Note: Les tests urlpatterns nécessitent un contexte async
// Ils sont testés dans les tests d'intégration plutôt que dans les tests unitaires

// Les tests urlpatterns nécessitent un contexte async et sont mieux testés
// dans les tests d'intégration avec une vraie application

#[test]
fn test_reverse_nonexistent_route() {
    // Test reverse avec une route qui n'existe pas
    // Devrait retourner None
    let url = reverse("nonexistent");
    assert_eq!(url, None);
}

#[test]
fn test_reverse_with_parameters_missing() {
    register_name_url("user", "/user/{id}");

    // Test avec paramètre manquant (devrait remplacer {id} par rien ou garder {id})
    let url = reverse_with_parameters("user", &[("wrong_param", "123")]);
    // Devrait retourner la route avec {id} non remplacé
    assert!(url.is_some());
}

#[test]
fn test_multiple_route_registrations() {
    // Test que plusieurs routes peuvent être enregistrées
    register_name_url("route1", "/route1");
    register_name_url("route2", "/route2");
    register_name_url("route3", "/route3");

    assert_eq!(reverse("route1"), Some("/route1".to_string()));
    assert_eq!(reverse("route2"), Some("/route2".to_string()));
    assert_eq!(reverse("route3"), Some("/route3".to_string()));
}

#[test]
fn test_route_with_special_chars() {
    register_name_url("api", "/api/v1/users");
    register_name_url("search", "/search?q={query}");

    let api_url = reverse("api");
    assert_eq!(api_url, Some("/api/v1/users".to_string()));

    // Les paramètres de query peuvent nécessiter un traitement spécial
    let search_url = reverse_with_parameters("search", &[("query", "test")]);
    assert!(search_url.is_some());
    assert!(search_url.unwrap().contains("test"));
}
