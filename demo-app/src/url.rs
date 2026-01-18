use crate::views;
use runique::app_state::AppState;

use runique::{post, urlpatterns, view, Router};

pub fn routes() -> Router<AppState> {
    urlpatterns! {
        // index
        "/" => view!{
            GET => views::index
        },
        name ="index",

        // footer links
        "/about" => view!{
            GET => views::about
        },
        name ="about",

        // New form test
        "/test-new-form" => view! {
            GET => views::test_champs_form,
            POST => views::soumission_champs_form
        },
        name = "test_champs_form",

        // Enregistrement utilisateur
        // // other links
        "/user" => view! {
            GET => views::inscription,
            POST => views::soumissioninscription
        },
        name = "user_profile",


        // Recupération et affichage profil utilisateur
        "/view-user" => view! {
            GET => views::cherche_user,
            POST => views::info_user
        },
        name = "view-user",

        // Ajax CSRF test
        "/test-csrf" => post(views::test_csrf),
        name ="test_csrf",


        // // Form with advanced fields
        "/blog" => view! {
        GET => views::blog_form,
        POST => views::soumission_blog_info
        },
        name ="blog_info",


        "/inscription" => view! {
        GET => views::affiche_form_generer,
        POST => views::soumission_form_generer
        },
        name ="inscription",
    }
}
