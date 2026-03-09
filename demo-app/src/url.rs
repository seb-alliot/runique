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
        "/blog/liste"       => view! { views::blog_list },              name = "blog_list",
        "/blog/nouveau"     => view! { views::blog_save },              name = "blog_info",
        "/blog/{id}"         => view! { views::blog_detail },            name = "blog_detail",

        // Outils & démo
        "/about"        => view! { views::about },                 name = "about",
        "/test-csrf"    => view! { views::test_csrf },             name = "test_csrf",
        "/upload-image" => view! { views::upload_image_submit },   name = "upload_image",
        "/test-fields"  => view! { views::test_fields },           name = "test_fields",

    }
}
