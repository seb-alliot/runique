use crate::views::*;

use runique::prelude::*;

pub fn routes() -> Router {
    let limiter = Arc::new(RateLimiter::new().max_requests(5).retry_after(60));

    register_pending("upload_image", "/upload-image");
    let upload_route = Router::new()
        .route("/upload-image", view!(upload_image_submit))
        .route_layer(middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ));

    urlpatterns! {
        "/"                              => view!{ index },                   name = "index",

        // Auth
        "/inscription"                   => view! { soumission_inscription }, name = "inscription",
        "/login"                         => view! { login_user },                  name = "login",
        "/logout"                        => view! { deconnexion },            name = "logout",

        // Profil
        "/profil"                        => view! { profil },                 name = "profil",
        "/view-user"                     => view! { info_user },              name = "search-user",

        // Blog
        "/blog/liste"                    => view! { blog_list },              name = "blog_list",
        "/blog/nouveau"                  => view! { blog_save },              name = "blog_info",
        "/blog/{id}"                     => view! { blog_detail },            name = "blog_detail",

        // Outils & démo
        "/about"                         => view! { about },                  name = "about",
        "/admin-demo"                    => view! { admin_hub },              name = "admin_hub",
        "/admin-demo/declaration"        => view! { admin_declaration },      name = "admin_declaration",
        "/admin-demo/setup"              => view! { admin_setup },            name = "admin_setup",
        "/admin-demo/surcharge"          => view! { admin_surcharge },        name = "admin_surcharge",
        "/admin-demo/surcharge/exemple"  => view! { surcharge_exemple },      name = "admin_surcharge_exemple",
        "/i18n"                          => view! { i18n_demo },              name = "i18n_demo",
        "/session"                       => view! { session_demo },           name = "session_demo",
        "/macros"                        => view! { macros_demo },            name = "macros_demo",
        "/database"                      => view! { database_demo },          name = "database_demo",
        "/installation"                  => view! { installation_demo },      name = "installation_demo",
        "/migrations"                    => view! { migrations_demo },        name = "migrations_demo",
        "/templates"                     => view! { template_demo },          name = "template_demo",
        "/configuration"                 => view! { configuration_demo },     name = "configuration_demo",
        "/orm"                           => view! { orm_demo },               name = "orm_demo",
        "/comparatif"                    => view! { comparatif_demo },        name = "comparatif_demo",
        "/routeur"                       => view! { router_demo },            name = "router_demo",
        "/modeles"                       => view! { model_demo },             name = "model_demo",
        "/roadmap"                       => view! { roadmap },                name = "roadmap",
        "/changelog"                     => view! { changelog },              name = "changelog",
        "/rgpd"                          => view! { rgpd },                   name = "rgpd",
        "/test-csrf"                     => view! { test_csrf },              name = "test_csrf",

        // Formulaires
        "/test-fields"                   => view! { test_fields },            name = "test_fields",
        "/formulaires"                   => view! { formulaires_hub },        name = "formulaires_hub",
        "/formulaires/champs"            => view! { formulaires_champs },     name = "formulaires_champs",
        "/formulaires/rendu"             => view! { formulaires_templates },  name = "formulaires_templates",
        "/formulaires/helpers"           => view! { formulaires_helpers },    name = "formulaires_helpers",
        "/formulaires/helpers/{id}"      => view! { formulaires_helpers },    name = "formulaires_helpers_detail",

        // Middlewares
        "/middleware"                    => view! { middleware_hub },         name = "middleware_hub",
        "/middleware/csrf"               => view! { middleware_csrf },        name = "middleware_csrf",
        "/middleware/csp"                => view! { middleware_csp },         name = "middleware_csp",
        "/middleware/rate-limiter"       => view! { middleware_rate_limit },  name = "middleware_rate_limit",
        "/middleware/login-guard"        => view! { middleware_login_guard }, name = "middleware_login_guard",
        "/middleware/host-validation"    => view! { middleware_hosts },       name = "middleware_hosts",
        "/middleware/https"              => view! { middleware_https },       name = "middleware_https",

        // Erreurs — vérifie que les pages d'erreur Runique s'affichent correctement
        "/erreurs/propose-error"         => view! { propos_template_error },  name = "propos_template_error",
        "/erreurs/404"                   =>   view! { force_not_found },      name = "force_404",
        "/erreurs/500"                   => view! { force_server_error },     name = "force_500",
        "/erreurs/429"                   => view! { force_to_many_requests }, name = "force_429",

        // Contributions
        "/contribution"                  => view! { contribution_submit },    name = "contribution",
        "/contributions"                 => view! { contribution_list },      name = "contribution_list",

    }
    .merge(upload_route)
}
