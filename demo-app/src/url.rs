use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view};

pub fn routes() -> Router {
    urlpatterns! {
        "/"                              => view!{ views::index },                   name = "index",

        // Auth
        "/inscription"                   => view! { views::soumission_inscription }, name = "inscription",
        "/login"                         => view! { views::login },                  name = "login",
        "/logout"                        => view! { views::deconnexion },            name = "logout",

        // Profil
        "/profil"                        => view! { views::profil },                 name = "profil",
        "/view-user"                     => view! { views::info_user },              name = "search-user",

        // Blog
        "/blog/liste"                    => view! { views::blog_list },              name = "blog_list",
        "/blog/nouveau"                  => view! { views::blog_save },              name = "blog_info",
        "/blog/{id}"                     => view! { views::blog_detail },            name = "blog_detail",

        // Outils & démo
        "/about"                         => view! { views::about },                  name = "about",
        "/admin-demo"                    => view! { views::admin_hub },              name = "admin_hub",
        "/admin-demo/declaration"        => view! { views::admin_declaration },      name = "admin_declaration",
        "/admin-demo/setup"              => view! { views::admin_setup },            name = "admin_setup",
        "/admin-demo/surcharge"          => view! { views::admin_surcharge },        name = "admin_surcharge",
        "/admin-demo/surcharge/exemple"  => view! { views::surcharge_exemple },      name = "admin_surcharge_exemple",
        "/i18n"                          => view! { views::i18n_demo },              name = "i18n_demo",
        "/session"                       => view! { views::session_demo },           name = "session_demo",
        "/macros"                        => view! { views::macros_demo },            name = "macros_demo",
        "/database"                      => view! { views::database_demo },          name = "database_demo",
        "/installation"                  => view! { views::installation_demo },      name = "installation_demo",
        "/migrations"                    => view! { views::migrations_demo },        name = "migrations_demo",
        "/templates"                     => view! { views::template_demo },          name = "template_demo",
        "/configuration"                 => view! { views::configuration_demo },     name = "configuration_demo",
        "/orm"                           => view! { views::orm_demo },               name = "orm_demo",
        "/comparatif"                    => view! { views::comparatif_demo },        name = "comparatif_demo",
        "/routeur"                       => view! { views::router_demo },            name = "router_demo",
        "/modeles"                       => view! { views::model_demo },             name = "model_demo",
        "/roadmap"                       => view! { views::roadmap },                name = "roadmap",
        "/rgpd"                          => view! { views::rgpd },                   name = "rgpd",
        "/test-csrf"                     => view! { views::test_csrf },              name = "test_csrf",
        "/upload-image"                  => view! { views::upload_image_submit },    name = "upload_image",
        "/test-fields"                   => view! { views::test_fields },            name = "test_fields",
        "/formulaires"                   => view! { views::formulaires_hub },        name = "formulaires_hub",
        "/formulaires/champs"            => view! { views::formulaires_champs },     name = "formulaires_champs",
        "/formulaires/rendu"             => view! { views::formulaires_templates },  name = "formulaires_templates",

        // Middlewares
        "/middleware"                    => view! { views::middleware_hub },         name = "middleware_hub",
        "/middleware/csrf"               => view! { views::middleware_csrf },        name = "middleware_csrf",
        "/middleware/csp"                => view! { views::middleware_csp },         name = "middleware_csp",
        "/middleware/rate-limiter"       => view! { views::middleware_rate_limit },  name = "middleware_rate_limit",
        "/middleware/login-guard"        => view! { views::middleware_login_guard }, name = "middleware_login_guard",
        "/middleware/host-validation"    => view! { views::middleware_hosts },       name = "middleware_hosts",
        "/middleware/https"              => view! { views::middleware_https },       name = "middleware_https",

        // Erreurs — vérifie que les pages d'erreur Runique s'affichent correctement
        "/erreurs/propose-error"         => view! { views::propos_template_error },  name = "propos_template_error",
        "/erreurs/404"                   =>   view! { views::force_not_found },      name = "force_404",
        "/erreurs/500"                   => view! { views::force_server_error },     name = "force_500",
        "/erreurs/429"                   => view! { views::force_to_many_requests }, name = "force_429",

        // Contributions
        "/contribution"                  => view! { views::contribution_submit },    name = "contribution",
        "/contributions"                 => view! { views::contribution_list },      name = "contribution_list",
    }
}
