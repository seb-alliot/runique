#[macro_export]
macro_rules! urlpatterns {
    (
    // On récupère les paires chemin-handler avec leur valeur syntaxique
    // Lire la documentation de macro_rules! pour comprendre cette syntaxe
    $($path:expr => $method:ident($handler:expr), name = $name:expr) ,* $(,)?
    ) => {{
        // On crée un routeur vide
        let mut router = $crate::Router::new();

        // Pour chaque paire chemin-handler trouvé précédemment on ajoute la route au routeur
        $(
            // On enregistre le nom et le chemin dans la table de routage
            $crate::register_name_url($name, $path);
            router = router.route(
                $path,
                $crate::axum::routing::$method($handler)
            );
        )*
        router
    }};
    (
        $($path:expr => $method:ident($handler:expr)) , * $(,)?
    ) => {{
        let mut router = $crate::Router::new();
        $(
            router = router.route(
                $path,
                $crate::axum::routing::$method($handler)
            );
        )*
        router
    }};
}