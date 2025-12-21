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


// /// Macro pour le reverse routing (url!)
// #[macro_export]
// macro_rules! url {
//     // 1. Cas Sans paramètres
//     ($name:expr) => {{
//             $crate::macro_perso::router::reverse($name)
//                 .unwrap_or_default()
//     }};

//     // 2. Cas Avec paramètres
//     ($name:expr, $($key:ident = $value:expr),+ $(,)?) => {{
//         // Créer un vecteur temporaire
//         let params_vec: Vec<(&str, &str)> = vec![
//             $((stringify!($key).to_string(), $value.to_string())),+
//         ];

//         // Convertir en références
//         let params_refs: Vec<(&str, &str)> = params_vec
//             .iter()
//             .map(|(k, v)| (k.as_str(), v.as_str()))
//             .collect();

//         $crate::routing::reverse_with_parameters($name, &params_refs)
//             .unwrap_or_default()
//     }};
// }