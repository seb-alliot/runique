use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/" => view!{ views::index }, name = "index",

        // Auth
        "/inscription" => view! { views::soumission_inscription }, name = "inscription",
        "/login"       => view! { views::login },                  name = "login",
        "/logout"      => view! { views::deconnexion },            name = "logout",

        // Profil
        "/profil"      => view! { views::profil },                 name = "profil",
        "/view-user"   => view! { views::info_user },              name = "search-user",

        // Blog
        "/blog/liste"       => view! { views::blog_list },         name = "blog_list",
        "/blog/nouveau"     => view! { views::blog_save },         name = "blog_info",
        "/blog/{id}"         => view! { views::blog_detail },      name = "blog_detail",

        // Outils & démo
        "/about"        => view! { views::about },                 name = "about",
        "/test-csrf"    => view! { views::test_csrf },             name = "test_csrf",
        "/upload-image" => view! { views::upload_image_submit },   name = "upload_image",
        "/test-fields"  => view! { views::test_fields },           name = "test_fields",

        // test template
        "/test-template" => view! { views::test_template },        name = "test_template",

        // Erreurs — vérifie que les pages d'erreur Runique s'affichent correctement
        "/erreurs/404" => view! { views::force_not_found },         name = "force_404",
        "/erreurs/500" => view! { views::force_server_error },      name = "force_500",
        "/erreurs/429" => view! { views::force_too_many_requests }, name = "force_429",

        // Contributions
        "/contribution"  => view! { views::contribution_submit }, name = "contribution",
        "/contributions" => view! { views::contribution_list },   name = "contribution_list",
    }
}
